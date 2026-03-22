use axum::{Router, Extension, routing::post, routing::get};
use sea_orm::DatabaseConnection;
use crate::handlers::{users, admin, profiles, listings, accounts, my_accounts, ab_testing, user_accounts, ad_purchases, directories, sessions, listing_attributes, health, auth_frontend};
use crate::middleware::{auth_middleware, site_context_middleware};
use crate::admin::routes::admin_routes;
use tower_http::trace::TraceLayer;
use crate::middleware::rate_limiter::RateLimiter;
use tower_http::cors::{CorsLayer, Any};
use axum::{extract::Request, middleware::Next, response::IntoResponse, http::{StatusCode, Response}};
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
    let public_routes = Router::<DatabaseConnection>::new()
        .merge(directories::public_routes(db.clone()))
        .merge(listings::public_routes(db.clone()))
        .merge(crate::handlers::leads::public_routes())
        .merge(crate::handlers::feeds::public_routes(db.clone()))
        .merge(auth_frontend::public_routes())
        .merge(ab_testing::public_routes())
        .merge(crate::handlers::passkeys::public_routes())
        .route("/health", get(health::health_check))
        .layer(Extension(db.clone()))
        .layer(axum::middleware::from_fn(site_context_middleware));

    let rate_limiter = RateLimiter::new();
    let db_clone = db.clone();

    // Authenticated routes (requires state)
    let authenticated_routes = Router::new()
        .route("/logout", post(users::logout_user))
        .merge(profiles::routes(db.clone()))
        .merge(listings::authenticated_routes())
        .merge(listing_attributes::routes())
        .merge(accounts::routes())
        .merge(user_accounts::routes())
        .merge(ad_purchases::routes())
        .merge(crate::handlers::leads::authenticated_routes())
        .merge(admin_routes(db.clone()))
        .merge(users::authenticated_routes(db.clone()))
        .merge(auth_frontend::authenticated_routes())
        .merge(my_accounts::authenticated_routes())
        .merge(ab_testing::authenticated_routes())
        .merge(crate::handlers::passkeys::authenticated_routes())
        .merge(crate::handlers::feeds::authenticated_routes(db.clone()))
        .merge(directories::authenticated_routes(db.clone()));

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