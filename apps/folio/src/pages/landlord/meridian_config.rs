//! G-27 Meridian — `/l/meridian/configure`
//!
//! Tabs:
//!   1. Dashboard — portfolio analytics / leaderboard / anomalies
//!   2. Configure — shared-ui `Configurator` in `TenantAdmin` mode
//!
//! Templates come from deployed-only `GET /api/scorecard-templates`.
//! Saves use tenant write APIs (display_config + display rules).

use leptos::prelude::*;
use leptos::task::spawn_local;
use serde::{Deserialize, Serialize};
use shared_ui::components::configurator::Configurator;
use shared_ui::components::scorecard::models::{
    ColdStartStrategy, ConfiguratorMode, DimensionForm, DisplayConfigForm, DisplayRuleForm,
    ModeScope, RuleAction, RuleOperator, ScaleType, ScoringMethod, TemplateForm,
    TemplateSavePayload, TemplateScope, TriggerCategory,
};
use uuid::Uuid;

use crate::components::page_header::PageHeader;

// ── Types ─────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScorecardTemplate {
    pub id: Uuid,
    pub name: String,
    pub entity_type: String,
    pub is_active: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateDetailDto {
    pub id: Uuid,
    pub name: String,
    pub entity_type: String,
    pub description: Option<String>,
    pub scoring_method: ScoringMethod,
    #[serde(deserialize_with = "deserialize_loose_number")]
    pub default_scale_min: String,
    #[serde(deserialize_with = "deserialize_loose_number")]
    pub default_scale_max: String,
    pub min_entries_to_publish: i32,
    pub is_published: bool,
    pub template_scope: TemplateScope,
    pub cold_start_strategy: ColdStartStrategy,
    pub cold_start_saturation_threshold: i32,
    #[serde(default, deserialize_with = "deserialize_opt_loose_number")]
    pub default_bayesian_prior_weight: Option<String>,
    pub calibration_minimum_entries: i32,
    pub display_config: Option<serde_json::Value>,
}

fn deserialize_loose_number<'de, D: serde::Deserializer<'de>>(d: D) -> Result<String, D::Error> {
    let v = serde_json::Value::deserialize(d)?;
    Ok(match v {
        serde_json::Value::String(s) => s,
        serde_json::Value::Number(n) => n.to_string(),
        other => other.to_string(),
    })
}

fn deserialize_opt_loose_number<'de, D: serde::Deserializer<'de>>(
    d: D,
) -> Result<Option<String>, D::Error> {
    let v = Option::<serde_json::Value>::deserialize(d)?;
    Ok(v.map(|v| match v {
        serde_json::Value::String(s) => s,
        serde_json::Value::Number(n) => n.to_string(),
        other => other.to_string(),
    }))
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DimensionDto {
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DisplayRuleDto {
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortfolioStats {
    pub cohort_size: i64,
    pub mean_score: Option<f64>,
    pub median_score: Option<f64>,
    pub anomaly_count_30d: Option<i64>,
    pub dimensions: Vec<DimensionStat>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DimensionStat {
    pub dimension_id: Uuid,
    pub dimension_name: String,
    pub mean_score: Option<f64>,
    pub trend: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LeaderboardEntry {
    pub rank: i64,
    pub entity_id: Uuid,
    pub entity_label: Option<String>,
    pub composite_score: f64,
    pub percentile_rank: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnomalyAlert {
    pub id: Uuid,
    pub entity_id: Uuid,
    pub entity_label: Option<String>,
    pub dimension_name: Option<String>,
    pub score: f64,
    pub detected_at: String,
    pub alert_message: Option<String>,
}

#[derive(Debug, Serialize)]
struct PatchTemplateBody {
    description: Option<String>,
    display_config: Option<serde_json::Value>,
}

#[derive(Debug, Serialize)]
struct CreateRuleBody {
    dimension_id: Option<Uuid>,
    category_target: Option<String>,
    trigger_category: TriggerCategory,
    field_reference: Option<String>,
    operator: RuleOperator,
    value: Option<String>,
    value_list: Option<serde_json::Value>,
    action: RuleAction,
    alert_message: Option<String>,
    mode_scope: Option<ModeScope>,
    priority: Option<i32>,
    description: Option<String>,
}

#[derive(Debug, Serialize)]
struct UpdateRuleBody {
    dimension_id: Option<Uuid>,
    category_target: Option<String>,
    trigger_category: Option<TriggerCategory>,
    field_reference: Option<String>,
    operator: Option<RuleOperator>,
    value: Option<String>,
    value_list: Option<serde_json::Value>,
    action: Option<RuleAction>,
    alert_message: Option<String>,
    mode_scope: Option<ModeScope>,
    priority: Option<i32>,
    description: Option<String>,
    is_active: Option<bool>,
}

// ── Server fns ────────────────────────────────────────────────────────────────

#[cfg(feature = "ssr")]
fn session_token(
    headers: &axum::http::HeaderMap,
) -> Result<String, server_fn::error::ServerFnError> {
    headers
        .get("cookie")
        .and_then(|v| v.to_str().ok())
        .and_then(|s| {
            s.split(';').find_map(|p| {
                let p = p.trim();
                p.strip_prefix("session=").map(|t| t.to_string())
            })
        })
        .ok_or_else(|| server_fn::error::ServerFnError::new("No session token"))
}

#[server(FetchG27Templates, "/api")]
pub async fn fetch_g27_templates() -> Result<Vec<ScorecardTemplate>, server_fn::error::ServerFnError>
{
    use axum::http::HeaderMap;
    use leptos_axum::extract;
    let headers = extract::<HeaderMap>().await.unwrap_or_default();
    let token = session_token(&headers)?;
    crate::atlas_client::authenticated_get::<Vec<ScorecardTemplate>>(
        "/api/scorecard-templates",
        &token,
        None,
    )
    .await
    .map_err(|e| server_fn::error::ServerFnError::new(e.to_string()))
}

#[server(FetchG27Analytics, "/api")]
pub async fn fetch_g27_analytics(
    template_id: String,
) -> Result<PortfolioStats, server_fn::error::ServerFnError> {
    use axum::http::HeaderMap;
    use leptos_axum::extract;
    let headers = extract::<HeaderMap>().await.unwrap_or_default();
    let token = session_token(&headers)?;
    let url = format!("/api/scorecard-templates/{template_id}/analytics");
    crate::atlas_client::authenticated_get::<PortfolioStats>(&url, &token, None)
        .await
        .map_err(|e| server_fn::error::ServerFnError::new(e.to_string()))
}

#[server(FetchG27Leaderboard, "/api")]
pub async fn fetch_g27_leaderboard(
    template_id: String,
) -> Result<Vec<LeaderboardEntry>, server_fn::error::ServerFnError> {
    use axum::http::HeaderMap;
    use leptos_axum::extract;
    let headers = extract::<HeaderMap>().await.unwrap_or_default();
    let token = session_token(&headers)?;
    let url = format!("/api/scorecard-templates/{template_id}/leaderboard?limit=10");
    crate::atlas_client::authenticated_get::<Vec<LeaderboardEntry>>(&url, &token, None)
        .await
        .map_err(|e| server_fn::error::ServerFnError::new(e.to_string()))
}

#[server(FetchG27Anomalies, "/api")]
pub async fn fetch_g27_anomalies(
    template_id: String,
) -> Result<Vec<AnomalyAlert>, server_fn::error::ServerFnError> {
    use axum::http::HeaderMap;
    use leptos_axum::extract;
    let headers = extract::<HeaderMap>().await.unwrap_or_default();
    let token = session_token(&headers)?;
    let url = format!("/api/scorecard-templates/{template_id}/anomalies?limit=20");
    crate::atlas_client::authenticated_get::<Vec<AnomalyAlert>>(&url, &token, None)
        .await
        .map_err(|e| server_fn::error::ServerFnError::new(e.to_string()))
}

#[server(FetchMeridianTemplateDetail, "/api")]
pub async fn fetch_meridian_template_detail(
    template_id: String,
) -> Result<TemplateDetailDto, server_fn::error::ServerFnError> {
    use axum::http::HeaderMap;
    use leptos_axum::extract;
    let headers = extract::<HeaderMap>().await.unwrap_or_default();
    let token = session_token(&headers)?;
    let url = format!("/api/scorecard-templates/{template_id}");
    crate::atlas_client::authenticated_get::<TemplateDetailDto>(&url, &token, None)
        .await
        .map_err(|e| server_fn::error::ServerFnError::new(e.to_string()))
}

#[server(FetchMeridianDimensions, "/api")]
pub async fn fetch_meridian_dimensions(
    template_id: String,
) -> Result<Vec<DimensionDto>, server_fn::error::ServerFnError> {
    use axum::http::HeaderMap;
    use leptos_axum::extract;
    let headers = extract::<HeaderMap>().await.unwrap_or_default();
    let token = session_token(&headers)?;
    let url = format!("/api/scorecard-templates/{template_id}/dimensions");
    crate::atlas_client::authenticated_get::<Vec<DimensionDto>>(&url, &token, None)
        .await
        .map_err(|e| server_fn::error::ServerFnError::new(e.to_string()))
}

#[server(FetchMeridianDisplayRules, "/api")]
pub async fn fetch_meridian_display_rules(
    template_id: String,
) -> Result<Vec<DisplayRuleDto>, server_fn::error::ServerFnError> {
    use axum::http::HeaderMap;
    use leptos_axum::extract;
    let headers = extract::<HeaderMap>().await.unwrap_or_default();
    let token = session_token(&headers)?;
    let url = format!("/api/scorecard-templates/{template_id}/display-rules");
    crate::atlas_client::authenticated_get::<Vec<DisplayRuleDto>>(&url, &token, None)
        .await
        .map_err(|e| server_fn::error::ServerFnError::new(e.to_string()))
}

#[server(SaveMeridianConfigurator, "/api")]
pub async fn save_meridian_configurator(
    template_id: String,
    description: String,
    display_config: serde_json::Value,
    rules_json: String,
) -> Result<(), server_fn::error::ServerFnError> {
    use axum::http::HeaderMap;
    use leptos_axum::extract;
    let headers = extract::<HeaderMap>().await.unwrap_or_default();
    let token = session_token(&headers)?;

    let patch = PatchTemplateBody {
        description: Some(description),
        display_config: Some(display_config),
    };
    let url = format!("/api/scorecard-templates/{template_id}");
    crate::atlas_client::authenticated_patch::<PatchTemplateBody, TemplateDetailDto>(
        &url, &token, patch,
    )
    .await
    .map_err(|e| server_fn::error::ServerFnError::new(e.to_string()))?;

    let rules: Vec<DisplayRuleForm> = serde_json::from_str(&rules_json)
        .map_err(|e| server_fn::error::ServerFnError::new(e.to_string()))?;

    for rule in rules {
        if let Some(id) = rule.id {
            let body = UpdateRuleBody {
                dimension_id: rule.dimension_id,
                category_target: if rule.category_target.is_empty() {
                    None
                } else {
                    Some(rule.category_target.clone())
                },
                trigger_category: Some(rule.trigger_category),
                field_reference: if rule.field_reference.is_empty() {
                    None
                } else {
                    Some(rule.field_reference.clone())
                },
                operator: Some(rule.operator),
                value: if rule.value.is_empty() {
                    None
                } else {
                    Some(rule.value.clone())
                },
                value_list: None,
                action: Some(rule.action),
                alert_message: if rule.alert_message.is_empty() {
                    None
                } else {
                    Some(rule.alert_message.clone())
                },
                mode_scope: Some(rule.mode_scope),
                priority: Some(rule.priority),
                description: if rule.description.is_empty() {
                    None
                } else {
                    Some(rule.description.clone())
                },
                is_active: Some(rule.is_active),
            };
            let rule_url = format!("/api/scorecard-display-rules/{id}");
            let _ = crate::atlas_client::authenticated_patch::<UpdateRuleBody, DisplayRuleDto>(
                &rule_url, &token, body,
            )
            .await;
        } else {
            let body = CreateRuleBody {
                dimension_id: rule.dimension_id,
                category_target: if rule.category_target.is_empty() {
                    None
                } else {
                    Some(rule.category_target)
                },
                trigger_category: rule.trigger_category,
                field_reference: if rule.field_reference.is_empty() {
                    None
                } else {
                    Some(rule.field_reference)
                },
                operator: rule.operator,
                value: if rule.value.is_empty() {
                    None
                } else {
                    Some(rule.value)
                },
                value_list: None,
                action: rule.action,
                alert_message: if rule.alert_message.is_empty() {
                    None
                } else {
                    Some(rule.alert_message)
                },
                mode_scope: Some(rule.mode_scope),
                priority: Some(rule.priority),
                description: if rule.description.is_empty() {
                    None
                } else {
                    Some(rule.description)
                },
            };
            let create_url = format!("/api/scorecard-templates/{template_id}/display-rules");
            let _ = crate::atlas_client::authenticated_post::<CreateRuleBody, DisplayRuleDto>(
                &create_url,
                &token,
                None,
                &body,
            )
            .await;
        }
    }

    Ok(())
}

// ── Helpers ───────────────────────────────────────────────────────────────────

fn parse_f64(s: &str) -> f64 {
    s.parse().unwrap_or(0.0)
}

fn template_to_form(t: &TemplateDetailDto) -> TemplateForm {
    let display_config = t
        .display_config
        .as_ref()
        .and_then(|v| serde_json::from_value::<DisplayConfigForm>(v.clone()).ok())
        .unwrap_or_default();
    TemplateForm {
        id: Some(t.id),
        name: t.name.clone(),
        entity_type: t.entity_type.clone(),
        description: t.description.clone().unwrap_or_default(),
        scoring_method: t.scoring_method,
        default_scale_min: parse_f64(&t.default_scale_min),
        default_scale_max: parse_f64(&t.default_scale_max),
        min_entries_to_publish: t.min_entries_to_publish,
        is_published: t.is_published,
        template_scope: t.template_scope,
        cold_start_strategy: t.cold_start_strategy,
        cold_start_saturation_threshold: t.cold_start_saturation_threshold,
        calibration_minimum_entries: t.calibration_minimum_entries,
        default_bayesian_prior_weight: t
            .default_bayesian_prior_weight
            .as_ref()
            .and_then(|s| s.parse().ok()),
        display_config,
    }
}

fn dimension_to_form(d: &DimensionDto, local_id: usize) -> DimensionForm {
    DimensionForm {
        local_id,
        id: Some(d.id),
        name: d.name.clone(),
        slug: d.slug.clone(),
        description: d.description.clone().unwrap_or_default(),
        category: d.category.clone().unwrap_or_default(),
        weight: parse_f64(&d.weight),
        scale_type: d.scale_type,
        scale_min: parse_f64(&d.scale_min),
        scale_max: parse_f64(&d.scale_max),
        unit_label: String::new(),
        is_inverted: false,
        is_community_ratable: true,
        is_active: d.is_active,
        sort_order: d.sort_order,
        is_tenant_extension: false,
        min_entries_to_show: 1,
        bayesian_prior_weight: None,
        global_reference_value: None,
        global_reference_label: String::new(),
        options: Vec::new(),
        ideal_score: None,
        range_min: None,
        range_max: None,
        search_weight: None,
    }
}

fn value_list_to_raw(v: &Option<serde_json::Value>) -> String {
    match v {
        Some(serde_json::Value::Array(arr)) => arr
            .iter()
            .filter_map(|x| x.as_str().map(|s| s.to_string()))
            .collect::<Vec<_>>()
            .join(", "),
        Some(other) => other.to_string(),
        None => String::new(),
    }
}

fn rule_to_form(r: &DisplayRuleDto, local_id: usize) -> DisplayRuleForm {
    DisplayRuleForm {
        local_id,
        id: Some(r.id),
        dimension_id: r.dimension_id,
        dimension_name: String::new(),
        category_target: r.category_target.clone().unwrap_or_default(),
        trigger_category: r.trigger_category,
        field_reference: r.field_reference.clone().unwrap_or_default(),
        operator: r.operator,
        value: r.value.clone().unwrap_or_default(),
        value_list_raw: value_list_to_raw(&r.value_list),
        action: r.action,
        alert_message: r.alert_message.clone().unwrap_or_default(),
        mode_scope: r.mode_scope,
        priority: r.priority,
        is_active: r.is_active,
        description: r.description.clone().unwrap_or_default(),
    }
}

fn trend_icon(trend: Option<&str>) -> &'static str {
    match trend {
        Some("up") => "↑",
        Some("down") => "↓",
        _ => "→",
    }
}

fn trend_color(trend: Option<&str>) -> &'static str {
    match trend {
        Some("up") => "#4ade80",
        Some("down") => "#f87171",
        _ => "#94a3b8",
    }
}

fn score_bar(score: f64) -> impl IntoView {
    let pct = (score * 100.0).clamp(0.0, 100.0);
    view! {
        <div class="g27-score-bar-wrap">
            <div class="g27-score-bar" style=format!("width:{pct}%")></div>
            <span class="g27-score-label">{format!("{:.0}%", pct)}</span>
        </div>
    }
}

// ── Dashboard ─────────────────────────────────────────────────────────────────

#[component]
fn G27DashboardTab(template_id: String) -> impl IntoView {
    let tid = template_id.clone();
    let tid2 = template_id.clone();
    let tid3 = template_id.clone();

    let stats_res = Resource::new(move || tid.clone(), |t| fetch_g27_analytics(t));
    let lead_res = Resource::new(move || tid2.clone(), |t| fetch_g27_leaderboard(t));
    let anom_res = Resource::new(move || tid3.clone(), |t| fetch_g27_anomalies(t));

    view! {
        <div class="g27-dash">
            <Suspense fallback=|| view! { <div class="doc-empty">"Loading analytics…"</div> }>
                {move || stats_res.get().map(|res| match res {
                    Ok(stats) => {
                        let mean = stats.mean_score.map(|s| format!("{:.0}%", s * 100.0)).unwrap_or_else(|| "—".into());
                        let median = stats.median_score.map(|s| format!("{:.0}%", s * 100.0)).unwrap_or_else(|| "—".into());
                        let anoms = stats.anomaly_count_30d.map(|n| n.to_string()).unwrap_or_else(|| "—".into());
                        view! {
                            <div>
                                <div class="kpi-row" style="margin-bottom:1.25rem;">
                                    <div class="kpi-card">
                                        <span class="kpi-label">"Cohort Size"</span>
                                        <span class="kpi-value" style="color:var(--cobalt)">{stats.cohort_size.to_string()}</span>
                                    </div>
                                    <div class="kpi-card">
                                        <span class="kpi-label">"Mean Score"</span>
                                        <span class="kpi-value" style="color:#fbbf24">{mean}</span>
                                    </div>
                                    <div class="kpi-card">
                                        <span class="kpi-label">"Median Score"</span>
                                        <span class="kpi-value" style="color:#4ade80">{median}</span>
                                    </div>
                                    <div class="kpi-card">
                                        <span class="kpi-label">"Anomalies (30d)"</span>
                                        <span class="kpi-value" style="color:#f87171">{anoms}</span>
                                    </div>
                                </div>
                                <div class="owner-section">
                                    <div class="owner-section-title">"Dimension Breakdown"</div>
                                    <div class="g27-dim-table">
                                        {stats.dimensions.iter().map(|d| {
                                            let score = d.mean_score.unwrap_or(0.0);
                                            let t_icon = trend_icon(d.trend.as_deref());
                                            let t_color = trend_color(d.trend.as_deref());
                                            let name = d.dimension_name.clone();
                                            view! {
                                                <div class="g27-dim-row">
                                                    <div class="g27-dim-name">{name}</div>
                                                    {score_bar(score)}
                                                    <span style=format!("font-size:.85rem;color:{t_color};font-weight:700;")>{t_icon}</span>
                                                </div>
                                            }
                                        }).collect::<Vec<_>>()}
                                    </div>
                                </div>
                            </div>
                        }.into_any()
                    }
                    Err(e) => view! { <div class="doc-empty">{e.to_string()}</div> }.into_any(),
                })}
            </Suspense>

            <div class="owner-section" style="margin-top:1.5rem;">
                <div class="owner-section-title">"Leaderboard"</div>
                <Suspense fallback=|| view! { <div class="doc-empty">"Loading…"</div> }>
                    {move || lead_res.get().map(|res| match res {
                        Ok(rows) if rows.is_empty() => view! { <div class="doc-empty">"No leaderboard data yet."</div> }.into_any(),
                        Ok(rows) => view! {
                            <ul class="space-y-2">
                                {rows.into_iter().map(|r| {
                                    let label = r.entity_label.unwrap_or_else(|| r.entity_id.to_string());
                                    view! {
                                        <li class="flex justify-between gap-4 py-2 border-b border-[var(--folio-border)]">
                                            <span>{format!("#{} {}", r.rank, label)}</span>
                                            <span>{format!("{:.1}", r.composite_score)}</span>
                                        </li>
                                    }
                                }).collect_view()}
                            </ul>
                        }.into_any(),
                        Err(e) => view! { <div class="doc-empty">{e.to_string()}</div> }.into_any(),
                    })}
                </Suspense>
            </div>

            <div class="owner-section" style="margin-top:1.5rem;">
                <div class="owner-section-title">"Anomalies"</div>
                <Suspense fallback=|| view! { <div class="doc-empty">"Loading…"</div> }>
                    {move || anom_res.get().map(|res| match res {
                        Ok(rows) if rows.is_empty() => view! { <div class="doc-empty">"No anomalies."</div> }.into_any(),
                        Ok(rows) => view! {
                            <ul class="space-y-2">
                                {rows.into_iter().map(|a| {
                                    let label = a.entity_label.unwrap_or_else(|| a.entity_id.to_string());
                                    let dim = a.dimension_name.unwrap_or_default();
                                    view! {
                                        <li class="py-2 border-b border-[var(--folio-border)]">
                                            <div class="font-medium">{label}</div>
                                            <div class="text-sm text-[var(--folio-muted)]">
                                                {format!("{} · {:.2} · {}", dim, a.score, a.detected_at)}
                                            </div>
                                        </li>
                                    }
                                }).collect_view()}
                            </ul>
                        }.into_any(),
                        Err(e) => view! { <div class="doc-empty">{e.to_string()}</div> }.into_any(),
                    })}
                </Suspense>
            </div>
        </div>
    }
}

// ── Configure tab ─────────────────────────────────────────────────────────────

#[component]
fn MeridianConfigureTab(template_id: Uuid) -> impl IntoView {
    let tid = template_id.to_string();
    let tid2 = tid.clone();
    let tid3 = tid.clone();
    let save_msg: RwSignal<Option<String>> = RwSignal::new(None);
    let save_err: RwSignal<Option<String>> = RwSignal::new(None);

    let detail_res = Resource::new(move || tid.clone(), |t| fetch_meridian_template_detail(t));
    let dims_res = Resource::new(move || tid2.clone(), |t| fetch_meridian_dimensions(t));
    let rules_res = Resource::new(move || tid3.clone(), |t| fetch_meridian_display_rules(t));

    view! {
        <div>
            <Show when=move || save_msg.get().is_some()>
                <p class="text-sm mb-3" style="color:#4ade80">{save_msg.get().unwrap_or_default()}</p>
            </Show>
            <Show when=move || save_err.get().is_some()>
                <p class="text-sm mb-3 text-red-400">{save_err.get().unwrap_or_default()}</p>
            </Show>
            <Suspense fallback=|| view! { <div class="doc-empty">"Loading configurator…"</div> }>
                {move || {
                    let detail = detail_res.get();
                    let dims = dims_res.get();
                    let rules = rules_res.get();
                    match (detail, dims, rules) {
                        (Some(Ok(t)), Some(Ok(d)), Some(Ok(r))) => {
                            let form = template_to_form(&t);
                            let dim_forms: Vec<_> = d.iter().enumerate().map(|(i, x)| dimension_to_form(x, i + 1)).collect();
                            let rule_forms: Vec<_> = r.iter().enumerate().map(|(i, x)| rule_to_form(x, 1000 + i)).collect();
                            let template_id = t.id;
                            view! {
                                <Configurator
                                    initial_template=form
                                    initial_dimensions=dim_forms
                                    initial_display_rules=rule_forms
                                    mode=ConfiguratorMode::TenantAdmin
                                    on_save=Callback::new(move |payload: TemplateSavePayload| {
                                        save_msg.set(None);
                                        save_err.set(None);
                                        spawn_local(async move {
                                            let cfg = serde_json::to_value(&payload.template.display_config)
                                                .unwrap_or(serde_json::json!({}));
                                            let rules_json = serde_json::to_string(&payload.display_rules)
                                                .unwrap_or_else(|_| "[]".into());
                                            match save_meridian_configurator(
                                                template_id.to_string(),
                                                payload.template.description,
                                                cfg,
                                                rules_json,
                                            ).await {
                                                Ok(()) => save_msg.set(Some("Saved display config and rules.".into())),
                                                Err(e) => save_err.set(Some(e.to_string())),
                                            }
                                        });
                                    })
                                />
                            }.into_any()
                        }
                        (Some(Err(e)), _, _) | (_, Some(Err(e)), _) | (_, _, Some(Err(e))) => {
                            view! { <div class="doc-empty">{e.to_string()}</div> }.into_any()
                        }
                        _ => view! { <div class="doc-empty">"Loading configurator…"</div> }.into_any(),
                    }
                }}
            </Suspense>
        </div>
    }
}

// ── Root ──────────────────────────────────────────────────────────────────────

#[component]
pub fn MeridianConfigurator() -> impl IntoView {
    let tab = RwSignal::new(0u8);
    let active_template = RwSignal::new(None::<ScorecardTemplate>);
    let templates_res = Resource::new(|| (), |_| fetch_g27_templates());

    view! {
        <div class="main-area">
            <PageHeader
                title=Signal::derive(|| "Meridian".to_string())
                subtitle=Signal::derive(|| "Scorecard analytics and tenant configuration".to_string())
            />

            <div class="g27-template-bar">
                <div class="g27-template-label">"Template:"</div>
                <Suspense fallback=|| view! { <select class="folio-select g27-template-select"><option>"Loading…"</option></select> }>
                    {move || templates_res.get().map(|res| match res {
                        Ok(tmpls) if !tmpls.is_empty() => {
                            if active_template.get().is_none() {
                                active_template.set(Some(tmpls[0].clone()));
                            }
                            view! {
                                <select class="folio-select g27-template-select"
                                    on:change=move |ev| {
                                        let sel_id = event_target_value(&ev);
                                        if let Some(t) = tmpls.iter().find(|t| t.id.to_string() == sel_id) {
                                            active_template.set(Some(t.clone()));
                                        }
                                    }
                                >
                                    {tmpls.iter().map(|t| {
                                        let tid = t.id.to_string();
                                        let tname = format!("{} ({})", t.name, t.entity_type);
                                        view! { <option value={tid.clone()}>{tname}</option> }
                                    }).collect::<Vec<_>>()}
                                </select>
                            }.into_any()
                        }
                        Ok(_) => view! {
                            <div class="doc-empty">"No deployed scorecard templates for this instance."</div>
                        }.into_any(),
                        Err(e) => view! { <div class="doc-empty">{e.to_string()}</div> }.into_any(),
                    })}
                </Suspense>
            </div>

            <div class="g27-tabs" style="display:flex;gap:.75rem;margin:1rem 0;">
                <button
                    class=move || if tab.get() == 0 { "cfg-btn cfg-btn--primary" } else { "cfg-btn cfg-btn--ghost" }
                    on:click=move |_| tab.set(0)
                >"Dashboard"</button>
                <button
                    class=move || if tab.get() == 1 { "cfg-btn cfg-btn--primary" } else { "cfg-btn cfg-btn--ghost" }
                    on:click=move |_| tab.set(1)
                >"Configure"</button>
            </div>

            {move || match active_template.get() {
                Some(t) if tab.get() == 0 => view! { <G27DashboardTab template_id=t.id.to_string() /> }.into_any(),
                Some(t) if tab.get() == 1 => view! { <MeridianConfigureTab template_id=t.id /> }.into_any(),
                _ => view! { <div class="doc-empty">"Select a deployed template."</div> }.into_any(),
            }}
        </div>
    }
}
