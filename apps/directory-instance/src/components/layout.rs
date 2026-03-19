use leptos::prelude::*;
use crate::app::DirectoryConfig;

#[component]
pub fn GlobalHeader() -> impl IntoView {
    let config = use_context::<DirectoryConfig>().expect("DirectoryConfig must be provided");
    
    view! {
        <header class="w-full glass-premium sticky top-0 z-[100] transition-all duration-300">
            // Subtle gradient accent line at the very top
            <div class="h-[2px] w-full bg-gradient-to-r from-transparent via-primary to-transparent opacity-40"></div>
            <div class="max-w-[1440px] mx-auto px-6 md:px-10 h-[72px] flex items-center justify-between">
                // Logo Section — refined with gradient pill
                <a href="/" class="flex flex-1 items-center gap-3 group">
                    <div class="w-9 h-9 rounded-xl bg-gradient-to-br from-primary via-primary to-purple-600 flex items-center justify-center text-white font-black text-lg shadow-glow transition-all duration-300 group-hover:scale-110 group-hover:shadow-glow-lg group-hover:rounded-lg group-active:scale-95">
                        {config.name.chars().next().unwrap_or('D').to_uppercase().to_string()}
                    </div>
                    <span class="font-bold text-xl tracking-tight text-foreground hidden sm:block transition-colors group-hover:text-primary">
                        {config.name}
                    </span>
                </a>
                
                // Premium Search Pill (Desktop) — thinner, softer, with inner shadow
                <div class="hidden md:flex flex-1 max-w-md mx-auto items-center justify-between px-1.5 py-1.5 border border-neutral-200/80 shadow-inner-soft rounded-full bg-white/90 hover:shadow-premium hover:border-neutral-300 transition-all duration-300 cursor-pointer group">
                    <a href="/search?category=contractors" class="px-4 text-[13px] font-semibold text-foreground border-r border-neutral-200 hover:bg-neutral-100 rounded-full py-2 transition-all duration-200">
                        "Browse"
                    </a>
                    <a href="/search" class="px-4 text-[13px] font-semibold text-foreground hover:bg-neutral-100 rounded-full py-2 transition-all duration-200">
                        "Search"
                    </a>
                    <a href="/search" class="px-4 text-[13px] text-muted-foreground hover:bg-neutral-100 rounded-full py-2 transition-all duration-200 truncate max-w-[120px]">
                        "Filters"
                    </a>
                    <a href="/search" class="w-9 h-9 bg-gradient-to-br from-primary to-primary/80 rounded-full flex items-center justify-center text-white hover:shadow-glow transition-all duration-300 group-hover:scale-105 ml-1">
                        <svg class="w-4 h-4 stroke-[2.5]" fill="none" viewBox="0 0 24 24" stroke="currentColor"><path stroke-linecap="round" stroke-linejoin="round" d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z"></path></svg>
                    </a>
                </div>
                
                // Right Navigation — refined
                <div class="flex flex-1 items-center justify-end gap-2 md:gap-3">
                    <a href="/claim" class="text-[13px] font-semibold text-foreground/80 px-4 py-2.5 rounded-full hover:bg-neutral-100 transition-all duration-200 hidden lg:block">
                        "List your business"
                    </a>
                    <a href="/search" class="p-2.5 rounded-full hover:bg-neutral-100 text-foreground transition-all duration-200 md:hidden">
                        <svg class="w-5 h-5 stroke-2" fill="none" viewBox="0 0 24 24" stroke="currentColor"><path stroke-linecap="round" stroke-linejoin="round" d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z"></path></svg>
                    </a>
                    <button class="flex items-center gap-2 border border-neutral-200 p-1.5 pl-3 rounded-full hover:shadow-premium transition-all duration-300 bg-white relative group">
                        <svg class="w-4 h-4 text-neutral-500" fill="none" viewBox="0 0 24 24" stroke="currentColor"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 6h16M4 12h16M4 18h16"></path></svg>
                        <div class="w-8 h-8 bg-gradient-to-br from-neutral-600 to-neutral-800 rounded-full flex items-center justify-center overflow-hidden border-2 border-transparent group-hover:border-primary/30 transition-all duration-300">
                            <svg class="w-4 h-4 text-white mt-1.5" fill="currentColor" viewBox="0 0 20 20"><path fill-rule="evenodd" d="M10 9a3 3 0 100-6 3 3 0 000 6zm-7 9a7 7 0 1114 0H3z" clip-rule="evenodd"></path></svg>
                        </div>
                    </button>
                </div>
            </div>
        </header>
    }
}

#[component]
pub fn Footer() -> impl IntoView {
    view! {
        <footer class="w-full bg-neutral-950 text-neutral-300 mt-auto relative overflow-hidden">
            // Gradient top border
            <div class="h-[2px] w-full bg-gradient-to-r from-primary via-purple-500 to-pink-500 opacity-60"></div>

            <div class="max-w-[1440px] mx-auto px-8 md:px-12 pt-20 pb-12 relative z-10">
                <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-12 lg:gap-16">
                    <div>
                        <h5 class="font-semibold text-white mb-6 text-sm tracking-wider uppercase">"Explore"</h5>
                        <ul class="space-y-4 text-sm">
                            <li><a href="/search" class="text-neutral-400 hover:text-white transition-colors duration-200">"All Listings"</a></li>
                            <li><a href="/categories" class="text-neutral-400 hover:text-white transition-colors duration-200">"Specialties"</a></li>
                            <li><a href="/locations" class="text-neutral-400 hover:text-white transition-colors duration-200">"Top Rated"</a></li>
                        </ul>
                    </div>
                    <div>
                        <h5 class="font-semibold text-white mb-6 text-sm tracking-wider uppercase">"For Business"</h5>
                        <ul class="space-y-4 text-sm">
                            <li><a href="/claim" class="text-neutral-400 hover:text-white transition-colors duration-200">"List your business"</a></li>
                            <li><a href="/pricing" class="text-neutral-400 hover:text-white transition-colors duration-200">"Premium Features"</a></li>
                            <li><a href="/community" class="text-neutral-400 hover:text-white transition-colors duration-200">"Community"</a></li>
                        </ul>
                    </div>
                    <div>
                        <h5 class="font-semibold text-white mb-6 text-sm tracking-wider uppercase">"Legal"</h5>
                        <ul class="space-y-4 text-sm">
                            <li><a href="/terms" class="text-neutral-400 hover:text-white transition-colors duration-200">"Terms of Service"</a></li>
                            <li><a href="/privacy" class="text-neutral-400 hover:text-white transition-colors duration-200">"Privacy Policy"</a></li>
                            <li><a href="/sitemap" class="text-neutral-400 hover:text-white transition-colors duration-200">"Sitemap"</a></li>
                        </ul>
                    </div>
                    <div>
                        <h5 class="font-semibold text-white mb-6 text-sm tracking-wider uppercase">"Support"</h5>
                        <ul class="space-y-4 text-sm">
                            <li><a href="/contact" class="text-neutral-400 hover:text-white transition-colors duration-200">"Help Center"</a></li>
                            <li><a href="/faq" class="text-neutral-400 hover:text-white transition-colors duration-200">"Safety Information"</a></li>
                            <li><a href="/cancellation" class="text-neutral-400 hover:text-white transition-colors duration-200">"Cancellation Options"</a></li>
                        </ul>
                    </div>
                </div>
                
                // Bottom bar
                <div class="mt-16 pt-8 border-t border-neutral-800 flex flex-col md:flex-row justify-between items-center gap-4">
                    <div class="flex items-center gap-2 text-sm text-neutral-500">
                        <span>"© 2026 Directory Platform, Inc."</span>
                        <span class="hidden md:inline">"·"</span>
                        <a href="/privacy" class="hover:text-neutral-300 transition-colors">"Privacy"</a>
                        <span>"·"</span>
                        <a href="/terms" class="hover:text-neutral-300 transition-colors">"Terms"</a>
                    </div>
                    <div class="flex items-center gap-6">
                        <button class="text-sm text-neutral-400 hover:text-white transition-colors flex items-center gap-2">
                            <svg class="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M21 12a9 9 0 01-9 9m9-9a9 9 0 00-9-9m9 9H3m9 9a9 9 0 01-9-9m9 9c1.657 0 3-4.03 3-9s-1.343-9-3-9m0 18c-1.657 0-3-4.03-3-9s1.343-9 3-9m-9 9a9 9 0 019-9"></path></svg>
                            "English (US)"
                        </button>
                        <button class="text-sm text-neutral-400 hover:text-white transition-colors font-medium">"$ USD"</button>
                    </div>
                </div>
            </div>
        </footer>
    }
}

#[component]
pub fn MainLayout(children: Children) -> impl IntoView {
    view! {
        <div class="min-h-screen flex flex-col font-sans text-foreground selection:bg-primary/15 bg-background">
            <GlobalHeader />
            <main class="flex-grow flex flex-col relative w-full">
                {children()}
            </main>
            <Footer />
        </div>
    }
}
