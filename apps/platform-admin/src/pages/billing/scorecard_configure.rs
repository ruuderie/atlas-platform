//! G-27 template configurator mount (platform-admin).
//!
//! Contract: `docs/contracts/g27_scorecard_platform.md` §8–§9.

use crate::api::admin::get_tenant_stats;
use crate::api::scorecards::{
    create_dimension, create_display_rule, create_template, get_template, list_dimensions,
    list_display_rules, update_dimension, update_display_rule, update_template,
    CreateDimensionInput, CreateDisplayRuleInput, CreateTemplateInput, DisplayRuleAdminView,
    ScorecardDimension, ScorecardTemplate, UpdateDimensionInput, UpdateDisplayRuleInput,
    UpdateTemplateInput,
};
use crate::app::GlobalToast;
use leptos::prelude::*;
use leptos_router::hooks::{use_navigate, use_params_map, use_query_map};
use shared_ui::components::configurator::Configurator;
use shared_ui::components::scorecard::models::{
    ConfiguratorMode, DisplayConfigForm, DisplayRuleForm, DimensionForm, ModeScope, RuleAction,
    RuleOperator, ScaleType, TemplateForm, TemplateSavePayload, TriggerCategory,
};
use std::str::FromStr;
use uuid::Uuid;

fn parse_f64(s: &str) -> f64 {
    s.parse().unwrap_or(0.0)
}

fn template_to_form(t: &ScorecardTemplate) -> TemplateForm {
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
        scoring_method: t.scoring_method.clone(),
        default_scale_min: parse_f64(&t.default_scale_min),
        default_scale_max: parse_f64(&t.default_scale_max),
        min_entries_to_publish: t.min_entries_to_publish,
        is_published: t.is_published,
        template_scope: t.template_scope.clone(),
        cold_start_strategy: t.cold_start_strategy.clone(),
        cold_start_saturation_threshold: t.cold_start_saturation_threshold,
        calibration_minimum_entries: t.calibration_minimum_entries,
        default_bayesian_prior_weight: t
            .default_bayesian_prior_weight
            .as_ref()
            .and_then(|s| s.parse().ok()),
        display_config,
    }
}

fn dimension_to_form(d: &ScorecardDimension, local_id: usize) -> DimensionForm {
    let scale_type = ScaleType::from_str(&d.scale_type).unwrap_or(ScaleType::Rating);
    DimensionForm {
        local_id,
        id: Some(d.id),
        name: d.name.clone(),
        slug: d.slug.clone(),
        description: d.description.clone().unwrap_or_default(),
        category: d.category.clone().unwrap_or_default(),
        weight: parse_f64(&d.weight),
        scale_type,
        scale_min: parse_f64(&d.scale_min),
        scale_max: parse_f64(&d.scale_max),
        unit_label: d.unit_label.clone().unwrap_or_default(),
        is_inverted: d.is_inverted,
        is_community_ratable: d.is_community_ratable,
        is_active: d.is_active,
        sort_order: d.sort_order,
        is_tenant_extension: d.is_tenant_extension,
        min_entries_to_show: d.min_entries_to_show,
        bayesian_prior_weight: d.bayesian_prior_weight.as_ref().and_then(|s| s.parse().ok()),
        global_reference_value: d
            .global_reference_value
            .as_ref()
            .and_then(|s| s.parse().ok()),
        global_reference_label: d.global_reference_label.clone().unwrap_or_default(),
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

fn rule_to_form(r: &DisplayRuleAdminView, local_id: usize) -> DisplayRuleForm {
    DisplayRuleForm {
        local_id,
        id: Some(r.id),
        dimension_id: r.dimension_id,
        dimension_name: String::new(),
        category_target: r.category_target.clone().unwrap_or_default(),
        trigger_category: TriggerCategory::from_str(&r.trigger_category)
            .unwrap_or(TriggerCategory::RecordState),
        field_reference: r.field_reference.clone().unwrap_or_default(),
        operator: RuleOperator::from_str(&r.operator).unwrap_or(RuleOperator::Equals),
        value: r.value.clone().unwrap_or_default(),
        value_list_raw: value_list_to_raw(&r.value_list),
        action: RuleAction::from_str(&r.action).unwrap_or(RuleAction::Show),
        alert_message: r.alert_message.clone().unwrap_or_default(),
        mode_scope: ModeScope::from_str(&r.mode_scope).unwrap_or(ModeScope::Always),
        priority: r.priority,
        is_active: r.is_active,
        description: r.description.clone().unwrap_or_default(),
    }
}

fn display_config_json(cfg: &DisplayConfigForm) -> Option<serde_json::Value> {
    serde_json::to_value(cfg).ok()
}

#[component]
pub fn ScorecardConfigure() -> impl IntoView {
    let params = use_params_map();
    let query = use_query_map();
    let navigate = use_navigate();
    let toast = use_context::<GlobalToast>().expect("toast context");

    let template_id_param = move || {
        params.with(|p| p.get("template_id").unwrap_or_else(|| "new".to_string()))
    };
    let is_create = move || template_id_param() == "new";

    let selected_tenant_id = RwSignal::new(
        query
            .with_untracked(|q| q.get("tenant_id").unwrap_or_default()),
    );

    let tenants_res =
        LocalResource::new(|| async move { get_tenant_stats().await.unwrap_or_default() });

    Effect::new(move |_| {
        if selected_tenant_id.get().is_empty() {
            if let Some(tenants) = tenants_res.get() {
                if let Some(first) = tenants.first() {
                    selected_tenant_id.set(first.tenant_id.clone());
                }
            }
        }
    });

    let load_res = LocalResource::new(move || {
        let tid = selected_tenant_id.get();
        let tmpl_id = template_id_param();
        async move {
            if tid.is_empty() {
                return Ok::<Option<(TemplateForm, Vec<DimensionForm>, Vec<DisplayRuleForm>)>, String>(
                    None,
                );
            }
            if tmpl_id == "new" {
                return Ok(Some((
                    TemplateForm::default(),
                    Vec::<DimensionForm>::new(),
                    Vec::<DisplayRuleForm>::new(),
                )));
            }
            let template = get_template(&tid, &tmpl_id).await?;
            let dims = list_dimensions(&tid, &tmpl_id).await.unwrap_or_default();
            let rules = list_display_rules(&tmpl_id).await.unwrap_or_default();
            let form = template_to_form(&template);
            let dim_forms: Vec<DimensionForm> = dims
                .iter()
                .enumerate()
                .map(|(i, d)| dimension_to_form(d, i + 1))
                .collect();
            let rule_forms: Vec<DisplayRuleForm> = rules
                .iter()
                .enumerate()
                .map(|(i, r)| rule_to_form(r, i + 1000))
                .collect();
            Ok(Some((form, dim_forms, rule_forms)))
        }
    });

    let saving = RwSignal::new(false);

    let on_cancel = {
        let navigate = navigate.clone();
        Callback::new(move |_| {
            navigate("/billing/scorecards", Default::default());
        })
    };

    let on_save = {
        let navigate = navigate.clone();
        let toast = toast.clone();
        Callback::new(move |payload: TemplateSavePayload| {
            let tid = selected_tenant_id.get_untracked();
            if tid.is_empty() {
                toast.show_toast("Error", "Select a tenant before saving.", "error");
                return;
            }
            if saving.get_untracked() {
                return;
            }
            saving.set(true);
            let toast = toast.clone();
            let navigate = navigate.clone();
            let is_new = is_create();
            leptos::task::spawn_local(async move {
                let result = persist_payload(&tid, is_new, payload).await;
                saving.set(false);
                match result {
                    Ok(saved_id) => {
                        toast.show_toast("Saved", "Scorecard template saved.", "success");
                        navigate(
                            &format!(
                                "/billing/scorecards/templates/{saved_id}/configure?tenant_id={tid}"
                            ),
                            Default::default(),
                        );
                    }
                    Err(e) => toast.show_toast("Error", &e, "error"),
                }
            });
        })
    };

    view! {
        <div class="w-full space-y-4">
            <div class="flex flex-col md:flex-row justify-between items-start md:items-center gap-3 bg-surface-container-low border border-outline-variant/20 p-4 rounded-2xl">
                <div>
                    <a href="/billing/scorecards" class="text-xs text-primary hover:underline">
                        "← Back to Scorecards"
                    </a>
                    <h1 class="text-xl font-extrabold tracking-tight text-on-surface mt-1">
                        {move || if is_create() { "New Scorecard Template" } else { "Configure Template" }}
                    </h1>
                </div>
                <div class="flex items-center gap-2">
                    <label class="text-[10px] font-bold uppercase tracking-wider text-on-surface-variant">
                        "Tenant"
                    </label>
                    <Suspense fallback=|| view! { <span class="text-xs text-on-surface-variant">"Loading…"</span> }>
                        {move || {
                            let tenants = tenants_res.get().unwrap_or_default();
                            view! {
                                <select
                                    class="text-sm bg-surface-container-highest border border-outline-variant/30 rounded-lg px-3 py-1.5"
                                    prop:value=move || selected_tenant_id.get()
                                    on:change=move |ev| selected_tenant_id.set(event_target_value(&ev))
                                >
                                    <For
                                        each=move || tenants.clone()
                                        key=|t| t.tenant_id.clone()
                                        children=move |t| {
                                            let id = t.tenant_id.clone();
                                            let label = format!(
                                                "{} ({})",
                                                t.name,
                                                &t.tenant_id[..8.min(t.tenant_id.len())]
                                            );
                                            view! {
                                                <option
                                                    value=id.clone()
                                                    selected=move || selected_tenant_id.get() == id
                                                >
                                                    {label}
                                                </option>
                                            }
                                        }
                                    />
                                </select>
                            }
                        }}
                    </Suspense>
                </div>
            </div>

            <Suspense fallback=|| view! {
                <div class="w-full p-8 text-sm text-on-surface-variant">"Loading configurator…"</div>
            }>
                {move || match load_res.get() {
                    None => view! {
                        <div class="w-full p-8 text-sm text-on-surface-variant">"Loading…"</div>
                    }.into_any(),
                    Some(Err(e)) => view! {
                        <div class="w-full p-6 rounded-xl border border-error/30 bg-error/5 text-sm text-error">
                            {"Failed to load template: "}{e}
                        </div>
                    }.into_any(),
                    Some(Ok(None)) => view! {
                        <div class="w-full p-6 text-sm text-on-surface-variant">
                            "Select a tenant to configure scorecard templates."
                        </div>
                    }.into_any(),
                    Some(Ok(Some((tmpl, dims, rules)))) => view! {
                        <Configurator
                            mode=ConfiguratorMode::PlatformOperator
                            initial_template=tmpl
                            initial_dimensions=dims
                            initial_display_rules=rules
                            on_save=on_save
                            on_cancel=on_cancel
                        />
                    }.into_any(),
                }}
            </Suspense>
        </div>
    }
}

async fn persist_payload(
    tenant_id: &str,
    is_new: bool,
    payload: TemplateSavePayload,
) -> Result<Uuid, String> {
    let t = &payload.template;
    let display_config = display_config_json(&payload.display_config);

    let template_id = if is_new || t.id.is_none() {
        let created = create_template(
            tenant_id,
            &CreateTemplateInput {
                name: t.name.clone(),
                entity_type: t.entity_type.clone(),
                description: Some(t.description.clone()).filter(|s| !s.is_empty()),
                scoring_method: Some(t.scoring_method.clone()),
                default_scale_min: Some(t.default_scale_min),
                default_scale_max: Some(t.default_scale_max),
                min_entries_to_publish: Some(t.min_entries_to_publish),
                is_published: Some(t.is_published),
                template_scope: Some(t.template_scope.clone()),
                cold_start_strategy: Some(t.cold_start_strategy.clone()),
                cold_start_saturation_threshold: Some(t.cold_start_saturation_threshold),
                default_bayesian_prior_weight: t.default_bayesian_prior_weight,
                calibration_minimum_entries: Some(t.calibration_minimum_entries),
                display_config: display_config.clone(),
            },
        )
        .await?;
        created.id
    } else {
        let id = t.id.unwrap();
        update_template(
            tenant_id,
            &id.to_string(),
            &UpdateTemplateInput {
                name: Some(t.name.clone()),
                description: Some(t.description.clone()),
                scoring_method: Some(t.scoring_method.clone()),
                default_scale_min: Some(t.default_scale_min),
                default_scale_max: Some(t.default_scale_max),
                min_entries_to_publish: Some(t.min_entries_to_publish),
                is_published: Some(t.is_published),
                template_scope: Some(t.template_scope.clone()),
                cold_start_strategy: Some(t.cold_start_strategy.clone()),
                cold_start_saturation_threshold: Some(t.cold_start_saturation_threshold),
                default_bayesian_prior_weight: t.default_bayesian_prior_weight,
                calibration_minimum_entries: Some(t.calibration_minimum_entries),
                display_config,
            },
        )
        .await?;
        id
    };

    for dim in &payload.dimensions {
        if let Some(dim_id) = dim.id {
            update_dimension(
                tenant_id,
                &dim_id.to_string(),
                &UpdateDimensionInput {
                    name: Some(dim.name.clone()),
                    description: Some(dim.description.clone()).filter(|s| !s.is_empty()),
                    category: Some(dim.category.clone()).filter(|s| !s.is_empty()),
                    weight: Some(dim.weight),
                    scale_type: Some(dim.scale_type.to_string()),
                    scale_min: Some(dim.scale_min),
                    scale_max: Some(dim.scale_max),
                    unit_label: Some(dim.unit_label.clone()).filter(|s| !s.is_empty()),
                    min_entries_to_show: Some(dim.min_entries_to_show),
                    is_community_ratable: Some(dim.is_community_ratable),
                    is_active: Some(dim.is_active),
                    sort_order: Some(dim.sort_order),
                    is_inverted: Some(dim.is_inverted),
                    bayesian_prior_weight: dim.bayesian_prior_weight,
                    is_tenant_extension: Some(dim.is_tenant_extension),
                    global_reference_value: dim.global_reference_value,
                    global_reference_label: Some(dim.global_reference_label.clone())
                        .filter(|s| !s.is_empty()),
                    ..Default::default()
                },
            )
            .await?;
        } else if !dim.name.trim().is_empty() && !dim.slug.trim().is_empty() {
            create_dimension(
                tenant_id,
                &template_id.to_string(),
                &CreateDimensionInput {
                    slug: dim.slug.clone(),
                    name: dim.name.clone(),
                    scale_type: dim.scale_type.to_string(),
                    description: Some(dim.description.clone()).filter(|s| !s.is_empty()),
                    category: Some(dim.category.clone()).filter(|s| !s.is_empty()),
                    weight: Some(dim.weight),
                    scale_min: Some(dim.scale_min),
                    scale_max: Some(dim.scale_max),
                    unit_label: Some(dim.unit_label.clone()).filter(|s| !s.is_empty()),
                    benchmark_tiers: None,
                    global_reference_value: dim.global_reference_value,
                    global_reference_label: Some(dim.global_reference_label.clone())
                        .filter(|s| !s.is_empty()),
                    min_entries_to_show: Some(dim.min_entries_to_show),
                    is_community_ratable: Some(dim.is_community_ratable),
                    is_active: Some(dim.is_active),
                    sort_order: Some(dim.sort_order),
                    is_inverted: Some(dim.is_inverted),
                    bayesian_prior_weight: dim.bayesian_prior_weight,
                    is_tenant_extension: Some(dim.is_tenant_extension),
                },
            )
            .await?;
        }
    }

    // Display rules API is session-tenant scoped (no path tenant_id). Best-effort.
    for rule in &payload.display_rules {
        let value_list = {
            let list = rule.value_list();
            if list.is_empty() {
                None
            } else {
                Some(serde_json::json!(list))
            }
        };
        if let Some(rule_id) = rule.id {
            let _ = update_display_rule(
                &rule_id.to_string(),
                &UpdateDisplayRuleInput {
                    dimension_id: rule.dimension_id,
                    category_target: Some(rule.category_target.clone()).filter(|s| !s.is_empty()),
                    trigger_category: Some(rule.trigger_category.to_string()),
                    field_reference: Some(rule.field_reference.clone()).filter(|s| !s.is_empty()),
                    operator: Some(rule.operator.to_string()),
                    value: Some(rule.value.clone()).filter(|s| !s.is_empty()),
                    value_list,
                    action: Some(rule.action.to_string()),
                    alert_message: Some(rule.alert_message.clone()).filter(|s| !s.is_empty()),
                    mode_scope: Some(rule.mode_scope.to_string()),
                    priority: Some(rule.priority),
                    description: Some(rule.description.clone()).filter(|s| !s.is_empty()),
                    is_active: Some(rule.is_active),
                },
            )
            .await;
        } else {
            let _ = create_display_rule(&CreateDisplayRuleInput {
                template_id,
                dimension_id: rule.dimension_id,
                category_target: Some(rule.category_target.clone()).filter(|s| !s.is_empty()),
                trigger_category: rule.trigger_category.to_string(),
                field_reference: Some(rule.field_reference.clone()).filter(|s| !s.is_empty()),
                operator: rule.operator.to_string(),
                value: Some(rule.value.clone()).filter(|s| !s.is_empty()),
                value_list,
                action: rule.action.to_string(),
                alert_message: Some(rule.alert_message.clone()).filter(|s| !s.is_empty()),
                mode_scope: Some(rule.mode_scope.to_string()),
                priority: Some(rule.priority),
                description: Some(rule.description.clone()).filter(|s| !s.is_empty()),
            })
            .await;
        }
    }

    Ok(template_id)
}
