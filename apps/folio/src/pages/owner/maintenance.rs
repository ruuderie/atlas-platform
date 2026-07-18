// apps/folio/src/pages/owner/maintenance.rs
//
// Owner Maintenance Approval — /o/maintenance
//
// Read-only view of all open maintenance cases across the owner's portfolio.
// Owners cannot create or edit cases — they can only view status and
// optionally send a note via the communications channel (future).
// Data from /api/folio/owner/maintenance + /api/folio/owner/inspections.
// ─────────────────────────────────────────────────────────────────────────────

use leptos::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// ── API types ─────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OwnerMaintenanceSummary {
    pub case_id: Uuid,
    pub asset_id: Option<Uuid>,
    pub subject: String,
    pub priority: String,
    pub status: String,
    pub created_at: String,
    pub completed_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OwnerInspectionEntry {
    pub case_id: Uuid,
    pub asset_id: Option<Uuid>,
    pub subject: String,
    pub status: String,
    pub scheduled_at: Option<String>,
    pub completed_at: Option<String>,
}

// ── Server functions ──────────────────────────────────────────────────────────

#[server(FetchOwnerMaintenanceCases, "/api")]
pub async fn fetch_owner_maintenance_cases(
) -> Result<Vec<OwnerMaintenanceSummary>, server_fn::error::ServerFnError> {
    use axum::http::HeaderMap;
    use leptos_axum::extract;
    let headers = extract::<HeaderMap>().await.unwrap_or_default();
    let token = session_token(&headers)?;
    crate::atlas_client::authenticated_get::<Vec<OwnerMaintenanceSummary>>(
        "/api/folio/owner/maintenance",
        &token,
        None,
    )
    .await
    .map_err(|e| server_fn::error::ServerFnError::new(e.to_string()))
}

#[server(FetchOwnerInspections, "/api")]
pub async fn fetch_owner_inspections(
) -> Result<Vec<OwnerInspectionEntry>, server_fn::error::ServerFnError> {
    use axum::http::HeaderMap;
    use leptos_axum::extract;
    let headers = extract::<HeaderMap>().await.unwrap_or_default();
    let token = session_token(&headers)?;
    crate::atlas_client::authenticated_get::<Vec<OwnerInspectionEntry>>(
        "/api/folio/owner/inspections",
        &token,
        None,
    )
    .await
    .map_err(|e| server_fn::error::ServerFnError::new(e.to_string()))
}

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

// ── Helpers ───────────────────────────────────────────────────────────────────

fn priority_color(p: &str) -> &'static str {
    match p.to_lowercase().as_str() {
        "urgent" | "emergency" => "#f87171",
        "high" => "#fb923c",
        "medium" => "#fbbf24",
        _ => "#94a3b8",
    }
}

fn priority_icon(p: &str) -> &'static str {
    match p.to_lowercase().as_str() {
        "urgent" | "emergency" => "🔴",
        "high" => "🟠",
        "medium" => "🟡",
        _ => "⚪",
    }
}

fn status_cls(s: &str) -> &'static str {
    match s.to_lowercase().as_str() {
        "resolved" | "completed" | "closed" => "owner-chip--green",
        "open" | "submitted" | "acknowledged" => "owner-chip--amber",
        "escalated" => "owner-chip--red",
        _ => "owner-chip--grey",
    }
}

// ── Component ─────────────────────────────────────────────────────────────────

#[component]
pub fn OwnerMaintenanceApproval() -> impl IntoView {
    let refresh = RwSignal::new(0u32);
    let tab = RwSignal::new("maintenance"); // "maintenance" | "inspections"

    let maint_res = Resource::new(move || refresh.get(), |_| fetch_owner_maintenance_cases());
    let insp_res = Resource::new(move || refresh.get(), |_| fetch_owner_inspections());

    view! {
        <div class="main-area">
            <div class="page-header">
                <div>
                    <h1 class="page-title">"Maintenance & Inspections"</h1>
                    <p class="page-subtitle">"Open maintenance and inspections"</p>
                </div>
                <div class="page-actions">
                    <span class="owner-readonly-badge">"👁 Read Only"</span>
                    <button class="btn btn-ghost btn-sm" on:click=move |_| refresh.update(|n| *n += 1)>"↻ Refresh"</button>
                </div>
            </div>

            // ── Tab bar ──
            <div class="owner-tabs">
                <button
                    class=move || format!("owner-tab {}", if tab.get() == "maintenance" { "owner-tab--active" } else { "" })
                    on:click=move |_| tab.set("maintenance")
                >"🔧 Maintenance"</button>
                <button
                    class=move || format!("owner-tab {}", if tab.get() == "inspections" { "owner-tab--active" } else { "" })
                    on:click=move |_| tab.set("inspections")
                >"🔍 Inspections"</button>
            </div>

            // ── Maintenance cases ──
            <Show when=move || tab.get() == "maintenance">
                <Suspense fallback=|| view! { <div class="doc-empty">"Loading cases…"</div> }>
                    {move || maint_res.get().map(|res| {
                        match res {
                            Ok(cases) if !cases.is_empty() => {
                                let open_count    = cases.iter().filter(|c| c.status.to_lowercase() != "resolved" && c.status.to_lowercase() != "closed").count();
                                view! {
                                    <div class="owner-cases-header">
                                        <span class="owner-cases-count">
                                            {open_count.to_string()} " open of " {cases.len().to_string()} " total"
                                        </span>
                                    </div>
                                    <div class="viol-list">
                                        <For
                                            each=move || cases.clone()
                                            key=|c| c.case_id
                                            children=move |case| {
                                                let icon  = priority_icon(&case.priority);
                                                let color = priority_color(&case.priority);
                                                let sc    = status_cls(&case.status);
                                                let date  = case.created_at.chars().take(10).collect::<String>();
                                                let done  = case.completed_at.as_ref().map(|d| d.chars().take(10).collect::<String>());
                                                view! {
                                                    <div class="viol-card">
                                                        <div class="viol-card-icon">{icon}</div>
                                                        <div class="viol-card-body">
                                                            <div class="viol-card-subject">{case.subject.clone()}</div>
                                                            <div class="viol-card-meta" style=format!("color:{color}")>
                                                                {case.priority.clone()} " priority"
                                                            </div>
                                                            <div class="viol-card-meta">"Filed " {date}
                                                                {done.map(|d| format!(" · Resolved {d}"))}
                                                            </div>
                                                        </div>
                                                        <span class=format!("owner-chip {sc}")>{case.status.clone()}</span>
                                                    </div>
                                                }
                                            }
                                        />
                                    </div>
                                }.into_any()
                            }
                            Ok(_) => view! { <div class="doc-empty">"No maintenance cases. ✓"</div> }.into_any(),
                            Err(e) => view! { <div class="doc-empty text-red-400">"Error: " {e.to_string()}</div> }.into_any(),
                        }
                    })}
                </Suspense>
            </Show>

            // ── Inspections ──
            <Show when=move || tab.get() == "inspections">
                <Suspense fallback=|| view! { <div class="doc-empty">"Loading inspections…"</div> }>
                    {move || insp_res.get().map(|res| {
                        match res {
                            Ok(inspections) if !inspections.is_empty() => view! {
                                <div class="viol-list">
                                    <For
                                        each=move || inspections.clone()
                                        key=|i| i.case_id
                                        children=move |insp| {
                                            let sc   = status_cls(&insp.status);
                                            let sched= insp.scheduled_at.as_ref().map(|d| d.chars().take(10).collect::<String>());
                                            let done = insp.completed_at.as_ref().map(|d| d.chars().take(10).collect::<String>());
                                            view! {
                                                <div class="viol-card">
                                                    <div class="viol-card-icon">"🔍"</div>
                                                    <div class="viol-card-body">
                                                        <div class="viol-card-subject">{insp.subject.clone()}</div>
                                                        <div class="viol-card-meta">
                                                            {sched.map(|d| format!("Scheduled {d}"))}
                                                            {done.map(|d| format!(" · Completed {d}"))}
                                                        </div>
                                                    </div>
                                                    <span class=format!("owner-chip {sc}")>{insp.status.clone()}</span>
                                                </div>
                                            }
                                        }
                                    />
                                </div>
                            }.into_any(),
                            Ok(_) => view! { <div class="doc-empty">"No scheduled inspections."</div> }.into_any(),
                            Err(e) => view! { <div class="doc-empty text-red-400">"Error: " {e.to_string()}</div> }.into_any(),
                        }
                    })}
                </Suspense>
            </Show>

        </div>
    }
}
