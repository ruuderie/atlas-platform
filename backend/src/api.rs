use axum::{Router, Extension, routing::post, routing::get};
use sea_orm::DatabaseConnection;
use crate::handlers::{users, admin, profiles, listings, accounts, user_accounts, ad_purchases, directories, sessions, listing_attributes, health};
use crate::middleware::auth_middleware;
use crate::admin::routes::admin_routes;
use tower_http::trace::TraceLayer;
use crate::middleware::rate_limiter::RateLimiter;
use tower_http::cors::{CorsLayer, Any};
use std::env;

pub fn create_router(db: DatabaseConnection) -> Router {
    // Check environment
    let is_production = env::var("ENVIRONMENT").unwrap_or_else(|_| "development".to_string()) == "production";
    tracing::info!("Environment: {}", if is_production { "production" } else { "development" });
    
    // Configure CORS based on environment
    let cors_layer = if is_production {
        // In production, only allow specific origins
        tracing::warn!("Running in PRODUCTION mode - CORS is restricted to specific origins");
        let frontend_url = env::var("FRONTEND_URL").expect("FRONTEND_URL must be set in production");
        let admin_url = env::var("ADMIN_URL").expect("ADMIN_URL must be set in production");
        
        // Allow additional origins from environment variable if specified
        let mut allowed_origins = vec![frontend_url.parse().unwrap(), admin_url.parse().unwrap()];
        
        // Optional: Allow additional origins from comma-separated env var
        if let Ok(additional_origins) = env::var("ADDITIONAL_ALLOWED_ORIGINS") {
            for origin in additional_origins.split(',') {
                if let Ok(origin) = origin.trim().parse() {
                    tracing::info!("Adding additional allowed origin: {:?}", origin);
                    allowed_origins.push(origin);
                }
            }
        }
        
        tracing::info!("Configured allowed origins: {:?}", allowed_origins);
        
        CorsLayer::new()
            .allow_origin(allowed_origins)
            .allow_methods(Any)
            .allow_headers(Any)
            .allow_credentials(true)
    } else {
        // In development, allow all origins
        tracing::info!("Running in DEVELOPMENT mode - CORS is permissive");
        CorsLayer::permissive()
    };

    // Auth routes with CORS headers - these should remain outside the /api prefix
    let auth_routes = Router::new()
        .route("/login", post(users::login_user))
        .route("/register", post(users::register_user))
        .route("/validate-session", get(sessions::validate_session))
        .route("/refresh-token", post(sessions::refresh_token))
        .layer(cors_layer.clone());

    // Public routes (requires state, so merge with state-applied routers)
    let public_routes = Router::<DatabaseConnection>::new()
        .merge(directories::public_routes(db.clone()))
        .merge(listings::public_routes(db.clone()))
        .route("/health", get(health::health_check))
        .layer(Extension(db.clone()))
        .layer(cors_layer.clone());  // Add CORS to public routes

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
        .merge(admin_routes(db.clone()))
        .merge(users::authenticated_routes(db.clone()))
        .merge(directories::authenticated_routes(db.clone()));

    // Combine all routes and apply state at the top level
    Router::new()
        .merge(auth_routes)  // Keep auth routes at the root level
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
        .layer(cors_layer)  // Add CORS at the top level with environment-specific settings
        .with_state(db) // Apply state to the entire router
}