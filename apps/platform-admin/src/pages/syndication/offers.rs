//! Syndication Offers — platform-admin page.
//!
//! Lists all `atlas_syndication_offer` records and provides create/update/retire
//! actions. Includes a one-click "Auto-Provision" button that scans all app
//! instances on mandatory billing tiers and creates missing active links.
//!
//! Route: /syndication/offers

use leptos::prelude::*;
use serde_json::json;

use crate::api::syndication::{
    list_syndication_offers, create_syndication_offer, retire_syndication_offer,
    auto_provision_mandatory_links, CreateOfferInput, SyndicationOfferModel,
};

#[component]
pub fn SyndicationOffers() -> impl IntoView {
    let toast = use_context::<crate::app::GlobalToast>().expect("toast context");

    // ── List offers ──────────────────────────────────────────────────────────
    let offers_res = LocalResource::new(|| async { list_syndication_offers().await.unwrap_or_default() });

    // ── Create modal state ───────────────────────────────────────────────────
    let show_create = RwSignal::new(false);
    let form_ni_config_id = RwSignal::new(String::new());
    let form_display_name = RwSignal::new(String::new());
    let form_description = RwSignal::new(String::new());
    let form_link_type = RwSignal::new("marketplace_syndication".to_string());
    let form_ltr = RwSignal::new(true);
    let form_str = RwSignal::new(false);
    let form_for_sale = RwSignal::new(false);
    let form_vendor_profile = RwSignal::new(false);
    let form_mandatory_free = RwSignal::new(false);
    let form_mandatory_starter = RwSignal::new(false);
    let form_self_service = RwSignal::new(false);
    let form_folio_mode = RwSignal::new(String::new());
    let is_saving = RwSignal::new(false);

    // ── Provision modal state ────────────────────────────────────────────────
    let provision_offer_id = RwSignal::new(Option::<String>::None);
    let provision_offer_name = RwSignal::new(String::new());
    let is_provisioning = RwSignal::new(false);
    let provision_result = RwSignal::new(Option::<(u32, u32)>::None);

    // ── Helpers ──────────────────────────────────────────────────────────────
    let t1 = toast.clone();
    let handle_create = move |_| {
        if form_display_name.get().trim().is_empty() || form_ni_config_id.get().trim().is_empty() {
            t1.show_toast("Validation", "Display name and NI Config ID are required.", "error");
            return;
        }
        is_saving.set(true);
        let mut types = vec![];
        if form_ltr.get() { types.push("ltr"); }
        if form_str.get() { types.push("str"); }
        if form_for_sale.get() { types.push("for_sale"); }
        if form_vendor_profile.get() { types.push("vendor_profile"); }

        let mut tiers: Vec<&str> = vec![];
        if form_mandatory_free.get() { tiers.push("free"); }
        if form_mandatory_starter.get() { tiers.push("starter"); }

        let mode = form_folio_mode.get();

        let input = CreateOfferInput {
            ni_config_id: form_ni_config_id.get(),
            display_name: form_display_name.get().trim().to_string(),
            description: {
                let d = form_description.get();
                if d.is_empty() { None } else { Some(d) }
            },
            syndication_types: json!(types),
            link_type: form_link_type.get(),
            is_mandatory_for_tiers: json!(tiers),
            self_service_allowed: form_self_service.get(),
            applies_to_folio_mode: if mode.is_empty() { None } else { Some(mode) },
            applies_to_app_slug: None,
        };

        let t = t1.clone();
        leptos::task::spawn_local(async move {
            match create_syndication_offer(input).await {
                Ok(_) => {
                    t.show_toast("Created", "Syndication offer created.", "success");
                    show_create.set(false);
                    offers_res.refetch();
                }
                Err(e) => t.show_toast("Error", &e, "error"),
            }
            is_saving.set(false);
        });
    };

    let t2 = toast.clone();
    let handle_retire = move |id: String, name: String| {
        let t = t2.clone();
        let id = id.clone();
        leptos::task::spawn_local(async move {
            match retire_syndication_offer(&id).await {
                Ok(_) => {
                    t.show_toast("Retired", &format!("\"{}\" retired. Existing links remain active.", name), "success");
                    offers_res.refetch();
                }
                Err(e) => t.show_toast("Error", &e, "error"),
            }
        });
    };

    let t3 = toast.clone();
    let handle_provision = move |_| {
        let Some(oid) = provision_offer_id.get() else { return; };
        is_provisioning.set(true);
        provision_result.set(None);
        let t = t3.clone();
        leptos::task::spawn_local(async move {
            match auto_provision_mandatory_links(&oid).await {
                Ok(r) => {
                    provision_result.set(Some((r.provisioned, r.skipped)));
                    t.show_toast(
                        "Auto-Provisioned",
                        &format!("{} new links created, {} skipped.", r.provisioned, r.skipped),
                        "success"
                    );
                }
                Err(e) => t.show_toast("Error", &e, "error"),
            }
            is_provisioning.set(false);
        });
    };

    view! {
        <div class="space-y-6">
            // ── Page Header ──────────────────────────────────────────────────
            <div class="flex items-start justify-between">
                <div>
                    <h1 class="text-2xl font-extrabold text-on-surface tracking-tight">
                        "Syndication Offers"
                    </h1>
                    <p class="text-sm text-on-surface-variant mt-1">
                        "Platform-wide catalog of available syndication connections. "
                        "Operators activate these from their Folio instance settings."
                    </p>
                </div>
                <div class="flex items-center gap-3">
                    <a
                        href="/syndication/links"
                        class="text-xs text-primary hover:underline font-semibold"
                    >
                        "View Active Links →"
                    </a>
                    <button
                        class="btn-primary-gradient px-4 py-2 rounded-lg text-sm font-semibold text-on-primary-container shadow-md hover:opacity-90 active:scale-95 transition-all"
                        on:click=move |_| show_create.set(true)
                    >
                        "+ New Offer"
                    </button>
                </div>
            </div>

            // ── Explainer Cards ──────────────────────────────────────────────
            <div class="grid grid-cols-1 md:grid-cols-3 gap-4">
                <div class="bg-surface-container-low border border-outline-variant/20 rounded-xl p-4">
                    <div class="text-xs font-bold uppercase tracking-wider text-primary mb-1">"Layer A — Offer Catalog"</div>
                    <p class="text-xs text-on-surface-variant leading-relaxed">
                        "You define what syndication connections exist on the platform, "
                        "their terms, and which billing tiers must activate them automatically."
                    </p>
                </div>
                <div class="bg-surface-container-low border border-outline-variant/20 rounded-xl p-4">
                    <div class="text-xs font-bold uppercase tracking-wider text-emerald-400 mb-1">"Layer B — Active Links"</div>
                    <p class="text-xs text-on-surface-variant leading-relaxed">
                        "One row per (source instance, NI) pair. Created via operator self-service "
                        "(if permitted), admin manual, or auto-provisioned for mandatory tiers."
                    </p>
                </div>
                <div class="bg-surface-container-low border border-outline-variant/20 rounded-xl p-4 border-l-4 border-l-amber-400">
                    <div class="text-xs font-bold uppercase tracking-wider text-amber-400 mb-1">"Monetization Gate"</div>
                    <p class="text-xs text-on-surface-variant leading-relaxed">
                        "Free-tier instances cannot opt out of mandatory offers. "
                        "Their listings syndicate to your platform marketplace — that's your revenue model."
                    </p>
                </div>
            </div>

            // ── Offers Table ─────────────────────────────────────────────────
            <div class="bg-surface-container-low border border-outline-variant/20 rounded-xl overflow-hidden shadow-sm">
                <div class="flex justify-between items-center px-6 py-4 border-b border-outline-variant/20 bg-surface-container-high/40">
                    <h3 class="text-xs font-bold uppercase tracking-wider text-on-surface-variant">
                        "Active Offers"
                    </h3>
                    <span class="text-xs text-on-surface-variant/60">
                        {move || match offers_res.get() {
                            Some(ref o) => format!("{} offers", o.len()),
                            None => "Loading…".to_string(),
                        }}
                    </span>
                </div>
                <table class="w-full text-left">
                    <thead>
                        <tr class="bg-surface-container-high/20 border-b border-outline-variant/10 text-[10px] uppercase tracking-wider text-on-surface-variant/70">
                            <th class="px-6 py-3 font-semibold">"Name"</th>
                            <th class="px-6 py-3 font-semibold">"Link Type"</th>
                            <th class="px-6 py-3 font-semibold">"Syndicates"</th>
                            <th class="px-6 py-3 font-semibold">"Mandatory For"</th>
                            <th class="px-6 py-3 font-semibold">"Self-Service"</th>
                            <th class="px-6 py-3 font-semibold">"Folio Mode"</th>
                            <th class="px-6 py-3 font-semibold text-right">"Actions"</th>
                        </tr>
                    </thead>
                    <tbody class="divide-y divide-outline-variant/10">
                        <Suspense fallback=move || view! {
                            <tr><td colspan="7" class="px-6 py-8 text-center text-on-surface-variant text-sm">
                                "Loading offers…"
                            </td></tr>
                        }>
                            {move || {
                                let offers = offers_res.get().unwrap_or_default();
                                if offers.is_empty() {
                                    return view! {
                                        <tr><td colspan="7" class="px-6 py-10 text-center text-on-surface-variant/60 text-sm">
                                            "No offers yet. Click \"+ New Offer\" to create one."
                                        </td></tr>
                                    }.into_any();
                                }
                                offers.into_iter().map(|offer| {
                                    // Pre-clone all fields to avoid partial-move / borrow conflicts
                                    // across multiple `move` closures for the same `offer`.
                                    let oid = offer.id.clone();
                                    let oname = offer.display_name.clone();
                                    let oid2 = oid.clone();
                                    let oname2 = oname.clone();
                                    let hr = handle_retire.clone();

                                    // Borrow-free copies for use inside closures
                                    let display_name = offer.display_name.clone();
                                    let id_prefix = offer.id.chars().take(8).collect::<String>() + "…";
                                    let link_type = offer.link_type.clone();
                                    let link_type2 = link_type.clone();
                                    let link_label = offer.link_type_label().to_string();
                                    let types_display = offer.types_display();
                                    let has_mandatory = offer.is_mandatory_for_tiers.as_array().map(|a| !a.is_empty()).unwrap_or(false);
                                    let has_mandatory2 = has_mandatory;
                                    let mandatory_display = offer.mandatory_tiers_display();
                                    let self_service = offer.self_service_allowed;
                                    let folio_mode = offer.applies_to_folio_mode.clone().unwrap_or_else(|| "all".to_string());

                                    view! {
                                        <tr class="hover:bg-surface-bright/5 transition-colors">
                                            <td class="px-6 py-4">
                                                <div class="font-semibold text-sm text-on-surface">{display_name}</div>
                                                <div class="text-[10px] text-on-surface-variant/60 font-mono mt-0.5">
                                                    {id_prefix}
                                                </div>
                                            </td>
                                            <td class="px-6 py-4">
                                                <span class=move || match link_type.as_str() {
                                                    "branded_portal" => "inline-flex items-center px-2 py-0.5 rounded text-[10px] font-bold bg-purple-500/10 text-purple-400 border border-purple-500/20 uppercase tracking-wider",
                                                    _ => "inline-flex items-center px-2 py-0.5 rounded text-[10px] font-bold bg-primary/10 text-primary border border-primary/20 uppercase tracking-wider",
                                                }>
                                                    {link_label}
                                                </span>
                                            </td>
                                            <td class="px-6 py-4 text-xs text-on-surface-variant font-mono">
                                                {types_display}
                                            </td>
                                            <td class="px-6 py-4">
                                                {if has_mandatory {
                                                    view! {
                                                        <span class="inline-flex items-center px-2 py-0.5 rounded text-[10px] font-bold bg-amber-500/10 text-amber-400 border border-amber-500/20">
                                                            {mandatory_display}
                                                        </span>
                                                    }.into_any()
                                                } else {
                                                    view! { <span class="text-xs text-on-surface-variant/40">"optional"</span> }.into_any()
                                                }}
                                            </td>
                                            <td class="px-6 py-4">
                                                {if self_service {
                                                    view! { <span class="text-emerald-400 text-xs font-semibold">"✓ Yes"</span> }.into_any()
                                                } else {
                                                    view! { <span class="text-on-surface-variant/40 text-xs">"Admin only"</span> }.into_any()
                                                }}
                                            </td>
                                            <td class="px-6 py-4 text-xs text-on-surface-variant font-mono">
                                                {folio_mode}
                                            </td>
                                            <td class="px-6 py-4 text-right">
                                                <div class="flex items-center justify-end gap-2">
                                                    {if has_mandatory2 {
                                                        let oid3 = oid.clone();
                                                        let oname3 = oname.clone();
                                                        view! {
                                                            <button
                                                                class="text-[10px] font-bold uppercase tracking-wider text-amber-400 hover:underline px-2 py-1 rounded hover:bg-amber-500/10 transition-colors"
                                                                title="Auto-provision mandatory links for all instances on matching tiers"
                                                                on:click=move |_| {
                                                                    provision_offer_id.set(Some(oid3.clone()));
                                                                    provision_offer_name.set(oname3.clone());
                                                                    provision_result.set(None);
                                                                }
                                                            >
                                                                "Auto-Provision"
                                                            </button>
                                                        }.into_any()
                                                    } else {
                                                        view! { <span></span> }.into_any()
                                                    }}
                                                    <button
                                                        class="text-[10px] font-bold uppercase tracking-wider text-error hover:underline px-2 py-1 rounded hover:bg-error/10 transition-colors"
                                                        on:click=move |_| hr(oid2.clone(), oname2.clone())
                                                    >
                                                        "Retire"
                                                    </button>
                                                </div>
                                            </td>
                                        </tr>
                                    }
                                }).collect_view().into_any()
                            }}
                        </Suspense>
                    </tbody>
                </table>
            </div>

            // ── Create Offer Modal ────────────────────────────────────────────
            <Show when=move || show_create.get()>
                <div class="fixed inset-0 z-[100] bg-black/80 backdrop-blur-sm flex items-center justify-center p-4">
                    <div class="bg-card w-full max-w-lg p-6 rounded-2xl border border-white/10 shadow-2xl relative space-y-4">
                        <button
                            class="absolute top-4 right-4 text-slate-400 hover:text-white"
                            on:click=move |_| show_create.set(false)
                        >"✕"</button>
                        <h3 class="text-lg font-bold text-on-surface">"New Syndication Offer"</h3>
                        <p class="text-xs text-on-surface-variant">"Define a new connection operators can activate. Mandatory tiers cannot opt out."</p>

                        <div class="space-y-3">
                            <div>
                                <label class="text-xs text-on-surface-variant font-semibold block mb-1">"Display Name *"</label>
                                <input
                                    class="w-full bg-surface-container border border-outline-variant/30 rounded-lg px-3 py-2 text-sm text-on-surface placeholder:text-on-surface-variant/40 focus:outline-none focus:border-primary/50"
                                    placeholder="e.g. \"Atlas Marketplace — LTR\""
                                    prop:value=move || form_display_name.get()
                                    on:input=move |e| form_display_name.set(event_target_value(&e))
                                />
                            </div>

                            <div>
                                <label class="text-xs text-on-surface-variant font-semibold block mb-1">"NI Config ID (UUID) *"</label>
                                <input
                                    class="w-full bg-surface-container border border-outline-variant/30 rounded-lg px-3 py-2 text-sm font-mono text-on-surface placeholder:text-on-surface-variant/40 focus:outline-none focus:border-primary/50"
                                    placeholder="atlas_app_deployment_config.id for the destination NI"
                                    prop:value=move || form_ni_config_id.get()
                                    on:input=move |e| form_ni_config_id.set(event_target_value(&e))
                                />
                            </div>

                            <div>
                                <label class="text-xs text-on-surface-variant font-semibold block mb-1">"Description"</label>
                                <textarea
                                    class="w-full bg-surface-container border border-outline-variant/30 rounded-lg px-3 py-2 text-sm text-on-surface placeholder:text-on-surface-variant/40 focus:outline-none focus:border-primary/50 resize-none"
                                    rows="2"
                                    placeholder="Shown to operators in their self-service UI"
                                    prop:value=move || form_description.get()
                                    on:input=move |e| form_description.set(event_target_value(&e))
                                />
                            </div>

                            <div>
                                <label class="text-xs text-on-surface-variant font-semibold block mb-2">"Link Type"</label>
                                <div class="flex gap-3">
                                    <label class="flex items-center gap-2 text-xs cursor-pointer">
                                        <input
                                            type="radio"
                                            name="link_type"
                                            value="marketplace_syndication"
                                            checked=move || form_link_type.get() == "marketplace_syndication"
                                            on:change=move |_| form_link_type.set("marketplace_syndication".to_string())
                                        />
                                        "Marketplace Syndication"
                                    </label>
                                    <label class="flex items-center gap-2 text-xs cursor-pointer">
                                        <input
                                            type="radio"
                                            name="link_type"
                                            value="branded_portal"
                                            checked=move || form_link_type.get() == "branded_portal"
                                            on:change=move |_| form_link_type.set("branded_portal".to_string())
                                        />
                                        "Branded Portal (1:1)"
                                    </label>
                                </div>
                            </div>

                            <div>
                                <label class="text-xs text-on-surface-variant font-semibold block mb-2">"Listing Types"</label>
                                <div class="flex flex-wrap gap-3">
                                    <label class="flex items-center gap-2 text-xs cursor-pointer">
                                        <input type="checkbox" prop:checked=move || form_ltr.get() on:change=move |e| form_ltr.set(event_target_checked(&e)) />
                                        "LTR"
                                    </label>
                                    <label class="flex items-center gap-2 text-xs cursor-pointer">
                                        <input type="checkbox" prop:checked=move || form_str.get() on:change=move |e| form_str.set(event_target_checked(&e)) />
                                        "STR"
                                    </label>
                                    <label class="flex items-center gap-2 text-xs cursor-pointer">
                                        <input type="checkbox" prop:checked=move || form_for_sale.get() on:change=move |e| form_for_sale.set(event_target_checked(&e)) />
                                        "For Sale"
                                    </label>
                                    <label class="flex items-center gap-2 text-xs cursor-pointer">
                                        <input type="checkbox" prop:checked=move || form_vendor_profile.get() on:change=move |e| form_vendor_profile.set(event_target_checked(&e)) />
                                        "Vendor Profile"
                                    </label>
                                </div>
                            </div>

                            <div>
                                <label class="text-xs text-on-surface-variant font-semibold block mb-2">
                                    "Mandatory For Billing Tiers"
                                    <span class="ml-1 text-on-surface-variant/50 font-normal">"(operators on these tiers cannot opt out)"</span>
                                </label>
                                <div class="flex gap-4">
                                    <label class="flex items-center gap-2 text-xs cursor-pointer">
                                        <input type="checkbox" prop:checked=move || form_mandatory_free.get() on:change=move |e| form_mandatory_free.set(event_target_checked(&e)) />
                                        <span class="text-amber-400 font-semibold">"free"</span>
                                    </label>
                                    <label class="flex items-center gap-2 text-xs cursor-pointer">
                                        <input type="checkbox" prop:checked=move || form_mandatory_starter.get() on:change=move |e| form_mandatory_starter.set(event_target_checked(&e)) />
                                        <span class="text-amber-400 font-semibold">"starter"</span>
                                    </label>
                                </div>
                            </div>

                            <div class="flex items-center gap-6">
                                <label class="flex items-center gap-2 text-xs cursor-pointer">
                                    <input type="checkbox" prop:checked=move || form_self_service.get() on:change=move |e| form_self_service.set(event_target_checked(&e)) />
                                    "Allow operator self-service activation"
                                </label>
                            </div>

                            <div>
                                <label class="text-xs text-on-surface-variant font-semibold block mb-1">
                                    "Restrict to Folio Mode "
                                    <span class="font-normal text-on-surface-variant/50">"(leave blank for all)"</span>
                                </label>
                                <select
                                    class="w-full bg-surface-container border border-outline-variant/30 rounded-lg px-3 py-2 text-sm text-on-surface focus:outline-none focus:border-primary/50"
                                    on:change=move |e| form_folio_mode.set(event_target_value(&e))
                                >
                                    <option value="">"All modes"</option>
                                    <option value="standard">"Standard (Landlord)"</option>
                                    <option value="pmc">"PMC (Property Manager)"</option>
                                    <option value="brokerage">"Brokerage"</option>
                                </select>
                            </div>
                        </div>

                        <div class="flex justify-end gap-3 pt-2">
                            <button
                                class="px-4 py-2 rounded-lg text-sm border border-outline-variant/30 text-on-surface-variant hover:bg-surface-bright/10 transition-colors"
                                on:click=move |_| show_create.set(false)
                            >"Cancel"</button>
                            <button
                                class="btn-primary-gradient px-5 py-2 rounded-lg text-sm font-semibold disabled:opacity-40 disabled:cursor-not-allowed"
                                disabled=move || is_saving.get()
                                on:click=handle_create.clone()
                            >
                                {move || if is_saving.get() { "Saving…" } else { "Create Offer" }}
                            </button>
                        </div>
                    </div>
                </div>
            </Show>

            // ── Auto-Provision Modal ──────────────────────────────────────────
            <Show when=move || provision_offer_id.get().is_some()>
                <div class="fixed inset-0 z-[100] bg-black/80 backdrop-blur-sm flex items-center justify-center p-4">
                    <div class="bg-card w-full max-w-md p-6 rounded-2xl border border-white/10 shadow-2xl relative space-y-4">
                        <button
                            class="absolute top-4 right-4 text-slate-400 hover:text-white"
                            on:click=move |_| { provision_offer_id.set(None); provision_result.set(None); }
                        >"✕"</button>
                        <div>
                            <h3 class="text-lg font-bold text-on-surface">"Auto-Provision Mandatory Links"</h3>
                            <p class="text-sm text-on-surface-variant mt-1">
                                {move || format!("Offer: \"{}\"", provision_offer_name.get())}
                            </p>
                        </div>
                        <div class="bg-amber-500/10 border border-amber-500/20 rounded-lg p-4 text-xs text-amber-300 leading-relaxed">
                            "This will scan all active app instances and create syndication links for any instance whose billing tier is in this offer's mandatory tier list. Existing links are skipped. This operation is safe to re-run."
                        </div>
                        {move || provision_result.get().map(|(p, s)| view! {
                            <div class="bg-emerald-500/10 border border-emerald-500/20 rounded-lg p-4 text-xs text-emerald-300">
                                <span class="font-bold">{format!("{} links created", p)}</span>
                                {format!(", {} skipped (already existed or tier mismatch).", s)}
                            </div>
                        })}
                        <div class="flex justify-end gap-3">
                            <button
                                class="px-4 py-2 rounded-lg text-sm border border-outline-variant/30 text-on-surface-variant hover:bg-surface-bright/10 transition-colors"
                                on:click=move |_| { provision_offer_id.set(None); provision_result.set(None); }
                            >"Close"</button>
                            <button
                                class="px-5 py-2 rounded-lg text-sm font-semibold bg-amber-500/20 border border-amber-500/40 text-amber-300 hover:bg-amber-500/30 transition-colors disabled:opacity-40"
                                disabled=move || is_provisioning.get()
                                on:click=handle_provision.clone()
                            >
                                {move || if is_provisioning.get() { "Provisioning…" } else { "Run Auto-Provision" }}
                            </button>
                        </div>
                    </div>
                </div>
            </Show>
        </div>
    }
}
