use axum::{Router, Extension, routing::post, routing::get};
use sea_orm::DatabaseConnection;
use crate::handlers::{users, admin, profiles, listings, accounts, user_accounts, ad_purchases, directories, sessions};
use crate::middleware::{auth_middleware};
use crate::admin::routes::admin_routes;
use tower_http::trace::TraceLayer;
use crate::middleware::rate_limiter::RateLimiter;

pub fn create_router(db: DatabaseConnection) -> Router {
    // Auth routes (no /api prefix, no state needed here)
    let auth_routes = Router::new()
        .route("/login", post(users::login_user))
        .route("/register", post(users::register_user))
        .route("/validate-session", get(sessions::validate_session));

    // Public routes (requires state, so merge with state-applied routers)
    let public_routes = Router::<DatabaseConnection>::new()
        .merge(admin::public_routes(db.clone()))
        .merge(directories::public_routes(db.clone())) // Already has .with_state(db) inside
        .merge(listings::public_routes(db.clone()));

    let rate_limiter = RateLimiter::new();
    let db_clone = db.clone();

    // Authenticated routes (requires state)
    let authenticated_routes = Router::new()
        .route("/logout", post(users::logout_user))
        .merge(profiles::routes(db.clone()))
        .merge(listings::authenticated_routes())
        .merge(accounts::routes())
        .merge(user_accounts::routes())
        .merge(ad_purchases::routes())
        .merge(admin_routes(db.clone()))
        .merge(users::authenticated_routes(db.clone()));

    // Combine all routes and apply state at the top level
    Router::new()
        .merge(auth_routes)
        .merge(public_routes)
        .nest(
            "/api",
            authenticated_routes
                .layer(Extension(rate_limiter.clone()))
                .layer(axum::middleware::from_fn(move |req, next| {
                    let db = db_clone.clone();
                    let rate_limiter = rate_limiter.clone();
                    async move {
                        auth_middleware(db, rate_limiter, req, next).await
                    }
                })),
        )
        .layer(Extension(db.clone())) // For middleware that might need it
        .layer(TraceLayer::new_for_http())
        .with_state(db) // Apply state to the entire router
}