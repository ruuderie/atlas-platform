//! Folio — Maintenance handler.
//!
//! # Routes
//!
//! ```ignore
//! GET  /api/folio/maintenance
//!      List all open reactive maintenance cases.
//!      -> 200 [MaintenanceSummary]
//!
//! POST /api/folio/maintenance
//!      File a reactive maintenance ticket (standard or emergency).
//!      Body: CreateTicketInput
//!      -> 201 { "id": uuid }
//!
//! GET  /api/folio/inspections
//!      List all upcoming scheduled inspections (status = "scheduled").
//!      -> 200 [InspectionDetail]
//!
//! POST /api/folio/inspections
//!      Schedule a proactive inspection on any lifecycle asset.
//!      Body: ScheduleInspectionHttpInput
//!      -> 201 { "id": uuid }
//!
//! POST /api/folio/inspections/:id/complete
//!      Complete an inspection — records findings + rolls asset lifecycle forward.
//!      Body: CompleteInspectionInput
//!      -> 200 {}
//!
//! GET  /api/folio/assets/:asset_id/inspections
//!      List all inspections (any status) for a specific asset.
//!      -> 200 [InspectionDetail]
//! ```

use axum::{
    extract::{Extension, Json, Path},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Router,
};
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::entities::user;
use crate::services::pm::maintenance::{
    CompleteInspectionInput, CreateMaintenanceTicketInput, MaintenanceService,
    ScheduleInspectionInput,
};
use crate::types::pm::MaintenanceCategory;

// ── Route registration ────────────────────────────────────────────────────────

pub fn authenticated_routes_raw() -> Router<DatabaseConnection> {
    Router::new()
        .route("/api/folio/maintenance", get(list_tickets).post(create_ticket))
        // Inspection scheduling routes
        .route("/api/folio/inspections", get(list_upcoming_inspections).post(schedule_inspection))
        .route("/api/folio/inspections/:id/complete", post(complete_inspection))
        .route("/api/folio/assets/:asset_id/inspections", get(list_inspections_for_asset))
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

/// HTTP input for scheduling an inspection (maps to ScheduleInspectionInput).
#[derive(Debug, Deserialize)]
pub struct ScheduleInspectionHttpInput {
    pub asset_id: Uuid,
    pub subject: String,
    pub notes: Option<String>,
    /// ISO 8601 DateTime string e.g. "2026-09-15T09:00:00Z".
    pub scheduled_at: chrono::DateTime<chrono::Utc>,
    pub service_provider_id: Option<Uuid>,
    pub assigned_user_id: Option<Uuid>,
    pub estimated_cost_cents: Option<i64>,
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


/// GET /api/folio/inspections
async fn list_upcoming_inspections(
    Extension(db): Extension<DatabaseConnection>,
    Extension(current_user): Extension<user::Model>,
) -> Result<impl IntoResponse, StatusCode> {
    let tenant_id = resolve_tenant_id(&db, current_user.id).await?;
    let inspections = MaintenanceService::list_upcoming_inspections(&db, tenant_id)
        .await
        .map_err(|e| {
            tracing::error!(%tenant_id, "list_upcoming_inspections error: {e:#}");
            StatusCode::INTERNAL_SERVER_ERROR
        })?;
    Ok(axum::response::Json(inspections))
}

/// POST /api/folio/inspections
async fn schedule_inspection(
    Extension(db): Extension<DatabaseConnection>,
    Extension(current_user): Extension<user::Model>,
    Json(input): Json<ScheduleInspectionHttpInput>,
) -> Result<impl IntoResponse, StatusCode> {
    let tenant_id = resolve_tenant_id(&db, current_user.id).await?;
    let id = MaintenanceService::schedule_inspection(
        &db,
        tenant_id,
        ScheduleInspectionInput {
            asset_id: input.asset_id,
            subject: input.subject,
            notes: input.notes,
            scheduled_at: input.scheduled_at,
            service_provider_id: input.service_provider_id,
            assigned_user_id: input.assigned_user_id,
            estimated_cost_cents: input.estimated_cost_cents,
        },
    )
    .await
    .map_err(|e| {
        tracing::error!(%tenant_id, "schedule_inspection error: {e:#}");
        StatusCode::INTERNAL_SERVER_ERROR
    })?;
    Ok((StatusCode::CREATED, axum::response::Json(serde_json::json!({ "id": id }))))
}

/// POST /api/folio/inspections/:id/complete
async fn complete_inspection(
    Extension(db): Extension<DatabaseConnection>,
    Extension(current_user): Extension<user::Model>,
    Path(case_id): Path<Uuid>,
    Json(mut input): Json<CompleteInspectionInput>,
) -> Result<impl IntoResponse, StatusCode> {
    let tenant_id = resolve_tenant_id(&db, current_user.id).await?;
    input.case_id = case_id;
    MaintenanceService::complete_inspection(&db, tenant_id, input)
        .await
        .map_err(|e| {
            let msg = e.to_string();
            if msg.contains("not found") {
                StatusCode::NOT_FOUND
            } else if msg.contains("already completed") {
                StatusCode::CONFLICT
            } else {
                tracing::error!(%tenant_id, "complete_inspection error: {e:#}");
                StatusCode::INTERNAL_SERVER_ERROR
            }
        })?;
    Ok(StatusCode::OK)
}

/// GET /api/folio/assets/:asset_id/inspections
async fn list_inspections_for_asset(
    Extension(db): Extension<DatabaseConnection>,
    Extension(current_user): Extension<user::Model>,
    Path(asset_id): Path<Uuid>,
) -> Result<impl IntoResponse, StatusCode> {
    let tenant_id = resolve_tenant_id(&db, current_user.id).await?;
    let inspections = MaintenanceService::list_inspections_for_asset(&db, tenant_id, asset_id)
        .await
        .map_err(|e| {
            tracing::error!(%tenant_id, %asset_id, "list_inspections_for_asset error: {e:#}");
            StatusCode::INTERNAL_SERVER_ERROR
        })?;
    Ok(axum::response::Json(inspections))
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
