//! G-27 Admin handlers for scorecard display rules.
//!
//! These routes are admin-only (called from the configurator UI or admin API).
//! All mutations require tenant ownership verification.
//!
//! # Routes
//!
//! ```ignore
//! GET    /api/admin/scorecard-templates/:template_id/display-rules
//!        -> 200 [Model] (all rules, including inactive — admin sees everything)
//!
//! POST   /api/admin/scorecard-display-rules
//!        Body: CreateDisplayRuleInput
//!        -> 201 Model
//!
//! PATCH  /api/admin/scorecard-display-rules/:id
//!        Body: UpdateDisplayRuleInput
//!        -> 200 Model
//!
//! DELETE /api/admin/scorecard-display-rules/:id
//!        -> 204 (soft-delete: sets is_active = false)
//! ```

use axum::{
    Router,
    extract::{Extension, Json, Path},
    http::StatusCode,
    response::IntoResponse,
    routing::{delete, get, patch, post},
};
use chrono::Utc;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, QueryOrder, Set,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::entities::{atlas_scorecard_display_rule as display_rules, user};

// ── Route registration ────────────────────────────────────────────────────────

pub fn routes() -> Router<DatabaseConnection> {
    Router::new()
        .route(
            "/api/admin/scorecard-templates/{template_id}/display-rules",
            get(list_display_rules),
        )
        .route(
            "/api/admin/scorecard-display-rules",
            post(create_display_rule),
        )
        .route(
            "/api/admin/scorecard-display-rules/{id}",
            patch(update_display_rule),
        )
        .route(
            "/api/admin/scorecard-display-rules/{id}",
            delete(delete_display_rule),
        )
}

// ── Request types ─────────────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct CreateDisplayRuleInput {
    pub template_id: Uuid,
    pub dimension_id: Option<Uuid>,
    pub category_target: Option<String>,
    // Trigger
    pub trigger_category: String,
    pub field_reference: Option<String>,
    pub operator: String,
    pub value: Option<String>,
    pub value_list: Option<serde_json::Value>,
    // Action
    pub action: String,
    pub alert_message: Option<String>,
    // Scope
    pub mode_scope: Option<String>,
    pub priority: Option<i32>,
    pub description: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateDisplayRuleInput {
    pub dimension_id: Option<Uuid>,
    pub category_target: Option<String>,
    pub trigger_category: Option<String>,
    pub field_reference: Option<String>,
    pub operator: Option<String>,
    pub value: Option<String>,
    pub value_list: Option<serde_json::Value>,
    pub action: Option<String>,
    pub alert_message: Option<String>,
    pub mode_scope: Option<String>,
    pub priority: Option<i32>,
    pub description: Option<String>,
    pub is_active: Option<bool>,
}

/// Serialisable view of a display rule returned to admin callers.
#[derive(Debug, Serialize)]
pub struct DisplayRuleAdminView {
    pub id: Uuid,
    pub template_id: Uuid,
    pub dimension_id: Option<Uuid>,
    pub tenant_id: Uuid,
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
    pub created_by_user_id: Option<Uuid>,
    pub created_at: chrono::DateTime<Utc>,
}

impl From<display_rules::Model> for DisplayRuleAdminView {
    fn from(m: display_rules::Model) -> Self {
        Self {
            id: m.id,
            template_id: m.template_id,
            dimension_id: m.dimension_id,
            tenant_id: m.tenant_id,
            category_target: m.category_target,
            trigger_category: m.trigger_category,
            field_reference: m.field_reference,
            operator: m.operator,
            value: m.value,
            value_list: m.value_list,
            action: m.action,
            alert_message: m.alert_message,
            mode_scope: m.mode_scope,
            priority: m.priority,
            is_active: m.is_active,
            description: m.description,
            created_by_user_id: m.created_by_user_id,
            created_at: m.created_at,
        }
    }
}

// ── Handlers ──────────────────────────────────────────────────────────────────

/// GET /api/admin/scorecard-templates/:template_id/display-rules
///
/// Returns all rules for the template (active + inactive). Admin sees all;
/// use the public user-facing endpoint for active-only filtered results.
async fn list_display_rules(
    Extension(db): Extension<DatabaseConnection>,
    Extension(current_user): Extension<user::Model>,
    Path(template_id): Path<Uuid>,
) -> Result<impl IntoResponse, StatusCode> {
    let tenant_id = resolve_tenant_id(&db, current_user.id).await?;

    let rules = display_rules::Entity::find()
        .filter(display_rules::Column::TemplateId.eq(template_id))
        .filter(display_rules::Column::TenantId.eq(tenant_id))
        .order_by_asc(display_rules::Column::Priority)
        .all(&db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let views: Vec<DisplayRuleAdminView> = rules.into_iter().map(Into::into).collect();
    Ok(axum::response::Json(views))
}

/// POST /api/admin/scorecard-display-rules
///
/// Create a new display rule. Validates that trigger_category + operator + action
/// form a coherent combination before inserting.
async fn create_display_rule(
    Extension(db): Extension<DatabaseConnection>,
    Extension(current_user): Extension<user::Model>,
    Json(input): Json<CreateDisplayRuleInput>,
) -> Result<impl IntoResponse, StatusCode> {
    let tenant_id = resolve_tenant_id(&db, current_user.id).await?;

    // Basic coherence check: dimension_id XOR category_target must be set
    if input.dimension_id.is_none() && input.category_target.is_none() {
        return Err(StatusCode::UNPROCESSABLE_ENTITY);
    }

    let new_rule = display_rules::ActiveModel {
        id: Set(Uuid::new_v4()),
        template_id: Set(input.template_id),
        dimension_id: Set(input.dimension_id),
        tenant_id: Set(tenant_id),
        category_target: Set(input.category_target),
        trigger_category: Set(input.trigger_category),
        field_reference: Set(input.field_reference),
        operator: Set(input.operator),
        value: Set(input.value),
        value_list: Set(input.value_list),
        action: Set(input.action),
        alert_message: Set(input.alert_message),
        mode_scope: Set(input.mode_scope.unwrap_or_else(|| "always".to_owned())),
        priority: Set(input.priority.unwrap_or(10)),
        is_active: Set(true),
        description: Set(input.description),
        created_by_user_id: Set(Some(current_user.id)),
        created_at: Set(Utc::now()),
    };

    let inserted = new_rule
        .insert(&db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok((
        StatusCode::CREATED,
        axum::response::Json(DisplayRuleAdminView::from(inserted)),
    ))
}

/// PATCH /api/admin/scorecard-display-rules/:id
///
/// Partial update — only fields present in the body are changed.
/// Tenant ownership is verified before any mutation.
async fn update_display_rule(
    Extension(db): Extension<DatabaseConnection>,
    Extension(current_user): Extension<user::Model>,
    Path(id): Path<Uuid>,
    Json(input): Json<UpdateDisplayRuleInput>,
) -> Result<impl IntoResponse, StatusCode> {
    let tenant_id = resolve_tenant_id(&db, current_user.id).await?;

    let rule = display_rules::Entity::find_by_id(id)
        .one(&db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;

    if rule.tenant_id != tenant_id {
        return Err(StatusCode::NOT_FOUND); // deliberate: don't leak existence
    }

    let mut am: display_rules::ActiveModel = rule.into();

    if let Some(v) = input.dimension_id {
        am.dimension_id = Set(Some(v));
    }
    if let Some(v) = input.category_target {
        am.category_target = Set(Some(v));
    }
    if let Some(v) = input.trigger_category {
        am.trigger_category = Set(v);
    }
    if let Some(v) = input.field_reference {
        am.field_reference = Set(Some(v));
    }
    if let Some(v) = input.operator {
        am.operator = Set(v);
    }
    if let Some(v) = input.value {
        am.value = Set(Some(v));
    }
    if let Some(v) = input.value_list {
        am.value_list = Set(Some(v));
    }
    if let Some(v) = input.action {
        am.action = Set(v);
    }
    if let Some(v) = input.alert_message {
        am.alert_message = Set(Some(v));
    }
    if let Some(v) = input.mode_scope {
        am.mode_scope = Set(v);
    }
    if let Some(v) = input.priority {
        am.priority = Set(v);
    }
    if let Some(v) = input.description {
        am.description = Set(Some(v));
    }
    if let Some(v) = input.is_active {
        am.is_active = Set(v);
    }

    let updated = am
        .update(&db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(axum::response::Json(DisplayRuleAdminView::from(updated)))
}

/// DELETE /api/admin/scorecard-display-rules/:id
///
/// Soft-delete: sets `is_active = false`. The rule is kept for audit purposes.
/// Hard delete is not exposed via the API.
async fn delete_display_rule(
    Extension(db): Extension<DatabaseConnection>,
    Extension(current_user): Extension<user::Model>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, StatusCode> {
    let tenant_id = resolve_tenant_id(&db, current_user.id).await?;

    let rule = display_rules::Entity::find_by_id(id)
        .one(&db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;

    if rule.tenant_id != tenant_id {
        return Err(StatusCode::NOT_FOUND);
    }

    let mut am: display_rules::ActiveModel = rule.into();
    am.is_active = Set(false);
    am.update(&db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(StatusCode::NO_CONTENT)
}

// ── Tenant resolution helper ──────────────────────────────────────────────────

async fn resolve_tenant_id(db: &DatabaseConnection, user_id: Uuid) -> Result<Uuid, StatusCode> {
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
