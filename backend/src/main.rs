mod api;
mod auth;
mod db;
mod entities;
mod migration;
mod middleware;
mod handlers;
mod admin;
mod models;
mod traits;
use axum::http::{self,HeaderName, HeaderValue, Method,Request, StatusCode};
use axum::body::Body;
use axum::middleware::{from_fn_with_state, from_fn, Next};
use axum::{
    routing::get,
    extract::State,
    Router,
    Extension,
};
use tower_http::cors::CorsLayer;
use tower::ServiceBuilder;
use crate::sea_orm::{Database, DatabaseConnection, ConnectionTrait};
use sea_orm_migration::prelude::*;
use migration::{Migrator};
use std::net::SocketAddr;
use tower_http::trace::TraceLayer;
use crate::api::create_router;
use crate::admin::setup::create_admin_user_if_not_exists;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use crate::middleware::{
    request_logger::RequestLogger,
    rate_limiter::RateLimiter,
    middleware::auth_middleware
};
use axum::response::{IntoResponse,Response};

async fn log_request_middleware<B>(
    State(state): State<RequestLogger>,
    request: Request<B>,
    next: Next<B>,
) -> Response {
    tracing::debug!("Logging request");
    match state.log_request(&request).await {
        Ok(_) => next.run(request).await,
        Err(status_code) => (status_code, "Error logging request").into_response(),
    }
}

async fn handle_error(error: Box<dyn std::error::Error + Send + Sync>) -> (http::StatusCode, String) {
    tracing::error!("Unhandled error: {:?}", error);
    (StatusCode::INTERNAL_SERVER_ERROR, "Internal Server Error".to_string())
}

fn configure_cors(directory_client: &str, admin_client: &str) -> CorsLayer {
    let allow_origin = vec![
        directory_client.parse::<HeaderValue>().unwrap(),
        admin_client.parse::<HeaderValue>().unwrap(),
    ];

    CorsLayer::new()
        .allow_origin(allow_origin)
        .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE, Method::OPTIONS])
        .allow_headers(vec![
            HeaderName::from_static("content-type"),
            HeaderName::from_static("authorization"),
        ])
        .allow_credentials(true)
}
#[tokio::main]
async fn main() {
    dotenv::dotenv().ok();
    let rust_log = std::env::var("RUST_LOG").unwrap_or_else(|_| "info".to_string());
    // Set up tracing
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(rust_log))
        .with(tracing_subscriber::fmt::layer())
        .init();

    
    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let admin_email = std::env::var("ADMIN_USER").expect("ADMIN_USER must be set");
    let admin_password = std::env::var("ADMIN_PASSWORD").expect("ADMIN_PASSWORD must be set");
    let create_admin = std::env::var("CREATE_ADMIN_ON_STARTUP")
        .unwrap_or_else(|_| "true".to_string())
        .parse::<bool>()
        .unwrap_or(true);
    tracing::info!("Create admin on startup: {}", create_admin);
    tracing::info!("Database URL: {}", database_url);

    // Connect to the database
    let conn = Database::connect(&database_url)
        .await
        .expect("Failed to connect to the database");

    let request_logger = RequestLogger::new(conn.clone());

    // Run migrations
    Migrator::up(&conn, None).await.unwrap();

    tracing::info!("Migrations completed");
    let table_exists = &conn.execute_unprepared("SELECT 1 FROM request_log LIMIT 1").await.is_ok();
    tracing::info!("request_log table exists: {}", table_exists);
    // Create admin user if flag is set
    if create_admin {
        tracing::info!("Verifying Admin");
        println!("Verifying Admin");
        match create_admin_user_if_not_exists(&conn, &admin_email, &admin_password).await {
            Ok(_) => tracing::info!("Admin user setup completed"),
            Err(e) => tracing::error!("Failed to set up admin user: {:?}", e),
        }
    }

    tracing::info!("Successfully connected to the database and ran migrations");

    let directory_client = std::env::var("FRONTEND_URL").unwrap_or_else(|_| "http://localhost:5001".to_string());
    let admin_client = "http://localhost:5150";
    tracing::info!("Directory URL: {}", directory_client);
    tracing::info!("Admin URL: {}", admin_client);

    let cors = configure_cors(&directory_client, admin_client);

    let rate_limiter = RateLimiter::new();

    let app = Router::new()
        .merge(create_router(conn.clone()))
        .layer(cors)
        .layer(
            ServiceBuilder::new()
                .layer(TraceLayer::new_for_http())
                .layer(from_fn_with_state(request_logger, log_request_middleware))
                .into_inner()
        )
        .layer(Extension(conn.clone()))
        .layer(Extension(rate_limiter));

    let addr = SocketAddr::from(([0, 0, 0, 0], 8000));
    tracing::info!("Listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}
