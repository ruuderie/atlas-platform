// apps/folio/src/pages/str_host/incidents.rs
//
// STR Incidents / Violations — /s/incidents
//
// View and manage incidents and violations tied to STR bookings.
// Uses /api/folio/cases?case_type=violation (G-13 atlas_cases).
// ─────────────────────────────────────────────────────────────────────────────

use leptos::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// ── API types ─────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CaseSummary {
    pub id: Uuid,
    pub case_type: String,
    pub subject: String,
    pub status: String,
    pub priority: Option<String>,
    pub description: Option<String>,
    pub asset_id: Option<Uuid>,
    pub created_at: String,
    pub resolved_at: Option<String>,
}

// ── Server functions ──────────────────────────────────────────────────────────

#[server(FetchStrIncidents, "/api")]
pub async fn fetch_str_incidents() -> Result<Vec<CaseSummary>, server_fn::error::ServerFnError> {
    use axum::http::HeaderMap;
    use leptos_axum::extract;
    let headers = extract::<HeaderMap>().await.unwrap_or_default();
    let token = session_token(&headers)?;
    crate::atlas_client::authenticated_get::<Vec<CaseSummary>>(
        "/api/folio/cases?case_type=violation",
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

fn status_cls(s: &str) -> &'static str {
    match s.to_lowercase().as_str() {
        "open" | "submitted" => "ph-badge--pending",
        "resolved" | "closed" => "ph-badge--paid",
        "escalated" => "ph-badge--overdue",
        _ => "ph-badge--default",
    }
}

// ── Component ─────────────────────────────────────────────────────────────────

#[component]
pub fn StrIncidents() -> impl IntoView {
    let refresh = RwSignal::new(0u32);

    let res = Resource::new(move || refresh.get(), |_| fetch_str_incidents());

    view! {
        <div class="main-area">
            <div class="page-header">
                <div>
                    <h1 class="page-title">"Incidents & Violations"</h1>
                    <p class="page-subtitle">"Guest-related incidents and compliance violations for your STR units"</p>
                </div>
                <div class="page-actions">
                    <button class="btn btn-ghost btn-sm" on:click=move |_| refresh.update(|n| *n += 1)>"↻ Refresh"</button>
                    <a href="/s/violations/new" class="btn btn-primary btn-sm">"+ File Violation"</a>
                </div>
            </div>

            <Suspense fallback=|| view! { <div class="doc-empty">"Loading incidents…"</div> }>
                {move || res.get().map(|result| {
                    match result {
                        Ok(cases) if !cases.is_empty() => {
                            let open = cases.iter().filter(|c| c.status != "resolved" && c.status != "closed").count();
                            view! {
                                <div class="owner-cases-header" style="margin-bottom:0.75rem;">
                                    <span class="owner-cases-count">{open.to_string()} " open · " {cases.len().to_string()} " total"</span>
                                </div>
                                <div class="viol-list">
                                    <For
                                        each=move || cases.clone()
                                        key=|c| c.id
                                        children=move |case| {
                                            let sc   = status_cls(&case.status);
                                            let date = case.created_at.chars().take(10).collect::<String>();
                                            let desc = case.description.clone().unwrap_or_default();
                                            let prio = case.priority.clone().unwrap_or_else(|| "Normal".to_string());
                                            view! {
                                                <div class="viol-card">
                                                    <div class="viol-card-icon">"⚠️"</div>
                                                    <div class="viol-card-body">
                                                        <div class="viol-card-subject">{case.subject.clone()}</div>
                                                        <div class="viol-card-meta">"Priority: " {prio} " · Filed " {date}</div>
                                                        {if !desc.is_empty() {
                                                            view! { <div class="viol-card-meta" style="font-style:italic;">{desc}</div> }.into_any()
                                                        } else { ().into_any() }}
                                                    </div>
                                                    <span class=format!("ph-badge {sc}")>{case.status.replace('_', " ")}</span>
                                                </div>
                                            }
                                        }
                                    />
                                </div>
                            }.into_any()
                        }
                        Ok(_) => view! {
                            <div class="doc-empty">
                                <div style="font-size:2rem;margin-bottom:0.5rem;">"✓"</div>
                                <div>"No incidents or violations. Keep it up!"</div>
                                <a href="/s/violations/new" class="btn btn-ghost btn-sm" style="margin-top:1rem;">"File a Violation →"</a>
                            </div>
                        }.into_any(),
                        Err(_) => view! {
                            <div class="doc-empty text-on-surface-variant">"No incident data available."</div>
                        }.into_any(),
                    }
                })}
            </Suspense>
        </div>
    }
}
