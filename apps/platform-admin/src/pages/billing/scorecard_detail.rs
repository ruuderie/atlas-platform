//! Stub: scorecard entity detail (G-27).

use crate::api::admin::get_tenant_stats;
use crate::api::scorecards::{get_scorecard, ScorecardDetail};
use leptos::prelude::*;
use leptos_router::hooks::use_params_map;

#[component]
pub fn ScorecardDetailPage() -> impl IntoView {
    let params = use_params_map();
    let scorecard_id = move || {
        params.with(|p| p.get("scorecard_id").unwrap_or_default())
    };
    let selected_tenant_id = RwSignal::new(String::new());

    let tenants_res = LocalResource::new(|| async move { get_tenant_stats().await.unwrap_or_default() });

    Effect::new(move |_| {
        if selected_tenant_id.get().is_empty() {
            if let Some(tenants) = tenants_res.get() {
                if let Some(first) = tenants.first() {
                    selected_tenant_id.set(first.tenant_id.clone());
                }
            }
        }
    });

    let detail_res = LocalResource::new(move || {
        let tid = selected_tenant_id.get();
        let sid = scorecard_id();
        async move {
            if tid.is_empty() || sid.is_empty() {
                return Err("Missing tenant or scorecard id".to_string());
            }
            get_scorecard(&tid, &sid).await
        }
    });

    view! {
        <div class="w-full space-y-4">
            <div class="bg-surface-container-low border border-outline-variant/20 p-6 rounded-2xl shadow-sm space-y-4">
                <div class="flex flex-col md:flex-row md:items-center justify-between gap-3">
                    <div>
                        <h1 class="text-2xl font-extrabold tracking-tight text-on-surface">"Scorecard"</h1>
                        <p class="text-xs text-on-surface-variant mt-1 font-mono">
                            {move || format!("scorecard_id: {}", scorecard_id())}
                        </p>
                    </div>
                    <label class="flex items-center gap-2 text-xs text-on-surface-variant">
                        <span class="font-semibold uppercase tracking-wider">"Tenant"</span>
                        <select
                            class="bg-surface-container border border-outline-variant/30 text-on-surface text-sm rounded-lg px-3 py-2 min-w-[200px]"
                            prop:value=move || selected_tenant_id.get()
                            on:change=move |ev| selected_tenant_id.set(event_target_value(&ev))
                        >
                            {move || {
                                tenants_res.get().unwrap_or_default().into_iter().map(|t| {
                                    let id = t.tenant_id.clone();
                                    let label = t.name.clone();
                                    view! {
                                        <option value=id.clone() selected=move || selected_tenant_id.get() == id>
                                            {label}
                                        </option>
                                    }
                                }).collect_view()
                            }}
                        </select>
                    </label>
                </div>

                <Suspense fallback=move || view! {
                    <p class="text-sm text-on-surface-variant">"Loading scorecard…"</p>
                }>
                    {move || match detail_res.get() {
                        None => view! { <p class="text-sm text-on-surface-variant">"Loading…"</p> }.into_any(),
                        Some(Err(e)) => view! {
                            <p class="text-sm text-red-400">{format!("Error: {e}")}</p>
                        }.into_any(),
                        Some(Ok(detail)) => view! { <ScorecardSummary detail=detail /> }.into_any(),
                    }}
                </Suspense>

                <a href="/billing/scorecards" class="inline-block text-sm text-primary hover:underline">
                    "← Back to Scorecards"
                </a>
            </div>
        </div>
    }
}

#[component]
fn ScorecardSummary(detail: ScorecardDetail) -> impl IntoView {
    let s = detail.scorecard;
    let aggs = detail.dimension_aggregates;
    let score = s.composite_score.clone().unwrap_or_else(|| "—".into());

    view! {
        <div class="space-y-4">
            <div class="grid grid-cols-2 md:grid-cols-4 gap-3 text-xs">
                <div>
                    <div class="text-on-surface-variant uppercase tracking-wider text-[10px] font-bold">"Composite"</div>
                    <div class="text-lg font-bold font-mono text-on-surface">{score}</div>
                </div>
                <div>
                    <div class="text-on-surface-variant uppercase tracking-wider text-[10px] font-bold">"Confidence"</div>
                    <div class="text-lg font-bold text-on-surface">{s.confidence_level.clone()}</div>
                </div>
                <div>
                    <div class="text-on-surface-variant uppercase tracking-wider text-[10px] font-bold">"Subject"</div>
                    <div class="font-mono text-on-surface">{format!("{} / {}", s.subject_entity_type, s.subject_entity_id)}</div>
                </div>
                <div>
                    <div class="text-on-surface-variant uppercase tracking-wider text-[10px] font-bold">"Entries"</div>
                    <div class="text-on-surface">{format!("{} sessions · {} entries · {} contributors", s.total_sessions, s.total_entries, s.total_contributors)}</div>
                </div>
            </div>

            <div class="border-t border-outline-variant/20 pt-4">
                <h3 class="text-xs font-bold uppercase tracking-wider text-on-surface-variant mb-3">
                    {format!("Dimension aggregates ({})", aggs.len())}
                </h3>
                <div class="space-y-2">
                    {aggs.into_iter().map(|a| {
                        let label = a.display_value.clone()
                            .or(a.benchmark_label.clone())
                            .or(a.weighted_mean_score.clone())
                            .unwrap_or_else(|| "—".into());
                        view! {
                            <div class="flex justify-between text-xs border-b border-outline-variant/10 pb-2">
                                <span class="font-mono text-on-surface-variant">{a.dimension_id.to_string()}</span>
                                <span class="text-on-surface font-semibold">{label}</span>
                            </div>
                        }
                    }).collect_view()}
                </div>
            </div>
        </div>
    }
}
