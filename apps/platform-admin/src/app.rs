// FIXME(platform-admin): Migrate to the `AtlasApp` trait defined at
// `backend/src/traits/atlas_app.rs`. Routes, SeaORM migrations, and
// background jobs are currently hardcoded globally. See
// `docs/atlas_app_integration.md` for the full protocol.
use leptos::prelude::*;
use leptos_router::components::{Router, Route, Routes};
use leptos_router::path;

use crate::pages::dashboard::Dashboard;
use crate::pages::apps::index::Apps;
use crate::pages::crm::leads::LeadsPage;
use crate::pages::crm::contacts::ContactsPage;
use crate::pages::crm::accounts::AccountsPage;
use crate::pages::crm::opportunities::OpportunitiesPage;
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
use crate::components::intel_sidebar::IntelSidebar;
use crate::pages::syndication::offers::SyndicationOffers;
use crate::pages::syndication::links::SyndicationLinks;


#[derive(Clone, Debug)]
pub struct ToastPayload {
    pub title: String,
    pub message: String,
    pub toast_type: String, // "success" | "error" | "info" | "warning"
}

#[derive(Copy, Clone, Debug)]
pub struct GlobalToast {
    payload: RwSignal<Option<ToastPayload>>,
}

impl GlobalToast {
    pub fn show_toast(&self, title: &str, msg: &str, toast_type: &str) {
        self.payload.set(Some(ToastPayload {
            title: title.to_string(),
            message: msg.to_string(),
            toast_type: toast_type.to_string(),
        }));
    }

    pub fn dismiss(&self) {
        self.payload.set(None);
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

    let toast = GlobalToast { payload: RwSignal::new(None) };
    provide_context(toast);

    // Validate session on load
    let _session_check = leptos::task::spawn_local(async move {
        if let Ok(valid_user) = validate_session().await {
            set_user.set(Some(valid_user));
        }
        set_auth_checked.set(true);
    });

    view! {
        <shared_ui::components::version_banner::VersionBanner api_base="/" />
        <div class="fixed bottom-4 right-4 z-[9999] pointer-events-none flex flex-col gap-2">
            {move || toast.payload.get().map(|p| {
                let border_color = match p.toast_type.as_str() {
                    "success" => "#22c55e",
                    "error"   => "#ef4444",
                    "warning" => "#f59e0b",
                    _         => "#6366f1", // info / default
                };
                let icon = match p.toast_type.as_str() {
                    "success" => "check_circle",
                    "error"   => "error",
                    "warning" => "warning",
                    _         => "info",
                };
                view! {
                    <div class="glass-panel text-on-surface rounded-xl flex items-start gap-3 min-w-[300px] max-w-[420px] pointer-events-auto border border-outline-variant/40 overflow-hidden"
                        style=format!("border-left: 3px solid {};", border_color)>
                        <span class="material-symbols-outlined text-[18px] mt-3 ml-3 shrink-0"
                            style=format!("color: {};", border_color)>{icon}</span>
                        <div class="flex-1 py-3 pr-2">
                            <div class="text-xs font-bold text-on-surface">{p.title.clone()}</div>
                            <div class="text-sm text-on-surface-variant mt-0.5">{p.message.clone()}</div>
                        </div>
                        <button class="mt-2 mr-2 hover:opacity-70 font-bold text-on-surface-variant text-lg shrink-0"
                            on:click=move |_| toast.dismiss()>"✕"</button>
                    </div>
                }
            })}
        </div>
        <crate::components::omnibar::Omnibar />
        <ErrorBoundary fallback=|errors| view! {
            <div style="display:flex;flex-direction:column;align-items:center;justify-content:center;height:100vh;gap:16px;font-family:system-ui;color:#dee5ff;background:#020f2e;">
                <span style="font-size:48px;">{"⚠️"}</span>
                <h1 style="font-size:20px;font-weight:700;margin:0;">"Something went wrong"</h1>
                <p style="font-size:13px;color:#91aaeb;margin:0;max-width:360px;text-align:center;">
                    "An unexpected error occurred. Please reload the page. If this keeps happening, contact support."
                </p>
                <p style="font-size:11px;font-family:monospace;color:#5b7ab0;max-width:480px;text-align:center;">
                    {move || errors.get().into_iter().map(|(_, e)| e.to_string()).collect::<Vec<_>>().join(", ")}
                </p>
                <button
                    style="margin-top:8px;padding:10px 24px;background:#1a3c8f;border:1px solid #2a5ccc;border-radius:8px;color:#dee5ff;font-size:13px;font-weight:600;cursor:pointer;"
                    on:click=|_| { let _ = web_sys::window().unwrap().location().reload(); }
                >
                    "Reload Page"
                </button>
            </div>
        }>
            <Router>
                <Routes fallback=|| "Not found.">
                    <Route path=path!("/login") view=Login />
                    <Route path=path!("/verify-token/:token") view=VerifyToken />
                    <Route path=path!("/magic-login") view=crate::pages::auth::magic_login::MagicLogin />
                    <Route path=path!("/setup") view=Setup />
                    <Route path=path!("/*any") view=AuthenticatedLayout />
                </Routes>
            </Router>
        </ErrorBoundary>
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
        let active = (path == "/" && p == "/") || (path != "/" && p.starts_with(path));
        if active {
            "nav-item active"
        } else {
            "nav-item"
        }
    };

    let show_intel_sidebar = Signal::derive(move || {
        let p = current_path.get();
        p == "/" || p == "/dashboard"
    });

    let shell_style = move || {
        if show_intel_sidebar.get() {
            "display: grid; grid-template-columns: 220px 1fr 280px; grid-template-rows: 48px 1fr; height: 100vh;"
        } else {
            "display: grid; grid-template-columns: 220px 1fr; grid-template-rows: 48px 1fr; height: 100vh;"
        }
    };

    view! {
        <Show when=move || user.get().is_some() fallback=move || view! {
            <div class="h-screen w-full flex items-center justify-center bg-surface text-on-surface-variant font-sans antialiased">
                <div>"Checking session..."</div>
            </div>
        }>
            <div class="shell" style=shell_style>
                // ── Top Nav Bar ──
                <header class="topbar">
                    <div class="topbar-logo">
                        <div class="mark">"A"</div>
                        <span class="wordmark">"Atlas Platform"</span>
                        <span class="badge">"Super-Admin"</span>
                    </div>
                    <div class="topbar-center">
                        <div class="search-wrap">
                            <input 
                                type="text"
                                placeholder="Search tenants, leads, products… ⌘K"
                            />
                            <span class="kbd">"⌘K"</span>
                        </div>
                    </div>
                    <div class="topbar-right">
                        // Site selector
                        <select
                            class="bg-[#1C2236] border border-outline-variant/30 text-on-surface text-xs rounded px-2.5 py-1 focus:ring-1 focus:ring-primary focus:border-primary text-on-surface max-w-[140px] select-none"
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
                        // Notification → Audit Ledger
                        <a href="/logs" class="icon-btn" title="Audit Logs">
                            <svg viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.5">
                                <path d="M8 2a4 4 0 0 0-4 4v2.5L2.5 10h11L12 8.5V6a4 4 0 0 0-4-4z"/><circle cx="8" cy="13" r="1.2"/>
                            </svg>
                        </a>
                        // Activity → Audit Ledger
                        <a href="/logs" class="icon-btn" title="Activity Log">
                            <svg viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.5">
                                <polyline points="3,9 6,5 9,7 13,3"/><line x1="3" y1="13" x2="13" y2="13"/>
                            </svg>
                        </a>
                        // Docs → Developer Console
                        <a href="/developer" class="icon-btn" title="Developer Console">
                            <svg viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.5">
                                <rect x="3" y="2" width="10" height="12" rx="1"/><line x1="5" y1="5" x2="11" y2="5"/><line x1="5" y1="8" x2="9" y2="8"/>
                            </svg>
                        </a>
                        <div class="topbar-divider"></div>
                        <div class="avatar-btn" on:click=move |e| { e.stop_propagation(); set_show_profile_menu.update(|v| *v = !*v) }>
                            {move || user.get().map(|u| format!("{}{}", u.first_name.chars().next().unwrap_or('J'), u.last_name.chars().next().unwrap_or('D'))).unwrap_or_else(|| "JD".to_string())}
                        </div>
                        <Show when=move || show_profile_menu.get()>
                            <div class="absolute right-4 top-11 mt-1 w-48 bg-[#1C2236] border border-outline-variant/40 rounded-lg py-1 z-[100] overflow-hidden shadow-2xl">
                                <div class="px-4 py-3 border-b border-outline-variant/20 text-sm">
                                    <p class="font-medium text-on-surface">{move || user.get().map(|u| format!("{} {}", u.first_name, u.last_name)).unwrap_or_else(|| "Admin User".to_string())}</p>
                                    <p class="text-on-surface-variant text-xs truncate">{move || user.get().map(|u| u.email.clone()).unwrap_or_else(|| "admin@foundry.local".to_string())}</p>
                                </div>
                                <a href="/settings" class="block w-full text-left px-4 py-2.5 text-sm text-on-surface hover:bg-[#111520] transition-colors" on:click=move |_| set_show_profile_menu.set(false)>"Account Settings"</a>
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
                </header>

                // ── Side Nav Bar ──
                <aside class="sidebar">
                    <span class="nav-label nav-section-label">"Overview"</span>
                    <a href="/" class=move || side_active_class("/")>
                        <svg viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.5"><rect x="2" y="2" width="5" height="5" rx="0.5"/><rect x="2" y="9" width="5" height="5" rx="0.5"/><rect x="9" y="2" width="5" height="5" rx="0.5"/><rect x="9" y="9" width="5" height="5" rx="0.5"/></svg>
                        "Command Center"
                    </a>
                    <a href="/analytics" class=move || side_active_class("/analytics")>
                        <svg viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.5"><polyline points="2,12 5,7 8,9 11,4 14,6"/></svg>
                        "Analytics"
                    </a>
                    <a href="/map" class=move || side_active_class("/map")>
                        <svg viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.5"><polygon points="1.5,3 5,1.5 11,3.5 14.5,2 14.5,13 11,14.5 5,12.5 1.5,14"/><line x1="5" y1="1.5" x2="5" y2="12.5"/><line x1="11" y1="3.5" x2="11" y2="14.5"/></svg>
                        "Platform Map"
                    </a>

                    <span class="nav-label nav-section-label">"CRM"</span>
                    <a href="/leads" class=move || side_active_class("/lead")>
                        <svg viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.5"><circle cx="6" cy="5" r="2.5"/><path d="M1 13c0-2.8 2.2-5 5-5h0a5 5 0 0 1 5 5"/></svg>
                        "Leads"
                    </a>
                    <a href="/accounts" class=move || side_active_class("/account")>
                        <svg viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.5"><rect x="2" y="4" width="12" height="9" rx="1"/><path d="M6 13V9h4v4"/></svg>
                        "Accounts"
                    </a>
                    <a href="/contacts" class=move || side_active_class("/contact")>
                        <svg viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.5"><circle cx="8" cy="6" r="3"/><path d="M2 13c0-3.3 2.7-6 6-6s6 2.7 6 6"/></svg>
                        "Contacts"
                    </a>
                    <a href="/pipeline" class=move || {
                        let p = current_path.get();
                        if p.starts_with("/pipeline") { "nav-item active" } else { "nav-item" }
                    }>
                        <svg viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.5"><circle cx="8" cy="8" r="5.5"/><path d="M8 5v3.5l2 2"/></svg>
                        "Pipeline"
                    </a>

                    <span class="nav-label nav-section-label">"Platform"</span>
                    <a href="/apps" class=move || {
                        // Exact match only — /apps/:id (app instance pages) should NOT
                        // highlight the Tenants nav item. Only /apps itself and /apps/create
                        // (the provisioning wizard) belong to this nav section.
                        let p = current_path.get();
                        let active = p == "/apps" || p == "/apps/create" || p == "/apps/new";
                        if active { "nav-item active" } else { "nav-item" }
                    }>
                        <svg viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.5"><rect x="2" y="7" width="12" height="7" rx="1"/><path d="M5 7V5a3 3 0 0 1 6 0v2"/></svg>
                        "Tenants"
                    </a>
                    <a href="/billing" class=move || side_active_class("/billing")>
                        <svg viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.5"><rect x="2" y="5" width="12" height="8" rx="1"/><line x1="2" y1="9" x2="14" y2="9"/></svg>
                        "Billing"
                    </a>
                    <a href="/billing/products" class=move || side_active_class("/billing/products")>
                        <svg viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.5"><path d="M14 5.5l-6-3.5L2 5.5v5l6 3.5 6-3.5v-5z"/><line x1="8" y1="2" x2="8" y2="14"/><line x1="2" y1="5.5" x2="14" y2="5.5"/></svg>
                        "Products & Plans"
                    </a>
                    <a href="/products" class=move || side_active_class("/products")>
                        <svg viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.5"><rect x="2" y="2" width="12" height="12" rx="1.5"/><line x1="2" y1="6" x2="14" y2="6"/><line x1="6" y1="6" x2="6" y2="14"/></svg>
                        "Storefront Pages"
                    </a>
                    <a href="/network" class=move || side_active_class("/network")>
                        <svg viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.5"><circle cx="8" cy="8" r="6"/><path d="M1 8h14M8 1a12 12 0 0 1 0 14A12 12 0 0 1 8 1"/></svg>
                        "Network Instances"
                    </a>
                    <a href="/network/syndication" class=move || side_active_class("/network/syndication")>
                        <svg viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.5"><path d="M13.5 4.5l-2-2m2 2l-2 2m2-2H2.5v4m-1 3.5l2 2m-2-2l2-2m-2 2h11v-4"/></svg>
                        "Syndication"
                    </a>
                    <a href="/syndication/offers" class=move || side_active_class("/syndication")>
                        <svg viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.5"><path d="M2 8h12M10 5l3 3-3 3M8 2v12"/></svg>
                        "Offer Catalog"
                    </a>
                    <a href="/verification" class=move || side_active_class("/verification")>
                        <svg viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.5"><path d="M8 2l5 2v4c0 3-2 5.5-5 6.5C5 13.5 3 11 3 8V4l5-2z"/></svg>
                        "Verification"
                    </a>
                    <a href="/billing/scorecards" class=move || side_active_class("/billing/scorecards")>
                        <svg viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.5"><path d="M8 2l1.5 3h3l-2.5 2 1 3L8 8.5 5 10l1-3L3.5 5h3z"/></svg>
                        "Scorecards"
                    </a>

                    <span class="nav-label nav-section-label">"Operations"</span>
                    <a href="/flags" class=move || side_active_class("/flags")>
                        <svg viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.5"><path d="M3 2v12M3 2h8l-2 3.5L11 9H3"/></svg>
                        "Feature Flags"
                    </a>
                    <a href="/support" class=move || side_active_class("/support")>
                        <svg viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.5"><path d="M8 2l5 2v4c0 3-2 5.5-5 6.5C5 13.5 3 11 3 8V4l5-2z"/></svg>
                        "Support Queue"
                    </a>
                    <a href="/logs" class=move || side_active_class("/logs")>
                        <svg viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.5"><rect x="3" y="2" width="10" height="12" rx="1.5"/><line x1="6" y1="5" x2="10" y2="5"/><line x1="6" y1="8" x2="10" y2="8"/><line x1="6" y1="11" x2="9" y2="11"/></svg>
                        "Audit Logs"
                    </a>
                    <a href="/admin/aitasks" class=move || side_active_class("/admin/aitasks")>
                        <svg viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.5"><polyline points="2,12 5,7 8,9 11,4 14,6"/></svg>
                        "AI Task Monitor"
                    </a>

                    <span class="nav-label nav-section-label">"Admin"</span>
                    <a href="/admins" class=move || side_active_class("/admins")>
                        <svg viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.5"><circle cx="5.5" cy="5" r="2"/><circle cx="10.5" cy="5" r="2"/><path d="M1 13c0-2.5 2-4.5 4.5-4.5"/><path d="M15 13c0-2.5-2-4.5-4.5-4.5"/><path d="M5 13c0-3 1.5-5 3-5s3 2 3 5"/></svg>
                        "User Access & Auth"
                    </a>
                    <a href="/admin/integrations" class=move || side_active_class("/admin/integrations")>
                        <svg viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.5"><circle cx="8" cy="8" r="2"/><path d="M8 2v1M8 13v1M2 8h1M13 8h1M3.5 3.5l.7.7M11.8 11.8l.7.7M3.5 12.5l.7-.7M11.8 4.2l.7-.7"/></svg>
                        "Integrations & Webhooks"
                    </a>
                    <a href="/admin/compliance" class=move || side_active_class("/admin/compliance")>
                        <svg viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.5"><path d="M8 2l5 2v4c0 3-2 5.5-5 6.5C5 13.5 3 11 3 8V4l5-2z"/></svg>
                        "Contracts & Compliance"
                    </a>

                    // ── Sidebar Footer ──
                    <div class="sidebar-footer">
                        <a href="/settings" class="nav-item">
                            <svg viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.5"><circle cx="8" cy="8" r="2"/><path d="M8 2v1M8 13v1M2 8h1M13 8h1M3.5 3.5l.7.7M11.8 11.8l.7.7M3.5 12.5l.7-.7M11.8 4.2l.7-.7"/></svg>
                            "My Profile & Settings"
                        </a>
                        <div class="mx-3 mt-2 py-1 border-t border-outline-variant/10 text-[10px] flex items-center justify-between text-on-surface-variant font-mono">
                            <span>"v1.0.0"</span>
                            <Suspense fallback=|| ()>
                                {move || version_res.get().map(|v| view! {
                                    <span class="opacity-60">{v.build_sha.chars().take(7).collect::<String>()}</span>
                                })}
                            </Suspense>
                        </div>
                    </div>
                </aside>

                // ── Main Content ──
                <main class="main-content-layout">
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
                        <Route path=path!("/leads")        view=LeadsPage />
                        <Route path=path!("/leads/:id")   view=crate::pages::crm::detail::CrmDetail />
                        <Route path=path!("/contacts")     view=ContactsPage />
                        <Route path=path!("/contacts/:id") view=crate::pages::crm::detail::CrmDetail />
                        <Route path=path!("/accounts")     view=AccountsPage />
                        <Route path=path!("/accounts/:id") view=crate::pages::crm::detail::CrmDetail />
                        <Route path=path!("/pipeline")     view=OpportunitiesPage />
                        <Route path=path!("/pipeline/:id") view=crate::pages::crm::detail::CrmDetail />
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
                        <Route path=path!("/admin/security") view=crate::pages::admin::security::SecurityPasskeys />
                        <Route path=path!("/flags") view=FeatureFlags />
                        <Route path=path!("/support") view=SupportQueue />
                        <Route path=path!("/marketing") view=MarketingLanding />
                        // Syndication Offer Catalog (platform admin)
                        <Route path=path!("/syndication/offers") view=SyndicationOffers />
                        <Route path=path!("/syndication/links") view=SyndicationLinks />
                    </Routes>
                </main>

                // ── Right Intelligence Sidebar ──
                <Show when=move || show_intel_sidebar.get()>
                    <IntelSidebar />
                </Show>
            </div>
        </Show>
    }
}
