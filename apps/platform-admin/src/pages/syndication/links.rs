//! Syndication Active Links — platform-admin page.
//!
//! Shows all `atlas_app_instance_syndication` rows (active links).
//! Allows admin-manual revoke (non-mandatory only).
//! Route: /syndication/links

use leptos::prelude::*;

use crate::api::syndication::{list_syndication_links, revoke_syndication_link};

#[component]
pub fn SyndicationLinks() -> impl IntoView {
    let toast = use_context::<crate::app::GlobalToast>().expect("toast context");
    let links_res = LocalResource::new(|| async { list_syndication_links().await.unwrap_or_default() });

    let t1 = toast.clone();
    let handle_revoke = move |id: String, is_mandatory: bool| {
        if is_mandatory {
            t1.show_toast("Cannot Revoke", "Mandatory links cannot be revoked. Change the operator's billing tier.", "error");
            return;
        }
        let t = t1.clone();
        leptos::task::spawn_local(async move {
            match revoke_syndication_link(&id).await {
                Ok(_) => {
                    t.show_toast("Revoked", "Syndication link revoked.", "success");
                    links_res.refetch();
                }
                Err(e) => t.show_toast("Error", &e, "error"),
            }
        });
    };

    view! {
        <div class="main-canvas">
            <div class="page-header">
                <div>
                    <div class="page-title">"Syndication Links"</div>
                    <div class="page-subtitle">"All live source → network syndication relationships. Mandatory links cannot be revoked here."</div>
                </div>
                <a href="/syndication/offers" class="btn btn-ghost btn-sm">"← Offer Catalog"</a>
            </div>

            <div class="bg-surface-container-low border border-outline-variant/20 rounded-xl overflow-hidden shadow-sm">
                <table class="w-full text-left">
                    <thead>
                        <tr class="bg-surface-container-high/20 border-b border-outline-variant/10 text-[10px] uppercase tracking-wider text-on-surface-variant/70">
                            <th class="px-6 py-3 font-semibold">"Source Instance"</th>
                            <th class="px-6 py-3 font-semibold">"Network Instance"</th>
                            <th class="px-6 py-3 font-semibold">"Type"</th>
                            <th class="px-6 py-3 font-semibold">"Syndicates"</th>
                            <th class="px-6 py-3 font-semibold">"Status"</th>
                            <th class="px-6 py-3 font-semibold">"Mandatory"</th>
                            <th class="px-6 py-3 font-semibold text-right">"Actions"</th>
                        </tr>
                    </thead>
                    <tbody class="divide-y divide-outline-variant/10">
                        <Suspense fallback=move || view! {
                            <tr><td colspan="7" class="px-6 py-8 text-center text-on-surface-variant text-sm">"Loading…"</td></tr>
                        }>
                            {move || {
                                let links = links_res.get().unwrap_or_default();
                                if links.is_empty() {
                                    return view! {
                                        <tr><td colspan="7" class="px-6 py-10 text-center text-on-surface-variant/60 text-sm">
                                            "No active links. Create offers and activate them from instance settings."
                                        </td></tr>
                                    }.into_any();
                                }
                                links.into_iter().map(|link| {
                                    let lid = link.id.clone();
                                    let is_mandatory = link.is_mandatory;
                                    let hr = handle_revoke.clone();
                                    let types = link.syndication_types.as_array()
                                        .map(|arr| arr.iter().filter_map(|v| v.as_str()).collect::<Vec<_>>().join(", "))
                                        .unwrap_or_else(|| "—".to_string());
                                    view! {
                                        <tr class="hover:bg-surface-bright/5 transition-colors">
                                            <td class="px-6 py-4 font-mono text-xs text-on-surface/80">
                                                {link.source_config_id.chars().take(8).collect::<String>() + "…"}
                                            </td>
                                            <td class="px-6 py-4 font-mono text-xs text-on-surface/80">
                                                {link.ni_config_id.chars().take(8).collect::<String>() + "…"}
                                            </td>
                                            <td class="px-6 py-4">
                                                <span class=move || match link.link_type.as_str() {
                                                    "branded_portal" => "inline-flex items-center px-2 py-0.5 rounded text-[10px] font-bold bg-purple-500/10 text-purple-400 border border-purple-500/20",
                                                    _ => "inline-flex items-center px-2 py-0.5 rounded text-[10px] font-bold bg-primary/10 text-primary border border-primary/20",
                                                }>
                                                    {link.link_type.replace('_', " ")}
                                                </span>
                                            </td>
                                            <td class="px-6 py-4 text-xs text-on-surface-variant font-mono">{types}</td>
                                            <td class="px-6 py-4">
                                                <span class=move || if link.status == "active" {
                                                    "inline-flex items-center gap-1 text-[10px] font-bold text-emerald-400"
                                                } else {
                                                    "inline-flex items-center gap-1 text-[10px] font-bold text-error"
                                                }>
                                                    <span class="w-1.5 h-1.5 rounded-full bg-current"></span>
                                                    {link.status.clone()}
                                                </span>
                                            </td>
                                            <td class="px-6 py-4">
                                                {if is_mandatory {
                                                    view! { <span class="text-amber-400 text-xs font-bold">"🔒 Yes"</span> }.into_any()
                                                } else {
                                                    view! { <span class="text-on-surface-variant/40 text-xs">"No"</span> }.into_any()
                                                }}
                                            </td>
                                            <td class="px-6 py-4 text-right">
                                                <button
                                                    class=move || if is_mandatory {
                                                        "text-[10px] font-bold uppercase tracking-wider text-on-surface-variant/30 cursor-not-allowed"
                                                    } else {
                                                        "text-[10px] font-bold uppercase tracking-wider text-error hover:underline"
                                                    }
                                                    on:click=move |_| hr(lid.clone(), is_mandatory)
                                                >
                                                    "Revoke"
                                                </button>
                                            </td>
                                        </tr>
                                    }
                                }).collect_view().into_any()
                            }}
                        </Suspense>
                    </tbody>
                </table>
            </div>
        </div>
    }
}
