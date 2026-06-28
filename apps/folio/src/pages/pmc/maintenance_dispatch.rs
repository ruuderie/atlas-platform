// apps/folio/src/pages/pmc/maintenance_dispatch.rs
//
// PMC Maintenance Dispatch — /pmc/maintenance
//
// Aggregate maintenance queue across all PMC client portfolios.
// PMs can view, triage, and dispatch vendors across all managed accounts.
// Uses /api/folio/maintenance (landlord scope, includes all managed assets).
// ─────────────────────────────────────────────────────────────────────────────

use leptos::prelude::*;
use uuid::Uuid;
use serde::{Deserialize, Serialize};

// ── API types ─────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MaintenanceSummary {
    pub id:              Uuid,
    pub subject:         String,
    pub category:        Option<String>,
    pub priority:        String,
    pub status:          String,
    pub asset_address:   Option<String>,
    pub vendor_name:     Option<String>,
    pub created_at:      String,
    pub updated_at:      String,
}

// ── Server functions ──────────────────────────────────────────────────────────

#[server(FetchPmcMaintenance, "/api")]
pub async fn fetch_pmc_maintenance() -> Result<Vec<MaintenanceSummary>, server_fn::error::ServerFnError> {
    use axum::http::HeaderMap;
    use leptos_axum::extract;
    let headers = extract::<HeaderMap>().await.unwrap_or_default();
    let token = session_token(&headers)?;
    crate::atlas_client::authenticated_get::<Vec<MaintenanceSummary>>(
        "/api/folio/maintenance", &token, None,
    ).await.map_err(|e| server_fn::error::ServerFnError::new(e.to_string()))
}

#[cfg(feature = "ssr")]
fn session_token(headers: &axum::http::HeaderMap) -> Result<String, server_fn::error::ServerFnError> {
    headers.get("cookie")
        .and_then(|v| v.to_str().ok())
        .and_then(|s| s.split(';').find_map(|p| {
            let p = p.trim();
            p.strip_prefix("session=").map(|t| t.to_string())
        }))
        .ok_or_else(|| server_fn::error::ServerFnError::new("No session token"))
}

// ── Helpers ───────────────────────────────────────────────────────────────────

fn priority_badge(p: &str) -> (&'static str, &'static str) {
    match p.to_lowercase().as_str() {
        "urgent" | "emergency" => ("maint-priority--urgent", "🔴 Urgent"),
        "high"                 => ("maint-priority--high",   "🟠 High"),
        "medium"               => ("maint-priority--medium", "🟡 Medium"),
        _                      => ("maint-priority--low",    "⚪ Low"),
    }
}

fn status_cls(s: &str) -> &'static str {
    match s.to_lowercase().as_str() {
        "open" | "submitted"   => "ph-badge--pending",
        "assigned"             => "ph-badge--pending",
        "in_progress"          => "ph-badge--pending",
        "resolved" | "closed"  => "ph-badge--paid",
        _                      => "ph-badge--default",
    }
}

// ── Component ─────────────────────────────────────────────────────────────────

#[component]
pub fn PmcMaintenanceDispatch() -> impl IntoView {
    let refresh      = RwSignal::new(0u32);
    let status_filter= RwSignal::new("open".to_string());
    let prio_filter  = RwSignal::new("all".to_string());

    let queue_res = Resource::new(
        move || refresh.get(),
        |_| fetch_pmc_maintenance(),
    );

    view! {
        <div class="main-area">
            <div class="page-header">
                <div>
                    <h1 class="page-title">"Maintenance Dispatch"</h1>
                    <p class="page-subtitle">"All open maintenance requests across managed portfolios"</p>
                </div>
                <div class="page-actions">
                    <button class="btn btn-ghost btn-sm" on:click=move |_| refresh.update(|n| *n += 1)>"↻ Refresh"</button>
                </div>
            </div>

            // ── Filter controls ──
            <div class="mkt-controls" style="margin-bottom:1rem;">
                <div class="mkt-filters">
                    {
                        let status_pill = move |v: &'static str, label: &'static str| view! {
                            <button
                                class=move || format!("filter-pill {}", if status_filter.get() == v { "filter-pill--active" } else { "" })
                                on:click=move |_| status_filter.set(v.to_string())
                            >{label}</button>
                        };
                        view! {
                            {status_pill("all",      "All")}
                            {status_pill("open",     "Open")}
                            {status_pill("assigned", "Assigned")}
                            {status_pill("resolved", "Resolved")}
                        }
                    }
                </div>
                <div class="mkt-filters">
                    {
                        let prio_pill = move |v: &'static str, label: &'static str| view! {
                            <button
                                class=move || format!("filter-pill {}", if prio_filter.get() == v { "filter-pill--active" } else { "" })
                                on:click=move |_| prio_filter.set(v.to_string())
                            >{label}</button>
                        };
                        view! {
                            {prio_pill("all",     "All Priorities")}
                            {prio_pill("urgent",  "🔴 Urgent")}
                            {prio_pill("high",    "🟠 High")}
                            {prio_pill("medium",  "🟡 Medium")}
                        }
                    }
                </div>
            </div>

            // ── Queue ──
            <Suspense fallback=|| view! { <div class="doc-empty">"Loading queue…"</div> }>
                {move || queue_res.get().map(|res| {
                    match res {
                        Ok(cases) => {
                            let sf = status_filter.get();
                            let pf = prio_filter.get();
                            let filtered: Vec<_> = cases.into_iter().filter(|c| {
                                let s_match = sf == "all"
                                    || (sf == "open"     && (c.status == "open" || c.status == "submitted"))
                                    || (sf == "assigned" && c.status == "assigned")
                                    || (sf == "resolved" && (c.status == "resolved" || c.status == "closed"));
                                let p_match = pf == "all" || c.priority.to_lowercase().contains(&pf);
                                s_match && p_match
                            }).collect();

                            if filtered.is_empty() {
                                return view! { <div class="doc-empty">"No cases match filters. ✓"</div> }.into_any();
                            }

                            view! {
                                <div class="pmc-dispatch-count">
                                    {filtered.len().to_string()} " cases"
                                </div>
                                <div class="pmc-queue-list">
                                    <For
                                        each=move || filtered.clone()
                                        key=|c| c.id
                                        children=move |case| {
                                            let (prio_cls, prio_label) = priority_badge(&case.priority);
                                            let sc    = status_cls(&case.status);
                                            let addr  = case.asset_address.clone().unwrap_or_else(|| "Unknown address".to_string());
                                            let vend  = case.vendor_name.clone();
                                            let date  = case.created_at.chars().take(10).collect::<String>();
                                            let cat   = case.category.clone().unwrap_or_else(|| "General".to_string()).replace('_', " ");
                                            view! {
                                                <div class="pmc-queue-card">
                                                    <div class="pmc-queue-card-left">
                                                        <span class=format!("maint-priority-badge {prio_cls}")>{prio_label}</span>
                                                        <div class="pmc-queue-subject">{case.subject.clone()}</div>
                                                        <div class="pmc-queue-meta">{cat} " · " {addr}</div>
                                                        <div class="pmc-queue-meta">"Filed " {date}
                                                            {vend.map(|v| view! { " · Vendor: " <strong>{v}</strong> })}
                                                        </div>
                                                    </div>
                                                    <div class="pmc-queue-card-right">
                                                        <span class=format!("ph-badge {sc}")>{case.status.replace('_', " ")}</span>
                                                        <a href="/l/vendors" class="btn btn-ghost btn-sm">"Assign Vendor"</a>
                                                    </div>
                                                </div>
                                            }
                                        }
                                    />
                                </div>
                            }.into_any()
                        }
                        Err(e) => view! {
                            <div class="doc-empty text-red-400">"Error: " {e.to_string()}</div>
                        }.into_any(),
                    }
                })}
            </Suspense>
        </div>
    }
}
