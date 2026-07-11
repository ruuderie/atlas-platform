//! GET  /api/folio/config  — read this tenant's Folio deployment config
//! PUT  /api/folio/config  — update deployment mode (Owner/Admin only)
//!
//! This is the app-level configuration API. An org admin can switch their
//! Folio instance deployment mode and update the configuration JSON.

use axum::{
    Extension, Json, Router,
    http::StatusCode,
    response::IntoResponse,
    routing::{get, put},
};
use sea_orm::{ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, Set};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::entities::atlas_app_deployment_config::{self, AppDeploymentMode};
use crate::entities::user_account::UserRole;
use crate::extractors::tenant::TenantContext;

/// Routes registered in the landlord_router (Owner/Admin can read; PUT requires Owner/Admin).
/// Read is available to all authenticated Folio users so the frontend can react to the mode.
pub fn authenticated_routes_raw() -> Router<DatabaseConnection> {
    Router::new().route("/api/folio/config", get(get_config).put(update_config))
}

// ── Response / request types ──────────────────────────────────────────────────

#[derive(Serialize)]
pub struct AppConfigResponse {
    pub tenant_id: Uuid,
    pub app_slug: &'static str,
    pub mode: AppDeploymentMode,
    pub config: serde_json::Value,
}

#[derive(Deserialize)]
pub struct UpdateConfigRequest {
    /// New platform deployment mode.
    pub mode: AppDeploymentMode,
    /// Arbitrary config JSON — passed through as-is.
    #[serde(default)]
    pub config: Option<serde_json::Value>,
}

// ── Read config ───────────────────────────────────────────────────────────────

async fn get_config(
    ctx: TenantContext,
    Extension(db): Extension<DatabaseConnection>,
) -> impl IntoResponse {
    let row = atlas_app_deployment_config::Entity::find()
        .filter(atlas_app_deployment_config::Column::TenantId.eq(ctx.tenant_id))
        .filter(atlas_app_deployment_config::Column::AppSlug.eq("folio"))
        .one(&db)
        .await;

    match row {
        Ok(Some(r)) => Json(AppConfigResponse {
            tenant_id: ctx.tenant_id,
            app_slug: "folio",
            mode: r.mode,
            config: r.config,
        })
        .into_response(),

        // No row = standard mode, no config
        Ok(None) => Json(AppConfigResponse {
            tenant_id: ctx.tenant_id,
            app_slug: "folio",
            mode: AppDeploymentMode::Standard,
            config: serde_json::json!({}),
        })
        .into_response(),

        Err(e) => {
            tracing::error!(error = %e, "folio/config: DB error");
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}

// ── Update config (Owner/Admin only) ─────────────────────────────────────────

async fn update_config(
    ctx: TenantContext,
    Extension(db): Extension<DatabaseConnection>,
    Json(body): Json<UpdateConfigRequest>,
) -> impl IntoResponse {
    // Only Owner or Admin may change the deployment mode.
    if !matches!(
        ctx.user_role,
        UserRole::Owner | UserRole::Admin | UserRole::PlatformSuperAdmin
    ) {
        return StatusCode::FORBIDDEN.into_response();
    }

    let final_config = body.config.unwrap_or_else(|| serde_json::json!({}));

    // Upsert: INSERT … ON CONFLICT (tenant_id, app_slug) DO UPDATE
    let existing = atlas_app_deployment_config::Entity::find()
        .filter(atlas_app_deployment_config::Column::TenantId.eq(ctx.tenant_id))
        .filter(atlas_app_deployment_config::Column::AppSlug.eq("folio"))
        .one(&db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR);

    match existing {
        Ok(Some(row)) => {
            let mut active: atlas_app_deployment_config::ActiveModel = row.into();
            active.mode = Set(body.mode.clone());
            active.config = Set(final_config.clone());
            match active.update(&db).await {
                Ok(_) => {}
                Err(e) => {
                    tracing::error!(error = %e, "folio/config: update failed");
                    return StatusCode::INTERNAL_SERVER_ERROR.into_response();
                }
            }
        }
        Ok(None) => {
            let new_row = atlas_app_deployment_config::ActiveModel {
                id: Set(Uuid::new_v4()),
                tenant_id: Set(ctx.tenant_id),
                app_slug: Set("folio".to_string()),
                mode: Set(body.mode.clone()),
                config: Set(final_config.clone()),
                ..Default::default()
            };
            match new_row.insert(&db).await {
                Ok(_) => {}
                Err(e) => {
                    tracing::error!(error = %e, "folio/config: insert failed");
                    return StatusCode::INTERNAL_SERVER_ERROR.into_response();
                }
            }
        }
        Err(_e) => return StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    }

    tracing::info!(
        tenant_id = %ctx.tenant_id,
        new_mode = ?body.mode,
        "folio/config: deployment mode updated"
    );

    Json(AppConfigResponse {
        tenant_id: ctx.tenant_id,
        app_slug: "folio",
        mode: body.mode,
        config: final_config,
    })
    .into_response()
}
