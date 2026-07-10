//! G-27 Scorecards list + analytics (platform-admin pilot).
//!
//! Contract: `docs/contracts/g27_scorecard_platform.md` §7, §9.

use crate::api::admin::get_tenant_stats;
use crate::api::models::TenantStatModel;
use crate::api::scorecards::{
    get_analytics, get_anomalies, get_leaderboard, list_templates, refresh_analytics,
    AnomalyAlert, LeaderboardEntry, PortfolioStats, ScorecardTemplate,
};
use leptos::prelude::*;

#[component]
pub fn Scorecards() -> impl IntoView {
    let active_tab = RwSignal::new("templates".to_string());
    let selected_tenant_id = RwSignal::new(String::new());
    let templates_refresh = RwSignal::new(0u32);

    // Analytics filters (Phase A — client-side where noted in contract §7)
    let analytics_template_id = RwSignal::new(String::new());
    let dimension_focus = RwSignal::new(String::new()); // dimension_id or empty = all
    let anomaly_direction = RwSignal::new(String::new()); // "" | "spike" | "drop"
    let confidence_filter = RwSignal::new(String::new()); // "" or confidence_level
    let entity_type_filter = RwSignal::new(String::new());
    let leaderboard_limit = RwSignal::new(25i64);
    let analytics_refresh = RwSignal::new(0u32);
    let refreshing = RwSignal::new(false);

    let tenants_res = LocalResource::new(|| async move { get_tenant_stats().await.unwrap_or_default() });

    // Auto-select first tenant when list loads and none selected.
    Effect::new(move |_| {
        if selected_tenant_id.get().is_empty() {
            if let Some(tenants) = tenants_res.get() {
                if let Some(first) = tenants.first() {
                    selected_tenant_id.set(first.tenant_id.clone());
                }
            }
        }
    });

    let templates_res = LocalResource::new(move || {
        let tid = selected_tenant_id.get();
        let _ = templates_refresh.get();
        async move {
            if tid.is_empty() {
                return Ok(Vec::<ScorecardTemplate>::new());
            }
            list_templates(&tid).await
        }
    });

    let analytics_bundle = LocalResource::new(move || {
        let tid = selected_tenant_id.get();
        let tmpl = analytics_template_id.get();
        let limit = leaderboard_limit.get();
        let _ = analytics_refresh.get();
        async move {
            if tid.is_empty() || tmpl.is_empty() {
                return Ok(None);
            }
            let stats = get_analytics(&tid, &tmpl).await?;
            let board = get_leaderboard(&tid, &tmpl, Some(limit)).await?;
            let anomalies = get_anomalies(&tid, &tmpl, Some(limit)).await?;
            Ok(Some((stats, board, anomalies)))
        }
    });

    view! {
        <div class="w-full space-y-6">
            // ── Page Header ──
            <div class="flex flex-col md:flex-row justify-between items-start md:items-center gap-4 bg-surface-container-low border border-outline-variant/20 p-6 rounded-2xl shadow-sm">
                <div>
                    <h1 class="text-2xl font-extrabold tracking-tight text-on-surface">"Scorecards"</h1>
                    <p class="text-xs text-on-surface-variant mt-1">
                        "Universal structured evaluation engine · customer tenant lens"
                    </p>
                </div>
                <div class="flex items-center gap-3 flex-wrap">
                    <TenantPicker
                        tenants_res=tenants_res
                        selected_tenant_id=selected_tenant_id
                    />
                    <a
                        href=move || {
                            let tid = selected_tenant_id.get();
                            if tid.is_empty() {
                                "/billing/scorecards".to_string()
                            } else {
                                format!("/billing/scorecards/templates/new/configure?tenant_id={tid}")
                            }
                        }
                        class="btn-primary-gradient px-4 py-2 rounded-lg text-sm font-semibold text-on-primary-container shadow-md shadow-primary/10 hover:opacity-90 active:scale-95 transition-all"
                    >
                        "+ New Template"
                    </a>
                </div>
            </div>

            // ── Sub Navigation ──
            <div class="flex border-b border-outline-variant/20 overflow-x-auto shrink-0 select-none">
                <button
                    class=move || tab_class(&active_tab.get(), "templates")
                    on:click=move |_| active_tab.set("templates".to_string())
                >
                    "Templates"
                </button>
                <button
                    class=move || tab_class(&active_tab.get(), "analytics")
                    on:click=move |_| active_tab.set("analytics".to_string())
                >
                    "Analytics"
                </button>
            </div>

            // ── VIEW: Templates ──
            <Show when=move || active_tab.get() == "templates">
                <TemplatesTab
                    templates_res=templates_res
                    selected_tenant_id=selected_tenant_id
                />
            </Show>

            // ── VIEW: Analytics ──
            <Show when=move || active_tab.get() == "analytics">
                <AnalyticsTab
                    templates_res=templates_res
                    analytics_bundle=analytics_bundle
                    selected_tenant_id=selected_tenant_id
                    analytics_template_id=analytics_template_id
                    dimension_focus=dimension_focus
                    anomaly_direction=anomaly_direction
                    confidence_filter=confidence_filter
                    entity_type_filter=entity_type_filter
                    leaderboard_limit=leaderboard_limit
                    analytics_refresh=analytics_refresh
                    refreshing=refreshing
                />
            </Show>
        </div>
    }
}

fn tab_class(active: &str, name: &str) -> &'static str {
    if active == name {
        "px-4 py-2.5 text-sm font-semibold text-primary border-b-2 border-primary transition-all shrink-0 bg-transparent"
    } else {
        "px-4 py-2.5 text-sm text-on-surface-variant hover:text-on-surface transition-all shrink-0 bg-transparent"
    }
}

#[component]
fn TenantPicker(
    tenants_res: LocalResource<Vec<TenantStatModel>>,
    selected_tenant_id: RwSignal<String>,
) -> impl IntoView {
    view! {
        <label class="flex items-center gap-2 text-xs text-on-surface-variant">
            <span class="font-semibold uppercase tracking-wider">"Tenant"</span>
            <select
                class="bg-surface-container border border-outline-variant/30 text-on-surface text-sm rounded-lg px-3 py-2 min-w-[200px] cursor-pointer"
                prop:value=move || selected_tenant_id.get()
                on:change=move |ev| selected_tenant_id.set(event_target_value(&ev))
            >
                <option value="">"Select tenant…"</option>
                {move || {
                    tenants_res.get().unwrap_or_default().into_iter().map(|t| {
                        let id = t.tenant_id.clone();
                        let label = format!("{} ({})", t.name, &t.tenant_id[..8.min(t.tenant_id.len())]);
                        view! {
                            <option value=id.clone() selected=move || selected_tenant_id.get() == id>
                                {label}
                            </option>
                        }
                    }).collect_view()
                }}
            </select>
        </label>
    }
}

#[component]
fn TemplatesTab(
    templates_res: LocalResource<Result<Vec<ScorecardTemplate>, String>>,
    selected_tenant_id: RwSignal<String>,
) -> impl IntoView {
    view! {
        <Show
            when=move || selected_tenant_id.get().is_empty()
            fallback=move || view! {
                <Suspense fallback=move || view! {
                    <p class="text-sm text-on-surface-variant">"Loading templates…"</p>
                }>
                    {move || match templates_res.get() {
                        None => view! { <p class="text-sm text-on-surface-variant">"Loading…"</p> }.into_any(),
                        Some(Err(e)) => view! {
                            <p class="text-sm text-red-400">{format!("Failed to load templates: {e}")}</p>
                        }.into_any(),
                        Some(Ok(templates)) => {
                            let published: Vec<_> = templates.iter().filter(|t| t.is_published).cloned().collect();
                            let drafts: Vec<_> = templates.iter().filter(|t| !t.is_published).cloned().collect();
                            view! {
                                <div class="space-y-6">
                                    <TemplateSection title=format!("Published Templates ({})", published.len()) templates=published />
                                    <TemplateSection title=format!("Draft Templates ({})", drafts.len()) templates=drafts />
                                    <Show when=move || templates.is_empty()>
                                        <p class="text-sm text-on-surface-variant">
                                            "No templates for this tenant yet."
                                        </p>
                                    </Show>
                                </div>
                            }.into_any()
                        }
                    }}
                </Suspense>
            }
        >
            <p class="text-sm text-on-surface-variant">"Select a tenant to list scorecard templates."</p>
        </Show>
    }
}

#[component]
fn TemplateSection(title: String, templates: Vec<ScorecardTemplate>) -> impl IntoView {
    let is_empty = templates.is_empty();
    view! {
        <Show when=move || !is_empty>
            <div class="space-y-4">
                <h3 class="text-[10px] font-bold uppercase tracking-wider text-on-surface-variant">{title.clone()}</h3>
                <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
                    {templates.iter().map(|t| {
                        let id = t.id.to_string();
                        let name = t.name.clone();
                        let entity = t.entity_type.clone();
                        let desc = t.description.clone().unwrap_or_default();
                        let scope = t.template_scope.clone();
                        let published = t.is_published;
                        let href = format!("/billing/scorecards/templates/{id}/configure");
                        view! {
                            <a
                                href=href
                                class="bg-surface-container-low border border-outline-variant/20 hover:border-primary/40 rounded-xl p-5 shadow-sm hover:shadow-md transition-all flex flex-col justify-between min-h-[180px] no-underline"
                            >
                                <div>
                                    <div class="flex justify-between items-start mb-3 gap-2">
                                        <span class="px-2 py-0.5 rounded text-[8px] font-bold bg-primary-container/20 text-primary border border-primary/20 uppercase tracking-wider">
                                            {entity}
                                        </span>
                                        <span class=if published {
                                            "inline-flex items-center px-1.5 py-0.5 rounded text-[8px] font-bold bg-emerald-500/10 text-emerald-400 uppercase tracking-wider"
                                        } else {
                                            "inline-flex items-center px-1.5 py-0.5 rounded text-[8px] font-bold bg-amber-500/10 text-amber-400 uppercase tracking-wider"
                                        }>
                                            {if published { "Published" } else { "Draft" }}
                                        </span>
                                    </div>
                                    <h4 class="text-sm font-bold text-on-surface">{name}</h4>
                                    <p class="text-[11px] text-on-surface-variant/70 mt-2 leading-relaxed line-clamp-3">
                                        {if desc.is_empty() { "No description.".to_string() } else { desc }}
                                    </p>
                                </div>
                                <div class="flex items-center justify-between mt-4 border-t border-outline-variant/10 pt-3 text-[10px] text-on-surface-variant">
                                    <span class="uppercase tracking-wider">{scope}</span>
                                    <span class="font-semibold text-primary">"Configure →"</span>
                                </div>
                            </a>
                        }
                    }).collect_view()}
                </div>
            </div>
        </Show>
    }
}

#[component]
fn AnalyticsTab(
    templates_res: LocalResource<Result<Vec<ScorecardTemplate>, String>>,
    analytics_bundle: LocalResource<Result<Option<(PortfolioStats, Vec<LeaderboardEntry>, Vec<AnomalyAlert>)>, String>>,
    selected_tenant_id: RwSignal<String>,
    analytics_template_id: RwSignal<String>,
    dimension_focus: RwSignal<String>,
    anomaly_direction: RwSignal<String>,
    confidence_filter: RwSignal<String>,
    entity_type_filter: RwSignal<String>,
    leaderboard_limit: RwSignal<i64>,
    analytics_refresh: RwSignal<u32>,
    refreshing: RwSignal<bool>,
) -> impl IntoView {
    // When templates load and no template selected, pick first.
    Effect::new(move |_| {
        if analytics_template_id.get().is_empty() {
            if let Some(Ok(tmpls)) = templates_res.get() {
                if let Some(first) = tmpls.first() {
                    analytics_template_id.set(first.id.to_string());
                }
            }
        }
    });

    let on_refresh = move |_| {
        let tid = selected_tenant_id.get();
        let tmpl = analytics_template_id.get();
        if tid.is_empty() || tmpl.is_empty() {
            return;
        }
        refreshing.set(true);
        leptos::task::spawn_local(async move {
            let _ = refresh_analytics(&tid, &tmpl).await;
            refreshing.set(false);
            analytics_refresh.update(|n| *n += 1);
        });
    };

    view! {
        <div class="space-y-6">
            // Filters
            <div class="flex flex-wrap gap-3 items-end bg-surface-container-low border border-outline-variant/20 rounded-xl p-4">
                <label class="flex flex-col gap-1 text-[10px] font-bold uppercase tracking-wider text-on-surface-variant">
                    "Template"
                    <select
                        class="bg-surface-container border border-outline-variant/30 text-on-surface text-sm rounded-lg px-3 py-2 min-w-[220px]"
                        prop:value=move || analytics_template_id.get()
                        on:change=move |ev| {
                            analytics_template_id.set(event_target_value(&ev));
                            dimension_focus.set(String::new());
                            analytics_refresh.update(|n| *n += 1);
                        }
                    >
                        <option value="">"Select template…"</option>
                        {move || {
                            templates_res.get()
                                .and_then(|r| r.ok())
                                .unwrap_or_default()
                                .into_iter()
                                .map(|t| {
                                    let id = t.id.to_string();
                                    let label = t.name.clone();
                                    view! {
                                        <option value=id.clone() selected=move || analytics_template_id.get() == id>
                                            {label}
                                        </option>
                                    }
                                })
                                .collect_view()
                        }}
                    </select>
                </label>

                <label class="flex flex-col gap-1 text-[10px] font-bold uppercase tracking-wider text-on-surface-variant">
                    "Dimension focus"
                    <select
                        class="bg-surface-container border border-outline-variant/30 text-on-surface text-sm rounded-lg px-3 py-2 min-w-[180px]"
                        prop:value=move || dimension_focus.get()
                        on:change=move |ev| dimension_focus.set(event_target_value(&ev))
                    >
                        <option value="">"All dimensions"</option>
                        {move || {
                            analytics_bundle.get()
                                .and_then(|r| r.ok())
                                .flatten()
                                .map(|(stats, _, _)| stats.dimensions)
                                .unwrap_or_default()
                                .into_iter()
                                .map(|d| {
                                    let id = d.dimension_id.to_string();
                                    let name = d.dimension_name.clone();
                                    view! {
                                        <option value=id.clone() selected=move || dimension_focus.get() == id>
                                            {name}
                                        </option>
                                    }
                                })
                                .collect_view()
                        }}
                    </select>
                </label>

                <label class="flex flex-col gap-1 text-[10px] font-bold uppercase tracking-wider text-on-surface-variant">
                    "Anomaly direction"
                    <select
                        class="bg-surface-container border border-outline-variant/30 text-on-surface text-sm rounded-lg px-3 py-2"
                        prop:value=move || anomaly_direction.get()
                        on:change=move |ev| anomaly_direction.set(event_target_value(&ev))
                    >
                        <option value="">"All"</option>
                        <option value="spike">"Spike"</option>
                        <option value="drop">"Drop"</option>
                    </select>
                </label>

                <label class="flex flex-col gap-1 text-[10px] font-bold uppercase tracking-wider text-on-surface-variant">
                    "Confidence"
                    <select
                        class="bg-surface-container border border-outline-variant/30 text-on-surface text-sm rounded-lg px-3 py-2"
                        prop:value=move || confidence_filter.get()
                        on:change=move |ev| confidence_filter.set(event_target_value(&ev))
                    >
                        <option value="">"All"</option>
                        <option value="very_high">"Very high"</option>
                        <option value="high">"High"</option>
                        <option value="medium">"Medium"</option>
                        <option value="low">"Low"</option>
                        <option value="insufficient">"Insufficient"</option>
                    </select>
                </label>

                <label class="flex flex-col gap-1 text-[10px] font-bold uppercase tracking-wider text-on-surface-variant">
                    "Entity type"
                    <input
                        type="text"
                        placeholder="e.g. atlas_account"
                        class="bg-surface-container border border-outline-variant/30 text-on-surface text-sm rounded-lg px-3 py-2 min-w-[160px]"
                        prop:value=move || entity_type_filter.get()
                        on:input=move |ev| entity_type_filter.set(event_target_value(&ev))
                    />
                </label>

                <label class="flex flex-col gap-1 text-[10px] font-bold uppercase tracking-wider text-on-surface-variant">
                    "Limit"
                    <input
                        type="number"
                        min="1"
                        max="100"
                        class="bg-surface-container border border-outline-variant/30 text-on-surface text-sm rounded-lg px-3 py-2 w-24"
                        prop:value=move || leaderboard_limit.get().to_string()
                        on:change=move |ev| {
                            if let Ok(n) = event_target_value(&ev).parse::<i64>() {
                                leaderboard_limit.set(n.clamp(1, 100));
                                analytics_refresh.update(|x| *x += 1);
                            }
                        }
                    />
                </label>

                <button
                    class="btn-primary px-4 py-2 rounded-lg text-sm font-semibold transition-all active:scale-95 disabled:opacity-50"
                    disabled=move || refreshing.get() || analytics_template_id.get().is_empty()
                    on:click=on_refresh
                >
                    {move || if refreshing.get() { "Refreshing…" } else { "Refresh" }}
                </button>
            </div>

            <Suspense fallback=move || view! {
                <p class="text-sm text-on-surface-variant">"Loading analytics…"</p>
            }>
                {move || match analytics_bundle.get() {
                    None => view! { <p class="text-sm text-on-surface-variant">"Loading…"</p> }.into_any(),
                    Some(Err(e)) => view! {
                        <p class="text-sm text-red-400">{format!("Analytics error: {e}")}</p>
                    }.into_any(),
                    Some(Ok(None)) => view! {
                        <p class="text-sm text-on-surface-variant">"Select a tenant and template to view analytics."</p>
                    }.into_any(),
                    Some(Ok(Some((stats, board, anomalies)))) => {
                        let focus = dimension_focus.get();
                        let dims: Vec<_> = stats
                            .dimensions
                            .iter()
                            .filter(|d| focus.is_empty() || d.dimension_id.to_string() == focus)
                            .cloned()
                            .collect();

                        let conf = confidence_filter.get();
                        let etype = entity_type_filter.get().to_lowercase();
                        let board: Vec<_> = board
                            .into_iter()
                            .filter(|e| {
                                (conf.is_empty() || e.confidence_level == conf)
                                    && (etype.is_empty()
                                        || e.subject_entity_type.to_lowercase().contains(&etype))
                            })
                            .collect();

                        let adir = anomaly_direction.get();
                        let anomalies: Vec<_> = anomalies
                            .into_iter()
                            .filter(|a| {
                                let dim_ok =
                                    focus.is_empty() || a.dimension_id.to_string() == focus;
                                let dir_ok = adir.is_empty()
                                    || a.anomaly_direction.as_deref() == Some(adir.as_str());
                                dim_ok && dir_ok
                            })
                            .collect();

                        let total_scorecards = stats.total_scorecards;
                        let template_hint =
                            format!("Template {}", &stats.template_id.to_string()[..8]);
                        let dims_count = dims.len();
                        let focus_hint = if focus.is_empty() {
                            "All dimensions".to_string()
                        } else {
                            "Focused".to_string()
                        };
                        let board_count = board.len();
                        let anomaly_count = anomalies.len();
                        let dims_empty = dims.is_empty();
                        let board_empty = board.is_empty();
                        let anomalies_empty = anomalies.is_empty();

                        view! {
                            <div class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-4">
                                <StatCard label="Total scorecards".into() value=total_scorecards.to_string() hint=template_hint />
                                <StatCard label="Dimensions".into() value=dims_count.to_string() hint=focus_hint />
                                <StatCard label="Leaderboard rows".into() value=board_count.to_string() hint="After client filters".into() />
                                <StatCard label="Anomalies".into() value=anomaly_count.to_string() hint="After client filters".into() />
                            </div>

                            <div class="grid grid-cols-1 lg:grid-cols-2 gap-6">
                                <div class="bg-surface-container-low border border-outline-variant/20 rounded-xl p-6 shadow-sm space-y-4">
                                    <h3 class="text-xs font-bold uppercase tracking-wider text-on-surface-variant">"Portfolio dimensions"</h3>
                                    {if dims_empty {
                                        view! { <p class="text-xs text-on-surface-variant">"No dimension stats."</p> }.into_any()
                                    } else {
                                        view! {
                                            <div class="space-y-3">
                                                {dims.into_iter().map(|d| {
                                                    let mean = d.pool_mean.map(|m| format!("{m:.2}")).unwrap_or_else(|| "—".into());
                                                    view! {
                                                        <div class="flex items-center justify-between text-xs border-b border-outline-variant/10 pb-2">
                                                            <div>
                                                                <div class="font-semibold text-on-surface">{d.dimension_name}</div>
                                                                <div class="text-on-surface-variant/60">{format!("{} · n={}", d.dimension_slug, d.cohort_size)}</div>
                                                            </div>
                                                            <div class="text-right font-mono">
                                                                <div class="text-on-surface">{mean}</div>
                                                                <div class="text-on-surface-variant/60">
                                                                    {format!("↑{} ↓{}", d.improving_count, d.declining_count)}
                                                                </div>
                                                            </div>
                                                        </div>
                                                    }
                                                }).collect_view()}
                                            </div>
                                        }.into_any()
                                    }}
                                </div>

                                <div class="bg-surface-container-low border border-outline-variant/20 rounded-xl p-6 shadow-sm space-y-4">
                                    <h3 class="text-xs font-bold uppercase tracking-wider text-on-surface-variant">"Anomalies"</h3>
                                    {if anomalies_empty {
                                        view! { <p class="text-xs text-on-surface-variant">"No anomalies for current filters."</p> }.into_any()
                                    } else {
                                        view! {
                                            <div class="space-y-3">
                                                {anomalies.into_iter().map(|a| {
                                                    let dir = a.anomaly_direction.unwrap_or_else(|| "—".into());
                                                    let z = a.z_score.map(|z| format!("{z:.2}")).unwrap_or_else(|| "—".into());
                                                    let href = format!("/billing/scorecards/{}", a.scorecard_id);
                                                    let sid_short = a.scorecard_id.to_string();
                                                    let sid_short = sid_short[..8.min(sid_short.len())].to_string();
                                                    view! {
                                                        <a href=href class="block text-xs border-b border-outline-variant/10 pb-2 no-underline hover:bg-surface-bright/10 rounded px-1 -mx-1">
                                                            <div class="flex justify-between">
                                                                <span class="font-semibold text-on-surface">{a.dimension_name}</span>
                                                                <span class="uppercase text-[10px] font-bold text-amber-400">{dir}</span>
                                                            </div>
                                                            <div class="text-on-surface-variant/60 mt-0.5">
                                                                {format!("{} · z={} · {}", a.period_start, z, sid_short)}
                                                            </div>
                                                        </a>
                                                    }
                                                }).collect_view()}
                                            </div>
                                        }.into_any()
                                    }}
                                </div>
                            </div>

                            <div class="bg-surface-container-low border border-outline-variant/20 rounded-xl p-6 shadow-sm space-y-4">
                                <h3 class="text-xs font-bold uppercase tracking-wider text-on-surface-variant">"Leaderboard"</h3>
                                {if board_empty {
                                    view! { <p class="text-xs text-on-surface-variant">"No leaderboard rows for current filters."</p> }.into_any()
                                } else {
                                    view! {
                                        <div class="overflow-x-auto">
                                            <table class="w-full text-xs">
                                                <thead>
                                                    <tr class="text-left text-on-surface-variant border-b border-outline-variant/20">
                                                        <th class="py-2 pr-3">"#"</th>
                                                        <th class="py-2 pr-3">"Subject"</th>
                                                        <th class="py-2 pr-3">"Type"</th>
                                                        <th class="py-2 pr-3">"Score"</th>
                                                        <th class="py-2 pr-3">"Confidence"</th>
                                                        <th class="py-2">"Trend"</th>
                                                    </tr>
                                                </thead>
                                                <tbody>
                                                    {board.into_iter().map(|e| {
                                                        let href = format!("/billing/scorecards/{}", e.scorecard_id);
                                                        let score = e.composite_score.map(|s| format!("{s:.2}")).unwrap_or_else(|| "—".into());
                                                        let trend = e.trend_direction.unwrap_or_else(|| "—".into());
                                                        view! {
                                                            <tr class="border-b border-outline-variant/10 hover:bg-surface-bright/10">
                                                                <td class="py-2 pr-3 font-mono">{e.rank}</td>
                                                                <td class="py-2 pr-3">
                                                                    <a href=href class="text-primary hover:underline font-mono">
                                                                        {e.subject_entity_id}
                                                                    </a>
                                                                </td>
                                                                <td class="py-2 pr-3">{e.subject_entity_type}</td>
                                                                <td class="py-2 pr-3 font-mono">{score}</td>
                                                                <td class="py-2 pr-3">{e.confidence_level}</td>
                                                                <td class="py-2">{trend}</td>
                                                            </tr>
                                                        }
                                                    }).collect_view()}
                                                </tbody>
                                            </table>
                                        </div>
                                    }.into_any()
                                }}
                            </div>
                        }.into_any()
                    }
                }}
            </Suspense>
        </div>
    }
}

#[component]
fn StatCard(label: String, value: String, hint: String) -> impl IntoView {
    view! {
        <div class="bg-surface-container-low border border-outline-variant/20 rounded-xl p-5 shadow-sm flex flex-col justify-between min-h-[100px]">
            <span class="text-[10px] font-bold text-on-surface-variant/70 uppercase tracking-widest">{label}</span>
            <span class="text-3xl font-extrabold text-on-surface tracking-tight mt-2">{value}</span>
            <span class="text-[10px] text-on-surface-variant/50 mt-1">{hint}</span>
        </div>
    }
}
