/*
 * DONE — AtlasApp migration complete as of 2026-05-02.
 *
 * This file now contains ONLY Tier 3 platform infrastructure:
 *   Auth (login/register/session/refresh/logout), Passkeys, Magic Links, OAuth,
 *   Accounts, User Accounts, Admin Panel, A/B Testing, Setup (first-run), /health.
 *
 * All Tier 1 routes (CMS, tenant, onboarding, forms, feeds, search, audit logs,
 * seeds, app_instance) are now owned by `CorePlatformApp` in:
 *   `backend/src/atlas_apps/core_platform.rs`
 *
 * Domain sub-app routes (listings, CRM, profiles, anchor pages) remain in their
 * respective AtlasApp implementations (AnchorApp, NetworkInstanceApp).
 *
 * See the full integration protocol at: `docs/atlas_app_integration.md`
 * Architecture layer map: `docs/architecture/platform_layer_map.md`
 */
use crate::handlers::version::version_header_middleware;
use crate::handlers::{
    ab_testing, accounts, auth_frontend, health, my_accounts, sessions, setup, user_accounts, users,
};
use crate::middleware::rate_limiter::RateLimiter;
use crate::middleware::{auth_middleware, site_context_middleware};
use axum::{Extension, Router, routing::delete, routing::get, routing::post};
use axum::{extract::Request, middleware::Next};
use sea_orm::DatabaseConnection;
use std::env;
use tower_http::trace::TraceLayer;

pub fn create_router(db: DatabaseConnection) -> Router {
    // Check environment
    let is_production =
        env::var("ENVIRONMENT").unwrap_or_else(|_| "development".to_string()) == "production";
    tracing::info!(
        "Environment: {}",
        if is_production {
            "production"
        } else {
            "development"
        }
    );

    // Note: CORS is now solely managed at the top-level Router in main.rs
    // Auth routes with CORS headers - these should remain outside the /api prefix
    let auth_routes = Router::new()
        .route("/login", post(users::login_user))
        .route("/register", post(users::register_user))
        // Legacy — kept for backward compat with platform-admin during migration
        .route("/validate-session", get(sessions::validate_session))
        .route("/refresh-token", post(sessions::refresh_token))
        // Unified Atlas Auth Protocol endpoints (all apps should use these)
        .route(
            "/api/auth/session/validate",
            get(sessions::validate_session),
        )
        .route("/api/auth/session/revoke", post(sessions::revoke_session))
        .route(
            "/api/auth/impersonate/exchange",
            post(sessions::exchange_impersonate_code),
        )
        // Session management — list and targeted revoke
        .route(
            "/api/me/sessions",
            get(sessions::list_user_sessions).delete(sessions::revoke_all_other_sessions),
        )
        .route(
            "/api/me/sessions/{session_id}",
            delete(sessions::revoke_other_session),
        )
        .layer(Extension(db.clone()))
        .layer(axum::middleware::from_fn(site_context_middleware));

    // Public routes — Tier 3 infrastructure only.
    // Tier 1 routes (CMS, tenant, onboarding, forms, feeds, search, audit_logs,
    // app_instance, app_seeds) are registered via CorePlatformApp in get_active_apps().
    // Tier 2 routes (listings, CRM, profiles, anchor pages) are registered via
    // AnchorApp and NetworkInstanceApp in get_active_apps().
    let mut public_routes = Router::<DatabaseConnection>::new()
        .merge(auth_frontend::public_routes())
        .merge(crate::handlers::otp::public_routes()) // inline OTP for wizard pre-step
        .merge(ab_testing::public_routes())
        .merge(crate::handlers::passkeys::public_routes())
        .merge(setup::public_routes())
        .merge(crate::handlers::version::public_routes()) // GET /api/version
        .route("/health", get(health::health_check))
        // ── Product Launch Engine — zero-auth, CDN-cacheable ─────────────────
        // Public product pages, variant pages, waitlist, pre-order, sitemap
        .merge(crate::handlers::pub_products::public_routes_raw())
        // Landing page funnel events — fire-and-forget from public Folio pages
        .route(
            "/api/pub/lp-events",
            post(crate::handlers::landing_pages::post_lp_event),
        )
        // Domain resolver: folio.app / miami.folio.app → product/variant context
        .route(
            "/api/pub/resolve",
            get(crate::handlers::pub_resolve::resolve_domain),
        );

    for app in crate::atlas_apps::get_active_apps() {
        public_routes = public_routes.merge(app.public_router(db.clone()));
    }

    let public_routes = public_routes
        .layer(Extension(db.clone()))
        .layer(axum::middleware::from_fn(site_context_middleware));

    let rate_limiter = RateLimiter::new();
    let db_clone = db.clone();

    // Authenticated routes — Tier 3 infrastructure only.
    // Tier 1 and Tier 2 authenticated routes are injected via the get_active_apps() loop below.
    // /api/admin/* routes are injected via PlatformAdminApp in get_active_apps() — no longer hardcoded here.
    let mut authenticated_routes = Router::new()
        .route("/logout", post(users::logout_user))
        .merge(accounts::routes())
        .merge(user_accounts::routes())
        .merge(users::authenticated_routes(db.clone()))
        .merge(auth_frontend::authenticated_routes())
        .merge(my_accounts::authenticated_routes())
        .merge(ab_testing::authenticated_routes())
        .merge(crate::handlers::passkeys::authenticated_routes())
        .merge(crate::handlers::communications::authenticated_routes(
            db.clone(),
        ))
        .merge(crate::handlers::crm_status_options::authenticated_routes())
        .merge(crate::handlers::telemetry::authenticated_routes())
        .merge(crate::handlers::notes::routes())
        .merge(crate::handlers::activities::routes())
        .merge(crate::handlers::scorecard_entries::routes())
        .merge(crate::handlers::scorecard_display_rules::routes())
        .merge(crate::handlers::scorecard_analytics::routes()) // Phase 3 — portfolio analytics
        .merge(crate::handlers::ws::authenticated_routes_raw()) // G07 — WebSocket relay
        .merge(crate::handlers::rbac::authenticated_routes_raw()); // G-32 — RBAC management

    for app in crate::atlas_apps::get_active_apps() {
        authenticated_routes = authenticated_routes.merge(app.authenticated_router(db.clone()));
    }

    // Combine all routes and apply state at the top level.
    // The version_header_middleware wraps the entire router so EVERY response
    // (including error responses from the auth middleware) carries X-Atlas-Version.
    Router::new()
        .merge(auth_routes) // Keep auth routes at the root level
        .merge(public_routes.layer(Extension(rate_limiter.clone())))
        .merge(
            authenticated_routes
                .layer(axum::middleware::from_fn(
                    |Extension(db): Extension<DatabaseConnection>,
                     Extension(rate_limiter): Extension<RateLimiter>,
                     req: Request<axum::body::Body>,
                     next: Next| async move {
                        auth_middleware(Extension(db), Extension(rate_limiter), req, next)
                            .await
                            .unwrap_or_else(|err| {
                                axum::http::Response::builder()
                                    .status(err)
                                    .body(axum::body::Body::empty())
                                    .unwrap()
                            })
                    },
                ))
                .layer(axum::middleware::from_fn(site_context_middleware))
                .layer(Extension(db_clone))
                .layer(Extension(rate_limiter)),
        )
        .layer(axum::middleware::from_fn(version_header_middleware)) // X-Atlas-Version on every response
        .layer(Extension(db.clone()))
        .layer(TraceLayer::new_for_http())
        .with_state(db)
}
