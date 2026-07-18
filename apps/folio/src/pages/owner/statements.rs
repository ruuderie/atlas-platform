// apps/folio/src/pages/owner/statements.rs
//
// Owner Statements — /o/statements
//
// Monthly financial statement summaries for the owner.
// Derived from /api/folio/owner/leases and /api/folio/owner/summary.
// In Phase 7 a dedicated /api/folio/owner/statements endpoint will provide
// pre-generated PDFs; this view gives a data-driven monthly breakdown now.
// ─────────────────────────────────────────────────────────────────────────────

use leptos::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// ── API types ─────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OwnerPortfolioSummary {
    pub owner_user_id: Uuid,
    pub total_properties: usize,
    pub occupied_units: usize,
    pub vacant_units: usize,
    pub occupancy_pct: f64,
    pub revenue_this_month_cents: i64,
    pub revenue_ytd_cents: i64,
    pub outstanding_balance_cents: i64,
    pub outstanding_payments: usize,
    pub on_time_payment_rate_pct: f64,
    pub active_leases: usize,
    pub open_maintenance_cases: usize,
    pub open_violations: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OwnerLeaseEntry {
    pub contract_id: Uuid,
    pub asset_id: Option<Uuid>,
    pub start_date: String,
    pub end_date: Option<String>,
    pub status: String,
    pub monthly_rent_cents: Option<i64>,
    pub currency: String,
}

// ── Server functions ──────────────────────────────────────────────────────────

#[server(FetchOwnerSummaryForStatements, "/api")]
pub async fn fetch_owner_summary_for_statements(
) -> Result<OwnerPortfolioSummary, server_fn::error::ServerFnError> {
    use axum::http::HeaderMap;
    use leptos_axum::extract;
    let headers = extract::<HeaderMap>().await.unwrap_or_default();
    let token = session_token(&headers)?;
    crate::atlas_client::authenticated_get::<OwnerPortfolioSummary>(
        "/api/folio/owner/summary",
        &token,
        None,
    )
    .await
    .map_err(|e| server_fn::error::ServerFnError::new(e.to_string()))
}

#[server(FetchOwnerLeasesForStatements, "/api")]
pub async fn fetch_owner_leases_for_statements(
) -> Result<Vec<OwnerLeaseEntry>, server_fn::error::ServerFnError> {
    use axum::http::HeaderMap;
    use leptos_axum::extract;
    let headers = extract::<HeaderMap>().await.unwrap_or_default();
    let token = session_token(&headers)?;
    crate::atlas_client::authenticated_get::<Vec<OwnerLeaseEntry>>(
        "/api/folio/owner/leases",
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

fn fmt_usd(cents: i64) -> String {
    format!("${:.2}", cents as f64 / 100.0)
}

// ── Component ─────────────────────────────────────────────────────────────────

#[component]
pub fn OwnerStatements() -> impl IntoView {
    let summary_res = Resource::new(|| (), |_| fetch_owner_summary_for_statements());
    let leases_res = Resource::new(|| (), |_| fetch_owner_leases_for_statements());

    view! {
        <div class="main-area">
            <div class="page-header">
                <div>
                    <h1 class="page-title">"Owner Statements"</h1>
                    <p class="page-subtitle">"Monthly statements"</p>
                </div>
                <div class="page-actions">
                    <span class="owner-readonly-badge">"👁 Read Only"</span>
                    <button class="btn btn-ghost btn-sm" disabled=true title="PDF statements — Phase 7">
                        "⬇ Download PDF"
                    </button>
                </div>
            </div>

            // ── Summary KPIs ──
            <Suspense fallback=|| ()>
                {move || summary_res.get().map(|res| {
                    match res {
                        Ok(s) => view! {
                            <div class="kpi-row" style="margin-bottom:1.5rem;">
                                <div class="kpi-card">
                                    <span class="kpi-label">"Revenue This Month"</span>
                                    <span class="kpi-value" style="color:var(--green)">{fmt_usd(s.revenue_this_month_cents)}</span>
                                </div>
                                <div class="kpi-card">
                                    <span class="kpi-label">"Revenue YTD"</span>
                                    <span class="kpi-value" style="color:var(--green)">{fmt_usd(s.revenue_ytd_cents)}</span>
                                </div>
                                <div class="kpi-card">
                                    <span class="kpi-label">"Outstanding"</span>
                                    <span class="kpi-value" style="color:var(--amber)">{fmt_usd(s.outstanding_balance_cents)}</span>
                                </div>
                                <div class="kpi-card">
                                    <span class="kpi-label">"On-Time Rate"</span>
                                    <span class="kpi-value" style="color:var(--cobalt)">{format!("{:.0}%", s.on_time_payment_rate_pct)}</span>
                                </div>
                                <div class="kpi-card">
                                    <span class="kpi-label">"Occupancy"</span>
                                    <span class="kpi-value" style="color:var(--cobalt)">{format!("{:.0}%", s.occupancy_pct)}</span>
                                </div>
                            </div>
                        }.into_any(),
                        Err(_) => ().into_any(),
                    }
                })}
            </Suspense>

            // ── Lease income table ──
            <div class="owner-section">
                <div class="owner-section-title">"Lease Income Schedule"</div>
                <Suspense fallback=|| view! { <div class="doc-empty">"Loading leases…"</div> }>
                    {move || leases_res.get().map(|res| {
                        match res {
                            Ok(leases) if !leases.is_empty() => {
                                let active: Vec<_> = leases.into_iter().filter(|l| l.status.to_lowercase() == "active").collect();
                                if active.is_empty() {
                                    return view! { <div class="doc-empty">"No active leases."</div> }.into_any();
                                }
                                let total_monthly: i64 = active.iter().filter_map(|l| l.monthly_rent_cents).sum();
                                view! {
                                    <div class="owner-table-wrap">
                                        <table class="ph-table">
                                            <thead><tr>
                                                <th>"Lease ID"</th>
                                                <th>"Start"</th><th>"End"</th>
                                                <th>"Currency"</th><th>"Monthly Rent"</th>
                                            </tr></thead>
                                            <tbody>
                                                <For each=move || active.clone() key=|l| l.contract_id children=move |l| {
                                                    let rent = l.monthly_rent_cents.map(|r| fmt_usd(r)).unwrap_or_else(|| "—".to_string());
                                                    let id_short = format!("…{}", &l.contract_id.to_string()[24..]);
                                                    view! {
                                                        <tr class="ph-row">
                                                            <td class="ph-date font-mono text-xs opacity-60">{id_short}</td>
                                                            <td class="ph-date">{l.start_date.chars().take(10).collect::<String>()}</td>
                                                            <td class="ph-date">{l.end_date.map(|d| d.chars().take(10).collect::<String>()).unwrap_or_else(|| "Open".to_string())}</td>
                                                            <td class="ph-date">{l.currency}</td>
                                                            <td class="ph-amount" style="color:var(--green)">{rent}</td>
                                                        </tr>
                                                    }
                                                }/>
                                                <tr class="ph-row" style="border-top:2px solid rgba(255,255,255,0.1);">
                                                    <td colspan="4" style="font-weight:700;font-size:0.8rem;">"Total Monthly"</td>
                                                    <td class="ph-amount" style="color:var(--green);font-weight:800;">{fmt_usd(total_monthly)}</td>
                                                </tr>
                                            </tbody>
                                        </table>
                                    </div>
                                }.into_any()
                            }
                            _ => view! { <div class="doc-empty">"No lease data."</div> }.into_any(),
                        }
                    })}
                </Suspense>
            </div>

            // ── Phase 7 note ──
            <div class="viol-info-banner">
                <span class="viol-info-icon">"ℹ️"</span>
                <p class="viol-info-text">"PDF statement generation (monthly reports, tax summaries) will be available in Phase 7. Contact your property manager to request a manual statement."</p>
            </div>
        </div>
    }
}
