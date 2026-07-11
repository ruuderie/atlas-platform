//! G-27 Phase 1 — Platform-admin scorecard REST (explicit `tenant_id` in path).
//!
//! Contract: `docs/contracts/g27_scorecard_platform.md` §6.
//!
//! Phase 1b: template deployments (per app-instance enablement).

use axum::{
    extract::{Extension, Path, Query},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, patch, post},
    Json, Router,
};
use chrono::Utc;
use rust_decimal::Decimal;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, QueryOrder, Set,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::entities::{
    app_instance, atlas_rating_session as sessions, atlas_scorecard as scorecards,
    atlas_scorecard_dimension as dimensions, atlas_scorecard_dimension_aggregate as aggregates,
    atlas_scorecard_entry as entries, atlas_scorecard_template as templates,
    atlas_scorecard_template_deployment as deployments, atlas_scorecard_time_series as time_series,
    user,
};
use crate::services::scorecard_analytics_service::ScorecardAnalyticsService;
use crate::services::scorecard_service::ScorecardService;
use crate::types::pm::TemplateScope;
use crate::types::scorecard::{
    ColdStartStrategy, ScaleType, ScorecardEntityType, ScoringMethod, SessionType, SourceType,
};

// ── Route registration ────────────────────────────────────────────────────────

pub fn routes() -> Router<DatabaseConnection> {
    Router::new()
        // Catalog (no tenant path — platform-scoped rows across tenants)
        .route(
            "/api/admin/scorecard-templates/catalog",
            get(list_catalog),
        )
        // Templates
        .route(
            "/api/admin/tenants/{tenant_id}/scorecard-templates",
            get(list_templates).post(create_template),
        )
        .route(
            "/api/admin/tenants/{tenant_id}/scorecard-templates/{id}",
            get(get_template).patch(update_template),
        )
        .route(
            "/api/admin/tenants/{tenant_id}/scorecard-templates/{id}/dimensions",
            get(list_dimensions).post(create_dimension),
        )
        .route(
            "/api/admin/tenants/{tenant_id}/scorecard-dimensions/{dim_id}",
            patch(update_dimension),
        )
        // Scorecards
        .route(
            "/api/admin/tenants/{tenant_id}/scorecards/get-or-create",
            post(get_or_create_scorecard),
        )
        .route(
            "/api/admin/tenants/{tenant_id}/scorecards/{id}",
            get(get_scorecard),
        )
        .route(
            "/api/admin/tenants/{tenant_id}/scorecards/{id}/sessions",
            get(list_sessions).post(open_session),
        )
        .route(
            "/api/admin/tenants/{tenant_id}/scorecards/{id}/sessions/{sid}/entries",
            get(list_entries),
        )
        .route(
            "/api/admin/tenants/{tenant_id}/scorecards/{id}/time-series",
            get(list_time_series),
        )
        .route(
            "/api/admin/tenants/{tenant_id}/scorecards/{id}/recompute",
            post(recompute_scorecard),
        )
        .route(
            "/api/admin/tenants/{tenant_id}/scorecard-sessions/{sid}/entries",
            post(submit_entry),
        )
        // Analytics wrappers
        .route(
            "/api/admin/tenants/{tenant_id}/scorecard-templates/{id}/analytics",
            get(portfolio_stats).post(portfolio_stats),
        )
        .route(
            "/api/admin/tenants/{tenant_id}/scorecard-templates/{id}/leaderboard",
            get(leaderboard),
        )
        .route(
            "/api/admin/tenants/{tenant_id}/scorecard-templates/{id}/anomalies",
            get(anomalies),
        )
        .route(
            "/api/admin/tenants/{tenant_id}/scorecard-templates/{id}/analytics/refresh",
            post(refresh_analytics),
        )
        // Deployments (Phase 1b)
        .route(
            "/api/admin/tenants/{tenant_id}/app-instances/{instance_id}/scorecard-deployments",
            get(list_instance_deployments).put(upsert_instance_deployments),
        )
        .route(
            "/api/admin/scorecard-templates/{id}/deployments",
            get(list_template_deployments),
        )
}

// ── Request / response types ──────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct CatalogQuery {
    pub is_published: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct CreateTemplateInput {
    pub name: String,
    pub entity_type: String,
    pub description: Option<String>,
    pub scoring_method: Option<String>,
    pub default_scale_min: Option<Decimal>,
    pub default_scale_max: Option<Decimal>,
    pub min_entries_to_publish: Option<i32>,
    pub is_published: Option<bool>,
    pub template_scope: Option<String>,
    pub cold_start_strategy: Option<String>,
    pub cold_start_saturation_threshold: Option<i32>,
    pub default_bayesian_prior_weight: Option<Decimal>,
    pub calibration_minimum_entries: Option<i32>,
    pub display_config: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateTemplateInput {
    pub name: Option<String>,
    pub description: Option<String>,
    pub scoring_method: Option<String>,
    pub default_scale_min: Option<Decimal>,
    pub default_scale_max: Option<Decimal>,
    pub min_entries_to_publish: Option<i32>,
    pub is_published: Option<bool>,
    pub template_scope: Option<String>,
    pub cold_start_strategy: Option<String>,
    pub cold_start_saturation_threshold: Option<i32>,
    pub default_bayesian_prior_weight: Option<Decimal>,
    pub calibration_minimum_entries: Option<i32>,
    pub display_config: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
pub struct CreateDimensionInput {
    pub slug: String,
    pub name: String,
    pub description: Option<String>,
    pub category: Option<String>,
    pub weight: Option<Decimal>,
    pub scale_type: String,
    pub scale_min: Option<Decimal>,
    pub scale_max: Option<Decimal>,
    pub unit_label: Option<String>,
    pub benchmark_tiers: Option<serde_json::Value>,
    pub global_reference_value: Option<Decimal>,
    pub global_reference_label: Option<String>,
    pub min_entries_to_show: Option<i32>,
    pub is_community_ratable: Option<bool>,
    pub is_active: Option<bool>,
    pub sort_order: Option<i32>,
    pub is_inverted: Option<bool>,
    pub bayesian_prior_weight: Option<Decimal>,
    pub is_tenant_extension: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateDimensionInput {
    pub name: Option<String>,
    pub description: Option<String>,
    pub category: Option<String>,
    pub weight: Option<Decimal>,
    pub scale_type: Option<String>,
    pub scale_min: Option<Decimal>,
    pub scale_max: Option<Decimal>,
    pub unit_label: Option<String>,
    pub benchmark_tiers: Option<serde_json::Value>,
    pub global_reference_value: Option<Decimal>,
    pub global_reference_label: Option<String>,
    pub min_entries_to_show: Option<i32>,
    pub is_community_ratable: Option<bool>,
    pub is_active: Option<bool>,
    pub sort_order: Option<i32>,
    pub is_inverted: Option<bool>,
    pub bayesian_prior_weight: Option<Decimal>,
    pub is_tenant_extension: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct GetOrCreateInput {
    pub template_id: Uuid,
    pub subject_entity_type: String,
    pub subject_entity_id: Uuid,
}

#[derive(Debug, Serialize)]
pub struct GetOrCreateResponse {
    pub id: Uuid,
}

#[derive(Debug, Deserialize)]
pub struct OpenSessionInput {
    pub session_type: String,
    pub occurred_at: Option<chrono::DateTime<Utc>>,
    pub context_entity_type: Option<String>,
    pub context_entity_id: Option<Uuid>,
    pub session_label: Option<String>,
    /// Optional app-instance attribution (Phase C).
    pub app_instance_id: Option<Uuid>,
}

#[derive(Debug, Serialize)]
pub struct OpenSessionResponse {
    pub id: Uuid,
}

#[derive(Debug, Deserialize)]
pub struct SubmitEntryInput {
    pub scorecard_id: Uuid,
    pub dimension_id: Uuid,
    pub score: Option<f64>,
    pub option_id: Option<Uuid>,
    pub source_type: Option<String>,
    pub context: Option<serde_json::Value>,
    pub note: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct SubmitEntryResponse {
    pub id: Uuid,
}

#[derive(Debug, Serialize)]
pub struct ScorecardDetailResponse {
    #[serde(flatten)]
    pub scorecard: scorecards::Model,
    pub dimension_aggregates: Vec<aggregates::Model>,
}

#[derive(Debug, Deserialize)]
pub struct TimeSeriesQuery {
    pub dimension_id: Option<Uuid>,
    pub period_type: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct LeaderboardQuery {
    pub limit: Option<i64>,
}

#[derive(Debug, Deserialize)]
pub struct AnomalyQuery {
    pub limit: Option<i64>,
}

#[derive(Debug, Serialize)]
pub struct DeploymentView {
    #[serde(flatten)]
    pub deployment: deployments::Model,
    pub template_name: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpsertDeploymentItem {
    pub template_id: Uuid,
    pub is_enabled: bool,
    pub trigger_event: Option<String>,
    pub trigger_context_entity_type: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpsertDeploymentsInput {
    pub deployments: Vec<UpsertDeploymentItem>,
}

// ── Catalog ───────────────────────────────────────────────────────────────────

/// GET /api/admin/scorecard-templates/catalog?is_published=
///
/// Lists platform-scoped templates across tenants (catalog lens).
async fn list_catalog(
    Extension(db): Extension<DatabaseConnection>,
    Extension(_user): Extension<user::Model>,
    Query(params): Query<CatalogQuery>,
) -> Result<impl IntoResponse, StatusCode> {
    let mut q = templates::Entity::find()
        .filter(templates::Column::TemplateScope.eq(TemplateScope::Platform.to_string()));

    if let Some(published) = params.is_published {
        q = q.filter(templates::Column::IsPublished.eq(published));
    }

    let rows = q
        .order_by_asc(templates::Column::Name)
        .all(&db)
        .await
        .map_err(|e| {
            tracing::error!("list_catalog error: {e:#}");
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    Ok(Json(rows))
}

// ── Templates ─────────────────────────────────────────────────────────────────

async fn list_templates(
    Extension(db): Extension<DatabaseConnection>,
    Extension(_user): Extension<user::Model>,
    Path(tenant_id): Path<Uuid>,
) -> Result<impl IntoResponse, StatusCode> {
    let rows = templates::Entity::find()
        .filter(templates::Column::TenantId.eq(tenant_id))
        .order_by_asc(templates::Column::Name)
        .all(&db)
        .await
        .map_err(|e| {
            tracing::error!(%tenant_id, "list_templates error: {e:#}");
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    Ok(Json(rows))
}

async fn create_template(
    Extension(db): Extension<DatabaseConnection>,
    Extension(current_user): Extension<user::Model>,
    Path(tenant_id): Path<Uuid>,
    Json(input): Json<CreateTemplateInput>,
) -> Result<impl IntoResponse, StatusCode> {
    let entity_type = parse_entity_type(&input.entity_type)?;
    let scoring_method = match input.scoring_method.as_deref() {
        Some(s) => ScoringMethod::try_from(s.to_owned()).map_err(|_| StatusCode::UNPROCESSABLE_ENTITY)?,
        None => ScoringMethod::WeightedMean,
    };
    let template_scope = match input.template_scope.as_deref() {
        Some(s) => TemplateScope::try_from(s.to_owned()).map_err(|_| StatusCode::UNPROCESSABLE_ENTITY)?,
        None => TemplateScope::Tenant,
    };
    let cold_start = match input.cold_start_strategy.as_deref() {
        Some(s) => ColdStartStrategy::try_from(s.to_owned()).map_err(|_| StatusCode::UNPROCESSABLE_ENTITY)?,
        None => ColdStartStrategy::Suppress,
    };

    let now = Utc::now();
    let model = templates::ActiveModel {
        id: Set(Uuid::new_v4()),
        tenant_id: Set(tenant_id),
        name: Set(input.name),
        entity_type: Set(entity_type),
        description: Set(input.description),
        scoring_method: Set(scoring_method.to_string()),
        default_scale_min: Set(input.default_scale_min.unwrap_or(dec("1.0"))),
        default_scale_max: Set(input.default_scale_max.unwrap_or(dec("10.0"))),
        min_entries_to_publish: Set(input.min_entries_to_publish.unwrap_or(5)),
        is_published: Set(input.is_published.unwrap_or(false)),
        template_scope: Set(template_scope.to_string()),
        cold_start_strategy: Set(cold_start.to_string()),
        cold_start_saturation_threshold: Set(input.cold_start_saturation_threshold.unwrap_or(50)),
        default_bayesian_prior_weight: Set(input.default_bayesian_prior_weight),
        calibration_minimum_entries: Set(input.calibration_minimum_entries.unwrap_or(100)),
        display_config: Set(input.display_config),
        created_by_user_id: Set(Some(current_user.id)),
        created_at: Set(now),
        updated_at: Set(now),
    };

    let inserted = model.insert(&db).await.map_err(|e| {
        tracing::error!(%tenant_id, "create_template error: {e:#}");
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    Ok((StatusCode::CREATED, Json(inserted)))
}

async fn get_template(
    Extension(db): Extension<DatabaseConnection>,
    Extension(_user): Extension<user::Model>,
    Path((tenant_id, id)): Path<(Uuid, Uuid)>,
) -> Result<impl IntoResponse, StatusCode> {
    let row = load_template(&db, tenant_id, id).await?;
    Ok(Json(row))
}

async fn update_template(
    Extension(db): Extension<DatabaseConnection>,
    Extension(_user): Extension<user::Model>,
    Path((tenant_id, id)): Path<(Uuid, Uuid)>,
    Json(input): Json<UpdateTemplateInput>,
) -> Result<impl IntoResponse, StatusCode> {
    let row = load_template(&db, tenant_id, id).await?;
    let mut am: templates::ActiveModel = row.into();

    if let Some(v) = input.name {
        am.name = Set(v);
    }
    if let Some(v) = input.description {
        am.description = Set(Some(v));
    }
    if let Some(v) = input.scoring_method {
        let method = ScoringMethod::try_from(v).map_err(|_| StatusCode::UNPROCESSABLE_ENTITY)?;
        am.scoring_method = Set(method.to_string());
    }
    if let Some(v) = input.default_scale_min {
        am.default_scale_min = Set(v);
    }
    if let Some(v) = input.default_scale_max {
        am.default_scale_max = Set(v);
    }
    if let Some(v) = input.min_entries_to_publish {
        am.min_entries_to_publish = Set(v);
    }
    if let Some(v) = input.is_published {
        am.is_published = Set(v);
    }
    if let Some(v) = input.template_scope {
        let scope = TemplateScope::try_from(v).map_err(|_| StatusCode::UNPROCESSABLE_ENTITY)?;
        am.template_scope = Set(scope.to_string());
    }
    if let Some(v) = input.cold_start_strategy {
        let strategy =
            ColdStartStrategy::try_from(v).map_err(|_| StatusCode::UNPROCESSABLE_ENTITY)?;
        am.cold_start_strategy = Set(strategy.to_string());
    }
    if let Some(v) = input.cold_start_saturation_threshold {
        am.cold_start_saturation_threshold = Set(v);
    }
    if let Some(v) = input.default_bayesian_prior_weight {
        am.default_bayesian_prior_weight = Set(Some(v));
    }
    if let Some(v) = input.calibration_minimum_entries {
        am.calibration_minimum_entries = Set(v);
    }
    if let Some(v) = input.display_config {
        am.display_config = Set(Some(v));
    }
    am.updated_at = Set(Utc::now());

    let updated = am.update(&db).await.map_err(|e| {
        tracing::error!(%tenant_id, %id, "update_template error: {e:#}");
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    Ok(Json(updated))
}

// ── Dimensions ────────────────────────────────────────────────────────────────

async fn list_dimensions(
    Extension(db): Extension<DatabaseConnection>,
    Extension(_user): Extension<user::Model>,
    Path((tenant_id, id)): Path<(Uuid, Uuid)>,
) -> Result<impl IntoResponse, StatusCode> {
    // Ensure template belongs to path tenant
    let _ = load_template(&db, tenant_id, id).await?;

    let rows = dimensions::Entity::find()
        .filter(dimensions::Column::TemplateId.eq(id))
        .filter(dimensions::Column::TenantId.eq(tenant_id))
        .order_by_asc(dimensions::Column::SortOrder)
        .all(&db)
        .await
        .map_err(|e| {
            tracing::error!(%tenant_id, template_id = %id, "list_dimensions error: {e:#}");
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    Ok(Json(rows))
}

async fn create_dimension(
    Extension(db): Extension<DatabaseConnection>,
    Extension(_user): Extension<user::Model>,
    Path((tenant_id, id)): Path<(Uuid, Uuid)>,
    Json(input): Json<CreateDimensionInput>,
) -> Result<impl IntoResponse, StatusCode> {
    let template = load_template(&db, tenant_id, id).await?;
    let scale_type =
        ScaleType::try_from(input.scale_type).map_err(|_| StatusCode::UNPROCESSABLE_ENTITY)?;

    let model = dimensions::ActiveModel {
        id: Set(Uuid::new_v4()),
        template_id: Set(template.id),
        tenant_id: Set(tenant_id),
        slug: Set(input.slug),
        name: Set(input.name),
        description: Set(input.description),
        category: Set(input.category),
        weight: Set(input.weight.unwrap_or(dec("1.0"))),
        scale_type: Set(scale_type.to_string()),
        scale_min: Set(input.scale_min.unwrap_or(template.default_scale_min)),
        scale_max: Set(input.scale_max.unwrap_or(template.default_scale_max)),
        unit_label: Set(input.unit_label),
        benchmark_tiers: Set(input.benchmark_tiers.unwrap_or_else(|| serde_json::json!([]))),
        global_reference_value: Set(input.global_reference_value),
        global_reference_label: Set(input.global_reference_label),
        min_entries_to_show: Set(input.min_entries_to_show.unwrap_or(1)),
        is_community_ratable: Set(input.is_community_ratable.unwrap_or(true)),
        is_active: Set(input.is_active.unwrap_or(true)),
        sort_order: Set(input.sort_order.unwrap_or(0)),
        is_inverted: Set(input.is_inverted.unwrap_or(false)),
        bayesian_prior_weight: Set(input.bayesian_prior_weight),
        is_tenant_extension: Set(input.is_tenant_extension.unwrap_or(false)),
    };

    let inserted = model.insert(&db).await.map_err(|e| {
        tracing::error!(%tenant_id, template_id = %id, "create_dimension error: {e:#}");
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    Ok((StatusCode::CREATED, Json(inserted)))
}

async fn update_dimension(
    Extension(db): Extension<DatabaseConnection>,
    Extension(_user): Extension<user::Model>,
    Path((tenant_id, dim_id)): Path<(Uuid, Uuid)>,
    Json(input): Json<UpdateDimensionInput>,
) -> Result<impl IntoResponse, StatusCode> {
    let row = dimensions::Entity::find_by_id(dim_id)
        .one(&db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;

    if row.tenant_id != tenant_id {
        return Err(StatusCode::NOT_FOUND);
    }

    let mut am: dimensions::ActiveModel = row.into();

    if let Some(v) = input.name {
        am.name = Set(v);
    }
    if let Some(v) = input.description {
        am.description = Set(Some(v));
    }
    if let Some(v) = input.category {
        am.category = Set(Some(v));
    }
    if let Some(v) = input.weight {
        am.weight = Set(v);
    }
    if let Some(v) = input.scale_type {
        let scale = ScaleType::try_from(v).map_err(|_| StatusCode::UNPROCESSABLE_ENTITY)?;
        am.scale_type = Set(scale.to_string());
    }
    if let Some(v) = input.scale_min {
        am.scale_min = Set(v);
    }
    if let Some(v) = input.scale_max {
        am.scale_max = Set(v);
    }
    if let Some(v) = input.unit_label {
        am.unit_label = Set(Some(v));
    }
    if let Some(v) = input.benchmark_tiers {
        am.benchmark_tiers = Set(v);
    }
    if let Some(v) = input.global_reference_value {
        am.global_reference_value = Set(Some(v));
    }
    if let Some(v) = input.global_reference_label {
        am.global_reference_label = Set(Some(v));
    }
    if let Some(v) = input.min_entries_to_show {
        am.min_entries_to_show = Set(v);
    }
    if let Some(v) = input.is_community_ratable {
        am.is_community_ratable = Set(v);
    }
    if let Some(v) = input.is_active {
        am.is_active = Set(v);
    }
    if let Some(v) = input.sort_order {
        am.sort_order = Set(v);
    }
    if let Some(v) = input.is_inverted {
        am.is_inverted = Set(v);
    }
    if let Some(v) = input.bayesian_prior_weight {
        am.bayesian_prior_weight = Set(Some(v));
    }
    if let Some(v) = input.is_tenant_extension {
        am.is_tenant_extension = Set(v);
    }

    let updated = am.update(&db).await.map_err(|e| {
        tracing::error!(%tenant_id, %dim_id, "update_dimension error: {e:#}");
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    Ok(Json(updated))
}

// ── Scorecards ────────────────────────────────────────────────────────────────

async fn get_scorecard(
    Extension(db): Extension<DatabaseConnection>,
    Extension(_user): Extension<user::Model>,
    Path((tenant_id, id)): Path<(Uuid, Uuid)>,
) -> Result<impl IntoResponse, StatusCode> {
    let scorecard = load_scorecard(&db, tenant_id, id).await?;

    let dimension_aggregates = aggregates::Entity::find()
        .filter(aggregates::Column::ScorecardId.eq(id))
        .all(&db)
        .await
        .map_err(|e| {
            tracing::error!(%tenant_id, scorecard_id = %id, "get_scorecard aggregates error: {e:#}");
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    Ok(Json(ScorecardDetailResponse {
        scorecard,
        dimension_aggregates,
    }))
}

async fn list_sessions(
    Extension(db): Extension<DatabaseConnection>,
    Extension(_user): Extension<user::Model>,
    Path((tenant_id, id)): Path<(Uuid, Uuid)>,
) -> Result<impl IntoResponse, StatusCode> {
    let _ = load_scorecard(&db, tenant_id, id).await?;

    let rows = sessions::Entity::find()
        .filter(sessions::Column::ScorecardId.eq(id))
        .filter(sessions::Column::TenantId.eq(tenant_id))
        .order_by_desc(sessions::Column::OccurredAt)
        .all(&db)
        .await
        .map_err(|e| {
            tracing::error!(%tenant_id, scorecard_id = %id, "list_sessions error: {e:#}");
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    Ok(Json(rows))
}

async fn list_entries(
    Extension(db): Extension<DatabaseConnection>,
    Extension(_user): Extension<user::Model>,
    Path((tenant_id, id, sid)): Path<(Uuid, Uuid, Uuid)>,
) -> Result<impl IntoResponse, StatusCode> {
    let _ = load_scorecard(&db, tenant_id, id).await?;

    let session = sessions::Entity::find_by_id(sid)
        .one(&db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;

    if session.tenant_id != tenant_id || session.scorecard_id != id {
        return Err(StatusCode::NOT_FOUND);
    }

    let rows = entries::Entity::find()
        .filter(entries::Column::SessionId.eq(sid))
        .filter(entries::Column::TenantId.eq(tenant_id))
        .order_by_asc(entries::Column::CreatedAt)
        .all(&db)
        .await
        .map_err(|e| {
            tracing::error!(%tenant_id, session_id = %sid, "list_entries error: {e:#}");
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    Ok(Json(rows))
}

async fn list_time_series(
    Extension(db): Extension<DatabaseConnection>,
    Extension(_user): Extension<user::Model>,
    Path((tenant_id, id)): Path<(Uuid, Uuid)>,
    Query(params): Query<TimeSeriesQuery>,
) -> Result<impl IntoResponse, StatusCode> {
    let _ = load_scorecard(&db, tenant_id, id).await?;

    let mut q = time_series::Entity::find().filter(time_series::Column::ScorecardId.eq(id));

    if let Some(dim_id) = params.dimension_id {
        q = q.filter(time_series::Column::DimensionId.eq(dim_id));
    }
    if let Some(ref period_type) = params.period_type {
        if period_type != "monthly" && period_type != "quarterly" {
            return Err(StatusCode::UNPROCESSABLE_ENTITY);
        }
        q = q.filter(time_series::Column::PeriodType.eq(period_type.clone()));
    }

    let rows = q
        .order_by_asc(time_series::Column::PeriodStart)
        .all(&db)
        .await
        .map_err(|e| {
            tracing::error!(%tenant_id, scorecard_id = %id, "list_time_series error: {e:#}");
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    Ok(Json(rows))
}

async fn recompute_scorecard(
    Extension(db): Extension<DatabaseConnection>,
    Extension(_user): Extension<user::Model>,
    Path((tenant_id, id)): Path<(Uuid, Uuid)>,
) -> Result<impl IntoResponse, StatusCode> {
    let _ = load_scorecard(&db, tenant_id, id).await?;

    ScorecardService::recompute_aggregates(&db, id)
        .await
        .map_err(|e| {
            tracing::error!(%tenant_id, scorecard_id = %id, "recompute error: {e:#}");
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    Ok(StatusCode::NO_CONTENT)
}

async fn get_or_create_scorecard(
    Extension(db): Extension<DatabaseConnection>,
    Extension(_user): Extension<user::Model>,
    Path(tenant_id): Path<Uuid>,
    Json(input): Json<GetOrCreateInput>,
) -> Result<impl IntoResponse, StatusCode> {
    let entity_type = parse_entity_type(&input.subject_entity_type)?;
    let _ = load_template(&db, tenant_id, input.template_id).await?;

    let id = ScorecardService::get_or_create(
        &db,
        tenant_id,
        input.template_id,
        &entity_type,
        input.subject_entity_id,
    )
    .await
    .map_err(|e| {
        tracing::error!(%tenant_id, "get_or_create error: {e:#}");
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    Ok((StatusCode::OK, Json(GetOrCreateResponse { id })))
}

async fn open_session(
    Extension(db): Extension<DatabaseConnection>,
    Extension(current_user): Extension<user::Model>,
    Path((tenant_id, id)): Path<(Uuid, Uuid)>,
    Json(input): Json<OpenSessionInput>,
) -> Result<impl IntoResponse, StatusCode> {
    let _ = load_scorecard(&db, tenant_id, id).await?;
    let session_type =
        SessionType::try_from(input.session_type).map_err(|_| StatusCode::UNPROCESSABLE_ENTITY)?;

    let sid = ScorecardService::open_session(
        &db,
        id,
        current_user.id,
        tenant_id,
        input.occurred_at.unwrap_or_else(Utc::now),
        &session_type.to_string(),
        input.context_entity_type.as_deref(),
        input.context_entity_id,
        input.session_label.as_deref(),
        input.app_instance_id,
    )
    .await
    .map_err(|e| {
        tracing::error!(%tenant_id, scorecard_id = %id, "open_session error: {e:#}");
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    Ok((StatusCode::CREATED, Json(OpenSessionResponse { id: sid })))
}

async fn submit_entry(
    Extension(db): Extension<DatabaseConnection>,
    Extension(current_user): Extension<user::Model>,
    Path((tenant_id, sid)): Path<(Uuid, Uuid)>,
    Json(input): Json<SubmitEntryInput>,
) -> Result<impl IntoResponse, StatusCode> {
    let session = sessions::Entity::find_by_id(sid)
        .one(&db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;

    if session.tenant_id != tenant_id {
        return Err(StatusCode::NOT_FOUND);
    }
    if session.scorecard_id != input.scorecard_id {
        return Err(StatusCode::UNPROCESSABLE_ENTITY);
    }

    let source_type = match input.source_type.as_deref() {
        Some(s) => SourceType::try_from(s.to_owned()).map_err(|_| StatusCode::UNPROCESSABLE_ENTITY)?,
        None => SourceType::Manual,
    };

    let entry_id = ScorecardService::submit_entry(
        &db,
        sid,
        input.scorecard_id,
        input.dimension_id,
        tenant_id,
        current_user.id,
        input.score,
        input.option_id,
        &source_type.to_string(),
        input.context,
        input.note.as_deref(),
    )
    .await
    .map_err(|e| {
        tracing::error!(%tenant_id, session_id = %sid, "submit_entry error: {e:#}");
        // Unique constraint / validation → 422; other → 500
        let msg = format!("{e:#}");
        if msg.contains("exactly one") || msg.contains("already") || msg.contains("invalid") {
            StatusCode::UNPROCESSABLE_ENTITY
        } else {
            StatusCode::INTERNAL_SERVER_ERROR
        }
    })?;

    Ok((StatusCode::CREATED, Json(SubmitEntryResponse { id: entry_id })))
}

// ── Analytics wrappers ────────────────────────────────────────────────────────

async fn portfolio_stats(
    Extension(db): Extension<DatabaseConnection>,
    Extension(_user): Extension<user::Model>,
    Path((tenant_id, id)): Path<(Uuid, Uuid)>,
) -> Result<impl IntoResponse, StatusCode> {
    let _ = load_template(&db, tenant_id, id).await?;

    let stats = ScorecardAnalyticsService::portfolio_stats(&db, id, tenant_id)
        .await
        .map_err(|e| {
            tracing::error!(%tenant_id, template_id = %id, "portfolio_stats error: {e:#}");
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    Ok(Json(stats))
}

async fn leaderboard(
    Extension(db): Extension<DatabaseConnection>,
    Extension(_user): Extension<user::Model>,
    Path((tenant_id, id)): Path<(Uuid, Uuid)>,
    Query(params): Query<LeaderboardQuery>,
) -> Result<impl IntoResponse, StatusCode> {
    let _ = load_template(&db, tenant_id, id).await?;
    let limit = params.limit.unwrap_or(25);

    let entries = ScorecardAnalyticsService::leaderboard(&db, id, tenant_id, limit)
        .await
        .map_err(|e| {
            tracing::error!(%tenant_id, template_id = %id, "leaderboard error: {e:#}");
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    Ok(Json(entries))
}

async fn anomalies(
    Extension(db): Extension<DatabaseConnection>,
    Extension(_user): Extension<user::Model>,
    Path((tenant_id, id)): Path<(Uuid, Uuid)>,
    Query(params): Query<AnomalyQuery>,
) -> Result<impl IntoResponse, StatusCode> {
    let _ = load_template(&db, tenant_id, id).await?;
    let limit = params.limit.unwrap_or(50);

    let alerts = ScorecardAnalyticsService::recent_anomalies(&db, id, tenant_id, limit)
        .await
        .map_err(|e| {
            tracing::error!(%tenant_id, template_id = %id, "anomalies error: {e:#}");
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    Ok(Json(alerts))
}

async fn refresh_analytics(
    Extension(db): Extension<DatabaseConnection>,
    Extension(current_user): Extension<user::Model>,
    Path((tenant_id, id)): Path<(Uuid, Uuid)>,
) -> Result<impl IntoResponse, StatusCode> {
    let _ = load_template(&db, tenant_id, id).await?;

    let started = std::time::Instant::now();
    ScorecardAnalyticsService::refresh_and_rerank(&db, id, tenant_id)
        .await
        .map_err(|e| {
            tracing::error!(%tenant_id, template_id = %id, "analytics refresh error: {e:#}");
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    tracing::info!(
        %tenant_id,
        template_id = %id,
        duration_ms = started.elapsed().as_millis(),
        user_id = %current_user.id,
        "Admin on-demand portfolio analytics refresh completed"
    );

    Ok(StatusCode::NO_CONTENT)
}

// ── Deployments (Phase 1b) ────────────────────────────────────────────────────

/// GET /api/admin/tenants/{tenant_id}/app-instances/{instance_id}/scorecard-deployments
async fn list_instance_deployments(
    Extension(db): Extension<DatabaseConnection>,
    Extension(_user): Extension<user::Model>,
    Path((tenant_id, instance_id)): Path<(Uuid, Uuid)>,
) -> Result<impl IntoResponse, StatusCode> {
    let _ = load_instance(&db, tenant_id, instance_id).await?;

    let rows = deployments::Entity::find()
        .filter(deployments::Column::AppInstanceId.eq(instance_id))
        .filter(deployments::Column::TenantId.eq(tenant_id))
        .order_by_asc(deployments::Column::CreatedAt)
        .all(&db)
        .await
        .map_err(|e| {
            tracing::error!(%tenant_id, %instance_id, "list_instance_deployments error: {e:#}");
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    let mut views = Vec::with_capacity(rows.len());
    for row in rows {
        let template_name = templates::Entity::find_by_id(row.template_id)
            .one(&db)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
            .map(|t| t.name);
        views.push(DeploymentView {
            deployment: row,
            template_name,
        });
    }

    Ok(Json(views))
}

/// PUT /api/admin/tenants/{tenant_id}/app-instances/{instance_id}/scorecard-deployments
///
/// Body: `{ deployments: [{ template_id, is_enabled, trigger_event? }] }` — upsert by
/// `(template_id, app_instance_id)`.
async fn upsert_instance_deployments(
    Extension(db): Extension<DatabaseConnection>,
    Extension(_user): Extension<user::Model>,
    Path((tenant_id, instance_id)): Path<(Uuid, Uuid)>,
    Json(input): Json<UpsertDeploymentsInput>,
) -> Result<impl IntoResponse, StatusCode> {
    let _ = load_instance(&db, tenant_id, instance_id).await?;

    let mut results = Vec::with_capacity(input.deployments.len());
    for item in input.deployments {
        // Template must belong to the same tenant.
        let _ = load_template(&db, tenant_id, item.template_id).await?;

        let existing = deployments::Entity::find()
            .filter(deployments::Column::TemplateId.eq(item.template_id))
            .filter(deployments::Column::AppInstanceId.eq(instance_id))
            .one(&db)
            .await
            .map_err(|e| {
                tracing::error!(%tenant_id, %instance_id, "upsert find error: {e:#}");
                StatusCode::INTERNAL_SERVER_ERROR
            })?;

        let trigger_event = item
            .trigger_event
            .unwrap_or_else(|| "manual".to_string());

        let saved = if let Some(row) = existing {
            let mut am: deployments::ActiveModel = row.into();
            am.is_enabled = Set(item.is_enabled);
            am.trigger_event = Set(trigger_event);
            if item.trigger_context_entity_type.is_some() {
                am.trigger_context_entity_type = Set(item.trigger_context_entity_type);
            }
            am.update(&db).await.map_err(|e| {
                tracing::error!(%tenant_id, %instance_id, "upsert update error: {e:#}");
                StatusCode::INTERNAL_SERVER_ERROR
            })?
        } else {
            let am = deployments::ActiveModel {
                id: Set(Uuid::new_v4()),
                template_id: Set(item.template_id),
                app_instance_id: Set(instance_id),
                tenant_id: Set(tenant_id),
                is_enabled: Set(item.is_enabled),
                trigger_event: Set(trigger_event),
                trigger_context_entity_type: Set(item.trigger_context_entity_type),
                created_at: Set(Utc::now()),
            };
            am.insert(&db).await.map_err(|e| {
                tracing::error!(%tenant_id, %instance_id, "upsert insert error: {e:#}");
                StatusCode::INTERNAL_SERVER_ERROR
            })?
        };

        let template_name = templates::Entity::find_by_id(saved.template_id)
            .one(&db)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
            .map(|t| t.name);

        results.push(DeploymentView {
            deployment: saved,
            template_name,
        });
    }

    Ok(Json(results))
}

/// GET /api/admin/scorecard-templates/{id}/deployments
async fn list_template_deployments(
    Extension(db): Extension<DatabaseConnection>,
    Extension(_user): Extension<user::Model>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, StatusCode> {
    let _ = templates::Entity::find_by_id(id)
        .one(&db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;

    let rows = deployments::Entity::find()
        .filter(deployments::Column::TemplateId.eq(id))
        .order_by_asc(deployments::Column::CreatedAt)
        .all(&db)
        .await
        .map_err(|e| {
            tracing::error!(template_id = %id, "list_template_deployments error: {e:#}");
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    Ok(Json(rows))
}

/// Templates enabled for an app instance (helper for Configurator / app list filters).
///
/// Prefer `ScorecardService::templates_enabled_for_instance` for new call sites.
/// This wrapper keeps the admin module's historical signature and requires
/// `tenant_id` for isolation.
pub async fn templates_enabled_for_instance(
    db: &DatabaseConnection,
    tenant_id: Uuid,
    app_instance_id: Uuid,
) -> Result<Vec<templates::Model>, StatusCode> {
    ScorecardService::templates_enabled_for_instance(db, tenant_id, app_instance_id, None)
        .await
        .map_err(|e| {
            tracing::error!(%tenant_id, %app_instance_id, "templates_enabled_for_instance error: {e:#}");
            StatusCode::INTERNAL_SERVER_ERROR
        })
}

// ── Helpers ───────────────────────────────────────────────────────────────────

fn dec(s: &str) -> Decimal {
    s.parse().expect("static decimal literal")
}

/// Accept global G-27 entity types plus PM provisioner vocabulary.
fn parse_entity_type(raw: &str) -> Result<String, StatusCode> {
    if ScorecardEntityType::try_from(raw).is_ok() {
        return Ok(raw.to_owned());
    }
    if crate::types::pm::ScorecardEntityType::try_from(raw.to_owned()).is_ok() {
        return Ok(raw.to_owned());
    }
    Err(StatusCode::UNPROCESSABLE_ENTITY)
}

async fn load_template(
    db: &DatabaseConnection,
    tenant_id: Uuid,
    id: Uuid,
) -> Result<templates::Model, StatusCode> {
    let row = templates::Entity::find_by_id(id)
        .one(db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;

    if row.tenant_id != tenant_id {
        return Err(StatusCode::NOT_FOUND);
    }
    Ok(row)
}

async fn load_instance(
    db: &DatabaseConnection,
    tenant_id: Uuid,
    instance_id: Uuid,
) -> Result<app_instance::Model, StatusCode> {
    let row = app_instance::Entity::find_by_id(instance_id)
        .one(db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;

    if row.tenant_id != tenant_id {
        return Err(StatusCode::NOT_FOUND);
    }
    Ok(row)
}

async fn load_scorecard(
    db: &DatabaseConnection,
    tenant_id: Uuid,
    id: Uuid,
) -> Result<scorecards::Model, StatusCode> {
    let row = scorecards::Entity::find_by_id(id)
        .filter(scorecards::Column::DeletedAt.is_null())
        .one(db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;

    if row.tenant_id != tenant_id {
        return Err(StatusCode::NOT_FOUND);
    }
    Ok(row)
}
