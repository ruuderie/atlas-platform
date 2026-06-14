use leptos::prelude::*;

#[component]
pub fn NetworkDetail() -> impl IntoView {
    let toast = use_context::<crate::app::GlobalToast>().expect("toast context");
    
    // Active Tab State
    let active_tab = RwSignal::new("overview".to_string());
    
    // Status State
    let is_suspended = RwSignal::new(false);
    
    // Settings inputs
    let name = RwSignal::new("leira Rentals".to_string());
    let slug = RwSignal::new("leira-rentals".to_string());
    let cap_ltr = RwSignal::new(true);
    let cap_str = RwSignal::new(false);
    let cap_vendor = RwSignal::new(true);
    let market_chi = RwSignal::new(true);
    let market_rio = RwSignal::new(true);
    let market_mia = RwSignal::new(false);
    let currency = RwSignal::new("BRL".to_string());
    
    // Custom domain mappings
    let new_domain = RwSignal::new("".to_string());
    let domains = RwSignal::new(vec![
        ("leira-rentals.app".to_string(), "Primary".to_string(), "app.atlas-platform.com".to_string(), "✓ Active (Cloudflare)".to_string()),
        ("rent.leira-chicago.com".to_string(), "Verifying".to_string(), "app.atlas-platform.com".to_string(), "Generating...".to_string())
    ]);
    
    // Syndicated Tenants
    let tenants = RwSignal::new(vec![
        ("Oakwood Property Management".to_string(), "Professional".to_string(), "28 / 34 listings".to_string(), 82, "2025-11-12".to_string()),
        ("South Loop STRs".to_string(), "Enterprise".to_string(), "12 / 12 listings".to_string(), 100, "2026-02-18".to_string())
    ]);

    // Theme branding
    let theme_mode = RwSignal::new("dark-slate".to_string());
    let primary_color = RwSignal::new("#0A84FF".to_string());
    let selected_font = RwSignal::new("inter".to_string());

    let toggle_status = move |_| {
        let suspended = is_suspended.get();
        is_suspended.set(!suspended);
        if suspended {
            toast.show_toast("Network Status", "Network instance resumed successfully.", "success");
        } else {
            toast.show_toast("Network Status", "Network instance suspended.", "warning");
        }
    };

    let add_domain = move |_| {
        let domain_str = new_domain.get().trim().to_string();
        if domain_str.is_empty() {
            return;
        }
        domains.update(|list| {
            list.push((domain_str, "Secondary".to_string(), "app.atlas-platform.com".to_string(), "Active".to_string()));
        });
        new_domain.set("".to_string());
        toast.show_toast("Domain Added", "Custom domain added successfully.", "success");
    };

    let save_overview = move |_| {
        toast.show_toast("Settings Saved", "Overview and capability settings saved successfully.", "success");
    };

    let save_branding = move |_| {
        toast.show_toast("Branding Saved", "Theme and color customization saved.", "success");
    };

    view! {
        <div class="space-y-6">
            // ── Breadcrumb ──
            <nav class="flex items-center gap-2 text-on-surface-variant text-xs mb-2">
                <a href="/" class="hover:text-primary transition-colors">"Dashboard"</a>
                <span class="material-symbols-outlined text-[12px]">"chevron_right"</span>
                <a href="/network" class="hover:text-primary transition-colors">"Network Instances"</a>
                <span class="material-symbols-outlined text-[12px]">"chevron_right"</span>
                <span class="text-primary/70">{move || name.get()}</span>
            </nav>

            // ── Instance Identity Header ──
            <div class="bg-surface-container-low border border-outline-variant/20 p-6 rounded-2xl shadow-sm flex flex-col md:flex-row justify-between items-start md:items-center gap-4">
                <div class="flex items-center gap-4">
                    <div class="w-12 h-12 rounded-xl bg-primary-container border border-primary/20 flex items-center justify-center text-primary text-xl font-bold font-mono">
                        {move || name.get().chars().next().unwrap_or('N').to_string().to_uppercase()}
                    </div>
                    <div>
                        <div class="flex items-center gap-2">
                            <h1 class="text-xl font-extrabold tracking-tight text-on-surface">{move || name.get()}</h1>
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
                            "ID: " <code class="text-primary">"inst_4c2d-9b8a-7e1f"</code> " · domain: " <code class="text-on-surface-variant/80">{move || slug.get()} ".atlas-platform.com"</code>
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
                                    placeholder="e.g. rent.leira.com"
                                    prop:value=new_domain
                                    on:input=move |ev| new_domain.set(event_target_value(&ev))
                                />
                                <button class="btn-primary px-3 py-2 rounded-lg text-xs font-semibold shrink-0" on:click=add_domain>"Add Domain"</button>
                            </div>

                            <div class="overflow-x-auto">
                                <table class="w-full border-collapse text-left text-xs">
                                    <thead>
                                        <tr class="bg-surface-container-high/40">
                                            <th class="p-3 font-semibold text-on-surface-variant/80 border-b border-outline-variant/20">"Domain"</th>
                                            <th class="p-3 font-semibold text-on-surface-variant/80 border-b border-outline-variant/20">"Status"</th>
                                            <th class="p-3 font-semibold text-on-surface-variant/80 border-b border-outline-variant/20">"CNAME Target"</th>
                                            <th class="p-3 font-semibold text-on-surface-variant/80 border-b border-outline-variant/20">"SSL Security"</th>
                                            <th class="p-3 font-semibold text-on-surface-variant/80 border-b border-outline-variant/20 text-right">"Actions"</th>
                                        </tr>
                                    </thead>
                                    <tbody class="divide-y divide-outline-variant/10">
                                        {move || domains.get().into_iter().enumerate().map(|(idx, (dom, status, target, ssl))| {
                                            let status_cls = status.clone();
                                            let status_show = status.clone();
                                            view! {
                                                <tr class="hover:bg-surface-container-high/20 transition-colors">
                                                    <td class="p-3 font-mono text-on-surface font-semibold">{dom}</td>
                                                    <td class="p-3">
                                                        <span class=move || {
                                                            let s = &status_cls;
                                                            format!("px-2 py-0.5 rounded text-[9px] font-bold uppercase tracking-wider border {}",
                                                                if s == "Primary" { "text-primary border-primary/20 bg-primary/10" }
                                                                else if s == "Verifying" { "text-amber-400 border-amber-500/20 bg-amber-500/10" }
                                                                else { "text-on-surface-variant border-outline-variant/20 bg-surface-container" }
                                                            )
                                                        }>
                                                            {status}
                                                        </span>
                                                    </td>
                                                    <td class="p-3 font-mono text-on-surface-variant">{target}</td>
                                                    <td class="p-3 text-emerald-400 font-semibold">{ssl}</td>
                                                    <td class="p-3 text-right">
                                                        <Show when=move || status_show.clone() != "Primary">
                                                            <button 
                                                                class="p-1 hover:bg-error/10 text-on-surface-variant hover:text-error rounded transition-colors flex items-center justify-center ml-auto"
                                                                on:click=move |_| {
                                                                    domains.update(|list| { list.remove(idx); });
                                                                    toast.show_toast("Domain Removed", "Domain mapping revoked.", "warning");
                                                                }
                                                            >
                                                                <span class="material-symbols-outlined text-sm">"delete"</span>
                                                            </button>
                                                        </Show>
                                                    </td>
                                                </tr>
                                            }
                                        }).collect_view()}
                                    </tbody>
                                </table>
                            </div>

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
                                <h3 class="text-xs font-bold text-on-surface uppercase tracking-wider">"Connected Listing Tenants"</h3>
                                <button class="btn-primary px-3 py-1.5 rounded-lg text-xs font-semibold shadow-xs" on:click=move |_| toast.show_toast("Connect", "Listing connection wizard started.", "success")>
                                    "+ Connect Tenant"
                                </button>
                            </div>

                            <div class="overflow-x-auto">
                                <table class="w-full border-collapse text-left text-xs">
                                    <thead>
                                        <tr class="bg-surface-container-high/40">
                                            <th class="p-3 font-semibold text-on-surface-variant/80 border-b border-outline-variant/20">"Tenant Name"</th>
                                            <th class="p-3 font-semibold text-on-surface-variant/80 border-b border-outline-variant/20">"Account Plan"</th>
                                            <th class="p-3 font-semibold text-on-surface-variant/80 border-b border-outline-variant/20">"Syndicated Listings"</th>
                                            <th class="p-3 font-semibold text-on-surface-variant/80 border-b border-outline-variant/20">"Sync Status"</th>
                                            <th class="p-3 font-semibold text-on-surface-variant/80 border-b border-outline-variant/20">"Date Linked"</th>
                                            <th class="p-3 font-semibold text-on-surface-variant/80 border-b border-outline-variant/20 text-right">"Actions"</th>
                                        </tr>
                                    </thead>
                                    <tbody class="divide-y divide-outline-variant/10">
                                        {move || tenants.get().into_iter().enumerate().map(|(idx, (t_name, plan, listings, sync_pct, date_linked))| {
                                            view! {
                                                <tr class="hover:bg-surface-container-high/20 transition-colors">
                                                    <td class="p-3 font-semibold text-on-surface">{t_name}</td>
                                                    <td class="p-3 text-on-surface-variant font-mono">{plan}</td>
                                                    <td class="p-3 text-on-surface font-semibold">{listings}</td>
                                                    <td class="p-3">
                                                        <div class="flex flex-col gap-1 w-28">
                                                            <div class="flex justify-between text-[10px] text-on-surface-variant">
                                                                <span>"Synced"</span>
                                                                <span class="font-bold">{sync_pct} "%"</span>
                                                            </div>
                                                            <div class="h-1.5 w-full bg-surface-container-highest rounded-full overflow-hidden border border-outline-variant/10">
                                                                <div class="h-full bg-primary" style=format!("width: {}%;", sync_pct)></div>
                                                            </div>
                                                        </div>
                                                    </td>
                                                    <td class="p-3 text-on-surface-variant font-mono">{date_linked}</td>
                                                    <td class="p-3 text-right">
                                                        <button 
                                                            class="p-1 hover:bg-error/10 text-on-surface-variant hover:text-error rounded transition-colors flex items-center justify-center ml-auto"
                                                            on:click=move |_| {
                                                                tenants.update(|list| { list.remove(idx); });
                                                                toast.show_toast("Tenant Disconnected", "Tenant syndication channel revoked.", "warning");
                                                            }
                                                        >
                                                            <span class="material-symbols-outlined text-sm">"link_off"</span>
                                                        </button>
                                                    </td>
                                                </tr>
                                            }
                                        }).collect_view()}
                                    </tbody>
                                </table>
                            </div>
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
                        // KPI widgets grid
                        <div class="grid grid-cols-2 md:grid-cols-4 gap-4">
                            <div class="bg-surface-container p-4 rounded-xl border border-outline-variant/20 flex flex-col gap-1.5">
                                <span class="text-[10px] font-bold text-on-surface-variant uppercase tracking-wider">"Daily Active Users"</span>
                                <span class="text-xl font-bold text-on-surface">"1,482"</span>
                            </div>
                            <div class="bg-surface-container p-4 rounded-xl border border-outline-variant/20 flex flex-col gap-1.5">
                                <span class="text-[10px] font-bold text-on-surface-variant uppercase tracking-wider">"Syndicated Listings"</span>
                                <span class="text-xl font-bold text-on-surface">"40"</span>
                            </div>
                            <div class="bg-surface-container p-4 rounded-xl border border-outline-variant/20 flex flex-col gap-1.5">
                                <span class="text-[10px] font-bold text-on-surface-variant uppercase tracking-wider">"Sync Success Rate"</span>
                                <span class="text-xl font-bold text-emerald-400">"99.8%"</span>
                            </div>
                            <div class="bg-surface-container p-4 rounded-xl border border-outline-variant/20 flex flex-col gap-1.5">
                                <span class="text-[10px] font-bold text-on-surface-variant uppercase tracking-wider">"API Latency"</span>
                                <span class="text-xl font-bold text-on-surface">"42ms"</span>
                            </div>
                        </div>

                        // Recent Sync Events Table
                        <div class="bg-surface-container p-6 rounded-xl border border-outline-variant/20 space-y-3">
                            <h3 class="text-xs font-bold text-on-surface uppercase tracking-wider">"Recent Syndication Sync Log"</h3>
                            <div class="overflow-x-auto">
                                <table class="w-full border-collapse text-left text-xs">
                                    <thead>
                                        <tr class="bg-surface-container-high/40">
                                            <th class="p-3 font-semibold text-on-surface-variant/80 border-b border-outline-variant/20">"Timestamp"</th>
                                            <th class="p-3 font-semibold text-on-surface-variant/80 border-b border-outline-variant/20">"Event Code"</th>
                                            <th class="p-3 font-semibold text-on-surface-variant/80 border-b border-outline-variant/20">"Severity"</th>
                                            <th class="p-3 font-semibold text-on-surface-variant/80 border-b border-outline-variant/20">"Details"</th>
                                        </tr>
                                    </thead>
                                    <tbody class="divide-y divide-outline-variant/10 text-on-surface-variant font-mono">
                                        <tr class="hover:bg-surface-container-high/20 transition-colors">
                                            <td class="p-3 text-[11px] whitespace-nowrap">"2026-06-14 11:10:04"</td>
                                            <td class="p-3 font-bold text-primary">"SYNC_COMPLETED"</td>
                                            <td class="p-3 text-emerald-400 font-bold">"INFO"</td>
                                            <td class="p-3 text-on-surface">"Successfully synced 28 Oakwood Listings into London cache."</td>
                                        </tr>
                                        <tr class="hover:bg-surface-container-high/20 transition-colors">
                                            <td class="p-3 text-[11px] whitespace-nowrap">"2026-06-14 10:42:18"</td>
                                            <td class="p-3 font-bold text-primary">"SSL_RENEWED"</td>
                                            <td class="p-3 text-emerald-400 font-bold">"INFO"</td>
                                            <td class="p-3 text-on-surface">"Let's Encrypt SSL certificate renewed for leira-rentals.app."</td>
                                        </tr>
                                        <tr class="hover:bg-surface-container-high/20 transition-colors">
                                            <td class="p-3 text-[11px] whitespace-nowrap">"2026-06-14 09:15:33"</td>
                                            <td class="p-3 font-bold text-primary">"SYNC_SKIPPED"</td>
                                            <td class="p-3 text-amber-400 font-bold">"WARN"</td>
                                            <td class="p-3 text-on-surface">"Listing ID lst_9812 skipped due to invalid zip code format."</td>
                                        </tr>
                                    </tbody>
                                </table>
                            </div>
                        </div>
                    </div>
                </Show>

            </div>
        </div>
    }
}
