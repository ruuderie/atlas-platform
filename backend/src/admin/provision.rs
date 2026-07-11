use axum::{
    Json,
    extract::{Extension, Path, State},
    http::StatusCode,
    response::IntoResponse,
};
use sea_orm::DatabaseConnection;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::entities::user;
use crate::services::audit::AuditService;

/// Response returned after a successful (or failed) provision call.
#[derive(Serialize, Deserialize, Debug)]
pub struct ProvisionResponse {
    pub tenant_id: Uuid,
    pub success: bool,
    pub message: String,
}

/// `POST /api/admin/platform/provision/{tenant_id}`
///
/// Bootstraps a new tenant by calling `provision()` on every registered AtlasApp.
/// Each app's provision implementation is idempotent — safe to call multiple times.
///
/// This endpoint is admin-only (protected by the auth middleware in api.rs).
pub async fn provision_tenant(
    Path(tenant_id): Path<Uuid>,
    State(db): State<DatabaseConnection>,
    Extension(current_user): Extension<user::Model>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let apps = crate::atlas_apps::get_active_apps();

    for app in &apps {
        if let Err(e) = app.provision(&db, tenant_id).await {
            tracing::error!(
                "provision_tenant: app '{}' failed for tenant {}: {}",
                app.app_id(),
                tenant_id,
                e
            );
            AuditService::log_action(
                db.clone(),
                Some(tenant_id),
                Some(current_user.id),
                "tenant.provision.failed".to_string(),
                "Tenant".to_string(),
                tenant_id,
                None,
                Some(serde_json::json!({
                    "app_id": app.app_id(),
                    "error": e.to_string(),
                })),
                None,
            );
            return Ok((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ProvisionResponse {
                    tenant_id,
                    success: false,
                    message: format!("App '{}' provision failed: {}", app.app_id(), e),
                }),
            ));
        }

        tracing::info!(
            "provision_tenant: app '{}' provisioned tenant {} successfully",
            app.app_id(),
            tenant_id
        );
    }

    AuditService::log_action(
        db.clone(),
        Some(tenant_id),
        Some(current_user.id),
        "tenant.provisioned".to_string(),
        "Tenant".to_string(),
        tenant_id,
        None,
        Some(serde_json::json!({
            "apps_count": apps.len(),
        })),
        None,
    );

    Ok((
        StatusCode::OK,
        Json(ProvisionResponse {
            tenant_id,
            success: true,
            message: format!(
                "Tenant {} provisioned successfully across {} apps.",
                tenant_id,
                apps.len()
            ),
        }),
    ))
}
