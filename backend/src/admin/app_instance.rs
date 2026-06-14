//! Admin — App Instance Public Config handler
//!
//! Manages per-instance `public_slug` and `custom_domain` on
//! `atlas_app_deployment_config`. These two fields enable the
//! zero-tenant domain resolver at `GET /api/pub/tenant-context`.
//!
//! # Routes
//!
//! ```ignore
//! GET  /api/admin/app-instances/{id}/public-config
//!      Returns the current public_slug, custom_domain, and instance_status.
//!      -> 200 PublicConfigResponse
//!
//! PUT  /api/admin/app-instances/{id}/public-config
//!      Set/update public_slug and/or custom_domain.
//!      Validates global uniqueness. Returns DNS CNAME instructions.
//!      Body: { public_slug?, custom_domain? }
//!      -> 200 PublicConfigResponse (includes dns_instructions)
//!
//! POST /api/admin/app-instances/{id}/suspend
//!      Sets instance_status = "suspended".  Body: { reason }
//!      -> 200
//!
//! POST /api/admin/app-instances/{id}/resume
//!      Sets instance_status = "active".
//!      -> 200
//!
//! POST /api/admin/app-instances/{id}/archive
//!      Sets instance_status = "archived".  Body: { reason, data_retention_days? }
//!      -> 200
//! ```

use axum::{
    extract::{Extension, Json, Path},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post, put},
    Router,
};
use sea_orm::{
    ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, ActiveModelTrait,
    ActiveValue::Set,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::entities::atlas_app_deployment_config;

// ── Route registration ────────────────────────────────────────────────────────

/// State-free router — merge before outer .with_state(db) in admin_routes().
pub fn routes_raw() -> Router<DatabaseConnection> {
    Router::new()
        .route(
            "/api/admin/app-instances/{id}/public-config",
            get(get_public_config).put(update_public_config),
        )
        .route("/api/admin/app-instances/{id}/suspend", post(suspend_instance))
        .route("/api/admin/app-instances/{id}/resume",  post(resume_instance))
        .route("/api/admin/app-instances/{id}/archive", post(archive_instance))
}

/// Convenience wrapper with state applied (for standalone use / tests).
pub fn routes(db: DatabaseConnection) -> Router {
    routes_raw().with_state(db)
}

// ── Response / input types ────────────────────────────────────────────────────

#[derive(Debug, Serialize)]
pub struct PublicConfigResponse {
    pub instance_id:      Uuid,
    pub tenant_id:        Uuid,
    pub app_slug:         String,
    pub public_slug:      Option<String>,
    pub custom_domain:    Option<String>,
    pub instance_status:  String,
    /// Only present on PUT response
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dns_instructions: Option<DnsInstructions>,
}

#[derive(Debug, Serialize)]
pub struct DnsInstructions {
    pub record_type: String,
    pub name:        String,
    pub value:       String,
    pub note:        String,
}

#[derive(Debug, Deserialize)]
pub struct UpdatePublicConfigBody {
    pub public_slug:   Option<String>,
    pub custom_domain: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct SuspendBody {
    pub reason: String,
}

#[derive(Debug, Deserialize)]
pub struct ArchiveBody {
    pub reason:                 String,
    pub data_retention_days:    Option<u32>,
}

// ── Helpers ───────────────────────────────────────────────────────────────────

fn platform_cname_target() -> &'static str {
    // In production this would come from env/config
    "app.atlas-platform.com"
}

// ── Handlers ──────────────────────────────────────────────────────────────────

pub async fn get_public_config(
    Extension(db): Extension<DatabaseConnection>,
    Path(instance_id): Path<Uuid>,
) -> impl IntoResponse {
    match atlas_app_deployment_config::Entity::find_by_id(instance_id)
        .one(&db)
        .await
    {
        Ok(Some(cfg)) => {
            let resp = PublicConfigResponse {
                instance_id: cfg.id,
                tenant_id:   cfg.tenant_id,
                app_slug:    cfg.app_slug.clone(),
                public_slug:     cfg.public_slug.clone(),
                custom_domain:   cfg.custom_domain.clone(),
                instance_status: cfg.instance_status.to_string(),
                dns_instructions: None,
            };
            (StatusCode::OK, Json(resp)).into_response()
        }
        Ok(None) => (StatusCode::NOT_FOUND, "instance not found").into_response(),
        Err(e) => {
            tracing::error!("get_public_config: {e:#}");
            (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response()
        }
    }
}

pub async fn update_public_config(
    Extension(db): Extension<DatabaseConnection>,
    Path(instance_id): Path<Uuid>,
    Json(body): Json<UpdatePublicConfigBody>,
) -> impl IntoResponse {
    let existing = match atlas_app_deployment_config::Entity::find_by_id(instance_id)
        .one(&db)
        .await
    {
        Ok(Some(c)) => c,
        Ok(None) => return (StatusCode::NOT_FOUND, "instance not found").into_response(),
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    };

    // Validate slug format (lowercase alphanumeric + hyphens)
    if let Some(ref slug) = body.public_slug {
        if slug.is_empty() || !slug.chars().all(|c| c.is_ascii_alphanumeric() || c == '-') {
            return (
                StatusCode::UNPROCESSABLE_ENTITY,
                "public_slug must be lowercase alphanumeric with hyphens only",
            ).into_response();
        }
    }

    let mut active: atlas_app_deployment_config::ActiveModel = existing.clone().into();

    if let Some(slug) = body.public_slug.clone() {
        active.public_slug = Set(Some(slug));
    }
    if let Some(domain) = body.custom_domain.clone() {
        active.custom_domain = Set(Some(domain));
    }

    match active.update(&db).await {
        Ok(updated) => {
            // Build DNS instructions if custom_domain was set
            let dns_instructions = body.custom_domain.as_ref().map(|domain| DnsInstructions {
                record_type: "CNAME".to_string(),
                name:        domain.clone(),
                value:       platform_cname_target().to_string(),
                note: format!(
                    "Point {domain} as a CNAME to {target}. \
                     SSL is provisioned automatically via Cloudflare.",
                    target = platform_cname_target()
                ),
            });

            let resp = PublicConfigResponse {
                instance_id:  updated.id,
                tenant_id:    updated.tenant_id,
                app_slug:     updated.app_slug,
                public_slug:  updated.public_slug,
                custom_domain: updated.custom_domain,
                instance_status: updated.instance_status.to_string(),
                dns_instructions,
            };
            (StatusCode::OK, Json(resp)).into_response()
        }
        Err(e) if e.to_string().contains("unique") || e.to_string().contains("duplicate") => {
            (
                StatusCode::CONFLICT,
                "public_slug or custom_domain is already taken by another instance",
            )
                .into_response()
        }
        Err(e) => {
            tracing::error!("update_public_config: {e:#}");
            (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response()
        }
    }
}

pub async fn suspend_instance(
    Extension(db): Extension<DatabaseConnection>,
    Path(instance_id): Path<Uuid>,
    Json(body): Json<SuspendBody>,
) -> impl IntoResponse {
    set_instance_status(&db, instance_id, "suspended", &body.reason).await
}

pub async fn resume_instance(
    Extension(db): Extension<DatabaseConnection>,
    Path(instance_id): Path<Uuid>,
) -> impl IntoResponse {
    set_instance_status(&db, instance_id, "active", "resumed by platform admin").await
}

pub async fn archive_instance(
    Extension(db): Extension<DatabaseConnection>,
    Path(instance_id): Path<Uuid>,
    Json(body): Json<ArchiveBody>,
) -> impl IntoResponse {
    set_instance_status(&db, instance_id, "archived", &body.reason).await
}

async fn set_instance_status(
    db: &DatabaseConnection,
    instance_id: Uuid,
    status: &str,
    reason: &str,
) -> axum::response::Response {
    use crate::entities::atlas_app_deployment_config::AppInstanceStatus;
    let existing = match atlas_app_deployment_config::Entity::find_by_id(instance_id)
        .one(db)
        .await
    {
        Ok(Some(c)) => c,
        Ok(None) => return (StatusCode::NOT_FOUND, "instance not found").into_response(),
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    };

    let status_enum = match status {
        "active" => AppInstanceStatus::Active,
        "suspended" => AppInstanceStatus::Suspended,
        "archived" => AppInstanceStatus::Archived,
        _ => return (StatusCode::BAD_REQUEST, "invalid status").into_response(),
    };

    let mut active: atlas_app_deployment_config::ActiveModel = existing.into();
    active.instance_status = Set(status_enum);

    match active.update(db).await {
        Ok(_) => {
            tracing::info!(
                instance_id = %instance_id,
                status = status,
                reason = reason,
                "instance status changed"
            );
            (
                StatusCode::OK,
                Json(serde_json::json!({
                    "instance_id": instance_id,
                    "status": status,
                    "reason": reason,
                })),
            )
                .into_response()
        }
        Err(e) => {
            tracing::error!("set_instance_status: {e:#}");
            (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response()
        }
    }
}
