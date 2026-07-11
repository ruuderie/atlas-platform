//! G-27 Scorecard entry handlers.
//!
//! # Routes
//!
//! ```ignore
//! GET    /api/scorecard-templates?app_instance_id=&is_published=
//!        -> 200 [TemplateListItem] — deployed+enabled for the instance
//!
//! PATCH  /api/scorecard-entries/:entry_id/verify
//!        Body: { "confirmed": bool }
//!        -> 204 on success
//!        -> 404 if entry not found / wrong tenant
//!
//! GET    /api/scorecard-templates/:template_id/display-rules
//!        -> 200 [DisplayRuleModel] (empty array for Starter tenants)
//! ```

use axum::{
    extract::{Extension, Path, Query, Json},
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
    routing::{get, patch, post},
    Router,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::entities::user;
use crate::services::scorecard_service::ScorecardService;
use crate::types::pm::TemplateScope;
use crate::types::scorecard::{
    ModeScope, RatingSessionStatus, RuleAction, RuleOperator, ScaleType, SessionType, SourceType,
    TriggerCategory,
};

// ── Route registration ────────────────────────────────────────────────────────

pub fn routes() -> Router<sea_orm::DatabaseConnection> {
    Router::new()
        .route(
            "/api/scorecard-templates",
            get(list_deployed_templates),
        )
        .route(
            "/api/scorecards/get-or-create",
            post(tenant_get_or_create),
        )
        .route(
            "/api/scorecards/{id}/sessions",
            post(tenant_open_session),
        )
        .route(
            "/api/scorecard-sessions/{sid}/entries",
            post(tenant_submit_entry),
        )
        .route(
            "/api/scorecard-sessions/pending",
            get(list_pending_sessions),
        )
        .route(
            "/api/scorecard-templates/{template_id}/dimensions",
            get(list_template_dimensions),
        )
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
pub struct ListTemplatesQuery {
    /// Prefer explicit instance. Falls back to `x-app-instance-id` header, then
    /// the tenant's first `property_management` app instance.
    pub app_instance_id: Option<Uuid>,
    /// When `true`, only published templates. Default: all deployed+enabled.
    pub is_published: Option<bool>,
}

/// Tenant-facing template list item (Folio Meridian / Configurator).
#[derive(Debug, Serialize)]
pub struct TemplateListItem {
    pub id: Uuid,
    pub name: String,
    /// Polymorphic: global `ScorecardEntityType` or PM provisioner vocabulary.
    pub entity_type: String,
    pub description: Option<String>,
    pub is_published: bool,
    /// Alias of `is_published` for Folio Meridian deserializers that expect `is_active`.
    pub is_active: bool,
    pub template_scope: TemplateScope,
}

#[derive(Debug, Deserialize)]
pub struct GetOrCreateInput {
    pub template_id: Uuid,
    /// Polymorphic subject type — validated at the write boundary.
    pub subject_entity_type: String,
    pub subject_entity_id: Uuid,
}

#[derive(Debug, Serialize)]
pub struct IdResponse {
    pub id: Uuid,
}

#[derive(Debug, Deserialize)]
pub struct OpenSessionInput {
    pub session_type: SessionType,
    pub context_entity_type: Option<String>,
    pub context_entity_id: Option<Uuid>,
    pub session_label: Option<String>,
    pub occurred_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Deserialize)]
pub struct SubmitEntryInput {
    pub scorecard_id: Uuid,
    pub dimension_id: Uuid,
    pub score: Option<f64>,
    pub option_id: Option<Uuid>,
    #[serde(default)]
    pub source_type: Option<SourceType>,
    pub context: Option<serde_json::Value>,
    pub note: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct PendingSessionView {
    pub session_id: Uuid,
    pub scorecard_id: Uuid,
    pub template_id: Uuid,
    pub subject_entity_type: String,
    pub subject_entity_id: Uuid,
    pub session_type: SessionType,
    pub context_entity_type: Option<String>,
    pub context_entity_id: Option<Uuid>,
    pub session_label: Option<String>,
    pub status: RatingSessionStatus,
    pub occurred_at: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
pub struct DimensionListItem {
    pub id: Uuid,
    pub template_id: Uuid,
    pub slug: String,
    pub name: String,
    pub description: Option<String>,
    pub category: Option<String>,
    pub scale_type: ScaleType,
    pub scale_min: String,
    pub scale_max: String,
    pub weight: String,
    pub sort_order: i32,
    pub is_active: bool,
}

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
    pub trigger_category: TriggerCategory,
    pub field_reference: Option<String>,
    pub operator: RuleOperator,
    pub value: Option<String>,
    pub value_list: Option<serde_json::Value>,
    pub action: RuleAction,
    pub alert_message: Option<String>,
    pub mode_scope: ModeScope,
    pub priority: i32,
    pub is_active: bool,
    pub description: Option<String>,
}

// ── Handlers ──────────────────────────────────────────────────────────────────

/// GET /api/scorecard-templates?app_instance_id=&is_published=
///
/// Lists templates that are **deployed and enabled** for the resolved app instance.
/// Explicit tenant isolation: deployments and templates must match the caller's tenant.
async fn list_deployed_templates(
    Extension(db): Extension<sea_orm::DatabaseConnection>,
    Extension(current_user): Extension<user::Model>,
    headers: HeaderMap,
    Query(query): Query<ListTemplatesQuery>,
) -> Result<impl IntoResponse, StatusCode> {
    let tenant_id = resolve_tenant_id(&db, current_user.id).await?;

    let app_instance_id = resolve_list_app_instance_id(&db, tenant_id, &headers, query.app_instance_id)
        .await?;

    // Verify the instance belongs to this tenant (403 if foreign).
    ensure_instance_belongs_to_tenant(&db, tenant_id, app_instance_id).await?;

    let published_only = query.is_published.filter(|v| *v);

    let rows = ScorecardService::templates_enabled_for_instance(
        &db,
        tenant_id,
        app_instance_id,
        published_only,
    )
    .await
    .map_err(|e| {
        tracing::error!(%tenant_id, %app_instance_id, "list_deployed_templates error: {e:#}");
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    let items: Vec<TemplateListItem> = rows
        .into_iter()
        .filter_map(|t| {
            let template_scope = TemplateScope::try_from(t.template_scope).ok()?;
            Some(TemplateListItem {
                id: t.id,
                name: t.name,
                entity_type: t.entity_type,
                description: t.description,
                is_published: t.is_published,
                is_active: t.is_published,
                template_scope,
            })
        })
        .collect();

    Ok(axum::response::Json(items))
}

/// POST /api/scorecards/get-or-create
async fn tenant_get_or_create(
    Extension(db): Extension<sea_orm::DatabaseConnection>,
    Extension(current_user): Extension<user::Model>,
    headers: HeaderMap,
    Query(query): Query<ListTemplatesQuery>,
    Json(input): Json<GetOrCreateInput>,
) -> Result<impl IntoResponse, StatusCode> {
    let tenant_id = resolve_tenant_id(&db, current_user.id).await?;
    let app_instance_id =
        resolve_list_app_instance_id(&db, tenant_id, &headers, query.app_instance_id).await?;
    ensure_instance_belongs_to_tenant(&db, tenant_id, app_instance_id).await?;

    // Template must belong to this tenant and be deployed+enabled on this instance.
    use sea_orm::EntityTrait;
    let template = crate::entities::atlas_scorecard_template::Entity::find_by_id(input.template_id)
        .one(&db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;
    if template.tenant_id != tenant_id {
        return Err(StatusCode::NOT_FOUND);
    }
    ensure_template_deployed_enabled(&db, tenant_id, app_instance_id, input.template_id).await?;

    let subject_type = parse_subject_entity_type(&input.subject_entity_type)?;

    let id = ScorecardService::get_or_create(
        &db,
        tenant_id,
        input.template_id,
        &subject_type,
        input.subject_entity_id,
    )
    .await
    .map_err(|e| {
        tracing::error!(%tenant_id, "tenant get_or_create error: {e:#}");
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    Ok(Json(IdResponse { id }))
}

/// POST /api/scorecards/{id}/sessions
async fn tenant_open_session(
    Extension(db): Extension<sea_orm::DatabaseConnection>,
    Extension(current_user): Extension<user::Model>,
    headers: HeaderMap,
    Query(query): Query<ListTemplatesQuery>,
    Path(scorecard_id): Path<Uuid>,
    Json(input): Json<OpenSessionInput>,
) -> Result<impl IntoResponse, StatusCode> {
    let tenant_id = resolve_tenant_id(&db, current_user.id).await?;
    let app_instance_id =
        resolve_list_app_instance_id(&db, tenant_id, &headers, query.app_instance_id).await?;
    ensure_instance_belongs_to_tenant(&db, tenant_id, app_instance_id).await?;

    use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};
    let sc = crate::entities::atlas_scorecard::Entity::find_by_id(scorecard_id)
        .filter(crate::entities::atlas_scorecard::Column::TenantId.eq(tenant_id))
        .filter(crate::entities::atlas_scorecard::Column::DeletedAt.is_null())
        .one(&db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;

    ensure_template_deployed_enabled(&db, tenant_id, app_instance_id, sc.template_id).await?;

    let sid = ScorecardService::open_session(
        &db,
        scorecard_id,
        current_user.id,
        tenant_id,
        input.occurred_at.unwrap_or_else(Utc::now),
        &input.session_type.to_string(),
        input.context_entity_type.as_deref(),
        input.context_entity_id,
        input.session_label.as_deref(),
    )
    .await
    .map_err(|e| {
        tracing::error!(%tenant_id, %scorecard_id, "tenant open_session error: {e:#}");
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    Ok(Json(IdResponse { id: sid }))
}

/// POST /api/scorecard-sessions/{sid}/entries
async fn tenant_submit_entry(
    Extension(db): Extension<sea_orm::DatabaseConnection>,
    Extension(current_user): Extension<user::Model>,
    Path(sid): Path<Uuid>,
    Json(input): Json<SubmitEntryInput>,
) -> Result<impl IntoResponse, StatusCode> {
    let tenant_id = resolve_tenant_id(&db, current_user.id).await?;

    let source = input.source_type.unwrap_or(SourceType::Manual);

    let entry_id = ScorecardService::submit_entry(
        &db,
        sid,
        input.scorecard_id,
        input.dimension_id,
        tenant_id,
        current_user.id,
        input.score,
        input.option_id,
        &source.to_string(),
        input.context,
        input.note.as_deref(),
    )
    .await
    .map_err(|e| {
        let msg = e.to_string();
        if msg.contains("not found for tenant") {
            StatusCode::NOT_FOUND
        } else if msg.contains("rater mismatch") {
            StatusCode::FORBIDDEN
        } else if msg.contains("UNIQUE") || msg.contains("already") || msg.contains("duplicate") {
            StatusCode::CONFLICT
        } else {
            tracing::error!(%tenant_id, session_id = %sid, "tenant submit_entry error: {e:#}");
            StatusCode::INTERNAL_SERVER_ERROR
        }
    })?;

    Ok(Json(IdResponse { id: entry_id }))
}

/// GET /api/scorecard-sessions/pending
///
/// Sessions for the current user that still need entries (opened by triggers).
async fn list_pending_sessions(
    Extension(db): Extension<sea_orm::DatabaseConnection>,
    Extension(current_user): Extension<user::Model>,
) -> Result<impl IntoResponse, StatusCode> {
    use sea_orm::{ColumnTrait, EntityTrait, PaginatorTrait, QueryFilter, QueryOrder};

    let tenant_id = resolve_tenant_id(&db, current_user.id).await?;

    let sessions = crate::entities::atlas_rating_session::Entity::find()
        .filter(crate::entities::atlas_rating_session::Column::TenantId.eq(tenant_id))
        .filter(crate::entities::atlas_rating_session::Column::RaterUserId.eq(current_user.id))
        .order_by_desc(crate::entities::atlas_rating_session::Column::OccurredAt)
        .all(&db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let mut out = Vec::new();
    for s in sessions {
        let sc = crate::entities::atlas_scorecard::Entity::find_by_id(s.scorecard_id)
            .one(&db)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
            .ok_or(StatusCode::INTERNAL_SERVER_ERROR)?;

        // Pending = fewer verified/submitted entries than active dimensions on the template.
        let dim_count = crate::entities::atlas_scorecard_dimension::Entity::find()
            .filter(crate::entities::atlas_scorecard_dimension::Column::TemplateId.eq(sc.template_id))
            .filter(crate::entities::atlas_scorecard_dimension::Column::TenantId.eq(tenant_id))
            .filter(crate::entities::atlas_scorecard_dimension::Column::IsActive.eq(true))
            .count(&db)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

        let entry_count = crate::entities::atlas_scorecard_entry::Entity::find()
            .filter(crate::entities::atlas_scorecard_entry::Column::SessionId.eq(s.id))
            .filter(crate::entities::atlas_scorecard_entry::Column::ContributorUserId.eq(current_user.id))
            .count(&db)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

        if dim_count > 0 && entry_count >= dim_count {
            continue;
        }
        // No dimensions yet → still show so the guest can see the nudge (empty widget).
        if dim_count == 0 && entry_count > 0 {
            continue;
        }

        out.push(PendingSessionView {
            session_id: s.id,
            scorecard_id: s.scorecard_id,
            template_id: sc.template_id,
            subject_entity_type: sc.subject_entity_type,
            subject_entity_id: sc.subject_entity_id,
            session_type: SessionType::try_from(s.session_type)
                .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?,
            context_entity_type: s.context_entity_type,
            context_entity_id: s.context_entity_id,
            session_label: s.session_label,
            status: RatingSessionStatus::try_from(s.status)
                .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?,
            occurred_at: s.occurred_at,
        });
    }

    Ok(Json(out))
}

/// GET /api/scorecard-templates/{template_id}/dimensions
async fn list_template_dimensions(
    Extension(db): Extension<sea_orm::DatabaseConnection>,
    Extension(current_user): Extension<user::Model>,
    Path(template_id): Path<Uuid>,
) -> Result<impl IntoResponse, StatusCode> {
    use sea_orm::{ColumnTrait, EntityTrait, QueryFilter, QueryOrder};

    let tenant_id = resolve_tenant_id(&db, current_user.id).await?;

    let template = crate::entities::atlas_scorecard_template::Entity::find_by_id(template_id)
        .one(&db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;
    if template.tenant_id != tenant_id {
        return Err(StatusCode::NOT_FOUND);
    }

    let dims = crate::entities::atlas_scorecard_dimension::Entity::find()
        .filter(crate::entities::atlas_scorecard_dimension::Column::TemplateId.eq(template_id))
        .filter(crate::entities::atlas_scorecard_dimension::Column::TenantId.eq(tenant_id))
        .filter(crate::entities::atlas_scorecard_dimension::Column::IsActive.eq(true))
        .order_by_asc(crate::entities::atlas_scorecard_dimension::Column::SortOrder)
        .all(&db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let items: Vec<DimensionListItem> = dims
        .into_iter()
        .filter_map(|d| {
            let scale_type = ScaleType::try_from(d.scale_type).ok()?;
            Some(DimensionListItem {
                id: d.id,
                template_id: d.template_id,
                slug: d.slug,
                name: d.name,
                description: d.description,
                category: d.category,
                scale_type,
                scale_min: d.scale_min.to_string(),
                scale_max: d.scale_max.to_string(),
                weight: d.weight.to_string(),
                sort_order: d.sort_order,
                is_active: d.is_active,
            })
        })
        .collect();

    Ok(Json(items))
}

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
        .filter_map(|r| {
            Some(DisplayRuleResponse {
                id: r.id,
                template_id: r.template_id,
                dimension_id: r.dimension_id,
                category_target: r.category_target,
                trigger_category: TriggerCategory::try_from(r.trigger_category).ok()?,
                field_reference: r.field_reference,
                operator: RuleOperator::try_from(r.operator).ok()?,
                value: r.value,
                value_list: r.value_list,
                action: RuleAction::try_from(r.action).ok()?,
                alert_message: r.alert_message,
                mode_scope: ModeScope::try_from(r.mode_scope).ok()?,
                priority: r.priority,
                is_active: r.is_active,
                description: r.description,
            })
        })
        .collect();

    Ok(axum::response::Json(response))
}

// ── Tenant / instance resolution helpers ──────────────────────────────────────

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

async fn resolve_list_app_instance_id(
    db: &sea_orm::DatabaseConnection,
    tenant_id: Uuid,
    headers: &HeaderMap,
    query_id: Option<Uuid>,
) -> Result<Uuid, StatusCode> {
    if let Some(id) = query_id {
        return Ok(id);
    }

    if let Some(raw) = headers
        .get("x-app-instance-id")
        .and_then(|v| v.to_str().ok())
    {
        if let Ok(id) = Uuid::parse_str(raw) {
            return Ok(id);
        }
    }

    // Fallback: first Folio (property_management) instance for this tenant.
    use sea_orm::{ColumnTrait, EntityTrait, QueryFilter, QueryOrder};
    crate::entities::app_instance::Entity::find()
        .filter(crate::entities::app_instance::Column::TenantId.eq(tenant_id))
        .filter(crate::entities::app_instance::Column::AppType.eq("property_management"))
        .order_by_asc(crate::entities::app_instance::Column::CreatedAt)
        .one(db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .map(|i| i.id)
        .ok_or(StatusCode::NOT_FOUND)
}

async fn ensure_instance_belongs_to_tenant(
    db: &sea_orm::DatabaseConnection,
    tenant_id: Uuid,
    app_instance_id: Uuid,
) -> Result<(), StatusCode> {
    use sea_orm::EntityTrait;

    let row = crate::entities::app_instance::Entity::find_by_id(app_instance_id)
        .one(db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;

    if row.tenant_id != tenant_id {
        return Err(StatusCode::FORBIDDEN);
    }
    Ok(())
}

/// Template must have an enabled deployment on this app instance.
async fn ensure_template_deployed_enabled(
    db: &sea_orm::DatabaseConnection,
    tenant_id: Uuid,
    app_instance_id: Uuid,
    template_id: Uuid,
) -> Result<(), StatusCode> {
    use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};

    let dep = crate::entities::atlas_scorecard_template_deployment::Entity::find()
        .filter(crate::entities::atlas_scorecard_template_deployment::Column::TenantId.eq(tenant_id))
        .filter(crate::entities::atlas_scorecard_template_deployment::Column::AppInstanceId.eq(app_instance_id))
        .filter(crate::entities::atlas_scorecard_template_deployment::Column::TemplateId.eq(template_id))
        .filter(crate::entities::atlas_scorecard_template_deployment::Column::IsEnabled.eq(true))
        .one(db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    if dep.is_none() {
        return Err(StatusCode::FORBIDDEN);
    }
    Ok(())
}

/// Accept global G-27 entity types plus PM provisioner vocabulary.
fn parse_subject_entity_type(raw: &str) -> Result<String, StatusCode> {
    use crate::types::scorecard::ScorecardEntityType;
    if ScorecardEntityType::try_from(raw.to_owned()).is_ok() {
        return Ok(raw.to_owned());
    }
    if crate::types::pm::ScorecardEntityType::try_from(raw.to_owned()).is_ok() {
        return Ok(raw.to_owned());
    }
    Err(StatusCode::UNPROCESSABLE_ENTITY)
}
