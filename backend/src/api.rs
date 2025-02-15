use axum::{Router, Extension, routing::post, routing::get};
use sea_orm::DatabaseConnection;
use crate::handlers::{users,admin, profiles, listings, accounts, user_accounts, ad_purchases, directories,sessions};
use crate::middleware::{auth_middleware};
use crate::admin::routes::admin_routes;
use tower_http::trace::TraceLayer;

pub fn create_router(db: DatabaseConnection) -> Router {
    // Auth routes (no /api prefix)
    let auth_routes = Router::new()
        .route("/login", post(users::login_user))
        .route("/register", post(users::register_user))
        .route("/validate-session", get(sessions::validate_session));

    // Public routes
    let public_routes = Router::new()
        .merge(admin::public_routes())
        .merge(directories::public_routes())
        .merge(listings::public_routes())
        .with_state(());

    // Authenticated routes
    let authenticated_routes = Router::new()
        .route("/logout", post(users::logout_user))  // Add logout route here
        .merge(profiles::routes(db.clone()))
        .merge(listings::authenticated_routes())
        .merge(accounts::routes())
        .merge(user_accounts::routes())
        .merge(ad_purchases::routes())
        .merge(admin_routes(db.clone()))
        .merge(users::authenticated_routes(db.clone()))
        .with_state(db.clone());

    // Combine all routes
    Router::new()
        .merge(auth_routes)  // Non-prefixed auth routes
        .merge(public_routes)  // Public routes
        .nest("/api", 
            authenticated_routes
                .layer(axum::middleware::from_fn_with_state(
                    db.clone(),
                    auth_middleware
                ))
        )
        .layer(Extension(db.clone()))
        .layer(TraceLayer::new_for_http())
        .with_state(db)
}