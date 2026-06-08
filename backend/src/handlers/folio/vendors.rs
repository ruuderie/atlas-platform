//! Folio — Vendors handler.
//!
//! # Routes
//!
//! ```ignore
//! GET  /api/folio/vendors
//!      List all service providers (vendors/contractors) for the tenant.
//!      -> 200 [VendorSummary]
//!
//! POST /api/folio/vendors
//!      Onboard a new vendor. Provisions a Contractor Performance G-27 scorecard (Phase 2).
//!      Body: CreateVendorHttpInput
//!      -> 201 { "id": uuid }
//!
//! PATCH /api/folio/vendors/:id/emergency
//!       Toggle emergency availability flag on a vendor.
//!       Body: { "available": bool }
//!       -> 204
//! ```

use axum::{
    extract::{Extension, Json, Path},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, patch},
    Router,
};
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::entities::user;
use crate::services::pm::vendor::{CreateVendorInput, VendorService};
use crate::types::pm::TradeType;

// ── Route registration ────────────────────────────────────────────────────────

pub fn authenticated_routes_raw() -> Router<DatabaseConnection> {
    Router::new()
        .route("/api/folio/vendors", get(list_vendors).post(create_vendor))
        .route("/api/folio/vendors/{id}/emergency", patch(toggle_emergency))
}

// ── Request / response types ──────────────────────────────────────────────────

#[derive(Debug, Serialize)]
struct VendorSummary {
    pub id: Uuid,
    pub business_name: String,
    pub trade_type: Option<String>,
    pub status: String,
    pub is_emergency_available: bool,
    pub rating_avg: Option<f64>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreateVendorHttpInput {
    /// Must link to an existing atlas user. Required by the entity.
    pub user_id: Uuid,
    pub business_name: String,
    /// Trade type enum value e.g. "plumbing", "electrical", "hvac".
    pub trade_type: String,
    pub license_number: Option<String>,
    pub license_state: Option<String>,
    pub is_emergency_available: bool,
    pub hourly_rate_cents: Option<i64>,
    pub is_insured: bool,
    pub is_bonded: bool,
}

#[derive(Debug, Serialize)]
struct CreateVendorResponse {
    pub id: Uuid,
}

#[derive(Debug, Deserialize)]
pub struct ToggleEmergencyInput {
    pub available: bool,
}

// ── Handlers ──────────────────────────────────────────────────────────────────

/// GET /api/folio/vendors
async fn list_vendors(
    Extension(db): Extension<DatabaseConnection>,
    Extension(current_user): Extension<user::Model>,
) -> Result<impl IntoResponse, StatusCode> {
    let tenant_id = resolve_tenant_id(&db, current_user.id).await?;

    let vendors = crate::entities::atlas_service_provider::Entity::find()
        .filter(crate::entities::atlas_service_provider::Column::TenantId.eq(tenant_id))
        .all(&db)
        .await
        .map_err(|e| {
            tracing::error!(%tenant_id, "list_vendors error: {e:#}");
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    let summaries: Vec<VendorSummary> = vendors
        .into_iter()
        .map(|v| {
            let trade_type = v
                .profile_metadata
                .as_ref()
                .and_then(|m| m.get("trade_type"))
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());

            let is_emergency_available = v
                .profile_metadata
                .as_ref()
                .and_then(|m| m.get("is_emergency_available"))
                .and_then(|v| v.as_bool())
                .unwrap_or(false);

            let rating_avg = v.rating_avg.map(|r| {
                r.to_string().parse::<f64>().unwrap_or(0.0)
            });

            VendorSummary {
                id: v.id,
                business_name: v.business_name.unwrap_or_default(),
                trade_type,
                status: v.status,
                is_emergency_available,
                rating_avg,
                created_at: v.created_at,
            }
        })
        .collect();

    Ok(axum::response::Json(summaries))
}

/// POST /api/folio/vendors
async fn create_vendor(
    Extension(db): Extension<DatabaseConnection>,
    Extension(current_user): Extension<user::Model>,
    Json(input): Json<CreateVendorHttpInput>,
) -> Result<impl IntoResponse, StatusCode> {
    let tenant_id = resolve_tenant_id(&db, current_user.id).await?;

    let trade_type = TradeType::try_from(input.trade_type.clone()).map_err(|_| {
        tracing::warn!("create_vendor: invalid trade_type '{}'", input.trade_type);
        StatusCode::UNPROCESSABLE_ENTITY
    })?;

    let id = VendorService::onboard(
        &db,
        tenant_id,
        CreateVendorInput {
            user_id: input.user_id,
            business_name: input.business_name,
            trade_type,
            license_number: input.license_number,
            license_state: input.license_state,
            is_emergency_available: input.is_emergency_available,
            hourly_rate_cents: input.hourly_rate_cents,
            is_insured: input.is_insured,
            is_bonded: input.is_bonded,
        },
    )
    .await
    .map_err(|e| {
        tracing::error!(%tenant_id, "create_vendor error: {e:#}");
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    Ok((StatusCode::CREATED, axum::response::Json(CreateVendorResponse { id })))
}

/// PATCH /api/folio/vendors/:id/emergency
async fn toggle_emergency(
    Extension(db): Extension<DatabaseConnection>,
    Extension(current_user): Extension<user::Model>,
    Path(vendor_id): Path<Uuid>,
    Json(input): Json<ToggleEmergencyInput>,
) -> Result<impl IntoResponse, StatusCode> {
    let tenant_id = resolve_tenant_id(&db, current_user.id).await?;

    VendorService::set_emergency_available(&db, tenant_id, vendor_id, input.available)
        .await
        .map_err(|e| {
            let msg = e.to_string();
            if msg.contains("not found") {
                StatusCode::NOT_FOUND
            } else {
                tracing::error!(%tenant_id, %vendor_id, "toggle_emergency error: {e:#}");
                StatusCode::INTERNAL_SERVER_ERROR
            }
        })?;

    Ok(StatusCode::NO_CONTENT)
}

// ── Helpers ───────────────────────────────────────────────────────────────────

async fn resolve_tenant_id(db: &DatabaseConnection, user_id: Uuid) -> Result<Uuid, StatusCode> {
    let user_accounts = crate::entities::user_account::Entity::find()
        .filter(crate::entities::user_account::Column::UserId.eq(user_id))
        .all(db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let account_ids: Vec<Uuid> = user_accounts.into_iter().map(|ua| ua.account_id).collect();

    let profile = crate::entities::profile::Entity::find()
        .filter(crate::entities::profile::Column::AccountId.is_in(account_ids))
        .one(db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::FORBIDDEN)?;

    Ok(profile.tenant_id)
}
