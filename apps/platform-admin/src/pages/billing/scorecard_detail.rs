//! Scorecard entity detail — Overview / Dimensions / Time Series / Sessions (G-27).

use crate::api::admin::get_tenant_stats;
use crate::api::scorecards::{
    get_scorecard, list_dimensions, list_entries, list_sessions, list_time_series, open_session,
    recompute, submit_entry, verify_scorecard_entry, DimensionAggregate, OpenSessionInput,
    RatingSession, ScorecardDetail, ScorecardEntry, SubmitEntryInput, TimeSeriesPoint,
};
use crate::pages::billing::scorecard_panel::to_session_dimension;
use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos_router::hooks::use_params_map;
use shared_ui::components::scorecard::{ScoreSubmission, ScorecardWidget, SessionDimension};
use std::collections::HashMap;
use uuid::Uuid;

#[component]
pub fn ScorecardDetailPage() -> impl IntoView {
    let params = use_params_map();
    let scorecard_id = move || params.with(|p| p.get("scorecard_id").unwrap_or_default());
    let selected_tenant_id = RwSignal::new(String::new());
    let refresh = RwSignal::new(0u32);
    let active_tab = RwSignal::new("overview".to_string());
    let busy = RwSignal::new(false);
    let error: RwSignal<Option<String>> = RwSignal::new(None);
    let rating: RwSignal<Option<(Uuid, Uuid, Vec<SessionDimension>)>> = RwSignal::new(None);

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
        let _ = refresh.get();
        async move {
            if tid.is_empty() || sid.is_empty() {
                return Err("Missing tenant or scorecard id".to_string());
            }
            let detail = get_scorecard(&tid, &sid).await?;
            let dims = list_dimensions(&tid, &detail.scorecard.template_id.to_string())
                .await
                .unwrap_or_default();
            Ok((detail, dims))
        }
    });

    view! {
        <div class="w-full" style="display:flex;flex-direction:column;gap:16px;">
            <div class="card" style="padding:18px 20px;">
                <div style="display:flex;flex-wrap:wrap;align-items:flex-start;justify-content:space-between;gap:12px;margin-bottom:14px;">
                    <div>
                        <h1 style="font-size:20px;font-weight:700;color:var(--text-primary);margin:0;">"Scorecard"</h1>
                        <p style="font-size:11px;color:var(--text-muted);font-family:monospace;margin:4px 0 0;">
                            {move || format!("scorecard_id: {}", scorecard_id())}
                        </p>
                    </div>
                    <label style="display:flex;align-items:center;gap:8px;font-size:11px;color:var(--text-muted);">
                        <span style="font-weight:600;text-transform:uppercase;letter-spacing:0.04em;">"Tenant"</span>
                        <select
                            style="background:var(--bg-elevated, transparent);border:1px solid var(--border-default);color:var(--text-primary);font-size:13px;border-radius:6px;padding:6px 10px;min-width:200px;"
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

                <Show when=move || error.get().is_some()>
                    <div style="margin-bottom:12px;font-size:12px;color:#f87171;">{move || error.get().unwrap_or_default()}</div>
                </Show>

                <Suspense fallback=move || view! {
                    <p style="font-size:13px;color:var(--text-muted);">"Loading scorecard…"</p>
                }>
                    {move || match detail_res.get() {
                        None => view! { <p style="font-size:13px;color:var(--text-muted);">"Loading…"</p> }.into_any(),
                        Some(Err(e)) => view! {
                            <p style="font-size:13px;color:#f87171;">{format!("Error: {e}")}</p>
                        }.into_any(),
                        Some(Ok((detail, dims))) => {
                            let tid_sv = StoredValue::new(selected_tenant_id.get());
                            let dim_names: HashMap<Uuid, String> = dims
                                .iter()
                                .map(|d| (d.id, d.name.clone()))
                                .collect();
                            let dim_names_sv = StoredValue::new(dim_names);
                            let sc_id = detail.scorecard.id;
                            let template_id = detail.scorecard.template_id;
                            let subject_label = format!(
                                "{} / {}",
                                detail.scorecard.subject_entity_type,
                                detail.scorecard.subject_entity_id
                            );
                            let subject_sv = StoredValue::new(subject_label);

                            view! {
                                <DetailBody
                                    tid_sv=tid_sv
                                    sc_id=sc_id
                                    template_id=template_id
                                    detail=detail
                                    dim_names_sv=dim_names_sv
                                    subject_sv=subject_sv
                                    active_tab=active_tab
                                    rating=rating
                                    busy=busy
                                    error=error
                                    refresh=refresh
                                />
                            }.into_any()
                        }
                    }}
                </Suspense>

                <a
                    href="/billing/scorecards"
                    style="display:inline-block;margin-top:16px;font-size:12px;color:var(--text-link);text-decoration:none;font-weight:600;"
                >
                    "← Back to Scorecards"
                </a>
            </div>
        </div>
    }
}

#[component]
fn DetailBody(
    tid_sv: StoredValue<String>,
    sc_id: Uuid,
    template_id: Uuid,
    detail: ScorecardDetail,
    dim_names_sv: StoredValue<HashMap<Uuid, String>>,
    subject_sv: StoredValue<String>,
    active_tab: RwSignal<String>,
    rating: RwSignal<Option<(Uuid, Uuid, Vec<SessionDimension>)>>,
    busy: RwSignal<bool>,
    error: RwSignal<Option<String>>,
    refresh: RwSignal<u32>,
) -> impl IntoView {
    let detail_sv = StoredValue::new(detail);

    view! {
        <div style="display:flex;flex-direction:column;gap:14px;">
            <Show when=move || rating.get().is_none()>
                <div style="display:flex;flex-direction:column;gap:14px;">
                    <div style="display:flex;gap:8px;flex-wrap:wrap;">
                        <button
                            type="button"
                            disabled=move || busy.get()
                            style=action_btn_style()
                            on:click=move |_| {
                                let tid = tid_sv.get_value();
                                busy.set(true);
                                error.set(None);
                                spawn_local(async move {
                                    let result = start_detail_rating(&tid, sc_id, template_id).await;
                                    busy.set(false);
                                    match result {
                                        Ok(state) => rating.set(Some(state)),
                                        Err(e) => error.set(Some(e)),
                                    }
                                });
                            }
                        >
                            {move || if busy.get() { "Working…" } else { "Rate" }}
                        </button>
                        <button
                            type="button"
                            disabled=move || busy.get()
                            style=action_btn_style()
                            on:click=move |_| {
                                let tid = tid_sv.get_value();
                                busy.set(true);
                                error.set(None);
                                spawn_local(async move {
                                    let result = recompute(&tid, &sc_id.to_string()).await;
                                    busy.set(false);
                                    match result {
                                        Ok(()) => refresh.update(|n| *n = n.wrapping_add(1)),
                                        Err(e) => error.set(Some(e)),
                                    }
                                });
                            }
                        >
                            "Recompute"
                        </button>
                    </div>

                    <div style="display:flex;gap:4px;border-bottom:1px solid var(--border-default);padding-bottom:0;">
                        {tab_btn(active_tab, "overview", "Overview")}
                        {tab_btn(active_tab, "dimensions", "Dimensions")}
                        {tab_btn(active_tab, "timeseries", "Time Series")}
                        {tab_btn(active_tab, "sessions", "Sessions")}
                    </div>

                    <Show when=move || active_tab.get() == "overview">
                        {
                            let d = detail_sv.get_value();
                            let names = dim_names_sv.get_value();
                            view! { <OverviewTab detail=d dim_names=names /> }
                        }
                    </Show>
                    <Show when=move || active_tab.get() == "dimensions">
                        {
                            let d = detail_sv.get_value();
                            let names = dim_names_sv.get_value();
                            view! {
                                <DimensionsTab
                                    aggregates=d.dimension_aggregates
                                    dim_names=names
                                />
                            }
                        }
                    </Show>
                    <Show when=move || active_tab.get() == "timeseries">
                        <TimeSeriesTab
                            tid_sv=tid_sv
                            scorecard_id=sc_id
                            dim_names_sv=dim_names_sv
                            refresh=refresh
                        />
                    </Show>
                    <Show when=move || active_tab.get() == "sessions">
                        <SessionsTab
                            tid_sv=tid_sv
                            scorecard_id=sc_id
                            dim_names_sv=dim_names_sv
                            refresh=refresh
                        />
                    </Show>
                </div>
            </Show>

            <Show when=move || rating.get().is_some()>
                {move || {
                    let Some((session_id, scorecard_id, session_dims)) = rating.get() else {
                        return view! { <div></div> }.into_any();
                    };
                    let subject = subject_sv.get_value();
                    view! {
                        <ScorecardWidget
                            scorecard_id=scorecard_id
                            session_id=session_id
                            subject_label=subject
                            dimensions=session_dims
                            on_submit=Callback::new(move |subs: Vec<ScoreSubmission>| {
                                let tid = tid_sv.get_value();
                                busy.set(true);
                                error.set(None);
                                spawn_local(async move {
                                    let result = submit_detail_ratings(
                                        &tid,
                                        session_id,
                                        scorecard_id,
                                        subs,
                                    )
                                    .await;
                                    busy.set(false);
                                    match result {
                                        Ok(()) => {
                                            rating.set(None);
                                            refresh.update(|n| *n = n.wrapping_add(1));
                                        }
                                        Err(e) => error.set(Some(e)),
                                    }
                                });
                            })
                            on_cancel=Callback::new(move |_| {
                                rating.set(None);
                                error.set(None);
                            })
                        />
                    }.into_any()
                }}
            </Show>
        </div>
    }
}

fn action_btn_style() -> &'static str {
    "font-size:12px;font-weight:600;padding:6px 12px;border-radius:4px;border:1px solid var(--border-default);background:var(--bg-elevated, transparent);color:var(--text-primary);cursor:pointer;"
}

fn tab_btn(active_tab: RwSignal<String>, id: &'static str, label: &'static str) -> impl IntoView {
    view! {
        <button
            type="button"
            style=move || {
                let active = active_tab.get() == id;
                format!(
                    "font-size:12px;font-weight:600;padding:8px 12px;border:none;border-bottom:2px solid {};background:transparent;color:{};cursor:pointer;",
                    if active { "var(--text-link, #60a5fa)" } else { "transparent" },
                    if active { "var(--text-primary)" } else { "var(--text-muted)" },
                )
            }
            on:click=move |_| active_tab.set(id.to_string())
        >
            {label}
        </button>
    }
}

async fn start_detail_rating(
    tenant_id: &str,
    scorecard_id: Uuid,
    template_id: Uuid,
) -> Result<(Uuid, Uuid, Vec<SessionDimension>), String> {
    let session = open_session(
        tenant_id,
        &scorecard_id.to_string(),
        &OpenSessionInput {
            session_type: "meeting".into(),
            occurred_at: None,
            context_entity_type: None,
            context_entity_id: None,
            session_label: Some("Platform admin rating".into()),
        },
    )
    .await?;

    let dims = list_dimensions(tenant_id, &template_id.to_string()).await?;
    let session_dims: Vec<_> = dims
        .into_iter()
        .filter(|d| d.is_active)
        .map(to_session_dimension)
        .collect();

    if session_dims.is_empty() {
        return Err("No active dimensions on this template".into());
    }

    Ok((session.id, scorecard_id, session_dims))
}

async fn submit_detail_ratings(
    tenant_id: &str,
    session_id: Uuid,
    scorecard_id: Uuid,
    submissions: Vec<ScoreSubmission>,
) -> Result<(), String> {
    for sub in submissions {
        let source_type = match sub.source_type.as_str() {
            "manual" | "direct_entry" => Some("manual".into()),
            other if !other.is_empty() => Some(other.to_string()),
            _ => Some("manual".into()),
        };
        submit_entry(
            tenant_id,
            &session_id.to_string(),
            &SubmitEntryInput {
                scorecard_id,
                dimension_id: sub.dimension_id,
                score: sub.score,
                option_id: sub.option_id,
                source_type,
                context: None,
                note: None,
            },
        )
        .await?;
    }
    recompute(tenant_id, &scorecard_id.to_string()).await?;
    Ok(())
}

#[component]
fn OverviewTab(detail: ScorecardDetail, dim_names: HashMap<Uuid, String>) -> impl IntoView {
    let s = detail.scorecard;
    let aggs = detail.dimension_aggregates;
    let score = s.composite_score.clone().unwrap_or_else(|| "—".into());

    view! {
        <div style="display:flex;flex-direction:column;gap:16px;">
            <div style="display:grid;grid-template-columns:repeat(auto-fit,minmax(140px,1fr));gap:12px;">
                <Kpi label="Composite".to_string() value=score mono=true />
                <Kpi label="Confidence".to_string() value=s.confidence_level.clone() mono=false />
                <Kpi
                    label="Subject".to_string()
                    value=format!("{} / {}", s.subject_entity_type, s.subject_entity_id)
                    mono=true
                />
                <Kpi
                    label="Activity".to_string()
                    value=format!(
                        "{} sessions · {} entries · {} contributors",
                        s.total_sessions, s.total_entries, s.total_contributors
                    )
                    mono=false
                />
            </div>

            <div>
                <h3 style="font-size:11px;font-weight:700;text-transform:uppercase;letter-spacing:0.05em;color:var(--text-muted);margin:0 0 10px;">
                    {format!("Dimension aggregates ({})", aggs.len())}
                </h3>
                <div style="display:flex;flex-direction:column;gap:8px;">
                    {aggs.into_iter().map(|a| {
                        let name = dim_names
                            .get(&a.dimension_id)
                            .cloned()
                            .unwrap_or_else(|| a.dimension_id.to_string());
                        let label = a
                            .display_value
                            .clone()
                            .or(a.benchmark_label.clone())
                            .or(a.weighted_mean_score.clone())
                            .or(a.mean_score.clone())
                            .unwrap_or_else(|| "—".into());
                        view! {
                            <div style="display:flex;justify-content:space-between;gap:12px;font-size:12px;padding-bottom:8px;border-bottom:1px solid var(--border-default);">
                                <span style="color:var(--text-secondary);">{name}</span>
                                <span style="color:var(--text-primary);font-weight:600;font-family:monospace;">{label}</span>
                            </div>
                        }
                    }).collect_view()}
                </div>
            </div>
        </div>
    }
}

#[component]
fn Kpi(label: String, value: String, mono: bool) -> impl IntoView {
    let value_style = if mono {
        "font-size:15px;font-weight:700;color:var(--text-primary);font-family:monospace;margin-top:4px;word-break:break-all;"
    } else {
        "font-size:15px;font-weight:700;color:var(--text-primary);margin-top:4px;"
    };
    view! {
        <div>
            <div style="font-size:10px;font-weight:700;text-transform:uppercase;letter-spacing:0.05em;color:var(--text-muted);">
                {label}
            </div>
            <div style=value_style>{value}</div>
        </div>
    }
}

#[component]
fn DimensionsTab(
    aggregates: Vec<DimensionAggregate>,
    dim_names: HashMap<Uuid, String>,
) -> impl IntoView {
    view! {
        <div style="overflow-x:auto;">
            <table style="width:100%;border-collapse:collapse;font-size:12px;">
                <thead>
                    <tr style="text-align:left;color:var(--text-muted);border-bottom:1px solid var(--border-default);">
                        <th style="padding:8px 6px;font-weight:600;">"Dimension"</th>
                        <th style="padding:8px 6px;font-weight:600;">"Mean"</th>
                        <th style="padding:8px 6px;font-weight:600;">"vs Global"</th>
                        <th style="padding:8px 6px;font-weight:600;">"Percentile"</th>
                        <th style="padding:8px 6px;font-weight:600;">"Contributors"</th>
                    </tr>
                </thead>
                <tbody>
                    {aggregates.into_iter().map(|a| {
                        let name = dim_names
                            .get(&a.dimension_id)
                            .cloned()
                            .unwrap_or_else(|| a.dimension_id.to_string());
                        let mean = a
                            .mean_score
                            .clone()
                            .or(a.weighted_mean_score.clone())
                            .unwrap_or_else(|| "—".into());
                        let vs = a
                            .vs_global_label
                            .clone()
                            .or(a.vs_global_delta.clone())
                            .unwrap_or_else(|| "—".into());
                        let pct = a
                            .percentile_rank
                            .clone()
                            .or(a.percentile_band.clone())
                            .unwrap_or_else(|| "—".into());
                        view! {
                            <tr style="border-bottom:1px solid var(--border-default);color:var(--text-primary);">
                                <td style="padding:8px 6px;">{name}</td>
                                <td style="padding:8px 6px;font-family:monospace;">{mean}</td>
                                <td style="padding:8px 6px;">{vs}</td>
                                <td style="padding:8px 6px;font-family:monospace;">{pct}</td>
                                <td style="padding:8px 6px;font-family:monospace;">{a.contributor_count}</td>
                            </tr>
                        }
                    }).collect_view()}
                </tbody>
            </table>
        </div>
    }
}

#[component]
fn TimeSeriesTab(
    tid_sv: StoredValue<String>,
    scorecard_id: Uuid,
    dim_names_sv: StoredValue<HashMap<Uuid, String>>,
    refresh: RwSignal<u32>,
) -> impl IntoView {
    let series_res = LocalResource::new(move || {
        let tid = tid_sv.get_value();
        let _ = refresh.get();
        async move {
            list_time_series(&tid, &scorecard_id.to_string(), None, Some("monthly")).await
        }
    });

    view! {
        <Suspense fallback=move || view! {
            <p style="font-size:12px;color:var(--text-muted);">"Loading time series…"</p>
        }>
            {move || match series_res.get() {
                None => view! { <p style="font-size:12px;color:var(--text-muted);">"Loading…"</p> }.into_any(),
                Some(Err(e)) => view! {
                    <p style="font-size:12px;color:#f87171;">{e}</p>
                }.into_any(),
                Some(Ok(points)) => {
                    let dim_names = dim_names_sv.get_value();
                    view! { <TimeSeriesTable points=points dim_names=dim_names /> }.into_any()
                }
            }}
        </Suspense>
    }
}

#[component]
fn TimeSeriesTable(points: Vec<TimeSeriesPoint>, dim_names: HashMap<Uuid, String>) -> impl IntoView {
    if points.is_empty() {
        return view! {
            <p style="font-size:12px;color:var(--text-muted);">"No time-series points yet."</p>
        }
        .into_any();
    }

    view! {
        <div style="overflow-x:auto;">
            <table style="width:100%;border-collapse:collapse;font-size:12px;">
                <thead>
                    <tr style="text-align:left;color:var(--text-muted);border-bottom:1px solid var(--border-default);">
                        <th style="padding:8px 6px;font-weight:600;">"Period"</th>
                        <th style="padding:8px 6px;font-weight:600;">"Dimension"</th>
                        <th style="padding:8px 6px;font-weight:600;">"Mean"</th>
                        <th style="padding:8px 6px;font-weight:600;">"Trend"</th>
                        <th style="padding:8px 6px;font-weight:600;">"Anomaly"</th>
                    </tr>
                </thead>
                <tbody>
                    {points.into_iter().map(|p| {
                        let name = dim_names
                            .get(&p.dimension_id)
                            .cloned()
                            .unwrap_or_else(|| p.dimension_id.to_string());
                        let mean = p.mean_score.clone().unwrap_or_else(|| "—".into());
                        let trend = p.trend_direction.clone().unwrap_or_else(|| "—".into());
                        let anomaly = if p.is_anomaly {
                            p.anomaly_direction.clone().unwrap_or_else(|| "yes".into())
                        } else {
                            "—".into()
                        };
                        view! {
                            <tr style="border-bottom:1px solid var(--border-default);color:var(--text-primary);">
                                <td style="padding:8px 6px;font-family:monospace;">{p.period_start.to_string()}</td>
                                <td style="padding:8px 6px;">{name}</td>
                                <td style="padding:8px 6px;font-family:monospace;">{mean}</td>
                                <td style="padding:8px 6px;">{trend}</td>
                                <td style="padding:8px 6px;">{anomaly}</td>
                            </tr>
                        }
                    }).collect_view()}
                </tbody>
            </table>
        </div>
    }
    .into_any()
}

#[component]
fn SessionsTab(
    tid_sv: StoredValue<String>,
    scorecard_id: Uuid,
    dim_names_sv: StoredValue<HashMap<Uuid, String>>,
    refresh: RwSignal<u32>,
) -> impl IntoView {
    let expanded = RwSignal::new(Option::<Uuid>::None);
    let entries: RwSignal<Option<Result<Vec<ScorecardEntry>, String>>> = RwSignal::new(None);
    let loading_entries = RwSignal::new(false);

    let sessions_res = LocalResource::new(move || {
        let tid = tid_sv.get_value();
        let _ = refresh.get();
        async move { list_sessions(&tid, &scorecard_id.to_string()).await }
    });

    view! {
        <Suspense fallback=move || view! {
            <p style="font-size:12px;color:var(--text-muted);">"Loading sessions…"</p>
        }>
            {move || match sessions_res.get() {
                None => view! { <p style="font-size:12px;color:var(--text-muted);">"Loading…"</p> }.into_any(),
                Some(Err(e)) => view! {
                    <p style="font-size:12px;color:#f87171;">{e}</p>
                }.into_any(),
                Some(Ok(sessions)) => {
                    if sessions.is_empty() {
                        return view! {
                            <p style="font-size:12px;color:var(--text-muted);">"No rating sessions yet."</p>
                        }
                        .into_any();
                    }
                    view! {
                        <div style="display:flex;flex-direction:column;gap:6px;">
                            {sessions.into_iter().map(|sess| {
                                view! {
                                    <SessionRow
                                        session=sess
                                        expanded=expanded
                                        entries=entries
                                        loading_entries=loading_entries
                                        tid_sv=tid_sv
                                        scorecard_id=scorecard_id
                                        dim_names_sv=dim_names_sv
                                        refresh=refresh
                                    />
                                }
                            }).collect_view()}
                        </div>
                    }
                    .into_any()
                }
            }}
        </Suspense>
    }
}

#[component]
fn SessionRow(
    session: RatingSession,
    expanded: RwSignal<Option<Uuid>>,
    entries: RwSignal<Option<Result<Vec<ScorecardEntry>, String>>>,
    loading_entries: RwSignal<bool>,
    tid_sv: StoredValue<String>,
    scorecard_id: Uuid,
    dim_names_sv: StoredValue<HashMap<Uuid, String>>,
    refresh: RwSignal<u32>,
) -> impl IntoView {
    let session_id = session.id;
    let label = session
        .session_label
        .clone()
        .unwrap_or_else(|| session.session_type.clone());
    let status = session.status.clone();
    let occurred = session.occurred_at.format("%Y-%m-%d %H:%M").to_string();
    let entry_busy: RwSignal<Option<Uuid>> = RwSignal::new(None);

    view! {
        <div style="border:1px solid var(--border-default);border-radius:6px;overflow:hidden;">
            <button
                type="button"
                style="width:100%;display:flex;justify-content:space-between;align-items:center;gap:10px;padding:10px 12px;background:transparent;border:none;color:var(--text-primary);cursor:pointer;text-align:left;font-size:12px;"
                on:click=move |_| {
                    if expanded.get() == Some(session_id) {
                        expanded.set(None);
                        entries.set(None);
                        return;
                    }
                    expanded.set(Some(session_id));
                    entries.set(None);
                    loading_entries.set(true);
                    let tid = tid_sv.get_value();
                    spawn_local(async move {
                        let result = list_entries(&tid, &scorecard_id.to_string(), &session_id.to_string()).await;
                        loading_entries.set(false);
                        entries.set(Some(result));
                    });
                }
            >
                <span style="display:flex;flex-direction:column;gap:2px;min-width:0;">
                    <span style="font-weight:600;">{label}</span>
                    <span style="font-size:11px;color:var(--text-muted);font-family:monospace;">
                        {format!("{occurred} · {status} · {session_id}")}
                    </span>
                </span>
                <span style="color:var(--text-muted);font-size:14px;">
                    {move || if expanded.get() == Some(session_id) { "▾" } else { "▸" }}
                </span>
            </button>

            <Show when=move || expanded.get() == Some(session_id)>
                <div style="padding:0 12px 12px;border-top:1px solid var(--border-default);">
                    <Show when=move || loading_entries.get()>
                        <p style="font-size:11px;color:var(--text-muted);padding-top:10px;">"Loading entries…"</p>
                    </Show>
                    {move || match entries.get() {
                        None => view! { <span></span> }.into_any(),
                        Some(Err(e)) => view! {
                            <p style="font-size:11px;color:#f87171;padding-top:10px;">{e}</p>
                        }.into_any(),
                        Some(Ok(ents)) => {
                            let dim_names = dim_names_sv.get_value();
                            if ents.is_empty() {
                                return view! {
                                    <p style="font-size:11px;color:var(--text-muted);padding-top:10px;">"No entries in this session."</p>
                                }
                                .into_any();
                            }
                            view! {
                                <table style="width:100%;border-collapse:collapse;font-size:11px;margin-top:10px;">
                                    <thead>
                                        <tr style="text-align:left;color:var(--text-muted);">
                                            <th style="padding:4px 6px;font-weight:600;">"Dimension"</th>
                                            <th style="padding:4px 6px;font-weight:600;">"Score"</th>
                                            <th style="padding:4px 6px;font-weight:600;">"Source"</th>
                                            <th style="padding:4px 6px;font-weight:600;">"Verified"</th>
                                            <th style="padding:4px 6px;font-weight:600;">"Actions"</th>
                                        </tr>
                                    </thead>
                                    <tbody>
                                        {ents.into_iter().map(|e| {
                                            let entry_id = e.id;
                                            let name = dim_names
                                                .get(&e.dimension_id)
                                                .cloned()
                                                .unwrap_or_else(|| e.dimension_id.to_string());
                                            let score = e.score.clone().unwrap_or_else(|| "—".into());
                                            let verified = e.is_verified;
                                            let verified_label = if verified { "yes" } else { "no" };
                                            view! {
                                                <tr style="color:var(--text-primary);border-top:1px solid var(--border-default);">
                                                    <td style="padding:5px 6px;">{name}</td>
                                                    <td style="padding:5px 6px;font-family:monospace;">{score}</td>
                                                    <td style="padding:5px 6px;">{e.source_type.clone()}</td>
                                                    <td style="padding:5px 6px;">{verified_label}</td>
                                                    <td style="padding:5px 6px;">
                                                        <Show
                                                            when=move || !verified
                                                            fallback=|| view! {
                                                                <span style="color:var(--text-muted);">"verified"</span>
                                                            }
                                                        >
                                                            <div style="display:flex;gap:6px;flex-wrap:wrap;">
                                                                <button
                                                                    type="button"
                                                                    style="font-size:11px;padding:2px 8px;border-radius:4px;border:1px solid var(--border-default);background:transparent;color:var(--text-primary);cursor:pointer;"
                                                                    disabled=move || entry_busy.get().is_some()
                                                                    on:click=move |_| {
                                                                        if entry_busy.get_untracked().is_some() {
                                                                            return;
                                                                        }
                                                                        entry_busy.set(Some(entry_id));
                                                                        let tid = tid_sv.get_value();
                                                                        spawn_local(async move {
                                                                            let result = verify_scorecard_entry(
                                                                                &entry_id.to_string(),
                                                                                true,
                                                                            )
                                                                            .await;
                                                                            match result {
                                                                                Ok(()) => {
                                                                                    let _ = recompute(
                                                                                        &tid,
                                                                                        &scorecard_id.to_string(),
                                                                                    )
                                                                                    .await;
                                                                                    let reloaded = list_entries(
                                                                                        &tid,
                                                                                        &scorecard_id.to_string(),
                                                                                        &session_id.to_string(),
                                                                                    )
                                                                                    .await;
                                                                                    entries.set(Some(reloaded));
                                                                                    refresh.update(|n| *n = n.wrapping_add(1));
                                                                                }
                                                                                Err(err) => {
                                                                                    entries.set(Some(Err(err)));
                                                                                }
                                                                            }
                                                                            entry_busy.set(None);
                                                                        });
                                                                    }
                                                                >
                                                                    {move || {
                                                                        if entry_busy.get() == Some(entry_id) {
                                                                            "…"
                                                                        } else {
                                                                            "Verify"
                                                                        }
                                                                    }}
                                                                </button>
                                                                <button
                                                                    type="button"
                                                                    style="font-size:11px;padding:2px 8px;border-radius:4px;border:1px solid var(--border-default);background:transparent;color:var(--text-muted);cursor:pointer;"
                                                                    disabled=move || entry_busy.get().is_some()
                                                                    on:click=move |_| {
                                                                        if entry_busy.get_untracked().is_some() {
                                                                            return;
                                                                        }
                                                                        entry_busy.set(Some(entry_id));
                                                                        let tid = tid_sv.get_value();
                                                                        spawn_local(async move {
                                                                            let result = verify_scorecard_entry(
                                                                                &entry_id.to_string(),
                                                                                false,
                                                                            )
                                                                            .await;
                                                                            match result {
                                                                                Ok(()) => {
                                                                                    let reloaded = list_entries(
                                                                                        &tid,
                                                                                        &scorecard_id.to_string(),
                                                                                        &session_id.to_string(),
                                                                                    )
                                                                                    .await;
                                                                                    entries.set(Some(reloaded));
                                                                                }
                                                                                Err(err) => {
                                                                                    entries.set(Some(Err(err)));
                                                                                }
                                                                            }
                                                                            entry_busy.set(None);
                                                                        });
                                                                    }
                                                                >
                                                                    "Reject"
                                                                </button>
                                                            </div>
                                                        </Show>
                                                    </td>
                                                </tr>
                                            }
                                        }).collect_view()}
                                    </tbody>
                                </table>
                            }
                            .into_any()
                        }
                    }}
                </div>
            </Show>
        </div>
    }
}
