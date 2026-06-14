use leptos::prelude::*;
use leptos_router::hooks::use_params_map;

#[component]
pub fn ProductDetail() -> impl IntoView {
    let params = use_params_map();
    let product_id = move || params.with(|p| p.get("id").unwrap_or_else(|| "folio-landlords".to_string()));

    let toast = use_context::<crate::app::GlobalToast>().expect("toast context");
    
    // Form states
    let product_name = RwSignal::new(String::new());
    let product_domain = RwSignal::new(String::new());
    let product_desc = RwSignal::new(String::new());
    
    // Tab state
    let active_tab = RwSignal::new("general".to_string());
    
    // SEO Score
    let seo_score = RwSignal::new(85);

    // Initialize values based on product ID
    Effect::new(move |_| {
        let pid = product_id();
        match pid.as_str() {
            "folio-network" => {
                product_name.set("Folio Network Instance".to_string());
                product_domain.set("folio.rentals/network".to_string());
                product_desc.set("Branded rental marketplace for PMCs, community associations, and syndicators. Offers config-driven custom domain publishing.".to_string());
                seo_score.set(90);
            }
            "meridian-insights" => {
                product_name.set("Folio Meridian Insights".to_string());
                product_domain.set("folio.rentals/meridian".to_string());
                product_desc.set("Market intelligence and carrier performance dashboards. Integrates G-27 rating telemetry with USDC invoicing hooks.".to_string());
                seo_score.set(70);
            }
            "meridian-fleet" => {
                product_name.set("Meridian Fleet Manager".to_string());
                product_domain.set("waitlist".to_string());
                product_desc.set("Asset operations & driver scorecards. FMCSA carrier safety calibration and DOT certification monitoring modules.".to_string());
                seo_score.set(45);
            }
            _ => {
                product_name.set("Folio for Landlords".to_string());
                product_domain.set("folio.rentals/landlord".to_string());
                product_desc.set("Property management SaaS for independent landlords and small PMCs. Includes automated onboarding and 3 flexible pricing tiers.".to_string());
                seo_score.set(85);
            }
        }
    });

    let handle_save = move |_| {
        toast.show_toast("Success", "Product marketing details saved successfully.", "success");
    };

    let handle_publish = move |_| {
        toast.show_toast("Published", "Edge cache invalidated. Marketing storefront updated.", "success");
    };

    view! {
        <div class="space-y-6">
            // ── Breadcrumb ──
            <nav class="flex items-center gap-2 text-on-surface-variant text-xs mb-2">
                <a href="/products" class="hover:text-primary transition-colors">"Products"</a>
                <span class="material-symbols-outlined text-[12px]">"chevron_right"</span>
                <span class="text-primary/70">{move || product_name.get()}</span>
            </nav>

            // ── Product Header / Toolbar ──
            <div class="flex flex-col md:flex-row justify-between items-start md:items-center gap-4 bg-surface-container-low border border-outline-variant/20 p-6 rounded-2xl shadow-sm">
                <div class="flex items-center gap-3 w-full md:w-auto">
                    <span class="w-8 h-8 rounded-lg bg-primary/20 text-primary flex items-center justify-center font-black text-sm select-none">
                        "P"
                    </span>
                    <input 
                        type="text" 
                        class="bg-transparent border-b border-transparent focus:border-primary text-xl font-extrabold text-on-surface tracking-tight outline-none focus:outline-none flex-1 max-w-sm transition-all"
                        prop:value=product_name
                        on:input=move |ev| product_name.set(event_target_value(&ev))
                    />
                    <span class="inline-flex items-center px-2 py-0.5 rounded text-[9px] font-bold bg-emerald-500/10 text-emerald-400 border border-emerald-500/20 uppercase tracking-wider">
                        "Active"
                    </span>
                </div>
                <div class="flex items-center gap-3">
                    <button 
                        class="btn-ghost px-4 py-2 rounded-lg text-sm font-semibold border border-outline-variant/30 hover:bg-surface-bright/20 hover:text-on-surface transition-all active:scale-95"
                        on:click=handle_save
                    >
                        "Save Changes"
                    </button>
                    <button 
                        class="btn-primary-gradient px-4 py-2 rounded-lg text-sm font-semibold text-on-primary-container shadow-md shadow-primary/10 hover:opacity-90 active:scale-95 transition-all"
                        on:click=handle_publish
                    >
                        "Publish Live"
                    </button>
                </div>
            </div>

            // ── Tab Navigation ──
            <div class="flex border-b border-outline-variant/20 overflow-x-auto shrink-0 select-none">
                {
                    let tab_btn = move |id: &str, label: &str| {
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
                        {tab_btn("general", "General Info")}
                        {tab_btn("pricing", "Pricing & Plans")}
                        {tab_btn("waitlist", "Waitlist Leads")}
                        {tab_btn("seo", "SEO & Metadata")}
                    }
                }
            </div>

            // ── TAB CONTENT: General Info ──
            <Show when=move || active_tab.get() == "general">
                <div class="grid grid-cols-1 lg:grid-cols-3 gap-6">
                    <div class="lg:col-span-2 space-y-6">
                        <div class="bg-surface-container-low border border-outline-variant/20 rounded-xl p-6 shadow-sm space-y-4">
                            <h3 class="text-sm font-bold uppercase tracking-wider text-on-surface-variant">"Marketing Profile"</h3>
                            
                            <div class="space-y-1">
                                <label class="text-xs font-semibold text-on-surface-variant">"Product Subdomain / Slug"</label>
                                <input 
                                    type="text" 
                                    class="w-full bg-surface-container border border-outline-variant/30 text-on-surface text-sm rounded-lg px-3 py-2.5 focus:ring-1 focus:ring-primary focus:border-primary transition-all"
                                    prop:value=product_domain
                                    on:input=move |ev| product_domain.set(event_target_value(&ev))
                                />
                                <p class="text-[10px] text-on-surface-variant/50">"The canonical vanity URL that maps inbound visitors directly to this product storefront"</p>
                            </div>

                            <div class="space-y-1 mt-4">
                                <label class="text-xs font-semibold text-on-surface-variant">"Vanity Description"</label>
                                <textarea 
                                    class="w-full bg-surface-container border border-outline-variant/30 text-on-surface text-sm rounded-lg px-3 py-2.5 focus:ring-1 focus:ring-primary focus:border-primary transition-all h-28 resize-none"
                                    on:input=move |ev| product_desc.set(event_target_value(&ev))
                                >
                                    {product_desc.get_untracked()}
                                </textarea>
                                <p class="text-[10px] text-on-surface-variant/50">"Public-facing tagline shown on cards, directories, and default SEO descriptions"</p>
                            </div>
                        </div>
                    </div>

                    <div class="space-y-6">
                        <div class="bg-surface-container-low border border-outline-variant/20 rounded-xl p-6 shadow-sm space-y-4">
                            <h3 class="text-sm font-bold uppercase tracking-wider text-on-surface-variant">"Deployment Telemetry"</h3>
                            <div class="divide-y divide-outline-variant/10 text-xs">
                                <div class="flex justify-between items-center py-2.5">
                                    <span class="text-on-surface-variant">"Environment"</span>
                                    <span class="font-bold font-mono">"production"</span>
                                </div>
                                <div class="flex justify-between items-center py-2.5">
                                    <span class="text-on-surface-variant">"Edge Cluster"</span>
                                    <span class="font-mono text-indigo-400">"us-east-cloudflare"</span>
                                </div>
                                <div class="flex justify-between items-center py-2.5">
                                    <span class="text-on-surface-variant">"Cache Invalidation"</span>
                                    <span class="text-emerald-400 font-semibold">"Passed / Active"</span>
                                </div>
                                <div class="flex justify-between items-center py-2.5">
                                    <span class="text-on-surface-variant">"Last published"</span>
                                    <span class="text-on-surface-variant/80">"14 hours ago"</span>
                                </div>
                            </div>
                        </div>
                    </div>
                </div>
            </Show>

            // ── TAB CONTENT: Pricing ──
            <Show when=move || active_tab.get() == "pricing">
                <div class="space-y-4">
                    <div class="bg-surface-container-low border border-outline-variant/20 rounded-xl p-6 shadow-sm">
                        <div class="flex justify-between items-center mb-6">
                            <h3 class="text-sm font-bold uppercase tracking-wider text-on-surface-variant">"Pricing Plans & Feature Matrix"</h3>
                            <button class="btn-ghost px-3 py-1.5 rounded-lg border border-outline-variant/30 hover:bg-surface-bright/20 text-xs font-bold uppercase tracking-wider">"+ Add Tier"</button>
                        </div>

                        <div class="grid grid-cols-1 md:grid-cols-3 gap-4">
                            <div class="bg-surface-container p-5 rounded-xl border border-outline-variant/20 flex flex-col justify-between">
                                <div>
                                    <div class="flex items-center justify-between mb-2">
                                        <h4 class="font-bold text-on-surface">"Basic Plan"</h4>
                                        <span class="px-2 py-0.5 rounded text-[9px] font-bold bg-indigo-500/10 text-indigo-400 border border-indigo-500/20">"$400/mo"</span>
                                    </div>
                                    <p class="text-xs text-on-surface-variant/70 mb-4 leading-relaxed">"Entry level hosting context for self-managed small portfolio landlords."</p>
                                    <ul class="text-xs text-on-surface-variant space-y-2">
                                        <li class="flex items-center gap-2">
                                            <span class="w-1.5 h-1.5 rounded-full bg-emerald-400"></span>
                                            "Up to 25 properties"
                                        </li>
                                        <li class="flex items-center gap-2">
                                            <span class="w-1.5 h-1.5 rounded-full bg-emerald-400"></span>
                                            "Standard support"
                                        </li>
                                    </ul>
                                </div>
                                <button class="btn-ghost w-full mt-6 text-xs justify-center py-2 border border-outline-variant/30 rounded-md">"Edit plan"</button>
                            </div>

                            <div class="bg-surface-container p-5 rounded-xl border border-primary/20 flex flex-col justify-between relative">
                                <div class="absolute -top-2.5 right-4 bg-primary text-on-primary-container px-2 py-0.5 rounded text-[8px] font-bold uppercase tracking-widest">"Popular"</div>
                                <div>
                                    <div class="flex items-center justify-between mb-2">
                                        <h4 class="font-bold text-on-surface">"Professional Plan"</h4>
                                        <span class="px-2 py-0.5 rounded text-[9px] font-bold bg-emerald-500/10 text-emerald-400 border border-emerald-500/20">"$900/mo"</span>
                                    </div>
                                    <p class="text-xs text-on-surface-variant/70 mb-4 leading-relaxed">"Automated TOT reporting and Stripe/Zelle payment routing."</p>
                                    <ul class="text-xs text-on-surface-variant space-y-2">
                                        <li class="flex items-center gap-2">
                                            <span class="w-1.5 h-1.5 rounded-full bg-emerald-400"></span>
                                            "Unlimited properties"
                                        </li>
                                        <li class="flex items-center gap-2">
                                            <span class="w-1.5 h-1.5 rounded-full bg-emerald-400"></span>
                                            "Priority SLA support"
                                        </li>
                                    </ul>
                                </div>
                                <button class="btn-primary w-full mt-6 text-xs justify-center py-2 rounded-md">"Edit plan"</button>
                            </div>

                            <div class="bg-surface-container p-5 rounded-xl border border-outline-variant/20 flex flex-col justify-between">
                                <div>
                                    <div class="flex items-center justify-between mb-2">
                                        <h4 class="font-bold text-on-surface">"Enterprise Plan"</h4>
                                        <span class="px-2 py-0.5 rounded text-[9px] font-bold bg-indigo-500/10 text-indigo-400 border border-indigo-500/20">"Custom"</span>
                                    </div>
                                    <p class="text-xs text-on-surface-variant/70 mb-4 leading-relaxed">"Custom SLA routing, FMCSA compliance audits, and dedicated storage reclamation."</p>
                                    <ul class="text-xs text-on-surface-variant space-y-2">
                                        <li class="flex items-center gap-2">
                                            <span class="w-1.5 h-1.5 rounded-full bg-emerald-400"></span>
                                            "Custom cloud boundaries"
                                        </li>
                                        <li class="flex items-center gap-2">
                                            <span class="w-1.5 h-1.5 rounded-full bg-emerald-400"></span>
                                            "Dedicated technical team"
                                        </li>
                                    </ul>
                                </div>
                                <button class="btn-ghost w-full mt-6 text-xs justify-center py-2 border border-outline-variant/30 rounded-md">"Edit plan"</button>
                            </div>
                        </div>
                    </div>
                </div>
            </Show>

            // ── TAB CONTENT: Waitlist ──
            <Show when=move || active_tab.get() == "waitlist">
                <div class="bg-surface-container-low border border-outline-variant/20 rounded-xl p-6 shadow-sm">
                    <h3 class="text-sm font-bold uppercase tracking-wider text-on-surface-variant mb-6">"Waitlist Leads"</h3>
                    
                    <div class="overflow-x-auto border border-outline-variant/20 rounded-lg">
                        <table class="w-full text-left border-collapse">
                            <thead>
                                <tr class="bg-surface-container-high/40 border-b border-outline-variant/20 text-xs tracking-wider uppercase text-on-surface-variant">
                                    <th class="px-6 py-4 font-medium">"Lead"</th>
                                    <th class="px-6 py-4 font-medium">"Email"</th>
                                    <th class="px-6 py-4 font-medium">"Submitted"</th>
                                    <th class="px-6 py-4 font-medium">"Status"</th>
                                </tr>
                            </thead>
                            <tbody class="divide-y divide-outline-variant/10 text-xs text-on-surface">
                                <tr class="hover:bg-surface-bright/5 transition-colors">
                                    <td class="px-6 py-4 font-semibold">"Jamie Delaney"</td>
                                    <td class="px-6 py-4 font-mono">"jamie@nexusproperties.com"</td>
                                    <td class="px-6 py-4">"June 12, 2026"</td>
                                    <td class="px-6 py-4">
                                        <span class="inline-flex items-center px-2 py-0.5 rounded text-[9px] font-bold bg-primary/10 text-primary border border-primary/20 uppercase tracking-wider">"New"</span>
                                    </td>
                                </tr>
                                <tr class="hover:bg-surface-bright/5 transition-colors">
                                    <td class="px-6 py-4 font-semibold">"Renato Santos"</td>
                                    <td class="px-6 py-4 font-mono">"santos.renato@saopaulo.br"</td>
                                    <td class="px-6 py-4">"June 10, 2026"</td>
                                    <td class="px-6 py-4">
                                        <span class="inline-flex items-center px-2 py-0.5 rounded text-[9px] font-bold bg-emerald-500/10 text-emerald-400 border border-emerald-500/20 uppercase tracking-wider">"Converted"</span>
                                    </td>
                                </tr>
                                <tr class="hover:bg-surface-bright/5 transition-colors">
                                    <td class="px-6 py-4 font-semibold">"Sarah Jenkins"</td>
                                    <td class="px-6 py-4 font-mono">"sarah@oakwoodpm.com"</td>
                                    <td class="px-6 py-4">"June 08, 2026"</td>
                                    <td class="px-6 py-4">
                                        <span class="inline-flex items-center px-2 py-0.5 rounded text-[9px] font-bold bg-muted text-on-surface-variant/60 border border-outline-variant/30 uppercase tracking-wider">"Cold"</span>
                                    </td>
                                </tr>
                            </tbody>
                        </table>
                    </div>
                </div>
            </Show>

            // ── TAB CONTENT: SEO ──
            <Show when=move || active_tab.get() == "seo">
                <div class="grid grid-cols-1 lg:grid-cols-3 gap-6">
                    <div class="lg:col-span-2 space-y-6">
                        <div class="bg-surface-container-low border border-outline-variant/20 rounded-xl p-6 shadow-sm space-y-6">
                            <h3 class="text-sm font-bold uppercase tracking-wider text-on-surface-variant">"SEO Checklist & Target Attributes"</h3>
                            
                            <div class="divide-y divide-outline-variant/10">
                                <div class="flex items-start gap-3 py-4 first:pt-0">
                                    <span class="material-symbols-outlined text-emerald-400 mt-0.5">"check_circle"</span>
                                    <div>
                                        <h4 class="text-xs font-bold text-on-surface">"Page Title Optimization"</h4>
                                        <p class="text-[10px] text-on-surface-variant/70 mt-1">"Title is under 60 characters and targets primary keywords."</p>
                                    </div>
                                </div>
                                <div class="flex items-start gap-3 py-4">
                                    <span class="material-symbols-outlined text-emerald-400 mt-0.5">"check_circle"</span>
                                    <div>
                                        <h4 class="text-xs font-bold text-on-surface">"Meta Description Configured"</h4>
                                        <p class="text-[10px] text-on-surface-variant/70 mt-1">"Comprehensive summary description has been bound to header metadata."</p>
                                    </div>
                                </div>
                                <div class="flex items-start gap-3 py-4 last:pb-0">
                                    <span class="material-symbols-outlined text-amber-400 mt-0.5">"warning"</span>
                                    <div>
                                        <h4 class="text-xs font-bold text-on-surface">"OpenGraph Image Missing"</h4>
                                        <p class="text-[10px] text-on-surface-variant/70 mt-1">"A generic fallback image is used. We recommend importing a custom high-fidelity banner asset."</p>
                                    </div>
                                </div>
                            </div>
                        </div>
                    </div>

                    <div class="space-y-6">
                        <div class="bg-surface-container-low border border-outline-variant/20 rounded-xl p-6 shadow-sm flex flex-col items-center justify-center text-center">
                            <h3 class="text-sm font-bold uppercase tracking-wider text-on-surface-variant mb-4 self-start">"SEO Score"</h3>
                            
                            // Progress Ring
                            <div class="relative w-36 h-36 flex items-center justify-center">
                                <svg class="w-full h-full transform -rotate-90">
                                    <circle cx="72" cy="72" r="60" class="stroke-outline-variant/20 fill-none stroke-[8]" />
                                    <circle 
                                        cx="72" 
                                        cy="72" 
                                        r="60" 
                                        class="stroke-primary fill-none stroke-[8] stroke-dasharray-[377] transition-all duration-500" 
                                        style=move || format!("stroke-dashoffset: {};", 377 - (377 * seo_score.get()) / 100)
                                    />
                                </svg>
                                <div class="absolute flex flex-col items-center">
                                    <span class="text-3xl font-extrabold text-on-surface font-mono">{move || seo_score.get()}</span>
                                    <span class="text-[9px] font-bold text-on-surface-variant/50 uppercase tracking-wider">"/ 100"</span>
                                </div>
                            </div>

                            <p class="text-xs text-on-surface-variant/80 mt-6 max-w-[200px]">
                                {move || if seo_score.get() >= 80 {
                                    "Excellent index scores. Pages are fully optimized."
                                } else {
                                    "Needs attention. Key meta fields remain unoptimized."
                                }}
                            </p>
                        </div>
                    </div>
                </div>
            </Show>
        </div>
    }
}
