//! InstanceSyndicationPanel — shows active syndication links for a specific
//! app instance and a list of available self-service offers to activate.
//! Used on the Syndication tab of the AppInstance detail page.

use leptos::prelude::*;

use crate::api::syndication::{
    list_syndication_links, list_syndication_offers, revoke_syndication_link,
    create_syndication_link, CreateLinkInput,
};

// ── Active links for this specific instance ───────────────────────────────────

#[component]
pub fn InstanceSyndicationPanel(instance_id: String) -> impl IntoView {
    let iid = instance_id.clone();
    let toast = use_context::<crate::app::GlobalToast>().expect("toast context");

    let links_res = LocalResource::new({
        let iid = iid.clone();
        move || {
            let iid = iid.clone();
            async move {
                list_syndication_links()
                    .await
                    .unwrap_or_default()
                    .into_iter()
                    .filter(|l| l.source_config_id == iid)
                    .collect::<Vec<_>>()
            }
        }
    });

    let t1 = toast.clone();
    let handle_revoke = move |id: String, is_mandatory: bool| {
        if is_mandatory {
            t1.show_toast("Cannot Revoke", "Mandatory links cannot be revoked on subsidised tiers.", "error");
            return;
        }
        let t = t1.clone();
        leptos::task::spawn_local(async move {
            match revoke_syndication_link(&id).await {
                Ok(_) => {
                    t.show_toast("Revoked", "Syndication link deactivated.", "success");
                    links_res.refetch();
                }
                Err(e) => t.show_toast("Error", &e, "error"),
            }
        });
    };

    view! {
        <div>
            <Suspense fallback=move || view! {
                <div class="px-6 py-8 text-center text-on-surface-variant text-sm">"Loading links…"</div>
            }>
                {move || {
                    let links = links_res.get().unwrap_or_default();
                    if links.is_empty() {
                        return view! {
                            <div class="px-6 py-8 text-center text-on-surface-variant/60 text-sm">
                                "No active syndication links. Activate an offer below or ask a platform admin to provision one."
                            </div>
                        }.into_any();
                    }
                    view! {
                        <table class="w-full text-left">
                            <thead>
                                <tr class="bg-surface-container-high/20 text-[10px] uppercase tracking-wider text-on-surface-variant/70">
                                    <th class="px-5 py-3 font-semibold">"Network Instance"</th>
                                    <th class="px-5 py-3 font-semibold">"Type"</th>
                                    <th class="px-5 py-3 font-semibold">"Syndicates"</th>
                                    <th class="px-5 py-3 font-semibold">"Mandatory"</th>
                                    <th class="px-5 py-3 font-semibold text-right">"Actions"</th>
                                </tr>
                            </thead>
                            <tbody class="divide-y divide-outline-variant/10">
                                {links.into_iter().map(|link| {
                                    // Pre-clone to avoid partial-move in multiple move closures
                                    let lid = link.id.clone();
                                    let is_mandatory = link.is_mandatory;
                                    let hr = handle_revoke.clone();
                                    let types = link.syndication_types.as_array()
                                        .map(|arr| arr.iter().filter_map(|v| v.as_str()).collect::<Vec<_>>().join(", "))
                                        .unwrap_or_else(|| "—".to_string());
                                    let ni_prefix = link.ni_config_id.chars().take(8).collect::<String>() + "…";
                                    let link_type = link.link_type.clone();
                                    let link_type_label = link.link_type.replace('_', " ");
                                    view! {
                                        <tr class="hover:bg-surface-bright/5 transition-colors">
                                            <td class="px-5 py-3 font-mono text-xs text-on-surface/80">
                                                {ni_prefix}
                                            </td>
                                            <td class="px-5 py-3">
                                                <span class=move || match link_type.as_str() {
                                                    "branded_portal" => "inline-flex px-2 py-0.5 rounded text-[10px] font-bold bg-purple-500/10 text-purple-400 border border-purple-500/20",
                                                    _ => "inline-flex px-2 py-0.5 rounded text-[10px] font-bold bg-primary/10 text-primary border border-primary/20",
                                                }>
                                                    {link_type_label}
                                                </span>
                                            </td>
                                            <td class="px-5 py-3 text-xs text-on-surface-variant font-mono">{types}</td>
                                            <td class="px-5 py-3">
                                                {if is_mandatory {
                                                    view! { <span class="text-amber-400 text-xs font-bold">"🔒 Yes"</span> }.into_any()
                                                } else {
                                                    view! { <span class="text-on-surface-variant/40 text-xs">"No"</span> }.into_any()
                                                }}
                                            </td>
                                            <td class="px-5 py-3 text-right">
                                                <button
                                                    class=move || if is_mandatory {
                                                        "text-[10px] font-bold uppercase text-on-surface-variant/30 cursor-not-allowed"
                                                    } else {
                                                        "text-[10px] font-bold uppercase text-error hover:underline"
                                                    }
                                                    on:click=move |_| hr(lid.clone(), is_mandatory)
                                                >
                                                    "Deactivate"
                                                </button>
                                            </td>
                                        </tr>
                                    }
                                }).collect_view()}
                            </tbody>
                        </table>
                    }.into_any()
                }}
            </Suspense>
        </div>
    }
}

// ── Self-service offer activations ────────────────────────────────────────────

#[component]
pub fn AvailableOffersPanel(instance_id: String) -> impl IntoView {
    let iid = instance_id.clone();
    let toast = use_context::<crate::app::GlobalToast>().expect("toast context");

    let offers_res = LocalResource::new(move || async {
        list_syndication_offers().await.unwrap_or_default()
            .into_iter()
            .filter(|o| o.self_service_allowed && !o.is_retired())
            .collect::<Vec<_>>()
    });

    let activating = RwSignal::new(Option::<String>::None);

    let t1 = toast.clone();
    let activate = move |offer_id: String, ni_config_id: String, instance_id: String| {
        let t = t1.clone();
        activating.set(Some(offer_id.clone()));
        leptos::task::spawn_local(async move {
            let input = CreateLinkInput {
                source_config_id: instance_id.clone(),
                ni_config_id: ni_config_id.clone(),
                offer_id: Some(offer_id),
                syndication_types: None,
                link_type: None,
                inbound_webhook_url: None,
                created_by_tenant_id: instance_id,
            };
            match create_syndication_link(input).await {
                Ok(_) => t.show_toast("Activated", "Syndication link activated.", "success"),
                Err(e) => t.show_toast("Error", &e, "error"),
            }
            activating.set(None);
        });
    };

    view! {
        <Suspense fallback=move || view! {
            <p class="text-xs text-on-surface-variant">"Loading offers…"</p>
        }>
            {move || {
                let offers = offers_res.get().unwrap_or_default();
                if offers.is_empty() {
                    return view! {
                        <p class="text-xs text-on-surface-variant/60">
                            "No self-service offers available. Platform admin can create offers from the Offer Catalog."
                        </p>
                    }.into_any();
                }
                view! {
                    <div class="space-y-3">
                        {offers.into_iter().map(|offer| {
                            // Pre-clone all fields consumed by multiple move closures
                            let oid = offer.id.clone();
                            let oid_activating = oid.clone();
                            let ni_cid = offer.ni_config_id.clone();
                            let iid2 = iid.clone();
                            let activate2 = activate.clone();
                            let display_name = offer.display_name.clone();
                            let description = offer.description.clone().unwrap_or_default();
                            let link_type = offer.link_type.clone();
                            let link_label = offer.link_type_label().to_string();
                            let types_display = offer.types_display();
                            let is_activating = {
                                let oid2 = oid.clone();
                                move || activating.get().as_deref() == Some(&oid2)
                            };
                            view! {
                                <div class="flex items-center justify-between border border-outline-variant/20 rounded-xl p-4 hover:bg-surface-bright/5 transition-colors">
                                    <div>
                                        <div class="text-sm font-semibold text-on-surface">{display_name}</div>
                                        <div class="text-xs text-on-surface-variant mt-0.5">
                                            {description}
                                        </div>
                                        <div class="flex gap-2 mt-2">
                                            <span class=move || match link_type.as_str() {
                                                "branded_portal" => "inline-flex px-2 py-0.5 rounded text-[10px] font-bold bg-purple-500/10 text-purple-400 border border-purple-500/20",
                                                _ => "inline-flex px-2 py-0.5 rounded text-[10px] font-bold bg-primary/10 text-primary border border-primary/20",
                                            }>
                                                {link_label}
                                            </span>
                                            <span class="text-[10px] text-on-surface-variant/60 font-mono">
                                                {types_display}
                                            </span>
                                        </div>
                                    </div>
                                    <button
                                        class="ml-4 shrink-0 px-4 py-1.5 rounded-lg text-xs font-semibold bg-primary/15 border border-primary/30 text-primary hover:bg-primary/25 transition-colors disabled:opacity-40"
                                        disabled=is_activating
                                        on:click=move |_| activate2(oid.clone(), ni_cid.clone(), iid2.clone())
                                    >
                                        {move || if activating.get().as_deref() == Some(&oid_activating) { "Activating…" } else { "Activate" }}
                                    </button>
                                </div>
                            }
                        }).collect_view()}
                    </div>
                }.into_any()
            }}
        </Suspense>
    }
}
