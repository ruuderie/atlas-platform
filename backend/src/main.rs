mod admin;
mod api;
pub mod atlas_apps;
mod auth;
mod config;
mod db;
mod entities;
mod extractors; // G-32: Axum extractors for declarative role enforcement
mod handlers;
pub mod metrics;
mod middleware;
mod migration;
mod models;
mod services;
mod traits;
mod types;
mod webauthn_registry;

use crate::api::create_router;
use crate::sea_orm::Database;
use axum::body::Body;
use axum::http::{self, HeaderMap, HeaderValue, Method, Request, StatusCode, header};
use axum::middleware::{from_fn, from_fn_with_state};
use axum::{Extension, Router, routing::get};
use migration::Migrator;
use sea_orm_migration::prelude::*;
use std::net::SocketAddr;
use tokio::net::TcpListener;
use tower::ServiceBuilder;
use tower_http::cors::CorsLayer;
use tower_http::trace::{DefaultMakeSpan, DefaultOnResponse, TraceLayer};
use tracing::Level;

use crate::middleware::{
    DynamicCorsRegistry, dynamic_cors_layer, middleware::log_request_middleware,
    rate_limiter::RateLimiter, request_id::request_id_middleware, request_logger::RequestLogger,
};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
// webauthn_rs::prelude::* intentionally not imported — symbols pulled in via explicit crate::handlers::passkeys items
use crate::handlers::passkeys::{WebauthnState, WebauthnStateRaw};
use crate::services::ingress_provisioner::IngressProvisioner;
use crate::webauthn_registry::WebauthnRegistry;
use moka::future::Cache;
use std::sync::Arc;
use std::time::Duration;

#[allow(dead_code)]
async fn handle_error(
    error: Box<dyn std::error::Error + Send + Sync>,
) -> (http::StatusCode, String) {
    tracing::error!("Unhandled error: {:?}", error);
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        "Internal Server Error".to_string(),
    )
}

#[allow(dead_code)]
fn configure_cors(network_client: &str, admin_client: &str) -> CorsLayer {
    let env = std::env::var("ENVIRONMENT").unwrap_or_else(|_| "production".to_string());
    let is_dev = env == "development" || env == "dev";

    let allow_origin = if is_dev {
        // [WARNING]: DANGEROUS IN PRODUCTION
        // This dynamically echoes ANY origin, useful only for local multi-tenant Orbstack/Docker networking.
        tower_http::cors::AllowOrigin::predicate(|_, _| true)
    } else {
        let mut origins = vec![
            network_client
                .parse::<HeaderValue>()
                .unwrap_or_else(|_| "http://frontend:5001".parse().unwrap()),
            admin_client
                .parse::<HeaderValue>()
                .unwrap_or_else(|_| "http://admin:5150".parse().unwrap()),
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
        .allow_methods([
            Method::GET,
            Method::POST,
            Method::PUT,
            Method::DELETE,
            Method::OPTIONS,
        ])
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
    let database_url =
        if std::env::var("USE_LOCAL_DB").unwrap_or_else(|_| "false".to_string()) == "true" {
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
    tracing::info!(
        "Using database URL: {}",
        database_url.replace(
            |c: char| !c.is_ascii_alphabetic() && !c.is_ascii_punctuation(),
            "*"
        )
    );

    // Connect to the database
    let conn = Database::connect(&database_url)
        .await
        .expect("Failed to connect to the database");

    let _request_logger = RequestLogger::new(conn.clone());

    use sea_orm::{ConnectionTrait, Statement};

    if std::env::var("WIPE_DB_ON_STARTUP").unwrap_or_else(|_| "false".to_string()) == "true" {
        tracing::warn!(
            "WIPE_DB_ON_STARTUP is enabled! Wiping the database to start from scratch..."
        );
        conn.execute(Statement::from_string(
            sea_orm::DatabaseBackend::Postgres,
            "DROP SCHEMA public CASCADE; CREATE SCHEMA public;".to_owned(),
        ))
        .await
        .expect("Failed to recreate public schema");
    }

    // Fix renamed migrations in the database before running Migrator.
    // Each pair is handled with two independent single-statement executes:
    //   1. DELETE old if new already exists (resolves the "both applied" conflict
    //      that occurs when a unique-key violation silently aborts a multi-statement batch)
    //   2. UPDATE old → new if only old exists (normal rename path)
    // Both steps are idempotent via EXISTS / NOT EXISTS guards.
    tracing::info!("Ensuring legacy migration names are updated in seaql_migrations...");
    let legacy_renames: &[(&str, &str)] = &[
        // 2023 renames — original filename-based names → timestamped names
        (
            "m20230912_create_users_table",
            "m20230912_000000_create_users_table",
        ),
        (
            "m20230912_create_user_accounts_table",
            "m20230912_000001_create_user_accounts_table",
        ),
        // G19/G23/G26 — registered via CorePlatformApp as m20260701_* names,
        // superseded by standalone base-vec migrations with new timestamps.
        (
            "m20260701_g23_reservations",
            "m20260802_g23_atlas_reservations",
        ),
        ("m20260701_g26_catalog", "m20260803_g26_atlas_catalog"),
        ("m20260701_g19_campaigns", "m20260804_g19_atlas_campaigns"),
    ];
    for (old_ver, new_ver) in legacy_renames {
        // Step 1: If BOTH old and new exist, delete the old (new is already applied).
        let sql_delete = format!(
            "DELETE FROM seaql_migrations WHERE version = '{old_ver}' \
             AND EXISTS (SELECT 1 FROM seaql_migrations WHERE version = '{new_ver}')"
        );
        match conn
            .execute(Statement::from_string(
                sea_orm::DatabaseBackend::Postgres,
                sql_delete,
            ))
            .await
        {
            Ok(r) if r.rows_affected() > 0 => {
                tracing::info!(
                    "seaql_migrations: removed duplicate old row '{}' (new '{}' already applied)",
                    old_ver,
                    new_ver
                );
            }
            Ok(_) => {}
            Err(e) => tracing::warn!("seaql_migrations DELETE '{}' failed: {}", old_ver, e),
        }
        // Step 2: If only old exists, rename it.
        let sql_update = format!(
            "UPDATE seaql_migrations SET version = '{new_ver}' \
             WHERE version = '{old_ver}' \
             AND NOT EXISTS (SELECT 1 FROM seaql_migrations WHERE version = '{new_ver}')"
        );
        match conn
            .execute(Statement::from_string(
                sea_orm::DatabaseBackend::Postgres,
                sql_update,
            ))
            .await
        {
            Ok(r) if r.rows_affected() > 0 => {
                tracing::info!("seaql_migrations: renamed '{}' → '{}'", old_ver, new_ver);
            }
            Ok(_) => {}
            Err(e) => tracing::warn!(
                "seaql_migrations rename '{}' → '{}' failed: {}",
                old_ver,
                new_ver,
                e
            ),
        }
    }

    // Run migrations
    Migrator::up(&conn, None).await.unwrap();

    tracing::info!("Migrations completed");

    let table_exists = &conn
        .execute_unprepared("SELECT 1 FROM request_log LIMIT 1")
        .await
        .is_ok();
    tracing::info!("request_log table exists: {}", table_exists);

    if create_admin {
        let admin_email =
            std::env::var("ADMIN_USER").unwrap_or_else(|_| "admin@oply.co".to_string());
        if let Err(e) = crate::admin::setup::create_admin_user_if_not_exists(
            &conn,
            &admin_email,
            &_admin_password,
        )
        .await
        {
            tracing::error!("Failed to create admin user: {}", e);
        } else {
            tracing::info!(
                "Successfully verified root administrative account for {}",
                admin_email
            );
        }
    }

    tracing::info!("Successfully connected to the database and ran migrations");

    let sync_db = conn.clone();
    crate::services::data_sync::DataSyncService::start_worker(sync_db).await;
    let outbox_db = conn.clone();
    crate::services::outbox_worker::OutboxWorker::start_worker(outbox_db).await;
    // G-05 Syndication Event Bus — polls atlas_syndication_outbox every 10s,
    // dispatches outbound events to linked NI webhook URLs with HMAC-SHA256 signing,
    // exponential back-off, and dead-letter after 5 failed attempts.
    let syndication_db = conn.clone();
    crate::services::syndication_event_bus::SyndicationEventBus::start_worker(syndication_db).await;
    let telemetry_db = conn.clone();
    tokio::spawn(async move {
        // Run every hour
        let mut interval = tokio::time::interval(Duration::from_secs(3600));
        loop {
            interval.tick().await;
            if let Err(e) =
                crate::services::telemetry::TelemetryService::process_daily_metrics(&telemetry_db)
                    .await
            {
                tracing::error!("Background telemetry processing failed: {}", e);
            }
        }
    });

    let webhook_db = conn.clone();
    crate::services::webhook::start_webhook_sweeper(webhook_db).await;

    let network_client =
        std::env::var("FRONTEND_URL").unwrap_or_else(|_| "http://localhost:5001".to_string());
    let admin_client =
        std::env::var("ADMIN_URL").unwrap_or_else(|_| "http://localhost:5002".to_string());
    tracing::info!("Network URL: {}", network_client);
    tracing::info!("Admin URL: {}", admin_client);

    let cors_registry = Arc::new(DynamicCorsRegistry::new(conn.clone()));
    cors_registry
        .hydrate(&[network_client.clone(), admin_client.clone()])
        .await;
    let cors = dynamic_cors_layer(cors_registry.clone());

    let rate_limiter = RateLimiter::new();

    let rp_origin_str = std::env::var("WEBAUTHN_ORIGIN").unwrap_or_else(|_| admin_client.clone());
    let rp_origin = url::Url::parse(&rp_origin_str).unwrap_or_else(|_| {
        url::Url::parse("https://platform-admin.atlas-platform.orb.local").unwrap()
    });

    let rp_id = std::env::var("RP_ID")
        .unwrap_or_else(|_| rp_origin.host_str().unwrap_or("localhost").to_string());

    let primary_host = rp_origin.host_str().map(|s| s.to_string());
    let primary_rp_id = Some(rp_id.clone());

    let registry = Arc::new(WebauthnRegistry::new(
        conn.clone(),
        10_000,
        primary_host.clone(),
        primary_rp_id.clone(),
    ));

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

        use sea_orm::EntityTrait;
        match app_domain::Entity::find().all(&conn).await {
            Ok(domains) => {
                tracing::info!(
                    "Pre-warming WebAuthn registry for {} tenant domain(s)...",
                    domains.len()
                );
                for domain in domains {
                    let origin_str = format!("https://{}", domain.domain_name);
                    match url::Url::parse(&origin_str) {
                        Ok(origin_url) => {
                            if let Some(domain_host) = origin_url.host_str() {
                                if let Some(ref prim_host) = primary_host {
                                    if domain_host.eq_ignore_ascii_case(prim_host) {
                                        tracing::info!(
                                            "Skipping WebAuthn pre-warm for primary platform domain '{}' to respect explicit RP_ID configuration.",
                                            domain.domain_name
                                        );
                                        continue;
                                    }
                                }
                            }
                            let domain_host = origin_url.host_str().unwrap_or(&domain.domain_name);
                            let domain_rp_id = if let (Some(prim_host), Some(prim_rp_id)) =
                                (&primary_host, &primary_rp_id)
                            {
                                if domain_host.eq_ignore_ascii_case(prim_host) {
                                    prim_rp_id.clone()
                                } else {
                                    domain_host.to_string()
                                }
                            } else {
                                domain_host.to_string()
                            };
                            if let Err(e) = registry.seed(&domain_rp_id, &origin_url).await {
                                tracing::warn!(
                                    "Could not pre-warm WebAuthn for domain '{}': {}",
                                    domain.domain_name,
                                    e
                                );
                            }
                        }
                        Err(e) => {
                            tracing::warn!(
                                "Skipping WebAuthn pre-warm for invalid domain '{}': {}",
                                domain.domain_name,
                                e
                            );
                        }
                    }
                }
            }
            Err(e) => {
                tracing::warn!(
                    "Could not load app_domain records for WebAuthn pre-warm: {}",
                    e
                );
            }
        }
    }

    // Additional origins (e.g. tenant custom domains listed in ADDITIONAL_ALLOWED_ORIGINS).
    // Must use eTLD+1 as rp_id — the same rule applied in get_or_create() and the tenant
    // pre-warm loop above. Using the full host (e.g. "dev.buildwithruud.com") as rp_id
    // would cause browsers to reject challenges for passkeys registered under "buildwithruud.com".
    if let Ok(additional_origins) = std::env::var("ADDITIONAL_ALLOWED_ORIGINS") {
        for origin in additional_origins.split(',') {
            let trimmed = origin.trim();
            if let Ok(parsed) = url::Url::parse(trimmed) {
                if let Some(parsed_host) = parsed.host_str() {
                    if let Some(ref prim_host) = primary_host {
                        if parsed_host.eq_ignore_ascii_case(prim_host) {
                            tracing::info!(
                                "Skipping WebAuthn seed for additional origin '{}' because it matches primary platform domain.",
                                trimmed
                            );
                            continue;
                        }
                    }
                }
                let host = parsed.host_str().unwrap_or("localhost");
                let add_rp_id =
                    if let (Some(prim_host), Some(prim_rp_id)) = (&primary_host, &primary_rp_id) {
                        if host.eq_ignore_ascii_case(prim_host) {
                            prim_rp_id.clone()
                        } else {
                            host.to_string()
                        }
                    } else {
                        host.to_string()
                    };
                if let Err(e) = registry.seed(&add_rp_id, &parsed).await {
                    tracing::warn!(
                        "Could not seed WebAuthn registry for additional origin '{}': {}",
                        trimmed,
                        e
                    );
                }
            }
        }
    }

    let webauthn_state: WebauthnState = Arc::new(WebauthnStateRaw {
        registry,
        reg_state: Cache::builder()
            .time_to_live(Duration::from_secs(300))
            .build(),
        auth_state: Cache::builder()
            .time_to_live(Duration::from_secs(300))
            .build(),
    });

    let ingress_provisioner = Arc::new(IngressProvisioner::new());

    let app = Router::new()
        .merge(create_router(conn.clone()))
        .route("/metrics", get(metrics_endpoint))
        .layer(cors)
        .layer(from_fn(request_id_middleware)) // request_id for correlation
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
                .on_response(DefaultOnResponse::new().level(Level::INFO)),
        )
        .layer(
            ServiceBuilder::new()
                .layer(from_fn_with_state(
                    conn.clone(),
                    log_request_middleware::<Body>,
                ))
                .into_inner(),
        )
        .layer(Extension(conn.clone()))
        .layer(Extension(rate_limiter))
        .layer(Extension(webauthn_state))
        .layer(Extension(cors_registry))
        .layer(Extension(ingress_provisioner));

    let addr = SocketAddr::from(([0, 0, 0, 0], 8000));
    tracing::info!("Listening on {}", addr);
    let listener = TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
