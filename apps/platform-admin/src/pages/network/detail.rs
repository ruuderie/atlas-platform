use leptos::prelude::*;
use leptos_router::hooks::use_params_map;
use uuid::Uuid;
use crate::api::admin::{
    get_app_domains, add_app_domain as api_add_domain, remove_app_domain as api_remove_domain,
    suspend_instance, resume_instance,
};

#[component]
pub fn NetworkDetail() -> impl IntoView {
    let toast = use_context::<crate::app::GlobalToast>().expect("toast context");

    // ── Route params ──────────────────────────────────────────────────────────
    let params = use_params_map();
    let instance_id = move || params.with(|p| p.get("instance_id").unwrap_or_default());

    // ── Tab State ─────────────────────────────────────────────────────────────
    let active_tab = RwSignal::new("overview".to_string());

    // ── Suspension state ─────────────────────────────────────────────────────
    let is_suspended = RwSignal::new(false);

    // ── Editable settings (initialised empty; set from resource when loaded) ──
    let name     = RwSignal::new(String::new());
    let slug     = RwSignal::new(String::new());
    let cap_ltr  = RwSignal::new(false);
    let cap_str  = RwSignal::new(false);
    let cap_vendor = RwSignal::new(false);
    let currency = RwSignal::new("USD".to_string());

    // Theme branding (editable, not persisted yet)
    let theme_mode    = RwSignal::new("dark-slate".to_string());
    let primary_color = RwSignal::new("#0A84FF".to_string());
    let selected_font = RwSignal::new("inter".to_string());

    // Geographic markets (local UI state; no backend persist endpoint yet)
    let market_chi = RwSignal::new(false);
    let market_rio = RwSignal::new(false);
    let market_mia = RwSignal::new(false);

    // ── Live domains resource ─────────────────────────────────────────────────
    let domains_res = LocalResource::new(move || {
        let id = instance_id();
        async move { get_app_domains(id).await.unwrap_or_default() }
    });

    // Local domain list for optimistic adds/removes (seeded from resource)
    let new_domain = RwSignal::new(String::new());

    // ── Syndicated tenants: use tenant-stats until NI-scoped endpoint lands ──
    let tenants_res = LocalResource::new(|| async move {
        crate::api::admin::get_tenant_stats().await.unwrap_or_default()
    });


    // ── Actions ───────────────────────────────────────────────────────────────
    let toggle_status = move |_| {
        let id_str = instance_id();
        if let Ok(id) = Uuid::parse_str(&id_str) {
            let suspended = is_suspended.get();
            let toast2 = toast.clone();
            if suspended {
                leptos::task::spawn_local(async move {
                    match resume_instance(id).await {
                        Ok(_) => { is_suspended.set(false); toast2.show_toast("Network Status", "Network instance resumed.", "success"); }
                        Err(e) => { toast2.show_toast("Error", &e, "error"); }
                    }
                });
            } else {
                leptos::task::spawn_local(async move {
                    match suspend_instance(id, "Manual suspension via admin panel.".to_string()).await {
                        Ok(_) => { is_suspended.set(true); toast2.show_toast("Network Status", "Network instance suspended.", "warning"); }
                        Err(e) => { toast2.show_toast("Error", &e, "error"); }
                    }
                });
            }
        } else {
            toast.show_toast("Error", "Invalid instance ID in URL.", "error");
        }
    };

    let add_domain_action = move |_| {
        let domain_str = new_domain.get().trim().to_string();
        if domain_str.is_empty() { return; }
        let id_str = instance_id();
        let toast2 = toast.clone();
        let domain_clone = domain_str.clone();
        leptos::task::spawn_local(async move {
            match api_add_domain(id_str, domain_clone).await {
                Ok(_) => { toast2.show_toast("Domain Added", "Custom domain added. DNS propagation may take up to 24h.", "success"); }
                Err(e) => { toast2.show_toast("Error", &e, "error"); }
            }
        });
        new_domain.set(String::new());
    };

    let save_overview = move |_| {
        toast.show_toast("Settings Saved", "Overview settings saved (persist endpoint pending).", "success");
    };

    let save_branding = move |_| {
        toast.show_toast("Branding Saved", "Theme customization saved (persist endpoint pending).", "success");
    };


    view! {
        <div class="space-y-6">
            // ── Breadcrumb ──
            <nav class="flex items-center gap-2 text-on-surface-variant text-xs mb-2">
                <a href="/" class="hover:text-primary transition-colors">"Dashboard"</a>
                <span class="material-symbols-outlined text-[12px]">"chevron_right"</span>
                <a href="/network" class="hover:text-primary transition-colors">"Network Instances"</a>
                <span class="material-symbols-outlined text-[12px]">"chevron_right"</span>
                <span class="text-primary/70">{move || if name.get().is_empty() { instance_id() } else { name.get() }}</span>
            </nav>

            // ── Instance Identity Header ──
            <div class="bg-surface-container-low border border-outline-variant/20 p-6 rounded-2xl shadow-sm flex flex-col md:flex-row justify-between items-start md:items-center gap-4">
                <div class="flex items-center gap-4">
                    <div class="w-12 h-12 rounded-xl bg-primary-container border border-primary/20 flex items-center justify-center text-primary text-xl font-bold font-mono">
                        {move || {
                            let n = name.get();
                            if n.is_empty() { "?".to_string() } else { n.chars().next().unwrap_or('?').to_string().to_uppercase() }
                        }}
                    </div>
                    <div>
                        <div class="flex items-center gap-2">
                            <h1 class="text-xl font-extrabold tracking-tight text-on-surface">
                                {move || if name.get().is_empty() { instance_id() } else { name.get() }}
                            </h1>
                            <span class=move || {
                                format!("inline-flex items-center gap-1 px-2.5 py-0.5 rounded-full border text-[9px] font-bold uppercase tracking-widest {}",
                                    if is_suspended.get() { "text-error bg-error/10 border-error/20" }
                                    else { "text-emerald-400 bg-emerald-400/10 border-emerald-400/20" }
                                )
                            }>
                                {move || if is_suspended.get() { "Suspended" } else { "Live" }}
                            </span>
                        </div>
                        <p class="text-[11px] text-on-surface-variant font-mono mt-1">
                            "ID: " <code class="text-primary">{move || instance_id()}</code>
                            " · domain: " <code class="text-on-surface-variant/80">{move || slug.get()} ".atlas-platform.com"</code>
                        </p>
                    </div>
                </div>
                <div>
                    <button 
                        class=move || {
                            format!("px-4 py-2 rounded-lg text-xs font-bold transition-all {}",
                                if is_suspended.get() { "btn-primary shadow-md active:scale-95" }
                                else { "border border-error/40 text-error bg-error/5 hover:bg-error hover:text-white" }
                            )
                        }
                        on:click=toggle_status
                    >
                        {move || if is_suspended.get() { "Resume Network" } else { "Suspend Network" }}
                    </button>
                </div>
            </div>

            // ── Navigation Tabs Bar ──
            <div class="border-b border-outline-variant/20 flex gap-1 bg-surface-container-low/40 px-2 rounded-t-xl">
                <button 
                    class=move || format!("px-4 py-3 text-xs font-semibold border-b-2 transition-all {}", if active_tab.get() == "overview" { "border-primary text-primary font-bold" } else { "border-transparent text-on-surface-variant hover:text-on-surface" })
                    on:click=move |_| active_tab.set("overview".to_string())
                >
                    "Overview & Settings"
                </button>
                <button 
                    class=move || format!("px-4 py-3 text-xs font-semibold border-b-2 transition-all {}", if active_tab.get() == "domains" { "border-primary text-primary font-bold" } else { "border-transparent text-on-surface-variant hover:text-on-surface" })
                    on:click=move |_| active_tab.set("domains".to_string())
                >
                    "Domains & SSL"
                </button>
                <button 
                    class=move || format!("px-4 py-3 text-xs font-semibold border-b-2 transition-all {}", if active_tab.get() == "tenants" { "border-primary text-primary font-bold" } else { "border-transparent text-on-surface-variant hover:text-on-surface" })
                    on:click=move |_| active_tab.set("tenants".to_string())
                >
                    "Syndicated Tenants"
                </button>
                <button 
                    class=move || format!("px-4 py-3 text-xs font-semibold border-b-2 transition-all {}", if active_tab.get() == "branding" { "border-primary text-primary font-bold" } else { "border-transparent text-on-surface-variant hover:text-on-surface" })
                    on:click=move |_| active_tab.set("branding".to_string())
                >
                    "Theme & Branding"
                </button>
                <button 
                    class=move || format!("px-4 py-3 text-xs font-semibold border-b-2 transition-all {}", if active_tab.get() == "telemetry" { "border-primary text-primary font-bold" } else { "border-transparent text-on-surface-variant hover:text-on-surface" })
                    on:click=move |_| active_tab.set("telemetry".to_string())
                >
                    "Telemetry & Logs"
                </button>
            </div>

            // ── Tab Panes ──
            <div class="bg-surface-container-low border-x border-b border-outline-variant/20 p-6 rounded-b-2xl min-h-[360px]">
                
                // Tab Pane 1: Overview & Settings
                <Show when=move || active_tab.get() == "overview">
                    <div class="grid grid-cols-1 lg:grid-cols-3 gap-6">
                        <div class="lg:col-span-2 space-y-6">
                            <div class="bg-surface-container p-6 rounded-xl border border-outline-variant/20 space-y-4">
                                <h3 class="text-xs font-bold text-on-surface uppercase tracking-wider">"Instance Info"</h3>
                                <div class="space-y-1.5">
                                    <label class="text-[10px] font-bold text-on-surface-variant uppercase tracking-wider">"Network Name"</label>
                                    <input 
                                        type="text" 
                                        class="w-full bg-[#05183c] border border-outline-variant/30 text-on-surface text-xs rounded-lg px-3 py-2 focus:ring-1 focus:ring-primary focus:border-primary outline-none transition-all"
                                        prop:value=name
                                        on:input=move |ev| name.set(event_target_value(&ev))
                                    />
                                </div>
                                <div class="space-y-1.5">
                                    <label class="text-[10px] font-bold text-on-surface-variant uppercase tracking-wider">"Subdomain Routing Slug"</label>
                                    <input 
                                        type="text" 
                                        class="w-full bg-[#05183c] border border-outline-variant/30 text-on-surface text-xs rounded-lg px-3 py-2 focus:ring-1 focus:ring-primary focus:border-primary outline-none transition-all"
                                        prop:value=slug
                                        on:input=move |ev| slug.set(event_target_value(&ev))
                                    />
                                </div>
                            </div>

                            <div class="bg-surface-container p-6 rounded-xl border border-outline-variant/20 space-y-3">
                                <h3 class="text-xs font-bold text-on-surface uppercase tracking-wider">"Instance Capabilities"</h3>
                                
                                <div class="flex items-center justify-between py-2 border-b border-outline-variant/10">
                                    <div class="flex flex-col gap-0.5">
                                        <span class="text-xs font-bold text-on-surface">"Long-Term Rentals (LTR)"</span>
                                        <span class="text-[10px] text-on-surface-variant/70">"Enable landlord syndications, leasing files, and unit directories."</span>
                                    </div>
                                    <label class="relative inline-flex items-center cursor-pointer">
                                        <input 
                                            type="checkbox" 
                                            class="sr-only peer" 
                                            prop:checked=cap_ltr
                                            on:change=move |ev| cap_ltr.set(event_target_checked(&ev))
                                        />
                                        <div class="w-8 h-4 bg-surface-container-highest peer-focus:outline-none rounded-full peer peer-checked:after:translate-x-full peer-checked:after:border-white after:content-[''] after:absolute after:top-[1px] after:left-[1px] after:bg-white after:border-gray-300 after:border after:rounded-full after:h-3.5 after:w-3.5 after:transition-all peer-checked:bg-primary"></div>
                                    </label>
                                </div>

                                <div class="flex items-center justify-between py-2 border-b border-outline-variant/10">
                                    <div class="flex flex-col gap-0.5">
                                        <span class="text-xs font-bold text-on-surface">"Short-Term Stays (STR)"</span>
                                        <span class="text-[10px] text-on-surface-variant/70">"Enable short-stay vacation postings and calendar reservations."</span>
                                    </div>
                                    <label class="relative inline-flex items-center cursor-pointer">
                                        <input 
                                            type="checkbox" 
                                            class="sr-only peer" 
                                            prop:checked=cap_str
                                            on:change=move |ev| cap_str.set(event_target_checked(&ev))
                                        />
                                        <div class="w-8 h-4 bg-surface-container-highest peer-focus:outline-none rounded-full peer peer-checked:after:translate-x-full peer-checked:after:border-white after:content-[''] after:absolute after:top-[1px] after:left-[1px] after:bg-white after:border-gray-300 after:border after:rounded-full after:h-3.5 after:w-3.5 after:transition-all peer-checked:bg-primary"></div>
                                    </label>
                                </div>

                                <div class="flex items-center justify-between py-2">
                                    <div class="flex flex-col gap-0.5">
                                        <span class="text-xs font-bold text-on-surface">"Vendor Marketplace"</span>
                                        <span class="text-[10px] text-on-surface-variant/70">"Expose maintenance repair dispatch and syndicated contractor bookings."</span>
                                    </div>
                                    <label class="relative inline-flex items-center cursor-pointer">
                                        <input 
                                            type="checkbox" 
                                            class="sr-only peer" 
                                            prop:checked=cap_vendor
                                            on:change=move |ev| cap_vendor.set(event_target_checked(&ev))
                                        />
                                        <div class="w-8 h-4 bg-surface-container-highest peer-focus:outline-none rounded-full peer peer-checked:after:translate-x-full peer-checked:after:border-white after:content-[''] after:absolute after:top-[1px] after:left-[1px] after:bg-white after:border-gray-300 after:border after:rounded-full after:h-3.5 after:w-3.5 after:transition-all peer-checked:bg-primary"></div>
                                    </label>
                                </div>
                            </div>
                        </div>

                        <div class="space-y-6">
                            <div class="bg-surface-container p-6 rounded-xl border border-outline-variant/20 space-y-4">
                                <h3 class="text-xs font-bold text-on-surface uppercase tracking-wider">"Geographics & Scope"</h3>
                                <div class="space-y-2">
                                    <label class="text-[10px] font-bold text-on-surface-variant uppercase tracking-wider">"Served Markets"</label>
                                    <div class="space-y-1.5 text-xs text-on-surface-variant">
                                        <label class="flex items-center gap-2 cursor-pointer">
                                            <input type="checkbox" class="rounded border-outline-variant text-primary focus:ring-primary h-4 w-4" prop:checked=market_chi on:change=move |ev| market_chi.set(event_target_checked(&ev)) />
                                            <span>"Chicago, IL"</span>
                                        </label>
                                        <label class="flex items-center gap-2 cursor-pointer">
                                            <input type="checkbox" class="rounded border-outline-variant text-primary focus:ring-primary h-4 w-4" prop:checked=market_rio on:change=move |ev| market_rio.set(event_target_checked(&ev)) />
                                            <span>"Rio de Janeiro, BR"</span>
                                        </label>
                                        <label class="flex items-center gap-2 cursor-pointer">
                                            <input type="checkbox" class="rounded border-outline-variant text-primary focus:ring-primary h-4 w-4" prop:checked=market_mia on:change=move |ev| market_mia.set(event_target_checked(&ev)) />
                                            <span>"Miami, FL"</span>
                                        </label>
                                    </div>
                                </div>
                                <div class="space-y-1.5 pt-2">
                                    <label class="text-[10px] font-bold text-on-surface-variant uppercase tracking-wider">"Primary Currency"</label>
                                    <select 
                                        class="w-full bg-[#05183c] border border-outline-variant/30 text-on-surface text-xs rounded-lg px-3 py-2 focus:ring-1 focus:ring-primary focus:border-primary outline-none transition-all"
                                        on:change=move |ev| currency.set(event_target_value(&ev))
                                    >
                                        <option value="USD" selected=move || currency.get() == "USD">"USD ($)"</option>
                                        <option value="BRL" selected=move || currency.get() == "BRL">"BRL (R$)"</option>
                                        <option value="BTC" selected=move || currency.get() == "BTC">"BTC (₿)"</option>
                                    </select>
                                </div>
                            </div>

                            <div class="flex justify-end gap-3">
                                <button class="btn-ghost px-4 py-2 border border-outline-variant/30 hover:bg-surface-bright/20 rounded-lg text-xs font-semibold" on:click=move |_| toast.show_toast("Discarded", "Modifications discarded.", "warning")>"Discard"</button>
                                <button class="btn-primary px-4 py-2 rounded-lg text-xs font-semibold shadow-md active:scale-95 transition-all" on:click=save_overview>"Save Settings"</button>
                            </div>
                        </div>
                    </div>
                </Show>

                // Tab Pane 2: Domains & SSL
                <Show when=move || active_tab.get() == "domains">
                    <div class="space-y-6">
                        <div class="bg-surface-container p-6 rounded-xl border border-outline-variant/20 space-y-4">
                            <h3 class="text-xs font-bold text-on-surface uppercase tracking-wider">"Custom Domain Mappings"</h3>
                            
                            <div class="flex gap-2 max-w-md">
                                <input 
                                    type="text" 
                                    class="flex-1 bg-[#05183c] border border-outline-variant/30 text-on-surface text-xs rounded-lg px-3 py-2 focus:ring-1 focus:ring-primary focus:border-primary outline-none transition-all"
                                    placeholder="e.g. rent.mynetwork.com"
                                    prop:value=new_domain
                                    on:input=move |ev| new_domain.set(event_target_value(&ev))
                                />
                                <button class="btn-primary px-3 py-2 rounded-lg text-xs font-semibold shrink-0" on:click=add_domain_action>"Add Domain"</button>
                            </div>

                            <Suspense fallback=move || view! { <div class="text-xs text-on-surface-variant">"Loading domains…"</div> }>
                            <div class="overflow-x-auto">
                                <table class="w-full border-collapse text-left text-xs">
                                    <thead>
                                        <tr class="bg-surface-container-high/40">
                                            <th class="p-3 font-semibold text-on-surface-variant/80 border-b border-outline-variant/20">"Domain"</th>
                                            <th class="p-3 font-semibold text-on-surface-variant/80 border-b border-outline-variant/20">"CNAME Target"</th>
                                            <th class="p-3 font-semibold text-on-surface-variant/80 border-b border-outline-variant/20 text-right">"Actions"</th>
                                        </tr>
                                    </thead>
                                    <tbody class="divide-y divide-outline-variant/10">
                                        {move || domains_res.get().unwrap_or_default().into_iter().map(|dom| {
                                            let dom_remove = dom.clone();
                                            let id_str = instance_id();
                                            let toast2 = toast.clone();
                                            view! {
                                                <tr class="hover:bg-surface-container-high/20 transition-colors">
                                                    <td class="p-3 font-mono text-on-surface font-semibold">{dom.clone()}</td>
                                                    <td class="p-3 font-mono text-on-surface-variant">"app.atlas-platform.com"</td>
                                                    <td class="p-3 text-right">
                                                        <button 
                                                            class="p-1 hover:bg-error/10 text-on-surface-variant hover:text-error rounded transition-colors flex items-center justify-center ml-auto"
                                                            on:click=move |_| {
                                                                let id = id_str.clone();
                                                                let d  = dom_remove.clone();
                                                                let t  = toast2.clone();
                                                                leptos::task::spawn_local(async move {
                                                                    match api_remove_domain(id, d).await {
                                                                        Ok(_)  => t.show_toast("Domain Removed", "Domain mapping revoked.", "warning"),
                                                                        Err(e) => t.show_toast("Error", &e, "error"),
                                                                    }
                                                                });
                                                            }
                                                        >
                                                            <span class="material-symbols-outlined text-sm">"delete"</span>
                                                        </button>
                                                    </td>
                                                </tr>
                                            }
                                        }).collect_view()}
                                    </tbody>
                                </table>
                            </div>
                            </Suspense>

                            <div class="bg-[#05183c]/50 border border-outline-variant/20 p-4 rounded-lg text-xs leading-relaxed space-y-1">
                                <p class="font-bold text-on-surface">"DNS Configuration Instructions"</p>
                                <p class="text-on-surface-variant">
                                    "To map a custom domain to this network instance, configure a CNAME record with your DNS provider pointing to: " 
                                    <strong class="text-primary font-mono bg-primary/10 border border-primary/20 px-1 py-0.5 rounded ml-1">"app.atlas-platform.com"</strong>
                                </p>
                            </div>
                        </div>
                    </div>
                </Show>

                // Tab Pane 3: Syndicated Tenants
                <Show when=move || active_tab.get() == "tenants">
                    <div class="space-y-6">
                        <div class="bg-surface-container p-6 rounded-xl border border-outline-variant/20 space-y-4">
                            <div class="flex justify-between items-center">
                                <h3 class="text-xs font-bold text-on-surface uppercase tracking-wider">"Connected Tenants"</h3>
                                <button class="btn-primary px-3 py-1.5 rounded-lg text-xs font-semibold shadow-xs" on:click=move |_| toast.show_toast("Connect", "Connection wizard coming soon.", "success")>
                                    "+ Connect Tenant"
                                </button>
                            </div>
                            <Suspense fallback=move || view! { <div class="text-xs text-on-surface-variant">"Loading tenants…"</div> }>
                            <div class="overflow-x-auto">
                                <table class="w-full border-collapse text-left text-xs">
                                    <thead>
                                        <tr class="bg-surface-container-high/40">
                                            <th class="p-3 font-semibold text-on-surface-variant/80 border-b border-outline-variant/20">"Tenant Name"</th>
                                            <th class="p-3 font-semibold text-on-surface-variant/80 border-b border-outline-variant/20">"Plan"</th>
                                            <th class="p-3 font-semibold text-on-surface-variant/80 border-b border-outline-variant/20">"Profiles"</th>
                                            <th class="p-3 font-semibold text-on-surface-variant/80 border-b border-outline-variant/20">"Listings"</th>
                                        </tr>
                                    </thead>
                                    <tbody class="divide-y divide-outline-variant/10">
                                        {move || tenants_res.get().unwrap_or_default().into_iter().map(|t| {
                                            view! {
                                                <tr class="hover:bg-surface-container-high/20 transition-colors">
                                                    <td class="p-3 font-semibold text-on-surface">{t.name}</td>
                                                    <td class="p-3 text-on-surface-variant font-mono">{t.plan.unwrap_or_else(|| "—".to_string())}</td>
                                                    <td class="p-3 text-on-surface">{t.profile_count.to_string()}</td>
                                                    <td class="p-3 text-on-surface">{t.listing_count.to_string()}</td>
                                                </tr>
                                            }
                                        }).collect_view()}
                                    </tbody>
                                </table>
                            </div>
                            </Suspense>
                        </div>
                    </div>
                </Show>

                // Tab Pane 4: Theme & Branding
                <Show when=move || active_tab.get() == "branding">
                    <div class="grid grid-cols-1 lg:grid-cols-3 gap-6">
                        <div class="lg:col-span-2 space-y-6">
                            <div class="bg-surface-container p-6 rounded-xl border border-outline-variant/20 space-y-4">
                                <h3 class="text-xs font-bold text-on-surface uppercase tracking-wider">"Colors & Theme Mode"</h3>
                                <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
                                    <div class="space-y-1.5">
                                        <label class="text-[10px] font-bold text-on-surface-variant uppercase tracking-wider">"Branding Theme Style"</label>
                                        <select 
                                            class="w-full bg-[#05183c] border border-outline-variant/30 text-on-surface text-xs rounded-lg px-3 py-2 focus:ring-1 focus:ring-primary focus:border-primary outline-none transition-all"
                                            on:change=move |ev| theme_mode.set(event_target_value(&ev))
                                        >
                                            <option value="dark-slate" selected=move || theme_mode.get() == "dark-slate">"Dark Slate (Institutional)"</option>
                                            <option value="light-clean" selected=move || theme_mode.get() == "light-clean">"Light Clean (Standard)"</option>
                                            <option value="high-contrast" selected=move || theme_mode.get() == "high-contrast">"High Contrast (Accessible)"</option>
                                        </select>
                                    </div>
                                    <div class="space-y-1.5">
                                        <label class="text-[10px] font-bold text-on-surface-variant uppercase tracking-wider">"Primary Brand Color"</label>
                                        <div class="flex gap-2">
                                            <input 
                                                type="color" 
                                                class="w-8 h-8 rounded border border-outline-variant/30 bg-transparent cursor-pointer"
                                                prop:value=primary_color
                                                on:input=move |ev| primary_color.set(event_target_value(&ev))
                                            />
                                            <input 
                                                type="text" 
                                                class="flex-1 bg-[#05183c] border border-outline-variant/30 text-on-surface text-xs rounded-lg px-3 py-2 outline-none font-mono"
                                                prop:value=primary_color
                                                on:input=move |ev| primary_color.set(event_target_value(&ev))
                                            />
                                        </div>
                                    </div>
                                </div>
                            </div>

                            <div class="bg-surface-container p-6 rounded-xl border border-outline-variant/20 space-y-4">
                                <h3 class="text-xs font-bold text-on-surface uppercase tracking-wider">"Typography & Typography Scale"</h3>
                                <div class="space-y-1.5">
                                    <label class="text-[10px] font-bold text-on-surface-variant uppercase tracking-wider">"Primary Display Font"</label>
                                    <select 
                                        class="w-full bg-[#05183c] border border-outline-variant/30 text-on-surface text-xs rounded-lg px-3 py-2 focus:ring-1 focus:ring-primary focus:border-primary outline-none transition-all"
                                        on:change=move |ev| selected_font.set(event_target_value(&ev))
                                    >
                                        <option value="inter" selected=move || selected_font.get() == "inter">"Inter (Modern & Clean)"</option>
                                        <option value="roboto" selected=move || selected_font.get() == "roboto">"Roboto (Technical & Regular)"</option>
                                        <option value="outfit" selected=move || selected_font.get() == "outfit">"Outfit (Vibrant & Warm)"</option>
                                    </select>
                                </div>
                            </div>
                        </div>

                        <div class="space-y-6">
                            <div class="bg-surface-container p-6 rounded-xl border border-outline-variant/20 space-y-3 text-xs">
                                <h3 class="text-xs font-bold text-on-surface uppercase tracking-wider">"Header Links Editor"</h3>
                                <div class="space-y-2">
                                    <div class="flex items-center justify-between p-2.5 bg-surface-container-high/40 border border-outline-variant/20 rounded-lg">
                                        <div class="flex items-center gap-2 font-semibold text-on-surface">
                                            <span class="material-symbols-outlined text-sm text-outline-variant">"drag_indicator"</span>
                                            <span>"Directory"</span>
                                        </div>
                                        <span class="text-[10px] font-mono text-outline-variant">"/search"</span>
                                    </div>
                                    <div class="flex items-center justify-between p-2.5 bg-surface-container-high/40 border border-outline-variant/20 rounded-lg">
                                        <div class="flex items-center gap-2 font-semibold text-on-surface">
                                            <span class="material-symbols-outlined text-sm text-outline-variant">"drag_indicator"</span>
                                            <span>"Landlords"</span>
                                        </div>
                                        <span class="text-[10px] font-mono text-outline-variant">"/landlords"</span>
                                    </div>
                                    <div class="flex items-center justify-between p-2.5 bg-surface-container-high/40 border border-outline-variant/20 rounded-lg">
                                        <div class="flex items-center gap-2 font-semibold text-on-surface">
                                            <span class="material-symbols-outlined text-sm text-outline-variant">"drag_indicator"</span>
                                            <span>"Apply Now"</span>
                                        </div>
                                        <span class="text-[10px] font-mono text-outline-variant">"/apply"</span>
                                    </div>
                                </div>
                            </div>

                            <div class="flex justify-end gap-3">
                                <button class="btn-ghost px-4 py-2 border border-outline-variant/30 hover:bg-surface-bright/20 rounded-lg text-xs font-semibold" on:click=move |_| toast.show_toast("Discarded", "Branding adjustments discarded.", "warning")>"Discard"</button>
                                <button class="btn-primary px-4 py-2 rounded-lg text-xs font-semibold shadow-md active:scale-95 transition-all" on:click=save_branding>"Save Theme"</button>
                            </div>
                        </div>
                    </div>
                </Show>

                // Tab Pane 5: Telemetry & Logs
                <Show when=move || active_tab.get() == "telemetry">
                    <div class="space-y-6">
                        <div class="grid grid-cols-2 md:grid-cols-4 gap-4">
                            <div class="bg-surface-container p-4 rounded-xl border border-outline-variant/20 flex flex-col gap-1.5">
                                <span class="text-[10px] font-bold text-on-surface-variant uppercase tracking-wider">"Profiles"</span>
                                <span class="text-xl font-bold text-on-surface">
                                    {move || tenants_res.get().map(|ts| ts.iter().map(|t| t.profile_count).sum::<u64>().to_string()).unwrap_or_else(|| "—".to_string())}
                                </span>
                            </div>
                            <div class="bg-surface-container p-4 rounded-xl border border-outline-variant/20 flex flex-col gap-1.5">
                                <span class="text-[10px] font-bold text-on-surface-variant uppercase tracking-wider">"Total Listings"</span>
                                <span class="text-xl font-bold text-on-surface">
                                    {move || tenants_res.get().map(|ts| ts.iter().map(|t| t.listing_count).sum::<u64>().to_string()).unwrap_or_else(|| "—".to_string())}
                                </span>
                            </div>
                            <div class="bg-surface-container p-4 rounded-xl border border-outline-variant/20 flex flex-col gap-1.5">
                                <span class="text-[10px] font-bold text-on-surface-variant uppercase tracking-wider">"Custom Domains"</span>
                                <span class="text-xl font-bold text-on-surface">
                                    {move || domains_res.get().map(|ds| ds.len().to_string()).unwrap_or_else(|| "—".to_string())}
                                </span>
                            </div>
                            <div class="bg-surface-container p-4 rounded-xl border border-outline-variant/20 flex flex-col gap-1.5">
                                <span class="text-[10px] font-bold text-on-surface-variant uppercase tracking-wider">"Status"</span>
                                <span class="text-xl font-bold" style=move || if is_suspended.get() { "color:var(--error)" } else { "color:var(--green)" }>
                                    {move || if is_suspended.get() { "Suspended" } else { "Live" }}
                                </span>
                            </div>
                        </div>
                        <div class="bg-surface-container p-6 rounded-xl border border-outline-variant/20">
                            <p class="text-xs text-on-surface-variant">"Real-time event streaming will be available once the telemetry pipeline (G-24) is connected to this instance."</p>
                        </div>
                    </div>
                </Show>



            </div>
        </div>
    }
}
