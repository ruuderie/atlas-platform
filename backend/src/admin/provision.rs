use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use sea_orm::DatabaseConnection;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

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
