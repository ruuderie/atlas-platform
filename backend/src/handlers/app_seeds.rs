#![allow(dead_code)]
use axum::{
    Json, Router,
    extract::{Path, State},
    http::StatusCode,
    routing::{get, post},
};
use sea_orm::DatabaseConnection;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::entities::app_instance;
use sea_orm::EntityTrait;

// ──────────────────────────────────────────────────────────────────────────────
// RESPONSE TYPES
// ──────────────────────────────────────────────────────────────────────────────

/// A seed pack as returned to the platform-admin UI.
#[derive(Serialize, Deserialize, Debug)]
pub struct SeedPackInfo {
    pub id: String,
    pub title: String,
    pub description: String,
    pub content_summary: String,
    /// ISO 8601 timestamp of the most recent application, if ever applied.
    pub last_applied_at: Option<String>,
    /// Total number of times this pack has been applied to this instance.
    pub apply_count: u32,
}

/// Response for POST apply — summarises what happened.
#[derive(Serialize, Deserialize, Debug)]
pub struct SeedApplyResponse {
    pub seed_id: String,
    pub success: bool,
    pub message: String,
}

// ──────────────────────────────────────────────────────────────────────────────
// ROUTES
// ──────────────────────────────────────────────────────────────────────────────

/// State-free authenticated route definitions.
/// Use inside `AtlasApp::authenticated_router()`. Never call `.with_state()` here.
pub fn authenticated_routes_raw() -> Router<DatabaseConnection> {
    Router::new()
        .route(
            "/api/app-instances/{app_instance_id}/seeds",
            get(list_seed_packs),
        )
        .route(
            "/api/app-instances/{app_instance_id}/seeds/{seed_id}/apply",
            post(apply_seed_pack),
        )
}

/// Legacy state-finalized constructor. Used by api.rs during transition period.
/// Remove after CorePlatformApp is active and api.rs is cleaned up (Phase 3).
pub fn authenticated_routes(db: DatabaseConnection) -> Router<DatabaseConnection> {
    authenticated_routes_raw().with_state(db)
}

// ──────────────────────────────────────────────────────────────────────────────
// HANDLERS
// ──────────────────────────────────────────────────────────────────────────────

/// Lists all seed packs available for an app instance's app type,
/// including whether each pack has been previously applied and when.
pub async fn list_seed_packs(
    Path(app_instance_id): Path<Uuid>,
    State(db): State<DatabaseConnection>,
) -> Result<(StatusCode, Json<Vec<SeedPackInfo>>), StatusCode> {
    let instance = app_instance::Entity::find_by_id(app_instance_id)
        .one(&db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;

    let apps = crate::atlas_apps::get_active_apps();
    let app = apps
        .iter()
        .find(|a| a.app_id() == instance.app_type.as_str())
        .ok_or(StatusCode::UNPROCESSABLE_ENTITY)?;

    let packs = app.seed_packs();
    if packs.is_empty() {
        return Ok((StatusCode::OK, Json(vec![])));
    }

    // Load all seed application records for this tenant in one query.
    use sea_orm::{ConnectionTrait, Statement};
    let rows = db
        .query_all(Statement::from_string(
            sea_orm::DatabaseBackend::Postgres,
            format!(
                "SELECT key, value FROM tenant_setting
             WHERE tenant_id = '{}' AND key LIKE 'seed_applied:%'",
                instance.tenant_id
            ),
        ))
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Build a map: seed_id → (latest_timestamp, count).
    // Each re-application overwrites the value with the latest timestamp.
    // For count we'd need a separate counter; for now we surface last_applied_at.
    let mut applied_map: std::collections::HashMap<String, String> =
        std::collections::HashMap::new();
    for row in rows {
        if let (Ok(key), Ok(value)) = (
            row.try_get::<String>("", "key"),
            row.try_get::<String>("", "value"),
        ) {
            let seed_id = key.trim_start_matches("seed_applied:").to_string();
            applied_map.insert(seed_id, value);
        }
    }

    let result: Vec<SeedPackInfo> = packs
        .iter()
        .map(|p| {
            let last_applied = applied_map.get(p.id).cloned();
            SeedPackInfo {
                id: p.id.to_string(),
                title: p.title.to_string(),
                description: p.description.to_string(),
                content_summary: p.content_summary.to_string(),
                apply_count: if last_applied.is_some() { 1 } else { 0 },
                last_applied_at: last_applied,
            }
        })
        .collect();

    Ok((StatusCode::OK, Json(result)))
}

/// Applies a seed pack to an app instance scoped to its tenant.
/// Seed packs are idempotent on global tables; re-application is allowed and
/// updates the last_applied_at timestamp for audit purposes.
pub async fn apply_seed_pack(
    Path((app_instance_id, seed_id)): Path<(Uuid, String)>,
    State(db): State<DatabaseConnection>,
) -> Result<(StatusCode, Json<SeedApplyResponse>), StatusCode> {
    let instance = app_instance::Entity::find_by_id(app_instance_id)
        .one(&db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;

    let apps = crate::atlas_apps::get_active_apps();
    let app = apps
        .iter()
        .find(|a| a.app_id() == instance.app_type.as_str())
        .ok_or(StatusCode::UNPROCESSABLE_ENTITY)?;

    // Guard: reject unknown seed IDs (same pattern as complete_step phantom ID guard).
    let pack = app
        .seed_packs()
        .into_iter()
        .find(|p| p.id == seed_id.as_str())
        .ok_or_else(|| {
            tracing::warn!(
                "apply_seed_pack: unknown seed_id '{}' for app '{}'",
                seed_id,
                instance.app_type
            );
            StatusCode::UNPROCESSABLE_ENTITY
        })?;

    // Execute the seed pack.
    match (pack.apply)(db, instance.tenant_id, app_instance_id).await {
        Ok(()) => {
            tracing::info!(
                "Seed pack '{}' applied successfully for tenant {}",
                seed_id,
                instance.tenant_id
            );
            Ok((
                StatusCode::OK,
                Json(SeedApplyResponse {
                    seed_id,
                    success: true,
                    message: format!("Seed pack '{}' applied successfully.", pack.title),
                }),
            ))
        }
        Err(e) => {
            tracing::error!(
                "Seed pack '{}' failed for tenant {}: {}",
                seed_id,
                instance.tenant_id,
                e
            );
            Ok((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(SeedApplyResponse {
                    seed_id,
                    success: false,
                    message: format!("Seed pack application failed: {e}"),
                }),
            ))
        }
    }
}
