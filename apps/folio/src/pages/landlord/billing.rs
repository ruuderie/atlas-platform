//! Billing (rent collection) — `/l/billing`
//!
//! Surfaces ledger entries (invoices / payments) via `GET /api/folio/ledger`.
//! Platform SaaS billing lives at `/l/account/billing`.

use leptos::prelude::*;
use leptos_router::components::A;

use crate::components::nav::FolioRoute;
use crate::components::page_header::PageHeader;
use crate::components::stat_card::StatCard;
use crate::pages::landlord::ledger::{list_ledger_entries, EntryStatus, LedgerEntrySummary};

#[component]
pub fn Billing() -> impl IntoView {
    let entries = Resource::new(|| (), |_| async move { list_ledger_entries().await });

    let title = Signal::derive(|| "Billing".to_string());
    let subtitle = Signal::derive(|| {
        "Rent invoices, payments, and outstanding balances.".to_string()
    });

    let outstanding = Signal::derive(move || {
        let cents: i64 = entries
            .get()
            .and_then(|r| r.ok())
            .map(|items| {
                items
                    .iter()
                    .filter(|e| {
                        matches!(
                            EntryStatus::from_str(&e.status),
                            EntryStatus::Pending | EntryStatus::Processing
                        )
                    })
                    .map(|e| e.net_amount_cents)
                    .sum()
            })
            .unwrap_or(0);
        format_money(cents)
    });

    let paid_count = Signal::derive(move || {
        entries
            .get()
            .and_then(|r| r.ok())
            .map(|items| {
                items
                    .iter()
                    .filter(|e| EntryStatus::from_str(&e.status) == EntryStatus::Paid)
                    .count()
            })
            .unwrap_or(0)
            .to_string()
    });

    let open_count = Signal::derive(move || {
        entries
            .get()
            .and_then(|r| r.ok())
            .map(|items| {
                items
                    .iter()
                    .filter(|e| {
                        matches!(
                            EntryStatus::from_str(&e.status),
                            EntryStatus::Pending | EntryStatus::Processing
                        )
                    })
                    .count()
            })
            .unwrap_or(0)
            .to_string()
    });

    view! {
        <div class="landlord-list-page">
            <PageHeader title=title subtitle=subtitle>
                <A href=FolioRoute::LandlordLedger.path() attr:class="folio-btn folio-btn--ghost press">
                    <span class="material-symbols-outlined">"account_balance"</span>
                    "Full ledger"
                </A>
            </PageHeader>
            <nav class="folio-related" aria-label="Related">
                <span class="folio-related__label">"Related"</span>
                <ul class="folio-related__list">
                    <li>
                        <A href=FolioRoute::LandlordLedger.path() attr:class="folio-related__link press">
                            "Ledger"
                        </A>
                    </li>
                    <li>
                        <A href=FolioRoute::LandlordAccountBilling.path() attr:class="folio-related__link press">
                            "Account billing"
                        </A>
                    </li>
                </ul>
            </nav>

            <div class="folio-stat-grid" style="margin-bottom:1.5rem">
                <StatCard label="Outstanding" value=outstanding icon="payments" />
                <StatCard label="Open Invoices" value=open_count icon="receipt_long" />
                <StatCard label="Paid" value=paid_count icon="check_circle" />
            </div>

            <section class="folio-section-card">
                <div class="folio-section-card__header">
                    <h2 class="folio-section-card__title">"Recent activity"</h2>
                </div>
                <div class="folio-section-card__body" style="padding:0">
                    <Suspense fallback=|| view! {
                        <div class="folio-empty" style="padding:1.25rem">
                            <p class="folio-empty__sub">"Loading billing…"</p>
                        </div>
                    }>
                        {move || entries.get().map(|result| match result {
                            Err(e) => view! {
                                <div class="folio-empty" style="padding:1.25rem">
                                    <p class="folio-empty__heading">"Could not load billing"</p>
                                    <p class="folio-empty__sub">{e.to_string()}</p>
                                </div>
                            }.into_any(),
                            Ok(items) if items.is_empty() => view! {
                                <div class="folio-empty" style="padding:1.25rem">
                                    <span class="material-symbols-outlined folio-empty__icon">"receipt_long"</span>
                                    <p class="folio-empty__heading">"No invoices yet"</p>
                                    <p class="folio-empty__sub">
                                        "When leases generate rent charges, they appear here and in the ledger."
                                    </p>
                                </div>
                            }.into_any(),
                            Ok(items) => {
                                let preview: Vec<LedgerEntrySummary> =
                                    items.into_iter().take(25).collect();
                                view! {
                                    <div class="landlord-table-wrap" style="border:none;border-radius:0;box-shadow:none">
                                        <table class="landlord-table">
                                            <thead>
                                                <tr>
                                                    <th>"Description"</th>
                                                    <th>"Amount"</th>
                                                    <th>"Status"</th>
                                                    <th>"Due"</th>
                                                </tr>
                                            </thead>
                                            <tbody>
                                                {preview.into_iter().map(|e| {
                                                    let desc = e.description.clone()
                                                        .unwrap_or_else(|| e.billable_entity_type.clone());
                                                    let amount = format_money(e.net_amount_cents);
                                                    let status = EntryStatus::from_str(&e.status);
                                                    let due = e.due_date
                                                        .map(|d| d.to_string())
                                                        .unwrap_or_else(|| "—".into());
                                                    let pill = match status {
                                                        EntryStatus::Paid => "landlord-pill landlord-pill--ok",
                                                        EntryStatus::Pending | EntryStatus::Processing => {
                                                            "landlord-pill landlord-pill--warn"
                                                        }
                                                        EntryStatus::Failed => "landlord-pill landlord-pill--muted",
                                                        _ => "landlord-pill landlord-pill--muted",
                                                    };
                                                    view! {
                                                        <tr>
                                                            <td>{desc}</td>
                                                            <td>{amount}</td>
                                                            <td>
                                                                <span class=pill>{status.as_str()}</span>
                                                            </td>
                                                            <td>{due}</td>
                                                        </tr>
                                                    }
                                                }).collect_view()}
                                            </tbody>
                                        </table>
                                    </div>
                                }.into_any()
                            }
                        })}
                    </Suspense>
                </div>
            </section>
        </div>
    }
}

fn format_money(cents: i64) -> String {
    let dollars = cents as f64 / 100.0;
    if cents == 0 {
        "$0".to_string()
    } else {
        format!("${dollars:.2}")
    }
}
