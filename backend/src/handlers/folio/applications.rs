//! Folio — Renter Application handler.
//!
//! # Route surface
//!
//! | Method | Path | Description |
//! |--------|------|-------------|
//! | POST | `/api/folio/applications` | Submit a rental application |
//! | GET  | `/api/folio/applications` | List applications for the tenant (landlord view) |
//! | GET  | `/api/folio/applications/{id}` | Get a single application |
//! | PATCH | `/api/folio/applications/{id}/decision` | Approve or deny an application |
//!
//! # FHA compliance
//!
//! `FairHousingFilter` is applied at the **service layer** (`ApplicationService::submit_full`).
//! Protected characteristics (age, race, religion, familial status, etc.) are never
//! stored or surfaced for US/VI applicants. The handler does not need to know about
//! the FHA — the service enforces the invariant structurally.
//!
//! # Credit bureau routing
//!
//! Determined by the asset's jurisdiction (resolved via `folio_jurisdiction_code`
//! tenant setting). The service embeds `screening_provider` in the application row.
//!
//! # Data source
//!
//! `atlas_applications` (G-18). No net-new tables.

use axum::{
    extract::{Extension, Json, Path},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, patch},
    Router,
};
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, QueryOrder, Set};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::entities::user;
use crate::services::pm::applications::ApplicationService;
use crate::types::pm::Jurisdiction;

// ── Route registration ────────────────────────────────────────────────────────

pub fn authenticated_routes_raw() -> Router<DatabaseConnection> {
    Router::new()
        .route(
            "/api/folio/applications",
            get(list_applications).post(submit_application),
        )
        .route("/api/folio/applications/{id}", get(get_application))
        .route(
            "/api/folio/applications/{id}/decision",
            patch(decide_application),
        )
}

// ── Request / response types ──────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
struct SubmitApplicationInput {
    /// The unit (atlas_assets.id) being applied for.
    pub asset_id: Uuid,
    /// The applicant's user_account.id.
    pub applicant_user_id: Uuid,
    /// ISO jurisdiction code: "US", "BR", "VI".
    /// Determines FHA applicability and credit bureau routing.
    pub jurisdiction: String,
    /// Self-reported monthly income in cents. Optional.
    pub monthly_income_cents: Option<i64>,
}

#[derive(Debug, Serialize)]
struct SubmitApplicationResponse {
    pub id: Uuid,
    pub screening_status: &'static str,
}

#[derive(Debug, Serialize)]
struct ApplicationSummary {
    pub id: Uuid,
    pub applicant_user_id: Uuid,
    pub target_asset_id: Option<Uuid>,
    pub status: String,
    pub screening_status: String,
    pub screening_provider: Option<String>,
    pub screening_passed: Option<bool>,
    pub monthly_income_cents: Option<i64>,
    pub submitted_at: Option<chrono::DateTime<chrono::Utc>>,
    pub decided_at: Option<chrono::DateTime<chrono::Utc>>,
    pub decision_reason: Option<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Deserialize)]
struct DecideApplicationInput {
    /// "approved" or "denied"
    pub decision: String,
    /// Human-readable reason (required for denials, FHA best practice).
    pub reason: Option<String>,
}

// ── Handlers ──────────────────────────────────────────────────────────────────

/// POST /api/folio/applications
///
/// Submit a rental application. FHA filter is applied at the service layer.
async fn submit_application(
    Extension(db): Extension<DatabaseConnection>,
    Extension(current_user): Extension<user::Model>,
    Json(input): Json<SubmitApplicationInput>,
) -> Result<impl IntoResponse, StatusCode> {
    let tenant_id = resolve_tenant_id(&db, current_user.id).await?;

    let jurisdiction = Jurisdiction::try_from(input.jurisdiction.clone()).map_err(|_| {
        tracing::warn!(
            "submit_application: invalid jurisdiction '{}'",
            input.jurisdiction
        );
        StatusCode::UNPROCESSABLE_ENTITY
    })?;

    let id = ApplicationService::submit_full(
        &db,
        tenant_id,
        input.asset_id,
        input.applicant_user_id,
        jurisdiction,
        input.monthly_income_cents,
    )
    .await
    .map_err(|e| {
        tracing::error!(%tenant_id, "submit_application error: {e:#}");
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    Ok((
        StatusCode::CREATED,
        axum::response::Json(SubmitApplicationResponse {
            id,
            screening_status: "pending",
        }),
    ))
}

/// GET /api/folio/applications
///
/// List all rental applications for the tenant (landlord view).
async fn list_applications(
    Extension(db): Extension<DatabaseConnection>,
    Extension(current_user): Extension<user::Model>,
) -> Result<impl IntoResponse, StatusCode> {
    let tenant_id = resolve_tenant_id(&db, current_user.id).await?;

    let applications = crate::entities::atlas_application::Entity::find()
        .filter(crate::entities::atlas_application::Column::TenantId.eq(tenant_id))
        .filter(
            crate::entities::atlas_application::Column::ApplicationType.eq("rental"),
        )
        .order_by_desc(crate::entities::atlas_application::Column::CreatedAt)
        .all(&db)
        .await
        .map_err(|e| {
            tracing::error!(%tenant_id, "list_applications DB error: {e:#}");
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    let summaries: Vec<ApplicationSummary> = applications.into_iter().map(to_summary).collect();

    Ok(axum::response::Json(summaries))
}

/// GET /api/folio/applications/{id}
async fn get_application(
    Extension(db): Extension<DatabaseConnection>,
    Extension(current_user): Extension<user::Model>,
    Path(application_id): Path<Uuid>,
) -> Result<impl IntoResponse, StatusCode> {
    let tenant_id = resolve_tenant_id(&db, current_user.id).await?;

    let application = crate::entities::atlas_application::Entity::find_by_id(application_id)
        .one(&db)
        .await
        .map_err(|e| {
            tracing::error!(%tenant_id, %application_id, "get_application DB error: {e:#}");
            StatusCode::INTERNAL_SERVER_ERROR
        })?
        .ok_or(StatusCode::NOT_FOUND)?;

    if application.tenant_id != tenant_id {
        return Err(StatusCode::NOT_FOUND);
    }

    Ok(axum::response::Json(to_summary(application)))
}

/// PATCH /api/folio/applications/{id}/decision
///
/// Approve or deny a rental application.
///
/// # FHA note
///
/// Decision reasons are stored verbatim. The landlord is responsible for
/// ensuring reasons comply with FHA — the platform does not audit decision text.
/// The `application_metadata.fha_applies` flag is available for UI enforcement.
async fn decide_application(
    Extension(db): Extension<DatabaseConnection>,
    Extension(current_user): Extension<user::Model>,
    Path(application_id): Path<Uuid>,
    Json(input): Json<DecideApplicationInput>,
) -> Result<impl IntoResponse, StatusCode> {
    use sea_orm::ActiveModelTrait;

    let tenant_id = resolve_tenant_id(&db, current_user.id).await?;

    // Validate decision value.
    let new_status = match input.decision.as_str() {
        "approved" => "approved",
        "denied" => "denied",
        other => {
            tracing::warn!(
                %tenant_id,
                "decide_application: invalid decision '{}', must be 'approved' or 'denied'",
                other
            );
            return Err(StatusCode::UNPROCESSABLE_ENTITY);
        }
    };

    // FHA best-practice: denial reason is required.
    if new_status == "denied" && input.reason.as_deref().map(|r| r.trim()).unwrap_or("").is_empty() {
        tracing::warn!(%tenant_id, %application_id, "decide_application: denial requires a reason");
        return Err(StatusCode::UNPROCESSABLE_ENTITY);
    }

    let application = crate::entities::atlas_application::Entity::find_by_id(application_id)
        .one(&db)
        .await
        .map_err(|e| {
            tracing::error!(%tenant_id, %application_id, "decide_application DB error: {e:#}");
            StatusCode::INTERNAL_SERVER_ERROR
        })?
        .ok_or(StatusCode::NOT_FOUND)?;

    if application.tenant_id != tenant_id {
        return Err(StatusCode::NOT_FOUND);
    }

    // Deny transition from terminal states.
    if matches!(application.status.as_str(), "approved" | "denied" | "withdrawn") {
        tracing::warn!(
            %application_id,
            current_status = %application.status,
            "decide_application: application already in terminal state"
        );
        return Err(StatusCode::CONFLICT);
    }

    let mut active: crate::entities::atlas_application::ActiveModel = application.into();
    active.status = Set(new_status.to_string());
    active.decided_at = Set(Some(chrono::Utc::now()));
    active.decision_reason = Set(input.reason);

    let updated = active.update(&db).await.map_err(|e| {
        tracing::error!(%tenant_id, %application_id, "decide_application update error: {e:#}");
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    tracing::info!(
        %application_id, %tenant_id, decision = new_status,
        "decide_application: application decision recorded"
    );

    Ok(axum::response::Json(to_summary(updated)))
}

// ── Helpers ───────────────────────────────────────────────────────────────────

fn to_summary(a: crate::entities::atlas_application::Model) -> ApplicationSummary {
    ApplicationSummary {
        id: a.id,
        applicant_user_id: a.applicant_user_id,
        target_asset_id: a.target_asset_id,
        status: a.status,
        screening_status: a.screening_status,
        screening_provider: a.screening_provider,
        screening_passed: a.screening_passed,
        monthly_income_cents: a.monthly_income_cents,
        submitted_at: a.submitted_at,
        decided_at: a.decided_at,
        decision_reason: a.decision_reason,
        created_at: a.created_at,
    }
}

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
