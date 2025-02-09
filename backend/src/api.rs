use axum::{Router, Extension, routing::post};
use sea_orm::DatabaseConnection;
use crate::handlers::{users,admin, profiles, listings, accounts, user_accounts, ad_purchases, directories};
use crate::middleware::{auth_middleware};
use crate::admin::routes::admin_routes;
use tower_http::trace::TraceLayer;

pub fn create_router(db: DatabaseConnection) -> Router {
    // Auth routes (login and register)
    //let auth_routes = users::auth_routes();
    tracing::info!("Auth routes set up");
    //add / health to public routes

    let public_routes = Router::new()
        .merge(admin::public_routes())
        .merge(directories::public_routes())
        .merge(listings::public_routes())
        .with_state(());

    // Authenticated routes (including admin routes)
    let authenticated_routes = Router::new()
        .merge(profiles::routes(db.clone()))
        .merge(listings::authenticated_routes())
        .merge(accounts::routes())
        .merge(user_accounts::routes())
        .merge(ad_purchases::routes())
        .merge(admin_routes(db.clone()))
        .merge(users::authenticated_routes(db.clone()))
        .with_state(());

    // Combine all routes
    Router::new()
        .merge(users::auth_routes())  // Remove the parentheses
        .merge(public_routes)  // This will be accessible without authentication
        .nest("/api", 
            authenticated_routes
                .layer(axum::middleware::from_fn_with_state(db.clone(), auth_middleware))
        )  
        .layer(Extension(db.clone()))
        .layer(
            TraceLayer::new_for_http()
        )
        .with_state(db)  // Add this line to set the state
}