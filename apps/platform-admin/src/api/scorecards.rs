//! G-27 scorecard platform-admin API client.
//!
//! Mirrors `backend/src/handlers/scorecard_admin.rs` response shapes.
//! Contract: `docs/contracts/g27_scorecard_platform.md` §6.

use crate::api::client::{api_get, api_post, api_put, api_request, api_url, create_client};
use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// ── Template ──────────────────────────────────────────────────────────────────

/// Matches `atlas_scorecard_templates` Model JSON (Decimal fields as strings).
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ScorecardTemplate {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub name: String,
    pub entity_type: String,
    pub description: Option<String>,
    pub scoring_method: String,
    pub default_scale_min: String,
    pub default_scale_max: String,
    pub min_entries_to_publish: i32,
    pub is_published: bool,
    pub template_scope: String,
    pub cold_start_strategy: String,
    pub cold_start_saturation_threshold: i32,
    pub default_bayesian_prior_weight: Option<String>,
    pub calibration_minimum_entries: i32,
    pub display_config: Option<serde_json::Value>,
    pub created_by_user_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Clone, Debug, Serialize, Default)]
pub struct CreateTemplateInput {
    pub name: String,
    pub entity_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scoring_method: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default_scale_min: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default_scale_max: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_entries_to_publish: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_published: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub template_scope: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cold_start_strategy: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cold_start_saturation_threshold: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default_bayesian_prior_weight: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub calibration_minimum_entries: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_config: Option<serde_json::Value>,
}

#[derive(Clone, Debug, Serialize, Default)]
pub struct UpdateTemplateInput {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scoring_method: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default_scale_min: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default_scale_max: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_entries_to_publish: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_published: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub template_scope: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cold_start_strategy: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cold_start_saturation_threshold: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default_bayesian_prior_weight: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub calibration_minimum_entries: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_config: Option<serde_json::Value>,
}

// ── Dimension ─────────────────────────────────────────────────────────────────

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ScorecardDimension {
    pub id: Uuid,
    pub template_id: Uuid,
    pub tenant_id: Uuid,
    pub slug: String,
    pub name: String,
    pub description: Option<String>,
    pub category: Option<String>,
    pub weight: String,
    pub scale_type: String,
    pub scale_min: String,
    pub scale_max: String,
    pub unit_label: Option<String>,
    pub benchmark_tiers: serde_json::Value,
    pub global_reference_value: Option<String>,
    pub global_reference_label: Option<String>,
    pub min_entries_to_show: i32,
    pub is_community_ratable: bool,
    pub is_active: bool,
    pub sort_order: i32,
    pub is_inverted: bool,
    pub bayesian_prior_weight: Option<String>,
    pub is_tenant_extension: bool,
}

#[derive(Clone, Debug, Serialize, Default)]
pub struct CreateDimensionInput {
    pub slug: String,
    pub name: String,
    pub scale_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub category: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub weight: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scale_min: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scale_max: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub unit_label: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub benchmark_tiers: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub global_reference_value: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub global_reference_label: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_entries_to_show: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_community_ratable: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_active: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sort_order: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_inverted: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bayesian_prior_weight: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_tenant_extension: Option<bool>,
}

#[derive(Clone, Debug, Serialize, Default)]
pub struct UpdateDimensionInput {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub category: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub weight: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scale_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scale_min: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scale_max: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub unit_label: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub benchmark_tiers: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub global_reference_value: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub global_reference_label: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_entries_to_show: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_community_ratable: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_active: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sort_order: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_inverted: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bayesian_prior_weight: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_tenant_extension: Option<bool>,
}

// ── Scorecard / session / entry ───────────────────────────────────────────────

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ScorecardModel {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub template_id: Uuid,
    pub subject_entity_type: String,
    pub subject_entity_id: Uuid,
    pub composite_score: Option<String>,
    pub confidence_level: String,
    pub total_contributors: i32,
    pub total_sessions: i32,
    pub total_entries: i32,
    pub dimension_vector: Option<serde_json::Value>,
    pub dimension_vector_v2: Option<serde_json::Value>,
    pub has_data_mask: Option<serde_json::Value>,
    pub last_computed_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DimensionAggregate {
    pub scorecard_id: Uuid,
    pub dimension_id: Uuid,
    pub mean_score: Option<String>,
    pub weighted_mean_score: Option<String>,
    pub percent_true: Option<String>,
    pub benchmark_label: Option<String>,
    pub benchmark_color: Option<String>,
    pub display_value: Option<String>,
    pub std_deviation: Option<String>,
    pub consensus_level: Option<String>,
    pub min_score: Option<String>,
    pub max_score: Option<String>,
    pub contributor_count: i32,
    pub session_count: i32,
    pub vs_global_delta: Option<String>,
    pub vs_global_label: Option<String>,
    pub percentile_rank: Option<String>,
    pub percentile_cohort_size: Option<i32>,
    pub percentile_band: Option<String>,
    pub last_computed_at: Option<DateTime<Utc>>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ScorecardDetail {
    #[serde(flatten)]
    pub scorecard: ScorecardModel,
    pub dimension_aggregates: Vec<DimensionAggregate>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RatingSession {
    pub id: Uuid,
    pub scorecard_id: Uuid,
    pub tenant_id: Uuid,
    pub rater_user_id: Uuid,
    pub occurred_at: DateTime<Utc>,
    pub session_type: String,
    pub context_entity_type: Option<String>,
    pub context_entity_id: Option<Uuid>,
    pub session_label: Option<String>,
    pub status: String,
    pub verification_request_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ScorecardEntry {
    pub id: Uuid,
    pub session_id: Uuid,
    pub scorecard_id: Uuid,
    pub dimension_id: Uuid,
    pub tenant_id: Uuid,
    pub contributor_user_id: Uuid,
    pub score: Option<String>,
    pub option_id: Option<Uuid>,
    pub source_type: String,
    pub context: Option<serde_json::Value>,
    pub note: Option<String>,
    pub is_verified: bool,
    pub verification_request_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TimeSeriesPoint {
    pub scorecard_id: Uuid,
    pub dimension_id: Uuid,
    pub period_start: NaiveDate,
    pub period_type: String,
    pub mean_score: Option<String>,
    pub session_count: i32,
    pub contributor_count: i32,
    pub delta_from_prior: Option<String>,
    pub trend_direction: Option<String>,
    pub z_score: Option<String>,
    pub is_anomaly: bool,
    pub anomaly_direction: Option<String>,
}

#[derive(Clone, Debug, Serialize)]
pub struct GetOrCreateInput {
    pub template_id: Uuid,
    pub subject_entity_type: String,
    pub subject_entity_id: Uuid,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GetOrCreateResponse {
    pub id: Uuid,
}

#[derive(Clone, Debug, Serialize)]
pub struct OpenSessionInput {
    pub session_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub occurred_at: Option<DateTime<Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context_entity_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context_entity_id: Option<Uuid>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub session_label: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct OpenSessionResponse {
    pub id: Uuid,
}

#[derive(Clone, Debug, Serialize)]
pub struct SubmitEntryInput {
    pub scorecard_id: Uuid,
    pub dimension_id: Uuid,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub score: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub option_id: Option<Uuid>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SubmitEntryResponse {
    pub id: Uuid,
}

// ── Analytics ─────────────────────────────────────────────────────────────────

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DimensionPortfolioStats {
    pub dimension_id: Uuid,
    pub dimension_slug: String,
    pub dimension_name: String,
    pub cohort_size: i64,
    pub pool_mean: Option<f64>,
    pub pool_std_dev: Option<f64>,
    pub pool_min: Option<f64>,
    pub pool_p25: Option<f64>,
    pub pool_p50: Option<f64>,
    pub pool_p75: Option<f64>,
    pub pool_p90: Option<f64>,
    pub pool_max: Option<f64>,
    pub improving_count: i64,
    pub declining_count: i64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PortfolioStats {
    pub template_id: Uuid,
    pub tenant_id: Uuid,
    pub total_scorecards: i64,
    pub refreshed_at: Option<DateTime<Utc>>,
    pub dimensions: Vec<DimensionPortfolioStats>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LeaderboardEntry {
    pub rank: i64,
    pub scorecard_id: Uuid,
    pub subject_entity_id: String,
    pub subject_entity_type: String,
    pub composite_score: Option<f64>,
    pub confidence_level: String,
    pub percentile_rank: Option<f64>,
    pub trend_direction: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AnomalyAlert {
    pub scorecard_id: Uuid,
    pub dimension_id: Uuid,
    pub dimension_slug: String,
    pub dimension_name: String,
    pub period_start: NaiveDate,
    pub mean_score: Option<f64>,
    pub z_score: Option<f64>,
    pub anomaly_direction: Option<String>,
}

// ── Deployments ───────────────────────────────────────────────────────────────

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ScorecardDeployment {
    pub id: Uuid,
    pub template_id: Uuid,
    pub app_instance_id: Uuid,
    pub tenant_id: Uuid,
    pub is_enabled: bool,
    pub trigger_event: String,
    pub trigger_context_entity_type: Option<String>,
    pub created_at: DateTime<Utc>,
    #[serde(default)]
    pub template_name: Option<String>,
}

#[derive(Clone, Debug, Serialize)]
pub struct UpsertDeploymentItem {
    pub template_id: Uuid,
    pub is_enabled: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trigger_event: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trigger_context_entity_type: Option<String>,
}

#[derive(Clone, Debug, Serialize)]
pub struct UpsertDeploymentsInput {
    pub deployments: Vec<UpsertDeploymentItem>,
}

// ── API functions ─────────────────────────────────────────────────────────────

pub async fn list_catalog(is_published: Option<bool>) -> Result<Vec<ScorecardTemplate>, String> {
    let path = match is_published {
        Some(v) => format!("api/admin/scorecard-templates/catalog?is_published={v}"),
        None => "api/admin/scorecard-templates/catalog".to_string(),
    };
    api_get(&path).await
}

pub async fn list_templates(tenant_id: &str) -> Result<Vec<ScorecardTemplate>, String> {
    api_get(&format!("api/admin/tenants/{tenant_id}/scorecard-templates")).await
}

pub async fn get_template(tenant_id: &str, id: &str) -> Result<ScorecardTemplate, String> {
    api_get(&format!("api/admin/tenants/{tenant_id}/scorecard-templates/{id}")).await
}

pub async fn create_template(
    tenant_id: &str,
    input: &CreateTemplateInput,
) -> Result<ScorecardTemplate, String> {
    api_post(
        &format!("api/admin/tenants/{tenant_id}/scorecard-templates"),
        input,
    )
    .await
}

pub async fn update_template(
    tenant_id: &str,
    id: &str,
    input: &UpdateTemplateInput,
) -> Result<ScorecardTemplate, String> {
    let client = create_client();
    let url = api_url(&format!(
        "api/admin/tenants/{tenant_id}/scorecard-templates/{id}"
    ));
    let req = client.patch(&url).json(input);
    api_request(req).await
}

pub async fn list_dimensions(
    tenant_id: &str,
    template_id: &str,
) -> Result<Vec<ScorecardDimension>, String> {
    api_get(&format!(
        "api/admin/tenants/{tenant_id}/scorecard-templates/{template_id}/dimensions"
    ))
    .await
}

pub async fn create_dimension(
    tenant_id: &str,
    template_id: &str,
    input: &CreateDimensionInput,
) -> Result<ScorecardDimension, String> {
    api_post(
        &format!("api/admin/tenants/{tenant_id}/scorecard-templates/{template_id}/dimensions"),
        input,
    )
    .await
}

pub async fn update_dimension(
    tenant_id: &str,
    dim_id: &str,
    input: &UpdateDimensionInput,
) -> Result<ScorecardDimension, String> {
    let client = create_client();
    let url = api_url(&format!(
        "api/admin/tenants/{tenant_id}/scorecard-dimensions/{dim_id}"
    ));
    let req = client.patch(&url).json(input);
    api_request(req).await
}

pub async fn get_analytics(tenant_id: &str, template_id: &str) -> Result<PortfolioStats, String> {
    api_get(&format!(
        "api/admin/tenants/{tenant_id}/scorecard-templates/{template_id}/analytics"
    ))
    .await
}

pub async fn get_leaderboard(
    tenant_id: &str,
    template_id: &str,
    limit: Option<i64>,
) -> Result<Vec<LeaderboardEntry>, String> {
    let path = match limit {
        Some(n) => format!(
            "api/admin/tenants/{tenant_id}/scorecard-templates/{template_id}/leaderboard?limit={n}"
        ),
        None => format!(
            "api/admin/tenants/{tenant_id}/scorecard-templates/{template_id}/leaderboard"
        ),
    };
    api_get(&path).await
}

pub async fn get_anomalies(
    tenant_id: &str,
    template_id: &str,
    limit: Option<i64>,
) -> Result<Vec<AnomalyAlert>, String> {
    let path = match limit {
        Some(n) => format!(
            "api/admin/tenants/{tenant_id}/scorecard-templates/{template_id}/anomalies?limit={n}"
        ),
        None => format!(
            "api/admin/tenants/{tenant_id}/scorecard-templates/{template_id}/anomalies"
        ),
    };
    api_get(&path).await
}

pub async fn refresh_analytics(tenant_id: &str, template_id: &str) -> Result<(), String> {
    let client = create_client();
    let url = api_url(&format!(
        "api/admin/tenants/{tenant_id}/scorecard-templates/{template_id}/analytics/refresh"
    ));
    let req = client.post(&url);
    // 204 No Content — api_request expects JSON; handle empty body.
    let req = crate::api::client::with_credentials(req);
    let res = req.send().await.map_err(|e| e.to_string())?;
    if res.status().is_success() {
        Ok(())
    } else {
        Err(format!("refresh failed: {}", res.status()))
    }
}

pub async fn get_scorecard(tenant_id: &str, id: &str) -> Result<ScorecardDetail, String> {
    api_get(&format!("api/admin/tenants/{tenant_id}/scorecards/{id}")).await
}

pub async fn list_sessions(tenant_id: &str, scorecard_id: &str) -> Result<Vec<RatingSession>, String> {
    api_get(&format!(
        "api/admin/tenants/{tenant_id}/scorecards/{scorecard_id}/sessions"
    ))
    .await
}

pub async fn list_entries(
    tenant_id: &str,
    scorecard_id: &str,
    session_id: &str,
) -> Result<Vec<ScorecardEntry>, String> {
    api_get(&format!(
        "api/admin/tenants/{tenant_id}/scorecards/{scorecard_id}/sessions/{session_id}/entries"
    ))
    .await
}

pub async fn list_time_series(
    tenant_id: &str,
    scorecard_id: &str,
    dimension_id: Option<&str>,
    period_type: Option<&str>,
) -> Result<Vec<TimeSeriesPoint>, String> {
    let mut path = format!(
        "api/admin/tenants/{tenant_id}/scorecards/{scorecard_id}/time-series"
    );
    let mut qs = Vec::new();
    if let Some(d) = dimension_id {
        qs.push(format!("dimension_id={d}"));
    }
    if let Some(p) = period_type {
        qs.push(format!("period_type={p}"));
    }
    if !qs.is_empty() {
        path.push('?');
        path.push_str(&qs.join("&"));
    }
    api_get(&path).await
}

pub async fn recompute(tenant_id: &str, scorecard_id: &str) -> Result<(), String> {
    let client = create_client();
    let url = api_url(&format!(
        "api/admin/tenants/{tenant_id}/scorecards/{scorecard_id}/recompute"
    ));
    let req = crate::api::client::with_credentials(client.post(&url));
    let res = req.send().await.map_err(|e| e.to_string())?;
    if res.status().is_success() {
        Ok(())
    } else {
        Err(format!("recompute failed: {}", res.status()))
    }
}

pub async fn get_or_create_scorecard(
    tenant_id: &str,
    input: &GetOrCreateInput,
) -> Result<GetOrCreateResponse, String> {
    api_post(
        &format!("api/admin/tenants/{tenant_id}/scorecards/get-or-create"),
        input,
    )
    .await
}

pub async fn open_session(
    tenant_id: &str,
    scorecard_id: &str,
    input: &OpenSessionInput,
) -> Result<OpenSessionResponse, String> {
    api_post(
        &format!("api/admin/tenants/{tenant_id}/scorecards/{scorecard_id}/sessions"),
        input,
    )
    .await
}

pub async fn submit_entry(
    tenant_id: &str,
    session_id: &str,
    input: &SubmitEntryInput,
) -> Result<SubmitEntryResponse, String> {
    api_post(
        &format!("api/admin/tenants/{tenant_id}/scorecard-sessions/{session_id}/entries"),
        input,
    )
    .await
}

pub async fn list_instance_deployments(
    tenant_id: &str,
    instance_id: &str,
) -> Result<Vec<ScorecardDeployment>, String> {
    api_get(&format!(
        "api/admin/tenants/{tenant_id}/app-instances/{instance_id}/scorecard-deployments"
    ))
    .await
}

pub async fn upsert_instance_deployments(
    tenant_id: &str,
    instance_id: &str,
    input: &UpsertDeploymentsInput,
) -> Result<Vec<ScorecardDeployment>, String> {
    api_put(
        &format!(
            "api/admin/tenants/{tenant_id}/app-instances/{instance_id}/scorecard-deployments"
        ),
        input,
    )
    .await
}

pub async fn list_template_deployments(
    template_id: &str,
) -> Result<Vec<ScorecardDeployment>, String> {
    api_get(&format!(
        "api/admin/scorecard-templates/{template_id}/deployments"
    ))
    .await
}
