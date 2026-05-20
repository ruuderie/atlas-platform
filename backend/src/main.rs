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
mod config;
mod services;
mod webauthn_registry;
pub mod atlas_apps;
pub mod metrics;

use axum::http::{self, HeaderMap, HeaderValue, Method, Request, StatusCode, header};
use axum::body::Body;
use axum::middleware::{from_fn, from_fn_with_state};
use axum::{
    Router,
    Extension,
    routing::get,
};
use tower_http::cors::CorsLayer;
use tower::ServiceBuilder;
use crate::sea_orm::{Database, ConnectionTrait};
use sea_orm_migration::prelude::*;
use migration::{Migrator};
use std::net::SocketAddr;
use tower_http::trace::{TraceLayer, DefaultMakeSpan, DefaultOnResponse};
use tracing::Level;
use tokio::net::TcpListener;
use crate::api::create_router;

use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use crate::middleware::{
    request_logger::RequestLogger,
    rate_limiter::RateLimiter,
    middleware::log_request_middleware,
    request_id::request_id_middleware,
    DynamicCorsRegistry,
    dynamic_cors_layer,
};
use webauthn_rs::prelude::*;
use crate::handlers::passkeys::{WebauthnStateRaw, WebauthnState};
use crate::webauthn_registry::WebauthnRegistry;
use moka::future::Cache;
use std::sync::Arc;
use std::time::Duration;

async fn handle_error(error: Box<dyn std::error::Error + Send + Sync>) -> (http::StatusCode, String) {
    tracing::error!("Unhandled error: {:?}", error);
    (StatusCode::INTERNAL_SERVER_ERROR, "Internal Server Error".to_string())
}

fn configure_cors(network_client: &str, admin_client: &str) -> CorsLayer {
    let is_dev = std::env::var("ENVIRONMENT").unwrap_or_else(|_| "production".to_string()) == "development";

    let allow_origin = if is_dev {
        // [WARNING]: DANGEROUS IN PRODUCTION
        // This dynamically echoes ANY origin, useful only for local multi-tenant Orbstack/Docker networking.
        tower_http::cors::AllowOrigin::predicate(|_, _| true)
    } else {
        let mut origins = vec![
            network_client.parse::<HeaderValue>().unwrap_or_else(|_| "http://frontend:5001".parse().unwrap()),
            admin_client.parse::<HeaderValue>().unwrap_or_else(|_| "http://admin:5150".parse().unwrap()),
        ];

        if let Ok(additional_origins) = std::env::var("ADDITIONAL_ALLOWED_ORIGINS") {
            for origin in additional_origins.split(',') {
                if let Ok(parsed) = origin.trim().parse::<HeaderValue>() {
                    origins.push(parsed);
                }
            }
        }
        
        tower_http::cors::AllowOrigin::list(origins)
    };

    CorsLayer::new()
        .allow_origin(allow_origin)
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

/// Prometheus metrics scrape endpoint.
///
/// Protected by a static bearer token (`METRICS_TOKEN` env var).
/// Prometheus scrape config must include:
///   `bearer_token: <same token>`
///
/// Without this, internal counters (magic link rates, session latency) are
/// publicly accessible to any external actor.
async fn metrics_endpoint(headers: HeaderMap) -> Result<String, StatusCode> {
    let expected = std::env::var("METRICS_TOKEN").unwrap_or_default();
    // If METRICS_TOKEN is unset, deny all — prevents misconfigured pods from leaking data.
    if expected.is_empty() {
        tracing::warn!("METRICS_TOKEN is not set — /metrics access denied");
        return Err(StatusCode::UNAUTHORIZED);
    }
    let provided = headers
        .get(header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .and_then(|s| s.strip_prefix("Bearer "))
        .unwrap_or("");
    if provided != expected {
        return Err(StatusCode::UNAUTHORIZED);
    }
    Ok(crate::metrics::metrics_handler())
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

    // Register Prometheus metrics
    crate::metrics::register_metrics();

    // Determine database URL based on environment
    let database_url = if std::env::var("USE_LOCAL_DB").unwrap_or_else(|_| "false".to_string()) == "true" {
        std::env::var("LOCAL_DATABASE_URL").unwrap_or_else(|_| {
            // Fallback to a default local connection string
            "postgresql://postgres:postgres@localhost:5432/oplydb".to_string()
        })
    } else {
        std::env::var("DATABASE_URL").expect("DATABASE_URL must be set")
    };
    
    let _admin_email = std::env::var("ADMIN_USER").expect("ADMIN_USER must be set");
    let _admin_password = std::env::var("ADMIN_PASSWORD").expect("ADMIN_PASSWORD must be set");
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

    let _request_logger = RequestLogger::new(conn.clone());

    if std::env::var("WIPE_DB_ON_STARTUP").unwrap_or_else(|_| "false".to_string()) == "true" {
        tracing::warn!("WIPE_DB_ON_STARTUP is enabled! Wiping the database to start from scratch...");
        use sea_orm::{ConnectionTrait, Statement};
        conn.execute(Statement::from_string(
            sea_orm::DatabaseBackend::Postgres,
            "DROP SCHEMA public CASCADE; CREATE SCHEMA public;".to_owned(),
        )).await.expect("Failed to recreate public schema");
    }

    // Fix renamed migrations in the database before running Migrator
    tracing::info!("Ensuring legacy migration names are updated in seaql_migrations...");
    use sea_orm::{ConnectionTrait, Statement};
    let _ = conn.execute(Statement::from_string(
        sea_orm::DatabaseBackend::Postgres,
        "
        UPDATE seaql_migrations SET version = 'm20230912_000000_create_users_table' WHERE version = 'm20230912_create_users_table';
        UPDATE seaql_migrations SET version = 'm20230912_000001_create_user_accounts_table' WHERE version = 'm20230912_create_user_accounts_table';
        ".to_owned()
    )).await;

    // Run migrations
    Migrator::up(&conn, None).await.unwrap();

    tracing::info!("Migrations completed");
    let table_exists = &conn.execute_unprepared("SELECT 1 FROM request_log LIMIT 1").await.is_ok();
    tracing::info!("request_log table exists: {}", table_exists);

    if create_admin {
        let admin_email = std::env::var("ADMIN_USER").unwrap_or_else(|_| "admin@oply.co".to_string());
        if let Err(e) = crate::admin::setup::create_admin_user_if_not_exists(&conn, &admin_email, &_admin_password).await {
            tracing::error!("Failed to create admin user: {}", e);
        } else {
            tracing::info!("Successfully verified root administrative account for {}", admin_email);
        }
    }

    tracing::info!("Successfully connected to the database and ran migrations");



    let sync_db = conn.clone();
    crate::services::data_sync::DataSyncService::start_worker(sync_db).await;
    let telemetry_db = conn.clone();
    tokio::spawn(async move {
        // Run every hour
        let mut interval = tokio::time::interval(Duration::from_secs(3600));
        loop {
            interval.tick().await;
            if let Err(e) = crate::services::telemetry::TelemetryService::process_daily_metrics(&telemetry_db).await {
                tracing::error!("Background telemetry processing failed: {}", e);
            }
        }
    });

    let webhook_db = conn.clone();
    crate::services::webhook::start_webhook_sweeper(webhook_db).await;

    let network_client = std::env::var("FRONTEND_URL").unwrap_or_else(|_| "http://localhost:5001".to_string());
    let admin_client = std::env::var("ADMIN_URL").unwrap_or_else(|_| "http://localhost:5002".to_string());
    tracing::info!("Network URL: {}", network_client);
    tracing::info!("Admin URL: {}", admin_client);

    let cors_registry = Arc::new(DynamicCorsRegistry::new(conn.clone()));
    cors_registry.hydrate(&[network_client.clone(), admin_client.clone()]).await;
    let cors = dynamic_cors_layer(cors_registry.clone());

    let rate_limiter = RateLimiter::new();

    let rp_origin_str = std::env::var("WEBAUTHN_ORIGIN").unwrap_or_else(|_| admin_client.clone());
    let rp_origin = url::Url::parse(&rp_origin_str)
        .unwrap_or_else(|_| url::Url::parse("https://platform-admin.atlas-platform.orb.local").unwrap());
    
    let rp_id = std::env::var("RP_ID").unwrap_or_else(|_| {
        rp_origin.host_str().unwrap_or("localhost").to_string()
    });
    
    let registry = Arc::new(WebauthnRegistry::new(conn.clone(), 10_000));
    
    // Seed primary platform origin
    if let Err(e) = registry.seed(&rp_id, &rp_origin).await {
        tracing::error!("Failed to seed WebauthnRegistry: {}", e);
    }

    // DB-driven startup warm: seed the registry for every tenant domain already
    // registered in app_domain. This ensures dynamically provisioned tenant origins
    // use the correct eTLD+1 rpId immediately, without waiting for a live request to
    // hit get_or_create. New tenants provisioned post-startup are still covered by
    // get_or_create via the DB-verify path — no pod restart needed.
    {
        use crate::entities::app_domain;
        use crate::webauthn_registry::effective_tld_plus_one;
        use sea_orm::EntityTrait;
        match app_domain::Entity::find().all(&conn).await {
            Ok(domains) => {
                tracing::info!("Pre-warming WebAuthn registry for {} tenant domain(s)...", domains.len());
                for domain in domains {
                    let origin_str = format!("https://{}", domain.domain_name);
                    match url::Url::parse(&origin_str) {
                        Ok(origin_url) => {
                            let domain_rp_id = effective_tld_plus_one(
                                origin_url.host_str().unwrap_or(&domain.domain_name)
                            );
                            if let Err(e) = registry.seed(&domain_rp_id, &origin_url).await {
                                tracing::warn!(
                                    "Could not pre-warm WebAuthn for domain '{}': {}",
                                    domain.domain_name, e
                                );
                            }
                        }
                        Err(e) => {
                            tracing::warn!(
                                "Skipping WebAuthn pre-warm for invalid domain '{}': {}",
                                domain.domain_name, e
                            );
                        }
                    }
                }
            }
            Err(e) => {
                tracing::warn!("Could not load app_domain records for WebAuthn pre-warm: {}", e);
            }
        }
    }
    
    // Additional origins (e.g. tenant custom domains listed in ADDITIONAL_ALLOWED_ORIGINS).
    // Must use eTLD+1 as rp_id — the same rule applied in get_or_create() and the tenant
    // pre-warm loop above. Using the full host (e.g. "dev.buildwithruud.com") as rp_id
    // would cause browsers to reject challenges for passkeys registered under "buildwithruud.com".
    if let Ok(additional_origins) = std::env::var("ADDITIONAL_ALLOWED_ORIGINS") {
        use crate::webauthn_registry::effective_tld_plus_one;
        for origin in additional_origins.split(',') {
            let trimmed = origin.trim();
            if let Ok(parsed) = url::Url::parse(trimmed) {
                let host = parsed.host_str().unwrap_or("localhost");
                let add_rp_id = effective_tld_plus_one(host);
                if let Err(e) = registry.seed(&add_rp_id, &parsed).await {
                    tracing::warn!(
                        "Could not seed WebAuthn registry for additional origin '{}': {}",
                        trimmed, e
                    );
                }
            }
        }
    }
    
    let webauthn_state: WebauthnState = Arc::new(WebauthnStateRaw {
        registry,
        reg_state: Cache::builder().time_to_live(Duration::from_secs(300)).build(),
        auth_state: Cache::builder().time_to_live(Duration::from_secs(300)).build(),
    });

    let app = Router::new()
        .merge(create_router(conn.clone()))
        .route("/metrics", get(metrics_endpoint))
        .layer(cors)
        .layer(from_fn(request_id_middleware))           // request_id for correlation
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(DefaultMakeSpan::new().level(Level::INFO))
                .on_request(|request: &Request<_>, _span: &tracing::Span| {
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
                .layer(from_fn_with_state(conn.clone(), log_request_middleware::<Body>))
                .into_inner()
        )
        .layer(Extension(conn.clone()))
        .layer(Extension(rate_limiter))
        .layer(Extension(webauthn_state))
        .layer(Extension(cors_registry));

    let addr = SocketAddr::from(([0, 0, 0, 0], 8000));
    tracing::info!("Listening on {}", addr);
    let listener = TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app)
        .await
        .unwrap();
}
