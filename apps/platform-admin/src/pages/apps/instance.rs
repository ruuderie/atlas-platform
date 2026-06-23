use leptos::prelude::*;
use leptos_router::hooks::use_params_map;
use crate::api::admin::{
    get_public_config, update_public_config,
    suspend_instance, resume_instance,
    upsert_module,
};

#[component]
pub fn AppInstance() -> impl IntoView {
    let params = use_params_map();
    let instance_id_str = move || params.with(|p| p.get("id").unwrap_or_else(|| "inst_pm_a1b2c3".to_string()));

    // Global toast context
    let toast = use_context::<crate::app::GlobalToast>().expect("toast context");

    // ── Load real instance config from API ──
    let instance_config = LocalResource::new(move || {
        let id_str = instance_id_str();
        async move {
            match uuid::Uuid::parse_str(&id_str) {
                Ok(id) => get_public_config(id).await.ok(),
                Err(_) => None,
            }
        }
    });

    // Reactive states
    let active_tab = RwSignal::new("t-overview".to_string());
    let active_instance_type = RwSignal::new("folio".to_string());
    let is_suspended = RwSignal::new(false);
    let show_suspend_modal = RwSignal::new(false);
    let show_provision_modal = RwSignal::new(false);
    let show_add_rail_modal = RwSignal::new(false);
    let show_edit_config_modal = RwSignal::new(false);
    let suspend_reason = RwSignal::new(String::new());

    // Form input states — pre-seeded with sensible defaults;
    // the Edit Config modal allows the operator to change them.
    let jurisdiction_code = RwSignal::new("US-FL".to_string());
    let market_config = RwSignal::new("MiamiDadeMarket".to_string());
    let str_ordinance = RwSignal::new("Miami-Dade Ord. 2023-89".to_string());
    let tdt_rate = RwSignal::new("7% (Miami-Dade)".to_string());
    let deployment_mode = RwSignal::new("standard".to_string());
    let lookback_hours = RwSignal::new("25".to_string());
    // public_slug / custom_domain are seeded from instance_config once loaded
    let custom_domain = RwSignal::new(String::new());
    let public_slug = RwSignal::new(String::new());

    // Seed public_slug / custom_domain / is_suspended from API once loaded
    let _seed_effect = Effect::new(move |_| {
        if let Some(Some(cfg)) = instance_config.get() {
            if let Some(slug) = cfg.public_slug {
                public_slug.set(slug);
            }
            if let Some(domain) = cfg.custom_domain {
                custom_domain.set(domain);
            }
            is_suspended.set(cfg.instance_status == "suspended");
        }
    });

    // Module toggle signals
    let module_portfolio = RwSignal::new(true);
    let module_leases = RwSignal::new(true);
    let module_maintenance = RwSignal::new(true);
    let module_vendors = RwSignal::new(true);
    let module_crm = RwSignal::new(true);
    let module_billing = RwSignal::new(true);
    let module_str = RwSignal::new(true);
    let module_leads = RwSignal::new(true);
    let module_opps = RwSignal::new(true);
    let module_events = RwSignal::new(false);
    let module_vault = RwSignal::new(true);
    let module_reporting = RwSignal::new(true);
    let module_violations = RwSignal::new(true);
    let module_geo = RwSignal::new(true);

    let pix_status = RwSignal::new("Disabled".to_string());

    // ── Derive tenant_id from instance config ──
    let tenant_id_sig = move || {
        instance_config.get().flatten().map(|c| c.tenant_id)
    };

    // ── Derived app_slug display ──
    let app_slug_display = move || {
        instance_config.get()
            .flatten()
            .map(|c| c.app_slug.clone())
            .unwrap_or_else(|| instance_id_str().chars().take(12).collect())
    };

    // ── Suspend handler ──
    let handle_suspend = move |_| {
        let reason = suspend_reason.get();
        let id_str = instance_id_str();
        let t = toast.clone();
        leptos::task::spawn_local(async move {
            if let Ok(id) = uuid::Uuid::parse_str(&id_str) {
                match suspend_instance(id, reason).await {
                    Ok(_) => {
                        is_suspended.set(true);
                        show_suspend_modal.set(false);
                        t.show_toast("Success", "App instance suspended.", "success");
                    }
                    Err(e) => {
                        t.show_toast("Error", &format!("Suspend failed: {}", e), "error");
                    }
                }
            } else {
                t.show_toast("Error", "Invalid instance ID.", "error");
            }
        });
    };

    // ── Resume handler ──
    let handle_resume = move |_| {
        let id_str = instance_id_str();
        let t = toast.clone();
        leptos::task::spawn_local(async move {
            if let Ok(id) = uuid::Uuid::parse_str(&id_str) {
                match resume_instance(id).await {
                    Ok(_) => {
                        is_suspended.set(false);
                        t.show_toast("Success", "App instance reactivated.", "success");
                    }
                    Err(e) => {
                        t.show_toast("Error", &format!("Resume failed: {}", e), "error");
                    }
                }
            } else {
                t.show_toast("Error", "Invalid instance ID.", "error");
            }
        });
    };

    // ── Save config handler ──
    let handle_save_config = move |_| {
        let id_str = instance_id_str();
        let slug = public_slug.get();
        let domain = custom_domain.get();
        let t = toast.clone();
        leptos::task::spawn_local(async move {
            if let Ok(id) = uuid::Uuid::parse_str(&id_str) {
                let slug_val = if slug.is_empty() { None } else { Some(slug) };
                let domain_val = if domain.is_empty() { None } else { Some(domain) };
                match update_public_config(id, slug_val, domain_val).await {
                    Ok(_) => {
                        show_edit_config_modal.set(false);
                        t.show_toast("Saved", "Instance configuration updated.", "success");
                    }
                    Err(e) => {
                        t.show_toast("Error", &format!("Save failed: {}", e), "error");
                    }
                }
            } else {
                t.show_toast("Error", "Invalid instance ID.", "error");
            }
        });
    };

    // ── Module toggle helper — persists to backend ──
    let make_module_handler = move |module_type: &'static str, signal: RwSignal<bool>| {
        move |checked: bool| {
            signal.set(checked);
            if let Some(tid) = tenant_id_sig() {
                let t = toast.clone();
                leptos::task::spawn_local(async move {
                    if let Err(e) = upsert_module(tid, module_type, checked, None, None).await {
                        t.show_toast("Warning", &format!("Module save failed: {}", e), "error");
                    }
                });
            }
        }
    };

    // Switch instance helper
    let select_instance = move |inst_type: &str| {
        active_instance_type.set(inst_type.to_string());
        toast.show_toast(
            "Instance Switched",
            &format!("Switched view to {} context", inst_type.to_uppercase()),
            "success"
        );
    };

    view! {
        <div class="space-y-6">
            // ── Breadcrumb ──
            <nav class="flex items-center gap-2 text-on-surface-variant text-xs mb-2">
                <a href="/apps" class="hover:text-primary transition-colors">"Tenants"</a>
                <span class="material-symbols-outlined text-[12px]">"chevron_right"</span>
                <span class="text-on-surface-variant/80">
                    {move || instance_config.get().flatten().map(|c| c.tenant_id.to_string().chars().take(8).collect::<String>() + "…").unwrap_or_else(|| "—".to_string())}
                </span>
                <span class="material-symbols-outlined text-[12px]">"chevron_right"</span>
                <span class="text-on-surface-variant/80">"App Instances"</span>
                <span class="material-symbols-outlined text-[12px]">"chevron_right"</span>
                <span class="text-primary/70">{move || app_slug_display()}</span>
            </nav>

            // ── App Header ──
            <div class="flex flex-col md:flex-row justify-between items-start md:items-center gap-4 bg-surface-container-low border border-outline-variant/20 p-6 rounded-2xl shadow-sm">
                <div class="flex items-center gap-4">
                    <div class="w-12 h-12 bg-primary/10 border border-primary/40 text-primary rounded-xl flex items-center justify-center font-black text-lg shadow-inner shadow-primary/5">
                        {move || match active_instance_type.get().as_str() {
                            "str" => "STR",
                            "anchor" => "⚓",
                            "network" => "NET",
                            _ => "PM"
                        }}
                    </div>
                    <div>
                        <div class="flex items-center gap-3">
                            <h1 class="text-2xl font-extrabold text-on-surface tracking-tight">
                                {move || match active_instance_type.get().as_str() {
                                    "str" => "Atlas STR — Miami",
                                    "anchor" => "Anchor",
                                    "network" => "Network Directory",
                                    _ => "Atlas PM — Residential"
                                }}
                            </h1>
                            {move || if is_suspended.get() {
                                view! {
                                    <span class="inline-flex items-center gap-1.5 px-2.5 py-0.5 rounded-full text-[10px] font-bold bg-error-container/20 text-error border border-error/30 uppercase tracking-wider">
                                        <span class="w-1.5 h-1.5 rounded-full bg-error animate-pulse"></span>
                                        "Suspended"
                                    </span>
                                }.into_any()
                            } else {
                                view! {
                                    <span class="inline-flex items-center gap-1.5 px-2.5 py-0.5 rounded-full text-[10px] font-bold bg-emerald-500/10 text-emerald-400 border border-emerald-500/20 uppercase tracking-wider">
                                        <span class="w-1.5 h-1.5 rounded-full bg-emerald-400 animate-pulse"></span>
                                        "Live"
                                    </span>
                                }.into_any()
                            }}
                        </div>
                        <div class="text-xs text-on-surface-variant font-mono mt-1 select-all">
                            {move || format!("app_id: {} · {}.atlas.app · inst: {}", 
                                if active_instance_type.get() == "anchor" { "anchor" } else if active_instance_type.get() == "network" { "network_instance" } else { "property_management" },
                                public_slug.get(),
                                instance_id_str()
                            )}
                        </div>
                        <div class="flex flex-wrap gap-2 mt-3">
                            <span class="px-2 py-0.5 rounded bg-primary-container/20 border border-primary/20 text-primary text-[10px] font-bold uppercase tracking-wider">"Folio"</span>
                            <span class="px-2 py-0.5 rounded bg-surface-container-high border border-outline-variant/30 text-on-surface-variant text-[10px] font-bold uppercase tracking-wider">
                                {move || {
                                    let code = jurisdiction_code.get();
                                    format!("Jurisdiction: US — {}", if code == "US-FL" { "Florida" } else { &code })
                                }}
                            </span>
                            <span class="px-2 py-0.5 rounded bg-purple-500/10 border border-purple-500/20 text-purple-400 text-[10px] font-bold uppercase tracking-wider">"G-27 Auto-seeded"</span>
                            <span class="px-2 py-0.5 rounded bg-surface-container-high border border-outline-variant/30 text-on-surface-variant text-[10px] font-bold uppercase tracking-wider">"4 background jobs"</span>
                        </div>
                    </div>
                </div>
                <div class="flex items-center gap-3">
                    {move || if is_suspended.get() {
                        view! {
                            <button 
                                class="bg-emerald-500/15 border border-emerald-500/30 text-emerald-400 hover:bg-emerald-500/25 px-4 py-2 rounded-lg text-sm font-semibold transition-all active:scale-95"
                                on:click=handle_resume
                            >
                                "Activate Instance"
                            </button>
                        }.into_any()
                    } else {
                        view! {
                            <button 
                                class="bg-error-container/20 border border-error/30 text-error hover:bg-error-container/30 px-4 py-2 rounded-lg text-sm font-semibold transition-all active:scale-95"
                                on:click=move |_| show_suspend_modal.set(true)
                            >
                                "Suspend Instance"
                            </button>
                        }.into_any()
                    }}
                    <button 
                        class="btn-primary-gradient px-4 py-2 rounded-lg text-sm font-semibold text-on-primary-container shadow-md shadow-primary/10 hover:opacity-90 active:scale-95 transition-all"
                        on:click=handle_save_config
                    >
                        "Save Changes"
                    </button>
                </div>
            </div>

            // ── App Switcher Pill Row ──
            <div class="flex items-center gap-2 overflow-x-auto bg-surface-container-low border border-outline-variant/10 rounded-xl p-3 shadow-inner">
                <span class="text-[10px] font-bold text-on-surface-variant/60 uppercase tracking-wider select-none shrink-0 px-2">"Switch Instance:"</span>
                <button 
                    class=move || if active_instance_type.get() == "folio" { "flex items-center gap-2 px-3 py-1.5 bg-surface-container-highest text-primary border border-outline-variant rounded-lg font-semibold text-xs shrink-0 transition-all shadow-sm" } else { "flex items-center gap-2 px-3 py-1.5 text-on-surface-variant hover:bg-surface-bright/20 hover:text-on-surface rounded-lg font-semibold text-xs shrink-0 transition-all border border-transparent" }
                    on:click=move |_| select_instance("folio")
                >
                    <span class="w-4 h-4 bg-primary/20 text-primary flex items-center justify-center rounded font-black text-[8px]">"PM"</span>
                    "Atlas PM — Residential"
                    <span class="text-emerald-400 text-[9px]">"●"</span>
                </button>
                <button 
                    class=move || if active_instance_type.get() == "str" { "flex items-center gap-2 px-3 py-1.5 bg-surface-container-highest text-primary border border-outline-variant rounded-lg font-semibold text-xs shrink-0 transition-all shadow-sm" } else { "flex items-center gap-2 px-3 py-1.5 text-on-surface-variant hover:bg-surface-bright/20 hover:text-on-surface rounded-lg font-semibold text-xs shrink-0 transition-all border border-transparent" }
                    on:click=move |_| select_instance("str")
                >
                    <span class="w-4 h-4 bg-amber-500/20 text-amber-400 flex items-center justify-center rounded font-black text-[8px]">"STR"</span>
                    "Atlas STR — Miami"
                    <span class="text-emerald-400 text-[9px]">"●"</span>
                </button>
                <button 
                    class=move || if active_instance_type.get() == "anchor" { "flex items-center gap-2 px-3 py-1.5 bg-surface-container-highest text-primary border border-outline-variant rounded-lg font-semibold text-xs shrink-0 transition-all shadow-sm" } else { "flex items-center gap-2 px-3 py-1.5 text-on-surface-variant hover:bg-surface-bright/20 hover:text-on-surface rounded-lg font-semibold text-xs shrink-0 transition-all border border-transparent" }
                    on:click=move |_| select_instance("anchor")
                >
                    <span class="w-4 h-4 bg-purple-500/20 text-purple-400 flex items-center justify-center rounded font-black text-[8px]">"⚓"</span>
                    "Anchor (CMS)"
                    <span class="text-on-surface-variant/40 text-[9px]">"●"</span>
                </button>
                <button 
                    class=move || if active_instance_type.get() == "network" { "flex items-center gap-2 px-3 py-1.5 bg-surface-container-highest text-primary border border-outline-variant rounded-lg font-semibold text-xs shrink-0 transition-all shadow-sm" } else { "flex items-center gap-2 px-3 py-1.5 text-on-surface-variant hover:bg-surface-bright/20 hover:text-on-surface rounded-lg font-semibold text-xs shrink-0 transition-all border border-transparent" }
                    on:click=move |_| select_instance("network")
                >
                    <span class="w-4 h-4 bg-white/10 text-on-surface-variant flex items-center justify-center rounded font-black text-[8px]">"NET"</span>
                    "Network Directory"
                    <span class="text-emerald-400 text-[9px]">"●"</span>
                </button>
                <button 
                    class="ml-auto flex items-center gap-1.5 bg-surface-bright/40 text-primary border border-primary/20 hover:bg-surface-bright/80 px-3 py-1.5 rounded-lg text-xs font-semibold shrink-0 transition-all"
                    on:click=move |_| show_provision_modal.set(true)
                >
                    <span class="material-symbols-outlined text-[14px]">"add"</span>
                    "Provision New"
                </button>
            </div>

            // ── Tab Bar Navigation ──
            <div class="flex border-b border-outline-variant/20 overflow-x-auto shrink-0 select-none">
                {
                    let tab_button = move |id: &str, label: &str| {
                        let id = id.to_string();
                        let label = label.to_string();
                        let id_class = id.clone();
                        let id_click = id.clone();
                        view! {
                            <button 
                                class=move || if active_tab.get() == id_class { "px-4 py-2.5 text-sm font-semibold text-primary border-b-2 border-primary transition-all shrink-0 bg-transparent" } else { "px-4 py-2.5 text-sm text-on-surface-variant hover:text-on-surface transition-all shrink-0 bg-transparent" }
                                on:click=move |_| active_tab.set(id_click.clone())
                            >
                                {label.clone()}
                            </button>
                        }
                    };
                    view! {
                        {tab_button("t-overview", "Overview")}
                        {tab_button("t-onboarding", "Onboarding")}
                        {tab_button("t-modules", "Modules")}
                        {tab_button("t-folio-config", "App Config (Folio)")}
                        {tab_button("t-scorecards", "G-27 Scorecards")}
                        {tab_button("t-jobs", "Background Jobs")}
                        {tab_button("t-domain", "Domains & Routing")}
                        {tab_button("t-compare", "App Type Comparison")}
                    }
                }
            </div>

            // ── TAB CONTENT: Overview ──
            <Show when=move || active_tab.get() == "t-overview">
                <div class="grid grid-cols-1 lg:grid-cols-2 gap-6">
                    <div class="space-y-6">
                        // Card 1
                        <div class="bg-surface-container-low border border-outline-variant/20 rounded-xl overflow-hidden shadow-sm">
                            <div class="flex justify-between items-center px-5 py-3.5 border-b border-outline-variant/20 bg-surface-container-high/40">
                                <h3 class="text-xs font-bold uppercase tracking-wider text-on-surface-variant flex items-center gap-2">
                                    <span class="material-symbols-outlined text-[16px] text-primary">"info"</span>
                                    "App Instance Identity"
                                </h3>
                                <button class="text-xs text-primary hover:underline hover:opacity-80 transition-opacity" on:click=move |_| show_edit_config_modal.set(true)>"Edit"</button>
                            </div>
                            <div class="divide-y divide-outline-variant/10">
                                <div class="flex justify-between items-center px-5 py-3 text-xs">
                                    <span class="text-on-surface-variant">"App Type"</span>
                                    <span class="font-bold text-primary">"Folio — Property Management"</span>
                                </div>
                                <div class="flex justify-between items-center px-5 py-3 text-xs">
                                    <span class="text-on-surface-variant">"app_id"</span>
                                    <span class="font-mono text-on-surface/80">"property_management"</span>
                                </div>
                                <div class="flex justify-between items-center px-5 py-3 text-xs">
                                    <span class="text-on-surface-variant">"Instance ID"</span>
                                    <span class="font-mono text-on-surface/80">{instance_id_str()}</span>
                                </div>
                                <div class="flex justify-between items-center px-5 py-3 text-xs">
                                    <span class="text-on-surface-variant">"Tenant"</span>
                                    <span class="font-medium font-mono">
                                        {move || instance_config.get().flatten()
                                            .map(|c| c.tenant_id.to_string())
                                            .unwrap_or_else(|| "—".to_string())}
                                    </span>
                                </div>
                                <div class="flex justify-between items-center px-5 py-3 text-xs">
                                    <span class="text-on-surface-variant">"Subdomain"</span>
                                    <span class="font-mono text-on-surface/80">{move || format!("{}.atlas.app", public_slug.get())}</span>
                                </div>
                                <div class="flex justify-between items-center px-5 py-3 text-xs">
                                    <span class="text-on-surface-variant">"Custom Domain"</span>
                                    <span class="font-mono text-on-surface/80">{move || custom_domain.get()}</span>
                                </div>
                                <div class="flex justify-between items-center px-5 py-3 text-xs">
                                    <span class="text-on-surface-variant">"Status"</span>
                                    {move || if is_suspended.get() {
                                        view! { <span class="text-error font-semibold">"● Suspended"</span> }.into_any()
                                    } else {
                                        view! { <span class="text-emerald-400 font-semibold">"● Live"</span> }.into_any()
                                    }}
                                </div>
                                <div class="flex justify-between items-center px-5 py-3 text-xs">
                                    <span class="text-on-surface-variant">"Provisioned"</span>
                                    <span class="text-on-surface-variant/80">"Feb 14, 2024"</span>
                                </div>
                            </div>
                        </div>

                        // Card 2
                        <div class="bg-surface-container-low border border-outline-variant/20 rounded-xl overflow-hidden shadow-sm">
                            <div class="px-5 py-3.5 border-b border-outline-variant/20 bg-surface-container-high/40">
                                <h3 class="text-xs font-bold uppercase tracking-wider text-on-surface-variant flex items-center gap-2">
                                    <span class="material-symbols-outlined text-[16px] text-primary">"analytics"</span>
                                    "Platform Activity"
                                </h3>
                            </div>
                            <div class="divide-y divide-outline-variant/10">
                                <div class="flex justify-between items-center px-5 py-3 text-xs">
                                    <span class="text-on-surface-variant">"Properties (atlas_assets)"</span>
                                    <span class="font-bold font-mono">"87"</span>
                                </div>
                                <div class="flex justify-between items-center px-5 py-3 text-xs">
                                    <span class="text-on-surface-variant">"Active Leases"</span>
                                    <span class="font-bold font-mono">"62"</span>
                                </div>
                                <div class="flex justify-between items-center px-5 py-3 text-xs">
                                    <span class="text-on-surface-variant">"Total Leads (G-31)"</span>
                                    <span class="font-bold font-mono">"342"</span>
                                </div>
                                <div class="flex justify-between items-center px-5 py-3 text-xs">
                                    <span class="text-on-surface-variant">"Active Vendors"</span>
                                    <span class="font-bold font-mono">"14"</span>
                                </div>
                                <div class="flex justify-between items-center px-5 py-3 text-xs">
                                    <span class="text-on-surface-variant">"Open Cases"</span>
                                    <span class="font-bold font-mono text-amber-400">"8"</span>
                                </div>
                                <div class="flex justify-between items-center px-5 py-3 text-xs">
                                    <span class="text-on-surface-variant">"G-27 Scorecards"</span>
                                    <span class="font-bold font-mono">"342"</span>
                                </div>
                                <div class="flex justify-between items-center px-5 py-3 text-xs">
                                    <span class="text-on-surface-variant">"STR Permits Tracked"</span>
                                    <span class="font-bold font-mono">"24"</span>
                                </div>
                            </div>
                        </div>
                    </div>

                    <div class="space-y-6">
                        // Card 3
                        <div class="bg-surface-container-low border border-outline-variant/20 rounded-xl overflow-hidden shadow-sm border-l-4 border-l-primary">
                            <div class="flex justify-between items-center px-5 py-3.5 border-b border-outline-variant/20 bg-surface-container-high/40">
                                <h3 class="text-xs font-bold uppercase tracking-wider text-on-surface-variant flex items-center gap-2">
                                    <span class="material-symbols-outlined text-[16px] text-primary">"settings_ethernet"</span>
                                    "Folio-Specific Config · atlas_app_deployment_config"
                                </h3>
                                <button class="text-xs text-primary hover:underline hover:opacity-80 transition-opacity" on:click=move |_| show_edit_config_modal.set(true)>"Edit Raw JSON"</button>
                            </div>
                            <div class="divide-y divide-outline-variant/10">
                                <div class="flex justify-between items-center px-5 py-3 text-xs">
                                    <span class="text-on-surface-variant">"Deployment Mode"</span>
                                    <span class="font-bold text-primary">{move || deployment_mode.get()}</span>
                                </div>
                                <div class="flex justify-between items-center px-5 py-3 text-xs">
                                    <span class="text-on-surface-variant">"folio_jurisdiction_code"</span>
                                    <span class="font-mono text-on-surface/80">{move || jurisdiction_code.get()}</span>
                                </div>
                                <div class="flex justify-between items-center px-5 py-3 text-xs">
                                    <span class="text-on-surface-variant">"folio_market_config"</span>
                                    <span class="font-mono text-on-surface/80">{move || market_config.get()}</span>
                                </div>
                                <div class="flex justify-between items-center px-5 py-3 text-xs">
                                    <span class="text-on-surface-variant">"folio_role (primary)"</span>
                                    <span class="font-medium">"Landlord"</span>
                                </div>
                                <div class="flex justify-between items-center px-5 py-3 text-xs">
                                    <span class="text-on-surface-variant">"folio_roles_enabled"</span>
                                    <span class="font-mono text-on-surface/70">"Landlord, Tenant, Vendor, Owner"</span>
                                </div>
                                <div class="flex justify-between items-center px-5 py-3 text-xs">
                                    <span class="text-on-surface-variant">"scorecard_display_rules_enabled"</span>
                                    <span class="text-emerald-400 font-medium">"true"</span>
                                </div>
                                <div class="flex justify-between items-center px-5 py-3 text-xs">
                                    <span class="text-on-surface-variant">"folio_payment_rails_configured"</span>
                                    <span class="text-emerald-400 font-medium">"true"</span>
                                </div>
                                <div class="flex justify-between items-center px-5 py-3 text-xs">
                                    <span class="text-on-surface-variant">"G-33 / PMC Enabled"</span>
                                    <span class="text-primary font-medium">"true (via config JSON)"</span>
                                </div>
                                <div class="flex justify-between items-center px-5 py-3 text-xs">
                                    <span class="text-on-surface-variant">"managed_account_id"</span>
                                    <span class="font-mono text-on-surface-variant/60">"null (single-landlord)"</span>
                                </div>
                            </div>
                        </div>

                        // Card 4
                        <div class="bg-surface-container-low border border-outline-variant/20 rounded-xl overflow-hidden shadow-sm">
                            <div class="px-5 py-3.5 border-b border-outline-variant/20 bg-surface-container-high/40">
                                <h3 class="text-xs font-bold uppercase tracking-wider text-on-surface-variant flex items-center gap-2">
                                    <span class="material-symbols-outlined text-[16px] text-primary">"speed"</span>
                                    "G-27 Health · This App Instance"
                                </h3>
                            </div>
                            <div class="p-6 border-b border-outline-variant/10 flex items-center gap-4 bg-surface-container-high/10">
                                <div class="text-4xl font-extrabold tracking-tighter text-emerald-400 font-mono">"9.2"</div>
                                <div>
                                    <div class="text-sm font-bold text-emerald-400">"Outstanding"</div>
                                    <div class="text-xs text-on-surface-variant/70">"Tenant Health Score · 47 samples"</div>
                                </div>
                            </div>
                            <div class="p-5 space-y-4">
                                <div>
                                    <div class="flex justify-between text-xs mb-1">
                                        <span class="text-on-surface-variant">"Contractor Scoring"</span>
                                        <span class="font-bold text-emerald-400">"8.1"</span>
                                    </div>
                                    <div class="w-full h-1.5 bg-surface-container rounded-full overflow-hidden">
                                        <div class="h-full bg-emerald-400 rounded-full" style="width: 81%"></div>
                                    </div>
                                </div>
                                <div>
                                    <div class="flex justify-between text-xs mb-1">
                                        <span class="text-on-surface-variant">"Listing Quality"</span>
                                        <span class="font-bold text-emerald-400">"8.4"</span>
                                    </div>
                                    <div class="w-full h-1.5 bg-surface-container rounded-full overflow-hidden">
                                        <div class="h-full bg-emerald-400 rounded-full" style="width: 84%"></div>
                                    </div>
                                </div>
                                <div>
                                    <div class="flex justify-between text-xs mb-1">
                                        <span class="text-on-surface-variant">"Deal Qualification"</span>
                                        <span class="font-bold text-emerald-400">"7.6"</span>
                                    </div>
                                    <div class="w-full h-1.5 bg-surface-container rounded-full overflow-hidden">
                                        <div class="h-full bg-emerald-400 rounded-full" style="width: 76%"></div>
                                    </div>
                                </div>
                                <div>
                                    <div class="flex justify-between text-xs mb-1">
                                        <span class="text-on-surface-variant">"Tenant Health"</span>
                                        <span class="font-bold text-emerald-400">"9.2"</span>
                                    </div>
                                    <div class="w-full h-1.5 bg-surface-container rounded-full overflow-hidden">
                                        <div class="h-full bg-emerald-400 rounded-full" style="width: 92%"></div>
                                    </div>
                                </div>
                            </div>
                        </div>
                    </div>
                </div>
            </Show>

            // ── TAB CONTENT: Onboarding ──
            <Show when=move || active_tab.get() == "t-onboarding">
                <div class="grid grid-cols-1 lg:grid-cols-2 gap-6">
                    <div class="bg-surface-container-low border border-outline-variant/20 rounded-xl overflow-hidden shadow-sm">
                        <div class="px-5 py-3.5 border-b border-outline-variant/20 bg-surface-container-high/40">
                            <h3 class="text-xs font-bold uppercase tracking-wider text-on-surface-variant">
                                "Onboarding Progress · Folio (property_management)"
                            </h3>
                        </div>
                        <div class="divide-y divide-outline-variant/10">
                            // Step 1
                            <div class="p-5 flex items-start gap-4 hover:bg-surface-bright/5 transition-colors">
                                <div class="w-6 h-6 rounded-full bg-emerald-500/10 border border-emerald-400 text-emerald-400 flex items-center justify-center font-bold text-xs shrink-0 mt-0.5">"✓"</div>
                                <div class="flex-1 min-w-0">
                                    <div class="flex items-center gap-2">
                                        <h4 class="text-sm font-bold">"1 · Jurisdiction Setup"</h4>
                                        <span class="text-[9px] font-extrabold uppercase bg-emerald-500/10 text-emerald-400 px-1.5 py-0.5 rounded border border-emerald-500/20">"Required"</span>
                                    </div>
                                    <p class="text-xs text-on-surface-variant mt-1">"Configure operating jurisdiction (US, Brazil, USVI, DR, Haiti) for tax, compliance, and payment rails."</p>
                                    <div class="text-[10px] font-mono text-on-surface-variant/60 mt-2">"tenant_setting: folio_jurisdiction_code = \"US-FL\" ✓"</div>
                                </div>
                                <button class="btn-ghost text-xs px-2.5 py-1 border border-outline-variant/30 rounded hover:bg-surface-bright/20" on:click=move |_| show_edit_config_modal.set(true)>"Edit"</button>
                            </div>
                            // Step 2
                            <div class="p-5 flex items-start gap-4 hover:bg-surface-bright/5 transition-colors">
                                <div class="w-6 h-6 rounded-full bg-emerald-500/10 border border-emerald-400 text-emerald-400 flex items-center justify-center font-bold text-xs shrink-0 mt-0.5">"✓"</div>
                                <div class="flex-1 min-w-0">
                                    <div class="flex items-center gap-2">
                                        <h4 class="text-sm font-bold">"2 · Add Your First Property"</h4>
                                        <span class="text-[9px] font-extrabold uppercase bg-emerald-500/10 text-emerald-400 px-1.5 py-0.5 rounded border border-emerald-500/20">"Required"</span>
                                    </div>
                                    <p class="text-xs text-on-surface-variant mt-1">"Register your first property to start managing leases, maintenance, and payments."</p>
                                    <div class="text-[10px] font-mono text-on-surface-variant/60 mt-2">"atlas_assets.count = 87 ≥ 1 ✓"</div>
                                </div>
                                <button class="btn-ghost text-xs px-2.5 py-1 border border-outline-variant/30 rounded hover:bg-surface-bright/20" on:click=move |_| {
                                    toast.show_toast("Info", "Properties details are managed under the assets registry.", "info");
                                }>"View"</button>
                            </div>
                            // Step 3
                            <div class="p-5 flex items-start gap-4 hover:bg-surface-bright/5 transition-colors">
                                <div class="w-6 h-6 rounded-full bg-emerald-500/10 border border-emerald-400 text-emerald-400 flex items-center justify-center font-bold text-xs shrink-0 mt-0.5">"✓"</div>
                                <div class="flex-1 min-w-0">
                                    <div class="flex items-center gap-2">
                                        <h4 class="text-sm font-bold">"3 · Payment Rails"</h4>
                                        <span class="text-[9px] font-extrabold uppercase bg-surface-container border border-outline-variant text-on-surface-variant px-1.5 py-0.5 rounded">"Optional"</span>
                                    </div>
                                    <p class="text-xs text-on-surface-variant mt-1">"Configure at least one payment method (Stripe, PIX, Bitcoin, or Zelle) so tenants can pay rent."</p>
                                    <div class="text-[10px] font-mono text-on-surface-variant/60 mt-2">"tenant_setting: folio_payment_rails_configured = \"true\" ✓"</div>
                                </div>
                                <button class="btn-ghost text-xs px-2.5 py-1 border border-outline-variant/30 rounded hover:bg-surface-bright/20" on:click=move |_| active_tab.set("t-folio-config".to_string())>"Manage"</button>
                            </div>
                        </div>
                        <div class="p-4 bg-emerald-500/10 border-t border-outline-variant/20 flex items-center gap-2 text-emerald-400 text-xs">
                            <span class="font-bold">"✓ All onboarding steps complete."</span>
                            <span class="text-on-surface-variant/80">"This instance is fully operational."</span>
                        </div>
                    </div>

                    <div class="space-y-6">
                        <div class="bg-surface-container-low border border-outline-variant/20 rounded-xl overflow-hidden shadow-sm border-l-4 border-l-amber-400">
                            <div class="px-5 py-3.5 border-b border-outline-variant/20 bg-surface-container-high/40 flex items-center gap-2">
                                <span class="text-amber-400 text-sm">"⚠"</span>
                                <h3 class="text-xs font-bold uppercase tracking-wider text-on-surface-variant">
                                    "How Folio Onboarding Differs"
                                </h3>
                            </div>
                            <div class="p-5 text-xs text-on-surface-variant leading-relaxed space-y-3">
                                <p><strong class="text-on-surface">"Folio"</strong> " requires a "<strong class="text-on-surface">"Jurisdiction"</strong>" first — it gates the entire tax/compliance/payment rail stack. No properties or payments can be configured until this is set."</p>
                                <p><strong class="text-on-surface">"Anchor"</strong> " starts with Brand Identity → Domain → Theme → First Page. No jurisdiction."</p>
                                <p><strong class="text-on-surface">"Network Instance"</strong> " starts with Identity → Domain → Categories → Listing Template → First Listing."</p>
                            </div>
                        </div>

                        <div class="bg-surface-container-low border border-outline-variant/20 rounded-xl overflow-hidden shadow-sm">
                            <div class="flex justify-between items-center px-5 py-3.5 border-b border-outline-variant/20 bg-surface-container-high/40">
                                <h3 class="text-xs font-bold uppercase tracking-wider text-on-surface-variant">
                                    "Jurisdiction Settings · MiamiDadeMarket"
                                </h3>
                                <button class="text-xs text-primary hover:underline hover:opacity-80 transition-opacity" on:click=move |_| show_edit_config_modal.set(true)>"Change"</button>
                            </div>
                            <div class="divide-y divide-outline-variant/10 text-xs">
                                <div class="flex justify-between items-center px-5 py-3">
                                    <span class="text-on-surface-variant">"Jurisdiction Code"</span>
                                    <span class="font-mono text-on-surface/80">{move || jurisdiction_code.get()}</span>
                                </div>
                                <div class="flex justify-between items-center px-5 py-3">
                                    <span class="text-on-surface-variant">"Market Config"</span>
                                    <span class="font-semibold">{move || market_config.get()}</span>
                                </div>
                                <div class="flex justify-between items-center px-5 py-3">
                                    <span class="text-on-surface-variant">"Tax Rail"</span>
                                    <span class="font-mono text-on-surface-variant/80">"TDT 7% (Miami-Dade)"</span>
                                </div>
                                <div class="flex justify-between items-center px-5 py-3">
                                    <span class="text-on-surface-variant">"Anti-Discrimination Law"</span>
                                    <span class="font-mono text-on-surface-variant/80">"FHA (Fair Housing Act)"</span>
                                </div>
                                <div class="flex justify-between items-center px-5 py-3">
                                    <span class="text-on-surface-variant">"STR Ordinance"</span>
                                    <span class="font-mono text-on-surface/80">{move || str_ordinance.get()}</span>
                                </div>
                                <div class="flex justify-between items-center px-5 py-3">
                                    <span class="text-on-surface-variant">"Renter Screening"</span>
                                    <span class="font-mono text-on-surface-variant/80">"TransUnion / Checkr"</span>
                                </div>
                                <div class="flex justify-between items-center px-5 py-3">
                                    <span class="text-on-surface-variant">"Currency"</span>
                                    <span class="font-mono font-bold">"USD"</span>
                                </div>
                            </div>
                        </div>
                    </div>
                </div>
            </Show>

            // ── TAB CONTENT: Modules ──
            <Show when=move || active_tab.get() == "t-modules">
                <div class="bg-surface-container-low border border-outline-variant/20 rounded-xl overflow-hidden shadow-sm">
                    <div class="flex justify-between items-center px-5 py-3.5 border-b border-outline-variant/20 bg-surface-container-high/40">
                        <h3 class="text-xs font-bold uppercase tracking-wider text-on-surface-variant">
                            "Admin Module Flags — Folio · app_instance_modules"
                        </h3>
                        <span class="text-[10px] text-on-surface-variant/60 italic">"Toggles save automatically"</span>
                    </div>
                    <p class="px-5 py-3 text-xs text-on-surface-variant/70 border-b border-outline-variant/10">
                        "Fixed platform modules (Dashboard, Settings, Security) cannot be disabled. All others are toggleable per instance."
                    </p>
                    <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4 gap-4 p-5">
                        {
                            let module_cell_fixed = |name: &str, desc: &str| {
                                let name = name.to_string();
                                let desc = desc.to_string();
                                view! {
                                    <div class="bg-surface-container p-4 rounded-xl border border-outline-variant/40 flex flex-col justify-between min-h-[100px]">
                                        <div>
                                            <div class="text-xs font-bold text-on-surface">{name}</div>
                                            <div class="text-[10px] text-on-surface-variant/70 mt-1">{desc}</div>
                                        </div>
                                        <div class="text-[9px] font-bold text-on-surface-variant/50 uppercase tracking-widest mt-3">"Fixed · Active"</div>
                                    </div>
                                }
                            };
                            let module_cell_toggle = |name: &'static str, desc: &'static str, module_key: &'static str, signal_state: RwSignal<bool>| {
                                let handler = make_module_handler(module_key, signal_state);
                                view! {
                                    <div class=move || if signal_state.get() { "bg-surface-container-high/50 p-4 rounded-xl border border-primary/20 flex flex-col justify-between min-h-[100px] transition-all" } else { "bg-surface-container p-4 rounded-xl border border-outline-variant/20 flex flex-col justify-between min-h-[100px] opacity-60 hover:opacity-100 transition-all" } >
                                        <div>
                                            <div class="text-xs font-bold text-on-surface">{name}</div>
                                            <div class="text-[10px] text-on-surface-variant/70 mt-1">{desc}</div>
                                        </div>
                                        <div class="flex items-center justify-between mt-4">
                                            <label class="relative inline-block w-8 h-4 shrink-0 cursor-pointer">
                                                <input 
                                                    type="checkbox" 
                                                    class="opacity-0 w-0 h-0 peer" 
                                                    prop:checked=signal_state
                                                    on:change=move |ev| {
                                                        let checked = event_target_checked(&ev);
                                                        handler(checked);
                                                    }
                                                />
                                                <div class="absolute inset-0 bg-outline-variant/40 rounded-full transition-colors peer-checked:bg-primary">
                                                    <div class="absolute top-[2px] left-[2px] w-[12px] h-[12px] rounded-full bg-white transition-transform peer-checked:translate-x-[16px]"></div>
                                                </div>
                                            </label>
                                            <span class="text-[10px] font-bold uppercase tracking-wider">{move || if signal_state.get() { view! { <span class="text-emerald-400">"On"</span> }.into_any() } else { view! { <span class="text-on-surface-variant/60">"Off"</span> }.into_any() }}</span>
                                        </div>
                                    </div>
                                }
                            };

                            view! {
                                {module_cell_fixed("Dashboard", "Platform overview")}
                                {module_cell_toggle("Portfolio", "NAV + asset hierarchy", "portfolio", module_portfolio)}
                                {module_cell_toggle("Leases", "Condomínio, auto-renew", "leases", module_leases)}
                                {module_cell_toggle("Maintenance", "Dispatch + WebSocket", "maintenance", module_maintenance)}
                                {module_cell_toggle("Vendors", "License + G-27 scoring", "vendors", module_vendors)}
                                {module_cell_toggle("Wholesale CRM", "MAO, Kanban, leads", "crm", module_crm)}
                                {module_cell_toggle("Billing · G-03", "Stripe + multi-rail", "billing", module_billing)}
                                {module_cell_toggle("STR Compliance", "Permit, OTA sync", "str_compliance", module_str)}
                                {module_cell_toggle("Leads · G-31", "Lifecycle + qualify", "leads", module_leads)}
                                {module_cell_toggle("Opportunities · G-15", "Sales pipeline", "opportunities", module_opps)}
                                {module_cell_toggle("Events · G-21", "Ticketing + check-in", "events", module_events)}
                                {module_cell_toggle("Vault · G-02", "PM doc taxonomy", "vault", module_vault)}
                                {module_cell_toggle("Reporting · G-33", "PMC client KPIs", "reporting", module_reporting)}
                                {module_cell_toggle("Violations · G-13", "Compliance lifecycle", "violations", module_violations)}
                                {module_cell_toggle("Geo · G-01", "PostGIS spatial", "geo", module_geo)}
                                {module_cell_fixed("Settings", "Instance settings")}
                            }
                        }
                    </div>
                </div>
            </Show>

            // ── TAB CONTENT: App Config (Folio) ──
            <Show when=move || active_tab.get() == "t-folio-config">
                <div class="space-y-6">
                    <div class="bg-primary-container/10 border border-primary/20 p-5 rounded-xl text-xs text-on-surface-variant leading-relaxed">
                        <span class="text-primary font-bold">"Folio App Config"</span> " lives in " <code class="text-primary text-[11px] font-bold">"atlas_app_deployment_config"</code> " (G-33). This config is scoped to this app instance only and controls jurisdiction, market config, role segmentation, payment rails, and PMC mode. These settings do " <strong class="text-on-surface">"not"</strong> " exist on Anchor or Network Instance apps."
                    </div>

                    <div class="grid grid-cols-1 lg:grid-cols-2 gap-6">
                        <div class="space-y-6">
                            // Card: Jurisdiction
                            <div class="bg-surface-container-low border border-outline-variant/20 rounded-xl overflow-hidden shadow-sm">
                                <div class="px-5 py-3.5 border-b border-outline-variant/20 bg-surface-container-high/40">
                                    <h3 class="text-xs font-bold uppercase tracking-wider text-on-surface-variant">
                                        "Jurisdiction & Market"
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
                                                <option value="BR">"BR (Brazil — Lei do Inquilinato)"</option>
                                                <option value="USVI">"USVI (US Virgin Islands)"</option>
                                                <option value="DR">"DR (Dominican Republic)"</option>
                                                <option value="HT">"HT (Haiti)"</option>
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

                            // Card: Deployment mode
                            <div class="bg-surface-container-low border border-outline-variant/20 rounded-xl overflow-hidden shadow-sm">
                                <div class="px-5 py-3.5 border-b border-outline-variant/20 bg-surface-container-high/40">
                                    <h3 class="text-xs font-bold uppercase tracking-wider text-on-surface-variant">
                                        "Platform Deployment Mode · G-33"
                                    </h3>
                                </div>
                                <div class="p-6 space-y-4 divide-y divide-outline-variant/10">
                                    <div class="flex justify-between items-center pb-4 text-xs">
                                        <span class="text-on-surface-variant">"Deployment Mode"</span>
                                        <select 
                                            class="bg-surface-container border border-outline-variant/40 rounded-lg p-2 text-xs text-on-surface outline-none cursor-pointer focus:border-primary"
                                            on:change=move |ev| deployment_mode.set(event_target_value(&ev))
                                            prop:value=deployment_mode
                                        >
                                            <option value="standard">"standard"</option>
                                            <option value="internal_operator">"internal_operator"</option>
                                        </select>
                                    </div>
                                    <div class="flex justify-between items-center py-4 text-xs">
                                        <span class="text-on-surface-variant">"pmc_enabled (JSON)"</span>
                                        <span class="font-bold text-emerald-400">"true"</span>
                                    </div>
                                    <div class="flex justify-between items-center py-4 text-xs">
                                        <span class="text-on-surface-variant">"PropertyManager role"</span>
                                        <span class="font-bold text-emerald-400">"Provisioned"</span>
                                    </div>
                                    <div class="pt-4 text-[10px] text-on-surface-variant/60 leading-normal">
                                        "App deployment modes strictly represent topology (standard vs internal_operator). App-specific settings like " <code class="text-on-surface-variant">"pmc_enabled: true"</code> " or " <code class="text-on-surface-variant">"broker_enabled: true"</code> " are toggled inside the JSON config payload."
                                    </div>
                                </div>
                            </div>
                        </div>

                        // Card: Rails
                        <div class="space-y-6">
                            <div class="bg-surface-container-low border border-outline-variant/20 rounded-xl overflow-hidden shadow-sm">
                                <div class="flex justify-between items-center px-5 py-3.5 border-b border-outline-variant/20 bg-surface-container-high/40">
                                    <h3 class="text-xs font-bold uppercase tracking-wider text-on-surface-variant">
                                        "Payment Rails · PaymentRailAdapter"
                                    </h3>
                                    <button class="text-xs text-primary hover:underline hover:opacity-80 transition-opacity" on:click=move |_| show_add_rail_modal.set(true)>"+ Add Rail"</button>
                                </div>
                                <div class="divide-y divide-outline-variant/10">
                                    // Stripe row
                                    <div class="p-4 flex items-center justify-between gap-4">
                                        <div class="w-8 h-8 rounded-lg bg-[#635BFF]/10 text-[#635bff] font-bold text-[10px] flex items-center justify-center shrink-0">"SC"</div>
                                        <div class="flex-1 min-w-0">
                                            <div class="text-xs font-bold text-on-surface">"Stripe Connect"</div>
                                            <div class="text-[10px] font-mono text-on-surface-variant/70 truncate mt-0.5">"Account: acct_1P8aVxFkjE · USD · Residential"</div>
                                        </div>
                                        <div class="flex items-center gap-3">
                                            <span class="px-2 py-0.5 rounded text-[9px] font-bold bg-emerald-500/10 text-emerald-400 border border-emerald-500/20">"● Active"</span>
                                            <button class="btn-ghost text-[10px] font-bold px-2 py-1 border border-outline-variant/30 rounded" on:click=move |_| {
                                                toast.show_toast("Config Opened", "Opening Stripe connection credentials panel", "info");
                                            }>"Config"</button>
                                        </div>
                                    </div>
                                    // BTC row
                                    <div class="p-4 flex items-center justify-between gap-4">
                                        <div class="w-8 h-8 rounded-lg bg-orange-500/10 text-orange-400 font-bold text-[12px] flex items-center justify-center shrink-0">"₿"</div>
                                        <div class="flex-1 min-w-0">
                                            <div class="text-xs font-bold text-on-surface">"Bitcoin On-chain"</div>
                                            <div class="text-[10px] font-mono text-on-surface-variant/70 truncate mt-0.5">"bc1q…8kx2 · Mempool: mempool.space · Confirms: 1"</div>
                                        </div>
                                        <div class="flex items-center gap-3">
                                            <span class="px-2 py-0.5 rounded text-[9px] font-bold bg-emerald-500/10 text-emerald-400 border border-emerald-500/20">"● Active"</span>
                                            <button class="btn-ghost text-[10px] font-bold px-2 py-1 border border-outline-variant/30 rounded" on:click=move |_| {
                                                toast.show_toast("Config Opened", "Opening BTC block explorer endpoints settings", "info");
                                            }>"Config"</button>
                                        </div>
                                    </div>
                                    // LN row
                                    <div class="p-4 flex items-center justify-between gap-4">
                                        <div class="w-8 h-8 rounded-lg bg-amber-500/10 text-amber-400 font-bold text-[10px] flex items-center justify-center shrink-0">"⚡"</div>
                                        <div class="flex-1 min-w-0">
                                            <div class="text-xs font-bold text-on-surface">"Lightning Network"</div>
                                            <div class="text-[10px] font-mono text-on-surface-variant/70 truncate mt-0.5">"LNURL: lnurl1… · Node: 03a1b…"</div>
                                        </div>
                                        <div class="flex items-center gap-3">
                                            <span class="px-2 py-0.5 rounded text-[9px] font-bold bg-amber-500/10 text-amber-400 border border-amber-500/20">"⚠ Unverified"</span>
                                            <button class="btn-ghost text-[10px] font-bold px-2 py-1 border border-outline-variant/30 rounded" on:click=move |_| {
                                                toast.show_toast("Config Opened", "Opening Lightning routing channels setup", "info");
                                            }>"Config"</button>
                                        </div>
                                    </div>
                                    // PIX row
                                    <div class="p-4 flex items-center justify-between gap-4">
                                        <div class="w-8 h-8 rounded-lg bg-surface-container border border-outline-variant/20 text-on-surface-variant/60 font-bold text-[10px] flex items-center justify-center shrink-0">"PIX"</div>
                                        <div class="flex-1 min-w-0">
                                            <div class="text-xs font-bold text-on-surface">"InfinitePay / PIX"</div>
                                            <div class="text-[10px] font-mono text-on-surface-variant/50 truncate mt-0.5">"Not configured · Brazil jurisdiction only"</div>
                                        </div>
                                        <div class="flex items-center gap-3">
                                            <span class="px-2 py-0.5 rounded text-[9px] font-bold bg-surface-container border border-outline-variant/30 text-on-surface-variant/40">"Disabled"</span>
                                            <button class="btn-ghost text-[10px] font-bold px-2 py-1 border border-outline-variant/30 rounded" on:click=move |_| {
                                                if jurisdiction_code.get() != "BR" {
                                                    toast.show_toast("Blocked", "PIX is restricted to Brazil (BR) jurisdiction.", "error");
                                                } else {
                                                    pix_status.set("Active".to_string());
                                                    toast.show_toast("Success", "PIX payment rail enabled.", "success");
                                                }
                                            }>"Enable"</button>
                                        </div>
                                    </div>
                                </div>
                            </div>

                            <div class="bg-surface-container-low border border-outline-variant/20 rounded-xl overflow-hidden shadow-sm">
                                <div class="px-5 py-3.5 border-b border-outline-variant/20 bg-surface-container-high/40">
                                    <h3 class="text-xs font-bold uppercase tracking-wider text-on-surface-variant">
                                        "Role Segmentation"
                                    </h3>
                                </div>
                                <div class="p-5 text-xs text-on-surface-variant/70 border-b border-outline-variant/10 bg-surface-container-high/5">
                                    "Folio's authenticated router is split into 5 sub-routers, each gated by role middleware."
                                </div>
                                <div class="divide-y divide-outline-variant/10 text-xs">
                                    <div class="flex justify-between items-start px-5 py-3 gap-8">
                                        <span class="text-on-surface font-semibold">"Landlord Router"</span>
                                        <span class="text-right text-emerald-400">"Active · portfolio, assets, leases, billing, STR, leads, opps"</span>
                                    </div>
                                    <div class="flex justify-between items-start px-5 py-3 gap-8">
                                        <span class="text-on-surface font-semibold">"Tenant Router"</span>
                                        <span class="text-right text-emerald-400">"Active · maintenance, reservations, applications, household"</span>
                                    </div>
                                    <div class="flex justify-between items-start px-5 py-3 gap-8">
                                        <span class="text-on-surface font-semibold">"Vendor Router"</span>
                                        <span class="text-right text-emerald-400">"Active · work orders, invoices"</span>
                                    </div>
                                    <div class="flex justify-between items-start px-5 py-3 gap-8">
                                        <span class="text-on-surface font-semibold">"PMC Router"</span>
                                        <span class="text-right text-primary">"Active · PMC clients, analytics, invite"</span>
                                    </div>
                                    <div class="flex justify-between items-start px-5 py-3 gap-8">
                                        <span class="text-on-surface font-semibold">"Owner Router"</span>
                                        <span class="text-right">"Active · read-only portfolio visibility"</span>
                                    </div>
                                </div>
                            </div>
                        </div>
                    </div>
                </div>
            </Show>

            // ── TAB CONTENT: Scorecards ──
            <Show when=move || active_tab.get() == "t-scorecards">
                <div class="space-y-6">
                    <div class="bg-purple-500/10 border border-purple-500/20 p-5 rounded-xl text-xs text-on-surface-variant leading-relaxed">
                        <span class="text-purple-400 font-bold">"G-27 Auto-seeded by Folio Provisioner."</span> " When this app instance was created, " <code class="text-purple-400 text-[11px] font-bold">"scorecard_provisioner::seed_pm_templates()"</code> " automatically created 4 canonical PM scorecard templates scoped to this tenant. Anchor and Network Instance do " <strong class="text-on-surface">"not"</strong> " auto-seed scorecards."
                    </div>

                    <div class="bg-surface-container-low border border-outline-variant/20 rounded-xl overflow-hidden shadow-sm">
                        <div class="overflow-x-auto">
                            <table class="w-full text-left border-collapse">
                                <thead>
                                    <tr class="text-xs uppercase tracking-wider text-on-surface-variant border-b border-outline-variant/20 bg-surface-container-high/40">
                                        <th class="py-3 px-5 font-medium">"Template Name"</th>
                                        <th class="py-3 px-5 font-medium">"Target Entity"</th>
                                        <th class="py-3 px-5 font-medium text-center">"Dimensions"</th>
                                        <th class="py-3 px-5 font-medium text-center">"Samples"</th>
                                        <th class="py-3 px-5 font-medium text-center">"Avg Score"</th>
                                        <th class="py-3 px-5 font-medium text-center">"Tier"</th>
                                        <th class="py-3 px-5 font-medium text-right"></th>
                                    </tr>
                                </thead>
                                <tbody class="divide-y divide-outline-variant/10 text-xs">
                                    <tr class="hover:bg-surface-bright/5 transition-colors">
                                        <td class="py-3 px-5 font-bold">"Contractor Performance"</td>
                                        <td class="py-3 px-5 font-mono text-on-surface-variant/70">"atlas_service_provider"</td>
                                        <td class="py-3 px-5 font-mono text-center">"6"</td>
                                        <td class="py-3 px-5 font-mono text-center">"128"</td>
                                        <td class="py-3 px-5 text-center font-bold text-emerald-400 font-mono">"8.1"</td>
                                        <td class="py-3 px-5 text-center"><span class="px-2 py-0.5 rounded bg-emerald-500/10 text-emerald-400 border border-emerald-500/20 text-[10px] font-bold uppercase tracking-wider">"Above"</span></td>
                                        <td class="py-3 px-5 text-right"><a href="/billing" class="text-primary hover:underline font-bold text-[10px] uppercase tracking-wider">"Configure →"</a></td>
                                    </tr>
                                    <tr class="hover:bg-surface-bright/5 transition-colors">
                                        <td class="py-3 px-5 font-bold">"Listing Quality Index"</td>
                                        <td class="py-3 px-5 font-mono text-on-surface-variant/70">"atlas_asset"</td>
                                        <td class="py-3 px-5 font-mono text-center">"5"</td>
                                        <td class="py-3 px-5 font-mono text-center">"87"</td>
                                        <td class="py-3 px-5 text-center font-bold text-emerald-400 font-mono">"8.4"</td>
                                        <td class="py-3 px-5 text-center"><span class="px-2 py-0.5 rounded bg-emerald-500/10 text-emerald-400 border border-emerald-500/20 text-[10px] font-bold uppercase tracking-wider">"Above"</span></td>
                                        <td class="py-3 px-5 text-right"><a href="#" class="text-primary hover:underline font-bold text-[10px] uppercase tracking-wider">"Configure →"</a></td>
                                    </tr>
                                    <tr class="hover:bg-surface-bright/5 transition-colors">
                                        <td class="py-3 px-5 font-bold">"Deal Qualification"</td>
                                        <td class="py-3 px-5 font-mono text-on-surface-variant/70">"atlas_lead"</td>
                                        <td class="py-3 px-5 font-mono text-center">"7"</td>
                                        <td class="py-3 px-5 font-mono text-center">"342"</td>
                                        <td class="py-3 px-5 text-center font-bold text-emerald-400 font-mono">"7.6"</td>
                                        <td class="py-3 px-5 text-center"><span class="px-2 py-0.5 rounded bg-emerald-500/10 text-emerald-400 border border-emerald-500/20 text-[10px] font-bold uppercase tracking-wider">"Above"</span></td>
                                        <td class="py-3 px-5 text-right"><a href="#" class="text-primary hover:underline font-bold text-[10px] uppercase tracking-wider">"Configure →"</a></td>
                                    </tr>
                                    <tr class="hover:bg-surface-bright/5 transition-colors">
                                        <td class="py-3 px-5 font-bold">"Tenant Health Score"</td>
                                        <td class="py-3 px-5 font-mono text-on-surface-variant/70">"atlas_scorecard_target (tenant)"</td>
                                        <td class="py-3 px-5 font-mono text-center">"5"</td>
                                        <td class="py-3 px-5 font-mono text-center">"47"</td>
                                        <td class="py-3 px-5 text-center font-bold text-emerald-400 font-mono">"9.2"</td>
                                        <td class="py-3 px-5 text-center"><span class="px-2 py-0.5 rounded bg-emerald-500/10 text-emerald-400 border border-emerald-500/20 text-[10px] font-bold uppercase tracking-wider">"Outstanding"</span></td>
                                        <td class="py-3 px-5 text-right"><a href="#" class="text-primary hover:underline font-bold text-[10px] uppercase tracking-wider">"Configure →"</a></td>
                                    </tr>
                                </tbody>
                            </table>
                        </div>
                    </div>

                    <div class="bg-surface-container p-4 rounded-xl border border-outline-variant/20 text-xs text-on-surface-variant leading-normal">
                        <span class="font-bold text-on-surface">"scorecard_display_rules_enabled = true"</span> " — Score display nudges (e.g. \"Rate this contractor\") are active on the Folio frontend. Requires ≥ 5 ratings for a confirmed score; below that threshold, Bayesian estimation (~score) is shown."
                    </div>
                </div>
            </Show>

            // ── TAB CONTENT: Background Jobs ──
            <Show when=move || active_tab.get() == "t-jobs">
                <div class="space-y-6">
                    <p class="text-xs text-on-surface-variant/80">
                        "Folio registers " <strong class="text-on-surface">"4 background jobs"</strong> " vs. Anchor's 1 (Bitcoin sync) and Network Instance's 0. These run per-tenant on the platform job scheduler."
                    </p>

                    <div class="bg-surface-container-low border border-outline-variant/20 rounded-xl overflow-hidden shadow-sm">
                        <div class="overflow-x-auto">
                            <table class="w-full text-left border-collapse">
                                <thead>
                                    <tr class="text-xs uppercase tracking-wider text-on-surface-variant border-b border-outline-variant/20 bg-surface-container-high/40">
                                        <th class="py-3 px-5 font-medium">"Job Type"</th>
                                        <th class="py-3 px-5 font-medium text-center">"Phase"</th>
                                        <th class="py-3 px-5 font-medium text-center">"Interval"</th>
                                        <th class="py-3 px-5 font-medium text-center">"Active by Default"</th>
                                        <th class="py-3 px-5 font-medium text-center">"Status"</th>
                                        <th class="py-3 px-5 font-medium text-center">"Last Run"</th>
                                        <th class="py-3 px-5 font-medium text-left">"Config Schema"</th>
                                    </tr>
                                </thead>
                                <tbody class="divide-y divide-outline-variant/10 text-xs">
                                    <tr class="hover:bg-surface-bright/5 transition-colors">
                                        <td class="py-3 px-5 font-bold">"pm_btc_mempool_poll"</td>
                                        <td class="py-3 px-5 text-on-surface-variant/70 text-center">"Phase 3"</td>
                                        <td class="py-3 px-5 font-mono text-center">"120s (2 min)"</td>
                                        <td class="py-3 px-5 text-center font-bold text-emerald-400">"Yes"</td>
                                        <td class="py-3 px-5 text-center font-bold text-emerald-400">"● Running"</td>
                                        <td class="py-3 px-5 text-on-surface-variant/70 text-center">"42s ago"</td>
                                        <td class="py-3 px-5 font-mono text-on-surface-variant/60">"confirmation_threshold: 1, mempool_host"</td>
                                    </tr>
                                    <tr class="hover:bg-surface-bright/5 transition-colors">
                                        <td class="py-3 px-5 font-bold">"pm_str_permit_expiry_scanner"</td>
                                        <td class="py-3 px-5 text-on-surface-variant/70 text-center">"Phase 4"</td>
                                        <td class="py-3 px-5 font-mono text-center">"86400s (daily)"</td>
                                        <td class="py-3 px-5 text-center font-bold text-emerald-400">"Yes"</td>
                                        <td class="py-3 px-5 text-center font-bold text-emerald-400">"● Running"</td>
                                        <td class="py-3 px-5 text-on-surface-variant/70 text-center">"4h ago"</td>
                                        <td class="py-3 px-5 font-mono text-on-surface-variant/60">"warning_days: 30"</td>
                                    </tr>
                                    <tr class="hover:bg-surface-bright/5 transition-colors">
                                        <td class="py-3 px-5 font-bold">"pm_ota_revenue_sync"</td>
                                        <td class="py-3 px-5 text-on-surface-variant/70 text-center">"Phase 5"</td>
                                        <td class="py-3 px-5 font-mono text-center">"3600s (hourly)"</td>
                                        <td class="py-3 px-5 text-center font-bold text-amber-400">"No (per-tenant)"</td>
                                        <td class="py-3 px-5 text-center font-bold text-amber-400">"⚠ Disabled"</td>
                                        <td class="py-3 px-5 text-on-surface-variant/70 text-center">"Never"</td>
                                        <td class="py-3 px-5 font-mono text-on-surface-variant/60">"ota_integration_id, lookback_hours: 25"</td>
                                    </tr>
                                    <tr class="hover:bg-surface-bright/5 transition-colors">
                                        <td class="py-3 px-5 font-bold">"pm_str_hold_expiry_sweeper"</td>
                                        <td class="py-3 px-5 text-on-surface-variant/70 text-center">"Phase 6"</td>
                                        <td class="py-3 px-5 font-mono text-center">"300s (5 min)"</td>
                                        <td class="py-3 px-5 text-center font-bold text-emerald-400">"Yes"</td>
                                        <td class="py-3 px-5 text-center font-bold text-emerald-400">"● Running"</td>
                                        <td class="py-3 px-5 text-on-surface-variant/70 text-center">"4m ago"</td>
                                        <td class="py-3 px-5 font-mono text-on-surface-variant/60">"none"</td>
                                    </tr>
                                </tbody>
                            </table>
                        </div>
                    </div>

                    // Card: OTA Sync Enable form
                    <div class="bg-surface-container-low border border-outline-variant/20 rounded-xl overflow-hidden shadow-sm">
                        <div class="px-5 py-3.5 border-b border-outline-variant/20 bg-surface-container-high/40">
                            <h3 class="text-xs font-bold uppercase tracking-wider text-on-surface-variant">
                                "Enable OTA Revenue Sync"
                            </h3>
                        </div>
                        <div class="p-6 space-y-4">
                            <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
                                <div class="space-y-1.5">
                                    <label class="text-[10px] font-bold uppercase tracking-wider text-on-surface-variant/80">"OTA Integration"</label>
                                    <select class="w-full bg-surface-container border border-outline-variant/40 rounded-lg p-2.5 text-xs text-on-surface outline-none cursor-pointer focus:border-primary">
                                        <option>"— Select integration —"</option>
                                        <option value="airbnb">"Airbnb Connect"</option>
                                        <option value="vrbo">"VRBO"</option>
                                        <option value="booking">"Booking.com"</option>
                                    </select>
                                </div>
                                <div class="space-y-1.5">
                                    <label class="text-[10px] font-bold uppercase tracking-wider text-on-surface-variant/80">"Lookback Hours"</label>
                                    <input 
                                        type="number" 
                                        class="w-full bg-surface-container border border-outline-variant/40 rounded-lg p-2.5 text-xs text-on-surface outline-none focus:border-primary"
                                        on:input=move |ev| lookback_hours.set(event_target_value(&ev))
                                        prop:value=lookback_hours
                                    />
                                </div>
                            </div>
                            <button class="btn-primary-gradient px-4 py-2 rounded-lg text-xs font-semibold text-on-primary-container shadow shadow-primary/10 hover:opacity-95 active:scale-95 transition-all" on:click=move |_| {
                                toast.show_toast("Success", "OTA Revenue Sync scheduler job enabled.", "success");
                            }>"Enable OTA Sync"</button>
                        </div>
                    </div>
                </div>
            </Show>

            // ── TAB CONTENT: Domain & SiteConfig ──
            <Show when=move || active_tab.get() == "t-domain">
                <div class="grid grid-cols-1 lg:grid-cols-2 gap-6">
                    <div class="bg-surface-container-low border border-outline-variant/20 rounded-xl overflow-hidden shadow-sm">
                        <div class="flex justify-between items-center px-5 py-3.5 border-b border-outline-variant/20 bg-surface-container-high/40">
                            <h3 class="text-xs font-bold uppercase tracking-wider text-on-surface-variant">
                                "Domain Configuration"
                            </h3>
                            <button class="text-xs text-primary hover:underline hover:opacity-80 transition-opacity" on:click=move |_| show_edit_config_modal.set(true)>"Edit"</button>
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
                                <span class="text-on-surface-variant">"Apex Domain"</span>
                                <span class="font-mono text-on-surface-variant/80">"atlas.app"</span>
                            </div>
                            <div class="flex justify-between items-center px-5 py-3">
                                <span class="text-on-surface-variant">"CNAME Status"</span>
                                <span class="text-emerald-400 font-semibold">"● Verified"</span>
                            </div>
                            <div class="flex justify-between items-center px-5 py-3">
                                <span class="text-on-surface-variant">"SSL/TLS"</span>
                                <span class="text-emerald-400 font-semibold">"● Let's Encrypt · Valid"</span>
                            </div>
                            <div class="flex justify-between items-center px-5 py-3">
                                <span class="text-on-surface-variant">"WebAuthn Origin"</span>
                                <span class="font-mono text-on-surface-variant/80">{move || custom_domain.get()}</span>
                            </div>
                            <div class="flex justify-between items-center px-5 py-3">
                                <span class="text-on-surface-variant">"Public Router"</span>
                                <span class="font-mono text-on-surface-variant/60 truncate max-w-[250px]">"/api/folio/me, /api/pub/leads, /api/billing/webhook"</span>
                            </div>
                            <div class="flex justify-between items-center px-5 py-3">
                                <span class="text-on-surface-variant">"Landlord Router"</span>
                                <span class="font-mono text-on-surface-variant/60 truncate max-w-[250px]">"/api/folio (requires FolioRole::Landlord)"</span>
                            </div>
                            <div class="flex justify-between items-center px-5 py-3">
                                <span class="text-on-surface-variant">"Tenant Router"</span>
                                <span class="font-mono text-on-surface-variant/60 truncate max-w-[250px]">"/api/folio/tenant (requires FolioRole::Tenant)"</span>
                            </div>
                        </div>
                    </div>

                    // SiteConfig raw JSON card
                    <div class="bg-surface-container-low border border-outline-variant/20 rounded-xl overflow-hidden shadow-sm">
                        <div class="flex justify-between items-center px-5 py-3.5 border-b border-outline-variant/20 bg-surface-container-high/40">
                            <h3 class="text-xs font-bold uppercase tracking-wider text-on-surface-variant">
                                "SiteConfig · tenant_setting keys"
                            </h3>
                            <button class="text-xs text-primary hover:underline hover:opacity-80 transition-opacity" on:click=move |_| show_edit_config_modal.set(true)>"Edit"</button>
                        </div>
                        <div class="p-6 bg-[#0a0c16] font-mono text-[11px] leading-relaxed text-on-surface-variant border border-outline-variant/10 m-5 rounded-lg">
                            <div>"{"</div>
                            <div class="pl-4">
                                <span class="text-primary">"\"folio_jurisdiction_code\""</span>": \""<span class="text-emerald-400">{move || jurisdiction_code.get()}</span>"\", "
                            </div>
                            <div class="pl-4">
                                <span class="text-primary">"\"folio_market_config\""</span>": \""<span class="text-emerald-400">{move || market_config.get()}</span>"\", "
                            </div>
                            <div class="pl-4">
                                <span class="text-primary">"\"scorecard_display_rules_enabled\""</span>": "<span class="text-purple-400">"true"</span>", "
                            </div>
                            <div class="pl-4">
                                <span class="text-primary">"\"folio_payment_rails_configured\""</span>": "<span class="text-purple-400">"true"</span>", "
                            </div>
                            <div class="pl-4">
                                <span class="text-primary">"\"b2b_mode\""</span>": "<span class="text-error">"false"</span>", "
                            </div>
                            <div class="pl-4">
                                <span class="text-primary">"\"lead_capture_mode\""</span>": \""<span class="text-emerald-400">"full_form"</span>"\", "
                            </div>
                            <div class="pl-4">
                                <span class="text-primary">"\"btc_confirmation_threshold\""</span>": "<span class="text-amber-400">"1"</span>", "
                            </div>
                            <div class="pl-4">
                                <span class="text-primary">"\"str_warning_days\""</span>": "<span class="text-amber-400">"30"</span>", "
                            </div>
                            <div class="pl-4">
                                <span class="text-primary">"\"site_title\""</span>": \""<span class="text-emerald-400">{move || app_slug_display()}</span>"\", "
                            </div>
                            <div class="pl-4">
                                <span class="text-primary">"\"locale\""</span>": \""<span class="text-emerald-400">"en-US"</span>"\", "
                            </div>
                            <div class="pl-4">
                                <span class="text-primary">"\"currency\""</span>": \""<span class="text-emerald-400">"USD"</span>"\""
                            </div>
                            <div>"}"</div>
                        </div>
                    </div>
                </div>
            </Show>

            // ── TAB CONTENT: App Type Comparison ──
            <Show when=move || active_tab.get() == "t-compare">
                <div class="space-y-6">
                    <p class="text-xs text-on-surface-variant/80">
                        "Each app type (atlas_app) implements the " <code class="text-primary font-bold">"AtlasApp"</code> " trait differently. The config surfaces the admin sees depend entirely on which app_id is running for this instance."
                    </p>

                    <div class="bg-surface-container-low border border-outline-variant/20 rounded-xl overflow-hidden shadow-sm">
                        <div class="overflow-x-auto">
                            <table class="w-full text-left border-collapse text-xs">
                                <thead>
                                    <tr class="text-xs uppercase tracking-wider text-on-surface-variant border-b border-outline-variant/20 bg-surface-container-high/40">
                                        <th class="py-3 px-5 font-semibold">"Config Dimension"</th>
                                        <th class="py-3 px-5 font-semibold text-primary">"Folio (PM)"</th>
                                        <th class="py-3 px-5 font-semibold text-purple-400">"Anchor (CMS)"</th>
                                        <th class="py-3 px-5 font-semibold text-amber-400">"Network Instance (Dir.)"</th>
                                    </tr>
                                </thead>
                                <tbody class="divide-y divide-outline-variant/10">
                                    <tr class="hover:bg-surface-bright/5 transition-colors">
                                        <td class="py-3 px-5 font-bold">"app_id"</td>
                                        <td class="py-3 px-5 font-mono text-primary">"property_management"</td>
                                        <td class="py-3 px-5 font-mono text-purple-400">"anchor"</td>
                                        <td class="py-3 px-5 font-mono text-amber-400">"network_instance"</td>
                                    </tr>
                                    <tr class="hover:bg-surface-bright/5 transition-colors">
                                        <td class="py-3 px-5 font-bold">"Onboarding Step 1"</td>
                                        <td class="py-3 px-5">"Jurisdiction (required)"</td>
                                        <td class="py-3 px-5">"Brand Identity"</td>
                                        <td class="py-3 px-5">"Network Identity"</td>
                                    </tr>
                                    <tr class="hover:bg-surface-bright/5 transition-colors">
                                        <td class="py-3 px-5 font-bold">"Onboarding Step 2"</td>
                                        <td class="py-3 px-5">"First Property"</td>
                                        <td class="py-3 px-5">"Custom Domain"</td>
                                        <td class="py-3 px-5">"Custom Domain"</td>
                                    </tr>
                                    <tr class="hover:bg-surface-bright/5 transition-colors">
                                        <td class="py-3 px-5 font-bold">"Onboarding Step 3"</td>
                                        <td class="py-3 px-5">"Payment Rails (optional)"</td>
                                        <td class="py-3 px-5">"Design Theme"</td>
                                        <td class="py-3 px-5">"Categories (required)"</td>
                                    </tr>
                                    <tr class="hover:bg-surface-bright/5 transition-colors">
                                        <td class="py-3 px-5 font-bold">"Onboarding Step 4"</td>
                                        <td class="py-3 px-5">"—"</td>
                                        <td class="py-3 px-5">"First Page"</td>
                                        <td class="py-3 px-5">"Listing Template (required)"</td>
                                    </tr>
                                    <tr class="hover:bg-surface-bright/5 transition-colors">
                                        <td class="py-3 px-5 font-bold">"Onboarding Step 5"</td>
                                        <td class="py-3 px-5">"—"</td>
                                        <td class="py-3 px-5">"Audience Mode (B2B/B2C)"</td>
                                        <td class="py-3 px-5">"First Listing (optional)"</td>
                                    </tr>
                                    <tr class="hover:bg-surface-bright/5 transition-colors">
                                        <td class="py-3 px-5 font-bold">"G-27 Auto-seeded?"</td>
                                        <td class="py-3 px-5 text-emerald-400 font-bold">"Yes — 4 PM templates"</td>
                                        <td class="py-3 px-5 text-error font-bold">"No"</td>
                                        <td class="py-3 px-5 text-error font-bold">"No"</td>
                                    </tr>
                                    <tr class="hover:bg-surface-bright/5 transition-colors">
                                        <td class="py-3 px-5 font-bold">"Background Jobs"</td>
                                        <td class="py-3 px-5">"4 (BTC poll, STR scanner, OTA sync, hold sweeper)"</td>
                                        <td class="py-3 px-5">"1 (BitcoinSync)"</td>
                                        <td class="py-3 px-5">"0"</td>
                                    </tr>
                                    <tr class="hover:bg-surface-bright/5 transition-colors">
                                        <td class="py-3 px-5 font-bold">"Jurisdiction Config"</td>
                                        <td class="py-3 px-5 text-emerald-400 font-bold">"Yes — US/BR/USVI/DR/HT"</td>
                                        <td class="py-3 px-5 text-on-surface-variant/50">"No"</td>
                                        <td class="py-3 px-5 text-on-surface-variant/50">"No"</td>
                                    </tr>
                                    <tr class="hover:bg-surface-bright/5 transition-colors">
                                        <td class="py-3 px-5 font-bold">"Payment Rails"</td>
                                        <td class="py-3 px-5">"Stripe, BTC, Lightning, PIX, Zelle, Kelviq"</td>
                                        <td class="py-3 px-5 text-on-surface-variant/50">"—"</td>
                                        <td class="py-3 px-5">"Stripe (ad purchases)"</td>
                                    </tr>
                                    <tr class="hover:bg-surface-bright/5 transition-colors">
                                        <td class="py-3 px-5 font-bold">"Role Sub-routers"</td>
                                        <td class="py-3 px-5 text-emerald-400">"5 (Landlord, Tenant, Vendor, PMC, Owner)"</td>
                                        <td class="py-3 px-5 text-on-surface-variant/80">"1 (authenticated)"</td>
                                        <td class="py-3 px-5 text-on-surface-variant/80">"1 (authenticated)"</td>
                                    </tr>
                                    <tr class="hover:bg-surface-bright/5 transition-colors">
                                        <td class="py-3 px-5 font-bold">"PMC/Brokerage Mode"</td>
                                        <td class="py-3 px-5 text-emerald-400 font-bold">"Yes — pmc_enabled/broker_enabled in config"</td>
                                        <td class="py-3 px-5 text-on-surface-variant/50">"No"</td>
                                        <td class="py-3 px-5 text-on-surface-variant/50">"No"</td>
                                    </tr>
                                </tbody>
                            </table>
                        </div>
                    </div>
                </div>
            </Show>

            // ── LOCAL STATE MODALS ──

            // 1. Suspend Modal
            <Show when=move || show_suspend_modal.get()>
                <div class="fixed inset-0 z-[100] bg-black/80 backdrop-blur-sm flex items-center justify-center p-4">
                    <div class="bg-surface-container-low w-full max-w-md p-6 rounded-2xl border border-outline-variant/30 shadow-2xl relative">
                        <button class="absolute top-4 right-4 text-on-surface-variant hover:text-on-surface" on:click=move |_| show_suspend_modal.set(false)>"✕"</button>
                        <h3 class="text-lg font-bold mb-2">"Suspend Instance"</h3>
                        <p class="text-on-surface-variant text-xs mb-4">"Provide a reason to suspend this tenant's app instance. Traffic will route to a warning gate."</p>
                        <div class="space-y-1.5 mb-6">
                            <label class="text-[10px] font-bold uppercase tracking-wider text-on-surface-variant/80">"Suspension Reason"</label>
                            <input 
                                type="text" 
                                class="w-full bg-surface-container border border-outline-variant/40 rounded-lg p-2.5 text-xs text-on-surface outline-none focus:border-primary" 
                                placeholder="e.g. Terms of Service violation"
                                prop:value=suspend_reason
                                on:input=move |ev| suspend_reason.set(event_target_value(&ev))
                            />
                        </div>
                        <div class="flex justify-end gap-3">
                            <button class="btn-ghost px-4 py-2 border border-outline-variant/30 rounded-lg text-xs font-semibold hover:bg-surface-bright/20" on:click=move |_| show_suspend_modal.set(false)>"Cancel"</button>
                            <button 
                                class="bg-error border border-error/20 text-on-error hover:opacity-90 px-4 py-2 rounded-lg text-xs font-semibold transition-all"
                                on:click=handle_suspend
                            >
                                "Suspend Tenant"
                            </button>
                        </div>
                    </div>
                </div>
            </Show>

            // 3. Provision Feature Modal
            <Show when=move || show_provision_modal.get()>
                <div class="fixed inset-0 z-[100] bg-black/80 backdrop-blur-sm flex items-center justify-center p-4">
                    <div class="bg-surface-container-low w-full max-w-md p-6 rounded-2xl border border-outline-variant/30 shadow-2xl relative">
                        <button class="absolute top-4 right-4 text-on-surface-variant hover:text-on-surface" on:click=move |_| show_provision_modal.set(false)>"✕"</button>
                        <h3 class="text-lg font-bold mb-2">"Provision Feature"</h3>
                        <p class="text-on-surface-variant text-xs mb-4">"Enable specialized integrations on this instance."</p>
                        <div class="space-y-4 mb-6">
                            <div class="space-y-1.5">
                                <label class="text-[10px] font-bold uppercase tracking-wider text-on-surface-variant/80">"Feature Integration"</label>
                                <select class="w-full bg-surface-container border border-outline-variant/40 rounded-lg p-2.5 text-xs text-on-surface outline-none cursor-pointer focus:border-primary">
                                    <option>"Stripe Connect"</option>
                                    <option>"BTC On-chain"</option>
                                    <option>"Lightning LND"</option>
                                    <option>"PIX"</option>
                                </select>
                            </div>
                        </div>
                        <div class="flex justify-end gap-3">
                            <button class="btn-ghost px-4 py-2 border border-outline-variant/30 rounded-lg text-xs font-semibold hover:bg-surface-bright/20" on:click=move |_| show_provision_modal.set(false)>"Cancel"</button>
                            <button 
                                class="btn-primary-gradient px-4 py-2 rounded-lg text-xs font-semibold text-on-primary-container"
                                on:click=move |_| {
                                    show_provision_modal.set(false);
                                    toast.show_toast("Success", "Feature integration provisioned.", "success");
                                }
                            >
                                "Provision"
                            </button>
                        </div>
                    </div>
                </div>
            </Show>

            // 4. Add Payment Rail Modal
            <Show when=move || show_add_rail_modal.get()>
                <div class="fixed inset-0 z-[100] bg-black/80 backdrop-blur-sm flex items-center justify-center p-4">
                    <div class="bg-surface-container-low w-full max-w-md p-6 rounded-2xl border border-outline-variant/30 shadow-2xl relative">
                        <button class="absolute top-4 right-4 text-on-surface-variant hover:text-on-surface" on:click=move |_| show_add_rail_modal.set(false)>"✕"</button>
                        <h3 class="text-lg font-bold mb-2">"Add Payment Rail"</h3>
                        <div class="space-y-4 mb-6">
                            <div class="space-y-1.5">
                                <label class="text-[10px] font-bold uppercase tracking-wider text-on-surface-variant/80">"Rail Type"</label>
                                <select class="w-full bg-surface-container border border-outline-variant/40 rounded-lg p-2.5 text-xs text-on-surface outline-none cursor-pointer focus:border-primary">
                                    <option>"Stripe Connect"</option>
                                    <option>"Bitcoin On-chain"</option>
                                    <option>"Lightning Network"</option>
                                    <option>"InfinitePay / PIX"</option>
                                </select>
                            </div>
                            <div class="space-y-1.5">
                                <label class="text-[10px] font-bold uppercase tracking-wider text-on-surface-variant/80">"Credentials"</label>
                                <input type="text" class="w-full bg-surface-container border border-outline-variant/40 rounded-lg p-2.5 text-xs text-on-surface outline-none focus:border-primary" placeholder="API key or connection string" />
                            </div>
                        </div>
                        <div class="flex justify-end gap-3">
                            <button class="btn-ghost px-4 py-2 border border-outline-variant/30 rounded-lg text-xs font-semibold hover:bg-surface-bright/20" on:click=move |_| show_add_rail_modal.set(false)>"Cancel"</button>
                            <button 
                                class="btn-primary-gradient px-4 py-2 rounded-lg text-xs font-semibold text-on-primary-container"
                                on:click=move |_| {
                                    show_add_rail_modal.set(false);
                                    toast.show_toast("Success", "Payment rail attached successfully.", "success");
                                }
                            >
                                "Add Rail"
                            </button>
                        </div>
                    </div>
                </div>
            </Show>

            // 5. Edit Config Modal
            <Show when=move || show_edit_config_modal.get()>
                <div class="fixed inset-0 z-[100] bg-black/80 backdrop-blur-sm flex items-center justify-center p-4">
                    <div class="bg-surface-container-low w-full max-w-lg p-6 rounded-2xl border border-outline-variant/30 shadow-2xl relative">
                        <button class="absolute top-4 right-4 text-on-surface-variant hover:text-on-surface" on:click=move |_| show_edit_config_modal.set(false)>"✕"</button>
                        <h3 class="text-lg font-bold mb-2">"Edit Raw Config Settings"</h3>
                        <p class="text-on-surface-variant text-xs mb-4">"Direct modifications to the SiteConfig metadata registry. Be careful changing keys."</p>
                        <div class="space-y-4 mb-6">
                            <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
                                <div class="space-y-1.5">
                                    <label class="text-[10px] font-bold uppercase tracking-wider text-on-surface-variant/80">"public_slug"</label>
                                    <input 
                                        type="text" 
                                        class="w-full bg-surface-container border border-outline-variant/40 rounded-lg p-2.5 text-xs text-on-surface outline-none focus:border-primary font-mono"
                                        on:input=move |ev| public_slug.set(event_target_value(&ev))
                                        prop:value=public_slug
                                    />
                                </div>
                                <div class="space-y-1.5">
                                    <label class="text-[10px] font-bold uppercase tracking-wider text-on-surface-variant/80">"custom_domain"</label>
                                    <input 
                                        type="text" 
                                        class="w-full bg-surface-container border border-outline-variant/40 rounded-lg p-2.5 text-xs text-on-surface outline-none focus:border-primary font-mono"
                                        on:input=move |ev| custom_domain.set(event_target_value(&ev))
                                        prop:value=custom_domain
                                    />
                                </div>
                            </div>
                            <div class="space-y-1.5">
                                <label class="text-[10px] font-bold uppercase tracking-wider text-on-surface-variant/80">"Config JSON Structure"</label>
                                <textarea class="w-full bg-surface-container border border-outline-variant/40 rounded-lg p-2.5 text-xs font-mono text-on-surface outline-none focus:border-primary h-36 resize-none">
                                    {move || format!("{{\n  \"folio_jurisdiction_code\": \"{}\",\n  \"folio_market_config\": \"{}\",\n  \"scorecard_display_rules_enabled\": true,\n  \"site_title\": \"{}\"\n}}", jurisdiction_code.get(), market_config.get(), app_slug_display())}
                                </textarea>
                            </div>
                        </div>
                        <div class="flex justify-end gap-3">
                            <button class="btn-ghost px-4 py-2 border border-outline-variant/30 rounded-lg text-xs font-semibold hover:bg-surface-bright/20" on:click=move |_| show_edit_config_modal.set(false)>"Cancel"</button>
                            <button 
                                class="btn-primary-gradient px-4 py-2 rounded-lg text-xs font-semibold text-on-primary-container"
                                on:click=handle_save_config
                            >
                                "Save Config"
                            </button>
                        </div>
                    </div>
                </div>
            </Show>
        </div>
    }
}
