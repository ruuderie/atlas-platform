//! Anchor (CMS / Content) instance detail page.
//!
//! Rendered by `AppInstance` when `cfg.app_slug == "anchor"`.
//!
//! Anchor manages content pages, templates, menus, and feed items for
//! a tenant's public-facing CMS. It does not auto-seed scorecards.
//!
//! Tabs:
//!   Overview         — Identity card + live content activity stats
//!   Content          — CMS page/lead activity from real stats API
//!   Domains & Routing — Editable public_slug + custom_domain, DNS instructions
//!   Users            — TenantUsersPanel
//!   Operational Config — InstanceOperationalConfigPanel

use leptos::prelude::*;
use crate::api::admin::{PublicConfigResponse, get_instance_stats, update_public_config};
use crate::components::instance_operational_config_panel::InstanceOperationalConfigPanel;
use crate::components::tenant_users_panel::TenantUsersPanel;
use crate::components::callout::Callout;

#[component]
pub fn AnchorInstance(
    cfg: PublicConfigResponse,
) -> impl IntoView {
    let toast = use_context::<crate::app::GlobalToast>().expect("toast context");

    let instance_id  = cfg.instance_id;
    let tenant_id    = cfg.tenant_id;
    // Human-readable name resolved by the backend — never show a raw UUID as the title.
    let tenant_name  = StoredValue::new(cfg.tenant_name.clone());

    // ── Tab state ──
    let active_tab = RwSignal::new("t-overview".to_string());

    // ── Signals from config ──
    let public_slug   = RwSignal::new(cfg.public_slug.clone().unwrap_or_default());
    let custom_domain = RwSignal::new(cfg.custom_domain.clone().unwrap_or_default());
    let is_suspended  = RwSignal::new(cfg.instance_status == "suspended");
    let billing_tier  = StoredValue::new(cfg.billing_tier.clone());

    // Pre-populate DNS instructions from the loaded config.
    // These are refreshed after a successful domain save.
    let dns_record_type = StoredValue::new(
        cfg.dns_instructions.as_ref().map(|d| d.record_type.clone()).unwrap_or_default()
    );
    let dns_name = RwSignal::new(
        cfg.dns_instructions.as_ref().map(|d| d.name.clone()).unwrap_or_default()
    );
    let dns_value = RwSignal::new(
        cfg.dns_instructions.as_ref().map(|d| d.value.clone()).unwrap_or_default()
    );
    let dns_note = RwSignal::new(
        cfg.dns_instructions.as_ref().map(|d| d.note.clone()).unwrap_or_default()
    );

    // ── Domain edit state ──
    let slug_draft   = RwSignal::new(cfg.public_slug.clone().unwrap_or_default());
    let domain_draft = RwSignal::new(cfg.custom_domain.clone().unwrap_or_default());
    let saving_domain = RwSignal::new(false);

    // ── Live stats ──
    let stats = LocalResource::new(move || async move {
        get_instance_stats(instance_id).await.ok()
    });

    view! {
        <div class="w-full space-y-6">

            // ── Instance header ──
            // Tenant name is the primary identity (e.g. "buildwithruud").
            // Instance/tenant IDs are secondary, shown as monospace pills.
            <div class="bg-surface-container-low border border-outline-variant/20 rounded-2xl p-6 shadow-sm">

                // Top row: icon + tenant name + badges
                <div class="flex flex-col sm:flex-row sm:items-start sm:justify-between gap-4">

                    // Left: identity
                    <div class="flex items-start gap-4">
                        // Anchor icon badge
                        <div class="w-14 h-14 rounded-2xl bg-amber-500/10 border border-amber-500/20 flex items-center justify-center text-2xl shrink-0">
                            "⚓"
                        </div>

                        <div class="min-w-0">
                            // Primary: tenant business name
                            <div class="flex items-center gap-2 flex-wrap">
                                <h1 class="text-xl font-bold text-on-surface truncate">
                                    {move || tenant_name.get_value()}
                                </h1>
                                // App type badge
                                <span class="inline-flex items-center gap-1 px-2.5 py-0.5 rounded-full text-[10px] font-bold uppercase tracking-wider bg-amber-500/10 text-amber-400 border border-amber-500/20 shrink-0">
                                    "⚓ Anchor CMS"
                                </span>
                                // Status badge
                                {move || if is_suspended.get() {
                                    view! {
                                        <span class="inline-flex items-center gap-1 px-2.5 py-0.5 rounded-full text-[10px] font-bold bg-error/10 text-error border border-error/20 uppercase tracking-wider shrink-0">
                                            "● Suspended"
                                        </span>
                                    }.into_any()
                                } else {
                                    view! {
                                        <span class="inline-flex items-center gap-1 px-2.5 py-0.5 rounded-full text-[10px] font-bold bg-emerald-500/10 text-emerald-400 border border-emerald-500/20 uppercase tracking-wider shrink-0">
                                            "● Live"
                                        </span>
                                    }.into_any()
                                }}
                            </div>

                            // Secondary: billing tier pill
                            <div class="flex items-center gap-2 mt-1.5 flex-wrap">
                                <span class="text-xs text-on-surface-variant">
                                    {move || billing_tier.get_value().to_uppercase()}
                                    " tier"
                                </span>
                                <span class="text-on-surface-variant/30">"·"</span>
                                <span class="text-xs text-on-surface-variant">
                                    "Content Management System"
                                </span>
                            </div>

                            // ID pills: instance + tenant side by side, copyable
                            <div class="flex flex-wrap gap-2 mt-3">
                                <div class="flex items-center gap-1.5 px-2.5 py-1 rounded-lg bg-surface-container-high border border-outline-variant/20">
                                    <span class="text-[9px] font-bold uppercase tracking-wider text-on-surface-variant/60">"INST"</span>
                                    <span class="font-mono text-[10px] text-on-surface/70">{instance_id.to_string()}</span>
                                </div>
                                <div class="flex items-center gap-1.5 px-2.5 py-1 rounded-lg bg-surface-container-high border border-outline-variant/20">
                                    <span class="text-[9px] font-bold uppercase tracking-wider text-on-surface-variant/60">"TENANT"</span>
                                    <span class="font-mono text-[10px] text-on-surface/70">{tenant_id.to_string()}</span>
                                </div>
                            </div>
                        </div>
                    </div>

                    // Right: action button, clearly separated
                    <div class="shrink-0 self-start">
                        {move || if is_suspended.get() {
                            view! {
                                <button
                                    class="btn btn-primary"
                                    on:click=move |_| {
                                        let id = instance_id;
                                        let t = toast.clone();
                                        leptos::task::spawn_local(async move {
                                            let _ = crate::api::admin::resume_instance(id).await;
                                            t.show_toast("Resumed", "Anchor instance is now active.", "success");
                                        });
                                        is_suspended.set(false);
                                    }
                                >
                                    "▶ Resume Instance"
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
                                            t.show_toast("Suspended", "Anchor instance suspended.", "warning");
                                        });
                                        is_suspended.set(true);
                                    }
                                >
                                    "⏸ Suspend Instance"
                                </button>
                            }.into_any()
                        }}
                    </div>
                </div>
            </div>

            // ── Tab bar ──
            <div class="tab-bar">
                {vec![
                    ("t-overview",          "Overview"),
                    ("t-content",           "Content"),
                    ("t-domain",            "Domains & Routing"),
                    ("t-operational-config","Operational Config"),
                    ("t-users",             "Users"),
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
            // Both cards use min-h-[180px] so the grid holds its full size
            // while the Suspense stats card is loading — prevents the tab
            // collapsing to a tiny height on first render.
            <Show when=move || active_tab.get() == "t-overview">
                <div class="w-full grid grid-cols-1 lg:grid-cols-2 gap-6">
                    // Identity card
                    <div class="bg-surface-container-low border border-outline-variant/20 rounded-xl overflow-hidden shadow-sm min-h-[180px]">
                        <div class="px-5 py-3.5 border-b border-outline-variant/20 bg-surface-container-high/40">
                            <h3 class="text-xs font-bold uppercase tracking-wider text-on-surface-variant">"Instance Identity"</h3>
                        </div>
                        <div class="divide-y divide-outline-variant/10 text-xs">
                            <div class="flex justify-between items-center px-5 py-3">
                                <span class="text-on-surface-variant">"Tenant"</span>
                                <div class="text-right">
                                    <div class="font-semibold text-on-surface">{move || tenant_name.get_value()}</div>
                                    <div class="font-mono text-[9px] text-on-surface/50 mt-0.5">{tenant_id.to_string()}</div>
                                </div>
                            </div>
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

                    // Live CMS Stats from real backend
                    <div class="bg-surface-container-low border border-outline-variant/20 rounded-xl overflow-hidden shadow-sm min-h-[180px]">
                        <div class="px-5 py-3.5 border-b border-outline-variant/20 bg-surface-container-high/40">
                            <h3 class="text-xs font-bold uppercase tracking-wider text-on-surface-variant">"CMS Activity"</h3>
                        </div>
                        <Suspense fallback=move || view! { <div class="p-5 text-xs text-on-surface-variant">"Loading…"</div> }>
                        {move || {
                            let s = stats.get().flatten();
                            view! {
                                <div class="divide-y divide-outline-variant/10 text-xs">
                                    <div class="flex justify-between items-center px-5 py-3">
                                        <span class="text-on-surface-variant">"Leads captured (G-31)"</span>
                                        <span class="font-bold font-mono text-on-surface">{s.as_ref().map(|s| s.lead_count.to_string()).unwrap_or_else(|| "—".into())}</span>
                                    </div>
                                    <div class="flex justify-between items-center px-5 py-3">
                                        <span class="text-on-surface-variant">"Active listings"</span>
                                        <span class="font-bold font-mono text-on-surface">{s.as_ref().map(|s| s.active_listing_count.to_string()).unwrap_or_else(|| "—".into())}</span>
                                    </div>
                                    <div class="flex justify-between items-center px-5 py-3">
                                        <span class="text-on-surface-variant">"Open support cases"</span>
                                        <span class="font-bold font-mono text-on-surface">{s.as_ref().map(|s| s.open_case_count.to_string()).unwrap_or_else(|| "—".into())}</span>
                                    </div>
                                    <div class="flex justify-between items-center px-5 py-3">
                                        <span class="text-on-surface-variant">"Tenant data loaded from"</span>
                                        <span class="font-mono text-[10px] text-on-surface-variant/70">"GET /api/admin/app-instances/{id}/stats"</span>
                                    </div>
                                </div>
                            }
                        }}
                        </Suspense>
                    </div>
                </div>
            </Show>

            // ── TAB: Content ──
            // Shows real CMS activity stats (same source as Overview stats card).
            // Links out to the tenant's Anchor admin panel once subdomain is confirmed.
            <Show when=move || active_tab.get() == "t-content">
                <div class="space-y-4">
                    <Callout variant="warning" title="Anchor CMS">
                        "Manages pages, menus, templates, and lead capture forms for this tenant's public site. "
                        "Content is served via the CMS router and optionally routed through a custom domain. "
                        "No scorecards are auto-seeded for Anchor instances."
                    </Callout>
                    <Suspense fallback=move || view! { <div class="p-5 text-xs text-on-surface-variant">"Loading stats…"</div> }>
                    {move || {
                        let s = stats.get().flatten();
                        view! {
                            <div class="bg-surface-container-low border border-outline-variant/20 rounded-xl overflow-hidden shadow-sm">
                                <div class="px-5 py-3.5 border-b border-outline-variant/20 bg-surface-container-high/40">
                                    <h3 class="text-xs font-bold uppercase tracking-wider text-on-surface-variant">"Live CMS Metrics"</h3>
                                </div>
                                <div class="divide-y divide-outline-variant/10 text-xs">
                                    <div class="flex justify-between items-center px-5 py-3">
                                        <span class="text-on-surface-variant">"Lead captures (G-31)"</span>
                                        <span class="font-bold font-mono text-on-surface">{s.as_ref().map(|s| s.lead_count.to_string()).unwrap_or_else(|| "—".into())}</span>
                                    </div>
                                    <div class="flex justify-between items-center px-5 py-3">
                                        <span class="text-on-surface-variant">"Published listings"</span>
                                        <span class="font-bold font-mono text-on-surface">{s.as_ref().map(|s| s.active_listing_count.to_string()).unwrap_or_else(|| "—".into())}</span>
                                    </div>
                                    <div class="flex justify-between items-center px-5 py-3">
                                        <span class="text-on-surface-variant">"Open support cases"</span>
                                        <span class="font-bold font-mono text-on-surface">{s.as_ref().map(|s| s.open_case_count.to_string()).unwrap_or_else(|| "—".into())}</span>
                                    </div>
                                    <div class="flex justify-between items-center px-5 py-3">
                                        <span class="text-on-surface-variant">"Vendor/service providers"</span>
                                        <span class="font-bold font-mono text-on-surface">{s.as_ref().map(|s| s.vendor_count.to_string()).unwrap_or_else(|| "—".into())}</span>
                                    </div>
                                </div>
                            </div>
                        }
                    }}
                    </Suspense>
                    // Link to tenant's anchor admin panel
                    <div class="bg-surface-container-low border border-outline-variant/20 rounded-xl p-5 text-xs space-y-2">
                        <p class="text-on-surface-variant">"The Anchor admin panel is accessible at the tenant's configured domain."</p>
                        <a
                            href=move || {
                                let d = custom_domain.get();
                                if d.is_empty() {
                                    format!("https://{}.atlas.app/admin", public_slug.get())
                                } else {
                                    format!("https://{}/admin", d)
                                }
                            }
                            target="_blank"
                            rel="noopener"
                            class="text-primary hover:underline"
                        >
                            "Open Anchor Admin Panel →"
                        </a>
                    </div>
                </div>
            </Show>

            // ── TAB: Domains & Routing ──
            // Editable public_slug and custom_domain.
            // After save, DNS instructions (CNAME record) are shown if a custom domain is set.
            <Show when=move || active_tab.get() == "t-domain">
                <div class="w-full grid grid-cols-1 lg:grid-cols-2 gap-6">
                    // ── Edit form ──
                    <div class="bg-surface-container-low border border-outline-variant/20 rounded-xl overflow-hidden shadow-sm">
                        <div class="px-5 py-3.5 border-b border-outline-variant/20 bg-surface-container-high/40">
                            <h3 class="text-xs font-bold uppercase tracking-wider text-on-surface-variant">"Domain Configuration"</h3>
                        </div>
                        <div class="p-5 space-y-4">
                            // Platform subdomain (derived from public_slug)
                            <div>
                                <label class="block text-[10px] font-bold uppercase tracking-wider text-on-surface-variant mb-1">
                                    "Platform Subdomain (public_slug)"
                                </label>
                                <div class="flex items-center gap-2">
                                    <input
                                        type="text"
                                        class="flex-1 bg-surface-container-high border border-outline-variant/30 rounded-lg px-3 py-2 text-xs font-mono text-on-surface focus:border-primary focus:ring-1 focus:ring-primary/20 outline-none transition-all"
                                        placeholder="e.g. buildwithruud"
                                        prop:value=move || slug_draft.get()
                                        on:input=move |ev| slug_draft.set(event_target_value(&ev))
                                    />
                                    <span class="text-xs text-on-surface-variant/60 shrink-0">".atlas.app"</span>
                                </div>
                                <p class="text-[10px] text-on-surface-variant/60 mt-1">"Lowercase alphanumeric and hyphens only."</p>
                            </div>
                            // Custom domain
                            <div>
                                <label class="block text-[10px] font-bold uppercase tracking-wider text-on-surface-variant mb-1">
                                    "Custom Domain (optional)"
                                </label>
                                <input
                                    type="text"
                                    class="w-full bg-surface-container-high border border-outline-variant/30 rounded-lg px-3 py-2 text-xs font-mono text-on-surface focus:border-primary focus:ring-1 focus:ring-primary/20 outline-none transition-all"
                                    placeholder="e.g. buildwithruud.com"
                                    prop:value=move || domain_draft.get()
                                    on:input=move |ev| domain_draft.set(event_target_value(&ev))
                                />
                                <p class="text-[10px] text-on-surface-variant/60 mt-1">
                                    "Leave blank to use platform subdomain only. "
                                    "DNS instructions will appear below after saving."
                                </p>
                            </div>
                            // Save button — right-aligned, same weight as all other primary CTAs
                            <div class="flex justify-end">
                            <button
                                class="btn btn-primary"
                                disabled=move || saving_domain.get()
                                on:click=move |_| {
                                    let id = instance_id;
                                    let slug = slug_draft.get();
                                    let domain = domain_draft.get();
                                    let t = toast.clone();
                                    saving_domain.set(true);
                                    leptos::task::spawn_local(async move {
                                        let slug_opt  = if slug.is_empty() { None } else { Some(slug.clone()) };
                                        let domain_opt = if domain.is_empty() { None } else { Some(domain.clone()) };
                                        match update_public_config(id, slug_opt, domain_opt).await {
                                            Ok(updated) => {
                                                // Refresh displayed slug + domain from server response
                                                public_slug.set(updated.public_slug.clone().unwrap_or(slug));
                                                custom_domain.set(updated.custom_domain.clone().unwrap_or_default());
                                                // Update DNS instruction signals
                                                if let Some(dns) = updated.dns_instructions {
                                                    dns_name.set(dns.name);
                                                    dns_value.set(dns.value);
                                                    dns_note.set(dns.note);
                                                } else {
                                                    dns_name.set(String::new());
                                                    dns_value.set(String::new());
                                                    dns_note.set(String::new());
                                                }
                                                t.show_toast("Saved", "Domain configuration updated.", "success");
                                            }
                                            Err(e) => {
                                                t.show_toast("Error", &e, "error");
                                            }
                                        }
                                        saving_domain.set(false);
                                    });
                                }
                            >
                                {move || if saving_domain.get() { "Saving…" } else { "Save Domain Config" }}
                            </button>
                            </div>
                        </div>
                        // Current routing summary
                        <div class="border-t border-outline-variant/10 divide-y divide-outline-variant/10 text-xs">
                            <div class="flex justify-between items-center px-5 py-3">
                                <span class="text-on-surface-variant">"Active Platform Subdomain"</span>
                                <span class="font-mono text-on-surface/80">{move || format!("{}.atlas.app", public_slug.get())}</span>
                            </div>
                            <div class="flex justify-between items-center px-5 py-3">
                                <span class="text-on-surface-variant">"Custom Domain"</span>
                                <span class="font-mono text-on-surface/80">{move || {
                                    let d = custom_domain.get();
                                    if d.is_empty() { "—".to_string() } else { d }
                                }}</span>
                            </div>
                        </div>
                    </div>

                    // ── DNS Instructions (shown only when a custom domain has instructions) ──
                    <div class="space-y-4">
                        {move || {
                            let name = dns_name.get();
                            if name.is_empty() {
                                view! {
                                    <div class="bg-surface-container-low border border-outline-variant/20 rounded-xl p-5 text-xs text-on-surface-variant/60 leading-relaxed">
                                        "DNS instructions will appear here once you configure a custom domain. "
                                        "Point your domain's CNAME record to the Atlas platform edge."
                                    </div>
                                }.into_any()
                            } else {
                                view! {
                                    <div class="bg-surface-container-low border border-outline-variant/20 rounded-xl overflow-hidden shadow-sm">
                                        <div class="px-5 py-3.5 border-b border-outline-variant/20 bg-surface-container-high/40 flex items-center justify-between">
                                            <h3 class="text-xs font-bold uppercase tracking-wider text-on-surface-variant">"DNS Instructions"</h3>
                                            <span class="text-[9px] font-bold px-2 py-0.5 rounded bg-amber-500/10 text-amber-400 border border-amber-500/20 uppercase tracking-wider">
                                                "Action Required"
                                            </span>
                                        </div>
                                        <div class="p-5 space-y-3">
                                            <p class="text-xs text-on-surface-variant leading-relaxed">
                                                {move || dns_note.get()}
                                            </p>
                                            <div class="bg-surface-container-high rounded-lg p-4 font-mono text-xs space-y-2">
                                                <div class="flex gap-4">
                                                    <span class="text-on-surface-variant/60 w-12 shrink-0">"Type"</span>
                                                    <span class="text-amber-400 font-bold">{dns_record_type.get_value()}</span>
                                                </div>
                                                <div class="flex gap-4">
                                                    <span class="text-on-surface-variant/60 w-12 shrink-0">"Name"</span>
                                                    <span class="text-on-surface">{move || dns_name.get()}</span>
                                                </div>
                                                <div class="flex gap-4">
                                                    <span class="text-on-surface-variant/60 w-12 shrink-0">"Value"</span>
                                                    <span class="text-primary">{move || dns_value.get()}</span>
                                                </div>
                                            </div>
                                            <p class="text-[10px] text-on-surface-variant/50 leading-relaxed">
                                                "SSL/TLS is provisioned automatically via Cloudflare once the CNAME is verified. "
                                                "Propagation may take up to 48 hours."
                                            </p>
                                        </div>
                                    </div>
                                }.into_any()
                            }
                        }}
                    </div>
                </div>
            </Show>

            // ── TAB: Operational Config ──
            // Uses a consistent outer div (space-y-6) to match all other tabs
            // and suppress layout shift when switching tabs.
            <Show when=move || active_tab.get() == "t-operational-config">
                <div class="w-full space-y-6">
                {move || {
                    let cfg_opt = Some(PublicConfigResponse {
                        instance_id,
                        tenant_id,
                        tenant_name: tenant_name.get_value(),
                        app_slug: "anchor".to_string(),
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
                        // app_slug="anchor" — Folio Mode section is hidden inside
                        // the panel because anchor != "property_management".
                        <InstanceOperationalConfigPanel
                            instance_id=instance_id
                            config=cfg_opt
                            app_slug="anchor".to_string()
                        />
                    }
                }}
                </div>
            </Show>

            // ── TAB: Users ──
            // Consistent outer div matches all other tab wrappers.
            <Show when=move || active_tab.get() == "t-users">
                <div class="w-full space-y-6">
                    <TenantUsersPanel tenant_id=tenant_id />
                </div>
            </Show>
        </div>
    }
}
