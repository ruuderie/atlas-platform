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
//! POST /api/folio/inspections/{id}/complete
//!      Complete an inspection — records findings + rolls asset lifecycle forward.
//!      Body: CompleteInspectionInput
//!      -> 200 {}
//!
//! GET  /api/folio/assets/{asset_id}/inspections
//!      List all inspections (any status) for a specific asset.
//!      -> 200 [InspectionDetail]
//! ```

use axum::{
    Router,
    extract::{Extension, Json, Path, Query},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, patch, post},
};
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, QueryOrder, Set,
};
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
        .route(
            "/api/folio/maintenance",
            get(list_tickets).post(create_ticket),
        )
        .route(
            "/api/folio/maintenance/log-paid",
            post(log_paid_ticket),
        )
        .route(
            "/api/folio/maintenance/{id}",
            get(get_ticket).patch(patch_ticket),
        )
        .route(
            "/api/folio/maintenance/{id}/complete",
            post(complete_ticket),
        )
        // Inspection scheduling routes
        .route(
            "/api/folio/inspections",
            get(list_upcoming_inspections).post(schedule_inspection),
        )
        .route(
            "/api/folio/inspections/{id}/complete",
            post(complete_inspection),
        )
        .route(
            "/api/folio/assets/{asset_id}/inspections",
            get(list_inspections_for_asset),
        )
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

/// Folio-shaped response for `GET /api/folio/assets/{asset_id}/inspections`.
///
/// Extends `InspectionDetail` with a denormalized `assigned_vendor_name` so the
/// asset detail timeline can display the contractor without a second round-trip.
/// The name is resolved from `atlas_service_providers` by
/// `assigned_service_provider_id`; it is `None` when no contractor is assigned
/// or the provider record has been deleted.
#[derive(Debug, Serialize)]
pub struct AssetCaseSummary {
    pub id: Uuid,
    /// G-13 `case_type` string — `"maintenance"` or `"scheduled_inspection"` for Folio.
    pub case_type: String,
    /// User-authored title, e.g. `"Annual flush & anode rod"`.
    pub subject: String,
    pub status: String,
    pub priority: String,
    pub scheduled_at: Option<chrono::DateTime<chrono::Utc>>,
    pub completed_at: Option<chrono::DateTime<chrono::Utc>>,
    pub estimated_cost_cents: Option<i64>,
    pub actual_cost_cents: Option<i64>,
    /// Vendor name resolved from `atlas_service_providers` — `None` when unassigned.
    pub assigned_vendor_name: Option<String>,
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

#[derive(Debug, Serialize)]
struct MaintenanceDetail {
    pub id: Uuid,
    pub asset_id: Option<Uuid>,
    pub case_type: String,
    pub subject: String,
    pub description: Option<String>,
    pub status: String,
    pub priority: String,
    pub estimated_cost_cents: Option<i64>,
    pub actual_cost_cents: Option<i64>,
    pub assigned_service_provider_id: Option<Uuid>,
    pub assigned_user_id: Option<Uuid>,
    pub scheduled_at: Option<chrono::DateTime<chrono::Utc>>,
    pub completed_at: Option<chrono::DateTime<chrono::Utc>>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    /// Parent renovation project when linked via G-22 child_work_order.
    pub project_id: Option<Uuid>,
}

#[derive(Debug, Deserialize)]
struct CompleteTicketInput {
    actual_cost_cents: Option<i64>,
    note: Option<String>,
}

#[derive(Debug, Deserialize)]
struct PatchTicketInput {
    actual_cost_cents: Option<i64>,
    estimated_cost_cents: Option<i64>,
    description: Option<String>,
}

#[derive(Debug, Deserialize)]
struct LogPaidInput {
    asset_id: Uuid,
    subject: String,
    description: Option<String>,
    actual_cost_cents: i64,
    service_provider_id: Option<Uuid>,
    /// Optional renovation project to link via G-22.
    project_id: Option<Uuid>,
    /// When set, record cost on this existing work order instead of creating a new case.
    /// Must belong to the same tenant and preferably the same asset.
    pub related_case_id: Option<Uuid>,
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

#[derive(Debug, Deserialize)]
struct ListTicketsQuery {
    /// Scope picker/list to one unit or property.
    pub asset_id: Option<Uuid>,
}

/// GET /api/folio/maintenance?asset_id=
async fn list_tickets(
    Extension(db): Extension<DatabaseConnection>,
    Extension(current_user): Extension<user::Model>,
    Query(query): Query<ListTicketsQuery>,
) -> Result<impl IntoResponse, StatusCode> {
    use crate::types::pm::PmCaseType;

    let tenant_id = resolve_tenant_id(&db, current_user.id).await?;

    let mut finder = crate::entities::atlas_case::Entity::find()
        .filter(crate::entities::atlas_case::Column::TenantId.eq(tenant_id))
        .filter(
            crate::entities::atlas_case::Column::CaseType.eq(PmCaseType::Maintenance.to_string()),
        );
    if let Some(aid) = query.asset_id {
        finder = finder.filter(crate::entities::atlas_case::Column::AssetId.eq(aid));
    }

    let cases = finder
        .order_by_desc(crate::entities::atlas_case::Column::CreatedAt)
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
            reported_by_user_id: current_user.id,
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

    Ok((
        StatusCode::CREATED,
        axum::response::Json(CreateTicketResponse { id }),
    ))
}

/// GET /api/folio/maintenance/:id
async fn get_ticket(
    Extension(db): Extension<DatabaseConnection>,
    Extension(current_user): Extension<user::Model>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, StatusCode> {
    use crate::types::pm::{PmCaseType, PmRelationshipType};

    let tenant_id = resolve_tenant_id(&db, current_user.id).await?;
    let c = crate::entities::atlas_case::Entity::find_by_id(id)
        .one(&db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;
    if c.tenant_id != tenant_id {
        return Err(StatusCode::NOT_FOUND);
    }

    let sources = crate::services::pm::record_relationship::RecordRelationshipService::find_sources(
        &db,
        tenant_id,
        "atlas_case",
        id,
        &PmRelationshipType::ChildWorkOrder.to_string(),
    )
    .await
    .unwrap_or_default();

    let mut project_id = None;
    for r in sources {
        if let Some(p) = crate::entities::atlas_case::Entity::find_by_id(r.source_entity_id)
            .one(&db)
            .await
            .ok()
            .flatten()
        {
            if p.case_type == PmCaseType::RenovationProject.to_string() {
                project_id = Some(p.id);
                break;
            }
        }
    }

    Ok(axum::response::Json(MaintenanceDetail {
        id: c.id,
        asset_id: c.asset_id,
        case_type: c.case_type,
        subject: c.subject,
        description: c.description,
        status: c.status,
        priority: c.priority,
        estimated_cost_cents: c.estimated_cost_cents,
        actual_cost_cents: c.actual_cost_cents,
        assigned_service_provider_id: c.assigned_service_provider_id,
        assigned_user_id: c.assigned_user_id,
        scheduled_at: c.scheduled_at,
        completed_at: c.completed_at,
        created_at: c.created_at,
        project_id,
    }))
}

/// PATCH /api/folio/maintenance/:id
async fn patch_ticket(
    Extension(db): Extension<DatabaseConnection>,
    Extension(current_user): Extension<user::Model>,
    Path(id): Path<Uuid>,
    Json(input): Json<PatchTicketInput>,
) -> Result<impl IntoResponse, StatusCode> {
    let tenant_id = resolve_tenant_id(&db, current_user.id).await?;
    let c = crate::entities::atlas_case::Entity::find_by_id(id)
        .one(&db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;
    if c.tenant_id != tenant_id {
        return Err(StatusCode::NOT_FOUND);
    }
    let mut am: crate::entities::atlas_case::ActiveModel = c.into();
    if let Some(v) = input.actual_cost_cents {
        am.actual_cost_cents = Set(Some(v));
    }
    if let Some(v) = input.estimated_cost_cents {
        am.estimated_cost_cents = Set(Some(v));
    }
    if let Some(d) = input.description {
        am.description = Set(Some(d));
    }
    am.update(&db).await.map_err(|e| {
        tracing::error!(%tenant_id, %id, "patch_ticket: {e:#}");
        StatusCode::INTERNAL_SERVER_ERROR
    })?;
    Ok(StatusCode::NO_CONTENT)
}

/// POST /api/folio/maintenance/:id/complete
async fn complete_ticket(
    Extension(db): Extension<DatabaseConnection>,
    Extension(current_user): Extension<user::Model>,
    Path(id): Path<Uuid>,
    Json(input): Json<CompleteTicketInput>,
) -> Result<impl IntoResponse, StatusCode> {
    let tenant_id = resolve_tenant_id(&db, current_user.id).await?;
    let c = crate::entities::atlas_case::Entity::find_by_id(id)
        .one(&db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;
    if c.tenant_id != tenant_id {
        return Err(StatusCode::NOT_FOUND);
    }

    let mut am: crate::entities::atlas_case::ActiveModel = c.clone().into();
    am.status = Set("closed".into());
    am.completed_at = Set(Some(chrono::Utc::now()));
    if let Some(cost) = input.actual_cost_cents {
        am.actual_cost_cents = Set(Some(cost));
    }
    if c.assigned_user_id.is_none() {
        am.assigned_user_id = Set(Some(current_user.id));
    }
    if let Some(note) = input.note {
        let mut meta = c.case_metadata.clone().unwrap_or_else(|| serde_json::json!({}));
        if let Some(obj) = meta.as_object_mut() {
            obj.insert(
                "landlord_complete_note".into(),
                serde_json::Value::String(note),
            );
        }
        am.case_metadata = Set(Some(meta));
    }
    let updated = am.update(&db).await.map_err(|e| {
        tracing::error!(%tenant_id, %id, "complete_ticket: {e:#}");
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    if let Err(e) = trigger_landlord_case_resolved(&db, &updated).await {
        tracing::warn!(case_id = %updated.id, "landlord case_resolved: {e:#}");
    }

    Ok(axum::response::Json(serde_json::json!({
        "id": updated.id,
        "status": updated.status,
        "completed_at": updated.completed_at,
    })))
}

/// POST /api/folio/maintenance/log-paid
///
/// Two modes:
/// - **Linked:** `related_case_id` set → attach cost to that work order (no new case).
/// - **Standalone:** create a closed off-platform expense case on the asset.
async fn log_paid_ticket(
    Extension(db): Extension<DatabaseConnection>,
    Extension(current_user): Extension<user::Model>,
    Json(input): Json<LogPaidInput>,
) -> Result<impl IntoResponse, StatusCode> {
    use crate::types::pm::{PmCaseType, PmRelationshipType};

    let tenant_id = resolve_tenant_id(&db, current_user.id).await?;
    if input.actual_cost_cents < 0 {
        return Err(StatusCode::UNPROCESSABLE_ENTITY);
    }

    // ── Link expense to an existing work order ───────────────────────────────
    if let Some(case_id) = input.related_case_id {
        let c = crate::entities::atlas_case::Entity::find_by_id(case_id)
            .one(&db)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
            .ok_or(StatusCode::NOT_FOUND)?;
        if c.tenant_id != tenant_id {
            return Err(StatusCode::NOT_FOUND);
        }
        if c.asset_id.is_some() && c.asset_id != Some(input.asset_id) {
            tracing::warn!(
                %tenant_id,
                %case_id,
                case_asset = ?c.asset_id,
                input_asset = %input.asset_id,
                "log_paid: related_case_id asset mismatch"
            );
            return Err(StatusCode::UNPROCESSABLE_ENTITY);
        }

        let mut meta = c.case_metadata.clone().unwrap_or_else(|| serde_json::json!({}));
        if let Some(obj) = meta.as_object_mut() {
            obj.insert("expense_linked".into(), serde_json::json!(true));
            if let Some(d) = &input.description {
                obj.insert("expense_note".into(), serde_json::Value::String(d.clone()));
            }
            if !input.subject.trim().is_empty() {
                obj.insert(
                    "expense_label".into(),
                    serde_json::Value::String(input.subject.trim().to_string()),
                );
            }
        }

        let mut am: crate::entities::atlas_case::ActiveModel = c.into();
        am.actual_cost_cents = Set(Some(input.actual_cost_cents));
        am.case_metadata = Set(Some(meta));
        if let Some(sp) = input.service_provider_id {
            am.assigned_service_provider_id = Set(Some(sp));
        }
        am.update(&db).await.map_err(|e| {
            tracing::error!(%tenant_id, %case_id, "log_paid link update: {e:#}");
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

        return Ok((
            StatusCode::OK,
            axum::response::Json(CreateTicketResponse { id: case_id }),
        ));
    }

    // ── Standalone historical / off-platform expense ─────────────────────────
    if input.subject.trim().is_empty() {
        return Err(StatusCode::UNPROCESSABLE_ENTITY);
    }

    let id = Uuid::new_v4();
    let am = crate::entities::atlas_case::ActiveModel {
        id: Set(id),
        tenant_id: Set(tenant_id),
        asset_id: Set(Some(input.asset_id)),
        case_type: Set(PmCaseType::Maintenance.to_string()),
        subject: Set(input.subject.trim().to_string()),
        description: Set(input.description),
        status: Set("closed".into()),
        priority: Set("normal".into()),
        actual_cost_cents: Set(Some(input.actual_cost_cents)),
        assigned_service_provider_id: Set(input.service_provider_id),
        assigned_user_id: Set(Some(current_user.id)),
        completed_at: Set(Some(chrono::Utc::now())),
        case_metadata: Set(Some(serde_json::json!({ "ticket_source": "off_platform" }))),
        created_at: Set(chrono::Utc::now()),
        ..Default::default()
    };
    let created = am.insert(&db).await.map_err(|e| {
        tracing::error!(%tenant_id, "log_paid insert: {e:#}");
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    if let Some(project_id) = input.project_id {
        let _ = crate::services::pm::record_relationship::RecordRelationshipService::create(
            &db,
            tenant_id,
            crate::services::pm::record_relationship::CreateRelationshipPayload {
                source_entity_type: "atlas_case".into(),
                source_entity_id: project_id,
                target_entity_type: "atlas_case".into(),
                target_entity_id: id,
                relationship_type: PmRelationshipType::ChildWorkOrder.to_string(),
                inverse_label: Some("parent_project".into()),
                relationship_metadata: None,
                created_by_user_id: Some(current_user.id),
            },
        )
        .await;
    }

    if let Err(e) = trigger_landlord_case_resolved(&db, &created).await {
        tracing::warn!(case_id = %created.id, "log_paid case_resolved: {e:#}");
    }

    Ok((
        StatusCode::CREATED,
        axum::response::Json(CreateTicketResponse { id }),
    ))
}

async fn trigger_landlord_case_resolved(
    db: &DatabaseConnection,
    case: &crate::entities::atlas_case::Model,
) -> anyhow::Result<()> {
    let Some(provider_id) = case.assigned_service_provider_id else {
        return Ok(());
    };
    let Some(rater_user_id) = case.assigned_user_id else {
        return Ok(());
    };
    let app_instance_id = crate::entities::app_instance::Entity::find()
        .filter(crate::entities::app_instance::Column::TenantId.eq(case.tenant_id))
        .filter(crate::entities::app_instance::Column::AppType.eq("property_management"))
        .order_by_asc(crate::entities::app_instance::Column::CreatedAt)
        .one(db)
        .await?
        .map(|i| i.id);
    let Some(app_instance_id) = app_instance_id else {
        return Ok(());
    };
    let opened = crate::services::scorecard_triggers::on_case_resolved(
        db,
        case.tenant_id,
        app_instance_id,
        case.id,
        provider_id,
        rater_user_id,
    )
    .await?;
    tracing::info!(
        case_id = %case.id,
        sessions = opened.len(),
        "landlord case_resolved: sessions opened"
    );
    Ok(())
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
    Ok((
        StatusCode::CREATED,
        axum::response::Json(serde_json::json!({ "id": id })),
    ))
}

/// POST /api/folio/inspections/{id}/complete
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

/// GET /api/folio/assets/{asset_id}/inspections
///
/// Returns all G-13 cases for this asset (`maintenance` + `scheduled_inspection`)
/// with vendor name denormalized. Sorted by `scheduled_at` descending so the
/// most recent activity appears first in the timeline.
async fn list_inspections_for_asset(
    Extension(db): Extension<DatabaseConnection>,
    Extension(current_user): Extension<user::Model>,
    Path(asset_id): Path<Uuid>,
) -> Result<impl IntoResponse, StatusCode> {
    use crate::types::pm::PmCaseType;

    let tenant_id = resolve_tenant_id(&db, current_user.id).await?;

    // Fetch all maintenance + inspection cases for this asset.
    let cases = crate::entities::atlas_case::Entity::find()
        .filter(crate::entities::atlas_case::Column::TenantId.eq(tenant_id))
        .filter(crate::entities::atlas_case::Column::AssetId.eq(asset_id))
        .filter(
            sea_orm::Condition::any()
                .add(
                    crate::entities::atlas_case::Column::CaseType
                        .eq(PmCaseType::Maintenance.to_string()),
                )
                .add(
                    crate::entities::atlas_case::Column::CaseType
                        .eq(PmCaseType::ScheduledInspection.to_string()),
                ),
        )
        .order_by_desc(crate::entities::atlas_case::Column::ScheduledAt)
        .all(&db)
        .await
        .map_err(|e| {
            tracing::error!(%tenant_id, %asset_id, "list_inspections_for_asset error: {e:#}");
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    let cases: Vec<crate::entities::atlas_case::Model> = cases;

    // Denormalize vendor names: collect unique service_provider_ids, batch fetch.
    let provider_ids: Vec<Uuid> = cases
        .iter()
        .filter_map(|c| c.assigned_service_provider_id)
        .collect::<std::collections::HashSet<_>>()
        .into_iter()
        .collect();

    let providers = if provider_ids.is_empty() {
        vec![]
    } else {
        crate::entities::atlas_service_provider::Entity::find()
            .filter(crate::entities::atlas_service_provider::Column::Id.is_in(provider_ids))
            .all(&db)
            .await
            .map_err(|e| {
                tracing::error!(%tenant_id, "list_inspections_for_asset: vendor lookup error: {e:#}");
                StatusCode::INTERNAL_SERVER_ERROR
            })?
    };

    // Build a lookup map id → business_name.
    let vendor_map: std::collections::HashMap<Uuid, Option<String>> = providers
        .into_iter()
        .map(|p| (p.id, p.business_name))
        .collect();

    let summaries: Vec<AssetCaseSummary> = cases
        .into_iter()
        .map(|c| {
            let vendor_name = c
                .assigned_service_provider_id
                .and_then(|id| vendor_map.get(&id))
                .and_then(|name| name.clone());
            AssetCaseSummary {
                id: c.id,
                case_type: c.case_type,
                subject: c.subject,
                status: c.status,
                priority: c.priority,
                scheduled_at: c.scheduled_at,
                completed_at: c.completed_at,
                estimated_cost_cents: c.estimated_cost_cents,
                actual_cost_cents: c.actual_cost_cents,
                assigned_vendor_name: vendor_name,
                created_at: c.created_at,
            }
        })
        .collect();

    Ok(axum::response::Json(summaries))
}

// ── Helpers ───────────────────────────────────────────────────────────────────

async fn resolve_tenant_id(db: &DatabaseConnection, user_id: Uuid) -> Result<Uuid, StatusCode> {
    crate::extractors::tenant::resolve_tenant_id(db, user_id).await
}
