//! Tenant-buyer lease-option portal — `/t/option`
//! Rent from active lease; option economics from creative-finance deals when present.

use leptos::prelude::*;

use crate::components::nav::FolioRoute;
use crate::pages::landlord::deals::{fetch_deals, DealSummary};
use crate::pages::landlord::leases::{list_leases, LeaseStatus};

fn fmt_cents(cents: Option<i64>) -> String {
    cents
        .map(|c| format!("${:.0}", c as f64 / 100.0))
        .unwrap_or_else(|| "—".into())
}

#[component]
pub fn TenantLeaseOption() -> impl IntoView {
    let leases = Resource::new(|| (), |_| async move { list_leases().await });
    let deals = Resource::new(
        || (),
        |_| async move { fetch_deals(Some("creative_finance".into())).await },
    );

    view! {
        <div class="main-area">
            <div class="page-header">
                <div>
                    <h1 class="page-title">"My Option"</h1>
                    <p class="page-subtitle">"Lease-option path to purchase · rent + Down Payment Assistance"</p>
                </div>
            </div>

            <Suspense fallback=|| view! { <div class="doc-empty">"Loading…"</div> }>
                {move || {
                    let lease = leases.get().and_then(|r| r.ok()).and_then(|items| {
                        items.into_iter().find(|l| {
                            LeaseStatus::from_str(&l.status) == LeaseStatus::Active
                        })
                    });
                    let deal: Option<DealSummary> = deals.get().and_then(|r| r.ok()).and_then(|items| {
                        items.into_iter().find(|d| {
                            d.acquisition_structure
                                .as_deref()
                                .map(|s| s.contains("option") || s.contains("lease"))
                                .unwrap_or(false)
                                || d.deal_amount_cents.is_some()
                        })
                    });

                    let rent = lease
                        .as_ref()
                        .and_then(|l| l.monthly_rent_cents)
                        .map(|c| format!("${:.0}", c as f64 / 100.0))
                        .unwrap_or_else(|| "—".into());

                    let option_price = deal.as_ref().and_then(|d| d.deal_amount_cents.or(d.offer_cents));
                    let has_option = option_price.is_some();

                    view! {
                        <div class="card p-5 mb-4" style="background:#191c1e;color:#fff;border-radius:1rem;">
                            <p class="text-xs uppercase opacity-60 mb-1">"Path to purchase"</p>
                            {if has_option {
                                view! {
                                    <div class="grid gap-4" style="grid-template-columns:1fr 1fr;">
                                        <div>
                                            <p class="text-sm opacity-70">"Option / deal amount"</p>
                                            <p class="text-2xl font-bold">{fmt_cents(option_price)}</p>
                                        </div>
                                        <div>
                                            <p class="text-sm opacity-70">"Structure"</p>
                                            <p class="text-xl font-bold">
                                                {deal.as_ref()
                                                    .and_then(|d| d.acquisition_structure.clone())
                                                    .unwrap_or_else(|| "Lease-option".into())}
                                            </p>
                                        </div>
                                        <div>
                                            <p class="text-sm opacity-70">"Property"</p>
                                            <p class="text-sm font-bold">
                                                {deal.as_ref().map(|d| d.property_address.clone()).unwrap_or_default()}
                                            </p>
                                        </div>
                                        <div>
                                            <p class="text-sm opacity-70">"Status"</p>
                                            <p class="text-sm font-bold">
                                                {deal.as_ref().map(|d| d.status.clone()).unwrap_or_default()}
                                            </p>
                                        </div>
                                    </div>
                                }.into_any()
                            } else {
                                view! {
                                    <p class="text-sm opacity-80 mt-2">
                                        "No lease-option economics on file yet. When your household has an active purchase path, option price and DPAP credits appear here."
                                    </p>
                                }.into_any()
                            }}
                        </div>

                        <div class="card p-4 mb-4">
                            <h3 class="font-bold mb-2">"This month"</h3>
                            <div class="flex justify-between py-2 border-b text-sm">
                                <span>"Rent (required)"</span>
                                <strong>{rent}</strong>
                            </div>
                            <p class="text-xs text-on-surface-variant mt-2">
                                "DPAP extras and price credits appear when your creative-finance deal includes them."
                            </p>
                            <a class="btn btn-primary btn-sm mt-3" href=FolioRoute::TenantPayments.path()>"Pay"</a>
                        </div>

                        <div class="card p-4 text-sm text-on-surface-variant">
                            <p>"Repairs after day 30 are typically your responsibility under lease-option addenda. Confirm terms in your documents."</p>
                            <a class="underline mt-2 inline-block" href=FolioRoute::TenantDocuments.path()>
                                "View documents"
                            </a>
                        </div>
                    }.into_any()
                }}
            </Suspense>
        </div>
    }
}
