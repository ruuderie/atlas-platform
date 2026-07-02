use leptos::prelude::*;

#[component]
pub fn MarketingLanding() -> impl IntoView {
    view! {
        <div class="min-h-screen bg-surface-dim text-on-surface font-sans antialiased overflow-y-auto">
            // ── Public Navigation ──
            <nav class="flex items-center justify-between px-10 h-14 border-b border-white/5 bg-surface-dim/90 backdrop-blur-md sticky top-0 z-50">
                <a href="/marketing" class="flex items-center gap-2.5 text-decoration-none">
                    <div class="w-6 h-6 bg-primary rounded-md flex items-center justify-center font-black text-xs text-white">"A"</div>
                    <span class="text-sm font-bold tracking-tight text-on-surface">"Atlas Platform"</span>
                </a>
                <div class="hidden md:flex items-center gap-7 text-xs font-medium text-on-surface-variant">
                    <a href="#products" class="hover:text-primary transition-colors">"Products"</a>
                    <a href="#metrics" class="hover:text-primary transition-colors">"Why Atlas"</a>
                    <a href="#contact" class="hover:text-primary transition-colors">"Contact"</a>
                </div>
                <div class="flex items-center gap-3">
                    <a href="/login" class="btn-ghost px-3.5 py-1.5 rounded-lg text-xs font-semibold border border-outline-variant/30 hover:bg-surface-bright/20">"Sign In"</a>
                    <a href="/login" class="btn-primary-gradient px-3.5 py-1.5 rounded-lg text-xs font-semibold text-on-primary-container shadow-md shadow-primary/10 hover:opacity-90">"Request Demo →"</a>
                </div>
            </nav>

            // ── Hero Section ──
            <section class="max-w-4xl mx-auto text-center px-6 py-20 space-y-6">
                <div class="text-[10px] font-bold text-primary uppercase tracking-widest">"Enterprise Operations Software"</div>
                <h1 class="text-5xl font-black tracking-tight leading-[1.1] text-on-surface">
                    "One platform for" <br/>
                    <span class="text-primary">"property, fleet, and creator ops"</span>
                </h1>
                <p class="text-base text-on-surface-variant/80 max-w-xl mx-auto leading-relaxed">
                    "Atlas Platform gives operators a unified command center — property management, STR compliance, creator monetization, and fleet G-27 scorecards. Built on Bitcoin rails."
                </p>
                <div class="flex items-center justify-center gap-3 pt-4">
                    <a href="/login" class="btn-primary-gradient px-6 py-3 rounded-lg text-sm font-bold text-on-primary-container shadow-lg shadow-primary/15 hover:opacity-95">"Get Started →"</a>
                    <a href="/login" class="btn-ghost px-6 py-3 rounded-lg text-sm font-semibold border border-outline-variant/30 hover:bg-surface-bright/20">"Sign In"</a>
                </div>
            </section>

            // ── Metrics Strip ──
            <section id="metrics" class="bg-surface-container-low border-y border-outline-variant/20 py-10">
                <div class="max-w-6xl mx-auto grid grid-cols-2 md:grid-cols-4 gap-6 text-center px-6">
                    <div class="space-y-1">
                        <div class="text-3xl font-black text-primary">"$2.1B+"</div>
                        <div class="text-[10px] font-bold uppercase tracking-wider text-on-surface-variant/60">"GMV Processed"</div>
                    </div>
                    <div class="space-y-1">
                        <div class="text-3xl font-black text-primary">"48+"</div>
                        <div class="text-[10px] font-bold uppercase tracking-wider text-on-surface-variant/60">"Tenants Worldwide"</div>
                    </div>
                    <div class="space-y-1">
                        <div class="text-3xl font-black text-primary">"4"</div>
                        <div class="text-[10px] font-bold uppercase tracking-wider text-on-surface-variant/60">"Platform Products"</div>
                    </div>
                    <div class="space-y-1">
                        <div class="text-3xl font-black text-primary">"BTC"</div>
                        <div class="text-[10px] font-bold uppercase tracking-wider text-on-surface-variant/60">"Native Rails Support"</div>
                    </div>
                </div>
            </section>

            // ── Products Grid ──
            <section id="products" class="max-w-6xl mx-auto px-6 py-20 space-y-10">
                <div class="text-center space-y-2">
                    <div class="text-[10px] font-bold text-primary uppercase tracking-widest">"Platform Products"</div>
                    <h2 class="text-3xl font-extrabold tracking-tight">"Built for every vertical you operate"</h2>
                    <p class="text-xs text-on-surface-variant/70">"Each product is a fully tenant-isolated app instance running on Atlas generics."</p>
                </div>

                <div class="grid grid-cols-1 md:grid-cols-2 gap-6">
                    // Product 1
                    <div class="bg-surface-container-low border border-outline-variant/20 rounded-2xl p-6 shadow-sm relative overflow-hidden flex flex-col justify-between min-h-[280px]">
                        <div class="absolute top-0 left-0 right-0 h-1 bg-indigo-500"></div>
                        <div class="space-y-4">
                            <span class="text-2xl">"🏢"</span>
                            <h3 class="text-lg font-bold">"Folio"</h3>
                            <p class="text-[10px] text-on-surface-variant/50">"Property Management · STR · Long-term Leases · Brazil"</p>
                            <ul class="text-xs text-on-surface-variant space-y-1.5 pt-2">
                                <li class="flex items-center gap-2">"✓ Full PM lifecycle: leases, rent, maintenance"</li>
                                <li class="flex items-center gap-2">"✓ STR Compliance OS — TOT remittance, OTA sync"</li>
                                <li class="flex items-center gap-2">"✓ Cross-border rents: USD, BRL, BTC, Lightning"</li>
                            </ul>
                        </div>
                        <span class="inline-block self-start mt-6 text-[9px] font-bold text-primary border border-primary/20 bg-primary/5 px-2.5 py-0.5 rounded uppercase tracking-wider">"Active · from $400/mo"</span>
                    </div>

                    // Product 2
                    <div class="bg-surface-container-low border border-outline-variant/20 rounded-2xl p-6 shadow-sm relative overflow-hidden flex flex-col justify-between min-h-[280px]">
                        <div class="absolute top-0 left-0 right-0 h-1 bg-purple-500"></div>
                        <div class="space-y-4">
                            <span class="text-2xl">"⚓"</span>
                            <h3 class="text-lg font-bold">"Anchor"</h3>
                            <p class="text-[10px] text-on-surface-variant/50">"Creator OS · Blog · Portfolio · Bitcoin Monetization"</p>
                            <ul class="text-xs text-on-surface-variant space-y-1.5 pt-2">
                                <li class="flex items-center gap-2">"✓ Creator profile & campaign management"</li>
                                <li class="flex items-center gap-2">"✓ Bitcoin & Lightning native payouts"</li>
                                <li class="flex items-center gap-2">"✓ G-27 audience quality scorecards"</li>
                            </ul>
                        </div>
                        <span class="inline-block self-start mt-6 text-[9px] font-bold text-purple-400 border border-purple-500/20 bg-purple-500/5 px-2.5 py-0.5 rounded uppercase tracking-wider">"Beta · from $900/mo"</span>
                    </div>

                    // Product 3
                    <div class="bg-surface-container-low border border-outline-variant/20 rounded-2xl p-6 shadow-sm relative overflow-hidden flex flex-col justify-between min-h-[280px]">
                        <div class="absolute top-0 left-0 right-0 h-1 bg-emerald-500"></div>
                        <div class="space-y-4">
                            <span class="text-2xl">"🔗"</span>
                            <h3 class="text-lg font-bold">"Network"</h3>
                            <p class="text-[10px] text-on-surface-variant/50">"Gated Community · Membership · Events · Access Control"</p>
                            <ul class="text-xs text-on-surface-variant space-y-1.5 pt-2">
                                <li class="flex items-center gap-2">"✓ Invite-only membership management"</li>
                                <li class="flex items-center gap-2">"✓ Event hosting with G-21 event OS"</li>
                                <li class="flex items-center gap-2">"✓ Community ledger & dues processing"</li>
                            </ul>
                        </div>
                        <span class="inline-block self-start mt-6 text-[9px] font-bold text-emerald-400 border border-emerald-500/20 bg-emerald-500/5 px-2.5 py-0.5 rounded uppercase tracking-wider">"Active · from $600/mo"</span>
                    </div>

                    // Product 4
                    <div class="bg-surface-container-low border border-outline-variant/20 rounded-2xl p-6 shadow-sm relative overflow-hidden flex flex-col justify-between min-h-[280px]">
                        <div class="absolute top-0 left-0 right-0 h-1 bg-amber-500"></div>
                        <div class="space-y-4">
                            <span class="text-2xl">"🚚"</span>
                            <h3 class="text-lg font-bold">"Meridian"</h3>
                            <p class="text-[10px] text-on-surface-variant/50">"Fleet Management · Driver Scorecards · Compliance OS"</p>
                            <ul class="text-xs text-on-surface-variant space-y-1.5 pt-2">
                                <li class="flex items-center gap-2">"✓ DOT / FMCSA compliance tracking"</li>
                                <li class="flex items-center gap-2">"✓ G-27 driver safety scorecards"</li>
                                <li class="flex items-center gap-2">"✓ Fleet asset lifecycle management"</li>
                            </ul>
                        </div>
                        <span class="inline-block self-start mt-6 text-[9px] font-bold text-amber-400 border border-amber-500/20 bg-amber-500/5 px-2.5 py-0.5 rounded uppercase tracking-wider">"Pre-launch · Waitlist"</span>
                    </div>
                </div>
            </section>

            // ── CTA Banner ──
            <section id="contact" class="bg-surface-container-low border-t border-outline-variant/20 text-center py-20 space-y-6">
                <h2 class="text-3xl font-extrabold tracking-tight">"Ready to run your operation on Atlas?"</h2>
                <p class="text-xs text-on-surface-variant max-w-md mx-auto leading-relaxed">
                    "Book a 30-minute demo. We'll show you the full platform configured for your vertical."
                </p>
                <div class="pt-2">
                    <a href="/login" class="btn-primary-gradient px-6 py-3 rounded-lg text-sm font-bold text-on-primary-container shadow-lg shadow-primary/15 hover:opacity-95">"Book Demo →"</a>
                </div>
            </section>

            // ── Footer ──
            <footer class="flex justify-between items-center px-10 py-6 border-t border-outline-variant/20 text-[10px] text-on-surface-variant/60 bg-surface-dim">
                <span>"© 2026 Atlas Platform · All rights reserved"</span>
                <div class="flex gap-4">
                    <a href="#" class="hover:text-primary transition-colors">"Privacy"</a>
                    <a href="#" class="hover:text-primary transition-colors">"Terms"</a>
                    <a href="/login" class="hover:text-primary transition-colors">"Admin Login"</a>
                </div>
            </footer>
        </div>
    }
}
