use leptos::prelude::*;

#[component]
pub fn NetworkCreate() -> impl IntoView {
    let toast = use_context::<crate::app::GlobalToast>().expect("toast context");
    
    // Wizard step state: 1 to 3
    let current_step = RwSignal::new(1);
    
    // Inputs state
    let name = RwSignal::new("".to_string());
    let slug = RwSignal::new("".to_string());
    let domain = RwSignal::new("".to_string());
    
    // Capabilities state
    let cap_ltr = RwSignal::new(true);
    let cap_str = RwSignal::new(false);
    let cap_vendor = RwSignal::new(true);
    
    // Markets state
    let market_chi = RwSignal::new(true);
    let market_mia = RwSignal::new(false);
    let market_rio = RwSignal::new(false);
    
    // Currency & templates
    let currency = RwSignal::new("USD".to_string());
    let template = RwSignal::new("standard".to_string());
    
    // Seeding state
    let seed_categories = RwSignal::new(true);
    let seed_templates = RwSignal::new(true);
    let seed_demo = RwSignal::new(false);

    let move_step = move |dir: i32| {
        let current = current_step.get();
        if dir == 1 {
            if current == 1 {
                if name.get().trim().is_empty() || slug.get().trim().is_empty() {
                    toast.show_toast("Validation Error", "Please fill out the Network Name and Slug Path.", "error");
                    return;
                }
            }
            if current == 3 {
                // Network instance provisioning is not yet wired to an atomic API.
                // The old multi-step flow was deprecated (see api/networks.rs).
                // Operators should use the Tenant provisioning flow (/apps/new) instead.
                toast.show_toast(
                    "Provisioning Unavailable",
                    "Network instance provisioning API is pending. Use the Tenant provisioning flow under Tenants > New Application.",
                    "error"
                );
                return;
            }
            current_step.set(current + 1);
        } else {
            if current == 1 {
                // Redirect back
                let window = web_sys::window().unwrap();
                let _ = window.location().set_href("/network");
                return;
            }
            current_step.set(current - 1);
        }
    };

    view! {
        <div class="main-canvas">
            // ── Page Header ──
            <div class="page-header">
                <div>
                    <nav style="display:flex;align-items:center;gap:6px;font-size:11px;color:var(--text-muted);margin-bottom:4px;">
                        <a href="/" style="color:inherit;text-decoration:none;hover:color:var(--cobalt)">"Dashboard"</a>
                        <span>"›"</span>
                        <a href="/network" style="color:inherit;text-decoration:none">"Network Instances"</a>
                        <span>"›"</span>
                        <span style="color:var(--cobalt)">"New Instance"</span>
                    </nav>
                    <h1 class="page-title">"Provision New Network Instance"</h1>
                    <p class="page-subtitle">"Spin up a new public-facing listing catalog directory and directory server"</p>
                </div>
            </div>

            // ── Steps Progress Indicator ──
            <div class="bg-surface-container border border-outline-variant/20 p-5 rounded-xl flex items-center justify-between shadow-xs">
                <div class=move || {
                    let active = current_step.get() == 1;
                    let completed = current_step.get() > 1;
                    format!("flex items-center gap-2.5 {}", 
                        if active { "text-primary font-bold" } 
                        else if completed { "text-emerald-400 font-bold" } 
                        else { "text-on-surface-variant/60 font-medium" }
                    )
                }>
                    <span class=move || {
                        let active = current_step.get() == 1;
                        let completed = current_step.get() > 1;
                        format!("w-6 h-6 rounded-full flex items-center justify-center font-bold text-xs border {}",
                            if active { "bg-primary/10 border-primary text-primary" }
                            else if completed { "bg-emerald-500/10 border-emerald-500 text-emerald-400" }
                            else { "bg-surface-container-highest border-outline-variant/30 text-on-surface-variant/60" }
                        )
                    }>
                        {move || if current_step.get() > 1 { "✓" } else { "1" }}
                    </span>
                    <span class="text-xs">"Identity & Host"</span>
                </div>
                <div class=move || format!("flex-1 h-px mx-4 {}", if current_step.get() > 1 { "bg-primary" } else { "bg-outline-variant/20" })></div>
                
                <div class=move || {
                    let active = current_step.get() == 2;
                    let completed = current_step.get() > 2;
                    format!("flex items-center gap-2.5 {}", 
                        if active { "text-primary font-bold" } 
                        else if completed { "text-emerald-400 font-bold" } 
                        else { "text-on-surface-variant/60 font-medium" }
                    )
                }>
                    <span class=move || {
                        let active = current_step.get() == 2;
                        let completed = current_step.get() > 2;
                        format!("w-6 h-6 rounded-full flex items-center justify-center font-bold text-xs border {}",
                            if active { "bg-primary/10 border-primary text-primary" }
                            else if completed { "bg-emerald-500/10 border-emerald-500 text-emerald-400" }
                            else { "bg-surface-container-highest border-outline-variant/30 text-on-surface-variant/60" }
                        )
                    }>
                        {move || if current_step.get() > 2 { "✓" } else { "2" }}
                    </span>
                    <span class="text-xs">"Capabilities"</span>
                </div>
                <div class=move || format!("flex-1 h-px mx-4 {}", if current_step.get() > 2 { "bg-primary" } else { "bg-outline-variant/20" })></div>

                <div class=move || {
                    let active = current_step.get() == 3;
                    format!("flex items-center gap-2.5 {}", 
                        if active { "text-primary font-bold" } 
                        else { "text-on-surface-variant/60 font-medium" }
                    )
                }>
                    <span class=move || {
                        let active = current_step.get() == 3;
                        format!("w-6 h-6 rounded-full flex items-center justify-center font-bold text-xs border {}",
                            if active { "bg-primary/10 border-primary text-primary" }
                            else { "bg-surface-container-highest border-outline-variant/30 text-on-surface-variant/60" }
                        )
                    }>
                        "3"
                    </span>
                    <span class="text-xs">"Templates & Seeding"</span>
                </div>
            </div>

            // ── Wizard Card Container ──
            <div class="bg-surface-container-low border border-outline-variant/20 rounded-2xl shadow-sm flex flex-col justify-between min-h-[380px] overflow-hidden">
                <div class="p-8 flex-1">
                    // Step 1: Identity & Host
                    <Show when=move || current_step.get() == 1>
                        <div class="space-y-6">
                            <div class="space-y-1.5">
                                <label class="text-xs font-bold text-on-surface-variant uppercase tracking-wider">"Network Instance Name"</label>
                                <input 
                                    type="text" 
                                    class="w-full bg-[#05183c] border border-outline-variant/30 text-on-surface text-sm rounded-lg px-3 py-2.5 focus:ring-1 focus:ring-primary focus:border-primary outline-none transition-all placeholder:text-[#91aaeb]/30"
                                    placeholder="e.g. London Luxury Stays"
                                    prop:value=name
                                    on:input=move |ev| name.set(event_target_value(&ev))
                                />
                                <p class="text-[10px] text-on-surface-variant/60">"Used for branding public headers, SEO meta titles, and user emails."</p>
                            </div>
                            <div class="grid grid-cols-1 md:grid-cols-2 gap-6">
                                <div class="space-y-1.5">
                                    <label class="text-xs font-bold text-on-surface-variant uppercase tracking-wider">"URL Slug Path"</label>
                                    <input 
                                        type="text" 
                                        class="w-full bg-[#05183c] border border-outline-variant/30 text-on-surface text-sm rounded-lg px-3 py-2.5 focus:ring-1 focus:ring-primary focus:border-primary outline-none transition-all placeholder:text-[#91aaeb]/30"
                                        placeholder="e.g. london-luxury"
                                        prop:value=slug
                                        on:input=move |ev| slug.set(event_target_value(&ev))
                                    />
                                    <p class="text-[10px] text-on-surface-variant/60">"System routing path: " <code class="font-mono text-primary">{move || format!("{}.atlas-platform.com", if slug.get().is_empty() { "slug".to_string() } else { slug.get() })}</code></p>
                                </div>
                                <div class="space-y-1.5">
                                    <label class="text-xs font-bold text-on-surface-variant uppercase tracking-wider">"Custom Domain (Optional)"</label>
                                    <input 
                                        type="text" 
                                        class="w-full bg-[#05183c] border border-outline-variant/30 text-on-surface text-sm rounded-lg px-3 py-2.5 focus:ring-1 focus:ring-primary focus:border-primary outline-none transition-all placeholder:text-[#91aaeb]/30"
                                        placeholder="e.g. directory.londonluxury.com"
                                        prop:value=domain
                                        on:input=move |ev| domain.set(event_target_value(&ev))
                                    />
                                    <p class="text-[10px] text-on-surface-variant/60">"Configure CNAME mapping after provisioning is complete."</p>
                                </div>
                            </div>
                        </div>
                    </Show>

                    // Step 2: Capabilities & Markets
                    <Show when=move || current_step.get() == 2>
                        <div class="space-y-6">
                            <div class="space-y-3">
                                <label class="text-xs font-bold text-on-surface-variant uppercase tracking-wider">"Supported Capabilities"</label>
                                
                                <div class="flex items-center justify-between bg-surface-container p-4 rounded-xl border border-outline-variant/20 hover:border-outline-variant/40 transition-colors">
                                    <div class="flex flex-col gap-0.5">
                                        <span class="text-xs font-bold text-on-surface">"Long-Term Rentals (LTR)"</span>
                                        <span class="text-[10px] text-on-surface-variant/70">"Enable landlord postings, standard leases, and applicant background checks."</span>
                                    </div>
                                    <label class="relative inline-flex items-center cursor-pointer">
                                        <input 
                                            type="checkbox" 
                                            class="sr-only peer" 
                                            prop:checked=cap_ltr
                                            on:change=move |ev| cap_ltr.set(event_target_checked(&ev))
                                        />
                                        <div class="w-9 h-5 bg-surface-container-highest peer-focus:outline-none rounded-full peer peer-checked:after:translate-x-full peer-checked:after:border-white after:content-[''] after:absolute after:top-[2px] after:left-[2px] after:bg-white after:border-gray-300 after:border after:rounded-full after:h-4 after:w-4 after:transition-all peer-checked:bg-primary"></div>
                                    </label>
                                </div>

                                <div class="flex items-center justify-between bg-surface-container p-4 rounded-xl border border-outline-variant/20 hover:border-outline-variant/40 transition-colors">
                                    <div class="flex flex-col gap-0.5">
                                        <span class="text-xs font-bold text-on-surface">"Short-Term Stays (STR)"</span>
                                        <span class="text-[10px] text-on-surface-variant/70">"Enable vacation rental listings, calendar bookings, and nights-based check-in."</span>
                                    </div>
                                    <label class="relative inline-flex items-center cursor-pointer">
                                        <input 
                                            type="checkbox" 
                                            class="sr-only peer" 
                                            prop:checked=cap_str
                                            on:change=move |ev| cap_str.set(event_target_checked(&ev))
                                        />
                                        <div class="w-9 h-5 bg-surface-container-highest peer-focus:outline-none rounded-full peer peer-checked:after:translate-x-full peer-checked:after:border-white after:content-[''] after:absolute after:top-[2px] after:left-[2px] after:bg-white after:border-gray-300 after:border after:rounded-full after:h-4 after:w-4 after:transition-all peer-checked:bg-primary"></div>
                                    </label>
                                </div>

                                <div class="flex items-center justify-between bg-surface-container p-4 rounded-xl border border-outline-variant/20 hover:border-outline-variant/40 transition-colors">
                                    <div class="flex flex-col gap-0.5">
                                        <span class="text-xs font-bold text-on-surface">"Vendor Marketplace (V)"</span>
                                        <span class="text-[10px] text-on-surface-variant/70">"Connect local service providers (plumbers, cleaning) for syndicated work orders."</span>
                                    </div>
                                    <label class="relative inline-flex items-center cursor-pointer">
                                        <input 
                                            type="checkbox" 
                                            class="sr-only peer" 
                                            prop:checked=cap_vendor
                                            on:change=move |ev| cap_vendor.set(event_target_checked(&ev))
                                        />
                                        <div class="w-9 h-5 bg-surface-container-highest peer-focus:outline-none rounded-full peer peer-checked:after:translate-x-full peer-checked:after:border-white after:content-[''] after:absolute after:top-[2px] after:left-[2px] after:bg-white after:border-gray-300 after:border after:rounded-full after:h-4 after:w-4 after:transition-all peer-checked:bg-primary"></div>
                                    </label>
                                </div>
                            </div>

                            <div class="grid grid-cols-1 md:grid-cols-2 gap-6 pt-2">
                                <div class="space-y-2">
                                    <label class="text-xs font-bold text-on-surface-variant uppercase tracking-wider">"Served Markets"</label>
                                    <div class="grid grid-cols-3 gap-2">
                                        <label class="cursor-pointer">
                                            <input 
                                                type="checkbox" 
                                                class="sr-only peer" 
                                                prop:checked=market_chi
                                                on:change=move |ev| market_chi.set(event_target_checked(&ev))
                                            />
                                            <div class="p-2.5 rounded-lg border text-center text-xs font-medium bg-surface-container border-outline-variant/20 peer-checked:bg-primary/10 peer-checked:border-primary peer-checked:text-primary transition-all">
                                                "Chicago"
                                            </div>
                                        </label>
                                        <label class="cursor-pointer">
                                            <input 
                                                type="checkbox" 
                                                class="sr-only peer" 
                                                prop:checked=market_mia
                                                on:change=move |ev| market_mia.set(event_target_checked(&ev))
                                            />
                                            <div class="p-2.5 rounded-lg border text-center text-xs font-medium bg-surface-container border-outline-variant/20 peer-checked:bg-primary/10 peer-checked:border-primary peer-checked:text-primary transition-all">
                                                "Miami"
                                            </div>
                                        </label>
                                        <label class="cursor-pointer">
                                            <input 
                                                type="checkbox" 
                                                class="sr-only peer" 
                                                prop:checked=market_rio
                                                on:change=move |ev| market_rio.set(event_target_checked(&ev))
                                            />
                                            <div class="p-2.5 rounded-lg border text-center text-xs font-medium bg-surface-container border-outline-variant/20 peer-checked:bg-primary/10 peer-checked:border-primary peer-checked:text-primary transition-all">
                                                "Rio"
                                            </div>
                                        </label>
                                    </div>
                                </div>

                                <div class="space-y-1.5">
                                    <label class="text-xs font-bold text-on-surface-variant uppercase tracking-wider">"Primary Currency"</label>
                                    <select 
                                        class="w-full bg-[#05183c] border border-outline-variant/30 text-on-surface text-sm rounded-lg px-3 py-2.5 focus:ring-1 focus:ring-primary focus:border-primary outline-none transition-all"
                                        on:change=move |ev| currency.set(event_target_value(&ev))
                                    >
                                        <option value="USD" selected=move || currency.get() == "USD">"USD ($)"</option>
                                        <option value="EUR" selected=move || currency.get() == "EUR">"EUR (€)"</option>
                                        <option value="BRL" selected=move || currency.get() == "BRL">"BRL (R$)"</option>
                                        <option value="BTC" selected=move || currency.get() == "BTC">"BTC (₿)"</option>
                                    </select>
                                </div>
                            </div>
                        </div>
                    </Show>

                    // Step 3: Templates & Seeding
                    <Show when=move || current_step.get() == 3>
                        <div class="space-y-6">
                            <div class="space-y-1.5">
                                <label class="text-xs font-bold text-on-surface-variant uppercase tracking-wider">"Layout Template"</label>
                                <select 
                                    class="w-full bg-[#05183c] border border-outline-variant/30 text-on-surface text-sm rounded-lg px-3 py-2.5 focus:ring-1 focus:ring-primary focus:border-primary outline-none transition-all"
                                    on:change=move |ev| template.set(event_target_value(&ev))
                                >
                                    <option value="standard" selected=move || template.get() == "standard">"Standard Directory (Grid + Listings Sidebar)"</option>
                                    <option value="map-centric" selected=move || template.get() == "map-centric">"Map-Centric Interface (Split View Map + Cards)"</option>
                                    <option value="marketplace-focused" selected=move || template.get() == "marketplace-focused">"Vendor Catalog (Aggregated Reviews)"</option>
                                </select>
                            </div>

                            <div class="space-y-3">
                                <label class="text-xs font-bold text-on-surface-variant uppercase tracking-wider">"Default Seeding Packages"</label>
                                
                                <div class="bg-surface-container rounded-xl border border-outline-variant/20 overflow-hidden divide-y divide-outline-variant/10 text-xs">
                                    <label class="flex items-start gap-3 p-4 cursor-pointer hover:bg-surface-container-high/40 transition-colors">
                                        <input 
                                            type="checkbox" 
                                            class="rounded border-outline-variant text-primary focus:ring-primary h-4 w-4 mt-0.5" 
                                            prop:checked=seed_categories
                                            on:change=move |ev| seed_categories.set(event_target_checked(&ev))
                                        />
                                        <div>
                                            <p class="font-bold text-on-surface">"Seed Default Categories"</p>
                                            <p class="text-[10px] text-on-surface-variant/70 mt-0.5">"Populates baseline categories (Apartment, Studio, Commercial, Maintenance)."</p>
                                        </div>
                                    </label>

                                    <label class="flex items-start gap-3 p-4 cursor-pointer hover:bg-surface-container-high/40 transition-colors">
                                        <input 
                                            type="checkbox" 
                                            class="rounded border-outline-variant text-primary focus:ring-primary h-4 w-4 mt-0.5" 
                                            prop:checked=seed_templates
                                            on:change=move |ev| seed_templates.set(event_target_checked(&ev))
                                        />
                                        <div>
                                            <p class="font-bold text-on-surface">"Seed Listing Templates"</p>
                                            <p class="text-[10px] text-on-surface-variant/70 mt-0.5">"Deploys default metadata structures and capability configurations."</p>
                                        </div>
                                    </label>

                                    <label class="flex items-start gap-3 p-4 cursor-pointer hover:bg-surface-container-high/40 transition-colors">
                                        <input 
                                            type="checkbox" 
                                            class="rounded border-outline-variant text-primary focus:ring-primary h-4 w-4 mt-0.5" 
                                            prop:checked=seed_demo
                                            on:change=move |ev| seed_demo.set(event_target_checked(&ev))
                                        />
                                        <div>
                                            <p class="font-bold text-on-surface">"Seed Mock Listings"</p>
                                            <p class="text-[10px] text-on-surface-variant/70 mt-0.5">"Provisions 5 demo landlord posts and 2 syndicated vendor records for testing."</p>
                                        </div>
                                    </label>
                                </div>
                            </div>
                        </div>
                    </Show>
                </div>

                // Wizard Footer
                <div class="px-8 py-4 bg-surface-container-high/20 border-t border-outline-variant/20 flex justify-between items-center">
                    <button 
                        class="px-4 py-2 border border-outline-variant/30 hover:bg-surface-bright/20 rounded-lg text-xs font-semibold text-on-surface-variant transition-colors"
                        on:click=move |_| move_step(-1)
                    >
                        {move || if current_step.get() == 1 { "Cancel" } else { "Back" }}
                    </button>
                    <button 
                        class="btn-primary px-4 py-2 rounded-lg text-xs font-semibold shadow-md active:scale-95 transition-all"
                        on:click=move |_| move_step(1)
                    >
                        {move || if current_step.get() == 3 { "Provision Network" } else { "Next Step" }}
                    </button>
                </div>
            </div>
        </div>
    }
}
