// apps/folio/src/pages/tenant/application_status.rs
//
// Tenant Application Status — /t/application
//
// Shows the tenant's current rental application status(es).
// Applications are listed from /api/folio/applications.
// Each card shows the unit applied for, screening status, decision, and timeline.
// ─────────────────────────────────────────────────────────────────────────────

use leptos::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// ── API types ─────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApplicationSummary {
    pub id: Uuid,
    pub applicant_user_id: Uuid,
    pub target_asset_id: Option<Uuid>,
    pub status: String,
    pub screening_status: String,
    pub screening_provider: Option<String>,
    pub screening_passed: Option<bool>,
    pub monthly_income_cents: Option<i64>,
    pub submitted_at: Option<String>,
    pub decided_at: Option<String>,
    pub decision_reason: Option<String>,
    pub created_at: String,
}

// ── Server functions ──────────────────────────────────────────────────────────

#[server(FetchMyApplications, "/api")]
pub async fn fetch_my_applications(
) -> Result<Vec<ApplicationSummary>, server_fn::error::ServerFnError> {
    use axum::http::HeaderMap;
    use leptos_axum::extract;
    let headers = extract::<HeaderMap>().await.unwrap_or_default();
    let token = session_token(&headers)?;
    crate::atlas_client::authenticated_get::<Vec<ApplicationSummary>>(
        "/api/folio/applications",
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

fn app_status_info(status: &str) -> (&'static str, &'static str, &'static str) {
    // (icon, css_class, label)
    match status.to_lowercase().as_str() {
        "pending" | "submitted" => ("⏳", "app-status--pending", "Under Review"),
        "approved" => ("✅", "app-status--approved", "Approved"),
        "denied" | "rejected" => ("❌", "app-status--denied", "Not Approved"),
        "withdrawn" | "cancelled" => ("🚪", "app-status--neutral", "Withdrawn"),
        _ => ("📋", "app-status--neutral", "Unknown"),
    }
}

fn screening_info(status: &str, passed: Option<bool>) -> (&'static str, &'static str) {
    match (status.to_lowercase().as_str(), passed) {
        ("pending" | "in_progress", _) => ("🔍 In Progress", "color:#60a5fa"),
        ("complete", Some(true)) => ("✓ Passed", "color:#4ade80"),
        ("complete", Some(false)) => ("✗ Did Not Pass", "color:#f87171"),
        ("waived", _) => ("— Waived", "color:#94a3b8"),
        _ => ("Pending", "color:#94a3b8"),
    }
}

fn cents_display(cents: i64) -> String {
    format!("${:.0}/mo", cents as f64 / 100.0)
}

// ── Component ─────────────────────────────────────────────────────────────────

#[component]
pub fn TenantApplicationStatus() -> impl IntoView {
    let refresh = RwSignal::new(0u32);

    let apps_res = Resource::new(move || refresh.get(), |_| fetch_my_applications());

    view! {
        <div class="main-area">

            <div class="page-header">
                <div>
                    <h1 class="page-title">"Rental Applications"</h1>
                    <p class="page-subtitle">"Track your application status and screening results"</p>
                </div>
                <div class="page-actions">
                    <button class="btn btn-ghost btn-sm" on:click=move |_| refresh.update(|n| *n += 1)>
                        "↻ Refresh"
                    </button>
                </div>
            </div>

            <Suspense fallback=|| view! { <div class="doc-empty">"Loading applications…"</div> }>
                {move || apps_res.get().map(|res| {
                    match res {
                        Ok(apps) if !apps.is_empty() => view! {
                            <div class="app-list">
                                <For
                                    each=move || apps.clone()
                                    key=|a| a.id
                                    children=move |app| {
                                        let (icon, sc, label) = app_status_info(&app.status);
                                        let sc = sc.to_string();
                                        let label = label.to_string();
                                        let (scr_text, scr_style) = screening_info(&app.screening_status, app.screening_passed);
                                        let scr_style = scr_style.to_string();
                                        let submitted = app.submitted_at.clone().unwrap_or_else(|| app.created_at.chars().take(10).collect());
                                        let decided   = app.decided_at.clone();
                                        let provider  = app.screening_provider.clone().unwrap_or_else(|| "—".to_string());
                                        let reason    = app.decision_reason.clone();
                                        let income    = app.monthly_income_cents.map(cents_display);
                                        let app_id    = app.id.to_string();

                                        view! {
                                            <div class="app-card">

                                                // Header row
                                                <div class="app-card-header">
                                                    <div class="app-card-title">
                                                        <span class="app-icon">"🏠"</span>
                                                        <div>
                                                            <div class="app-card-label">"Rental Application"</div>
                                                            <div class="font-mono text-xs opacity-40">{app_id}</div>
                                                        </div>
                                                    </div>
                                                    <div class=format!("app-status-badge {sc}")>
                                                        {icon} " " {label}
                                                    </div>
                                                </div>

                                                // Detail grid
                                                <div class="app-detail-grid">
                                                    <div class="app-detail-item">
                                                        <div class="app-detail-label">"Submitted"</div>
                                                        <div class="app-detail-value">{submitted}</div>
                                                    </div>
                                                    {decided.map(|d| view! {
                                                        <div class="app-detail-item">
                                                            <div class="app-detail-label">"Decision Date"</div>
                                                            <div class="app-detail-value">{d.chars().take(10).collect::<String>()}</div>
                                                        </div>
                                                    })}
                                                    <div class="app-detail-item">
                                                        <div class="app-detail-label">"Background Check"</div>
                                                        <div class="app-detail-value" style=scr_style>{scr_text}</div>
                                                    </div>
                                                    <div class="app-detail-item">
                                                        <div class="app-detail-label">"Screening Provider"</div>
                                                        <div class="app-detail-value">{provider}</div>
                                                    </div>
                                                    {income.map(|i| view! {
                                                        <div class="app-detail-item">
                                                            <div class="app-detail-label">"Stated Income"</div>
                                                            <div class="app-detail-value">{i}</div>
                                                        </div>
                                                    })}
                                                </div>

                                                // Decision reason (denials only)
                                                {reason.map(|r| view! {
                                                    <div class="app-decision-reason">
                                                        <div class="app-decision-label">"Decision Reason"</div>
                                                        <div class="app-decision-text">{r}</div>
                                                    </div>
                                                })}

                                                // FHA notice (always shown)
                                                <div class="app-fha-notice">
                                                    "🏛 This application was evaluated in compliance with Fair Housing Act (FHA) guidelines. "
                                                    "Protected characteristics are never used in screening decisions."
                                                </div>

                                            </div>
                                        }
                                    }
                                />
                            </div>
                        }.into_any(),
                        Ok(_) => view! {
                            <div class="app-empty-state">
                                <div class="app-empty-icon">"📋"</div>
                                <div class="app-empty-title">"No Applications on File"</div>
                                <div class="app-empty-sub">
                                    "When you apply for a unit, your application status will appear here."
                                </div>
                            </div>
                        }.into_any(),
                        Err(e) => view! {
                            <div class="doc-empty text-red-400">"Error: " {e.to_string()}</div>
                        }.into_any(),
                    }
                })}
            </Suspense>

        </div>
    }
}
