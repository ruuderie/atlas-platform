//! Buyer CRM — `/l/buyers?track=wholesale|creative_finance`

use leptos::prelude::*;
use leptos_router::hooks::use_query_map;

use crate::pages::landlord::deals::fetch_deals;

fn is_buyer(opp_type: &str) -> bool {
    matches!(
        opp_type,
        "wholesale_buyer" | "creative_finance_disposition"
    )
}

#[component]
pub fn LandlordBuyers() -> impl IntoView {
    let query = use_query_map();
    let track = Memo::new(move |_| {
        query
            .get()
            .get("track")
            .map(|s| s.to_string())
            .unwrap_or_else(|| "wholesale".to_string())
    });

    let deals = Resource::new(
        move || track.get(),
        |t| fetch_deals(Some(t)),
    );

    view! {
        <div class="main-area">
            <div class="page-header">
                <div>
                    <h1 class="page-title">"Buyers"</h1>
                    <p class="page-subtitle">
                        {move || if track.get() == "creative_finance" {
                            "Tenant-buyers · money × credit matrix"
                        } else {
                            "Cash investors · capacity × close speed"
                        }}
                    </p>
                </div>
                <div class="page-actions">
                    <a class="btn btn-ghost btn-sm" href="/l/deals">"← Deals"</a>
                </div>
            </div>

            <div class="flex gap-2 mb-4">
                <a class=move || if track.get()=="wholesale"{"btn btn-primary btn-sm"}else{"btn btn-ghost btn-sm"}
                   href="/l/buyers?track=wholesale">"Cash buyers"</a>
                <a class=move || if track.get()=="creative_finance"{"btn btn-primary btn-sm"}else{"btn btn-ghost btn-sm"}
                   href="/l/buyers?track=creative_finance">"Tenant buyers"</a>
            </div>

            <Suspense fallback=|| view!{ <div class="doc-empty">"Loading…"</div> }>
                {move || deals.get().map(|res| match res {
                    Ok(all) => {
                        let buyers: Vec<_> = all.into_iter().filter(|d| is_buyer(&d.opportunity_type)).collect();
                        if buyers.is_empty() {
                            view! {
                                <div class="doc-empty">
                                    "No buyer leads yet. Create from Deal Ops → Disposition → + New."
                                </div>
                            }.into_any()
                        } else {
                            view! {
                                <table class="w-full text-sm">
                                    <thead>
                                        <tr>
                                            <th class="text-left p-2">"Name / address"</th>
                                            <th class="text-left p-2">"Status"</th>
                                            <th class="text-left p-2">"Offer / capacity"</th>
                                        </tr>
                                    </thead>
                                    <tbody>
                                        {buyers.into_iter().map(|b| {
                                            let href = format!("/l/deals/{}", b.id);
                                            view! {
                                                <tr>
                                                    <td class="p-2"><a href=href>{b.property_address}</a></td>
                                                    <td class="p-2">{b.status}</td>
                                                    <td class="p-2">{format!("${}", b.offer_cents.or(b.deal_amount_cents).unwrap_or(0)/100)}</td>
                                                </tr>
                                            }
                                        }).collect::<Vec<_>>()}
                                    </tbody>
                                </table>
                            }.into_any()
                        }
                    }
                    Err(_) => view!{ <div class="doc-empty">"Could not load buyers."</div> }.into_any(),
                })}
            </Suspense>
        </div>
    }
}
