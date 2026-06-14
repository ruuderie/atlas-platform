use leptos::prelude::*;

#[component]
pub fn PlatformProducts() -> impl IntoView {
    let toast = use_context::<crate::app::GlobalToast>().expect("toast context");

    let handle_import = move |_| {
        toast.show_toast("Import", "Storefront page import template initialized.", "info");
    };

    view! {
        <div class="space-y-6">
            // ── Page Header ──
            <div class="flex flex-col md:flex-row justify-between items-start md:items-center gap-4 bg-surface-container-low border border-outline-variant/20 p-6 rounded-2xl shadow-sm">
                <div>
                    <h1 class="text-2xl font-extrabold tracking-tight text-on-surface">"Platform Products"</h1>
                    <p class="text-xs text-on-surface-variant mt-1">"Marketing pages, pricing tiers, and public storefront catalog management"</p>
                </div>
                <div class="flex items-center gap-3">
                    <button 
                        class="btn-ghost px-4 py-2 rounded-lg text-sm font-semibold border border-outline-variant/30 hover:bg-surface-bright/20 hover:text-on-surface transition-all active:scale-95"
                        on:click=handle_import
                    >
                        "Import Page"
                    </button>
                    <a href="/products/new">
                        <button class="btn-primary-gradient px-4 py-2 rounded-lg text-sm font-semibold text-on-primary-container shadow-md shadow-primary/10 hover:opacity-90 active:scale-95 transition-all">
                            "+ New Product"
                        </button>
                    </a>
                </div>
            </div>

            // ── KPI Ribbon ──
            <div class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-4">
                <div class="bg-surface-container-low border border-outline-variant/20 rounded-xl p-5 shadow-sm hover:border-outline-variant/40 transition-colors flex flex-col justify-between min-h-[100px]">
                    <span class="text-[10px] font-bold text-on-surface-variant/70 uppercase tracking-widest">"Total Products"</span>
                    <span class="text-3xl font-extrabold text-on-surface tracking-tight mt-2">"4"</span>
                    <span class="text-[10px] text-on-surface-variant/50 mt-1">"Across all engine tiers"</span>
                </div>
                <div class="bg-surface-container-low border border-outline-variant/20 rounded-xl p-5 shadow-sm hover:border-outline-variant/40 transition-colors flex flex-col justify-between min-h-[100px]">
                    <span class="text-[10px] font-bold text-on-surface-variant/70 uppercase tracking-widest">"Live Pages"</span>
                    <span class="text-3xl font-extrabold text-emerald-400 tracking-tight mt-2">"3"</span>
                    <span class="text-[10px] text-on-surface-variant/50 mt-1">"Active Edge deployments"</span>
                </div>
                <div class="bg-surface-container-low border border-outline-variant/20 rounded-xl p-5 shadow-sm hover:border-outline-variant/40 transition-colors flex flex-col justify-between min-h-[100px]">
                    <span class="text-[10px] font-bold text-on-surface-variant/70 uppercase tracking-widest">"Total Leads (30d)"</span>
                    <span class="text-3xl font-extrabold text-on-surface tracking-tight mt-2">"218"</span>
                    <span class="text-[10px] text-emerald-400 font-semibold mt-1 flex items-center gap-1">
                        "↑ 34%" <span class="text-on-surface-variant/40 font-normal">"vs prev period"</span>
                    </span>
                </div>
                <div class="bg-surface-container-low border border-outline-variant/20 rounded-xl p-5 shadow-sm hover:border-outline-variant/40 transition-colors flex flex-col justify-between min-h-[100px]">
                    <span class="text-[10px] font-bold text-on-surface-variant/70 uppercase tracking-widest">"Conversion Rate"</span>
                    <span class="text-3xl font-extrabold text-on-surface tracking-tight mt-2">"4.1%"</span>
                    <span class="text-[10px] text-emerald-400 font-semibold mt-1 flex items-center gap-1">
                        "↑ 0.7pp" <span class="text-on-surface-variant/40 font-normal">"optimizations active"</span>
                    </span>
                </div>
            </div>

            // ── Products Grid ──
            <div class="grid grid-cols-1 md:grid-cols-2 gap-6">
                // Card 1
                <a href="/products/folio-landlords" class="block group text-decoration-none">
                    <div class="bg-surface-container-low border border-outline-variant/20 hover:border-primary/40 rounded-2xl p-6 shadow-sm hover:shadow-lg transition-all duration-200 hover:-translate-y-1 relative overflow-hidden flex flex-col justify-between min-h-[220px]">
                        <div class="absolute top-0 left-0 right-0 h-1.5 bg-indigo-500"></div>
                        <div>
                            <div class="flex items-center justify-between mb-3">
                                <span class="inline-flex items-center px-2 py-0.5 rounded text-[9px] font-bold bg-emerald-500/10 text-emerald-400 border border-emerald-500/20 uppercase tracking-wider">"Live"</span>
                                <span class="text-[10px] text-on-surface-variant/50 font-mono">"folio.rentals/landlord"</span>
                            </div>
                            <h3 class="text-lg font-bold text-on-surface group-hover:text-primary transition-colors">"Folio for Landlords"</h3>
                            <p class="text-xs text-on-surface-variant/70 mt-2 leading-relaxed">"Property management SaaS for independent landlords and small PMCs. Includes automated onboarding and 3 flexible pricing tiers."</p>
                        </div>
                        <div class="flex items-center gap-6 mt-6 border-t border-outline-variant/10 pt-4 text-xs text-on-surface-variant">
                            <span>"Leads / 30d: " <strong class="text-on-surface font-bold">"142"</strong></span>
                            <span>"Variants: " <strong class="text-on-surface font-bold">"3"</strong></span>
                        </div>
                    </div>
                </a>

                // Card 2
                <a href="/products/folio-network" class="block group text-decoration-none">
                    <div class="bg-surface-container-low border border-outline-variant/20 hover:border-emerald-500/40 rounded-2xl p-6 shadow-sm hover:shadow-lg transition-all duration-200 hover:-translate-y-1 relative overflow-hidden flex flex-col justify-between min-h-[220px]">
                        <div class="absolute top-0 left-0 right-0 h-1.5 bg-emerald-500"></div>
                        <div>
                            <div class="flex items-center justify-between mb-3">
                                <span class="inline-flex items-center px-2 py-0.5 rounded text-[9px] font-bold bg-emerald-500/10 text-emerald-400 border border-emerald-500/20 uppercase tracking-wider">"Live"</span>
                                <span class="text-[10px] text-on-surface-variant/50 font-mono">"folio.rentals/network"</span>
                            </div>
                            <h3 class="text-lg font-bold text-on-surface group-hover:text-emerald-400 transition-colors">"Folio Network Instance"</h3>
                            <p class="text-xs text-on-surface-variant/70 mt-2 leading-relaxed">"Branded rental marketplace for PMCs, community associations, and syndicators. Offers config-driven custom domain publishing."</p>
                        </div>
                        <div class="flex items-center gap-6 mt-6 border-t border-outline-variant/10 pt-4 text-xs text-on-surface-variant">
                            <span>"Leads / 30d: " <strong class="text-on-surface font-bold">"67"</strong></span>
                            <span>"Variants: " <strong class="text-on-surface font-bold">"2"</strong></span>
                        </div>
                    </div>
                </a>

                // Card 3
                <a href="/products/meridian-insights" class="block group text-decoration-none">
                    <div class="bg-surface-container-low border border-outline-variant/20 hover:border-amber-500/40 rounded-2xl p-6 shadow-sm hover:shadow-lg transition-all duration-200 hover:-translate-y-1 relative overflow-hidden flex flex-col justify-between min-h-[220px]">
                        <div class="absolute top-0 left-0 right-0 h-1.5 bg-amber-500"></div>
                        <div>
                            <div class="flex items-center justify-between mb-3">
                                <span class="inline-flex items-center px-2 py-0.5 rounded text-[9px] font-bold bg-emerald-500/10 text-emerald-400 border border-emerald-500/20 uppercase tracking-wider">"Live"</span>
                                <span class="text-[10px] text-on-surface-variant/50 font-mono">"folio.rentals/meridian"</span>
                            </div>
                            <h3 class="text-lg font-bold text-on-surface group-hover:text-amber-400 transition-colors">"Folio Meridian Insights"</h3>
                            <p class="text-xs text-on-surface-variant/70 mt-2 leading-relaxed">"Market intelligence and carrier performance dashboards. Integrates G-27 rating telemetry with USDC invoicing hooks."</p>
                        </div>
                        <div class="flex items-center gap-6 mt-6 border-t border-outline-variant/10 pt-4 text-xs text-on-surface-variant">
                            <span>"Leads / 30d: " <strong class="text-on-surface font-bold">"9"</strong></span>
                            <span>"Variants: " <strong class="text-on-surface font-bold">"1"</strong></span>
                        </div>
                    </div>
                </a>

                // Card 4
                <a href="/products/meridian-fleet" class="block group text-decoration-none">
                    <div class="bg-surface-container-low border border-outline-variant/20 hover:border-outline-variant/50 rounded-2xl p-6 shadow-sm hover:shadow-lg transition-all duration-200 hover:-translate-y-1 relative overflow-hidden flex flex-col justify-between min-h-[220px]">
                        <div class="absolute top-0 left-0 right-0 h-1.5 bg-gray-500"></div>
                        <div>
                            <div class="flex items-center justify-between mb-3">
                                <span class="inline-flex items-center px-2 py-0.5 rounded text-[9px] font-bold bg-muted text-on-surface-variant/60 border border-outline-variant/30 uppercase tracking-wider">"Draft"</span>
                                <span class="text-[10px] text-on-surface-variant/40 font-mono">"waitlist"</span>
                            </div>
                            <h3 class="text-lg font-bold text-on-surface group-hover:text-on-surface transition-colors">"Meridian Fleet Manager"</h3>
                            <p class="text-xs text-on-surface-variant/70 mt-2 leading-relaxed">"Asset operations & driver scorecards. FMCSA carrier safety calibration and DOT certification monitoring modules."</p>
                        </div>
                        <div class="flex items-center gap-6 mt-6 border-t border-outline-variant/10 pt-4 text-xs text-on-surface-variant">
                            <span>"Leads / 30d: " <strong class="text-on-surface font-bold">"0"</strong></span>
                            <span>"Variants: " <strong class="text-on-surface font-bold">"0"</strong></span>
                        </div>
                    </div>
                </a>
            </div>
        </div>
    }
}
