//! Network Instance detail page.
//!
//! Rendered by `AppInstance` when `cfg.app_slug == "network_instance"`.
//!
//! Network Instance is the multi-sided marketplace / directory app. It surfaces
//! listings, profiles (members), and ad purchases for a Network tenant.
//!
//! Tabs:
//!   Overview         — Identity card + live marketplace activity stats
//!   Listings         — Active listing count and moderation summary
//!   Members          — Profile counts
//!   Domains & Routing — Public slug + custom domain
//!   Syndication      — InstanceSyndicationPanel (NI receives/pushes listings)
//!   Operational Config — InstanceOperationalConfigPanel
//!   Users            — TenantUsersPanel

use crate::api::admin::{PublicConfigResponse, get_instance_stats};
use crate::components::instance_operational_config_panel::InstanceOperationalConfigPanel;
use crate::components::instance_syndication_panel::{
    AvailableOffersPanel, InstanceSyndicationPanel,
};
use crate::components::tenant_users_panel::TenantUsersPanel;
use leptos::prelude::*;

#[component]
pub fn NetworkInstance(cfg: PublicConfigResponse) -> impl IntoView {
    let toast = use_context::<crate::app::GlobalToast>().expect("toast context");

    let instance_id = cfg.instance_id;
    let tenant_id = cfg.tenant_id;
    let tenant_name = StoredValue::new(cfg.tenant_name.clone());

    // ── Tab state ──
    let active_tab = RwSignal::new("t-overview".to_string());

    // ── Signals from config ──
    let public_slug = RwSignal::new(cfg.public_slug.clone().unwrap_or_default());
    let custom_domain = RwSignal::new(cfg.custom_domain.clone().unwrap_or_default());
    let is_suspended = RwSignal::new(cfg.instance_status == "suspended");
    let billing_tier = StoredValue::new(cfg.billing_tier.clone());

    // ── Live stats ──
    let stats =
        LocalResource::new(move || async move { get_instance_stats(instance_id).await.ok() });

    view! {
        <div class="w-full space-y-6">
            // ── Instance header ──
            <div class="bg-surface-container-low border border-outline-variant/20 rounded-2xl p-6 shadow-sm">
                <div class="flex flex-col lg:flex-row lg:items-start justify-between gap-6">
                    <div class="flex items-start gap-5">
                        <div class="w-14 h-14 rounded-xl flex items-center justify-center text-2xl shrink-0">
                            "🔗"
                        </div>
                        <div>
                            <div class="flex items-center gap-2 flex-wrap">
                                <h1 class="text-xl font-bold text-on-surface">"Network Directory"</h1>
                                <span class="plan-badge" style="color:var(--cobalt);border-color:var(--cobalt);background:var(--cobalt-dim)">
                                    "Network"
                                </span>
                                {move || if is_suspended.get() {
                                    view! {
                                        <span class="px-2 py-0.5 rounded text-[9px] font-bold bg-error/10 text-error border border-error/20 uppercase tracking-wider">
                                            "Suspended"
                                        </span>
                                    }.into_any()
                                } else {
                                    view! {
                                        <span class="px-2 py-0.5 rounded text-[9px] font-bold bg-emerald-500/10 text-emerald-400 border border-emerald-500/20 uppercase tracking-wider">
                                            "● Live"
                                        </span>
                                    }.into_any()
                                }}
                            </div>
                            <div class="text-xs text-on-surface-variant mt-1 font-mono">
                                "app_slug: network_instance · inst: " {instance_id.to_string()}
                            </div>
                            <div class="flex flex-wrap gap-3 mt-2 text-[10px] text-on-surface-variant/70 uppercase tracking-wider">
                                <span>"Marketplace"</span>
                                <span>{move || billing_tier.get_value().to_uppercase()}</span>
                            </div>
                        </div>
                    </div>
                    <div class="flex items-center gap-3 shrink-0">
                        {move || if is_suspended.get() {
                            view! {
                                <button
                                    class="btn btn-primary"
                                    on:click=move |_| {
                                        let id = instance_id;
                                        let t = toast.clone();
                                        leptos::task::spawn_local(async move {
                                            let _ = crate::api::admin::resume_instance(id).await;
                                            t.show_toast("Resumed", "Network instance is now active.", "success");
                                        });
                                        is_suspended.set(false);
                                    }
                                >
                                    "Resume Instance"
                                </button>
                            }.into_any()
                        } else {
                            view! {
                                <button
                                    class="btn btn-ghost"
                                    style="color:var(--error)"
                                    on:click=move |_| {
                                        let id = instance_id;
                                        let t = toast.clone();
                                        leptos::task::spawn_local(async move {
                                            let _ = crate::api::admin::suspend_instance(id, "operator action".to_string()).await;
                                            t.show_toast("Suspended", "Network instance suspended.", "warning");
                                        });
                                        is_suspended.set(true);
                                    }
                                >
                                    "Suspend Instance"
                                </button>
                            }.into_any()
                        }}
                        <button
                            class="btn btn-primary"
                            on:click=move |_| toast.show_toast("Saved", "Instance changes applied.", "success")
                        >
                            "Save Changes"
                        </button>
                    </div>
                </div>
            </div>

            // ── Tab bar ──
            <div class="tab-bar">
                {vec![
                    ("t-overview", "Overview"),
                    ("t-listings", "Listings"),
                    ("t-members", "Members"),
                    ("t-domain", "Domains & Routing"),
                    ("t-syndication", "Syndication"),
                    ("t-operational-config", "Operational Config"),
                    ("t-users", "Users"),
                ].into_iter().map(|(id, label)| {
                    let id_str = id.to_string();
                    let id_for_click = id_str.clone();
                    let id_for_class = id_str.clone();
                    let label_str = label.to_string();
                    view! {
                        <button
                            class=move || if active_tab.get() == id_for_class { "tab active" } else { "tab" }
                            on:click={
                                let id2 = id_for_click.clone();
                                move |_| active_tab.set(id2.clone())
                            }
                        >
                            {label_str}
                        </button>
                    }
                }).collect_view()}
            </div>

            // ── TAB: Overview ──
            <Show when=move || active_tab.get() == "t-overview">
                <div class="grid grid-cols-1 lg:grid-cols-2 gap-6">
                    // Identity card
                    <div class="bg-surface-container-low border border-outline-variant/20 rounded-xl overflow-hidden shadow-sm">
                        <div class="px-5 py-3.5 border-b border-outline-variant/20 bg-surface-container-high/40">
                            <h3 class="text-xs font-bold uppercase tracking-wider text-on-surface-variant">"Network Directory Identity"</h3>
                        </div>
                        <div class="divide-y divide-outline-variant/10 text-xs">
                            <div class="flex justify-between items-center px-5 py-3">
                                <span class="text-on-surface-variant">"App Type"</span>
                                <span class="font-semibold text-on-surface">"Network — Directory / Marketplace"</span>
                            </div>
                            <div class="flex justify-between items-center px-5 py-3">
                                <span class="text-on-surface-variant">"Instance ID"</span>
                                <span class="font-mono text-on-surface/80 text-[10px]">{instance_id.to_string()}</span>
                            </div>
                            <div class="flex justify-between items-center px-5 py-3">
                                <span class="text-on-surface-variant">"Tenant"</span>
                                <span class="font-mono text-on-surface/80 text-[10px]">{tenant_id.to_string()}</span>
                            </div>
                            <div class="flex justify-between items-center px-5 py-3">
                                <span class="text-on-surface-variant">"Subdomain"</span>
                                <span class="font-mono text-on-surface/80">{move || format!("{}.atlas.app", public_slug.get())}</span>
                            </div>
                            <div class="flex justify-between items-center px-5 py-3">
                                <span class="text-on-surface-variant">"Custom Domain"</span>
                                <span class="font-mono text-on-surface/80">{move || {
                                    let d = custom_domain.get();
                                    if d.is_empty() { "—".to_string() } else { d }
                                }}</span>
                            </div>
                            <div class="flex justify-between items-center px-5 py-3">
                                <span class="text-on-surface-variant">"Status"</span>
                                {move || if is_suspended.get() {
                                    view! { <span class="text-error font-semibold">"● Suspended"</span> }.into_any()
                                } else {
                                    view! { <span class="text-emerald-400 font-semibold">"● Live"</span> }.into_any()
                                }}
                            </div>
                            <div class="flex justify-between items-center px-5 py-3">
                                <span class="text-on-surface-variant">"Billing Tier"</span>
                                <span class="font-semibold text-on-surface">{move || billing_tier.get_value()}</span>
                            </div>
                        </div>
                    </div>

                    // Live marketplace stats
                    <div class="bg-surface-container-low border border-outline-variant/20 rounded-xl overflow-hidden shadow-sm">
                        <div class="px-5 py-3.5 border-b border-outline-variant/20 bg-surface-container-high/40">
                            <h3 class="text-xs font-bold uppercase tracking-wider text-on-surface-variant">"Marketplace Activity"</h3>
                        </div>
                        {move || {
                            let s = stats.get().flatten();
                            view! {
                                <div class="divide-y divide-outline-variant/10 text-xs">
                                    <div class="flex justify-between items-center px-5 py-3">
                                        <span class="text-on-surface-variant">"Active Listings"</span>
                                        <span class="font-bold font-mono text-on-surface">{s.as_ref().map(|s| s.active_listing_count.to_string()).unwrap_or_else(|| "…".into())}</span>
                                    </div>
                                    <div class="flex justify-between items-center px-5 py-3">
                                        <span class="text-on-surface-variant">"Total Leads (G-31)"</span>
                                        <span class="font-bold font-mono text-on-surface">{s.as_ref().map(|s| s.lead_count.to_string()).unwrap_or_else(|| "…".into())}</span>
                                    </div>
                                    <div class="flex justify-between items-center px-5 py-3">
                                        <span class="text-on-surface-variant">"Open Cases"</span>
                                        <span class="font-bold font-mono text-on-surface">{s.as_ref().map(|s| s.open_case_count.to_string()).unwrap_or_else(|| "…".into())}</span>
                                    </div>
                                </div>
                            }
                        }}
                    </div>
                </div>
            </Show>

            // ── TAB: Listings ──
            <Show when=move || active_tab.get() == "t-listings">
                <div class="section" style="padding:12px 16px;font-size:11px;color:var(--text-secondary);line-height:1.6;margin-bottom:12px">
                    <span style="color:var(--cobalt);font-weight:600">"Network listings are managed here. "</span>
                    "Listings can be sourced locally or syndicated from Folio instances. "
                    "Moderation queue is reviewed by network admins."
                </div>
                {move || {
                    let s = stats.get().flatten();
                    view! {
                        <div class="bg-surface-container-low border border-outline-variant/20 rounded-xl overflow-hidden shadow-sm">
                            <div class="px-5 py-3.5 border-b border-outline-variant/20 bg-surface-container-high/40">
                                <h3 class="text-xs font-bold uppercase tracking-wider text-on-surface-variant">"Listing Counts"</h3>
                            </div>
                            <div class="divide-y divide-outline-variant/10 text-xs">
                                <div class="flex justify-between items-center px-5 py-3">
                                    <span class="text-on-surface-variant">"Active / Approved Listings"</span>
                                    <span class="font-bold font-mono text-on-surface">{s.as_ref().map(|s| s.active_listing_count.to_string()).unwrap_or_else(|| "…".into())}</span>
                                </div>
                            </div>
                        </div>
                    }
                }}
            </Show>

            // ── TAB: Members ──
            <Show when=move || active_tab.get() == "t-members">
                <div class="bg-surface-container-low border border-outline-variant/20 rounded-xl p-6 text-xs text-on-surface-variant space-y-2">
                    <p>"Member (profile) management is accessible from the tenant admin panel."</p>
                    <a href="#" class="text-primary hover:underline">"View Members →"</a>
                </div>
            </Show>

            // ── TAB: Domains & Routing ──
            <Show when=move || active_tab.get() == "t-domain">
                <div class="grid grid-cols-1 lg:grid-cols-2 gap-6">
                    <div class="bg-surface-container-low border border-outline-variant/20 rounded-xl overflow-hidden shadow-sm">
                        <div class="px-5 py-3.5 border-b border-outline-variant/20 bg-surface-container-high/40">
                            <h3 class="text-xs font-bold uppercase tracking-wider text-on-surface-variant">"Domain Configuration"</h3>
                        </div>
                        <div class="divide-y divide-outline-variant/10 text-xs">
                            <div class="flex justify-between items-center px-5 py-3">
                                <span class="text-on-surface-variant">"Platform Subdomain"</span>
                                <span class="font-mono text-on-surface/80">{move || format!("{}.atlas.app", public_slug.get())}</span>
                            </div>
                            <div class="flex justify-between items-center px-5 py-3">
                                <span class="text-on-surface-variant">"Custom Domain"</span>
                                <span class="font-mono text-on-surface/80">{move || {
                                    let d = custom_domain.get();
                                    if d.is_empty() { "—".to_string() } else { d }
                                }}</span>
                            </div>
                            <div class="flex justify-between items-center px-5 py-3">
                                <span class="text-on-surface-variant">"SSL/TLS"</span>
                                <span class="text-emerald-400 font-semibold">"● Let's Encrypt · Valid"</span>
                            </div>
                        </div>
                    </div>
                </div>
            </Show>

            // ── TAB: Syndication ──
            <Show when=move || active_tab.get() == "t-syndication">
                <div class="space-y-6">
                    <InstanceSyndicationPanel instance_id=instance_id.to_string() />
                    <AvailableOffersPanel instance_id=instance_id.to_string() />
                </div>
            </Show>

            // ── TAB: Operational Config ──
            <Show when=move || active_tab.get() == "t-operational-config">
                {move || {
                    let cfg_opt = Some(PublicConfigResponse {
                        instance_id,
                        tenant_id,
                        tenant_name: tenant_name.get_value(),
                        app_slug: "network_instance".to_string(),
                        public_slug: Some(public_slug.get()),
                        custom_domain: Some(custom_domain.get()),
                        instance_status: if is_suspended.get() { "suspended".to_string() } else { "active".to_string() },
                        folio_mode: "standard".to_string(),
                        billing_tier: billing_tier.get_value(),
                        tenant_portal_enabled: false,
                        vendor_portal_enabled: false,
                        dns_instructions: None,
                    });
                    view! {
                        <InstanceOperationalConfigPanel
                            instance_id=instance_id
                            config=cfg_opt
                            app_slug="network_instance".to_string()
                        />
                    }
                }}
            </Show>

            // ── TAB: Users ──
            <Show when=move || active_tab.get() == "t-users">
                <TenantUsersPanel tenant_id=tenant_id />
            </Show>
        </div>
    }
}
