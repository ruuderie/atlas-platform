// apps/folio/src/pages/landlord/account_billing.rs
//
// Account Billing — /l/account/billing
//
// Operator-level billing: Atlas Platform subscription plan, usage, and invoices.
// This is distinct from /l/billing (tenant rent billing).
// Uses /api/folio/billing/invoice/btc/audit and ledger for platform charges.
// ─────────────────────────────────────────────────────────────────────────────

use leptos::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// ── API types ─────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LedgerEntry {
    pub id: Uuid,
    pub description: Option<String>,
    pub gross_amount_cents: i64,
    pub fee_amount_cents: i64,
    pub net_amount_cents: i64,
    pub currency: String,
    pub payment_rail: Option<String>,
    pub status: String,
    pub due_date: Option<String>,
    pub paid_at: Option<String>,
    pub created_at: String,
}

// ── Server functions ──────────────────────────────────────────────────────────

#[server(FetchPlatformBillingLedger, "/api")]
pub async fn fetch_platform_billing_ledger(
) -> Result<Vec<LedgerEntry>, server_fn::error::ServerFnError> {
    use axum::http::HeaderMap;
    use leptos_axum::extract;
    let headers = extract::<HeaderMap>().await.unwrap_or_default();
    let token = session_token(&headers)?;
    crate::atlas_client::authenticated_get::<Vec<LedgerEntry>>("/api/folio/ledger", &token, None)
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
pub fn LandlordAccountBilling() -> impl IntoView {
    let refresh = RwSignal::new(0u32);

    let ledger_res = Resource::new(move || refresh.get(), |_| fetch_platform_billing_ledger());

    view! {
        <div class="main-area">

            <div class="page-header">
                <div>
                    <h1 class="page-title">"Account Billing"</h1>
                    <p class="page-subtitle">"Your Atlas Platform subscription, usage, and transaction history"</p>
                </div>
                <div class="page-actions">
                    <button class="btn btn-ghost btn-sm" on:click=move |_| refresh.update(|n| *n += 1)>
                        "↻ Refresh"
                    </button>
                </div>
            </div>

            // ── Subscription plan banner ──
            <div class="acct-plan-card">
                <div class="acct-plan-info">
                    <div class="acct-plan-badge">"⚡ Atlas Pro"</div>
                    <div class="acct-plan-desc">"Unlimited assets · Advanced analytics · Priority support"</div>
                </div>
                <div class="acct-plan-actions">
                    <button class="btn btn-ghost btn-sm" disabled=true title="Billing portal (Phase 7)">"Manage Plan"</button>
                </div>
            </div>

            // ── Payment Methods ──
            <div class="acct-section">
                <div class="acct-section-title">"Payment Methods"</div>
                <div class="acct-pm-row">
                    <div class="acct-pm-card">
                        <span class="acct-pm-icon">"💳"</span>
                        <div>
                            <div class="acct-pm-label">"Stripe"</div>
                            <div class="acct-pm-sub">"Credit / debit card · Automatic billing"</div>
                        </div>
                        <button class="btn btn-ghost btn-sm">"Configure"</button>
                    </div>
                    <div class="acct-pm-card">
                        <span class="acct-pm-icon">"₿"</span>
                        <div>
                            <div class="acct-pm-label">"Bitcoin Lightning"</div>
                            <div class="acct-pm-sub">"Self-sovereign payments via on-chain or Lightning"</div>
                        </div>
                        <button class="btn btn-ghost btn-sm">"Configure"</button>
                    </div>
                </div>
            </div>

            // ── Invoices / ledger ──
            <div class="acct-section">
                <div class="acct-section-title">"Transaction History"</div>

                <Suspense fallback=|| view! { <div class="doc-empty">"Loading billing history…"</div> }>
                    {move || ledger_res.get().map(|res| {
                        match res {
                            Ok(entries) if !entries.is_empty() => {
                                let total_fees = entries.iter()
                                    .filter(|e| e.status == "paid")
                                    .map(|e| e.fee_amount_cents)
                                    .sum::<i64>();
                                view! {
                                    <div class="kpi-row" style="margin-bottom:1rem;">
                                        <div class="kpi-card">
                                            <span class="kpi-label">"Platform Fees Paid"</span>
                                            <span class="kpi-value" style="color:var(--cobalt)">{fmt_usd(total_fees)}</span>
                                        </div>
                                        <div class="kpi-card">
                                            <span class="kpi-label">"Transactions"</span>
                                            <span class="kpi-value">{entries.len().to_string()}</span>
                                        </div>
                                    </div>

                                    <div class="ph-table-wrap">
                                        <table class="ph-table">
                                            <thead>
                                                <tr>
                                                    <th>"Date"</th>
                                                    <th>"Description"</th>
                                                    <th>"Gross"</th>
                                                    <th>"Platform Fee"</th>
                                                    <th>"Net"</th>
                                                    <th>"Status"</th>
                                                </tr>
                                            </thead>
                                            <tbody>
                                                <For
                                                    each=move || entries.clone()
                                                    key=|e| e.id
                                                    children=move |entry| {
                                                        let date = entry.created_at.chars().take(10).collect::<String>();
                                                        let desc = entry.description.clone().unwrap_or_else(|| "Transaction".to_string());
                                                        let sc   = match entry.status.to_lowercase().as_str() {
                                                            "paid"    => "ph-badge--paid",
                                                            "pending" => "ph-badge--pending",
                                                            "overdue" => "ph-badge--overdue",
                                                            _         => "ph-badge--default",
                                                        }.to_string();
                                                        let status = entry.status.clone();
                                                        view! {
                                                            <tr class="ph-row">
                                                                <td class="ph-date">{date}</td>
                                                                <td class="ph-desc">{desc}</td>
                                                                <td class="ph-amount">{fmt_usd(entry.gross_amount_cents)}</td>
                                                                <td class="ph-amount" style="color:var(--amber)">{fmt_usd(entry.fee_amount_cents)}</td>
                                                                <td class="ph-amount" style="color:var(--green)">{fmt_usd(entry.net_amount_cents)}</td>
                                                                <td><span class=format!("ph-badge {sc}")>{status}</span></td>
                                                            </tr>
                                                        }
                                                    }
                                                />
                                            </tbody>
                                        </table>
                                    </div>
                                }.into_any()
                            }
                            _ => view! {
                                <div class="doc-empty">"No billing history yet."</div>
                            }.into_any(),
                        }
                    })}
                </Suspense>
            </div>

            // ── Danger zone ──
            <div class="acct-section acct-section--danger">
                <div class="acct-section-title text-red-400">"Billing Actions"</div>
                <div class="acct-danger-row">
                    <div>
                        <div class="text-sm font-semibold">"Cancel Subscription"</div>
                        <div class="text-xs text-on-surface-variant">"Data retained for 90 days after cancellation."</div>
                    </div>
                    <button class="btn btn-sm" style="background:rgba(239,68,68,0.08);border:1px solid rgba(239,68,68,0.25);color:#f87171;" disabled=true>
                        "Contact Support"
                    </button>
                </div>
            </div>

        </div>
    }
}
