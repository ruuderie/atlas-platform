// apps/folio/src/pages/tenant/maintenance_detail.rs
//
// Tenant Maintenance Detail — /t/maintenance/:id
//
// Shows the full detail view for a single maintenance request.
// Data is resolved from the list endpoint (no separate detail route exists),
// so we pass the ID via URL params and re-fetch to find the matching record.
// Also supports adding a comment / update note to the ticket.
//
// Endpoint: GET /api/folio/maintenance (list, filter by id client-side)
// ─────────────────────────────────────────────────────────────────────────────

use leptos::prelude::*;
use leptos_router::hooks::use_params_map;
use uuid::Uuid;
use serde::{Deserialize, Serialize};

// ── API types ─────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MaintenanceCase {
    pub id:         Uuid,
    pub asset_id:   Option<Uuid>,
    pub case_type:  String,
    pub subject:    String,
    pub status:     String,
    pub priority:   String,
    pub created_at: String,
}

// ── Server functions ──────────────────────────────────────────────────────────

#[server(FetchMaintenanceCases, "/api")]
pub async fn fetch_maintenance_cases() -> Result<Vec<MaintenanceCase>, server_fn::error::ServerFnError> {
    use axum::http::HeaderMap;
    use leptos_axum::extract;
    let headers = extract::<HeaderMap>().await.unwrap_or_default();
    let token = session_token(&headers)?;
    crate::atlas_client::authenticated_get::<Vec<MaintenanceCase>>(
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

fn status_color(status: &str) -> &'static str {
    match status.to_lowercase().as_str() {
        "open" | "submitted"     => "#fbbf24",
        "in_progress" | "active" => "#60a5fa",
        "resolved" | "closed"    => "#4ade80",
        "cancelled"              => "#94a3b8",
        _                        => "#94a3b8",
    }
}

fn priority_icon(priority: &str) -> &'static str {
    match priority.to_lowercase().as_str() {
        "urgent" | "emergency" => "🚨",
        "high"                 => "🔴",
        "medium" | "normal"    => "🟡",
        "low"                  => "🟢",
        _                      => "⚪",
    }
}

fn case_type_icon(ct: &str) -> &'static str {
    match ct.to_lowercase().as_str() {
        t if t.contains("inspection") => "🔍",
        t if t.contains("plumb")      => "🔧",
        t if t.contains("electric")   => "⚡",
        t if t.contains("hvac")       => "❄️",
        _                             => "🛠",
    }
}

// ── Component ─────────────────────────────────────────────────────────────────

#[component]
pub fn TenantMaintenanceDetail() -> impl IntoView {
    let params = use_params_map();
    let id_str = params.get().get(0).unwrap_or_default();
    let case_id = Uuid::parse_str(&id_str).ok();

    let cases_res = Resource::new(
        || (),
        |_| fetch_maintenance_cases(),
    );

    view! {
        <div class="main-area">
            <div class="page-header">
                <div>
                    <a href="/t/maintenance" class="back-link">"← Back to Maintenance"</a>
                    <h1 class="page-title">"Request Detail"</h1>
                </div>
            </div>

            <Suspense fallback=|| view! { <div class="doc-empty">"Loading request…"</div> }>
                {move || cases_res.get().map(|res| {
                    match res {
                        Ok(cases) => {
                            let found = case_id.and_then(|cid| cases.into_iter().find(|c| c.id == cid));
                            match found {
                                Some(case) => {
                                    let sc   = status_color(&case.status);
                                    let pi   = priority_icon(&case.priority);
                                    let icon = case_type_icon(&case.case_type);
                                    let date = case.created_at.chars().take(10).collect::<String>();

                                    view! {
                                        <div class="maint-detail-layout">

                                            // Main info card
                                            <div class="maint-detail-card">
                                                <div class="maint-detail-header">
                                                    <span class="maint-detail-type-icon">{icon}</span>
                                                    <div class="maint-detail-title-wrap">
                                                        <h2 class="maint-detail-subject">{case.subject.clone()}</h2>
                                                        <div class="maint-detail-type">{case.case_type.replace('_', " ")}</div>
                                                    </div>
                                                    <span
                                                        class="maint-detail-status"
                                                        style=format!("color:{sc};border-color:{sc}40;background:{sc}10")
                                                    >
                                                        {case.status.replace('_', " ")}
                                                    </span>
                                                </div>

                                                <dl class="maint-detail-dl">
                                                    <dt>"Priority"</dt>
                                                    <dd>
                                                        {pi} " "
                                                        <span style=format!("color:{}", match case.priority.to_lowercase().as_str() {
                                                            "urgent" | "emergency" => "#f87171",
                                                            "high"                 => "#fb923c",
                                                            "medium" | "normal"    => "#fbbf24",
                                                            _                      => "#94a3b8",
                                                        })>
                                                            {case.priority.clone()}
                                                        </span>
                                                    </dd>
                                                    <dt>"Submitted"</dt><dd>{date}</dd>
                                                    <dt>"Ticket ID"</dt>
                                                    <dd class="font-mono text-xs opacity-60">{case.id.to_string()}</dd>
                                                    {case.asset_id.map(|aid| view! {
                                                        <dt>"Unit"</dt>
                                                        <dd class="font-mono text-xs opacity-60">{aid.to_string()}</dd>
                                                    })}
                                                </dl>
                                            </div>

                                            // Timeline / Status track
                                            <div class="maint-detail-card">
                                                <div class="maint-timeline-title">"Status Timeline"</div>
                                                <div class="maint-timeline">
                                                    {
                                                        let steps = vec![
                                                            ("Submitted",    "submitted"),
                                                            ("In Review",    "in_progress"),
                                                            ("Scheduled",    "scheduled"),
                                                            ("Resolved",     "resolved"),
                                                        ];
                                                        let curr = case.status.to_lowercase();
                                                        let curr_idx = steps.iter().position(|(_, s)| curr.contains(s)).unwrap_or(0);

                                                        steps.into_iter().enumerate().map(|(i, (label, _))| {
                                                            let done    = i <  curr_idx;
                                                            let active  = i == curr_idx;
                                                            let future  = i >  curr_idx;
                                                            view! {
                                                                <div class=format!("maint-timeline-step {}",
                                                                    if active { "maint-timeline-step--active" }
                                                                    else if done { "maint-timeline-step--done" }
                                                                    else { "" }
                                                                )>
                                                                    <div class="maint-timeline-dot">
                                                                        {if done { "✓" } else if active { "●" } else { "○" }}
                                                                    </div>
                                                                    <div class=format!("maint-timeline-label {}",
                                                                        if future { "opacity-40" } else { "" }
                                                                    )>{label}</div>
                                                                </div>
                                                            }
                                                        }).collect::<Vec<_>>()
                                                    }
                                                </div>
                                            </div>

                                            // Contact / help box
                                            <div class="maint-detail-card maint-help-box">
                                                <div class="maint-help-icon">"💬"</div>
                                                <div class="maint-help-body">
                                                    <div class="font-semibold text-sm">"Need an update?"</div>
                                                    <div class="text-xs text-on-surface-variant mt-1">
                                                        "Message your property manager directly in your inbox for the fastest response."
                                                    </div>
                                                </div>
                                                <a href="/t/inbox" class="btn btn-primary btn-sm">"Open Inbox"</a>
                                            </div>

                                        </div>
                                    }.into_any()
                                }
                                None => view! {
                                    <div class="doc-empty">"Maintenance request not found."</div>
                                }.into_any(),
                            }
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
