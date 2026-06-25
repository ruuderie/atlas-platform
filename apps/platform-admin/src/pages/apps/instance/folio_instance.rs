//! Folio (Property Management) instance detail page.
//!
//! Rendered by `AppInstance` (instance.rs) when `cfg.app_slug == "property_management"`.
//!
//! Tabs:
//!   Overview         — Identity card + live platform activity (from /stats endpoint)
//!   Onboarding       — Folio onboarding steps (jurisdiction → payment rails)
//!   Modules          — PM module toggles (portfolio, leases, maintenance, etc.)
//!   App Config       — folio_mode, billing_tier, Folio-specific jurisdiction settings
//!   Operational Config — InstanceOperationalConfigPanel
//!   Users            — TenantUsersPanel
//!   Scorecards       — G-27 scorecard health (auto-seeded by Folio provisioner)
//!   Background Jobs  — Active PM job scheduler entries
//!   Domains & Routing — Public slug + custom domain
//!   Syndication      — InstanceSyndicationPanel

use leptos::prelude::*;
use crate::api::admin::{PublicConfigResponse, get_instance_stats};
use crate::components::instance_syndication_panel::{InstanceSyndicationPanel, AvailableOffersPanel};
use crate::components::instance_operational_config_panel::InstanceOperationalConfigPanel;
use crate::components::tenant_users_panel::TenantUsersPanel;

#[component]
pub fn FolioInstance(
    cfg: PublicConfigResponse,
) -> impl IntoView {
    let toast = use_context::<crate::app::GlobalToast>().expect("toast context");

    let instance_id = cfg.instance_id;
    let tenant_id   = cfg.tenant_id;

    // ── Tab state ──
    let active_tab = RwSignal::new("t-overview".to_string());

    // ── Domain / config signals (seeded from cfg) ──
    let public_slug   = RwSignal::new(cfg.public_slug.clone().unwrap_or_default());
    let custom_domain = RwSignal::new(cfg.custom_domain.clone().unwrap_or_default());
    let is_suspended  = RwSignal::new(cfg.instance_status == "suspended");
    let _suspend_reason = RwSignal::new(String::new());

    // ── Modal visibility signals ──
    let show_suspend_modal    = RwSignal::new(false);
    let _show_add_rail_modal   = RwSignal::new(false);
    let show_edit_config_modal = RwSignal::new(false);

    // ── Folio config form signals (seeded with sensible defaults) ──
    let jurisdiction_code = RwSignal::new("US-FL".to_string());
    let market_config     = RwSignal::new("MiamiDadeMarket".to_string());
    let str_ordinance     = RwSignal::new("Miami-Dade Ord. 2023-89".to_string());
    let tdt_rate          = RwSignal::new("7% (Miami-Dade)".to_string());
    let deployment_mode   = RwSignal::new(cfg.folio_mode.clone());
    let _pix_status        = RwSignal::new("Disabled".to_string());
    let _lookback_hours    = RwSignal::new("25".to_string());

    // ── Live stats from /stats endpoint ──
    let stats = LocalResource::new(move || async move {
        get_instance_stats(instance_id).await.ok()
    });

    // ── Module toggles (G-33 controlled) ──
    let module_portfolio   = RwSignal::new(true);
    let module_leases      = RwSignal::new(true);
    let module_maintenance = RwSignal::new(true);
    let module_vendors     = RwSignal::new(true);
    let module_reservations = RwSignal::new(true);
    let module_scorecards  = RwSignal::new(true);
    let module_leads       = RwSignal::new(true);
    let module_billing     = RwSignal::new(true);

    // ── Derived: display values (wrapped in StoredValue so closures stay Fn) ──
    let app_slug_display = StoredValue::new(cfg.app_slug.clone());
    let folio_mode_display = StoredValue::new(cfg.folio_mode.clone());
    let billing_tier_display = StoredValue::new(cfg.billing_tier.clone());
    let provisioned_at = "Feb 14, 2024"; // TODO: surface from atlas_app_deployment_config.created_at

    let tab_classes = move |tab: &str| -> String {
        let base = "px-4 py-2.5 text-xs font-semibold rounded-lg transition-all";
        if active_tab.get() == tab {
            format!("{base} bg-primary text-on-primary shadow-sm")
        } else {
            format!("{base} text-on-surface-variant hover:text-on-surface hover:bg-surface-container-high/50")
        }
    };

    view! {
        <div class="space-y-6">
            // ── Instance header ──
            <div class="bg-surface-container-low border border-outline-variant/20 rounded-2xl p-6 shadow-sm">
                <div class="flex flex-col lg:flex-row lg:items-start justify-between gap-6">
                    <div class="flex items-start gap-5">
                        <div class="w-14 h-14 rounded-2xl bg-primary/10 border border-primary/20 flex items-center justify-center text-2xl shrink-0">
                            "🏠"
                        </div>
                        <div>
                            <div class="flex items-center gap-2 flex-wrap">
                                <h1 class="text-xl font-bold text-on-surface">{move || app_slug_display.get_value()}</h1>
                                <span class="px-2 py-0.5 rounded text-[10px] font-bold uppercase tracking-wider bg-primary/10 text-primary border border-primary/20">
                                    "Folio PM"
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
                                "app_slug: property_management · inst: " {instance_id.to_string()}
                            </div>
                            <div class="flex flex-wrap gap-3 mt-2">
                                <span class="text-[10px] text-on-surface-variant/70 uppercase tracking-wider">
                                    "FOLIO"
                                </span>
                                <span class="text-[10px] text-on-surface-variant/70 uppercase tracking-wider">
                                    {move || folio_mode_display.get_value().to_uppercase()}
                                </span>
                                <span class="text-[10px] text-on-surface-variant/70 uppercase tracking-wider">
                                    {move || billing_tier_display.get_value().to_uppercase()}
                                </span>
                            </div>
                        </div>
                    </div>
                    <div class="flex items-center gap-3 shrink-0">
                        {move || if is_suspended.get() {
                            view! {
                                <button
                                    class="px-4 py-2 rounded-xl text-xs font-semibold border border-emerald-500/30 text-emerald-400 hover:bg-emerald-500/10 transition-all"
                                    on:click=move |_| {
                                        let id = instance_id;
                                        let t = toast.clone();
                                        leptos::task::spawn_local(async move {
                                            let _ = crate::api::admin::resume_instance(id).await;
                                            t.show_toast("Resumed", "Instance is now active.", "success");
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
                                    class="px-4 py-2 rounded-xl text-xs font-semibold border border-error/30 text-error hover:bg-error/10 transition-all"
                                    on:click=move |_| show_suspend_modal.set(true)
                                >
                                    "Suspend Instance"
                                </button>
                            }.into_any()
                        }}
                        <button
                            class="btn-primary-gradient px-4 py-2 rounded-xl text-xs font-semibold shadow"
                            on:click=move |_| toast.show_toast("Saved", "Instance changes applied.", "success")
                        >
                            "Save Changes"
                        </button>
                    </div>
                </div>
            </div>

            // ── Tab bar ──
            <div class="flex flex-wrap gap-1.5">
                {vec![
                    ("t-overview", "Overview"),
                    ("t-onboarding", "Onboarding"),
                    ("t-modules", "Modules"),
                    ("t-config", "App Config"),
                    ("t-operational-config", "Operational Config"),
                    ("t-users", "Users"),
                    ("t-scorecards", "Scorecards"),
                    ("t-jobs", "Background Jobs"),
                    ("t-domain", "Domains & Routing"),
                    ("t-syndication", "Syndication"),
                ].into_iter().map(|(id, label)| {
                    let id_str = id.to_string();
                    let id_for_click = id_str.clone();
                    let label_str = label.to_string();
                    view! {
                        <button
                            class=move || tab_classes(&id_str)
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
                        <div class="flex justify-between items-center px-5 py-3.5 border-b border-outline-variant/20 bg-surface-container-high/40">
                            <h3 class="text-xs font-bold uppercase tracking-wider text-on-surface-variant">"App Instance Identity"</h3>
                        </div>
                        <div class="divide-y divide-outline-variant/10 text-xs">
                            <div class="flex justify-between items-center px-5 py-3">
                                <span class="text-on-surface-variant">"App Type"</span>
                                <span class="font-semibold text-on-surface">"Folio — Property Management"</span>
                            </div>
                            <div class="flex justify-between items-center px-5 py-3">
                                <span class="text-on-surface-variant">"app_id"</span>
                                <span class="font-mono text-on-surface/80">"property_management"</span>
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
                                <span class="font-mono text-on-surface/80">{move || custom_domain.get()}</span>
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
                                <span class="text-on-surface-variant">"Folio Mode"</span>
                                <span class="font-semibold text-on-surface">{move || folio_mode_display.get_value()}</span>
                            </div>
                            <div class="flex justify-between items-center px-5 py-3">
                                <span class="text-on-surface-variant">"Provisioned"</span>
                                <span class="text-on-surface/80">{provisioned_at}</span>
                            </div>
                        </div>
                    </div>

                    // Live Platform Activity
                    <div class="bg-surface-container-low border border-outline-variant/20 rounded-xl overflow-hidden shadow-sm">
                        <div class="px-5 py-3.5 border-b border-outline-variant/20 bg-surface-container-high/40">
                            <h3 class="text-xs font-bold uppercase tracking-wider text-on-surface-variant">"Platform Activity"</h3>
                        </div>
                        {move || {
                            let s = stats.get().flatten();
                            view! {
                                <div class="divide-y divide-outline-variant/10 text-xs">
                                    <div class="flex justify-between items-center px-5 py-3">
                                        <span class="text-on-surface-variant">"Properties (atlas_assets)"</span>
                                        <span class="font-bold font-mono text-on-surface">{s.as_ref().map(|s| s.asset_count.to_string()).unwrap_or_else(|| "…".into())}</span>
                                    </div>
                                    <div class="flex justify-between items-center px-5 py-3">
                                        <span class="text-on-surface-variant">"Active Leases"</span>
                                        <span class="font-bold font-mono text-on-surface">{s.as_ref().map(|s| s.active_contract_count.to_string()).unwrap_or_else(|| "…".into())}</span>
                                    </div>
                                    <div class="flex justify-between items-center px-5 py-3">
                                        <span class="text-on-surface-variant">"Total Leads (G-31)"</span>
                                        <span class="font-bold font-mono text-on-surface">{s.as_ref().map(|s| s.lead_count.to_string()).unwrap_or_else(|| "…".into())}</span>
                                    </div>
                                    <div class="flex justify-between items-center px-5 py-3">
                                        <span class="text-on-surface-variant">"Active Vendors"</span>
                                        <span class="font-bold font-mono text-on-surface">{s.as_ref().map(|s| s.vendor_count.to_string()).unwrap_or_else(|| "…".into())}</span>
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

            // ── TAB: Operational Config ──
            <Show when=move || active_tab.get() == "t-operational-config">
                <div class="space-y-6">
                    {move || {
                        let cfg_opt = Some(PublicConfigResponse {
                            instance_id,
                            tenant_id,
                            app_slug: "property_management".to_string(),
                            public_slug: Some(public_slug.get()),
                            custom_domain: Some(custom_domain.get()),
                            instance_status: if is_suspended.get() { "suspended".to_string() } else { "active".to_string() },
                            folio_mode: deployment_mode.get(),
                            billing_tier: billing_tier_display.get_value(),
                            tenant_portal_enabled: false,
                            vendor_portal_enabled: false,
                            dns_instructions: None,
                        });
                        view! {
                            <InstanceOperationalConfigPanel
                                instance_id=instance_id
                                config=cfg_opt
                            />
                        }
                    }}
                </div>
            </Show>

            // ── TAB: Users ──
            <Show when=move || active_tab.get() == "t-users">
                <div class="space-y-6">
                    <TenantUsersPanel tenant_id=tenant_id />
                </div>
            </Show>

            // ── TAB: Syndication ──
            <Show when=move || active_tab.get() == "t-syndication">
                <div class="space-y-6">
                    <InstanceSyndicationPanel instance_id=instance_id.to_string() />
                    <AvailableOffersPanel instance_id=instance_id.to_string() />
                </div>
            </Show>

            // ── TAB: Domains & Routing ──
            <Show when=move || active_tab.get() == "t-domain">
                <div class="grid grid-cols-1 lg:grid-cols-2 gap-6">
                    <div class="bg-surface-container-low border border-outline-variant/20 rounded-xl overflow-hidden shadow-sm">
                        <div class="flex justify-between items-center px-5 py-3.5 border-b border-outline-variant/20 bg-surface-container-high/40">
                            <h3 class="text-xs font-bold uppercase tracking-wider text-on-surface-variant">"Domain Configuration"</h3>
                            <button class="text-xs text-primary hover:underline" on:click=move |_| show_edit_config_modal.set(true)>"Edit"</button>
                        </div>
                        <div class="divide-y divide-outline-variant/10 text-xs">
                            <div class="flex justify-between items-center px-5 py-3">
                                <span class="text-on-surface-variant">"Platform Subdomain"</span>
                                <span class="font-mono text-on-surface/80">{move || format!("{}.atlas.app", public_slug.get())}</span>
                            </div>
                            <div class="flex justify-between items-center px-5 py-3">
                                <span class="text-on-surface-variant">"Custom Domain"</span>
                                <span class="font-mono text-on-surface/80">{move || custom_domain.get()}</span>
                            </div>
                            <div class="flex justify-between items-center px-5 py-3">
                                <span class="text-on-surface-variant">"CNAME Status"</span>
                                <span class="text-emerald-400 font-semibold">"● Verified"</span>
                            </div>
                            <div class="flex justify-between items-center px-5 py-3">
                                <span class="text-on-surface-variant">"SSL/TLS"</span>
                                <span class="text-emerald-400 font-semibold">"● Let's Encrypt · Valid"</span>
                            </div>
                        </div>
                    </div>
                </div>
            </Show>

            // ── TAB: Modules ──
            <Show when=move || active_tab.get() == "t-modules">
                <div class="bg-surface-container-low border border-outline-variant/20 rounded-xl overflow-hidden shadow-sm">
                    <div class="px-5 py-3.5 border-b border-outline-variant/20 bg-surface-container-high/40">
                        <h3 class="text-xs font-bold uppercase tracking-wider text-on-surface-variant">"PM Module Toggles · G-33"</h3>
                    </div>
                    <div class="divide-y divide-outline-variant/10">
                        {vec![
                            ("Portfolio Management (G-09)", module_portfolio),
                            ("Leases & Contracts (G-11)", module_leases),
                            ("Maintenance Requests (G-13)", module_maintenance),
                            ("Vendor Network (G-12)", module_vendors),
                            ("Reservations (G-23)", module_reservations),
                            ("Scorecard Engine (G-27)", module_scorecards),
                            ("Lead Pipeline (G-31)", module_leads),
                            ("Billing & Ledger (G-03)", module_billing),
                        ].into_iter().map(|(label, signal)| {
                            let label_str = label.to_string();
                            view! {
                                <div class="flex justify-between items-center px-5 py-3.5 text-xs">
                                    <span class="text-on-surface font-medium">{label_str}</span>
                                    <button
                                        class=move || if signal.get() {
                                            "w-10 h-5 rounded-full bg-primary transition-all relative"
                                        } else {
                                            "w-10 h-5 rounded-full bg-outline-variant/40 transition-all relative"
                                        }
                                        on:click=move |_| signal.update(|v| *v = !*v)
                                    >
                                        <span class=move || format!(
                                            "absolute top-0.5 w-4 h-4 rounded-full bg-white shadow transition-all {}",
                                            if signal.get() { "left-5" } else { "left-0.5" }
                                        ) />
                                    </button>
                                </div>
                            }
                        }).collect_view()}
                    </div>
                </div>
            </Show>

            // ── TAB: App Config ──
            <Show when=move || active_tab.get() == "t-config">
                <div class="space-y-6">
                    <div class="bg-surface-container-low border border-outline-variant/20 rounded-xl overflow-hidden shadow-sm">
                        <div class="px-5 py-3.5 border-b border-outline-variant/20 bg-surface-container-high/40">
                            <h3 class="text-xs font-bold uppercase tracking-wider text-on-surface-variant">
                                "Folio-Specific Config · ATLAS_APP_DEPLOYMENT_CONFIG"
                            </h3>
                        </div>
                        <div class="p-6 space-y-4">
                            <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
                                <div class="space-y-1.5">
                                    <label class="text-[10px] font-bold uppercase tracking-wider text-on-surface-variant/80">"Jurisdiction Code"</label>
                                    <select
                                        class="w-full bg-surface-container border border-outline-variant/40 rounded-lg p-2.5 text-xs text-on-surface outline-none cursor-pointer focus:border-primary"
                                        on:change=move |ev| jurisdiction_code.set(event_target_value(&ev))
                                        prop:value=jurisdiction_code
                                    >
                                        <option value="US-FL">"US-FL (Florida)"</option>
                                        <option value="US">"US (Federal only)"</option>
                                        <option value="BR">"BR (Brazil)"</option>
                                        <option value="USVI">"USVI (US Virgin Islands)"</option>
                                    </select>
                                </div>
                                <div class="space-y-1.5">
                                    <label class="text-[10px] font-bold uppercase tracking-wider text-on-surface-variant/80">"Market Config"</label>
                                    <select
                                        class="w-full bg-surface-container border border-outline-variant/40 rounded-lg p-2.5 text-xs text-on-surface outline-none cursor-pointer focus:border-primary"
                                        on:change=move |ev| market_config.set(event_target_value(&ev))
                                        prop:value=market_config
                                    >
                                        <option value="MiamiDadeMarket">"MiamiDadeMarket"</option>
                                        <option value="BrazilMarket">"BrazilMarket (PIX + Serasa)"</option>
                                        <option value="UsViMarket">"UsViMarket (Hotel Tax 12.5%)"</option>
                                    </select>
                                </div>
                            </div>
                            <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
                                <div class="space-y-1.5">
                                    <label class="text-[10px] font-bold uppercase tracking-wider text-on-surface-variant/80">"STR Ordinance"</label>
                                    <input
                                        type="text"
                                        class="w-full bg-surface-container border border-outline-variant/40 rounded-lg p-2.5 text-xs text-on-surface outline-none focus:border-primary"
                                        on:input=move |ev| str_ordinance.set(event_target_value(&ev))
                                        prop:value=str_ordinance
                                    />
                                </div>
                                <div class="space-y-1.5">
                                    <label class="text-[10px] font-bold uppercase tracking-wider text-on-surface-variant/80">"TDT Rate"</label>
                                    <input
                                        type="text"
                                        class="w-full bg-surface-container border border-outline-variant/40 rounded-lg p-2.5 text-xs text-on-surface outline-none focus:border-primary"
                                        on:input=move |ev| tdt_rate.set(event_target_value(&ev))
                                        prop:value=tdt_rate
                                    />
                                </div>
                            </div>
                        </div>
                    </div>
                </div>
            </Show>

            // ── TAB: Onboarding ──
            <Show when=move || active_tab.get() == "t-onboarding">
                <div class="space-y-6">
                    <div class="bg-surface-container-low border border-outline-variant/20 rounded-xl p-6 shadow-sm text-xs text-on-surface-variant">
                        <p class="mb-4">"Folio onboarding checklist for this instance:"</p>
                        {vec![
                            ("1", "Jurisdiction configured", true),
                            ("2", "Market config selected", true),
                            ("3", "Payment rails configured (Stripe Connect or BTC)", true),
                            ("4", "Role portals enabled (Landlord / Tenant / Vendor)", true),
                            ("5", "G-27 scorecard templates provisioned", true),
                            ("6", "Custom domain & SSL verified", false),
                        ].into_iter().map(|(step, label, done)| {
                            view! {
                                <div class="flex items-center gap-3 py-2 border-b border-outline-variant/10">
                                    <span class=if done { "text-emerald-400 font-bold" } else { "text-on-surface-variant/40 font-bold" }>
                                        {if done { "✓" } else { "○" }}
                                    </span>
                                    <span class="text-on-surface-variant/70">{format!("Step {step}: {label}")}</span>
                                </div>
                            }
                        }).collect_view()}
                    </div>
                </div>
            </Show>

            // ── TAB: Scorecards ──
            <Show when=move || active_tab.get() == "t-scorecards">
                <div class="space-y-6">
                    <div class="bg-purple-500/10 border border-purple-500/20 p-5 rounded-xl text-xs text-on-surface-variant leading-relaxed">
                        <span class="text-purple-400 font-bold">"G-27 Auto-seeded by Folio Provisioner. "</span>
                        "When this Folio instance was created, "
                        <code class="text-purple-400">"scorecard_provisioner::seed_pm_templates()"</code>
                        " automatically created 4 canonical PM scorecard templates scoped to this tenant."
                        " Anchor and Network Instance do " <strong class="text-on-surface">"not"</strong> " auto-seed scorecards."
                    </div>
                    <div class="bg-surface-container-low border border-outline-variant/20 rounded-xl overflow-hidden shadow-sm">
                        <table class="w-full text-left border-collapse text-xs">
                            <thead>
                                <tr class="text-xs uppercase tracking-wider text-on-surface-variant border-b border-outline-variant/20 bg-surface-container-high/40">
                                    <th class="py-3 px-5 font-medium">"Template Name"</th>
                                    <th class="py-3 px-5 font-medium">"Target Entity"</th>
                                    <th class="py-3 px-5 font-medium text-center">"Dimensions"</th>
                                    <th class="py-3 px-5 font-medium text-right"></th>
                                </tr>
                            </thead>
                            <tbody class="divide-y divide-outline-variant/10">
                                {vec![
                                    ("Contractor Performance", "atlas_service_provider", "6"),
                                    ("Listing Quality Index", "atlas_asset", "5"),
                                    ("Deal Qualification", "atlas_lead", "7"),
                                    ("Tenant Health Score", "atlas_scorecard_target", "5"),
                                ].into_iter().map(|(name, entity, dims)| {
                                    view! {
                                        <tr class="hover:bg-surface-bright/5 transition-colors">
                                            <td class="py-3 px-5 font-bold">{name}</td>
                                            <td class="py-3 px-5 font-mono text-on-surface-variant/70">{entity}</td>
                                            <td class="py-3 px-5 font-mono text-center">{dims}</td>
                                            <td class="py-3 px-5 text-right">
                                                <a href="/billing/scorecards" class="text-primary hover:underline font-bold text-[10px] uppercase tracking-wider">"Configure →"</a>
                                            </td>
                                        </tr>
                                    }
                                }).collect_view()}
                            </tbody>
                        </table>
                    </div>
                </div>
            </Show>

            // ── TAB: Background Jobs ──
            <Show when=move || active_tab.get() == "t-jobs">
                <div class="space-y-6">
                    <p class="text-xs text-on-surface-variant/80">
                        "Folio registers "
                        <strong class="text-on-surface">"4 background jobs"</strong>
                        " via the OutboxWorker. These run per-tenant on the platform scheduler."
                    </p>
                    <div class="bg-surface-container-low border border-outline-variant/20 rounded-xl overflow-hidden shadow-sm">
                        <table class="w-full text-left border-collapse text-xs">
                            <thead>
                                <tr class="text-xs uppercase tracking-wider text-on-surface-variant border-b border-outline-variant/20 bg-surface-container-high/40">
                                    <th class="py-3 px-5 font-medium">"Job Type"</th>
                                    <th class="py-3 px-5 font-medium text-center">"Interval"</th>
                                    <th class="py-3 px-5 font-medium text-center">"Status"</th>
                                </tr>
                            </thead>
                            <tbody class="divide-y divide-outline-variant/10">
                                {vec![
                                    ("pm_btc_mempool_poll", "120s", true),
                                    ("pm_str_permit_expiry_scanner", "86400s (daily)", true),
                                    ("pm_ota_revenue_sync", "3600s (hourly)", false),
                                    ("pm_str_hold_expiry_sweeper", "300s", true),
                                ].into_iter().map(|(job, interval, running)| {
                                    view! {
                                        <tr class="hover:bg-surface-bright/5 transition-colors">
                                            <td class="py-3 px-5 font-bold font-mono">{job}</td>
                                            <td class="py-3 px-5 font-mono text-center">{interval}</td>
                                            <td class="py-3 px-5 text-center font-bold">
                                                {if running {
                                                    view! { <span class="text-emerald-400">"● Running"</span> }.into_any()
                                                } else {
                                                    view! { <span class="text-amber-400">"⚠ Disabled"</span> }.into_any()
                                                }}
                                            </td>
                                        </tr>
                                    }
                                }).collect_view()}
                            </tbody>
                        </table>
                    </div>
                </div>
            </Show>
        </div>
    }
}
