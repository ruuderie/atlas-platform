use axum::{
    Json, Router, extract::Request, http::HeaderValue, middleware::Next, response::Response,
    routing::get,
};
use sea_orm::DatabaseConnection;
use serde::Serialize;

// ── Version constants ─────────────────────────────────────────────────────────
// Semver comes from Cargo.toml at compile time.
// SHA and date are injected by CI via env vars; fall back to static strings.

pub const ATLAS_VERSION: &str = env!("CARGO_PKG_VERSION");

// option_env! returns Option<&str> at compile time. We can't call .unwrap_or()
// in a const context on stable Rust, so we match explicitly.
pub const ATLAS_BUILD_SHA: &str = match option_env!("ATLAS_BUILD_SHA") {
    Some(s) => s,
    None => "dev",
};

pub const ATLAS_BUILD_DATE: &str = match option_env!("ATLAS_BUILD_DATE") {
    Some(s) => s,
    None => "unknown",
};

// ── Response model ────────────────────────────────────────────────────────────

#[derive(Serialize)]
pub struct VersionResponse {
    pub version: &'static str,
    pub build_sha: &'static str,
    pub build_date: &'static str,
    pub environment: String,
}

// ── Handler ───────────────────────────────────────────────────────────────────

/// `GET /api/version`
///
/// Returns the current platform version, build SHA, and build date.
/// Used by platform-admin, health monitors, and multi-tenant drift detection
/// (via the `X-Atlas-Version` response header on every request).
pub async fn get_version() -> Json<VersionResponse> {
    let environment = std::env::var("ENVIRONMENT").unwrap_or_else(|_| "dev".to_string());
    Json(VersionResponse {
        version: ATLAS_VERSION,
        build_sha: ATLAS_BUILD_SHA,
        build_date: ATLAS_BUILD_DATE,
        environment,
    })
}

/// Public route — no auth required (monitoring and admin tooling need this).
/// Returns a Router<DatabaseConnection> to satisfy the outer router's state type.
pub fn public_routes() -> Router<DatabaseConnection> {
    Router::new().route("/api/version", get(get_version))
}

// ── X-Atlas-Version middleware ────────────────────────────────────────────────

/// Injects `X-Atlas-Version: <semver>+<sha>` into every response.
///
/// This allows platform-admin and ops tooling to detect version drift across
/// nodes without polling the `/api/version` endpoint. Simply inspect any
/// API response header.
pub async fn version_header_middleware(req: Request, next: Next) -> Response {
    let mut response = next.run(req).await;
    let version_str = format!("{}+{}", ATLAS_VERSION, ATLAS_BUILD_SHA);
    if let Ok(val) = HeaderValue::from_str(&version_str) {
        response.headers_mut().insert("X-Atlas-Version", val);
    }
    response
}
