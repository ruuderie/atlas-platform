use leptos::prelude::*;
use leptos_router::hooks::use_params_map;
use uuid::Uuid;

use shared_ui::components::ui::button::{Button, ButtonVariant};
use shared_ui::components::badge::{Badge, BadgeIntent};
use shared_ui::components::ui::input::{Input, InputType};
use shared_ui::components::ui::label::Label;

use crate::components::upsell_banner::UpsellBanner;
use crate::components::onboarding_wizard::OnboardingWizard;
use crate::components::seed_picker::SeedPicker;
use crate::api::onboarding::get_onboarding_status;
use crate::api::admin::{suspend_instance, resume_instance};
use crate::api::listings::update_listing;
use crate::api::models::ListingUpdate;

/// Maps a canonical `app_slug` / `app_type` to (icon, display label, accent css suffix).
/// accent suffix is used as `text-{accent}-400` / `bg-{accent}-500/10`.
fn app_type_display(slug: &str) -> (&'static str, &'static str, &'static str) {
    match slug {
        "property_management" | "folio" => ("🏠", "Folio PM",        "violet"),
        "anchor"                        => ("⚓", "Anchor CMS",       "amber"),
        "network_instance" | "network"  => ("🔗", "Network Directory","blue"),
        "str"                           => ("🏖️","Atlas STR",         "emerald"),
        _                               => ("📦", "App Instance",     "slate"),
    }
}

#[component]
pub fn AppDashboard() -> impl IntoView {
    let params = use_params_map();
    let site_id = move || params.with(|p| p.get("id").unwrap_or_default());

    let toast = use_context::<crate::app::GlobalToast>().expect("toast context");
    
    let (show_add_listing, set_show_add_listing) = signal(false);
    let (show_add_category, set_show_add_category) = signal(false);
    let (show_add_template, set_show_add_template) = signal(false);
    
    // (listing_id, display_name) — both needed to call PUT /api/admin/listings/{id}
    let (editing_listing, set_editing_listing) = signal(None::<(String, String)>);
    let editing_alias = RwSignal::new(String::new());
    let is_saving_listing = RwSignal::new(false);
    let (managing_user_name, set_managing_user_name) = signal(None::<String>);

    let active_tab = RwSignal::new("settings".to_string());

    let dirs = use_context::<LocalResource<Vec<crate::api::models::PlatformAppModel>>>().expect("dirs context");
    let domain_bind = RwSignal::new(String::new());
    
    Effect::new(move |_| {
        let current_id = site_id();
        if let Some(d) = dirs.get() {
            if let Some(dir) = d.into_iter().find(|dir| dir.instance_id.to_string() == current_id) {
                domain_bind.set(dir.domain.clone());
            } else {
                domain_bind.set(format!("{}.example.com", current_id));
            }
        }
    });
    
    let site_id_str = site_id().to_string();
    // Wrap in StoredValue so reactive `move ||` closures can clone it
    // without consuming the binding (avoids FnOnce / Fn mismatch).
    let site_id_stored = StoredValue::new(site_id_str.clone());
    let listings_res = LocalResource::new({
        let sid = site_id_str.clone();
        move || {
            let sid = sid.clone();
            async move { crate::api::listings::get_listings(&sid).await.unwrap_or_default() }
        }
    });


    let domains_res = LocalResource::new({
        let sid = site_id_str.clone();
        move || {
            let sid = sid.clone();
            async move { crate::api::admin::get_app_domains(sid).await.unwrap_or_default() }
        }
    });

    let (show_domain_modal, set_show_domain_modal) = signal(false);
    let new_domain_input = RwSignal::new(String::new());
    
    let add_domain_action = Action::new_local({
        let toast = toast.clone();
        let sid = site_id_str.clone();
        move |domain: &String| {
            let domain = domain.clone();
            let sid = sid.clone();
            let toast = toast.clone();
            async move {
                match crate::api::admin::add_app_domain(sid, domain).await {
                    Ok(_) => { toast.show_toast("Domains", "Domain securely attached.", "success"); }
                    Err(e) => { toast.show_toast("Error", &format!("Error adding domain: {}", e), "error"); }
                }
            }
        }
    });

    // ── Onboarding readiness gate ──────────────────────────────────────────
    // Fetches step status and drives the full-page wizard takeover.
    let ob_site_id = site_id_str.clone();
    let onboarding_status = LocalResource::new(move || {
        let sid = ob_site_id.clone();
        async move { get_onboarding_status(&sid).await }
    });

    // Derive per-instance tenant_id from the dirs context for the wizard
    let tenant_id_for_wizard = Signal::derive(move || {
        let current_id = site_id();
        if let Some(d) = dirs.get() {
            d.into_iter()
                .find(|dir| dir.instance_id.to_string() == current_id)
                .map(|dir| dir.tenant_id.to_string())
                .unwrap_or_default()
        } else {
            String::new()
        }
    });

    let app_manifest = Signal::derive(move || {
        let current_id = site_id();
        let app_type_str = if let Some(d) = dirs.get() {
            if let Some(dir) = d.into_iter().find(|dir| dir.instance_id.to_string() == current_id) {
                dir.app_type.clone()
            } else {
                "network".to_string()
            }
        } else {
            "network".to_string()
        };
        crate::components::app_manifest::get_manifest_for_app_type(&app_type_str)
    });

    let ob_site_id2 = site_id_str.clone();
    let ob_site_id3 = site_id_str.clone();

    view! {
        // ── Onboarding Wizard — full-page takeover ─────────────────────────
        {move || {
            match onboarding_status.get() {
                Some(Ok(ref status)) if !status.is_ready && status.dismissed_at.is_none() => {
                    let ai = ob_site_id2.clone();
                    let tid = tenant_id_for_wizard.get();
                    // on_dismiss: called by the wizard after the API call resolves.
                    // Triggers a refetch so the parent banner appears immediately,
                    // no page reload required.
                    let on_dismiss = Callback::new(move |_: ()| {
                        onboarding_status.refetch();
                    });
                    view! {
                        <OnboardingWizard
                            app_instance_id=ai
                            tenant_id=tid
                            on_dismiss=on_dismiss
                        />
                    }.into_any()
                }
                _ => view! { <div></div> }.into_any()
            }
        }}
        // ── Persistent incomplete banner (shown after dismissal) ────────────
        {move || {
            match onboarding_status.get() {
                Some(Ok(ref status)) if !status.is_ready && status.dismissed_at.is_some() => {
                    let incomplete = status.steps.iter()
                        .filter(|s| s.is_required && !s.is_complete)
                        .count();
                    let ob_sid = ob_site_id3.clone();
                    view! {
                        <div class="bg-amber-50 border border-amber-200 rounded-lg px-4 py-3 flex items-center justify-between gap-4 mb-4 mx-6 mt-4">
                            <div class="flex items-center gap-2">
                                <span class="text-amber-600 text-lg">"⚠️"</span>
                                <p class="text-sm text-amber-800 font-medium">
                                    {format!("{} required setup step{} remaining before your app goes live.",
                                        incomplete, if incomplete == 1 { "" } else { "s" })}
                                </p>
                            </div>
                            <a
                                href=format!("/apps/{}", ob_sid)
                                id="ob-reopen-wizard"
                                class="text-sm text-amber-700 underline font-semibold whitespace-nowrap"
                            >
                                "Resume Setup →"
                            </a>
                        </div>
                    }.into_any()
                }
                _ => view! { <div></div> }.into_any()
            }
        }}
        <Show
            when=move || dirs.get().is_some()
            fallback=|| view! {
                <div class="p-8 text-center text-on-surface-variant flex flex-col items-center justify-center min-h-[400px]">
                    <div class="animate-spin h-8 w-8 border-4 border-primary border-t-transparent rounded-full mb-4"></div>
                    "Loading Application Workspace..."
                </div> 
            }
        >
            <div class="main-canvas">
                // ── Tenant Hero ──
                <div class="tenant-hero">
                    <div>
                        <div class="breadcrumb">
                            <a href="/">"Platform"</a>
                            <span>" › "</span>
                            <a href="/apps">"Tenants"</a>
                            <span>" › "</span>
                            <span>{site_id}</span>
                        </div>
                        <div class="tenant-identity">
                            <div class="tenant-avatar">
                                {move || {
                                    let current_id = site_id();
                                    if let Some(d) = dirs.get() {
                                        d.into_iter()
                                            .find(|dir| dir.instance_id.to_string() == current_id)
                                            .and_then(|dir| dir.domain.chars().next())
                                            .or_else(|| current_id.chars().next())
                                            .map(|c| c.to_uppercase().to_string())
                                            .unwrap_or_else(|| "?".to_string())
                                    } else {
                                        "?".to_string()
                                    }
                                }}
                            </div>
                            <div>
                                <div class="tenant-name-row">
                                    <span class="tenant-name">{site_id}</span>
                                    {move || {
                                        let current_id = site_id();
                                        if let Some(d) = dirs.get() {
                                            if let Some(dir) = d.into_iter().find(|dir| dir.instance_id.to_string() == current_id) {
                                                let status_class = match dir.site_status.as_str() {
                                                    "active" => "tag tag-active",
                                                    "suspended" => "tag tag-warn",
                                                    _ => "tag",
                                                };
                                                let status_label = dir.site_status.clone();
                                                return view! {
                                                    <span class=status_class>{status_label}</span>
                                                }.into_any();
                                            }
                                        }
                                        view! { <span></span> }.into_any()
                                    }}
                                </div>
                                <div class="tenant-domain">"Domain: " {move || domain_bind.get()} " · instance: " {move || tenant_id_for_wizard.get()}</div>
                            </div>
                        </div>
                    </div>
                        <div class="hero-right">
                        <a href=move || {
                            let d = domain_bind.get();
                            if d.starts_with("http") { d } else if !d.is_empty() { format!("https://{}", d) } else { "#".to_string() }
                        } target="_blank" rel="noopener noreferrer">
                            <Button variant=ButtonVariant::Outline class="bg-background".to_string()>"View Live App"</Button>
                        </a>
                        <button
                            class="btn btn-ghost opacity-40 cursor-not-allowed"
                            title="Per-app impersonation endpoint pending — use Tenant-level Impersonate from /apps"
                            disabled
                        >"Impersonate"</button>
                        {
                            // Suspend/Resume — wired to POST /api/admin/app-instances/{id}/suspend|resume
                            let is_suspending = RwSignal::new(false);
                            let is_suspended = RwSignal::new(false);
                            let site_id_s = site_id();
                            // Seed status from dirs context
                            if let Some(dirs_val) = dirs.get_untracked() {
                                if let Some(dir) = dirs_val.into_iter().find(|d| d.instance_id.to_string() == site_id_s) {
                                    is_suspended.set(dir.site_status == "suspended" || dir.site_status == "Suspended");
                                }
                            }
                            let toast2 = toast.clone();
                            let site_id_for_suspend = StoredValue::new(site_id());
                            let handle_suspend = move |_| {
                                let id_str = site_id_for_suspend.get_value();
                                let Ok(id) = Uuid::parse_str(&id_str) else {
                                    toast2.show_toast("Error", "Invalid instance ID", "error");
                                    return;
                                };
                                is_suspending.set(true);
                                let suspended = is_suspended.get();
                                let t = toast2.clone();
                                leptos::task::spawn_local(async move {
                                    let result = if suspended {
                                        resume_instance(id).await.map(|_| ())
                                    } else {
                                        suspend_instance(id, "Manual suspension via admin panel.".to_string()).await.map(|_| ())
                                    };
                                    match result {
                                        Ok(_) => {
                                            is_suspended.update(|v| *v = !*v);
                                            let msg = if is_suspended.get() { "Instance suspended." } else { "Instance resumed." };
                                            t.show_toast("Status Updated", msg, if is_suspended.get() { "warning" } else { "success" });
                                        }
                                        Err(e) => t.show_toast("Error", &e, "error"),
                                    }
                                    is_suspending.set(false);
                                });
                            };
                            view! {
                                <button
                                    class=move || format!("btn {} transition-all {}",
                                        if is_suspended.get() { "btn-primary" } else { "btn-ghost border border-error/40 text-error hover:bg-error hover:text-white" },
                                        if is_suspending.get() { "opacity-40 cursor-not-allowed" } else { "" }
                                    )
                                    disabled=move || is_suspending.get()
                                    on:click=handle_suspend
                                >
                                    {move || match (is_suspending.get(), is_suspended.get()) {
                                        (true, _)    => "Working…",
                                        (false, true)  => "Resume Instance",
                                        (false, false) => "Suspend Instance",
                                    }}
                                </button>
                            }
                        }
                        // Provision New — navigate to the full wizard
                        <a
                            href="/apps/new"
                            class="btn btn-primary"
                            style="text-decoration:none"
                        >"+ Provision New"</a>
                    </div>
                </div>

                // ── KPI Strip ──
                <div class="hero-kpi-strip">
                    {move || {
                        let current_id = site_id();
                        if let Some(dirs_val) = dirs.get() {
                            if let Some(dir) = dirs_val.into_iter().find(|d| d.instance_id.to_string() == current_id) {
                                let status = dir.site_status.clone();
                                let status_color = match status.as_str() {
                                    "active" => "color:var(--green)",
                                    "suspended" => "color:var(--red)",
                                    _ => "color:var(--text-muted)",
                                };
                                return view! {
                                    <div class="hkpi">
                                        <span class="hkpi-label">"Status"</span>
                                        <div class="hkpi-value" style=status_color>{status.clone()}</div>
                                        <div class="hkpi-sub">"Site status"</div>
                                    </div>
                                    <div class="hkpi-sep"></div>
                                    <div class="hkpi">
                                        <span class="hkpi-label">"Domain"</span>
                                        <div class="hkpi-value" style="font-size:12px">{dir.domain.clone()}</div>
                                        <div class="hkpi-sub">"Primary hostname"</div>
                                    </div>
                                    <div class="hkpi-sep"></div>
                                    <div class="hkpi">
                                        <span class="hkpi-label">"App Type"</span>
                                        <div class="hkpi-value" style="font-size:12px">{dir.app_type.clone()}</div>
                                        <div class="hkpi-sub">"Platform type"</div>
                                    </div>
                                }.into_any();
                            }
                        }
                        view! { <div class="hkpi"><span class="hkpi-label">"Loading..."</span></div> }.into_any()
                    }}
                </div>

                <Show when=move || listings_res.get().map(|lst| lst.is_empty()).unwrap_or(false)>
                    <div class="px-6 mt-4">
                        <UpsellBanner 
                            title="Supercharge your new application!".to_string()
                            description="Jumpstart your marketplace with pre-populated leads and premium business listings."
                                .to_string()
                            cta_text="Get 100 Premium Listings - $49".to_string()
                            on_click=Callback::new(move |_| {
                                leptos::logging::log!("Upsell Clicked: Application Injection on Dashboard");
                            })
                        />
                    </div>
                </Show>

                // ── Instance Cards — all instances belonging to this tenant ──
                {move || {
                    let current_id = site_id();
                    // Find the tenant_id for the current instance
                    let tenant_id_opt = dirs.get().and_then(|d| {
                        d.into_iter()
                            .find(|dir| dir.instance_id == current_id)
                            .map(|dir| dir.tenant_id.clone())
                    });
                    if let Some(tenant_id) = tenant_id_opt {
                        // Collect all instances under the same tenant
                        let siblings: Vec<_> = dirs.get().unwrap_or_default()
                            .into_iter()
                            .filter(|dir| dir.tenant_id == tenant_id)
                            .collect();
                        if siblings.len() > 1 || siblings.iter().any(|s| s.instance_id != current_id) {
                            let cards = siblings.into_iter().map(|dir| {
                                let (icon, label, _accent) = app_type_display(&dir.app_type);
                                let is_current = dir.instance_id == current_id;
                                let status_live = dir.site_status == "active" || dir.site_status == "Active";
                                let manage_url = format!("/apps/{}/instance", dir.instance_id);
                                let iid_short = dir.instance_id.chars().take(8).collect::<String>() + "…";
                                view! {
                                    <a
                                        href=manage_url
                                        class=move || if is_current {
                                            "flex items-center gap-3 px-4 py-3 rounded-xl border border-primary/40 bg-primary/5 text-on-surface hover:bg-primary/10 transition-all no-underline"
                                        } else {
                                            "flex items-center gap-3 px-4 py-3 rounded-xl border border-outline-variant/20 bg-surface-container-low hover:bg-surface-container-high/60 text-on-surface transition-all no-underline"
                                        }
                                    >
                                        <span class="text-xl shrink-0">{icon}</span>
                                        <div class="min-w-0 flex-1">
                                            <div class="flex items-center gap-2">
                                                <span class="text-xs font-bold text-on-surface">{label}</span>
                                                {if is_current {
                                                    view! { <span class="text-[9px] font-bold bg-primary/20 text-primary px-1.5 py-0.5 rounded uppercase tracking-wider">"CURRENT"</span> }.into_any()
                                                } else {
                                                    view! { <span></span> }.into_any()
                                                }}
                                            </div>
                                            <div class="text-[10px] font-mono text-on-surface-variant/60 mt-0.5">{iid_short}</div>
                                        </div>
                                        <span class=if status_live {
                                            "text-[9px] font-bold bg-emerald-500/10 text-emerald-400 border border-emerald-500/20 px-1.5 py-0.5 rounded uppercase tracking-wider shrink-0"
                                        } else {
                                            "text-[9px] font-bold bg-error/10 text-error border border-error/20 px-1.5 py-0.5 rounded uppercase tracking-wider shrink-0"
                                        }>{
                                            if status_live { "● Live" } else { "⊘ Suspended" }
                                        }</span>
                                    </a>
                                }
                            }).collect_view();
                            view! {
                                <div class="px-6 mt-4">
                                    <div class="flex items-center justify-between mb-2">
                                        <span class="text-xs font-bold uppercase tracking-wider text-on-surface-variant/60">"Tenant App Instances"</span>
                                        <a href="/apps/new" class="text-xs text-primary hover:underline font-semibold">"+ Provision New"</a>
                                    </div>
                                    <div class="flex flex-wrap gap-2">{cards}</div>
                                </div>
                            }.into_any()
                        } else {
                            // Single instance — still show a manage link
                            let iid = current_id.clone();
                            view! {
                                <div class="px-6 mt-4 flex items-center justify-between">
                                    <a
                                        href=format!("/apps/{}/instance", iid)
                                        class="inline-flex items-center gap-2 text-xs font-semibold text-primary hover:underline"
                                    >
                                        "View Instance Config →"
                                    </a>
                                    <a href="/apps/new" class="text-xs text-on-surface-variant hover:text-primary hover:underline font-semibold">"+ Provision New Instance"</a>
                                </div>
                            }.into_any()
                        }
                    } else {
                        view! { <div></div> }.into_any()
                    }
                }}

                // ── Tab Bar ──
                <div class="tab-bar">
                    {move || app_manifest.get().panels.into_iter().map(|panel| {
                        let panel_id = panel.id.clone();
                        let panel_id_clone = panel_id.clone();
                        let panel_title = panel.title.clone();
                        view! {
                            <button
                                type="button"
                                class="tab"
                                class:active=move || active_tab.get() == panel_id
                                on:click=move |_| active_tab.set(panel_id_clone.clone())
                            >
                                {panel_title}
                            </button>
                        }
                    }).collect_view()}
                    <button
                        type="button"
                        class="tab"
                        class:active=move || active_tab.get() == "seed_data"
                        on:click=move |_| active_tab.set("seed_data".to_string())
                    >
                        "Seed Data"
                    </button>
                    <button
                        type="button"
                        class="tab"
                        class:active=move || active_tab.get() == "domains"
                        on:click=move |_| active_tab.set("domains".to_string())
                    >
                        "Routing & Domains"
                    </button>
                </div>

                // ── Tab Content ──
                <div class="tab-content">
                    {move || {
                        app_manifest.get().panels.into_iter().map(|panel| {
                            let panel_id = panel.id.clone();
                            let panel_id_clone = panel_id.clone();
                            view! {
                                <div class="pane" class:active=move || active_tab.get() == panel_id_clone>
                                    <crate::pages::apps::panel::DynamicPanel panel_id=panel_id.clone() />
                                </div>
                            }
                        }).collect_view()
                    }}

                    <div class="pane" class:active=move || active_tab.get() == "seed_data">
                        <div class="bg-[#111520] border border-outline-variant/30 rounded-xl p-6 shadow-sm">
                            <SeedPicker app_instance_id=site_id_stored.get_value() />
                        </div>
                    </div>

                    <div class="pane" class:active=move || active_tab.get() == "domains">
                        <div class="space-y-6">
                            <div class="flex justify-between items-center bg-[#111520] p-6 rounded-xl border border-outline-variant/30 shadow-sm">
                                <div>
                                    <h3 class="text-lg font-medium">"Custom Hostnames"</h3>
                                    <p class="text-sm text-muted-foreground">"Manage DNS routing for this application instance. Tenant traffic routes here natively."</p>
                                </div>
                                <Button variant=ButtonVariant::Default on:click=move |_| set_show_domain_modal.set(true)>
                                    "Add Domain"
                                </Button>
                            </div>
                            
                            <div class="bg-[#111520] border border-outline-variant/30 rounded-xl shadow-sm overflow-hidden">
                                <table class="w-full text-left border-collapse">
                                    <thead>
                                        <tr class="bg-[#0A0C16] border-b border-outline-variant/20 text-xs tracking-wider uppercase text-muted-foreground">
                                            <th class="px-6 py-4 font-medium">"Domain Name"</th>
                                            <th class="px-6 py-4 font-medium">"Edge SSL Status"</th>
                                            <th class="px-6 py-4 font-medium text-right">"Actions"</th>
                                        </tr>
                                    </thead>
                                    <tbody class="divide-y divide-outline-variant/10">
                                        <Suspense fallback=move || view! { <tr><td colspan="3" class="p-6 text-center text-muted-foreground">"Loading connected routes..."</td></tr> }>
                                            {move || {
                                                match domains_res.get() {
                                                    Some(domains) if domains.is_empty() => {
                                                        view! {
                                                            <tr>
                                                                <td colspan="3" class="px-6 py-8 text-center text-muted-foreground">
                                                                    "No custom domains attached. Traffic uses primary wildcard via instance ID."
                                                                </td>
                                                            </tr>
                                                        }.into_any()
                                                    },
                                                    Some(domains) => {
                                                        domains.into_iter().map(|domain| {
                                                            view! {
                                                                <tr class="hover:bg-muted/30 transition-colors">
                                                                    <td class="px-6 py-4 font-mono text-sm text-primary">
                                                                        {domain.clone()}
                                                                    </td>
                                                                    <td class="px-6 py-4">
                                                                        <Badge intent=BadgeIntent::Success>"Active / Managed"</Badge>
                                                                    </td>
                                                                    <td class="px-6 py-4 text-right">
                                                                        <button
                                                                            class="text-destructive hover:underline text-xs font-bold uppercase tracking-widest"
                                                                            on:click={
                                                                                let d = domain.clone();
                                                                                let toast = toast;
                                                                                move |_| {
                                                                                    let sid = site_id_stored.get_value();
                                                                                    let d = d.clone();
                                                                                    leptos::task::spawn_local(async move {
                                                                                        match crate::api::admin::remove_app_domain(sid, d).await {
                                                                                            Ok(_) => toast.show_toast("Domains", "Domain detached.", "success"),
                                                                                            Err(e) => toast.show_toast("Error", &format!("Failed: {}", e), "error"),
                                                                                        }
                                                                                    });
                                                                                }
                                                                            }
                                                                        >
                                                                            "DELETE"
                                                                        </button>
                                                                    </td>
                                                                </tr>
                                                            }
                                                        }).collect_view().into_any()
                                                    },
                                                    None => view! { <tr></tr> }.into_any()
                                                }
                                            }}
                                        </Suspense>
                                    </tbody>
                                </table>
                            </div>
                        </div>
                    </div>
                </div>

            <Show when=move || show_add_listing.get()>
                <div class="fixed inset-0 z-[100] bg-black/80 backdrop-blur-sm flex items-center justify-center p-4">
                    <div class="bg-card w-full max-w-md p-6 rounded-2xl border border-white/10 shadow-2xl relative">
                        <button class="absolute top-4 right-4 text-slate-400 hover:text-white" on:click=move |_| set_show_add_listing.set(false)>"✕"</button>
                        <h3 class="text-xl font-semibold mb-2 text-foreground">"Register Business"</h3>
                        <p class="text-muted-foreground text-sm mb-6">"Add a new commercial entity to this active network."</p>
                        <div class="space-y-4 mb-6">
                            <div class="grid gap-2 text-left">
                                <Label>"Business Name"</Label>
                                <Input r#type=InputType::Text placeholder="e.g. Acme Corp".to_string() bind_value=RwSignal::new("".to_string()) />
                            </div>
                        </div>
                        <div class="flex justify-end gap-3">
                            <Button variant=ButtonVariant::Outline on:click=move |_| set_show_add_listing.set(false)>"Cancel"</Button>
                            <Button
                                variant=ButtonVariant::Default
                                attr:disabled=true
                                attr:title="Per-instance listing registration requires a network_id — use Network → Listings"
                                class="opacity-40 cursor-not-allowed".to_string()
                            >"Save Listing"</Button>
                        </div>
                    </div>
                </div>
            </Show>

            <Show when=move || editing_listing.get().is_some()>
                <div class="fixed inset-0 z-[100] bg-black/80 backdrop-blur-sm flex items-center justify-center p-4">
                    <div class="bg-card w-full max-w-md p-6 rounded-2xl border border-white/10 shadow-2xl relative">
                        <button class="absolute top-4 right-4 text-slate-400 hover:text-white"
                            on:click=move |_| set_editing_listing.set(None)>"✕"</button>
                        <h3 class="text-xl font-semibold mb-2 text-foreground">
                            {move || format!("Edit {}", editing_listing.get().map(|(_, n)| n).unwrap_or_default())}
                        </h3>
                        <p class="text-muted-foreground text-sm mb-6">"Update metadata properties."</p>
                        <div class="space-y-4 mb-6">
                            <div class="grid gap-2 text-left">
                                <Label>"Organization Alias"</Label>
                                <Input r#type=InputType::Text bind_value=editing_alias />
                            </div>
                        </div>
                        <div class="flex justify-end gap-3">
                            <Button variant=ButtonVariant::Outline on:click=move |_| set_editing_listing.set(None)>"Cancel"</Button>
                            <Button
                                variant=ButtonVariant::Default
                                attr:disabled=move || is_saving_listing.get()
                                on:click=move |_| {
                                    let Some((id, _)) = editing_listing.get() else { return; };
                                    let alias = editing_alias.get().trim().to_string();
                                    if alias.is_empty() {
                                        toast.show_toast("Validation", "Alias cannot be empty.", "error");
                                        return;
                                    }
                                    if is_saving_listing.get() { return; }
                                    is_saving_listing.set(true);
                                    let t = toast.clone();
                                    leptos::task::spawn_local(async move {
                                        match update_listing(&id, ListingUpdate {
                                            title: Some(alias),
                                            ..Default::default()
                                        }).await {
                                            Ok(_) => {
                                                t.show_toast("Saved", "Listing metadata updated.", "success");
                                                set_editing_listing.set(None);
                                                listings_res.refetch();
                                            }
                                            Err(e) => t.show_toast("Error", &format!("Update failed: {e}"), "error"),
                                        }
                                        is_saving_listing.set(false);
                                    });
                                }
                            >
                                {move || if is_saving_listing.get() { "Saving…" } else { "Apply Changes" }}
                            </Button>
                        </div>
                    </div>
                </div>
            </Show>

            <Show when=move || managing_user_name.get().is_some()>
                <div class="fixed inset-0 z-[100] bg-black/80 backdrop-blur-sm flex items-center justify-center p-4">
                    <div class="bg-card w-full max-w-md p-6 rounded-2xl border border-white/10 shadow-2xl relative">
                        <button class="absolute top-4 right-4 text-slate-400 hover:text-white" on:click=move |_| set_managing_user_name.set(None)>"✕"</button>
                        <h3 class="text-xl font-semibold mb-2 text-foreground">{move || format!("Manage {}", managing_user_name.get().unwrap_or_default())}</h3>
                        <p class="text-muted-foreground text-sm mb-6">"Configure robust access and permissions."</p>
                        <div class="flex justify-end gap-3 mt-8">
                            <Button variant=ButtonVariant::Outline on:click=move |_| set_managing_user_name.set(None)>"Close"</Button>
                            <Button
                                variant=ButtonVariant::Destructive
                                attr:disabled=true
                                attr:title="User revoke API requires a user_id — open from Profiles panel"
                                class="opacity-40 cursor-not-allowed".to_string()
                            >"Revoke Access"</Button>
                        </div>
                    </div>
                </div>
            </Show>
            
            <Show when=move || show_add_category.get()>
                <div class="fixed inset-0 z-[100] bg-black/80 backdrop-blur-sm flex items-center justify-center p-4">
                    <div class="bg-card w-full max-w-md p-6 rounded-2xl border border-white/10 shadow-2xl relative">
                        <button class="absolute top-4 right-4 text-slate-400 hover:text-white" on:click=move |_| set_show_add_category.set(false)>"✕"</button>
                        <h3 class="text-xl font-semibold mb-2 text-foreground">"Add Category"</h3>
                        <p class="text-muted-foreground text-sm mb-6">"Define a new taxonomy level for listings."</p>
                        <div class="flex justify-end gap-3 mt-8">
                            <Button variant=ButtonVariant::Outline on:click=move |_| set_show_add_category.set(false)>"Cancel"</Button>
                            <Button
                                variant=ButtonVariant::Default
                                attr:disabled=true
                                attr:title="Category create endpoint pending — use Network → Categories"
                                class="opacity-40 cursor-not-allowed".to_string()
                            >"Save"</Button>
                        </div>
                    </div>
                </div>
            </Show>
            
            <Show when=move || show_add_template.get()>
                <div class="fixed inset-0 z-[100] bg-black/80 backdrop-blur-sm flex items-center justify-center p-4">
                    <div class="bg-card w-full max-w-md p-6 rounded-2xl border border-white/10 shadow-2xl relative">
                        <button class="absolute top-4 right-4 text-slate-400 hover:text-white" on:click=move |_| set_show_add_template.set(false)>"✕"</button>
                        <h3 class="text-xl font-semibold mb-2 text-foreground">"Assign Template"</h3>
                        <p class="text-muted-foreground text-sm mb-6">"Link a structural template to format listings here."</p>
                        <div class="flex justify-end gap-3 mt-8">
                            <Button variant=ButtonVariant::Outline on:click=move |_| set_show_add_template.set(false)>"Cancel"</Button>
                            <Button
                                variant=ButtonVariant::Default
                                attr:disabled=true
                                attr:title="Template assignment endpoint pending — use Network → Templates"
                                class="opacity-40 cursor-not-allowed".to_string()
                            >"Save"</Button>
                        </div>
                    </div>
                </div>
            </Show>
            <Show when=move || show_domain_modal.get()>
                <div class="fixed inset-0 z-[100] bg-black/80 backdrop-blur-sm flex items-center justify-center p-4">
                    <div class="bg-card w-full max-w-md p-6 rounded-2xl border border-white/10 shadow-2xl relative">
                        <button class="absolute top-4 right-4 text-slate-400 hover:text-white" on:click=move |_| set_show_domain_modal.set(false)>"✕"</button>
                        <h3 class="text-xl font-semibold mb-2 text-foreground">"Attach Domain"</h3>
                        <p class="text-muted-foreground text-sm mb-6">"Provision a new hostname. A Cloudflare SSL certificate will be automatically requested."</p>
                        <div class="space-y-4 mb-6">
                            <div class="grid gap-2 text-left">
                                <Label>"Hostname (e.g. dev.buildwithruud.com)"</Label>
                                <Input r#type=InputType::Text bind_value=new_domain_input />
                            </div>
                        </div>
                        <div class="flex justify-end gap-3">
                            <Button variant=ButtonVariant::Outline on:click=move |_| set_show_domain_modal.set(false)>"Cancel"</Button>
                            <Button variant=ButtonVariant::Default on:click=move |_| {
                                let d = new_domain_input.get();
                                add_domain_action.dispatch(d);
                                set_show_domain_modal.set(false);
                            }>"Provision Pipeline"</Button>
                        </div>
                    </div>
                </div>
            </Show>
        </div>
        </Show>
    }
}
