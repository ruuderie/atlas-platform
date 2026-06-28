//! InstanceOperationalConfigPanel — per-app-type operational configuration UI.
//!
//! The panel adapts its content entirely based on `app_slug`:
//!
//!   "property_management" → Folio Mode + Billing Tier + Portal Toggles
//!   "anchor"              → CMS Billing Tier + CMS-specific notes (no PM portals)
//!   "network_instance"    → Network Billing Tier + NI-specific notes
//!   anything else         → Billing Tier only
//!
//! All writes go to:
//!   PATCH /api/admin/app-instances/{id}/operational-config

use leptos::prelude::*;
use uuid::Uuid;
use crate::api::admin::{update_operational_config, PublicConfigResponse};

// ── Props ─────────────────────────────────────────────────────────────────────

#[component]
pub fn InstanceOperationalConfigPanel(
    /// UUID of the app instance to configure
    instance_id: Uuid,
    /// Initial config loaded by parent; panel derives its signal state from this
    config: Option<PublicConfigResponse>,
    /// App type slug — drives which sections are visible.
    /// "property_management" | "anchor" | "network_instance"
    #[prop(default = String::new())]
    app_slug: String,
) -> impl IntoView {
    let toast = use_context::<crate::app::GlobalToast>().expect("toast context");

    // ── Local signals seeded from the current config ──────────────────────────
    let folio_mode = RwSignal::new(
        config.as_ref().map(|c| c.folio_mode.clone()).unwrap_or_else(|| "standard".into()),
    );
    let billing_tier = RwSignal::new(
        config.as_ref().map(|c| c.billing_tier.clone()).unwrap_or_else(|| "starter".into()),
    );
    let tenant_portal = RwSignal::new(
        config.as_ref().map(|c| c.tenant_portal_enabled).unwrap_or(false),
    );
    let vendor_portal = RwSignal::new(
        config.as_ref().map(|c| c.vendor_portal_enabled).unwrap_or(false),
    );

    let saving = RwSignal::new(false);

    // Whether this is a Folio PM instance (canonical: "property_management"; alias: "folio")
    let is_folio = app_slug == "property_management" || app_slug == "folio";
    let is_anchor = app_slug == "anchor";
    let is_network = app_slug == "network_instance" || app_slug == "network";

    // ── Save handler ──────────────────────────────────────────────────────────
    let handle_save = move |_| {
        let t = toast.clone();
        let id = instance_id;
        let mode = if is_folio { folio_mode.get() } else { "standard".to_string() };
        let tier = billing_tier.get();
        // Portal flags are only meaningful for Folio — send false for other types
        let tp = if is_folio { tenant_portal.get() } else { false };
        let vp = if is_folio { vendor_portal.get() } else { false };

        saving.set(true);
        leptos::task::spawn_local(async move {
            match update_operational_config(
                id,
                Some(mode),
                Some(tier),
                Some(tp),
                Some(vp),
            ).await {
                Ok(_) => t.show_toast("Saved", "Operational config updated.", "success"),
                Err(e) => t.show_toast("Error", &format!("Save failed: {}", e), "error"),
            }
            saving.set(false);
        });
    };

    view! {
        <div class="w-full bg-surface-container-low border border-outline-variant/20 rounded-xl overflow-hidden shadow-sm">

            // ── Header ───────────────────────────────────────────────────────
            <div class="px-5 py-3.5 border-b border-outline-variant/20 bg-surface-container-high/40 flex items-center justify-between">
                <h3 class="text-xs font-bold uppercase tracking-wider text-on-surface-variant">
                    "Operational Configuration"
                </h3>
                // App-type badge so the operator always knows what type they're editing
                <span class=if is_folio {
                    "inline-flex items-center gap-1 px-2 py-0.5 rounded-full text-[9px] font-bold uppercase tracking-wider bg-violet-500/10 text-violet-400 border border-violet-500/20"
                } else if is_anchor {
                    "inline-flex items-center gap-1 px-2 py-0.5 rounded-full text-[9px] font-bold uppercase tracking-wider bg-amber-500/10 text-amber-400 border border-amber-500/20"
                } else {
                    "inline-flex items-center gap-1 px-2 py-0.5 rounded-full text-[9px] font-bold uppercase tracking-wider bg-sky-500/10 text-sky-400 border border-sky-500/20"
                }>
                    {if is_folio { "⚙ Folio PM" } else if is_anchor { "⚓ Anchor CMS" } else { "🌐 Network" }}
                </span>
            </div>

            <div class="p-5 flex flex-col gap-6">

                // ── FOLIO PM: Mode selector ───────────────────────────────────
                // Only shown for property_management instances.
                {if is_folio { Some(view! {
                    <div>
                        <div class="text-[11px] font-bold text-on-surface-variant uppercase tracking-wider mb-2.5">
                            "Folio Mode"
                        </div>
                        <div class="flex gap-2 flex-wrap">
                            <ModeCard
                                value="standard"
                                label="Standard"
                                description="Landlord-operated property management. Supports LTR and STR modules."
                                icon="🏠"
                                signal=folio_mode
                            />
                            <ModeCard
                                value="pmc"
                                label="PMC"
                                description="Property Management Company — manages multiple client landlord accounts."
                                icon="🏢"
                                signal=folio_mode
                            />
                            <ModeCard
                                value="brokerage"
                                label="Brokerage"
                                description="Licensed brokerage office — agent + broker portals, commission plans."
                                icon="🤝"
                                signal=folio_mode
                            />
                        </div>
                        <div class="mt-2.5 px-3 py-2 bg-amber-500/10 border border-amber-500/30 rounded-lg text-[11px] text-amber-400">
                            "⚠ Changing mode affects which portals users can access. Existing sessions are not revoked automatically — plan a maintenance window for live tenants."
                        </div>
                    </div>
                }) } else { None }}

                // ── ANCHOR CMS: App-specific context ─────────────────────────
                // Anchor does not have PM portals. Show CMS-relevant info instead.
                {if is_anchor { Some(view! {
                    <div class="space-y-3">
                        <div class="text-[11px] font-bold text-on-surface-variant uppercase tracking-wider">
                            "CMS Configuration"
                        </div>
                        <div class="grid grid-cols-1 sm:grid-cols-2 gap-3">
                            <div class="flex items-start gap-3 px-4 py-3.5 rounded-xl border border-outline-variant/20 bg-surface-container">
                                <span class="text-lg shrink-0">"📄"</span>
                                <div>
                                    <div class="text-xs font-bold text-on-surface">"Page Engine"</div>
                                    <div class="text-[11px] text-on-surface-variant mt-0.5 leading-relaxed">
                                        "CMS pages, menus, templates, and rich content blocks are managed through the Anchor admin panel."
                                    </div>
                                </div>
                            </div>
                            <div class="flex items-start gap-3 px-4 py-3.5 rounded-xl border border-outline-variant/20 bg-surface-container">
                                <span class="text-lg shrink-0">"🔗"</span>
                                <div>
                                    <div class="text-xs font-bold text-on-surface">"Lead Capture (G-31)"</div>
                                    <div class="text-[11px] text-on-surface-variant mt-0.5 leading-relaxed">
                                        "Form builders on CMS pages route leads into the Atlas lead pipeline for this tenant."
                                    </div>
                                </div>
                            </div>
                            <div class="flex items-start gap-3 px-4 py-3.5 rounded-xl border border-outline-variant/20 bg-surface-container">
                                <span class="text-lg shrink-0">"🌍"</span>
                                <div>
                                    <div class="text-xs font-bold text-on-surface">"Domain Routing"</div>
                                    <div class="text-[11px] text-on-surface-variant mt-0.5 leading-relaxed">
                                        "Configure the public slug and custom domain in the Domains & Routing tab."
                                    </div>
                                </div>
                            </div>
                            <div class="flex items-start gap-3 px-4 py-3.5 rounded-xl border border-outline-variant/20 bg-surface-container">
                                <span class="text-lg shrink-0">"📊"</span>
                                <div>
                                    <div class="text-xs font-bold text-on-surface">"Analytics"</div>
                                    <div class="text-[11px] text-on-surface-variant mt-0.5 leading-relaxed">
                                        "CMS engagement metrics are aggregated by the platform telemetry service hourly."
                                    </div>
                                </div>
                            </div>
                        </div>
                        <p class="text-[11px] text-on-surface-variant/60 leading-relaxed pt-1">
                            "Anchor CMS instances do not have Folio PM portals (tenant, vendor, owner). "
                            "PM features are available on Folio (property_management) instances only."
                        </p>
                    </div>
                }) } else { None }}

                // ── NETWORK INSTANCE: App-specific context ────────────────────
                {if is_network { Some(view! {
                    <div class="space-y-3">
                        <div class="text-[11px] font-bold text-on-surface-variant uppercase tracking-wider">
                            "Network Configuration"
                        </div>
                        <div class="grid grid-cols-1 sm:grid-cols-2 gap-3">
                            <div class="flex items-start gap-3 px-4 py-3.5 rounded-xl border border-outline-variant/20 bg-surface-container">
                                <span class="text-lg shrink-0">"🗂"</span>
                                <div>
                                    <div class="text-xs font-bold text-on-surface">"Listing Directory"</div>
                                    <div class="text-[11px] text-on-surface-variant mt-0.5 leading-relaxed">
                                        "This instance hosts the public marketplace directory — search, listings, categories, and lead forms."
                                    </div>
                                </div>
                            </div>
                            <div class="flex items-start gap-3 px-4 py-3.5 rounded-xl border border-outline-variant/20 bg-surface-container">
                                <span class="text-lg shrink-0">"🔗"</span>
                                <div>
                                    <div class="text-xs font-bold text-on-surface">"Syndication Links (G-05)"</div>
                                    <div class="text-[11px] text-on-surface-variant mt-0.5 leading-relaxed">
                                        "Manage inbound syndication offers from partner networks in the Syndication tab."
                                    </div>
                                </div>
                            </div>
                        </div>
                        <p class="text-[11px] text-on-surface-variant/60 leading-relaxed pt-1">
                            "Network instances do not have Folio PM portals. PM features are available on Folio instances only."
                        </p>
                    </div>
                }) } else { None }}

                // ── Billing Tier — all instance types ────────────────────────
                <div>
                    <div class="text-[11px] font-bold text-on-surface-variant uppercase tracking-wider mb-2.5">
                        "Billing Tier"
                    </div>
                    <div class="flex gap-2 flex-wrap">
                        <TierCard value="free"       label="Free"       color="var(--text-muted)"  signal=billing_tier/>
                        <TierCard value="starter"    label="Starter"    color="var(--cobalt)"      signal=billing_tier/>
                        <TierCard value="growth"     label="Growth"     color="var(--green)"       signal=billing_tier/>
                        <TierCard value="enterprise" label="Enterprise" color="var(--amber)"       signal=billing_tier/>
                    </div>
                    <div class="mt-2 text-[11px] text-on-surface-variant">
                        {if is_folio {
                            "Billing tier controls which syndication offers are mandatory and which optional Folio features are accessible."
                        } else if is_anchor {
                            "Billing tier controls CMS page limits, CDN bandwidth, and available marketing integrations."
                        } else {
                            "Billing tier controls listing capacity, search indexing frequency, and directory API rate limits."
                        }}
                    </div>
                </div>

                // ── FOLIO PM ONLY: Self-Service Portal Flags ──────────────────
                // Tenant and Vendor portals only apply to Folio PM instances.
                // Showing them on CMS or Network instances is architecturally incorrect.
                {if is_folio { Some(view! {
                    <div>
                        <div class="text-[11px] font-bold text-on-surface-variant uppercase tracking-wider mb-3">
                            "Self-Service Portals"
                        </div>
                        <div class="flex flex-col gap-3">
                            <PortalToggle
                                label="Tenant Portal"
                                description="Tenants can register, view leases, submit maintenance requests, and pay rent."
                                icon="🏡"
                                signal=tenant_portal
                            />
                            <PortalToggle
                                label="Vendor Portal"
                                description="Vendors can accept work orders, submit invoices, and track payment status."
                                icon="🔧"
                                signal=vendor_portal
                            />
                        </div>
                    </div>
                }) } else { None }}

                // ── Save Button ───────────────────────────────────────────────
                <div class="flex justify-end">
                    <button
                        class="px-4 py-2 rounded-xl text-xs font-semibold bg-primary text-on-primary hover:opacity-90 transition-all disabled:opacity-50"
                        on:click=handle_save
                        disabled=move || saving.get()
                    >
                        {move || if saving.get() { "Saving…" } else { "Save Config" }}
                    </button>
                </div>

            </div>
        </div>
    }
}

// ── Sub-components ────────────────────────────────────────────────────────────

#[component]
fn ModeCard(
    value: &'static str,
    label: &'static str,
    description: &'static str,
    icon: &'static str,
    signal: RwSignal<String>,
) -> impl IntoView {
    let is_active = move || signal.get() == value;
    view! {
        <button
            style=move || {
                if is_active() {
                    "flex:1;min-width:160px;padding:14px 16px;border-radius:10px;border:2px solid var(--primary);background:color-mix(in srgb, var(--primary) 10%, var(--surface-container));text-align:left;cursor:pointer;transition:all 0.15s;"
                } else {
                    "flex:1;min-width:160px;padding:14px 16px;border-radius:10px;border:1px solid var(--border-default);background:var(--surface-container-low);text-align:left;cursor:pointer;transition:all 0.15s;"
                }
            }
            on:click=move |_| signal.set(value.to_string())
        >
            <div style="font-size:18px;margin-bottom:6px;">{icon}</div>
            <div style=move || format!("font-size:13px;font-weight:700;color:{};margin-bottom:3px;",
                if is_active() { "var(--primary)" } else { "var(--text-primary)" })>
                {label}
            </div>
            <div style="font-size:11px;color:var(--text-muted);line-height:1.4;">{description}</div>
        </button>
    }
}

#[component]
fn TierCard(
    value: &'static str,
    label: &'static str,
    color: &'static str,
    signal: RwSignal<String>,
) -> impl IntoView {
    let is_active = move || signal.get() == value;
    view! {
        <button
            style=move || {
                if is_active() {
                    format!("padding:8px 18px;border-radius:8px;border:2px solid {};background:color-mix(in srgb, {} 15%, var(--surface-container));font-size:12px;font-weight:700;color:{};cursor:pointer;transition:all 0.15s;",
                        color, color, color)
                } else {
                    "padding:8px 18px;border-radius:8px;border:1px solid var(--border-default);background:var(--surface-container-low);font-size:12px;font-weight:600;color:var(--text-muted);cursor:pointer;transition:all 0.15s;".to_string()
                }
            }
            on:click=move |_| signal.set(value.to_string())
        >
            {label}
        </button>
    }
}

#[component]
fn PortalToggle(
    label: &'static str,
    description: &'static str,
    icon: &'static str,
    signal: RwSignal<bool>,
) -> impl IntoView {
    view! {
        <div class="flex items-center gap-3.5 px-3.5 py-3 rounded-lg border border-outline-variant/20 bg-surface-container">
            <span style="font-size:20px;flex-shrink:0;">{icon}</span>
            <div class="flex-1">
                <div class="text-[13px] font-semibold text-on-surface">{label}</div>
                <div class="text-[11px] text-on-surface-variant mt-0.5">{description}</div>
            </div>
            // Toggle switch
            <button
                style=move || {
                    if signal.get() {
                        "width:40px;height:22px;border-radius:11px;border:none;background:var(--primary);cursor:pointer;position:relative;transition:background 0.2s;flex-shrink:0;"
                    } else {
                        "width:40px;height:22px;border-radius:11px;border:none;background:var(--border-default);cursor:pointer;position:relative;transition:background 0.2s;flex-shrink:0;"
                    }
                }
                on:click=move |_| signal.set(!signal.get())
                role="switch"
                aria-checked=move || if signal.get() { "true" } else { "false" }
            >
                <span style=move || {
                    if signal.get() {
                        "position:absolute;top:3px;left:20px;width:16px;height:16px;border-radius:50%;background:white;transition:left 0.2s;"
                    } else {
                        "position:absolute;top:3px;left:4px;width:16px;height:16px;border-radius:50%;background:white;transition:left 0.2s;"
                    }
                }></span>
            </button>
        </div>
    }
}
