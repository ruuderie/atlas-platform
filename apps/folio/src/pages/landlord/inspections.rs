// apps/folio/src/pages/landlord/inspections.rs
//
// Inspections page — /l/inspections
//
// Portfolio-wide proactive inspection queue. Data source:
//   GET  /api/folio/inspections              → [InspectionDetail]  (upcoming, status=scheduled)
//   POST /api/folio/inspections              → schedule new inspection
//   POST /api/folio/inspections/{id}/complete → complete + record findings
//
// ─── Panels ───────────────────────────────────────────────────────────────────
//  KPI strip   : Scheduled | Overdue | Completed | Avg cost
//  Filter bar  : search subject · status chips (All / Scheduled / Overdue / Completed)
//  Table       : date | subject | asset | completed | status | cost | actions
//  Schedule modal   : form to POST /api/folio/inspections
//  Complete modal   : form to POST /api/folio/inspections/{id}/complete
// ─────────────────────────────────────────────────────────────────────────────

use leptos::prelude::*;
use uuid::Uuid;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

// ── API types (mirrors InspectionDetail from maintenance.rs handler) ──────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InspectionDetail {
    pub id:                    Uuid,
    pub asset_id:              Option<Uuid>,
    pub subject:               String,
    pub status:                String,            // "scheduled" | "completed" | "cancelled"
    pub scheduled_at:          Option<DateTime<Utc>>,
    pub completed_at:          Option<DateTime<Utc>>,
    pub service_provider_id:   Option<Uuid>,
    pub assigned_user_id:      Option<Uuid>,
    pub estimated_cost_cents:  Option<i64>,
    pub actual_cost_cents:     Option<i64>,
    pub metadata:              Option<serde_json::Value>,
    pub created_at:            DateTime<Utc>,
}

// ── Local enums ───────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq)]
enum InspStatus {
    Scheduled,
    Overdue,
    Completed,
    Cancelled,
}

impl InspStatus {
    fn from_insp(insp: &InspectionDetail) -> Self {
        match insp.status.as_str() {
            "completed"  => InspStatus::Completed,
            "cancelled"  => InspStatus::Cancelled,
            _ => {
                if let Some(dt) = &insp.scheduled_at {
                    if *dt < Utc::now() { return InspStatus::Overdue; }
                }
                InspStatus::Scheduled
            }
        }
    }

    fn label(self) -> &'static str {
        match self {
            InspStatus::Scheduled  => "Scheduled",
            InspStatus::Overdue    => "Overdue",
            InspStatus::Completed  => "Completed",
            InspStatus::Cancelled  => "Cancelled",
        }
    }

    fn css_class(self) -> &'static str {
        match self {
            InspStatus::Scheduled  => "insp-badge insp-badge--scheduled",
            InspStatus::Overdue    => "insp-badge insp-badge--overdue",
            InspStatus::Completed  => "insp-badge insp-badge--completed",
            InspStatus::Cancelled  => "insp-badge insp-badge--cancelled",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum StatusFilter {
    All,
    Scheduled,
    Overdue,
    Completed,
}

impl StatusFilter {
    fn label(self) -> &'static str {
        match self {
            StatusFilter::All       => "All",
            StatusFilter::Scheduled => "Scheduled",
            StatusFilter::Overdue   => "Overdue",
            StatusFilter::Completed => "Completed",
        }
    }

    fn matches(self, s: InspStatus) -> bool {
        match self {
            StatusFilter::All       => true,
            StatusFilter::Scheduled => s == InspStatus::Scheduled,
            StatusFilter::Overdue   => s == InspStatus::Overdue,
            StatusFilter::Completed => s == InspStatus::Completed,
        }
    }
}

// ── Helpers ───────────────────────────────────────────────────────────────────

fn fmt_date(dt: Option<&DateTime<Utc>>) -> String {
    match dt {
        Some(d) => d.format("%b %d, %Y").to_string(),
        None    => "—".to_string(),
    }
}

fn fmt_cost(cents: Option<i64>) -> String {
    match cents {
        Some(c) => format!("${:.0}", c as f64 / 100.0),
        None    => "—".to_string(),
    }
}

fn days_label(dt: Option<&DateTime<Utc>>) -> String {
    match dt {
        None    => "—".to_string(),
        Some(d) => {
            let diff = (*d - Utc::now()).num_days();
            if diff < 0       { format!("{} days ago", diff.abs()) }
            else if diff == 0 { "Today".to_string() }
            else              { format!("in {} days", diff) }
        }
    }
}

fn asset_label(insp: &InspectionDetail) -> String {
    if let Some(meta) = &insp.metadata {
        if let Some(n) = meta.get("asset_name").and_then(|v| v.as_str()) { return n.to_string(); }
        if let Some(k) = meta.get("asset_type").and_then(|v| v.as_str()) { return k.to_string(); }
    }
    insp.asset_id
        .map(|id| format!("…{}", &id.to_string()[32..]))
        .unwrap_or_else(|| "—".to_string())
}

// ── Token helper (SSR only) ───────────────────────────────────────────────────

#[cfg(feature = "ssr")]
fn extract_token(headers: &axum::http::HeaderMap) -> Option<String> {
    headers.get("cookie")
        .and_then(|v| v.to_str().ok())
        .and_then(|s| {
            s.split(';').find_map(|p| {
                let p = p.trim();
                p.strip_prefix("session=").map(|t| t.to_string())
            })
        })
}

// ── Server functions ──────────────────────────────────────────────────────────

#[server(FetchInspections, "/api")]
pub async fn fetch_inspections() -> Result<Vec<InspectionDetail>, server_fn::error::ServerFnError> {
    use axum::http::HeaderMap;
    use leptos_axum::extract;
    let headers = extract::<HeaderMap>().await.unwrap_or_default();
    let token = extract_token(&headers)
        .ok_or_else(|| server_fn::error::ServerFnError::new("No session token"))?;
    crate::atlas_client::authenticated_get::<Vec<InspectionDetail>>(
        "/api/folio/inspections",
        &token,
        None,
    )
    .await
    .map_err(|e| server_fn::error::ServerFnError::new(format!("Fetch inspections failed: {e}")))
}

#[server(ScheduleInspection, "/api")]
pub async fn schedule_inspection(
    asset_id:             String,
    subject:              String,
    notes:                Option<String>,
    scheduled_at:         String,
    estimated_cost_cents: Option<i64>,
) -> Result<(), server_fn::error::ServerFnError> {
    use axum::http::HeaderMap;
    use leptos_axum::extract;
    let headers = extract::<HeaderMap>().await.unwrap_or_default();
    let token = extract_token(&headers)
        .ok_or_else(|| server_fn::error::ServerFnError::new("No session token"))?;
    let body = serde_json::json!({
        "asset_id":             asset_id,
        "subject":              subject,
        "notes":                notes,
        "scheduled_at":         scheduled_at,
        "estimated_cost_cents": estimated_cost_cents,
    });
    crate::atlas_client::authenticated_post::<_, serde_json::Value>(
        "/api/folio/inspections", &token, None, &body,
    )
    .await
    .map(|_| ())
    .map_err(|e| server_fn::error::ServerFnError::new(format!("Schedule inspection failed: {e}")))
}

#[server(CompleteInspection, "/api")]
pub async fn complete_inspection(
    case_id:              String,
    findings:             String,
    condition_after:      Option<String>,
    next_inspection_date: Option<String>,
    actual_cost_cents:    Option<i64>,
) -> Result<(), server_fn::error::ServerFnError> {
    use axum::http::HeaderMap;
    use leptos_axum::extract;
    let headers = extract::<HeaderMap>().await.unwrap_or_default();
    let token = extract_token(&headers)
        .ok_or_else(|| server_fn::error::ServerFnError::new("No session token"))?;
    let url = format!("/api/folio/inspections/{}/complete", case_id);
    let body = serde_json::json!({
        "case_id":               case_id,
        "findings":              findings,
        "condition_after":       condition_after,
        "next_inspection_date":  next_inspection_date,
        "actual_cost_cents":     actual_cost_cents,
        "attachment_r2_keys":    [],
    });
    crate::atlas_client::authenticated_post::<_, serde_json::Value>(
        &url, &token, None, &body,
    )
    .await
    .map(|_| ())
    .map_err(|e| server_fn::error::ServerFnError::new(format!("Complete inspection failed: {e}")))
}

// ── KPI strip ─────────────────────────────────────────────────────────────────

#[component]
fn InspKpiStrip(inspections: Vec<InspectionDetail>) -> impl IntoView {
    let scheduled  = inspections.iter().filter(|i| InspStatus::from_insp(i) == InspStatus::Scheduled).count();
    let overdue    = inspections.iter().filter(|i| InspStatus::from_insp(i) == InspStatus::Overdue).count();
    let completed  = inspections.iter().filter(|i| i.status == "completed").count();
    let costs: Vec<i64> = inspections.iter()
        .filter_map(|i| i.actual_cost_cents.or(i.estimated_cost_cents))
        .collect();
    let avg_cost_str = if costs.is_empty() { "—".to_string() }
    else { format!("${:.0}", costs.iter().sum::<i64>() as f64 / costs.len() as f64 / 100.0) };

    view! {
        <div class="insp-kpi-strip">
            <div class="insp-kpi-card">
                <span class="insp-kpi-icon material-symbols-outlined">"event_available"</span>
                <div class="insp-kpi-body">
                    <span class="insp-kpi-value">{scheduled}</span>
                    <span class="insp-kpi-label">"Scheduled"</span>
                </div>
            </div>
            <div class="insp-kpi-card insp-kpi-card--overdue">
                <span class="insp-kpi-icon material-symbols-outlined">"event_busy"</span>
                <div class="insp-kpi-body">
                    <span class="insp-kpi-value">{overdue}</span>
                    <span class="insp-kpi-label">"Overdue"</span>
                </div>
            </div>
            <div class="insp-kpi-card insp-kpi-card--completed">
                <span class="insp-kpi-icon material-symbols-outlined">"task_alt"</span>
                <div class="insp-kpi-body">
                    <span class="insp-kpi-value">{completed}</span>
                    <span class="insp-kpi-label">"Completed"</span>
                </div>
            </div>
            <div class="insp-kpi-card">
                <span class="insp-kpi-icon material-symbols-outlined">"payments"</span>
                <div class="insp-kpi-body">
                    <span class="insp-kpi-value">{avg_cost_str}</span>
                    <span class="insp-kpi-label">"Avg Cost"</span>
                </div>
            </div>
        </div>
    }
}

// ── Schedule modal ────────────────────────────────────────────────────────────

#[component]
fn ScheduleModal(
    show:           RwSignal<bool>,
    refetch_count:  RwSignal<u32>,
) -> impl IntoView {
    let subject    = RwSignal::new(String::new());
    let asset_id   = RwSignal::new(String::new());
    let sched_at   = RwSignal::new(String::new());
    let est_cost   = RwSignal::new(String::new());
    let notes      = RwSignal::new(String::new());
    let submitting = RwSignal::new(false);
    let error      = RwSignal::new(Option::<String>::None);

    let close = move || { show.set(false); };

    let submit = move |_| {
        let subj = subject.get();
        let aid  = asset_id.get();
        let sat  = sched_at.get();
        if subj.trim().is_empty() || aid.trim().is_empty() || sat.trim().is_empty() {
            error.set(Some("Subject, Asset ID, and Date are required.".into()));
            return;
        }
        submitting.set(true);
        error.set(None);
        let cost: Option<i64> = est_cost.get().trim().parse::<f64>().ok().map(|v| (v * 100.0) as i64);
        let n = notes.get();
        let notes_opt = if n.trim().is_empty() { None } else { Some(n) };
        leptos::task::spawn_local(async move {
            match schedule_inspection(aid, subj, notes_opt, format!("{}T09:00:00Z", sat), cost).await {
                Ok(_)  => {
                    refetch_count.update(|c| *c += 1);
                    show.set(false);
                }
                Err(e) => { error.set(Some(e.to_string())); }
            }
            submitting.set(false);
        });
    };

    view! {
        <Show when=move || show.get()>
            <div class="insp-modal-backdrop" on:click=move |_| close()>
                <div class="insp-modal" on:click=|e| e.stop_propagation()>
                    <div class="insp-modal-header">
                        <h3>"Schedule Inspection"</h3>
                        <button class="insp-modal-close" on:click=move |_| close()>
                            <span class="material-symbols-outlined">"close"</span>
                        </button>
                    </div>
                    <div class="insp-modal-body">
                        <label class="insp-form-label">"Subject *"</label>
                        <input class="insp-form-input" placeholder="e.g. Annual boiler inspection"
                            prop:value=move || subject.get()
                            on:input=move |e| subject.set(event_target_value(&e)) />

                        <label class="insp-form-label">"Asset ID *"</label>
                        <input class="insp-form-input" placeholder="UUID of asset"
                            prop:value=move || asset_id.get()
                            on:input=move |e| asset_id.set(event_target_value(&e)) />

                        <label class="insp-form-label">"Scheduled Date *"</label>
                        <input type="date" class="insp-form-input"
                            prop:value=move || sched_at.get()
                            on:input=move |e| sched_at.set(event_target_value(&e)) />

                        <label class="insp-form-label">"Estimated Cost ($)"</label>
                        <input type="number" class="insp-form-input" placeholder="0.00"
                            prop:value=move || est_cost.get()
                            on:input=move |e| est_cost.set(event_target_value(&e)) />

                        <label class="insp-form-label">"Notes"</label>
                        <textarea class="insp-form-textarea" placeholder="Inspector instructions…"
                            prop:value=move || notes.get()
                            on:input=move |e| notes.set(event_target_value(&e)) />

                        {move || error.get().map(|e| view! { <p class="insp-form-error">{e}</p> })}
                    </div>
                    <div class="insp-modal-footer">
                        <button class="insp-btn insp-btn--ghost" on:click=move |_| close()>
                            "Cancel"
                        </button>
                        <button class="insp-btn insp-btn--primary" on:click=submit
                            disabled=move || submitting.get()>
                            {move || if submitting.get() { "Scheduling…" } else { "Schedule" }}
                        </button>
                    </div>
                </div>
            </div>
        </Show>
    }
}

// ── Complete modal ────────────────────────────────────────────────────────────

#[component]
fn CompleteModal(
    show:          RwSignal<bool>,
    case_id:       RwSignal<Option<Uuid>>,
    refetch_count: RwSignal<u32>,
) -> impl IntoView {
    let findings    = RwSignal::new(String::new());
    let condition   = RwSignal::new(String::new());
    let next_date   = RwSignal::new(String::new());
    let actual_cost = RwSignal::new(String::new());
    let submitting  = RwSignal::new(false);
    let error       = RwSignal::new(Option::<String>::None);

    let close = move || { show.set(false); case_id.set(None); };

    let submit = move |_| {
        let cid = match case_id.get() { Some(id) => id.to_string(), None => return };
        let f   = findings.get();
        if f.trim().is_empty() {
            error.set(Some("Findings are required.".into()));
            return;
        }
        submitting.set(true);
        error.set(None);
        let cost: Option<i64> = actual_cost.get().trim().parse::<f64>().ok().map(|v| (v * 100.0) as i64);
        let next = { let n = next_date.get(); if n.trim().is_empty() { None } else { Some(n) } };
        let cond = { let c = condition.get(); if c.trim().is_empty() { None } else { Some(c) } };
        leptos::task::spawn_local(async move {
            match complete_inspection(cid, f, cond, next, cost).await {
                Ok(_)  => {
                    refetch_count.update(|c| *c += 1);
                    show.set(false);
                    case_id.set(None);
                }
                Err(e) => { error.set(Some(e.to_string())); }
            }
            submitting.set(false);
        });
    };

    view! {
        <Show when=move || show.get()>
            <div class="insp-modal-backdrop" on:click=move |_| close()>
                <div class="insp-modal" on:click=|e| e.stop_propagation()>
                    <div class="insp-modal-header">
                        <h3>"Mark Inspection Complete"</h3>
                        <button class="insp-modal-close" on:click=move |_| close()>
                            <span class="material-symbols-outlined">"close"</span>
                        </button>
                    </div>
                    <div class="insp-modal-body">
                        <label class="insp-form-label">"Findings *"</label>
                        <textarea class="insp-form-textarea"
                            placeholder="Describe what was inspected and any issues found…"
                            prop:value=move || findings.get()
                            on:input=move |e| findings.set(event_target_value(&e)) />

                        <label class="insp-form-label">"Asset Condition After"</label>
                        <input class="insp-form-input" placeholder="e.g. Good, Fair, Poor"
                            prop:value=move || condition.get()
                            on:input=move |e| condition.set(event_target_value(&e)) />

                        <label class="insp-form-label">"Next Inspection Date"</label>
                        <input type="date" class="insp-form-input"
                            prop:value=move || next_date.get()
                            on:input=move |e| next_date.set(event_target_value(&e)) />

                        <label class="insp-form-label">"Actual Cost ($)"</label>
                        <input type="number" class="insp-form-input" placeholder="0.00"
                            prop:value=move || actual_cost.get()
                            on:input=move |e| actual_cost.set(event_target_value(&e)) />

                        {move || error.get().map(|e| view! { <p class="insp-form-error">{e}</p> })}
                    </div>
                    <div class="insp-modal-footer">
                        <button class="insp-btn insp-btn--ghost" on:click=move |_| close()>
                            "Cancel"
                        </button>
                        <button class="insp-btn insp-btn--primary" on:click=submit
                            disabled=move || submitting.get()>
                            {move || if submitting.get() { "Saving…" } else { "Complete" }}
                        </button>
                    </div>
                </div>
            </div>
        </Show>
    }
}

// ── Table row ─────────────────────────────────────────────────────────────────

#[component]
fn InspTableRow(
    insp:        InspectionDetail,
    on_complete: impl Fn(Uuid) + 'static,
) -> impl IntoView {
    let status      = InspStatus::from_insp(&insp);
    let al          = asset_label(&insp);
    let dl          = days_label(insp.scheduled_at.as_ref());
    let cost_disp   = insp.actual_cost_cents.or(insp.estimated_cost_cents);
    let id_copy     = insp.id;
    let can_complete = insp.status == "scheduled";
    let sched_date  = fmt_date(insp.scheduled_at.as_ref());
    let done_date   = fmt_date(insp.completed_at.as_ref());

    view! {
        <tr class="insp-table-row">
            <td class="insp-table-cell">
                <span class="insp-date-primary">{sched_date}</span>
                <span class="insp-date-sub">{dl}</span>
            </td>
            <td class="insp-table-cell">
                <span class="insp-subject">{insp.subject.clone()}</span>
            </td>
            <td class="insp-table-cell insp-td-asset">
                <span class="insp-asset-label">{al}</span>
            </td>
            <td class="insp-table-cell">
                <span class="insp-date-primary">{done_date}</span>
            </td>
            <td class="insp-table-cell">
                <span class=status.css_class()>{status.label()}</span>
            </td>
            <td class="insp-table-cell insp-td-cost">
                {fmt_cost(cost_disp)}
            </td>
            <td class="insp-table-cell insp-td-actions">
                {if can_complete {
                    view! {
                        <button class="insp-action-btn insp-action-btn--complete"
                            title="Mark complete"
                            on:click=move |_| on_complete(id_copy)>
                            <span class="material-symbols-outlined">"task_alt"</span>
                        </button>
                    }.into_any()
                } else {
                    view! {
                        <span class="insp-action-done material-symbols-outlined">"check_circle"</span>
                    }.into_any()
                }}
            </td>
        </tr>
    }
}

// ── Skeleton ──────────────────────────────────────────────────────────────────

#[component]
fn InspTableSkeleton() -> impl IntoView {
    view! {
        <div class="insp-skeleton-wrap">
            {(0..6).map(|_| view! {
                <div class="insp-skeleton-row">
                    <div class="insp-skel insp-skel--date"></div>
                    <div class="insp-skel insp-skel--subject"></div>
                    <div class="insp-skel insp-skel--asset"></div>
                    <div class="insp-skel insp-skel--date"></div>
                    <div class="insp-skel insp-skel--badge"></div>
                    <div class="insp-skel insp-skel--cost"></div>
                    <div class="insp-skel insp-skel--action"></div>
                </div>
            }).collect::<Vec<_>>()}
        </div>
    }
}

// ── Main page component ───────────────────────────────────────────────────────

#[component]
pub fn Inspections() -> impl IntoView {
    let refetch_count  = RwSignal::new(0u32);
    let inspections    = Resource::new(move || refetch_count.get(), |_| fetch_inspections());
    let search         = RwSignal::new(String::new());
    let status_filter  = RwSignal::new(StatusFilter::All);
    let show_schedule  = RwSignal::new(false);
    let show_complete  = RwSignal::new(false);
    let complete_id    = RwSignal::new(Option::<Uuid>::None);

    let open_complete = move |id: Uuid| {
        complete_id.set(Some(id));
        show_complete.set(true);
    };

    view! {
        <div class="insp-page">
            // Header
            <div class="insp-header">
                <div class="insp-header-left">
                    <h1 class="insp-title">"Inspections"</h1>
                    <p class="insp-subtitle">"Proactive inspection schedule across your portfolio"</p>
                </div>
                <button class="insp-btn insp-btn--primary"
                    on:click=move |_| show_schedule.set(true)>
                    <span class="material-symbols-outlined">"add"</span>
                    "Schedule Inspection"
                </button>
            </div>

            // KPI strip
            <Suspense fallback=|| view! {
                <div class="insp-kpi-strip insp-kpi-strip--loading">
                    {(0..4).map(|_| view! { <div class="insp-kpi-skel"></div> }).collect::<Vec<_>>()}
                </div>
            }>
                {move || inspections.get().map(|res| match res {
                    Ok(data)  => view! { <InspKpiStrip inspections=data /> }.into_any(),
                    Err(_)    => view! { <div></div> }.into_any(),
                })}
            </Suspense>

            // Filter bar
            <div class="insp-filter-bar">
                <div class="insp-search-wrap">
                    <span class="material-symbols-outlined insp-search-icon">"search"</span>
                    <input class="insp-search-input" placeholder="Search by subject…"
                        prop:value=move || search.get()
                        on:input=move |e| search.set(event_target_value(&e)) />
                </div>
                <div class="insp-chips">
                    {[StatusFilter::All, StatusFilter::Scheduled, StatusFilter::Overdue, StatusFilter::Completed]
                        .iter()
                        .map(|&f| view! {
                            <button
                                class=move || if status_filter.get() == f { "insp-chip insp-chip--active" } else { "insp-chip" }
                                on:click=move |_| status_filter.set(f)>
                                {f.label()}
                            </button>
                        })
                        .collect::<Vec<_>>()}
                </div>
            </div>

            // Table
            <div class="insp-table-wrap">
                <Suspense fallback=|| view! { <InspTableSkeleton /> }>
                    {move || inspections.get().map(|res| match res {
                        Err(e) => view! {
                            <div class="insp-error">
                                <span class="material-symbols-outlined">"error"</span>
                                <p>"Failed to load inspections: " {e.to_string()}</p>
                                <button class="insp-btn insp-btn--ghost"
                                    on:click=move |_| inspections.refetch()>"Retry"</button>
                            </div>
                        }.into_any(),
                        Ok(data) => {
                            let q  = search.get().to_lowercase();
                            let sf = status_filter.get();
                            let filtered: Vec<InspectionDetail> = data.into_iter()
                                .filter(|i| {
                                    let st = InspStatus::from_insp(i);
                                    sf.matches(st) && (q.is_empty() || i.subject.to_lowercase().contains(&q))
                                })
                                .collect();

                            if filtered.is_empty() {
                                view! {
                                    <div class="insp-empty">
                                        <span class="material-symbols-outlined insp-empty-icon">"event_available"</span>
                                        <p class="insp-empty-title">"No inspections found"</p>
                                        <p class="insp-empty-sub">"Schedule a proactive inspection to get started."</p>
                                        <button class="insp-btn insp-btn--primary"
                                            on:click=move |_| show_schedule.set(true)>
                                            "Schedule Inspection"
                                        </button>
                                    </div>
                                }.into_any()
                            } else {
                                view! {
                                    <table class="insp-table">
                                        <thead>
                                            <tr>
                                                <th class="insp-th">"Scheduled"</th>
                                                <th class="insp-th">"Subject"</th>
                                                <th class="insp-th">"Asset"</th>
                                                <th class="insp-th">"Completed"</th>
                                                <th class="insp-th">"Status"</th>
                                                <th class="insp-th insp-th--right">"Cost"</th>
                                                <th class="insp-th insp-th--center">"Actions"</th>
                                            </tr>
                                        </thead>
                                        <tbody>
                                            {filtered.into_iter().map(|insp| {
                                                let oc = open_complete.clone();
                                                view! {
                                                    <InspTableRow
                                                        insp=insp
                                                        on_complete=move |id| oc(id)
                                                    />
                                                }
                                            }).collect::<Vec<_>>()}
                                        </tbody>
                                    </table>
                                }.into_any()
                            }
                        }
                    })}
                </Suspense>
            </div>

            // Modals
            <ScheduleModal
                show=show_schedule
                refetch_count=refetch_count
            />
            <CompleteModal
                show=show_complete
                case_id=complete_id
                refetch_count=refetch_count
            />
        </div>
    }
}
