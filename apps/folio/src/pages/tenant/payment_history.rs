// apps/folio/src/pages/tenant/payment_history.rs
//
// Tenant Payment History — /t/payments/history
//
// Full ledger view for the tenant. Shows all billing events (rent, charges,
// credits, fees) with status, amounts, and payment method.
//
// Endpoint: GET /api/folio/ledger
// ─────────────────────────────────────────────────────────────────────────────

use leptos::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// ── API types ─────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LedgerEntry {
    pub id: Uuid,
    pub billable_entity_type: String,
    pub billable_entity_id: Uuid,
    pub description: Option<String>,
    pub gross_amount_cents: i64,
    pub fee_amount_cents: i64,
    pub net_amount_cents: i64,
    pub currency: String,
    pub payment_rail: Option<String>,
    pub status: String,
    pub due_date: Option<String>,
    pub paid_at: Option<String>,
    pub reconciled_at: Option<String>,
    pub reconciliation_note: Option<String>,
    pub created_at: String,
}

// ── Server functions ──────────────────────────────────────────────────────────

#[server(FetchLedger, "/api")]
pub async fn fetch_ledger() -> Result<Vec<LedgerEntry>, server_fn::error::ServerFnError> {
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

fn cents_fmt(cents: i64, currency: &str) -> String {
    let upper = currency.to_uppercase();
    let symbol = match upper.as_str() {
        "USD" => "$",
        "EUR" => "€",
        "GBP" => "£",
        "CAD" => "CA$",
        "BTC" => "₿",
        _ => currency,
    };
    if currency.to_uppercase() == "BTC" {
        // BTC stored in satoshis (cents = satoshis)
        format!("₿{:.8}", cents as f64 / 1e8)
    } else {
        format!("{}{:.2}", symbol, cents as f64 / 100.0)
    }
}

fn status_style(status: &str) -> &'static str {
    match status.to_lowercase().as_str() {
        "paid" | "settled" | "reconciled" => "ph-badge--paid",
        "pending" | "processing" => "ph-badge--pending",
        "overdue" | "failed" => "ph-badge--overdue",
        "cancelled" | "voided" => "ph-badge--cancelled",
        _ => "ph-badge--default",
    }
}

fn rail_icon(rail: Option<&str>) -> &'static str {
    match rail {
        Some(r) if r.contains("stripe") => "💳",
        Some(r) if r.contains("btc") || r.contains("bitcoin") => "₿",
        Some(r) if r.contains("pix") => "🇧🇷",
        Some(r) if r.contains("wire") || r.contains("ach") => "🏦",
        Some(r) if r.contains("cash") => "💵",
        _ => "💰",
    }
}

// ── Component ─────────────────────────────────────────────────────────────────

#[component]
pub fn TenantPaymentHistory() -> impl IntoView {
    let refresh = RwSignal::new(0u32);
    let status_filter = RwSignal::new("all".to_string());

    let ledger_res = Resource::new(move || refresh.get(), |_| fetch_ledger());

    view! {
        <div class="main-area">

            // ── Header ──
            <div class="page-header">
                <div>
                    <h1 class="page-title">"Payment History"</h1>
                    <p class="page-subtitle">"All billing events, charges, and payments on your account"</p>
                </div>
                <div class="page-actions">
                    <button class="btn btn-ghost btn-sm" on:click=move |_| refresh.update(|n| *n += 1)>
                        "↻ Refresh"
                    </button>
                </div>
            </div>

            // ── Summary KPIs ──
            <Suspense fallback=|| ()>
                {move || ledger_res.get().map(|res| {
                    match res.as_ref() {
                        Ok(entries) => {
                            let total_paid  = entries.iter().filter(|e| e.status.to_lowercase() == "paid").map(|e| e.net_amount_cents).sum::<i64>();
                            let total_due   = entries.iter().filter(|e| e.status.to_lowercase() == "pending" || e.status.to_lowercase() == "overdue").map(|e| e.gross_amount_cents).sum::<i64>();
                            let overdue_ct  = entries.iter().filter(|e| e.status.to_lowercase() == "overdue").count();
                            let currency    = entries.first().map(|e| e.currency.clone()).unwrap_or_else(|| "USD".to_string());

                            view! {
                                <div class="kpi-row" style="margin-bottom:1.25rem;">
                                    <div class="kpi-card">
                                        <span class="kpi-label">"Total Paid"</span>
                                        <span class="kpi-value" style="color:var(--green)">{cents_fmt(total_paid, &currency)}</span>
                                    </div>
                                    <div class="kpi-card">
                                        <span class="kpi-label">"Outstanding"</span>
                                        <span class="kpi-value" style="color:var(--amber)">{cents_fmt(total_due, &currency)}</span>
                                    </div>
                                    <div class="kpi-card">
                                        <span class="kpi-label">"Overdue Items"</span>
                                        <span class="kpi-value" style="color:var(--red)">{overdue_ct.to_string()}</span>
                                    </div>
                                    <div class="kpi-card">
                                        <span class="kpi-label">"Total Entries"</span>
                                        <span class="kpi-value" style="color:var(--cobalt)">{entries.len().to_string()}</span>
                                    </div>
                                </div>
                            }.into_any()
                        }
                        Err(_) => ().into_any(),
                    }
                })}
            </Suspense>

            // ── Status filter pills ──
            <div class="doc-filter-row">
                {
                    let pill = move |scope: &'static str, label: &'static str| {
                        view! {
                            <button
                                class=move || format!("filter-pill {}", if status_filter.get() == scope { "filter-pill--active" } else { "" })
                                on:click=move |_| status_filter.set(scope.to_string())
                            >{label}</button>
                        }
                    };
                    view! {
                        {pill("all",     "All")}
                        {pill("paid",    "Paid")}
                        {pill("pending", "Pending")}
                        {pill("overdue", "Overdue")}
                    }
                }
            </div>

            // ── Ledger table ──
            <Suspense fallback=|| view! { <div class="doc-empty">"Loading payment history…"</div> }>
                {move || ledger_res.get().map(|res| {
                    match res {
                        Ok(entries) => {
                            let sf = status_filter.get();
                            let visible: Vec<_> = entries.into_iter().filter(|e| {
                                sf == "all" || e.status.to_lowercase().contains(&sf)
                            }).collect();

                            if visible.is_empty() {
                                return view! {
                                    <div class="doc-empty">"No entries found."</div>
                                }.into_any();
                            }

                            view! {
                                <div class="ph-table-wrap">
                                    <table class="ph-table">
                                        <thead>
                                            <tr>
                                                <th>"Date"</th>
                                                <th>"Description"</th>
                                                <th>"Method"</th>
                                                <th>"Amount"</th>
                                                <th>"Due"</th>
                                                <th>"Status"</th>
                                            </tr>
                                        </thead>
                                        <tbody>
                                            <For
                                                each=move || visible.clone()
                                                key=|e| e.id
                                                children=move |entry| {
                                                    let date   = entry.created_at.chars().take(10).collect::<String>();
                                                    let desc   = entry.description.clone().unwrap_or_else(|| entry.billable_entity_type.replace('_', " "));
                                                    let amount = cents_fmt(entry.gross_amount_cents, &entry.currency);
                                                    let due    = entry.due_date.clone().unwrap_or_else(|| "—".to_string());
                                                    let status = entry.status.clone();
                                                    let sc     = status_style(&entry.status).to_string();
                                                    let icon   = rail_icon(entry.payment_rail.as_deref());
                                                    let rail   = entry.payment_rail.clone().unwrap_or_else(|| "—".to_string());
                                                    let paid   = entry.paid_at.as_deref().map(|s| s.chars().take(10).collect::<String>());

                                                    view! {
                                                        <tr class="ph-row">
                                                            <td class="ph-date">{date}</td>
                                                            <td class="ph-desc">{desc}</td>
                                                            <td class="ph-rail">
                                                                <span class="ph-rail-wrap">
                                                                    {icon} " " {rail}
                                                                </span>
                                                            </td>
                                                            <td class="ph-amount">{amount}</td>
                                                            <td class="ph-due">
                                                                {if let Some(p) = paid {
                                                                    view! { <span class="ph-paid-at">"Paid " {p}</span> }.into_any()
                                                                } else {
                                                                    view! { <span>{due}</span> }.into_any()
                                                                }}
                                                            </td>
                                                            <td>
                                                                <span class=format!("ph-badge {sc}")>{status}</span>
                                                            </td>
                                                        </tr>
                                                    }
                                                }
                                            />
                                        </tbody>
                                    </table>
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
