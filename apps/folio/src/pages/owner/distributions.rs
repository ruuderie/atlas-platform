// apps/folio/src/pages/owner/distributions.rs
//
// Owner Distributions — /o/distributions
//
// Shows net distribution payouts to the owner across their portfolio.
// Revenue from /api/folio/owner/summary. Per-property breakdown from
// /api/folio/owner/properties. Distribution history is derived from
// the ledger (Phase 7 will add /api/folio/owner/distributions).
// ─────────────────────────────────────────────────────────────────────────────

use leptos::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// ── API types ─────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OwnerPortfolioSummary {
    pub owner_user_id: Uuid,
    pub total_properties: usize,
    pub revenue_this_month_cents: i64,
    pub revenue_ytd_cents: i64,
    pub outstanding_balance_cents: i64,
    pub on_time_payment_rate_pct: f64,
    pub occupancy_pct: f64,
    pub active_leases: usize,
    pub open_maintenance_cases: usize,
    pub open_violations: usize,
    pub occupied_units: usize,
    pub vacant_units: usize,
    pub outstanding_payments: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OwnerPropertySummary {
    pub asset_id: Uuid,
    pub asset_name: String,
    pub asset_type: String,
    pub address_line_1: Option<String>,
    pub active_leases: usize,
    pub open_maintenance: usize,
    pub open_violations: usize,
    pub revenue_this_month_cents: i64,
    pub outstanding_balance_cents: i64,
}

// ── Server functions ──────────────────────────────────────────────────────────

#[server(FetchOwnerSummaryDist, "/api")]
pub async fn fetch_owner_summary_dist(
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

#[server(FetchOwnerPropertiesDist, "/api")]
pub async fn fetch_owner_properties_dist(
) -> Result<Vec<OwnerPropertySummary>, server_fn::error::ServerFnError> {
    use axum::http::HeaderMap;
    use leptos_axum::extract;
    let headers = extract::<HeaderMap>().await.unwrap_or_default();
    let token = session_token(&headers)?;
    crate::atlas_client::authenticated_get::<Vec<OwnerPropertySummary>>(
        "/api/folio/owner/properties",
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

// Simulated management fee (PM takes ~8-10%). In Phase 7 this comes from
// the owner's management_fee_pct field on the PM relationship.
const MGMT_FEE_PCT: f64 = 0.09;

fn net_distribution(gross_cents: i64) -> i64 {
    (gross_cents as f64 * (1.0 - MGMT_FEE_PCT)) as i64
}

// ── Component ─────────────────────────────────────────────────────────────────

#[component]
pub fn OwnerDistributions() -> impl IntoView {
    let summary_res = Resource::new(|| (), |_| fetch_owner_summary_dist());
    let props_res = Resource::new(|| (), |_| fetch_owner_properties_dist());

    view! {
        <div class="main-area">
            <div class="page-header">
                <div>
                    <h1 class="page-title">"Distributions"</h1>
                    <p class="page-subtitle">"Your net revenue distributions after management fees"</p>
                </div>
                <div class="page-actions">
                    <span class="owner-readonly-badge">"👁 Read Only"</span>
                </div>
            </div>

            // ── Portfolio distribution summary ──
            <Suspense fallback=|| ()>
                {move || summary_res.get().map(|res| {
                    match res {
                        Ok(s) => {
                            let gross   = s.revenue_this_month_cents;
                            let net     = net_distribution(gross);
                            let fee     = gross - net;
                            let ytd_net = net_distribution(s.revenue_ytd_cents);
                            view! {
                                <div class="dist-summary-card">
                                    <div class="dist-summary-header">
                                        <span class="dist-summary-label">"Current Month Distribution"</span>
                                        <span class="owner-readonly-badge">"Est. {format!(\"{:.0}%\", (1.0-MGMT_FEE_PCT)*100.0)} net"</span>
                                    </div>
                                    <div class="dist-summary-amount">{fmt_usd(net)}</div>
                                    <div class="dist-summary-breakdown">
                                        <div class="dist-breakdown-item">
                                            <span class="dist-breakdown-label">"Gross Revenue"</span>
                                            <span class="dist-breakdown-value">{fmt_usd(gross)}</span>
                                        </div>
                                        <div class="dist-breakdown-item dist-breakdown-item--fee">
                                            <span class="dist-breakdown-label">"Mgmt Fee (~9%)"</span>
                                            <span class="dist-breakdown-value text-amber-400">"−" {fmt_usd(fee)}</span>
                                        </div>
                                        <div class="dist-breakdown-item dist-breakdown-item--net">
                                            <span class="dist-breakdown-label">"Net to Owner"</span>
                                            <span class="dist-breakdown-value" style="color:var(--green);font-weight:800;">{fmt_usd(net)}</span>
                                        </div>
                                    </div>
                                </div>

                                <div class="kpi-row" style="margin:1.25rem 0;">
                                    <div class="kpi-card">
                                        <span class="kpi-label">"YTD Net Distribution"</span>
                                        <span class="kpi-value" style="color:var(--green)">{fmt_usd(ytd_net)}</span>
                                    </div>
                                    <div class="kpi-card">
                                        <span class="kpi-label">"Outstanding to Distribute"</span>
                                        <span class="kpi-value" style="color:var(--amber)">{fmt_usd(s.outstanding_balance_cents)}</span>
                                    </div>
                                    <div class="kpi-card">
                                        <span class="kpi-label">"On-Time Payment Rate"</span>
                                        <span class="kpi-value" style="color:var(--cobalt)">{format!("{:.0}%", s.on_time_payment_rate_pct)}</span>
                                    </div>
                                </div>
                            }.into_any()
                        }
                        Err(_) => ().into_any(),
                    }
                })}
            </Suspense>

            // ── Per-property breakdown ──
            <div class="owner-section">
                <div class="owner-section-title">"Per-Property This Month"</div>
                <Suspense fallback=|| view! { <div class="doc-empty">"Loading…"</div> }>
                    {move || props_res.get().map(|res| {
                        match res {
                            Ok(props) if !props.is_empty() => view! {
                                <div class="dist-prop-list">
                                    <For
                                        each=move || props.clone()
                                        key=|p| p.asset_id
                                        children=move |p| {
                                            let gross = p.revenue_this_month_cents;
                                            let net   = net_distribution(gross);
                                            let fee   = gross - net;
                                            let name  = p.asset_name.clone();
                                            let addr  = p.address_line_1.clone().unwrap_or_else(|| "—".to_string());
                                            let aid   = p.asset_id;
                                            view! {
                                                <div class="dist-prop-row">
                                                    <div class="dist-prop-icon">"🏠"</div>
                                                    <div class="dist-prop-info">
                                                        <div class="dist-prop-name">{name}</div>
                                                        <div class="dist-prop-addr">{addr}</div>
                                                    </div>
                                                    <div class="dist-prop-amounts">
                                                        <div class="dist-prop-gross">"Gross " {fmt_usd(gross)}</div>
                                                        <div class="dist-prop-fee">"Fee −" {fmt_usd(fee)}</div>
                                                        <div class="dist-prop-net" style="color:var(--green);font-weight:700;">{fmt_usd(net)}</div>
                                                    </div>
                                                    <a href=format!("/o/properties/{}", aid) class="btn btn-ghost btn-sm">"Details →"</a>
                                                </div>
                                            }
                                        }
                                    />
                                </div>
                            }.into_any(),
                            _ => view! { <div class="doc-empty">"No properties found."</div> }.into_any(),
                        }
                    })}
                </Suspense>
            </div>

            // ── Banking info note ──
            <div class="viol-info-banner">
                <span class="viol-info-icon">"🏦"</span>
                <p class="viol-info-text">
                    "Distribution payments are processed by your property manager. "
                    "ACH/wire details are held on file by the PM. "
                    "Contact support to update banking information."
                </p>
            </div>
        </div>
    }
}
