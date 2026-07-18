//! Tenant payments — `/t/payments`
//! Wired to `GET /api/folio/ledger` (same source as payment history).

use leptos::prelude::*;
use leptos_router::components::A;

use crate::components::nav::FolioRoute;
use crate::components::page_header::PageHeader;
use crate::components::status_pill::{StatusPill, StatusPillTone};
use crate::pages::tenant::payment_history::fetch_ledger;

fn tone(status: &str) -> StatusPillTone {
    match status.to_ascii_lowercase().as_str() {
        "paid" | "settled" | "reconciled" => StatusPillTone::Ok,
        "pending" | "processing" => StatusPillTone::Warn,
        "overdue" | "failed" => StatusPillTone::Danger,
        _ => StatusPillTone::Neutral,
    }
}

fn fmt_cents(cents: i64, currency: &str) -> String {
    if currency.eq_ignore_ascii_case("BTC") {
        format!("₿{:.8}", cents as f64 / 1e8)
    } else {
        format!("${:.2}", cents as f64 / 100.0)
    }
}

#[component]
pub fn TenantPayments() -> impl IntoView {
    let ledger = Resource::new(|| (), |_| async move { fetch_ledger().await });

    view! {
        <div class="landlord-list-page">
            <PageHeader
                title=Signal::derive(|| "Payments".to_string())
                subtitle=Signal::derive(|| "Upcoming dues and recent charges.".to_string())
            >
                <A href=FolioRoute::TenantPaymentHistory.path() attr:class="folio-btn folio-btn--ghost press">
                    "Full history"
                </A>
            </PageHeader>

            <Suspense fallback=|| view! {
                <div class="folio-empty"><p class="folio-empty__sub">"Loading payments…"</p></div>
            }>
                {move || ledger.get().map(|result| match result {
                    Err(e) => view! {
                        <div class="folio-empty">
                            <span class="material-symbols-outlined folio-empty__icon">"error"</span>
                            <p class="folio-empty__heading">"Could not load payments"</p>
                            <p class="folio-empty__sub">{e.to_string()}</p>
                        </div>
                    }.into_any(),
                    Ok(items) if items.is_empty() => view! {
                        <div class="folio-empty">
                            <span class="material-symbols-outlined folio-empty__icon">"payments"</span>
                            <p class="folio-empty__heading">"No payment activity"</p>
                            <p class="folio-empty__sub">"Invoices and receipts appear here when billed."</p>
                        </div>
                    }.into_any(),
                    Ok(mut items) => {
                        items.sort_by(|a, b| b.created_at.cmp(&a.created_at));
                        let open: Vec<_> = items.iter().filter(|e| {
                            let s = e.status.to_ascii_lowercase();
                            s == "pending" || s == "overdue" || s == "processing"
                        }).cloned().collect();
                        let recent: Vec<_> = items.into_iter().take(12).collect();
                        let open_section = if open.is_empty() {
                            None
                        } else {
                            Some(view! {
                                <section class="proj-section" style="margin-bottom:1.5rem;">
                                    <div class="proj-section__head">
                                        <h3 class="proj-section__title">"Needs payment"</h3>
                                    </div>
                                    {open.into_iter().map(|e| {
                                        let amt = fmt_cents(e.gross_amount_cents, &e.currency);
                                        let label = e.description.clone().unwrap_or_else(|| "Invoice".into());
                                        view! {
                                            <div class="hub-activity-rail__row">
                                                <StatusPill label=e.status.clone() tone=tone(&e.status)/>
                                                <div class="hub-activity-rail__body">
                                                    <p class="hub-activity-rail__row-title">{label}</p>
                                                    <p class="hub-activity-rail__row-meta">{amt}</p>
                                                </div>
                                            </div>
                                        }
                                    }).collect_view()}
                                </section>
                            })
                        };
                        view! {
                            {open_section}
                            <section class="proj-section">
                                <div class="proj-section__head">
                                    <h3 class="proj-section__title">"Recent"</h3>
                                </div>
                                {recent.into_iter().map(|e| {
                                    let amt = fmt_cents(e.gross_amount_cents, &e.currency);
                                    let label = e.description.clone().unwrap_or_else(|| "Ledger entry".into());
                                    view! {
                                        <div class="hub-activity-rail__row">
                                            <StatusPill label=e.status.clone() tone=tone(&e.status)/>
                                            <div class="hub-activity-rail__body">
                                                <p class="hub-activity-rail__row-title">{label}</p>
                                                <p class="hub-activity-rail__row-meta">
                                                    {format!("{amt} · {}", e.created_at.chars().take(10).collect::<String>())}
                                                </p>
                                            </div>
                                        </div>
                                    }
                                }).collect_view()}
                            </section>
                        }.into_any()
                    }
                })}
            </Suspense>
        </div>
    }
}
