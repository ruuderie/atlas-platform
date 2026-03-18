use leptos::prelude::*;
use leptos_router::components::{Router, Route, Routes};
use leptos_router::path;

use crate::pages::dashboard::Dashboard;
use crate::pages::multi_site::MultiSite;
use crate::pages::crm_grid::CrmGrid;
use crate::pages::cms_editor::CmsEditor;
use crate::pages::login::Login;

use shared_ui::components::ui::header::Header;
use shared_ui::components::ui::select::{Select, SelectTrigger, SelectValue, SelectContent, SelectGroup, SelectLabel, SelectOption};
use shared_ui::components::ui::avatar::{Avatar, AvatarImage, AvatarFallback};

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
                <div class="bg-destructive text-destructive-foreground px-4 py-3 rounded-md shadow-xl flex items-center justify-between min-w-[300px] pointer-events-auto border border-border">
                    <span class="text-sm font-medium">{msg}</span>
                    <button class="ml-4 hover:opacity-70 font-bold" on:click=move |_| toast.message.set(None)>"✕"</button>
                </div>
            })}
        </div>
        <Router>
            <Routes fallback=|| "Not found.">
                <Route path=path!("/login") view=Login />
                
                // Catch-all auth wrapper for other routes
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
    let (is_mobile_menu_open, set_is_mobile_menu_open) = signal(false);
    let (show_profile_menu, set_show_profile_menu) = signal(false);

    Effect::new(move |_| {
        // Redirect to login if not authenticated
        if user.get().is_none() {
            // navigate("/login", Default::default());
        }
    });

    view! {
        <Show when=move || user.get().is_some() fallback=move || view! { 
            <div class="h-screen w-full flex items-center justify-center dark bg-slate-950 bg-gradient-to-br from-blue-500/10 to-cyan-500/10 text-slate-300 font-sans antialiased">
                // if it's explicitly navigating, this is a brief flash, or we show a loader
                <div class="text-muted-foreground">"Checking session..."</div>
                {
                   // Fallback logic to trigger navigation if user is really none
                   // Effect sometimes runs too late in SSR, but this is CSR
                   navigate("/login", Default::default());
                   ""
                }
            </div> 
        }>
            <div class="flex h-screen w-full dark bg-slate-950 bg-gradient-to-br from-blue-500/10 to-cyan-500/10 text-slate-300 font-sans antialiased overflow-hidden">
                <Show when=move || is_mobile_menu_open.get()>
                    <div class="fixed inset-0 bg-background/80 backdrop-blur-sm z-[90] lg:hidden" on:click=move |_| set_is_mobile_menu_open.set(false)></div>
                </Show>

                <aside class=move || format!(
                    "{} lg:relative lg:flex flex-col w-64 border-r border-border bg-card z-[100] transition-transform duration-300 ease-in-out shrink-0 {}",
                    if is_mobile_menu_open.get() { "fixed inset-y-0 left-0 translate-x-0 flex shadow-2xl" } else { "fixed inset-y-0 left-0 -translate-x-full lg:translate-x-0 hidden lg:flex" },
                    ""
                )>
                    <div class="h-16 flex items-center justify-between px-6 border-b border-border shrink-0">
                        <h2 class="text-lg font-semibold text-primary">"Admin Portal"</h2>
                        <button class="lg:hidden p-2 -mr-2 text-muted-foreground hover:text-foreground" on:click=move |_| set_is_mobile_menu_open.set(false)>
                            <svg xmlns="http://www.w3.org/2000/svg" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M18 6 6 18"/><path d="m6 6 12 12"/></svg>
                        </button>
                    </div>
                    <nav class="flex-1 p-4 space-y-2 overflow-y-auto">
                        <a href="/" on:click=move |_| { set_is_mobile_menu_open.set(false); } class="block px-4 py-2 text-sm rounded-md hover:bg-secondary hover:text-secondary-foreground transition-colors">"Platform Overview"</a>
                        <a href="/sites" on:click=move |_| { set_is_mobile_menu_open.set(false); } class="block px-4 py-2 text-sm rounded-md hover:bg-secondary hover:text-secondary-foreground transition-colors">"Network Directories"</a>
                        <a href="/crm" on:click=move |_| { set_is_mobile_menu_open.set(false); } class="block px-4 py-2 text-sm rounded-md hover:bg-secondary hover:text-secondary-foreground transition-colors">"Sales & Relationships"</a>
                        <a href="/cms" on:click=move |_| { set_is_mobile_menu_open.set(false); } class="block px-4 py-2 text-sm rounded-md hover:bg-secondary hover:text-secondary-foreground transition-colors">"Content Management"</a>
                    </nav>
                </aside>
                
                <div class="flex-1 flex flex-col overflow-hidden w-full min-w-0 relative z-0 backdrop-blur-3xl">
                    <Header>
                        <div class="h-16 flex items-center justify-between px-4 lg:px-6 border-b border-border bg-card/50 backdrop-blur-sm relative z-50 shrink-0">
                            <div class="flex items-center space-x-2 lg:space-x-4">
                                <button class="lg:hidden p-2 -ml-2 rounded-md hover:bg-muted" on:click=move |_| set_is_mobile_menu_open.update(|v| *v = !*v)>
                                    <svg xmlns="http://www.w3.org/2000/svg" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><line x1="4" x2="20" y1="12" y2="12"/><line x1="4" x2="20" y1="6" y2="6"/><line x1="4" x2="20" y1="18" y2="18"/></svg>
                                </button>
                                <span class="hidden sm:inline text-sm font-medium text-muted-foreground whitespace-nowrap">"Active Site:"</span>
                                <div class="w-[180px] sm:w-[240px]">
                                    <Select default_value="all".to_string()>
                                        <SelectTrigger id="site-selector".to_string() class="w-full bg-background relative z-50">
                                            <SelectValue placeholder="Select a site...".to_string() />
                                        </SelectTrigger>
                                        <SelectContent class="z-[100] mt-1 bg-popover shadow-md border rounded-md">
                                            <SelectGroup>
                                                <SelectLabel>"Global"</SelectLabel>
                                                <SelectOption value="all".to_string()>"Global (All Sites)"</SelectOption>
                                            </SelectGroup>
                                            <SelectGroup>
                                                <SelectLabel>"Network Directories"</SelectLabel>
                                                <Suspense fallback=move || view! { <div class="px-2 py-1 text-sm text-muted-foreground">"Loading..."</div> }>
                                                    {move || dirs_res.get().map(|dirs| view! {
                                                        <For
                                                            each=move || dirs.clone()
                                                            key=|dir| dir.id.clone()
                                                            children=move |dir| view! {
                                                                <SelectOption value=dir.id.clone()>{dir.name}</SelectOption>
                                                            }
                                                        />
                                                    })}
                                                </Suspense>
                                            </SelectGroup>
                                        </SelectContent>
                                    </Select>
                                </div>
                            </div>
                            <div class="flex items-center space-x-3 relative cursor-pointer" on:click=move |_| set_show_profile_menu.update(|v| *v = !*v)>
                                <span class="text-sm font-medium">{move || user.get().map(|u| format!("{} {}", u.first_name, u.last_name)).unwrap_or_else(|| "Admin".to_string())}</span>
                                <Avatar class="w-8 h-8 hover:ring-2 hover:ring-primary transition-all".to_string()>
                                    <AvatarFallback>{move || user.get().map(|u| u.first_name.chars().next().unwrap_or('A').to_string()).unwrap_or_else(|| "A".to_string())}</AvatarFallback>
                                </Avatar>
                                <Show when=move || show_profile_menu.get()>
                                    <div class="absolute right-0 top-10 mt-2 w-48 bg-card border border-border rounded-xl shadow-2xl py-1 z-[100] overflow-hidden">
                                        <div class="px-4 py-3 border-b border-border/50 text-sm">
                                            <p class="font-medium text-foreground">{move || user.get().map(|u| format!("{} {}", u.first_name, u.last_name)).unwrap_or_else(|| "Admin".to_string())}</p>
                                            <p class="text-muted-foreground text-xs truncate">{move || user.get().map(|u| u.email.clone()).unwrap_or_else(|| "admin@foundry.local".to_string())}</p>
                                        </div>
                                        <a href="/settings" class="block w-full text-left px-4 py-2.5 text-sm text-foreground hover:bg-muted transition-colors" on:click=move |e| e.stop_propagation()>"Account Settings"</a>
                                        <button class="block w-full text-left px-4 py-2.5 text-sm text-destructive hover:bg-destructive/10 transition-colors" on:click=move |e| { e.stop_propagation(); set_show_profile_menu.set(false); }>"Sign out"</button>
                                    </div>
                                </Show>
                            </div>
                        </div>
                    </Header>
                    
                    <main class="flex-1 overflow-auto p-8 relative z-0">
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
            </div>
        </Show>
    }
}
