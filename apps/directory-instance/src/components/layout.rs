use leptos::prelude::*;
use crate::app::DirectoryConfig;

#[component]
pub fn GlobalHeader() -> impl IntoView {
    let config = use_context::<DirectoryConfig>().expect("DirectoryConfig must be provided");
    
    view! {
        <nav class="fixed top-0 w-full z-50 bg-white/80 dark:bg-slate-900/80 backdrop-blur-xl shadow-sm dark:shadow-none transition-all duration-300">
            <div class="flex justify-between items-center px-8 py-4 max-w-7xl mx-auto">
                <a href="/" class="text-2xl font-extrabold text-primary dark:text-blue-500 font-headline tracking-tight hover:opacity-90 transition-opacity">
                    {config.name.clone()}
                </a>
                <div class="hidden md:flex items-center space-x-8 font-headline font-bold tracking-tight text-sm">
                    <a class="text-slate-600 dark:text-slate-400 hover:text-primary transition-all duration-300" href="/search">"Browse"</a>
                    <a class="text-slate-600 dark:text-slate-400 hover:text-primary transition-all duration-300" href="/expertise">"Expertise"</a>
                    <a class="text-slate-600 dark:text-slate-400 hover:text-primary transition-all duration-300" href="/about">"About"</a>
                    <a class="text-slate-600 dark:text-slate-400 hover:text-primary transition-all duration-300" href="/contact">"Contact"</a>
                </div>
                <div class="flex items-center gap-6">
                    <a href="/list-property" class="bg-primary text-white px-6 py-2.5 rounded-lg font-bold transition-all duration-300 hover:opacity-80 active:scale-95 shadow-sm text-sm inline-block text-center">
                        "List a Property"
                    </a>
                </div>
            </div>
            <div class="bg-slate-100 dark:bg-slate-800 h-[1px] w-4/5 mx-auto opacity-80"></div>
        </nav>
    }
}

#[component]
pub fn Footer() -> impl IntoView {
    let config = use_context::<DirectoryConfig>().expect("DirectoryConfig must be provided");
    
    view! {
        <footer class="w-full border-t border-slate-200 dark:border-slate-800 bg-slate-50 dark:bg-slate-950 mt-auto">
            <div class="grid grid-cols-1 md:grid-cols-4 gap-12 px-8 py-16 max-w-7xl mx-auto">
                <div class="md:col-span-1">
                    <div class="text-xl font-bold text-slate-900 dark:text-slate-100 font-headline mb-4">
                        {config.name.clone()}
                    </div>
                    <p class="font-body text-sm text-slate-500 dark:text-slate-400 leading-relaxed">
                        {config.description.clone()}
                    </p>
                </div>
                <div>
                    <h5 class="font-bold text-slate-900 dark:text-slate-100 mb-6 uppercase tracking-wider text-xs">"Directory"</h5>
                    <ul class="space-y-4 font-body text-sm text-slate-500 dark:text-slate-400">
                        <li><a class="hover:text-primary dark:hover:text-primary hover:underline underline-offset-4 transition-colors" href="/search">"Directory Index"</a></li>
                        <li><a class="hover:text-primary dark:hover:text-primary hover:underline underline-offset-4 transition-colors" href="#">"Partner Program"</a></li>
                        <li><a class="hover:text-primary dark:hover:text-primary hover:underline underline-offset-4 transition-colors" href="#">"Support Center"</a></li>
                    </ul>
                </div>
                <div>
                    <h5 class="font-bold text-slate-900 dark:text-slate-100 mb-6 uppercase tracking-wider text-xs">"Legal"</h5>
                    <ul class="space-y-4 font-body text-sm text-slate-500 dark:text-slate-400">
                        <li><a class="hover:text-primary dark:hover:text-primary hover:underline underline-offset-4 transition-colors" href="#">"Privacy Policy"</a></li>
                        <li><a class="hover:text-primary dark:hover:text-primary hover:underline underline-offset-4 transition-colors" href="#">"Terms of Service"</a></li>
                    </ul>
                </div>
                <div>
                    <h5 class="font-bold text-slate-900 dark:text-slate-100 mb-6 uppercase tracking-wider text-xs">"Newsletter"</h5>
                    <div class="flex gap-2">
                        <input class="bg-white dark:bg-slate-900 border border-slate-200 dark:border-slate-800 rounded px-4 py-2 text-sm w-full focus:ring-1 focus:ring-primary focus:outline-none" placeholder="Enter your email" type="email"/>
                        <button class="bg-primary text-white p-2 px-3 rounded hover:opacity-90 transition-opacity flex items-center justify-center">
                            <span class="material-symbols-outlined text-sm" data-icon="send">"send"</span>
                        </button>
                    </div>
                </div>
            </div>
            <div class="max-w-7xl mx-auto px-8 py-8 border-t border-slate-200 dark:border-slate-800 flex flex-col md:flex-row justify-between items-center gap-4">
                <p class="font-body text-sm text-slate-500 dark:text-slate-400">
                    "© 2026 " {config.name.clone()} ". All rights reserved."
                </p>
                <div class="flex gap-6">
                    <span class="material-symbols-outlined text-slate-400 cursor-pointer hover:text-primary transition-colors text-lg" data-icon="public">"public"</span>
                    <span class="material-symbols-outlined text-slate-400 cursor-pointer hover:text-primary transition-colors text-lg" data-icon="contact_support">"contact_support"</span>
                </div>
            </div>
        </footer>
    }
}

#[component]
pub fn MainLayout(children: Children) -> impl IntoView {
    view! {
        <div class="min-h-screen flex flex-col font-body text-foreground selection:bg-primary-fixed selection:text-on-primary-fixed bg-background">
            <GlobalHeader />
            <main class="flex-grow flex flex-col relative w-full pt-16 md:pt-20">
                {children()}
            </main>
            <Footer />
        </div>
    }
}
