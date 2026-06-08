//! Folio — Maintenance handler.
//!
//! # Routes
//!
//! ```ignore
//! GET  /api/folio/maintenance
//!      List all open maintenance cases for the tenant.
//!      -> 200 [MaintenanceSummary]
//!
//! POST /api/folio/maintenance
//!      File a new maintenance ticket (standard or emergency).
//!      Body: CreateTicketInput
//!      -> 201 { "id": uuid }
//!
//! Emergency tickets (`is_emergency: true`) are logged at WARN level and
//! bypass standard scheduling — Phase 4 will queue immediate dispatch.
//! ```

use axum::{
    extract::{Extension, Json},
    http::StatusCode,
    response::IntoResponse,
    routing::get,
    Router,
};
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::entities::user;
use crate::services::pm::maintenance::{CreateMaintenanceTicketInput, MaintenanceService};
use crate::types::pm::MaintenanceCategory;

// ── Route registration ────────────────────────────────────────────────────────

pub fn authenticated_routes_raw() -> Router<DatabaseConnection> {
    Router::new()
        .route("/api/folio/maintenance", get(list_tickets).post(create_ticket))
}

// ── Request / response types ──────────────────────────────────────────────────

#[derive(Debug, Serialize)]
struct MaintenanceSummary {
    pub id: Uuid,
    pub asset_id: Option<Uuid>,
    pub case_type: String,
    pub subject: String,
    pub status: String,
    pub priority: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreateTicketHttpInput {
    pub asset_id: Uuid,
    pub reported_by_user_id: Uuid,
    pub category: String,
    pub description: String,
    pub is_emergency: bool,
    pub voice_note_r2_key: Option<String>,
}

#[derive(Debug, Serialize)]
struct CreateTicketResponse {
    pub id: Uuid,
}

// ── Handlers ──────────────────────────────────────────────────────────────────

/// GET /api/folio/maintenance
async fn list_tickets(
    Extension(db): Extension<DatabaseConnection>,
    Extension(current_user): Extension<user::Model>,
) -> Result<impl IntoResponse, StatusCode> {
    use crate::types::pm::PmCaseType;

    let tenant_id = resolve_tenant_id(&db, current_user.id).await?;

    let cases = crate::entities::atlas_case::Entity::find()
        .filter(crate::entities::atlas_case::Column::TenantId.eq(tenant_id))
        .filter(crate::entities::atlas_case::Column::CaseType.eq(PmCaseType::Maintenance.to_string()))
        .all(&db)
        .await
        .map_err(|e| {
            tracing::error!(%tenant_id, "list_maintenance_tickets error: {e:#}");
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    let summaries: Vec<MaintenanceSummary> = cases
        .into_iter()
        .map(|c| MaintenanceSummary {
            id: c.id,
            asset_id: c.asset_id,
            case_type: c.case_type,
            subject: c.subject,
            status: c.status,
            priority: c.priority,
            created_at: c.created_at,
        })
        .collect();

    Ok(axum::response::Json(summaries))
}

/// POST /api/folio/maintenance
async fn create_ticket(
    Extension(db): Extension<DatabaseConnection>,
    Extension(current_user): Extension<user::Model>,
    Json(input): Json<CreateTicketHttpInput>,
) -> Result<impl IntoResponse, StatusCode> {
    let tenant_id = resolve_tenant_id(&db, current_user.id).await?;

    let category = MaintenanceCategory::try_from(input.category.clone()).map_err(|_| {
        tracing::warn!("create_ticket: invalid category '{}'", input.category);
        StatusCode::UNPROCESSABLE_ENTITY
    })?;

    let id = MaintenanceService::create_ticket(
        &db,
        tenant_id,
        CreateMaintenanceTicketInput {
            asset_id: input.asset_id,
            reported_by_user_id: input.reported_by_user_id,
            category,
            description: input.description,
            is_emergency: input.is_emergency,
            voice_note_r2_key: input.voice_note_r2_key,
        },
    )
    .await
    .map_err(|e| {
        tracing::error!(%tenant_id, "create_ticket error: {e:#}");
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    Ok((StatusCode::CREATED, axum::response::Json(CreateTicketResponse { id })))
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
