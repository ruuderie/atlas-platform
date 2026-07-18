//! Offer structure — `/l/deals/:id/structure`

use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos_router::hooks::use_params_map;
use uuid::Uuid;

use crate::components::nav::FolioRoute;
use crate::pages::landlord::deals::fetch_deals;
use crate::pages::landlord::deal_workspace::post_deal_action;

#[component]
pub fn DealStructure() -> impl IntoView {
    let params = use_params_map();
    let id = Memo::new(move |_| {
        params
            .get()
            .get("id")
            .and_then(|s| Uuid::parse_str(s.as_str()).ok())
    });

    let structure = RwSignal::new("all_cash_mao".to_string());
    let exit_mode = RwSignal::new("wholesale_assignment".to_string());
    let offer = RwSignal::new(String::new());
    let rent = RwSignal::new(String::new());
    let loan = RwSignal::new(String::new());
    let piti = RwSignal::new(String::new());
    let deposit = RwSignal::new(String::new());
    let sale = RwSignal::new(String::new());
    let msg = RwSignal::new(String::new());
    let saving = RwSignal::new(false);

    let deal = Resource::new(
        move || id.get(),
        |maybe_id| async move {
            let Some(deal_id) = maybe_id else {
                return Err(server_fn::error::ServerFnError::new("missing id"));
            };
            let all = fetch_deals(None).await?;
            all.into_iter()
                .find(|d| d.id == deal_id)
                .ok_or_else(|| server_fn::error::ServerFnError::new("not found"))
        },
    );

    Effect::new(move |_| {
        if let Some(Ok(d)) = deal.get() {
            if d.track == "creative_finance" {
                structure.set("subject_to_free_equity".into());
                exit_mode.set("lease_option".into());
            }
            if let Some(s) = d.acquisition_structure {
                structure.set(s);
            }
            if let Some(e) = d.exit_mode {
                exit_mode.set(e);
            }
            if let Some(o) = d.offer_cents {
                offer.set((o / 100).to_string());
            }
        }
    });

    let save = move |_| {
        let Some(deal_id) = id.get() else { return };
        let body = serde_json::json!({
            "acquisition_structure": structure.get(),
            "exit_mode": exit_mode.get(),
            "offer_cents": offer.get().parse::<f64>().ok().map(|v| (v * 100.0) as i64),
            "planned_rent_cents": rent.get().parse::<f64>().ok().map(|v| (v * 100.0) as i64),
            "loan_balance_cents": loan.get().parse::<f64>().ok().map(|v| (v * 100.0) as i64),
            "piti_cents": piti.get().parse::<f64>().ok().map(|v| (v * 100.0) as i64),
            "option_deposit_target_cents": deposit.get().parse::<f64>().ok().map(|v| (v * 100.0) as i64),
            "planned_sale_price_cents": sale.get().parse::<f64>().ok().map(|v| (v * 100.0) as i64),
        });
        saving.set(true);
        spawn_local(async move {
            match post_deal_action(deal_id, "structure".into(), body).await {
                Ok(_) => msg.set("Offer structured".into()),
                Err(e) => msg.set(e.to_string()),
            }
            saving.set(false);
        });
    };

    view! {
        <div class="main-area">
            <Suspense fallback=|| view! { <div class="doc-empty">"Loading…"</div> }>
                {move || deal.get().map(|res| match res {
                    Ok(d) => {
                        let back = FolioRoute::LandlordDealDetail
                            .path()
                            .replace(":id", &d.id.to_string());
                        view! {
                            <div class="page-header">
                                <div>
                                    <a class="text-sm" href=back.clone()>"← Workspace"</a>
                                    <h1 class="page-title">"Structure offer"</h1>
                                    <p class="page-subtitle">{d.property_address.clone()}</p>
                                </div>
                            </div>

                            <div class="card p-4 space-y-4" style="max-width:40rem;">
                                <div class="form-field">
                                    <label class="form-label">"Acquisition structure"</label>
                                    <select class="form-input" prop:value=structure
                                        on:change=move |ev| structure.set(event_target_value(&ev))>
                                        <option value="all_cash_mao">"All-cash MAO (wholesale)"</option>
                                        <option value="subject_to_free_equity">"Subject-to free equity"</option>
                                        <option value="subject_to_cash_equity">"Subject-to cash equity"</option>
                                        <option value="subject_to_deferred_equity">"Subject-to deferred equity"</option>
                                        <option value="subject_to_seller_second">"Subject-to + seller second"</option>
                                        <option value="seller_finance_wrap">"Seller finance / wrap"</option>
                                        <option value="purchase_option">"Purchase option"</option>
                                    </select>
                                </div>
                                <div class="form-field">
                                    <label class="form-label">"Exit mode"</label>
                                    <select class="form-input" prop:value=exit_mode
                                        on:change=move |ev| exit_mode.set(event_target_value(&ev))>
                                        <option value="wholesale_assignment">"Wholesale assignment"</option>
                                        <option value="simultaneous_close">"Simultaneous close"</option>
                                        <option value="lease_option">"Lease-option"</option>
                                        <option value="owner_finance_wrap">"Owner finance wrap"</option>
                                        <option value="land_contract">"Land contract"</option>
                                        <option value="retail_cash">"Retail cash"</option>
                                    </select>
                                </div>
                                <div class="form-field">
                                    <label class="form-label">"Offer / takeover ($)"</label>
                                    <input type="number" class="form-input" prop:value=offer
                                        on:input=move |ev| offer.set(event_target_value(&ev)) />
                                </div>
                                <div class="form-field">
                                    <label class="form-label">"Loan balance ($)"</label>
                                    <input type="number" class="form-input" prop:value=loan
                                        on:input=move |ev| loan.set(event_target_value(&ev)) />
                                </div>
                                <div class="form-field">
                                    <label class="form-label">"PITI ($/mo)"</label>
                                    <input type="number" class="form-input" prop:value=piti
                                        on:input=move |ev| piti.set(event_target_value(&ev)) />
                                </div>
                                <div class="form-field">
                                    <label class="form-label">"Planned rent / payment in ($/mo)"</label>
                                    <input type="number" class="form-input" prop:value=rent
                                        on:input=move |ev| rent.set(event_target_value(&ev)) />
                                </div>
                                <div class="form-field">
                                    <label class="form-label">"Option deposit target ($)"</label>
                                    <input type="number" class="form-input" prop:value=deposit
                                        on:input=move |ev| deposit.set(event_target_value(&ev)) />
                                </div>
                                <div class="form-field">
                                    <label class="form-label">"Planned sale / option price ($)"</label>
                                    <input type="number" class="form-input" prop:value=sale
                                        on:input=move |ev| sale.set(event_target_value(&ev)) />
                                </div>
                                <Show when=move || !msg.get().is_empty()>
                                    <p class="text-sm">{move || msg.get()}</p>
                                </Show>
                                <button class="btn btn-primary" on:click=save disabled=move || saving.get()>
                                    {move || if saving.get() { "Saving…" } else { "Save structure" }}
                                </button>
                            </div>
                        }.into_any()
                    }
                    Err(e) => view! { <div class="doc-empty">{e.to_string()}</div> }.into_any(),
                })}
            </Suspense>
        </div>
    }
}
