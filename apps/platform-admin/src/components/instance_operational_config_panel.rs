//! InstanceOperationalConfigPanel — folio_mode selector, billing tier, portal toggles.
//!
//! Consumes the `PublicConfigResponse` already loaded by the parent `AppInstance`
//! page and lets a platform operator change any of the four operational knobs
//! without touching the public URL config (slug / domain).
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
    /// App type slug — used to conditionally show/hide Folio-specific controls.
    /// Pass "property_management" to show Folio Mode; omit or pass anything else to hide it.
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

    // ── Save handler ──────────────────────────────────────────────────────────
    let handle_save = move |_| {
        let t = toast.clone();
        let id = instance_id;
        let mode = folio_mode.get();
        let tier = billing_tier.get();
        let tp = tenant_portal.get();
        let vp = vendor_portal.get();

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
        // Full-width card — same Tailwind card pattern as every other tab so
        // the component always fills its container and layout is consistent.
        <div class="w-full bg-surface-container-low border border-outline-variant/20 rounded-xl overflow-hidden shadow-sm">

            // ── Header ───────────────────────────────────────────────────────
            <div class="px-5 py-3.5 border-b border-outline-variant/20 bg-surface-container-high/40 flex items-center justify-between">
                <h3 class="text-xs font-bold uppercase tracking-wider text-on-surface-variant">
                    "Operational Configuration"
                </h3>
            </div>

            <div class="p-5 flex flex-col gap-6">

                // ── Folio Mode (property_management instances only) ───────────
                // Folio mode controls PMC / brokerage / standard roles.
                // Irrelevant for Anchor (CMS) and Network instances.
                {if app_slug == "property_management" { Some(view! {
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

                // ── Billing Tier ──────────────────────────────────────────────
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
                        "Billing tier controls which syndication offers are mandatory and which optional features are accessible."
                    </div>
                </div>

                // ── Portal Flags ──────────────────────────────────────────────
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

                // ── Save Button ───────────────────────────────────────────────
                <div class="flex justify-end">
                    <button
                        class="px-4 py-2 rounded-xl text-xs font-semibold bg-primary text-on-primary hover:opacity-90 transition-all disabled:opacity-50"
                        on:click=handle_save
                        disabled=move || saving.get()
                    >
                        {move || if saving.get() { "Saving…" } else { "Save Operational Config" }}
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
