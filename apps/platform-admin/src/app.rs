/* 
 * TODO(next-developer): MIGRATION TO AtlasApp API TRAIT REQUIRED
 * 
 * This legacy application currently has its routes, migrations, and background jobs
 * hardcoded into the global Atlas platform core. 
 * 
 * We have introduced a strict, standardized Rust API trait: `AtlasApp` 
 * located at `backend/src/traits/atlas_app.rs`. 
 * 
 * Future work requires refactoring this app to implement the `AtlasApp` trait 
 * (providing perfect encapsulation for its Axum Router, SeaORM Migrations, and Background Jobs) 
 * instead of manually merging them globally.
 * 
 * See the full integration protocol at: `docs/atlas_app_integration.md`
 */
use leptos::prelude::*;
use leptos_router::components::{Router, Route, Routes};
use leptos_router::path;

use crate::pages::dashboard::Dashboard;
use crate::pages::apps::index::Apps;
use crate::pages::crm::grid::CrmGrid;
use crate::pages::products::index::PlatformProducts;
use crate::pages::products::detail::ProductDetail;
use crate::pages::billing::scorecards::Scorecards;
use crate::pages::billing::scorecard_session::ScorecardSession;
use crate::pages::network::syndication::SyndicationManager;
use crate::pages::network::index::NetworkRegistry;
use crate::pages::network::create::NetworkCreate;
use crate::pages::network::detail::NetworkDetail;
use crate::pages::marketing::index::MarketingLanding;
use crate::pages::auth::login::Login;
use crate::pages::auth::verify_token::VerifyToken;
use crate::pages::auth::setup::Setup;
use crate::pages::network::types::index::NetworkTypes;
use crate::pages::network::types::detail::NetworkTypeDetail;
use crate::pages::network::types::create::NetworkTypeCreate;
use crate::pages::network::categories::index::Categories;
use crate::pages::network::categories::detail::CategoryDetail;
use crate::pages::network::categories::create::CategoryCreate;
use crate::pages::network::templates::index::Templates;
use crate::pages::network::templates::detail::TemplateDetail;
use crate::pages::network::templates::create::TemplateCreate;
use crate::pages::network::listings::index::Listings;
use crate::pages::network::listings::create::ListingCreate;
use crate::pages::network::listings::detail::ListingDetail;
use crate::pages::admin::users::PlatformAdmins;
use crate::pages::apps::instance::AppInstance;
use crate::pages::analytics::index::Analytics;
use crate::pages::verification::index::Verification;
use crate::pages::admin::ai_tasks::AiTasks;
use crate::pages::admin::integrations::Integrations;
use crate::pages::admin::compliance::Compliance;
use crate::pages::flags::index::FeatureFlags;
use crate::pages::support::index::SupportQueue;
use crate::api::auth::validate_session;
use crate::api::models::{UserInfo, PlatformAppModel};
use crate::api::networks::get_networks;
use crate::api::version::get_version;

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
    let _session_check = leptos::task::spawn_local(async move {
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
        <crate::components::omnibar::Omnibar />
        <Router>
            <Routes fallback=|| "Not found.">
                <Route path=path!("/login") view=Login />
                <Route path=path!("/verify-token/:token") view=VerifyToken />
                <Route path=path!("/magic-login") view=crate::pages::auth::magic_login::MagicLogin />
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
    let _active_network = use_context::<ReadSignal<Option<uuid::Uuid>>>().expect("active network");
    let set_active_network = use_context::<WriteSignal<Option<uuid::Uuid>>>().expect("set active network");
    let navigate = leptos_router::hooks::use_navigate();
    let location = leptos_router::hooks::use_location();
    let (show_profile_menu, set_show_profile_menu) = signal(false);

    let version_res = LocalResource::new(|| async move {
        get_version().await.unwrap_or_default()
    });

    Effect::new(move |_| {
        if user.get().is_none() && auth_checked.get() {
            navigate("/login", Default::default());
        }
    });

    // Derive active nav state from the current path
    let current_path = Signal::derive(move || location.pathname.get());


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
                <aside class="fixed left-0 top-16 bottom-0 w-64 flex flex-col py-4 px-3 bg-[#06122d] overflow-y-auto">
                    <div class="px-3 mb-4 flex items-center gap-3">
                        <div class="w-8 h-8 rounded-lg bg-primary-container flex items-center justify-center">
                            <span class="material-symbols-outlined text-primary text-sm">"terminal"</span>
                        </div>
                        <div>
                            <div class="text-on-surface font-bold text-xs">"Admin Console"</div>
                            <div class="text-on-surface-variant text-[9px] uppercase tracking-widest">"Network Root"</div>
                        </div>
                    </div>
                    <nav class="flex-1 space-y-1">
                        <div class="px-3 text-[9px] font-bold text-on-surface-variant/60 uppercase tracking-widest mb-1.5 mt-2">"Overview"</div>
                        <a href="/" class=move || side_active_class("/")>
                            <span class="material-symbols-outlined text-sm">"grid_view"</span>
                            <span class="text-xs">"Command Center"</span>
                        </a>
                        <a href="/analytics" class=move || side_active_class("/analytics")>
                            <span class="material-symbols-outlined text-sm">"analytics"</span>
                            <span class="text-xs">"Analytics"</span>
                        </a>
                        <a href="/map" class=move || side_active_class("/map")>
                            <span class="material-symbols-outlined text-sm">"map"</span>
                            <span class="text-xs">"Platform Map"</span>
                        </a>

                        <div class="px-3 text-[9px] font-bold text-on-surface-variant/60 uppercase tracking-widest mb-1.5 mt-4">"CRM"</div>
                        <a href="/crm" class=move || side_active_class("/crm")>
                            <span class="material-symbols-outlined text-sm">"person"</span>
                            <span class="text-xs">"Leads"</span>
                            <span class="ml-auto text-[9px] font-bold bg-amber-500/10 text-amber-400 border border-amber-500/20 px-1.5 py-0.5 rounded-full">"6"</span>
                        </a>
                        <a href="/crm" class=move || side_active_class("/crm")>
                            <span class="material-symbols-outlined text-sm">"domain"</span>
                            <span class="text-xs">"Accounts"</span>
                        </a>
                        <a href="/crm" class=move || side_active_class("/crm")>
                            <span class="material-symbols-outlined text-sm">"contacts"</span>
                            <span class="text-xs">"Contacts"</span>
                        </a>
                        <a href="/crm" class=move || side_active_class("/crm")>
                            <span class="material-symbols-outlined text-sm">"radio_button_checked"</span>
                            <span class="text-xs">"Opportunities"</span>
                        </a>

                        <div class="px-3 text-[9px] font-bold text-on-surface-variant/60 uppercase tracking-widest mb-1.5 mt-4">"Platform"</div>
                        <a href="/apps" class=move || side_active_class("/apps")>
                            <span class="material-symbols-outlined text-sm">"dns"</span>
                            <span class="text-xs">"Tenants"</span>
                        </a>
                        <a href="/billing" class=move || side_active_class("/billing")>
                            <span class="material-symbols-outlined text-sm">"payments"</span>
                            <span class="text-xs">"Billing"</span>
                        </a>
                        <a href="/billing/products" class=move || side_active_class("/billing/products")>
                            <span class="material-symbols-outlined text-sm">"sell"</span>
                            <span class="text-xs">"Products & Plans"</span>
                        </a>
                        <a href="/products" class=move || side_active_class("/products")>
                            <span class="material-symbols-outlined text-sm">"store"</span>
                            <span class="text-xs">"Storefront Pages"</span>
                        </a>
                        <a href="/network" class=move || side_active_class("/network")>
                            <span class="material-symbols-outlined text-sm">"lan"</span>
                            <span class="text-xs">"Network Instances"</span>
                        </a>
                        <a href="/network/syndication" class=move || side_active_class("/network/syndication")>
                            <span class="material-symbols-outlined text-sm">"sync_alt"</span>
                            <span class="text-xs">"Syndication"</span>
                        </a>
                        <a href="/verification" class=move || side_active_class("/verification")>
                            <span class="material-symbols-outlined text-sm">"verified_user"</span>
                            <span class="text-xs">"Verification"</span>
                            <span class="ml-auto text-[9px] font-bold bg-error-container/20 text-error border border-error/20 px-1.5 py-0.5 rounded-full">"3"</span>
                        </a>
                        <a href="/billing/scorecards" class=move || side_active_class("/billing/scorecards")>
                            <span class="material-symbols-outlined text-sm">"star"</span>
                            <span class="text-xs">"Scorecards"</span>
                        </a>

                        <div class="px-3 text-[9px] font-bold text-on-surface-variant/60 uppercase tracking-widest mb-1.5 mt-4">"Operations"</div>
                        <a href="/flags" class=move || side_active_class("/flags")>
                            <span class="material-symbols-outlined text-sm">"flag"</span>
                            <span class="text-xs">"Feature Flags"</span>
                            <span class="ml-auto text-[9px] font-bold bg-amber-500/10 text-amber-400 border border-amber-500/20 px-1.5 py-0.5 rounded-full">"1"</span>
                        </a>
                        <a href="/support" class=move || side_active_class("/support")>
                            <span class="material-symbols-outlined text-sm">"support_agent"</span>
                            <span class="text-xs">"Support Queue"</span>
                            <span class="ml-auto text-[9px] font-bold bg-error-container/20 text-error border border-error/20 px-1.5 py-0.5 rounded-full">"4"</span>
                        </a>
                        <a href="/logs" class=move || side_active_class("/logs")>
                            <span class="material-symbols-outlined text-sm">"history"</span>
                            <span class="text-xs">"Audit Logs"</span>
                        </a>
                        <a href="/admin/aitasks" class=move || side_active_class("/admin/aitasks")>
                            <span class="material-symbols-outlined text-sm">"smart_toy"</span>
                            <span class="text-xs">"AI Task Monitor"</span>
                        </a>

                        <div class="px-3 text-[9px] font-bold text-on-surface-variant/60 uppercase tracking-widest mb-1.5 mt-4">"Admin"</div>
                        <a href="/admins" class=move || side_active_class("/admins")>
                            <span class="material-symbols-outlined text-sm">"group"</span>
                            <span class="text-xs">"User Access & Auth"</span>
                        </a>
                        <a href="/admin/integrations" class=move || side_active_class("/admin/integrations")>
                            <span class="material-symbols-outlined text-sm">"integration_instructions"</span>
                            <span class="text-xs">"Integrations & Webhooks"</span>
                        </a>
                        <a href="/admin/compliance" class=move || side_active_class("/admin/compliance")>
                            <span class="material-symbols-outlined text-sm">"gavel"</span>
                            <span class="text-xs">"Contracts & Compliance"</span>
                        </a>
                    </nav>

                    // ── Sidebar Footer: version + utility links ──
                    <div class="border-t border-outline-variant/10 pt-4 mt-4 space-y-1">
                        // Version chip — fetched once on mount
                        <Suspense fallback=|| ()>
                            {move || version_res.get().map(|v| {
                                let env_label = v.environment.clone();
                                let env_color = match env_label.as_str() {
                                    "prod" => "text-emerald-400 bg-emerald-400/10 border-emerald-400/20",
                                    "uat"  => "text-amber-400 bg-amber-400/10 border-amber-400/20",
                                    _      => "text-sky-400 bg-sky-400/10 border-sky-400/20",
                                };
                                view! {
                                    <div class="mx-3 mb-3 px-3 py-2 rounded-lg bg-surface-container-high/60 border border-outline-variant/10 space-y-1.5">
                                        // Environment badge
                                        <div class=format!("inline-flex items-center gap-1 px-2 py-0.5 rounded-full border text-[9px] font-bold uppercase tracking-widest {}", env_color)>
                                            <span class="material-symbols-outlined text-[10px]">
                                                {if env_label == "prod" { "verified" } else { "science" }}
                                            </span>
                                            {env_label.to_uppercase()}
                                        </div>
                                        // Version + SHA
                                        <div class="flex items-center justify-between">
                                            <div class="flex items-center gap-1.5">
                                                <span class="material-symbols-outlined text-[13px] text-primary/60">"commit"</span>
                                                <span class="text-[10px] font-mono text-on-surface-variant">"v"</span>
                                                <span class="text-[10px] font-bold font-mono text-on-surface">{v.version.clone()}</span>
                                            </div>
                                            <span class="text-[9px] font-mono text-on-surface-variant/60 truncate max-w-[60px]" title=v.build_sha.clone()>
                                                {v.build_sha.chars().take(7).collect::<String>()}
                                            </span>
                                        </div>
                                    </div>
                                }
                            })}
                        </Suspense>
                        <a href="/settings" class=move || side_active_class("/settings")>
                            <span class="material-symbols-outlined text-sm">"settings"</span>
                            <span>"My Profile & Settings"</span>
                        </a>
                        <a href="/apps/new" class="block w-full mt-4">
                            <button class="w-full btn-primary-gradient text-on-primary-container py-2.5 rounded-md text-xs font-bold uppercase tracking-widest shadow-lg shadow-primary/10 hover:opacity-90 transition-opacity">
                                "New Application"
                            </button>
                        </a>
                    </div>
                </aside>

                // ── Main Content ──
                <main class="ml-64 mt-16 p-8 h-[calc(100vh-64px)] overflow-y-auto bg-surface-container">
                    <Routes fallback=|| "Not found.">
                        <Route path=path!("/") view=Dashboard />
                        <Route path=path!("/analytics") view=Analytics />
                        <Route path=path!("/map") view=crate::pages::map::index::PlatformMap />
                        <Route path=path!("/apps") view=Apps />
                        <Route path=path!("/apps/new") view=crate::pages::apps::create::AppCreate />
                        <Route path=path!("/apps/:id") view=crate::pages::apps::detail::AppDashboard />
                        <Route path=path!("/apps/:id/instance") view=AppInstance />
                        <Route path=path!("/network") view=NetworkRegistry />
                        <Route path=path!("/network/new") view=NetworkCreate />
                        <Route path=path!("/network/:id") view=NetworkDetail />
                        <Route path=path!("/network/syndication") view=SyndicationManager />
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
                        <Route path=path!("/crm/new") view=crate::pages::crm::create::CrmCreate />
                        <Route path=path!("/crm/:entity/:id") view=crate::pages::crm::detail::CrmDetail />
                        <Route path=path!("/products") view=PlatformProducts />
                        <Route path=path!("/products/:id") view=ProductDetail />
                        <Route path=path!("/admins") view=PlatformAdmins />
                        <Route path=path!("/billing") view=crate::pages::billing::dashboard::BillingDashboard />
                        <Route path=path!("/billing/tenant/:id") view=crate::pages::billing::tenant::TenantLedger />
                        <Route path=path!("/billing/products") view=crate::pages::billing::products::BillingProducts />
                        <Route path=path!("/billing/scorecards") view=Scorecards />
                        <Route path=path!("/billing/scorecards/session") view=ScorecardSession />
                        <Route path=path!("/verification") view=Verification />
                        <Route path=path!("/developer") view=crate::pages::admin::developer::DeveloperConsole />
                        <Route path=path!("/settings") view=crate::pages::admin::profile::Settings />
                        <Route path=path!("/logs") view=crate::pages::logs::index::AuditLogs />
                        <Route path=path!("/admin/aitasks") view=AiTasks />
                        <Route path=path!("/admin/integrations") view=Integrations />
                        <Route path=path!("/admin/compliance") view=Compliance />
                        <Route path=path!("/flags") view=FeatureFlags />
                        <Route path=path!("/support") view=SupportQueue />
                        <Route path=path!("/marketing") view=MarketingLanding />
                    </Routes>
                </main>
            </div>
        </Show>
    }
}
