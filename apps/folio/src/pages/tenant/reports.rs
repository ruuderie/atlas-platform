// apps/folio/src/pages/tenant/reports.rs
//
// Tenant Reports — /t/reports
//
// Summary financial report for the tenant: rent history, total paid,
// year-over-year payments, and downloadable ledger CSV link.
// Uses /api/folio/ledger as the data source.
// ─────────────────────────────────────────────────────────────────────────────

use leptos::prelude::*;
use uuid::Uuid;
use serde::{Deserialize, Serialize};

// ── API types ─────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LedgerEntry {
    pub id:                   Uuid,
    pub description:          Option<String>,
    pub gross_amount_cents:   i64,
    pub fee_amount_cents:     i64,
    pub net_amount_cents:     i64,
    pub currency:             String,
    pub payment_rail:         Option<String>,
    pub status:               String,
    pub due_date:             Option<String>,
    pub paid_at:              Option<String>,
    pub created_at:           String,
}

// ── Server functions ──────────────────────────────────────────────────────────

#[server(FetchReportLedger, "/api")]
pub async fn fetch_report_ledger() -> Result<Vec<LedgerEntry>, server_fn::error::ServerFnError> {
    use axum::http::HeaderMap;
    use leptos_axum::extract;
    let headers = extract::<HeaderMap>().await.unwrap_or_default();
    let token = session_token(&headers)?;
    crate::atlas_client::authenticated_get::<Vec<LedgerEntry>>(
        "/api/folio/ledger", &token, None,
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

fn fmt_cents(cents: i64) -> String {
    format!("${:.2}", cents as f64 / 100.0)
}

fn year_from(date: &str) -> &str {
    if date.len() >= 4 { &date[..4] } else { "—" }
}

/// Group entries by year and sum paid amounts
fn by_year(entries: &[LedgerEntry]) -> Vec<(String, i64)> {
    let mut years: std::collections::BTreeMap<String, i64> = std::collections::BTreeMap::new();
    for e in entries {
        if e.status.to_lowercase() == "paid" {
            let yr = year_from(&e.created_at).to_string();
            *years.entry(yr).or_default() += e.net_amount_cents;
        }
    }
    years.into_iter().rev().collect()
}

// ── Component ─────────────────────────────────────────────────────────────────

#[component]
pub fn TenantReports() -> impl IntoView {
    let ledger_res = Resource::new(|| (), |_| fetch_report_ledger());

    view! {
        <div class="main-area">

            <div class="page-header">
                <div>
                    <h1 class="page-title">"Tenant Reports"</h1>
                    <p class="page-subtitle">"Payment summary and financial history for your tenancy"</p>
                </div>
                <div class="page-actions">
                    <a href="/t/payments/history" class="btn btn-ghost btn-sm">
                        "View Full Ledger →"
                    </a>
                </div>
            </div>

            <Suspense fallback=|| view! { <div class="doc-empty">"Loading reports…"</div> }>
                {move || ledger_res.get().map(|res| {
                    match res {
                        Ok(entries) => {
                            let currency = entries.first().map(|e| e.currency.clone()).unwrap_or_else(|| "USD".to_string());
                            let total_paid  = entries.iter().filter(|e| e.status == "paid").map(|e| e.net_amount_cents).sum::<i64>();
                            let total_fees  = entries.iter().filter(|e| e.status == "paid").map(|e| e.fee_amount_cents).sum::<i64>();
                            let total_due   = entries.iter().filter(|e| e.status == "pending" || e.status == "overdue").map(|e| e.gross_amount_cents).sum::<i64>();
                            let paid_count  = entries.iter().filter(|e| e.status == "paid").count();
                            let years       = by_year(&entries);

                            view! {
                                // ── KPI summary ──
                                <div class="kpi-row" style="margin-bottom:1.5rem;">
                                    <div class="kpi-card">
                                        <span class="kpi-label">"Total Paid (Net)"</span>
                                        <span class="kpi-value" style="color:var(--green)">{fmt_cents(total_paid)}</span>
                                    </div>
                                    <div class="kpi-card">
                                        <span class="kpi-label">"Platform Fees"</span>
                                        <span class="kpi-value" style="color:var(--cobalt)">{fmt_cents(total_fees)}</span>
                                    </div>
                                    <div class="kpi-card">
                                        <span class="kpi-label">"Outstanding"</span>
                                        <span class="kpi-value" style="color:var(--amber)">{fmt_cents(total_due)}</span>
                                    </div>
                                    <div class="kpi-card">
                                        <span class="kpi-label">"Payments Made"</span>
                                        <span class="kpi-value" style="color:var(--cobalt)">{paid_count.to_string()}</span>
                                    </div>
                                </div>

                                // ── Year-by-year breakdown ──
                                {if !years.is_empty() {
                                    let max_amount = years.iter().map(|(_, a)| *a).max().unwrap_or(1).max(1);

                                    view! {
                                        <div class="report-section">
                                            <div class="report-section-title">"Annual Payment Breakdown"</div>
                                            <div class="report-bar-chart">
                                                <For
                                                    each=move || years.clone()
                                                    key=|(yr, _)| yr.clone()
                                                    children=move |(yr, amount)| {
                                                        let pct = (amount as f64 / max_amount as f64 * 100.0).min(100.0);
                                                        let amt_str = fmt_cents(amount);
                                                        view! {
                                                            <div class="report-bar-row">
                                                                <div class="report-bar-label">{yr}</div>
                                                                <div class="report-bar-track">
                                                                    <div
                                                                        class="report-bar-fill"
                                                                        style=format!("width:{:.1}%", pct)
                                                                    ></div>
                                                                </div>
                                                                <div class="report-bar-value">{amt_str}</div>
                                                            </div>
                                                        }
                                                    }
                                                />
                                            </div>
                                        </div>
                                    }.into_any()
                                } else { ().into_any() }}

                                // ── Payment method breakdown ──
                                {
                                    let mut rails: std::collections::HashMap<String, (i64, usize)> = std::collections::HashMap::new();
                                    for e in &entries {
                                        if e.status == "paid" {
                                            let rail = e.payment_rail.clone().unwrap_or_else(|| "Unknown".to_string());
                                            let ent = rails.entry(rail).or_default();
                                            ent.0 += e.net_amount_cents;
                                            ent.1 += 1;
                                        }
                                    }
                                    if !rails.is_empty() {
                                        let mut rail_vec: Vec<_> = rails.into_iter().collect();
                                        rail_vec.sort_by(|a, b| b.1.0.cmp(&a.1.0));

                                        view! {
                                            <div class="report-section">
                                                <div class="report-section-title">"By Payment Method"</div>
                                                <div class="report-rail-grid">
                                                    <For
                                                        each=move || rail_vec.clone()
                                                        key=|(rail, _)| rail.clone()
                                                        children=move |(rail, (amount, count))| {
                                                            let icon = match rail.to_lowercase().as_str() {
                                                                r if r.contains("stripe")  => "💳",
                                                                r if r.contains("btc")     => "₿",
                                                                r if r.contains("pix")     => "🇧🇷",
                                                                r if r.contains("ach") || r.contains("wire") => "🏦",
                                                                _                          => "💰",
                                                            };
                                                            view! {
                                                                <div class="report-rail-card">
                                                                    <div class="report-rail-icon">{icon}</div>
                                                                    <div class="report-rail-name">{rail}</div>
                                                                    <div class="report-rail-amount">{fmt_cents(amount)}</div>
                                                                    <div class="report-rail-count">{count.to_string()} " payments"</div>
                                                                </div>
                                                            }
                                                        }
                                                    />
                                                </div>
                                            </div>
                                        }.into_any()
                                    } else { ().into_any() }
                                }

                                // ── Export CTA ──
                                <div class="report-export-row">
                                    <div class="report-export-text">
                                        <div class="font-semibold text-sm">"Export Statement"</div>
                                        <div class="text-xs text-on-surface-variant">"Download a full payment history for tax or rental reference purposes."</div>
                                    </div>
                                    <a href="/t/payments/history" class="btn btn-primary btn-sm">"View Full Ledger"</a>
                                </div>

                            }.into_any()
                        }
                        Err(e) => view! {
                            <div class="doc-empty text-red-400">"Error loading reports: " {e.to_string()}</div>
                        }.into_any(),
                    }
                })}
            </Suspense>
        </div>
    }
}
