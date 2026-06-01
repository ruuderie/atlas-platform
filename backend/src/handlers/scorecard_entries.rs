//! G-27 Scorecard entry handlers.
//!
//! # Routes
//!
//! ```text
//! PATCH  /api/scorecard-entries/:entry_id/verify
//!        Body: { "confirmed": bool }
//!        → 204 on success
//!        → 404 if entry not found / wrong tenant
//!
//! GET    /api/scorecard-templates/:template_id/display-rules
//!        → 200 [DisplayRuleModel] (empty array for Starter tenants)
//! ```

use axum::{
    extract::{Extension, Path, Json},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, patch},
    Router,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::entities::user;
use crate::services::scorecard_service::ScorecardService;

// ── Route registration ────────────────────────────────────────────────────────

pub fn routes() -> Router<sea_orm::DatabaseConnection> {
    Router::new()
        .route(
            "/api/scorecard-entries/{entry_id}/verify",
            patch(verify_entry),
        )
        .route(
            "/api/scorecard-templates/{template_id}/display-rules",
            get(get_display_rules_for_session),
        )
}

// ── Request / response types ──────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct VerifyEntryInput {
    /// true = confirm the AI suggestion (sets is_verified = true + queues recompute)
    /// false = reject the suggestion (deletes the entry)
    pub confirmed: bool,
}

#[derive(Debug, Serialize)]
struct DisplayRuleResponse {
    pub id: Uuid,
    pub template_id: Uuid,
    pub dimension_id: Option<Uuid>,
    pub category_target: Option<String>,
    pub trigger_category: String,
    pub field_reference: Option<String>,
    pub operator: String,
    pub value: Option<String>,
    pub value_list: Option<serde_json::Value>,
    pub action: String,
    pub alert_message: Option<String>,
    pub mode_scope: String,
    pub priority: i32,
    pub is_active: bool,
    pub description: Option<String>,
}

// ── Handlers ──────────────────────────────────────────────────────────────────

/// PATCH /api/scorecard-entries/:entry_id/verify
///
/// Confirm or reject a transcript-inferred scorecard entry.
///
/// - `confirmed: true`  → sets `is_verified = true`, queues aggregate recompute
/// - `confirmed: false` → deletes the entry (rejected AI suggestions are not kept)
///
/// Returns 204 No Content on success.
/// Returns 404 if the entry does not exist or belongs to a different tenant.
async fn verify_entry(
    Extension(db): Extension<sea_orm::DatabaseConnection>,
    Extension(current_user): Extension<user::Model>,
    Path(entry_id): Path<Uuid>,
    Json(input): Json<VerifyEntryInput>,
) -> Result<impl IntoResponse, StatusCode> {
    let tenant_id = resolve_tenant_id(&db, current_user.id).await?;

    ScorecardService::verify_entry(&db, entry_id, tenant_id, input.confirmed)
        .await
        .map_err(|e| {
            // Distinguish "not found / wrong tenant" from genuine server errors.
            // The service returns anyhow::Error; we inspect the message string
            // because we don't have a custom error enum yet.
            let msg = e.to_string();
            if msg.contains("not found for tenant") {
                StatusCode::NOT_FOUND
            } else {
                tracing::error!("verify_entry error: {e:#}");
                StatusCode::INTERNAL_SERVER_ERROR
            }
        })?;

    Ok(StatusCode::NO_CONTENT)
}

/// GET /api/scorecard-templates/:template_id/display-rules
///
/// Returns the active display rules for a template, ordered by priority.
///
/// Starter tenants receive an empty array — all dimensions render
/// unconditionally for them. The tier gate lives in `ScorecardService::get_display_rules`.
async fn get_display_rules_for_session(
    Extension(db): Extension<sea_orm::DatabaseConnection>,
    Extension(current_user): Extension<user::Model>,
    Path(template_id): Path<Uuid>,
) -> Result<impl IntoResponse, StatusCode> {
    let tenant_id = resolve_tenant_id(&db, current_user.id).await?;

    let rules = ScorecardService::get_display_rules(&db, template_id, tenant_id)
        .await
        .map_err(|e| {
            tracing::error!("get_display_rules error: {e:#}");
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    let response: Vec<DisplayRuleResponse> = rules
        .into_iter()
        .map(|r| DisplayRuleResponse {
            id: r.id,
            template_id: r.template_id,
            dimension_id: r.dimension_id,
            category_target: r.category_target,
            trigger_category: r.trigger_category,
            field_reference: r.field_reference,
            operator: r.operator,
            value: r.value,
            value_list: r.value_list,
            action: r.action,
            alert_message: r.alert_message,
            mode_scope: r.mode_scope,
            priority: r.priority,
            is_active: r.is_active,
            description: r.description,
        })
        .collect();

    Ok(axum::response::Json(response))
}

// ── Tenant resolution helper ──────────────────────────────────────────────────

/// Resolve the tenant_id for the current user via their profile.
/// Returns 403 if the user has no profile, 500 on DB error.
async fn resolve_tenant_id(
    db: &sea_orm::DatabaseConnection,
    user_id: Uuid,
) -> Result<Uuid, StatusCode> {
    use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};

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
