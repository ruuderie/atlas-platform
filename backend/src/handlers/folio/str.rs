//! Folio — STR Compliance handler.
//!
//! # Route surface
//!
//! | Method | Path | Description |
//! |--------|------|-------------|
//! | POST | `/api/folio/str/permits` | Register an STR operating permit |
//! | GET  | `/api/folio/str/permits` | List all STR permits for the tenant |
//! | GET  | `/api/folio/str/permits/{id}` | Get a single STR permit |
//! | POST | `/api/folio/str/scan` | Trigger on-demand expiry scan (admin) |
//!
//! # Data source
//!
//! All data lives in `atlas_regulatory_registrations` (G-16).
//! `compliance_violation` cases live in `atlas_cases` (G-13).
//! No net-new tables.
//!
//! # Expiry warning
//!
//! The `pm_str_permit_expiry_scanner` background job runs daily.
//! `POST /api/folio/str/scan` allows landlords (admin role) to trigger
//! an immediate scan without waiting for the next scheduled run.

use axum::{
    extract::{Extension, Json, Path},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Router,
};
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, QueryOrder};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::entities::user;
use crate::services::pm::str_compliance::StrComplianceService;
use crate::types::pm::StrPermitCategory;

// ── Route registration ────────────────────────────────────────────────────────

pub fn authenticated_routes_raw() -> Router<DatabaseConnection> {
    Router::new()
        .route("/api/folio/str/permits", get(list_str_permits).post(register_str_permit))
        .route("/api/folio/str/permits/{id}", get(get_str_permit))
        .route("/api/folio/str/scan", post(trigger_expiry_scan))
}

// ── Request / response types ──────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
struct RegisterStrPermitInput {
    pub asset_id: Uuid,
    /// Typed category: "owner_occupied", "investor_non_owner", etc.
    pub permit_category: String,
    pub permit_number: String,
    /// ISO 8601 date: "2026-12-31"
    pub expires_at: chrono::NaiveDate,
    /// e.g. "US-FL-MIAMI-DADE"
    pub jurisdiction_code: String,
}

#[derive(Debug, Serialize)]
struct RegisterStrPermitResponse {
    pub id: Uuid,
}

#[derive(Debug, Serialize)]
struct StrPermitSummary {
    pub id: Uuid,
    pub asset_id: Option<Uuid>,
    pub permit_number: String,
    pub jurisdiction_code: String,
    pub status: String,
    pub expires_at: Option<chrono::NaiveDate>,
    pub permit_category: Option<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Deserialize)]
struct TriggerScanInput {
    /// Days before expiry to scan for. Defaults to 30.
    pub warning_days: Option<u32>,
}

#[derive(Debug, Serialize)]
struct TriggerScanResponse {
    pub cases_opened: u32,
    pub warning_days: u32,
}

// ── Handlers ──────────────────────────────────────────────────────────────────

/// POST /api/folio/str/permits
async fn register_str_permit(
    Extension(db): Extension<DatabaseConnection>,
    Extension(current_user): Extension<user::Model>,
    Json(input): Json<RegisterStrPermitInput>,
) -> Result<impl IntoResponse, StatusCode> {
    let tenant_id = resolve_tenant_id(&db, current_user.id).await?;

    let category = StrPermitCategory::try_from(input.permit_category.clone()).map_err(|_| {
        tracing::warn!(
            "register_str_permit: invalid permit_category '{}'",
            input.permit_category
        );
        StatusCode::UNPROCESSABLE_ENTITY
    })?;

    let id = StrComplianceService::register_permit(
        &db,
        tenant_id,
        input.asset_id,
        &input.permit_number,
        category,
        input.expires_at,
        &input.jurisdiction_code,
    )
    .await
    .map_err(|e| {
        tracing::error!(%tenant_id, "register_str_permit error: {e:#}");
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    Ok((
        StatusCode::CREATED,
        axum::response::Json(RegisterStrPermitResponse { id }),
    ))
}

/// GET /api/folio/str/permits
async fn list_str_permits(
    Extension(db): Extension<DatabaseConnection>,
    Extension(current_user): Extension<user::Model>,
) -> Result<impl IntoResponse, StatusCode> {
    use crate::types::pm::PmRegistrationType;

    let tenant_id = resolve_tenant_id(&db, current_user.id).await?;

    let permits = crate::entities::atlas_regulatory_registration::Entity::find()
        .filter(
            crate::entities::atlas_regulatory_registration::Column::TenantId.eq(tenant_id),
        )
        .filter(
            crate::entities::atlas_regulatory_registration::Column::RegistrationType
                .eq(PmRegistrationType::StrPermit.to_string()),
        )
        .order_by_desc(
            crate::entities::atlas_regulatory_registration::Column::CreatedAt,
        )
        .all(&db)
        .await
        .map_err(|e| {
            tracing::error!(%tenant_id, "list_str_permits DB error: {e:#}");
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    let summaries: Vec<StrPermitSummary> = permits
        .into_iter()
        .map(|p| StrPermitSummary {
            id: p.id,
            asset_id: p.asset_id,
            permit_number: p.registration_number,
            jurisdiction_code: p.jurisdiction_code,
            status: p.status,
            expires_at: p.expires_at,
            permit_category: p.jurisdiction_metadata
                .as_ref()
                .and_then(|m| m.get("permit_category"))
                .and_then(|v| v.as_str())
                .map(|s| s.to_string()),
            created_at: p.created_at,
        })
        .collect();

    Ok(axum::response::Json(summaries))
}

/// GET /api/folio/str/permits/{id}
async fn get_str_permit(
    Extension(db): Extension<DatabaseConnection>,
    Extension(current_user): Extension<user::Model>,
    Path(permit_id): Path<Uuid>,
) -> Result<impl IntoResponse, StatusCode> {
    let tenant_id = resolve_tenant_id(&db, current_user.id).await?;

    let permit = crate::entities::atlas_regulatory_registration::Entity::find_by_id(permit_id)
        .one(&db)
        .await
        .map_err(|e| {
            tracing::error!(%tenant_id, %permit_id, "get_str_permit DB error: {e:#}");
            StatusCode::INTERNAL_SERVER_ERROR
        })?
        .ok_or(StatusCode::NOT_FOUND)?;

    if permit.tenant_id != tenant_id {
        return Err(StatusCode::NOT_FOUND);
    }

    Ok(axum::response::Json(StrPermitSummary {
        id: permit.id,
        asset_id: permit.asset_id,
        permit_number: permit.registration_number,
        jurisdiction_code: permit.jurisdiction_code,
        status: permit.status,
        expires_at: permit.expires_at,
        permit_category: permit.jurisdiction_metadata
            .as_ref()
            .and_then(|m| m.get("permit_category"))
            .and_then(|v| v.as_str())
            .map(|s| s.to_string()),
        created_at: permit.created_at,
    }))
}

/// POST /api/folio/str/scan
///
/// Trigger an immediate STR permit expiry scan for the tenant.
/// Opens `compliance_violation` cases for any permits expiring within `warning_days`.
/// Idempotent: the underlying service uses a NOT EXISTS guard.
async fn trigger_expiry_scan(
    Extension(db): Extension<DatabaseConnection>,
    Extension(current_user): Extension<user::Model>,
    Json(input): Json<TriggerScanInput>,
) -> Result<impl IntoResponse, StatusCode> {
    let tenant_id = resolve_tenant_id(&db, current_user.id).await?;
    let warning_days = input.warning_days.unwrap_or(30);

    let cases_opened = StrComplianceService::scan_expiring_permits(&db, tenant_id, warning_days)
        .await
        .map_err(|e| {
            tracing::error!(%tenant_id, "trigger_expiry_scan error: {e:#}");
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    Ok(axum::response::Json(TriggerScanResponse {
        cases_opened,
        warning_days,
    }))
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
