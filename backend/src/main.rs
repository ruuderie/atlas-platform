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
use axum::http::{self,HeaderName, HeaderValue, Method,Request, StatusCode, header};
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
use tower_http::trace::{TraceLayer, DefaultMakeSpan, DefaultOnResponse};
use tracing::Level;
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
    let origins = vec![
        directory_client.parse::<HeaderValue>().unwrap_or_else(|_| "http://frontend:5001".parse().unwrap()),
        admin_client.parse::<HeaderValue>().unwrap_or_else(|_| "http://admin:5150".parse().unwrap()),
        "http://localhost:5150".parse::<HeaderValue>().unwrap(),
        "http://127.0.0.1:5150".parse::<HeaderValue>().unwrap(),
        "http://localhost:8001".parse::<HeaderValue>().unwrap(),
    ];

    CorsLayer::new()
        .allow_origin(origins)
        .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE, Method::OPTIONS])
        .allow_headers([
            header::AUTHORIZATION,
            header::ACCEPT,
            header::CONTENT_TYPE,
            header::ORIGIN,
            header::ACCESS_CONTROL_ALLOW_ORIGIN,
            header::ACCESS_CONTROL_ALLOW_CREDENTIALS,
            header::ACCESS_CONTROL_ALLOW_HEADERS,
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

    // Determine database URL based on environment
    let database_url = if std::env::var("USE_LOCAL_DB").unwrap_or_else(|_| "false".to_string()) == "true" {
        std::env::var("LOCAL_DATABASE_URL").unwrap_or_else(|_| {
            // Fallback to a default local connection string
            "postgresql://postgres:postgres@localhost:5432/oplydb".to_string()
        })
    } else {
        std::env::var("DATABASE_URL").expect("DATABASE_URL must be set")
    };
    
    let admin_email = std::env::var("ADMIN_USER").expect("ADMIN_USER must be set");
    let admin_password = std::env::var("ADMIN_PASSWORD").expect("ADMIN_PASSWORD must be set");
    let create_admin = std::env::var("CREATE_ADMIN_ON_STARTUP")
        .unwrap_or_else(|_| "true".to_string())
        .parse::<bool>()
        .unwrap_or(true);
    tracing::info!("Create admin on startup: {}", create_admin);
    tracing::info!("Using database URL: {}", database_url.replace(|c: char| !c.is_ascii_alphabetic() && !c.is_ascii_punctuation(), "*"));

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
    let admin_client = std::env::var("ADMIN_URL").unwrap_or_else(|_| "http://localhost:5002".to_string());
    tracing::info!("Directory URL: {}", directory_client);
    tracing::info!("Admin URL: {}", admin_client);

    let cors = configure_cors(&directory_client, &admin_client);

    let rate_limiter = RateLimiter::new();

    let app = Router::new()
        .merge(create_router(conn.clone()))
        .layer(cors)
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(DefaultMakeSpan::new().level(Level::INFO))
                .on_request(|request: &Request<_>, _span: &tracing::Span| {
                    // Log every single request that hits the server
                    tracing::info!(
                        "RAW REQUEST: method={}, uri={}, headers={:?}",
                        request.method(),
                        request.uri(),
                        request.headers()
                    );
                })
                .on_response(DefaultOnResponse::new().level(Level::INFO))
        )
        .layer(
            ServiceBuilder::new()
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
