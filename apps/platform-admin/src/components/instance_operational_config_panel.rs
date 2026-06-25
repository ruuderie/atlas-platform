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
        <div class="section" style="margin-bottom:0;">
            <div class="section-header">
                <div class="section-title">
                    <svg viewBox="0 0 14 14" width="13" height="13" fill="none" stroke="currentColor" stroke-width="1.5">
                        <circle cx="7" cy="7" r="5"/>
                        <path d="M7 4v3l2 2"/>
                    </svg>
                    "Operational Configuration"
                </div>
            </div>

            <div style="padding:20px;display:flex;flex-direction:column;gap:24px;">

                // ── Folio Mode (property_management instances only) ───────────
                // Folio mode controls PMC / brokerage / standard roles.
                // It is irrelevant for Anchor (CMS) and Network instances.
                {if app_slug == "property_management" { Some(view! {
                <div>
                    <div style="font-size:11px;font-weight:700;color:var(--text-muted);text-transform:uppercase;letter-spacing:0.06em;margin-bottom:10px;">
                        "Folio Mode"
                    </div>
                    <div style="display:flex;gap:8px;flex-wrap:wrap;">
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
                    // Warning when changing a live instance mode
                    <div style="margin-top:10px;padding:8px 12px;background:color-mix(in srgb, var(--amber) 10%, transparent);border:1px solid color-mix(in srgb, var(--amber) 30%, transparent);border-radius:6px;font-size:11px;color:var(--amber);">
                        "⚠ Changing mode affects which portals users can access. Existing sessions are not revoked automatically — plan a maintenance window for live tenants."
                    </div>
                </div>
                }) } else { None }}

                // ── Billing Tier ──────────────────────────────────────────────
                <div>
                    <div style="font-size:11px;font-weight:700;color:var(--text-muted);text-transform:uppercase;letter-spacing:0.06em;margin-bottom:10px;">
                        "Billing Tier"
                    </div>
                    <div style="display:flex;gap:8px;flex-wrap:wrap;">
                        <TierCard value="free"       label="Free"       color="var(--text-muted)"  signal=billing_tier/>
                        <TierCard value="starter"    label="Starter"    color="var(--cobalt)"      signal=billing_tier/>
                        <TierCard value="growth"     label="Growth"     color="var(--green)"       signal=billing_tier/>
                        <TierCard value="enterprise" label="Enterprise" color="var(--amber)"       signal=billing_tier/>
                    </div>
                    <div style="margin-top:8px;font-size:11px;color:var(--text-muted);">
                        "Billing tier controls which syndication offers are mandatory and which optional features are accessible."
                    </div>
                </div>

                // ── Portal Flags ──────────────────────────────────────────────
                <div>
                    <div style="font-size:11px;font-weight:700;color:var(--text-muted);text-transform:uppercase;letter-spacing:0.06em;margin-bottom:12px;">
                        "Self-Service Portals"
                    </div>
                    <div style="display:flex;flex-direction:column;gap:12px;">
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
                <div style="display:flex;justify-content:flex-end;">
                    <button
                        class="btn btn-primary"
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
        <div style="display:flex;align-items:center;gap:14px;padding:12px 14px;border-radius:8px;border:1px solid var(--border-subtle);background:var(--surface-container-low);">
            <span style="font-size:20px;flex-shrink:0;">{icon}</span>
            <div style="flex:1;">
                <div style="font-size:13px;font-weight:600;color:var(--text-primary);">{label}</div>
                <div style="font-size:11px;color:var(--text-muted);margin-top:2px;">{description}</div>
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
