//! Folio Vendor — Work Orders handler.
//!
//! Work orders are `atlas_cases` rows where:
//!   - `case_type = 'maintenance'`
//!   - `assigned_service_provider_id` matches the vendor's service_provider record
//!
//! # Authorization
//! All routes use the `VendorOnly` extractor — role is checked declaratively
//! before the handler body executes, with no manual `ensure_vendor_role()` call.
//!
//! # Routes
//!
//! ```ignore
//! GET  /api/folio/vendor/work-orders
//!      List work orders assigned to this vendor. ?status=open|in_progress|completed|all
//!      -> 200 [WorkOrderSummary]
//!
//! GET  /api/folio/vendor/work-orders/:id
//!      Work order detail + cost breakdown.
//!      -> 200 WorkOrderDetail
//!
//! POST /api/folio/vendor/work-orders/:id/complete
//!      Mark complete, record actual_cost_cents, optional completion note.
//!      -> 200 { "id": uuid, "status": "completed" }
//! ```

use axum::{
    extract::{Extension, Json, Path, Query},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Router,
};
use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, DatabaseConnection,
    EntityTrait, QueryFilter, QueryOrder,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::entities::{atlas_case, atlas_service_provider, user};
use crate::extractors::folio_role::VendorOnly;

// ── Route constructors ────────────────────────────────────────────────────────

pub fn authenticated_routes_raw() -> Router<DatabaseConnection> {
    Router::new()
        .route("/api/folio/vendor/work-orders",              get(list_work_orders))
        .route("/api/folio/vendor/work-orders/:id",          get(get_work_order))
        .route("/api/folio/vendor/work-orders/:id/complete", post(complete_work_order))
}

// ── Request / Response types ──────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct ListWorkOrdersQuery {
    #[serde(default = "default_status_filter")]
    pub status: String,
}
fn default_status_filter() -> String { "active".to_string() }

#[derive(Debug, Serialize)]
pub struct WorkOrderSummary {
    pub id:             Uuid,
    pub subject:        String,
    pub priority:       String,
    pub status:         String,
    pub scheduled_at:   Option<chrono::DateTime<chrono::Utc>>,
    pub estimated_cost: Option<i64>,
    pub asset_id:       Option<Uuid>,
}

#[derive(Debug, Serialize)]
pub struct WorkOrderDetail {
    pub id:                   Uuid,
    pub subject:              String,
    pub description:          Option<String>,
    pub priority:             String,
    pub status:               String,
    pub scheduled_at:         Option<chrono::DateTime<chrono::Utc>>,
    pub completed_at:         Option<chrono::DateTime<chrono::Utc>>,
    pub estimated_cost_cents: Option<i64>,
    pub actual_cost_cents:    Option<i64>,
    pub asset_id:             Option<Uuid>,
    pub contract_id:          Option<Uuid>,
    pub ledger_entry_id:      Option<Uuid>,
    pub metadata:             Option<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
pub struct CompleteWorkOrderInput {
    pub actual_cost_cents: Option<i64>,
    pub completion_note:   Option<String>,
}

// ── Handlers ──────────────────────────────────────────────────────────────────

/// GET /api/folio/vendor/work-orders
/// `VendorOnly` extractor enforces role — handler only runs for verified vendors.
async fn list_work_orders(
    _guard: VendorOnly,                                // ← declarative role gate
    Extension(db): Extension<DatabaseConnection>,
    Extension(current_user): Extension<user::Model>,
    Query(params): Query<ListWorkOrdersQuery>,
) -> Result<impl IntoResponse, StatusCode> {
    let (tenant_id, sp) = resolve_vendor_context(&db, current_user.id).await?;

    let mut query = atlas_case::Entity::find()
        .filter(atlas_case::Column::TenantId.eq(tenant_id))
        .filter(atlas_case::Column::CaseType.eq("maintenance"))
        .filter(atlas_case::Column::AssignedServiceProviderId.eq(sp.id));

    match params.status.as_str() {
        "open"        => { query = query.filter(atlas_case::Column::Status.eq("open")); }
        "in_progress" => { query = query.filter(atlas_case::Column::Status.eq("in_progress")); }
        "completed"   => { query = query.filter(atlas_case::Column::Status.eq("completed")); }
        "all"         => {}
        _ => {
            query = query.filter(
                sea_orm::Condition::any()
                    .add(atlas_case::Column::Status.eq("open"))
                    .add(atlas_case::Column::Status.eq("in_progress"))
            );
        }
    }

    let cases = query
        .order_by_asc(atlas_case::Column::ScheduledAt)
        .all(&db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let summaries: Vec<WorkOrderSummary> = cases
        .into_iter()
        .map(|c| WorkOrderSummary {
            id:             c.id,
            subject:        c.subject,
            priority:       c.priority,
            status:         c.status,
            scheduled_at:   c.scheduled_at,
            estimated_cost: c.estimated_cost_cents,
            asset_id:       c.asset_id,
        })
        .collect();

    Ok(Json(summaries))
}

/// GET /api/folio/vendor/work-orders/:id
async fn get_work_order(
    _guard: VendorOnly,
    Extension(db): Extension<DatabaseConnection>,
    Extension(current_user): Extension<user::Model>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, StatusCode> {
    let (tenant_id, sp) = resolve_vendor_context(&db, current_user.id).await?;

    let case = atlas_case::Entity::find_by_id(id)
        .filter(atlas_case::Column::TenantId.eq(tenant_id))
        .filter(atlas_case::Column::AssignedServiceProviderId.eq(sp.id))
        .one(&db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;

    Ok(Json(WorkOrderDetail {
        id:                   case.id,
        subject:              case.subject,
        description:          case.description,
        priority:             case.priority,
        status:               case.status,
        scheduled_at:         case.scheduled_at,
        completed_at:         case.completed_at,
        estimated_cost_cents: case.estimated_cost_cents,
        actual_cost_cents:    case.actual_cost_cents,
        asset_id:             case.asset_id,
        contract_id:          case.contract_id,
        ledger_entry_id:      case.ledger_entry_id,
        metadata:             case.case_metadata,
    }))
}

/// POST /api/folio/vendor/work-orders/:id/complete
async fn complete_work_order(
    _guard: VendorOnly,
    Extension(db): Extension<DatabaseConnection>,
    Extension(current_user): Extension<user::Model>,
    Path(id): Path<Uuid>,
    Json(input): Json<CompleteWorkOrderInput>,
) -> Result<impl IntoResponse, StatusCode> {
    let (tenant_id, sp) = resolve_vendor_context(&db, current_user.id).await?;

    let case = atlas_case::Entity::find_by_id(id)
        .filter(atlas_case::Column::TenantId.eq(tenant_id))
        .filter(atlas_case::Column::AssignedServiceProviderId.eq(sp.id))
        .one(&db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;

    if case.status == "completed" {
        return Ok((StatusCode::OK, Json(serde_json::json!({
            "id": case.id, "status": "completed", "note": "already completed"
        }))));
    }

    let mut active: atlas_case::ActiveModel = case.into();
    active.status       = Set("completed".to_string());
    active.completed_at = Set(Some(chrono::Utc::now()));

    if let Some(cost) = input.actual_cost_cents {
        active.actual_cost_cents = Set(Some(cost));
    }
    if let Some(note) = input.completion_note {
        active.case_metadata = Set(Some(serde_json::json!({ "completion_note": note })));
    }

    let updated = active.update(&db).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok((StatusCode::OK, Json(serde_json::json!({
        "id": updated.id,
        "status": updated.status,
        "completed_at": updated.completed_at,
    }))))
}

// ── Shared helpers ────────────────────────────────────────────────────────────

/// Resolve (tenant_id, service_provider) for the current vendor user.
/// Returns 403 if no active service_provider record is found.
async fn resolve_vendor_context(
    db:      &DatabaseConnection,
    user_id: Uuid,
) -> Result<(Uuid, atlas_service_provider::Model), StatusCode> {
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

    let tenant_id = profile.tenant_id;

    let sp = atlas_service_provider::Entity::find()
        .filter(atlas_service_provider::Column::TenantId.eq(tenant_id))
        .filter(atlas_service_provider::Column::UserId.eq(user_id))
        .filter(atlas_service_provider::Column::Status.ne("inactive"))
        .one(db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::FORBIDDEN)?;

    Ok((tenant_id, sp))
}
