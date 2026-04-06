use leptos::prelude::*;
use leptos_router::components::{Router, Route, Routes};
use leptos_router::path;

use crate::pages::dashboard::Dashboard;
use crate::pages::apps::Apps;
use crate::pages::crm_grid::CrmGrid;
use crate::pages::cms_editor::CmsEditor;
use crate::pages::login::Login;
use crate::pages::verify_token::VerifyToken;
use crate::pages::setup::Setup;
use crate::pages::network::network_types::NetworkTypes;
use crate::pages::network::network_type_detail::NetworkTypeDetail;
use crate::pages::network::network_type_create::NetworkTypeCreate;
use crate::pages::network::categories::Categories;
use crate::pages::network::category_detail::CategoryDetail;
use crate::pages::network::category_create::CategoryCreate;
use crate::pages::network::templates::Templates;
use crate::pages::network::template_detail::TemplateDetail;
use crate::pages::network::template_create::TemplateCreate;
use crate::pages::network::listings::Listings;
use crate::pages::network::listing_create::ListingCreate;
use crate::pages::network::listing_detail::ListingDetail;
use crate::pages::platform_admins::PlatformAdmins;
use crate::api::auth::validate_session;
use crate::api::models::{UserInfo, PlatformAppModel};
use crate::api::networks::get_networks;

#[derive(Copy, Clone, Debug)]
pub struct GlobalToast {
    pub message: RwSignal<Option<String>>,
}

impl GlobalToast {
    pub fn show_toast(&self, _title: &str, msg: &str, _type: &str) {
        self.message.set(Some(msg.to_string()));
    }
}

#[component]
pub fn App() -> impl IntoView {
    let (user, set_user) = signal(None::<UserInfo>);
    let (auth_checked, set_auth_checked) = signal(false);
    provide_context(set_user);
    provide_context(user);
    provide_context(auth_checked);

    let dirs_res = LocalResource::new(|| async move { get_networks().await.unwrap_or_default() });
    provide_context(dirs_res);

    let (active_network, set_active_network) = signal(None::<uuid::Uuid>);
    provide_context(active_network);
    provide_context(set_active_network);

    let toast = GlobalToast { message: RwSignal::new(None) };
    provide_context(toast);

    // Validate session on load
    let session_check = leptos::task::spawn_local(async move {
        if let Ok(valid_user) = validate_session().await {
            set_user.set(Some(valid_user));
        }
        set_auth_checked.set(true);
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
                <Route path=path!("/verify-token/:token") view=VerifyToken />
                <Route path=path!("/magic-login") view=crate::pages::magic_login::MagicLogin />
                <Route path=path!("/setup") view=Setup />
                <Route path=path!("/*any") view=AuthenticatedLayout />
            </Routes>
        </Router>
    }
}

#[component]
pub fn AuthenticatedLayout() -> impl IntoView {
    let user = use_context::<ReadSignal<Option<UserInfo>>>().expect("user context");
    let set_user = use_context::<WriteSignal<Option<crate::api::models::UserInfo>>>().expect("set user context");
    let auth_checked = use_context::<ReadSignal<bool>>().expect("auth checked context");
    let dirs_res = use_context::<LocalResource<Vec<PlatformAppModel>>>().expect("dirs context");
    let active_network = use_context::<ReadSignal<Option<uuid::Uuid>>>().expect("active network");
    let set_active_network = use_context::<WriteSignal<Option<uuid::Uuid>>>().expect("set active network");
    let navigate = leptos_router::hooks::use_navigate();
    let location = leptos_router::hooks::use_location();
    let (show_profile_menu, set_show_profile_menu) = signal(false);

    Effect::new(move |_| {
        if user.get().is_none() && auth_checked.get() {
            navigate("/login", Default::default());
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
            </div>
        }>
            <div class="h-screen w-full bg-surface text-on-surface font-sans antialiased overflow-hidden">
                // ── Top Nav Bar ──
                <header class="fixed top-0 w-full z-50 flex justify-between items-center px-6 h-16 bg-[#060e20]">
                    <div class="flex items-center gap-8 w-full max-w-2xl">
                        <a href="/" class="text-xl font-bold text-[#dee5ff] tracking-[-0.02em] whitespace-nowrap">"The Intelligence Layer"</a>
                        <div class="hidden md:flex flex-1 relative items-center max-w-lg w-full ml-4">
                            <span class="material-symbols-outlined absolute left-3 text-on-surface-variant text-lg">"search"</span>
                            <input 
                                type="text"
                                placeholder="Search across networks, users, and listings (Cmd+K)..."
                                class="w-full bg-[#05183c] border border-outline-variant/30 text-on-surface text-sm rounded-lg pl-10 pr-4 py-2 focus:ring-1 focus:ring-primary focus:border-primary transition-all placeholder:text-[#91aaeb]/60"
                            />
                            <div class="absolute right-3 px-1.5 py-0.5 rounded bg-surface-container-highest border border-outline-variant/20 text-[#91aaeb] text-[10px] font-mono font-bold tracking-widest hidden lg:block hover:text-primary transition-colors cursor-pointer">
                                "⌘K"
                            </div>
                        </div>
                    </div>
                    <div class="flex items-center gap-4">
                        <select
                            class="bg-surface-container-high px-3 py-1.5 rounded-md text-sm font-medium border border-outline-variant/20 hover:bg-surface-bright/20 focus:ring-primary focus:border-primary text-on-surface min-w-[150px]"
                            on:change=move |ev| {
                                let val = event_target_value(&ev);
                                if val.is_empty() {
                                    set_active_network.set(None);
                                } else if let Ok(parsed) = uuid::Uuid::parse_str(&val) {
                                    set_active_network.set(Some(parsed));
                                }
                            }
                        >
                            <option value="">"All Sites"</option>
                            <Suspense fallback=move || view! { <option>"Loading..."</option> }>
                                {move || dirs_res.get().map(|networks| view! {
                                    <For
                                        each=move || networks.clone()
                                        key=|dir| dir.tenant_id.clone()
                                        children=move |dir| {
                                            view! {
                                                <option value=dir.tenant_id.clone()>{dir.name.clone()}</option>
                                            }
                                        }
                                    />
                                })}
                            </Suspense>
                        </select>
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
                                        <a href="/settings" class="block w-full text-left px-4 py-2.5 text-sm text-on-surface hover:bg-surface-bright/20 transition-colors" on:click=move |_| set_show_profile_menu.set(false)>"Account Settings"</a>
                                        <button class="block w-full text-left px-4 py-2.5 text-sm text-error hover:bg-error-container/20 transition-colors" on:click=move |e| { 
                                            e.stop_propagation(); 
                                            set_show_profile_menu.set(false); 
                                            leptos::task::spawn_local(async move {
                                                let _ = crate::api::auth::logout().await;
                                                set_user.set(None);
                                                let _ = web_sys::window().unwrap().location().assign("/login");
                                            });
                                        }>"Sign out"</button>
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
                        <a href="/apps" class=move || side_active_class("/apps")>
                            <span class="material-symbols-outlined">"dns"</span>
                            <span>"Applications"</span>
                        </a>
                        <a href="/network/network-types" class=move || side_active_class("/network/network-types")>
                            <span class="material-symbols-outlined">"schema"</span>
                            <span>"Network Types"</span>
                        </a>
                        <a href="/network/categories" class=move || side_active_class("/network/categories")>
                            <span class="material-symbols-outlined">"category"</span>
                            <span>"Categories"</span>
                        </a>
                        <a href="/network/templates" class=move || side_active_class("/network/templates")>
                            <span class="material-symbols-outlined">"draw"</span>
                            <span>"Templates"</span>
                        </a>
                        <a href="/network/listings" class=move || side_active_class("/network/listings")>
                            <span class="material-symbols-outlined">"store"</span>
                            <span>"Listings"</span>
                        </a>
                        <a href="/crm" class=move || side_active_class("/crm")>
                            <span class="material-symbols-outlined">"handshake"</span>
                            <span>"Sales"</span>
                        </a>
                        <a href="/cms" class=move || side_active_class("/cms")>
                            <span class="material-symbols-outlined">"article"</span>
                            <span>"Content"</span>
                        </a>
                        <a href="/admins" class=move || side_active_class("/admins")>
                            <span class="material-symbols-outlined">"group"</span>
                            <span>"Users"</span>
                        </a>
                    </nav>
                    <div class="mt-auto border-t border-outline-variant/10 pt-4 space-y-1">
                        <a href="/support" class="flex items-center gap-3 px-3 py-2 text-[#91aaeb] hover:text-[#dee5ff] font-['Inter'] text-xs font-medium tracking-wide uppercase">
                            <span class="material-symbols-outlined text-sm">"help"</span>
                            <span>"Support"</span>
                        </a>
                        <a href="/logs" class="flex items-center gap-3 px-3 py-2 text-[#91aaeb] hover:text-[#dee5ff] font-['Inter'] text-xs font-medium tracking-wide uppercase">
                            <span class="material-symbols-outlined text-sm">"terminal"</span>
                            <span>"Logs"</span>
                        </a>
                        <a href="/apps/new" class="block w-full mt-4">
                            <button class="w-full btn-primary-gradient text-on-primary-container py-2.5 rounded-md text-xs font-bold uppercase tracking-widest shadow-lg shadow-primary/10 hover:opacity-90 transition-opacity">
                                "New Application"
                            </button>
                        </a>
                    </div>
                </aside>

                // ── Main Content ──
                <main class="ml-64 mt-16 p-8 min-h-screen bg-surface-container">
                    <Routes fallback=|| "Not found.">
                        <Route path=path!("/") view=Dashboard />
                        <Route path=path!("/apps") view=Apps />
                        <Route path=path!("/apps/new") view=crate::pages::app_create::AppCreate />
                        <Route path=path!("/apps/:id") view=crate::pages::app_dashboard::AppDashboard />
                        <Route path=path!("/network/network-types") view=NetworkTypes />
                        <Route path=path!("/network/network-types/new") view=NetworkTypeCreate />
                        <Route path=path!("/network/network-types/:id") view=NetworkTypeDetail />
                        <Route path=path!("/network/categories") view=Categories />
                        <Route path=path!("/network/categories/new") view=CategoryCreate />
                        <Route path=path!("/network/categories/:id") view=CategoryDetail />
                        <Route path=path!("/network/templates") view=Templates />
                        <Route path=path!("/network/templates/new") view=TemplateCreate />
                        <Route path=path!("/network/templates/:id") view=TemplateDetail />
                        <Route path=path!("/network/listings") view=Listings />
                        <Route path=path!("/network/listings/new") view=ListingCreate />
                        <Route path=path!("/network/listings/:id") view=ListingDetail />
                        <Route path=path!("/crm") view=CrmGrid />
                        <Route path=path!("/crm/new") view=crate::pages::crm_create::CrmCreate />
                        <Route path=path!("/crm/:entity/:id") view=crate::pages::crm_detail::CrmDetail />
                        <Route path=path!("/cms") view=CmsEditor />
                        <Route path=path!("/admins") view=PlatformAdmins />
                        <Route path=path!("/settings") view=crate::pages::settings::Settings />
                    </Routes>
                </main>
            </div>
        </Show>
    }
}
