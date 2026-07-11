//! G-27 scorecard mount for network-instance — `/dashboard/scorecards`
//!
//! TenantAdmin Configurator + ScorecardWidget against the same tenant APIs as Folio.
//! See: `docs/architecture/g27/g27_app_instance_runtime.md`

use leptos::prelude::*;
use leptos::task::spawn_local;
use serde::{Deserialize, Serialize};
use shared_ui::components::configurator::Configurator;
use shared_ui::components::scorecard::models::{
    ColdStartStrategy, ConfiguratorMode, DisplayConfigForm, DisplayRuleForm, DimensionForm,
    ModeScope, RenderMode, RuleAction, RuleOperator, ScaleType, ScoringMethod, SessionDimension,
    SessionType, SourceType, TemplateForm, TemplateSavePayload, TemplateScope, TriggerCategory,
};
use shared_ui::components::scorecard::{ScoreSubmission, ScorecardWidget};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateListItem {
    pub id: Uuid,
    pub name: String,
    pub entity_type: String,
    pub is_active: bool,
    pub is_published: bool,
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
struct IdResponse {
    pub id: Uuid,
}

#[derive(Debug, Serialize)]
struct PatchTemplateBody {
    description: Option<String>,
    display_config: Option<serde_json::Value>,
}

#[derive(Debug, Serialize)]
struct GetOrCreateBody {
    template_id: Uuid,
    subject_entity_type: String,
    subject_entity_id: Uuid,
}

#[derive(Debug, Serialize)]
struct OpenSessionBody {
    session_type: SessionType,
    session_label: Option<String>,
}

#[derive(Debug, Serialize)]
struct SubmitEntryBody {
    scorecard_id: Uuid,
    dimension_id: Uuid,
    score: Option<f64>,
    option_id: Option<Uuid>,
    source_type: Option<SourceType>,
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

fn session_cookie() -> String {
    use axum::http::request::Parts;
    if let Some(req_parts) = leptos::prelude::use_context::<Parts>() {
        req_parts
            .headers
            .get("cookie")
            .and_then(|v| v.to_str().ok())
            .and_then(|cookies| {
                cookies.split(';').find_map(|part| {
                    let part = part.trim();
                    part.strip_prefix("session=").map(|t| t.to_string())
                })
            })
            .unwrap_or_default()
    } else {
        String::new()
    }
}

#[server]
pub async fn ni_list_scorecard_templates() -> Result<Vec<TemplateListItem>, ServerFnError> {
    let token = session_cookie();
    let url = format!("{}/api/scorecard-templates", crate::get_api_base_url());
    let res = reqwest::Client::new()
        .get(&url)
        .header("Cookie", format!("session={token}"))
        .send()
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;
    if !res.status().is_success() {
        return Err(ServerFnError::new(format!("API {}", res.status())));
    }
    res.json().await.map_err(|e| ServerFnError::new(e.to_string()))
}

#[server]
pub async fn ni_fetch_template_detail(
    template_id: String,
) -> Result<TemplateDetailDto, ServerFnError> {
    let token = session_cookie();
    let url = format!(
        "{}/api/scorecard-templates/{template_id}",
        crate::get_api_base_url()
    );
    let res = reqwest::Client::new()
        .get(&url)
        .header("Cookie", format!("session={token}"))
        .send()
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;
    if !res.status().is_success() {
        return Err(ServerFnError::new(format!("API {}", res.status())));
    }
    res.json().await.map_err(|e| ServerFnError::new(e.to_string()))
}

#[server]
pub async fn ni_fetch_dimensions(template_id: String) -> Result<Vec<DimensionDto>, ServerFnError> {
    let token = session_cookie();
    let url = format!(
        "{}/api/scorecard-templates/{template_id}/dimensions",
        crate::get_api_base_url()
    );
    let res = reqwest::Client::new()
        .get(&url)
        .header("Cookie", format!("session={token}"))
        .send()
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;
    if !res.status().is_success() {
        return Err(ServerFnError::new(format!("API {}", res.status())));
    }
    res.json().await.map_err(|e| ServerFnError::new(e.to_string()))
}

#[server]
pub async fn ni_fetch_display_rules(
    template_id: String,
) -> Result<Vec<DisplayRuleDto>, ServerFnError> {
    let token = session_cookie();
    let url = format!(
        "{}/api/scorecard-templates/{template_id}/display-rules",
        crate::get_api_base_url()
    );
    let res = reqwest::Client::new()
        .get(&url)
        .header("Cookie", format!("session={token}"))
        .send()
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;
    if !res.status().is_success() {
        return Err(ServerFnError::new(format!("API {}", res.status())));
    }
    res.json().await.map_err(|e| ServerFnError::new(e.to_string()))
}

#[server]
pub async fn ni_save_configurator(
    template_id: String,
    description: String,
    display_config: serde_json::Value,
) -> Result<(), ServerFnError> {
    let token = session_cookie();
    let url = format!(
        "{}/api/scorecard-templates/{template_id}",
        crate::get_api_base_url()
    );
    let body = PatchTemplateBody {
        description: Some(description),
        display_config: Some(display_config),
    };
    let res = reqwest::Client::new()
        .patch(&url)
        .header("Cookie", format!("session={token}"))
        .json(&body)
        .send()
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;
    if !res.status().is_success() {
        return Err(ServerFnError::new(format!("API {}", res.status())));
    }
    Ok(())
}

#[server]
pub async fn ni_open_rating_session(
    template_id: String,
    subject_entity_type: String,
    subject_entity_id: String,
) -> Result<(Uuid, Uuid, Vec<DimensionDto>), ServerFnError> {
    let token = session_cookie();
    let base = crate::get_api_base_url();
    let client = reqwest::Client::new();
    let template_id = Uuid::parse_str(&template_id)
        .map_err(|e| ServerFnError::new(e.to_string()))?;
    let subject_entity_id = Uuid::parse_str(&subject_entity_id)
        .map_err(|e| ServerFnError::new(e.to_string()))?;

    let goc = GetOrCreateBody {
        template_id,
        subject_entity_type,
        subject_entity_id,
    };
    let sc: IdResponse = client
        .post(format!("{base}/api/scorecards/get-or-create"))
        .header("Cookie", format!("session={token}"))
        .json(&goc)
        .send()
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?
        .json()
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;

    let open = OpenSessionBody {
        session_type: SessionType::Visit,
        session_label: Some("Network listing rating".into()),
    };
    let sess: IdResponse = client
        .post(format!("{base}/api/scorecards/{}/sessions", sc.id))
        .header("Cookie", format!("session={token}"))
        .json(&open)
        .send()
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?
        .json()
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;

    let dims: Vec<DimensionDto> = client
        .get(format!(
            "{base}/api/scorecard-templates/{template_id}/dimensions"
        ))
        .header("Cookie", format!("session={token}"))
        .send()
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?
        .json()
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;

    Ok((sc.id, sess.id, dims))
}

#[server]
pub async fn ni_submit_rating_entry(
    session_id: String,
    scorecard_id: String,
    dimension_id: String,
    score: Option<f64>,
) -> Result<(), ServerFnError> {
    let token = session_cookie();
    let body = SubmitEntryBody {
        scorecard_id: Uuid::parse_str(&scorecard_id)
            .map_err(|e| ServerFnError::new(e.to_string()))?,
        dimension_id: Uuid::parse_str(&dimension_id)
            .map_err(|e| ServerFnError::new(e.to_string()))?,
        score,
        option_id: None,
        source_type: Some(SourceType::Manual),
    };
    let url = format!(
        "{}/api/scorecard-sessions/{session_id}/entries",
        crate::get_api_base_url()
    );
    let res = reqwest::Client::new()
        .post(&url)
        .header("Cookie", format!("session={token}"))
        .json(&body)
        .send()
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;
    if !res.status().is_success() {
        return Err(ServerFnError::new(format!("API {}", res.status())));
    }
    Ok(())
}

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
        value_list_raw: String::new(),
        action: r.action,
        alert_message: r.alert_message.clone().unwrap_or_default(),
        mode_scope: r.mode_scope,
        priority: r.priority,
        is_active: r.is_active,
        description: r.description.clone().unwrap_or_default(),
    }
}

fn dims_to_session(dims: Vec<DimensionDto>) -> Vec<SessionDimension> {
    dims.into_iter()
        .map(|d| SessionDimension {
            dimension_id: d.id,
            slug: d.slug,
            name: d.name,
            description: d.description.unwrap_or_default(),
            scale_type: d.scale_type,
            scale_min: parse_f64(&d.scale_min),
            scale_max: parse_f64(&d.scale_max),
            unit_label: None,
            is_inverted: false,
            is_required: false,
            render_mode: RenderMode::Normal,
            draft_score: None,
            inferred_score: None,
            inferred_confidence: None,
            draft_option_id: None,
        })
        .collect()
}

#[component]
fn NiConfigurePanel(template_id: Uuid) -> impl IntoView {
    let tid = template_id.to_string();
    let tid2 = tid.clone();
    let tid3 = tid.clone();
    let msg: RwSignal<Option<String>> = RwSignal::new(None);

    let detail = Resource::new(move || tid.clone(), |t| ni_fetch_template_detail(t));
    let dims = Resource::new(move || tid2.clone(), |t| ni_fetch_dimensions(t));
    let rules = Resource::new(move || tid3.clone(), |t| ni_fetch_display_rules(t));

    view! {
        <Suspense fallback=|| view! { <p>"Loading configurator…"</p> }>
            {move || {
                match (detail.get(), dims.get(), rules.get()) {
                    (Some(Ok(t)), Some(Ok(d)), Some(Ok(r))) => {
                        let form = template_to_form(&t);
                        let dim_forms: Vec<_> = d.iter().enumerate().map(|(i, x)| dimension_to_form(x, i + 1)).collect();
                        let rule_forms: Vec<_> = r.iter().enumerate().map(|(i, x)| rule_to_form(x, 1000 + i)).collect();
                        let template_id = t.id;
                        view! {
                            <Show when=move || msg.get().is_some()>
                                <p class="text-sm mb-2">{msg.get().unwrap_or_default()}</p>
                            </Show>
                            <Configurator
                                initial_template=form
                                initial_dimensions=dim_forms
                                initial_display_rules=rule_forms
                                mode=ConfiguratorMode::TenantAdmin
                                on_save=Callback::new(move |payload: TemplateSavePayload| {
                                    spawn_local(async move {
                                        let cfg = serde_json::to_value(&payload.template.display_config)
                                            .unwrap_or(serde_json::json!({}));
                                        match ni_save_configurator(
                                            template_id.to_string(),
                                            payload.template.description,
                                            cfg,
                                        ).await {
                                            Ok(()) => msg.set(Some("Saved.".into())),
                                            Err(e) => msg.set(Some(e.to_string())),
                                        }
                                    });
                                })
                            />
                        }.into_any()
                    }
                    (Some(Err(e)), _, _) | (_, Some(Err(e)), _) | (_, _, Some(Err(e))) => {
                        view! { <p>{e.to_string()}</p> }.into_any()
                    }
                    _ => view! { <p>"Loading…"</p> }.into_any(),
                }
            }}
        </Suspense>
    }
}

#[component]
fn NiRateListingPanel(template_id: Uuid, entity_type: String) -> impl IntoView {
    let listing_id = RwSignal::new(String::new());
    let active: RwSignal<Option<(Uuid, Uuid, Vec<SessionDimension>)>> = RwSignal::new(None);
    let err: RwSignal<Option<String>> = RwSignal::new(None);

    view! {
        <div class="mt-8">
            <h2 class="text-lg font-semibold mb-2">"Rate a listing"</h2>
            <p class="text-sm text-on-surface-variant mb-3">
                "Opens a session for subject type " {entity_type.clone()} " using ScorecardWidget."
            </p>
            <div class="flex gap-2 mb-4">
                <input
                    class="cfg-input"
                    type="text"
                    placeholder="Listing UUID"
                    prop:value=move || listing_id.get()
                    on:input=move |ev| listing_id.set(event_target_value(&ev))
                />
                <button
                    class="cfg-btn cfg-btn--primary"
                    type="button"
                    on:click=move |_| {
                        let tid = template_id.to_string();
                        let et = entity_type.clone();
                        let sid = listing_id.get();
                        err.set(None);
                        spawn_local(async move {
                            match ni_open_rating_session(tid, et, sid).await {
                                Ok((sc, sess, dims)) => {
                                    active.set(Some((sc, sess, dims_to_session(dims))));
                                }
                                Err(e) => err.set(Some(e.to_string())),
                            }
                        });
                    }
                >"Start rating"</button>
            </div>
            <Show when=move || err.get().is_some()>
                <p class="text-red-400 text-sm">{err.get().unwrap_or_default()}</p>
            </Show>
            {move || active.get().map(|(scorecard_id, session_id, dims)| {
                view! {
                    <ScorecardWidget
                        scorecard_id=scorecard_id
                        session_id=session_id
                        subject_label="Listing rating".into()
                        dimensions=dims
                        on_submit=Callback::new(move |subs: Vec<ScoreSubmission>| {
                            spawn_local(async move {
                                for sub in subs {
                                    let _ = ni_submit_rating_entry(
                                        session_id.to_string(),
                                        scorecard_id.to_string(),
                                        sub.dimension_id.to_string(),
                                        sub.score,
                                    ).await;
                                }
                                active.set(None);
                            });
                        })
                        on_cancel=Callback::new(move |_| active.set(None))
                    />
                }
            })}
        </div>
    }
}

/// Full TenantAdmin Configurator + listing ScorecardWidget mount.
#[component]
pub fn ScorecardMountStub() -> impl IntoView {
    let templates = Resource::new(|| (), |_| ni_list_scorecard_templates());
    let selected = RwSignal::new(None::<TemplateListItem>);

    view! {
        <div class="w-full">
            <h1 class="text-2xl font-semibold mb-2">"Scorecards"</h1>
            <p class="text-sm text-on-surface-variant mb-4">
                "Deployed templates for this app instance — TenantAdmin Configurator."
            </p>

            <Suspense fallback=|| view! { <p>"Loading templates…"</p> }>
                {move || templates.get().map(|res| match res {
                    Ok(list) if list.is_empty() => view! {
                        <p>"No deployed scorecard templates for this instance."</p>
                    }.into_any(),
                    Ok(list) => {
                        if selected.get().is_none() {
                            selected.set(Some(list[0].clone()));
                        }
                        view! {
                            <select
                                class="cfg-select mb-4"
                                on:change=move |ev| {
                                    let id = event_target_value(&ev);
                                    if let Some(t) = list.iter().find(|t| t.id.to_string() == id) {
                                        selected.set(Some(t.clone()));
                                    }
                                }
                            >
                                {list.iter().map(|t| {
                                    let id = t.id.to_string();
                                    let label = format!("{} ({})", t.name, t.entity_type);
                                    view! { <option value=id>{label}</option> }
                                }).collect::<Vec<_>>()}
                            </select>
                        }.into_any()
                    }
                    Err(e) => view! { <p>{e.to_string()}</p> }.into_any(),
                })}
            </Suspense>

            {move || selected.get().map(|t| {
                view! {
                    <NiConfigurePanel template_id=t.id />
                    <NiRateListingPanel template_id=t.id entity_type=t.entity_type.clone() />
                }.into_any()
            })}
        </div>
    }
}
