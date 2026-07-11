//! G-36 Program detail.

use leptos::prelude::*;
use leptos_router::hooks::use_params_map;
use uuid::Uuid;

use crate::api::admin::get_all_platform_apps;
use crate::api::programs::{
    Program, ProgramAnalytics, ProgramOutcomeType, ProgramRewardBeneficiary, ProgramRewardType,
    RewardRule, RewardRuleInput, amount_display, get_program, get_program_analytics,
    list_instance_enablements, list_program_actions, list_program_grants, list_reward_rules,
    replace_reward_rules, set_program_enabled_for_instance,
};
use crate::components::gtm_process_strip::{GtmProcessStrip, GtmStage};

#[derive(Debug, Clone)]
struct RewardRuleForm {
    beneficiary: ProgramRewardBeneficiary,
    reward_type: ProgramRewardType,
    amount: String,
    trigger_outcome_type: ProgramOutcomeType,
    is_active: bool,
}

impl From<RewardRule> for RewardRuleForm {
    fn from(rule: RewardRule) -> Self {
        Self {
            beneficiary: rule.beneficiary,
            reward_type: rule.reward_type,
            amount: amount_display(&rule.amount),
            trigger_outcome_type: rule.trigger_outcome_type,
            is_active: rule.is_active,
        }
    }
}

#[component]
pub fn ProgramDetail() -> impl IntoView {
    let params = use_params_map();
    let toast = use_context::<crate::app::GlobalToast>().expect("toast context");

    let program_id = move || params.with(|p| p.get("id").and_then(|id| Uuid::parse_str(&id).ok()));

    let active_tab = RwSignal::new("overview".to_string());
    let rule_forms = RwSignal::new(Vec::<RewardRuleForm>::new());
    let rules_synced = RwSignal::new(false);

    let program_res = LocalResource::new(move || async move {
        match program_id() {
            Some(id) => get_program(id).await.ok(),
            None => None,
        }
    });
    let analytics_res = LocalResource::new(move || async move {
        match program_id() {
            Some(id) => get_program_analytics(id).await.ok(),
            None => None,
        }
    });
    let rules_res = LocalResource::new(move || async move {
        match program_id() {
            Some(id) => list_reward_rules(id).await.unwrap_or_default(),
            None => vec![],
        }
    });
    let actions_res = LocalResource::new(move || async move {
        match program_id() {
            Some(id) => list_program_actions(id).await.unwrap_or_default(),
            None => vec![],
        }
    });
    let grants_res = LocalResource::new(move || async move {
        match program_id() {
            Some(id) => list_program_grants(id).await.unwrap_or_default(),
            None => vec![],
        }
    });
    let enablements_res = LocalResource::new(move || async move {
        match program_id() {
            Some(id) => list_instance_enablements(id).await.unwrap_or_default(),
            None => vec![],
        }
    });
    let apps_res =
        LocalResource::new(|| async { get_all_platform_apps().await.unwrap_or_default() });

    Effect::new(move |_| {
        if !rules_synced.get() {
            if let Some(rules) = rules_res.get() {
                rule_forms.set(rules.into_iter().map(RewardRuleForm::from).collect());
                rules_synced.set(true);
            }
        }
    });

    let save_rules = {
        let toast = toast.clone();
        move |_| {
            let Some(id) = program_id() else {
                return;
            };
            let rules = rule_forms
                .get()
                .into_iter()
                .map(|rule| RewardRuleInput {
                    beneficiary: rule.beneficiary,
                    reward_type: rule.reward_type,
                    amount: rule.amount,
                    trigger_outcome_type: rule.trigger_outcome_type,
                    is_active: Some(rule.is_active),
                })
                .collect::<Vec<_>>();
            let toast = toast.clone();
            leptos::task::spawn_local(async move {
                match replace_reward_rules(id, rules).await {
                    Ok(_) => {
                        toast.show_toast("Saved", "Reward rules updated.", "success");
                        rules_synced.set(false);
                        rules_res.refetch();
                    }
                    Err(e) => toast.show_toast("Error", &e, "error"),
                }
            });
        }
    };

    view! {
        <div class="main-canvas">
            <GtmProcessStrip
                active=GtmStage::Programs
                subtitle="Tune rewards, actions, grants, and per-instance coverage."
            />

            <Suspense fallback=move || view! {
                <div class="page-header"><div class="page-title">"Loading program..."</div></div>
            }>
                {move || {
                    match program_res.get().flatten() {
                        None => view! {
                            <div class="page-header">
                                <div>
                                    <div class="page-title">"Program not found"</div>
                                    <div class="page-subtitle">"Check the program ID and try again."</div>
                                </div>
                                <a href="/programs" class="btn btn-ghost">"Back to Programs"</a>
                            </div>
                        }.into_any(),
                        Some(program) => view! {
                            <div class="page-header">
                                <div>
                                    <div class="page-title">{program.name.clone()}</div>
                                    <div class="page-subtitle">
                                        <span>{program.description.clone().unwrap_or_else(|| "Growth program template".to_string())}</span>
                                        <span class="mx-2 text-on-surface-variant/40">"·"</span>
                                        <code class="text-primary">{program.slug.clone()}</code>
                                    </div>
                                </div>
                                <a href="/programs" class="btn btn-ghost">"Back to Programs"</a>
                            </div>

                            <div class="tab-bar">
                                {["overview", "rewards", "actions", "grants", "coverage"].into_iter().map(|tab| {
                                    let label = match tab {
                                        "overview" => "Overview",
                                        "rewards" => "Reward rules",
                                        "actions" => "Actions",
                                        "grants" => "Grants",
                                        "coverage" => "Instance coverage",
                                        _ => tab,
                                    };
                                    view! {
                                        <button
                                            class=move || if active_tab.get() == tab { "tab active" } else { "tab" }
                                            on:click=move |_| active_tab.set(tab.to_string())
                                        >
                                            {label}
                                        </button>
                                    }
                                }).collect_view()}
                            </div>

                            {move || match active_tab.get().as_str() {
                                "overview" => view! {
                                    <OverviewTab program=program.clone() analytics=analytics_res.get().flatten() />
                                }.into_any(),
                                "rewards" => view! {
                                    <RewardRulesTab rule_forms=rule_forms on_save=save_rules />
                                }.into_any(),
                                "actions" => view! {
                                    <ActionsTab actions_res=actions_res />
                                }.into_any(),
                                "grants" => view! {
                                    <GrantsTab grants_res=grants_res />
                                }.into_any(),
                                "coverage" => view! {
                                    <CoverageTab
                                        program_id=program.id
                                        apps_res=apps_res
                                        enablements_res=enablements_res
                                    />
                                }.into_any(),
                                _ => view! { <></> }.into_any(),
                            }}
                        }.into_any(),
                    }
                }}
            </Suspense>
        </div>
    }
}

#[component]
fn OverviewTab(program: Program, analytics: Option<ProgramAnalytics>) -> impl IntoView {
    let analytics = analytics.unwrap_or(ProgramAnalytics {
        total_actions: 0,
        total_grants: 0,
        actions_by_status: vec![],
        outcomes_by_status: vec![],
        grants_by_status: vec![],
    });

    view! {
        <div class="space-y-4">
            <div class="kpi-row">
                <KpiCard label="Actions" value=analytics.total_actions.to_string() hint="all time" />
                <KpiCard label="Grants" value=analytics.total_grants.to_string() hint="all time" />
                <KpiCard label="Outcome" value=program.default_outcome_type.label().to_string() hint="default" />
                <KpiCard label="Status" value=if program.is_active { "Active".to_string() } else { "Inactive".to_string() } hint="catalog" />
            </div>
            <div class="section">
                <div class="section-hdr"><span class="section-title">"Program Configuration"</span></div>
                <div class="grid grid-cols-1 md:grid-cols-2 gap-4 p-4 text-xs">
                    <Info label="Kind" value=program.program_kind.label().to_string() />
                    <Info label="Slug" value=program.slug.clone() />
                    <Info label="Actor roles" value=empty_dash(program.actor_roles_display()) />
                    <Info label="Target roles" value=empty_dash(program.target_roles_display()) />
                </div>
            </div>
        </div>
    }
}

#[component]
fn RewardRulesTab(
    rule_forms: RwSignal<Vec<RewardRuleForm>>,
    on_save: impl Fn(leptos::ev::MouseEvent) + Clone + 'static,
) -> impl IntoView {
    let add_rule = move |_| {
        rule_forms.update(|rules| {
            rules.push(RewardRuleForm {
                beneficiary: ProgramRewardBeneficiary::Actor,
                reward_type: ProgramRewardType::SubscriptionCreditDays,
                amount: "30".to_string(),
                trigger_outcome_type: ProgramOutcomeType::Signup,
                is_active: true,
            });
        });
    };

    view! {
        <div class="section">
            <div class="section-hdr">
                <span class="section-title">"Editable Reward Rules"</span>
                <div class="flex gap-2">
                    <button class="btn btn-ghost btn-sm" on:click=add_rule>"+ Add Rule"</button>
                    <button class="btn btn-primary btn-sm" on:click=on_save>"Save"</button>
                </div>
            </div>
            <div class="divide-y divide-outline-variant/10">
                {move || {
                    let rules = rule_forms.get();
                    if rules.is_empty() {
                        return view! {
                            <div class="p-8 text-center text-sm text-on-surface-variant/70">
                                "No reward rules yet. Add a rule to start granting incentives."
                            </div>
                        }.into_any();
                    }

                    rules.into_iter().enumerate().map(|(idx, rule)| {
                        view! {
                            <div class="grid grid-cols-1 md:grid-cols-6 gap-3 p-4 items-end">
                                <FieldSelect
                                    label="Beneficiary"
                                    value=rule.beneficiary.to_string()
                                    options=vec![
                                        ("actor".to_string(), "Actor".to_string()),
                                        ("target".to_string(), "Target".to_string()),
                                    ]
                                    on_change=move |value| {
                                        rule_forms.update(|rules| {
                                            if let Some(rule) = rules.get_mut(idx) {
                                                rule.beneficiary = if value == "target" {
                                                    ProgramRewardBeneficiary::Target
                                                } else {
                                                    ProgramRewardBeneficiary::Actor
                                                };
                                            }
                                        });
                                    }
                                />
                                <FieldSelect
                                    label="Reward"
                                    value=rule.reward_type.to_string()
                                    options=vec![
                                        ("subscription_credit_days".to_string(), "Credit days".to_string()),
                                        ("feature_unlock".to_string(), "Feature unlock".to_string()),
                                        ("none".to_string(), "None".to_string()),
                                    ]
                                    on_change=move |value| {
                                        rule_forms.update(|rules| {
                                            if let Some(rule) = rules.get_mut(idx) {
                                                rule.reward_type = match value.as_str() {
                                                    "feature_unlock" => ProgramRewardType::FeatureUnlock,
                                                    "none" => ProgramRewardType::None,
                                                    _ => ProgramRewardType::SubscriptionCreditDays,
                                                };
                                            }
                                        });
                                    }
                                />
                                <div class="space-y-1">
                                    <label class="text-[10px] font-bold uppercase tracking-wider text-on-surface-variant/70">"Amount"</label>
                                    <input
                                        class="input input-sm"
                                        prop:value=rule.amount.clone()
                                        on:input=move |e| {
                                            let value = event_target_value(&e);
                                            rule_forms.update(|rules| {
                                                if let Some(rule) = rules.get_mut(idx) {
                                                    rule.amount = value.clone();
                                                }
                                            });
                                        }
                                    />
                                </div>
                                <FieldSelect
                                    label="Trigger"
                                    value=rule.trigger_outcome_type.to_string()
                                    options=outcome_options()
                                    on_change=move |value| {
                                        rule_forms.update(|rules| {
                                            if let Some(rule) = rules.get_mut(idx) {
                                                rule.trigger_outcome_type = parse_outcome(&value);
                                            }
                                        });
                                    }
                                />
                                <label class="flex items-center gap-2 text-xs text-on-surface-variant pb-2">
                                    <input
                                        type="checkbox"
                                        prop:checked=rule.is_active
                                        on:change=move |_| {
                                            rule_forms.update(|rules| {
                                                if let Some(rule) = rules.get_mut(idx) {
                                                    rule.is_active = !rule.is_active;
                                                }
                                            });
                                        }
                                    />
                                    "Active"
                                </label>
                                <button
                                    class="btn btn-ghost btn-sm"
                                    on:click=move |_| {
                                        rule_forms.update(|rules| {
                                            if idx < rules.len() {
                                                rules.remove(idx);
                                            }
                                        });
                                    }
                                >
                                    "Remove"
                                </button>
                            </div>
                        }
                    }).collect_view().into_any()
                }}
            </div>
        </div>
    }
}

#[component]
fn ActionsTab(
    actions_res: LocalResource<Vec<crate::api::programs::ProgramAction>>,
) -> impl IntoView {
    view! {
        <div class="section">
            <div class="section-hdr"><span class="section-title">"Program Actions"</span></div>
            <TableShell columns=7>
                <thead>
                    <tr class="text-[10px] uppercase tracking-wider text-on-surface-variant/70 border-b border-outline-variant/10">
                        <th class="px-4 py-3 text-left">"Target"</th>
                        <th class="px-4 py-3 text-left">"Role"</th>
                        <th class="px-4 py-3 text-left">"Status"</th>
                        <th class="px-4 py-3 text-left">"Invite"</th>
                        <th class="px-4 py-3 text-left">"Outcome"</th>
                        <th class="px-4 py-3 text-left">"Created"</th>
                        <th class="px-4 py-3 text-left">"Actor"</th>
                    </tr>
                </thead>
                <tbody class="divide-y divide-outline-variant/10">
                    <Suspense fallback=move || view! { <tr><td colspan="7" class="px-4 py-8 text-center text-sm text-on-surface-variant">"Loading actions..."</td></tr> }>
                        {move || {
                            let actions = actions_res.get().unwrap_or_default();
                            if actions.is_empty() {
                                return empty_row("No actions recorded yet.", 7).into_any();
                            }
                            actions.into_iter().map(|action| view! {
                                <tr>
                                    <td class="px-4 py-3 text-xs">{action.target_email.unwrap_or_else(|| "-".to_string())}</td>
                                    <td class="px-4 py-3 text-xs">{action.target_role.unwrap_or_else(|| "-".to_string())}</td>
                                    <td class="px-4 py-3 text-xs"><span class="pill">{format!("{:?}", action.status)}</span></td>
                                    <td class="px-4 py-3 text-xs font-mono">{action.invite_code.unwrap_or_else(|| "-".to_string())}</td>
                                    <td class="px-4 py-3 text-xs">{action.outcome_type.map(|o| o.label().to_string()).unwrap_or_else(|| "-".to_string())}</td>
                                    <td class="px-4 py-3 text-xs">{action.created_at}</td>
                                    <td class="px-4 py-3 text-[10px] font-mono">{action.actor_user_id.to_string()}</td>
                                </tr>
                            }).collect_view().into_any()
                        }}
                    </Suspense>
                </tbody>
            </TableShell>
        </div>
    }
}

#[component]
fn GrantsTab(grants_res: LocalResource<Vec<crate::api::programs::ProgramGrant>>) -> impl IntoView {
    view! {
        <div class="section">
            <div class="section-hdr"><span class="section-title">"Reward Grants"</span></div>
            <TableShell columns=6>
                <thead>
                    <tr class="text-[10px] uppercase tracking-wider text-on-surface-variant/70 border-b border-outline-variant/10">
                        <th class="px-4 py-3 text-left">"Beneficiary"</th>
                        <th class="px-4 py-3 text-left">"Reward"</th>
                        <th class="px-4 py-3 text-left">"Amount"</th>
                        <th class="px-4 py-3 text-left">"Status"</th>
                        <th class="px-4 py-3 text-left">"Granted"</th>
                        <th class="px-4 py-3 text-left">"Created"</th>
                    </tr>
                </thead>
                <tbody class="divide-y divide-outline-variant/10">
                    <Suspense fallback=move || view! { <tr><td colspan="6" class="px-4 py-8 text-center text-sm text-on-surface-variant">"Loading grants..."</td></tr> }>
                        {move || {
                            let grants = grants_res.get().unwrap_or_default();
                            if grants.is_empty() {
                                return empty_row("No reward grants yet.", 6).into_any();
                            }
                            grants.into_iter().map(|grant| view! {
                                <tr>
                                    <td class="px-4 py-3 text-[10px] font-mono">{grant.beneficiary_user_id.to_string()}</td>
                                    <td class="px-4 py-3 text-xs">{grant.reward_type.map(|r| r.label().to_string()).unwrap_or_else(|| "-".to_string())}</td>
                                    <td class="px-4 py-3 text-xs">{grant.amount.as_ref().map(amount_display).unwrap_or_else(|| "-".to_string())}</td>
                                    <td class="px-4 py-3 text-xs"><span class="pill">{format!("{:?}", grant.status)}</span></td>
                                    <td class="px-4 py-3 text-xs">{grant.granted_at.unwrap_or_else(|| "-".to_string())}</td>
                                    <td class="px-4 py-3 text-xs">{grant.created_at}</td>
                                </tr>
                            }).collect_view().into_any()
                        }}
                    </Suspense>
                </tbody>
            </TableShell>
        </div>
    }
}

#[component]
fn CoverageTab(
    program_id: Uuid,
    apps_res: LocalResource<Vec<crate::api::models::PlatformAppSummary>>,
    enablements_res: LocalResource<Vec<crate::api::programs::ProgramInstanceEnablement>>,
) -> impl IntoView {
    let toast = use_context::<crate::app::GlobalToast>().expect("toast context");

    view! {
        <div class="section">
            <div class="section-hdr"><span class="section-title">"Instance Coverage"</span></div>
            <Suspense fallback=move || view! {
                <div class="p-8 text-center text-sm text-on-surface-variant">"Loading instances..."</div>
            }>
                {move || {
                    let apps = apps_res.get().unwrap_or_default();
                    let enablements = enablements_res.get().unwrap_or_default();

                    if apps.is_empty() {
                        return view! {
                            <div class="p-8 text-center text-sm text-on-surface-variant/70">
                                "No app instances available."
                            </div>
                        }.into_any();
                    }

                    view! {
                        <div class="divide-y divide-outline-variant/10">
                            {apps.into_iter().map(|app| {
                                let parsed_id = Uuid::parse_str(&app.instance_id).ok();
                                let explicit = parsed_id.and_then(|id| {
                                    enablements.iter().find(|e| e.app_instance_id == id).cloned()
                                });
                                let enabled = explicit.as_ref().map(|e| e.is_enabled).unwrap_or(true);
                                let instance_id = app.instance_id.clone();
                                let name = app.name.clone();
                                let toast = toast.clone();
                                view! {
                                    <div class="flex items-center justify-between gap-4 p-4">
                                        <div>
                                            <div class="text-sm font-semibold text-on-surface">{name}</div>
                                            <div class="text-[10px] font-mono text-on-surface-variant/60">{instance_id.clone()}</div>
                                            <div class="text-xs text-on-surface-variant/60">{format!("{} · {}", app.app_type, app.domain)}</div>
                                        </div>
                                        <button
                                            class=if enabled { "btn btn-primary btn-sm" } else { "btn btn-ghost btn-sm" }
                                            on:click=move |_| {
                                                let Some(app_instance_id) = Uuid::parse_str(&instance_id).ok() else {
                                                    toast.show_toast("Invalid instance", "Instance ID is not a UUID.", "error");
                                                    return;
                                                };
                                                let next = !enabled;
                                                let toast = toast.clone();
                                                leptos::task::spawn_local(async move {
                                                    match set_program_enabled_for_instance(program_id, app_instance_id, next).await {
                                                        Ok(_) => {
                                                            toast.show_toast("Coverage updated", "Program enablement saved.", "success");
                                                            enablements_res.refetch();
                                                        }
                                                        Err(e) => toast.show_toast("Error", &e, "error"),
                                                    }
                                                });
                                            }
                                        >
                                            {if enabled { "Enabled" } else { "Disabled" }}
                                        </button>
                                    </div>
                                }
                            }).collect_view()}
                        </div>
                    }.into_any()
                }}
            </Suspense>
        </div>
    }
}

#[component]
fn FieldSelect(
    label: &'static str,
    value: String,
    options: Vec<(String, String)>,
    on_change: impl Fn(String) + Clone + 'static,
) -> impl IntoView {
    view! {
        <div class="space-y-1">
            <label class="text-[10px] font-bold uppercase tracking-wider text-on-surface-variant/70">{label}</label>
            <select
                class="input input-sm"
                prop:value=value
                on:change=move |e| on_change(event_target_value(&e))
            >
                {options.into_iter().map(|(value, label)| view! {
                    <option value=value>{label}</option>
                }).collect_view()}
            </select>
        </div>
    }
}

#[component]
fn KpiCard(label: &'static str, value: String, hint: &'static str) -> impl IntoView {
    view! {
        <div class="kpi-card">
            <div class="kpi-label">{label}</div>
            <div class="kpi-value">{value}</div>
            <div class="kpi-delta positive">{hint}</div>
        </div>
    }
}

#[component]
fn Info(label: &'static str, value: String) -> impl IntoView {
    view! {
        <div>
            <div class="text-[10px] uppercase tracking-wider text-on-surface-variant/50 mb-1">{label}</div>
            <div class="text-sm text-on-surface">{value}</div>
        </div>
    }
}

#[component]
fn TableShell(columns: usize, children: Children) -> impl IntoView {
    let _ = columns;
    view! {
        <div class="overflow-x-auto">
            <table class="w-full text-left">{children()}</table>
        </div>
    }
}

fn empty_row(message: &'static str, columns: usize) -> impl IntoView {
    view! {
        <tr>
            <td colspan=columns.to_string() class="px-4 py-8 text-center text-sm text-on-surface-variant/70">
                {message}
            </td>
        </tr>
    }
}

fn outcome_options() -> Vec<(String, String)> {
    [
        ProgramOutcomeType::Signup,
        ProgramOutcomeType::WizardComplete,
        ProgramOutcomeType::FormSubmit,
        ProgramOutcomeType::ReviewSubmitted,
        ProgramOutcomeType::FirstJobLogged,
        ProgramOutcomeType::SubscriptionActivated,
    ]
    .into_iter()
    .map(|outcome| (outcome.to_string(), outcome.label().to_string()))
    .collect()
}

fn parse_outcome(value: &str) -> ProgramOutcomeType {
    match value {
        "wizard_complete" => ProgramOutcomeType::WizardComplete,
        "form_submit" => ProgramOutcomeType::FormSubmit,
        "review_submitted" => ProgramOutcomeType::ReviewSubmitted,
        "first_job_logged" => ProgramOutcomeType::FirstJobLogged,
        "subscription_activated" => ProgramOutcomeType::SubscriptionActivated,
        _ => ProgramOutcomeType::Signup,
    }
}

fn empty_dash(value: String) -> String {
    if value.trim().is_empty() {
        "-".to_string()
    } else {
        value
    }
}
