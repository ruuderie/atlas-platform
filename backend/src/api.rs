/* 
 * TODO(next-developer): MIGRATION TO AtlasApp API TRAIT REQUIRED
 * 
 * This legacy application router mapping currently has its routes hardcoded 
 * into the global Atlas platform core. 
 * 
 * We have introduced a strict, standardized Rust API trait: `AtlasApp` 
 * located at `backend/src/traits/atlas_app.rs`. 
 * 
 * Future work requires refactoring the sub-apps to implement the `AtlasApp` trait 
 * (providing perfect encapsulation for its Axum Router, SeaORM Migrations, and Background Jobs) 
 * instead of manually merging them into `backend/src/api.rs`.
 * 
 * See the full integration protocol at: `docs/atlas_app_integration.md`
 */
use axum::{Router, Extension, routing::post, routing::get};
use sea_orm::DatabaseConnection;
use crate::handlers::{users, profiles, listings, accounts, my_accounts, ab_testing, user_accounts, ad_purchases, tenant, app_instance, app_pages, app_menus, sessions, health, auth_frontend, communications, setup, magic_links, search};
use crate::middleware::{auth_middleware, site_context_middleware};
use crate::admin::routes::admin_routes;
use tower_http::trace::TraceLayer;
use crate::middleware::rate_limiter::RateLimiter;
use axum::{extract::Request, middleware::Next};
use std::env;

// async fn auth_middleware_wrapper(
//     Extension(db): Extension<DatabaseConnection>,
//     Extension(rate_limiter): Extension<RateLimiter>,
//     req: Request<axum::body::Body>,
//     next: Next,
// ) -> axum::response::Response {
//     auth_middleware(Extension(db), Extension(rate_limiter), req, next).await.unwrap_or_else(|err| {
//         axum::http::Response::builder()
//             .status(err)
//             .body(axum::body::Body::empty())
//             .unwrap()
//     })
// }

pub fn create_router(db: DatabaseConnection) -> Router {
    // Check environment
    let is_production = env::var("ENVIRONMENT").unwrap_or_else(|_| "development".to_string()) == "production";
    tracing::info!("Environment: {}", if is_production { "production" } else { "development" });
    
    // Note: CORS is now solely managed at the top-level Router in main.rs
    // Auth routes with CORS headers - these should remain outside the /api prefix
    let auth_routes = Router::new()
        .route("/login", post(users::login_user))
        .route("/register", post(users::register_user))
        .route("/validate-session", get(sessions::validate_session))
        .route("/refresh-token", post(sessions::refresh_token))
        .layer(Extension(db.clone()))
        .layer(axum::middleware::from_fn(site_context_middleware));

    // Public routes (requires state, so merge with state-applied routers)
    let mut public_routes = Router::<DatabaseConnection>::new()
        .merge(tenant::public_routes(db.clone()))
        .merge(crate::handlers::feeds::public_routes(db.clone()))
        .merge(auth_frontend::public_routes())
        .merge(ab_testing::public_routes())
        .merge(crate::handlers::passkeys::public_routes())
        .merge(setup::public_routes())
        .merge(magic_links::public_routes())
        .merge(app_instance::public_routes(db.clone()))
        .route("/health", get(health::health_check));

    for app in crate::atlas_apps::get_active_apps() {
        public_routes = public_routes.merge(app.public_router(db.clone()));
    }

    let public_routes = public_routes
        .layer(Extension(db.clone()))
        .layer(axum::middleware::from_fn(site_context_middleware));

    let rate_limiter = RateLimiter::new();
    let db_clone = db.clone();

    // Authenticated routes (requires state)
    let mut authenticated_routes = Router::new()
        .route("/logout", post(users::logout_user))
        .merge(accounts::routes())
        .merge(user_accounts::routes())
        .merge(admin_routes(db.clone()))
        .merge(users::authenticated_routes(db.clone()))
        .merge(auth_frontend::authenticated_routes())
        .merge(my_accounts::authenticated_routes())
        .merge(ab_testing::authenticated_routes())
        .merge(crate::handlers::passkeys::authenticated_routes())
        .merge(crate::handlers::feeds::authenticated_routes(db.clone()))
        .merge(tenant::authenticated_routes(db.clone()))
        .merge(app_instance::authenticated_routes(db.clone()))
        .merge(communications::authenticated_routes(db.clone()))
        .merge(search::authenticated_routes())
        .merge(crate::handlers::audit_logs::authenticated_routes())
        .merge(crate::handlers::telemetry::authenticated_routes());

    for app in crate::atlas_apps::get_active_apps() {
        authenticated_routes = authenticated_routes.merge(app.authenticated_router(db.clone()));
    }

    // Combine all routes and apply state at the top level
    Router::new()
        .merge(auth_routes)  // Keep auth routes at the root level
        .merge(public_routes)
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
        .layer(Extension(db.clone())) // For middleware that might need it
        .layer(TraceLayer::new_for_http())
        .with_state(db) // Apply state to the entire router
}