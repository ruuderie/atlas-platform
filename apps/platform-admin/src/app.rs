use leptos::prelude::*;
use leptos_router::components::{Router, Route, Routes};
use leptos_router::path;

use crate::pages::dashboard::Dashboard;
use crate::pages::multi_site::MultiSite;
use crate::pages::crm_grid::CrmGrid;
use crate::pages::cms_editor::CmsEditor;
use crate::pages::login::Login;

use crate::api::auth::validate_session;
use crate::api::models::{UserInfo, DirectoryModel};
use crate::api::directories::get_directories;

#[derive(Copy, Clone, Debug)]
pub struct GlobalToast {
    pub message: RwSignal<Option<String>>,
}

#[component]
pub fn App() -> impl IntoView {
    let (user, set_user) = signal(None::<UserInfo>);
    provide_context(set_user);
    provide_context(user);

    let dirs_res = LocalResource::new(|| async move { get_directories().await.unwrap_or_default() });
    provide_context(dirs_res);

    let toast = GlobalToast { message: RwSignal::new(None) };
    provide_context(toast);

    // Validate session on load
    let session_check = leptos::task::spawn_local(async move {
        if let Ok(valid_user) = validate_session().await {
            set_user.set(Some(valid_user));
        }
    });

    view! {
        <div class="fixed bottom-4 right-4 z-[9999] pointer-events-none">
            {move || toast.message.get().map(|msg| view! {
                <div class="glass-panel text-on-surface px-4 py-3 rounded-xl flex items-center justify-between min-w-[300px] pointer-events-auto border border-outline-variant/40">
                    <span class="text-sm font-medium">{msg}</span>
                    <button class="ml-4 hover:opacity-70 font-bold text-on-surface-variant" on:click=move |_| toast.message.set(None)>"✕"</button>
                </div>
            })}
        </div>
        <Router>
            <Routes fallback=|| "Not found.">
                <Route path=path!("/login") view=Login />
                <Route path=path!("/*any") view=AuthenticatedLayout />
            </Routes>
        </Router>
    }
}

#[component]
pub fn AuthenticatedLayout() -> impl IntoView {
    let user = use_context::<ReadSignal<Option<UserInfo>>>().expect("user context");
    let dirs_res = use_context::<LocalResource<Vec<DirectoryModel>>>().expect("dirs context");
    let navigate = leptos_router::hooks::use_navigate();
    let location = leptos_router::hooks::use_location();
    let (show_profile_menu, set_show_profile_menu) = signal(false);

    Effect::new(move |_| {
        if user.get().is_none() {
            // navigate("/login", Default::default());
        }
    });

    // Derive active nav state from the current path
    let current_path = Signal::derive(move || location.pathname.get());

    let nav_active_class = move |path: &'static str| {
        let p = current_path.get();
        if (path == "/" && p == "/") || (path != "/" && p.starts_with(path)) {
            "text-[#7bd0ff] border-b-2 border-[#7bd0ff] pb-1 font-semibold tracking-[-0.02em]"
        } else {
            "text-[#91aaeb] hover:text-[#dee5ff] font-semibold tracking-[-0.02em] transition-colors"
        }
    };

    let side_active_class = move |path: &'static str| {
        let p = current_path.get();
        if (path == "/" && p == "/") || (path != "/" && p.starts_with(path)) {
            "flex items-center gap-3 px-3 py-2.5 bg-[#05183c] text-[#7bd0ff] rounded-md font-['Inter'] text-sm font-medium tracking-wide uppercase active:translate-x-1 duration-150 transition-all"
        } else {
            "flex items-center gap-3 px-3 py-2.5 text-[#91aaeb] hover:bg-[#05183c]/50 hover:text-[#dee5ff] rounded-md font-['Inter'] text-sm font-medium tracking-wide uppercase active:translate-x-1 duration-150 transition-all"
        }
    };

    view! {
        <Show when=move || user.get().is_some() fallback=move || view! {
            <div class="h-screen w-full flex items-center justify-center bg-surface text-on-surface-variant font-sans antialiased">
                <div>"Checking session..."</div>
                {
                   navigate("/login", Default::default());
                   ""
                }
            </div>
        }>
            <div class="h-screen w-full bg-surface text-on-surface font-sans antialiased overflow-hidden">
                // ── Top Nav Bar ──
                <header class="fixed top-0 w-full z-50 flex justify-between items-center px-6 h-16 bg-[#060e20]">
                    <div class="flex items-center gap-8">
                        <a href="/" class="text-xl font-bold text-[#dee5ff] tracking-[-0.02em] whitespace-nowrap">"The Intelligence Layer"</a>
                        <nav class="hidden md:flex gap-6">
                            <a href="/" class=move || nav_active_class("/")>"Platform Overview"</a>
                            <a href="/sites" class=move || nav_active_class("/sites")>"Network Directories"</a>
                            <a href="/crm" class=move || nav_active_class("/crm")>"Sales & Relationships"</a>
                            <a href="/cms" class=move || nav_active_class("/cms")>"Content Management"</a>
                        </nav>
                    </div>
                    <div class="flex items-center gap-4">
                        <button class="bg-surface-container-high px-3 py-1.5 rounded-md flex items-center gap-2 text-sm font-medium border border-outline-variant/20 hover:bg-surface-bright/20 transition-all">
                            <span class="text-on-surface-variant uppercase tracking-widest text-[10px]">"Site Selector"</span>
                            <span class="text-primary font-bold">"All"</span>
                            <span class="material-symbols-outlined text-sm">"expand_more"</span>
                        </button>
                        <div class="flex items-center gap-2 ml-2">
                            <button class="p-2 text-[#91aaeb] hover:bg-[#002867]/20 transition-colors rounded-full">
                                <span class="material-symbols-outlined">"notifications"</span>
                            </button>
                            <button class="p-2 text-[#91aaeb] hover:bg-[#002867]/20 transition-colors rounded-full">
                                <span class="material-symbols-outlined">"settings"</span>
                            </button>
                            <div class="relative cursor-pointer" on:click=move |_| set_show_profile_menu.update(|v| *v = !*v)>
                                <div class="w-8 h-8 rounded-full bg-surface-container-highest overflow-hidden ml-2 border border-outline-variant/30 flex items-center justify-center text-primary text-xs font-bold">
                                    {move || user.get().map(|u| u.first_name.chars().next().unwrap_or('A').to_string()).unwrap_or_else(|| "A".to_string())}
                                </div>
                                <Show when=move || show_profile_menu.get()>
                                    <div class="absolute right-0 top-10 mt-2 w-48 glass-panel border border-outline-variant/40 rounded-xl py-1 z-[100] overflow-hidden">
                                        <div class="px-4 py-3 border-b border-outline-variant/20 text-sm">
                                            <p class="font-medium text-on-surface">{move || user.get().map(|u| format!("{} {}", u.first_name, u.last_name)).unwrap_or_else(|| "Admin".to_string())}</p>
                                            <p class="text-on-surface-variant text-xs truncate">{move || user.get().map(|u| u.email.clone()).unwrap_or_else(|| "admin@foundry.local".to_string())}</p>
                                        </div>
                                        <a href="/settings" class="block w-full text-left px-4 py-2.5 text-sm text-on-surface hover:bg-surface-bright/20 transition-colors" on:click=move |e| e.stop_propagation()>"Account Settings"</a>
                                        <button class="block w-full text-left px-4 py-2.5 text-sm text-error hover:bg-error-container/20 transition-colors" on:click=move |e| { e.stop_propagation(); set_show_profile_menu.set(false); }>"Sign out"</button>
                                    </div>
                                </Show>
                            </div>
                        </div>
                    </div>
                </header>

                // ── Side Nav Bar ──
                <aside class="fixed left-0 top-16 bottom-0 w-64 flex flex-col py-4 px-3 bg-[#06122d]">
                    <div class="px-3 mb-6 flex items-center gap-3">
                        <div class="w-10 h-10 rounded-xl bg-primary-container flex items-center justify-center">
                            <span class="material-symbols-outlined text-primary">"terminal"</span>
                        </div>
                        <div>
                            <div class="text-on-surface font-bold text-sm">"Admin Console"</div>
                            <div class="text-on-surface-variant text-[10px] uppercase tracking-widest">"Network Root"</div>
                        </div>
                    </div>
                    <nav class="flex-1 space-y-1">
                        <a href="/" class=move || side_active_class("/")>
                            <span class="material-symbols-outlined">"dashboard"</span>
                            <span>"Overview"</span>
                        </a>
                        <a href="/sites" class=move || side_active_class("/sites")>
                            <span class="material-symbols-outlined">"lan"</span>
                            <span>"Directories"</span>
                        </a>
                        <a href="/crm" class=move || side_active_class("/crm")>
                            <span class="material-symbols-outlined">"handshake"</span>
                            <span>"Sales"</span>
                        </a>
                        <a href="/cms" class=move || side_active_class("/cms")>
                            <span class="material-symbols-outlined">"article"</span>
                            <span>"Content"</span>
                        </a>
                    </nav>
                    <div class="mt-auto border-t border-outline-variant/10 pt-4 space-y-1">
                        <a href="#" class="flex items-center gap-3 px-3 py-2 text-[#91aaeb] hover:text-[#dee5ff] font-['Inter'] text-xs font-medium tracking-wide uppercase">
                            <span class="material-symbols-outlined text-sm">"help"</span>
                            <span>"Support"</span>
                        </a>
                        <a href="#" class="flex items-center gap-3 px-3 py-2 text-[#91aaeb] hover:text-[#dee5ff] font-['Inter'] text-xs font-medium tracking-wide uppercase">
                            <span class="material-symbols-outlined text-sm">"terminal"</span>
                            <span>"Logs"</span>
                        </a>
                        <a href="/sites/new" class="block w-full mt-4">
                            <button class="w-full btn-primary-gradient text-on-primary-container py-2.5 rounded-md text-xs font-bold uppercase tracking-widest shadow-lg shadow-primary/10 hover:opacity-90 transition-opacity">
                                "New Site"
                            </button>
                        </a>
                    </div>
                </aside>

                // ── Main Content ──
                <main class="ml-64 mt-16 p-8 min-h-screen bg-surface-container">
                    <Routes fallback=|| "Not found.">
                        <Route path=path!("/") view=Dashboard />
                        <Route path=path!("/sites") view=MultiSite />
                        <Route path=path!("/sites/new") view=crate::pages::site_create::SiteCreate />
                        <Route path=path!("/sites/:id") view=crate::pages::site_dashboard::SiteDashboard />
                        <Route path=path!("/crm") view=CrmGrid />
                        <Route path=path!("/crm/new") view=crate::pages::crm_create::CrmCreate />
                        <Route path=path!("/crm/:entity/:id") view=crate::pages::crm_detail::CrmDetail />
                        <Route path=path!("/cms") view=CmsEditor />
                    </Routes>
                </main>
            </div>
        </Show>
    }
}
