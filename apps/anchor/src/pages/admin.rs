// Build: ship Leptos hydration fix; pipeline now uses CI_COMMIT_CHANGED_FILES.
pub mod leads;
pub mod contacts;

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq)]
pub struct MailingListRecord {
    pub id: i32,
    pub email: String,
    pub list_type: String,
    pub preferences: String,
    pub created_at: String,
}

use leptos::prelude::*;
use leptos_router::hooks::use_query_map;
use shared_ui::components::auth::atlas_login_panel::AtlasLoginPanel;
use shared_ui::components::auth::passkey_nudge::PasskeyNudge;
use shared_ui::auth::atlas_auth::check_has_passkey;
use shared_ui::utils::ResourceState;
use shared_ui::components::admin_module_sidebar::{
    AdminModuleConfig, AdminModuleType, SidebarTheme,
};

use crate::auth::*;
use crate::components::admin_modal::*;

/// Proxies a session-revocation request to the Atlas backend and clears the HttpOnly cookie.
/// Must be a server function — `reqwest` and `atlas_client` are SSR-only and cannot be called
/// from WASM directly.
#[server(RevokeSession, "/api")]
pub async fn revoke_session() -> Result<(), ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        use leptos_axum::ResponseOptions;
        // Clear the HttpOnly session cookie immediately on the response.
        let response = expect_context::<ResponseOptions>();
        response.append_header(
            axum::http::header::SET_COOKIE,
            axum::http::HeaderValue::from_static(
                "session=; HttpOnly; Path=/; SameSite=Strict; Max-Age=0",
            ),
        );
        // Best-effort call to the backend to deactivate the session record.
        let url = format!(
            "{}/api/auth/session/revoke",
            crate::atlas_client::get_atlas_api_url()
        );
        let _ = reqwest::Client::new().post(&url).send().await;
    }
    Ok(())
}

/// Fetches the enabled admin module set for the authenticated tenant.
/// Sorted by `sort_order` ascending. Returns `Vec<AdminModuleConfig>`.
/// Falls back to an empty vec on error — the sidebar will show nothing (safe failure mode).
///
/// Uses the same AppState/TenantContext injection pattern as all anchor server fns.
#[server(GetAdminModules, "/api")]
pub async fn get_admin_modules() -> Result<Vec<AdminModuleConfig>, ServerFnError> {
    use axum::Extension;
    use leptos_axum::extract;
    use shared_ui::components::admin_module_sidebar::ModuleCategory;

    // Local helper: parse SCREAMING_SNAKE_CASE string → AdminModuleType.
    fn parse_module(s: &str) -> Option<AdminModuleType> {
        match s {
            "DASHBOARD"       => Some(AdminModuleType::Dashboard),
            "SETTINGS"        => Some(AdminModuleType::Settings),
            "SECURITY"        => Some(AdminModuleType::Security),
            "BLOG"            => Some(AdminModuleType::Blog),
            "RESUME_PROFILES" => Some(AdminModuleType::ResumeProfiles),
            "RESUME_ENTRIES"  => Some(AdminModuleType::ResumeEntries),
            "LANDING_PAGES"   => Some(AdminModuleType::LandingPages),
            "WEBFORMS"        => Some(AdminModuleType::Webforms),
            "NAVIGATION"      => Some(AdminModuleType::Navigation),
            "FOOTER"          => Some(AdminModuleType::Footer),
            "PAGE_HEADERS"    => Some(AdminModuleType::PageHeaders),
            "LEADS"           => Some(AdminModuleType::Leads),
            "CONTACTS"        => Some(AdminModuleType::Contacts),
            "LEAD_OPTIONS"    => Some(AdminModuleType::LeadOptions),
            "SERVICES"        => Some(AdminModuleType::Services),
            "CASE_STUDIES"    => Some(AdminModuleType::CaseStudies),
            "HIGHLIGHTS"      => Some(AdminModuleType::Highlights),
            "PROPERTIES"      => Some(AdminModuleType::Properties),
            "LISTINGS"        => Some(AdminModuleType::Listings),
            // IMPORTANT: Unknown strings are dropped (None), not mapped to Custom.
            // This prevents future backend variants or DB typos from creating
            // phantom tabs in the sidebar. Matches the backend's own filter_map/.ok()?.
            _                 => None,
        }
    }

    let Extension(state) = extract::<Extension<crate::state::AppState>>().await?;
    let Extension(tenant) = extract::<Extension<crate::state::TenantContext>>().await?;

    // Resolve the app_instance for this tenant.
    let app_instance_id: Option<uuid::Uuid> = sqlx::query_scalar(
        "SELECT ai.id FROM app_instances ai WHERE ai.tenant_id = $1 LIMIT 1"
    )
    .bind(tenant.0)
    .fetch_optional(&state.pool)
    .await
    .map_err(|e| ServerFnError::new(e.to_string()))?;

    let Some(instance_id) = app_instance_id else {
        return Ok(vec![]);
    };

    let rows = sqlx::query_as::<_, (String, String, Option<String>, i32, bool)>(
        "SELECT module_type, display_name, icon, sort_order, is_fixed \
         FROM app_instance_module \
         WHERE app_instance_id = $1 AND is_enabled = true \
         ORDER BY sort_order ASC"
    )
    .bind(instance_id)
    .fetch_all(&state.pool)
    .await
    .map_err(|e| ServerFnError::new(e.to_string()))?;

    let configs = rows
        .into_iter()
        .filter_map(|(type_str, display_name, icon, sort_order, is_fixed)| {
            let module_type = parse_module(&type_str)?;
            // Derive category from module_type via a local lookup table.
            let category = match module_type {
                AdminModuleType::Dashboard
                | AdminModuleType::Settings
                | AdminModuleType::Security => ModuleCategory::Platform,
                AdminModuleType::Blog
                | AdminModuleType::ResumeProfiles
                | AdminModuleType::ResumeEntries
                | AdminModuleType::LandingPages
                | AdminModuleType::Webforms => ModuleCategory::Content,
                AdminModuleType::Navigation
                | AdminModuleType::Footer
                | AdminModuleType::PageHeaders => ModuleCategory::Appearance,
                AdminModuleType::Leads
                | AdminModuleType::Contacts
                | AdminModuleType::LeadOptions => ModuleCategory::CrmAndComms,
                AdminModuleType::Services
                | AdminModuleType::CaseStudies
                | AdminModuleType::Highlights => ModuleCategory::B2B,
                _ => ModuleCategory::Advanced,
            };
            Some(AdminModuleConfig {
                module_type,
                display_name,
                icon,
                sort_order,
                is_fixed,
                category,
            })
        })
        .collect();

    Ok(configs)
}

#[component]
pub fn WebformsTable() -> impl IntoView {
    view! {
        <div class="space-y-6">
            <div class="flex justify-between items-center mb-6">
                <div>
                    <h3 class="text-xl font-bold text-on-surface">"Lead Capture & Origination Schemas"</h3>
                    <p class="text-sm text-on-surface-variant">"Manage multi-step form sequences mapped into the JSON layout blocks."</p>
                </div>
            </div>

            <div class="bg-surface-container overflow-hidden border border-outline-variant/30 hidden md:block">
                <table class="w-full text-left border-collapse">
                    <thead>
                        <tr class="bg-surface-container-high border-b border-outline-variant/30 text-xs tracking-wider uppercase text-on-surface-variant jetbrains">
                            <th class="px-6 py-4 font-medium">"Form ID (Slug)"</th>
                            <th class="px-6 py-4 font-medium">"Name"</th>
                            <th class="px-6 py-4 font-medium">"Description"</th>
                            <th class="px-6 py-4 font-medium">"Integrations"</th>
                            <th class="px-6 py-4 font-medium text-right">"Actions"</th>
                        </tr>
                    </thead>
                    <tbody class="divide-y divide-outline-variant/30">
                        <tr class="hover:bg-surface-container-high/50 transition-colors">
                            <td class="px-6 py-4 jetbrains text-xs text-primary font-bold">"cre-application"</td>
                            <td class="px-6 py-4 text-sm font-medium">"Commercial Real Estate Loan"</td>
                            <td class="px-6 py-4 text-sm text-on-surface-variant truncate max-w-[200px]">"Standard CRE loan application for multifamily, retail, office, etc."</td>
                            <td class="px-6 py-4">
                                <span class="bg-primary/10 text-primary px-2 py-1 text-xs font-bold rounded">"Webhook Active"</span>
                            </td>
                            <td class="px-6 py-4 text-right">
                                <button class="text-primary hover:underline text-xs jetbrains font-bold uppercase tracking-widest mr-4">"EDIT JSON"</button>
                                <button class="text-error hover:underline text-xs jetbrains font-bold uppercase tracking-widest">"DELETE"</button>
                            </td>
                        </tr>
                        <tr class="hover:bg-surface-container-high/50 transition-colors">
                            <td class="px-6 py-4 jetbrains text-xs text-primary font-bold">"hoa-condo-application"</td>
                            <td class="px-6 py-4 text-sm font-medium">"HOA & Condominium Association Loan"</td>
                            <td class="px-6 py-4 text-sm text-on-surface-variant truncate max-w-[200px]">"Unsecured lending for condo associations to fund capital improvements."</td>
                            <td class="px-6 py-4">
                                <span class="bg-primary/10 text-primary px-2 py-1 text-xs font-bold rounded">"Webhook Active"</span>
                            </td>
                            <td class="px-6 py-4 text-right">
                                <button class="text-primary hover:underline text-xs jetbrains font-bold uppercase tracking-widest mr-4">"EDIT JSON"</button>
                                <button class="text-error hover:underline text-xs jetbrains font-bold uppercase tracking-widest">"DELETE"</button>
                            </td>
                        </tr>
                    </tbody>
                </table>
            </div>
            <div class="bg-surface-container border border-outline-variant/30 p-8 text-center mt-6">
                <span class="material-symbols-outlined text-4xl text-primary mb-4 block">"view_list"</span>
                <p class="text-on-surface-variant max-w-lg mx-auto">
                    "These schemas dynamically populate the `<FormBuilderBlock />` when mapped into App Pages."
                </p>
            </div>
        </div>
    }
}

/// Login UI — delegates entirely to the shared AtlasLoginPanel.
/// Token verification, expired-link UI, URL cleanup — all owned by the panel.
#[component]
fn LoginPanel(on_authenticated: Callback<()>) -> impl IntoView {
    view! {
        <AtlasLoginPanel
            app_title="SYSTEM_CMS"
            success_path="/admin"
            skip_reload=true
            on_authenticated=on_authenticated
        />
    }
}

#[component]
pub fn Admin() -> impl IntoView {
    // query is available for downstream components via context if needed
    let _query = use_query_map();

    let (auth_trigger, set_auth_trigger) = signal(0i32);

    // ── Auth resource ─────────────────────────────────────────────────────────
    // Resource::new_blocking blocks the SSR byte stream until check_session()
    // resolves and serialises Ok(false)/Ok(true) into the HTML payload.
    // The WASM client reads it synchronously — no None→Ready transition,
    // no Suspense DOM swap, no pre-hydration click vulnerability window.
    let auth_resource = Resource::new_blocking(
        move || auth_trigger.get(),
        |_| async move { check_session().await },
    );

    let on_auth = Callback::new(move |_: ()| {
        #[cfg(not(feature = "ssr"))]
        if let Some(w) = web_sys::window() {
            if let Ok(loc) = w.location().pathname() {
                if let Ok(history) = w.history() {
                    let _ = history.replace_state_with_url(
                        &leptos::wasm_bindgen::JsValue::NULL,
                        "",
                        Some(&loc),
                    );
                }
            }
        }
        set_auth_trigger.update(|t| *t += 1);
    });

    let (active_tab, set_active_tab) = signal(AdminModuleType::Dashboard);

    let (modal_state, set_modal_state) = signal(ModalState::None);
    provide_context(modal_state);
    provide_context(set_modal_state);

    let (refresh, set_refresh) = signal(0i32);
    provide_context(refresh);
    provide_context(set_refresh);

    view! {
        <main class="min-h-screen bg-surface-container-low text-on-surface flex flex-col pt-24 px-4 md:px-[8.5rem]">
            // Suspense is INSIDE <main> so there is always exactly one <main>
            // in the DOM — the hydration walker never sees a structural mismatch.
            <Suspense fallback=move || view! {
                <div class="flex-1 flex justify-center items-center">
                    <span class="material-symbols-outlined animate-spin text-4xl text-primary">"progress_activity"</span>
                </div>
            }>
                {move || match ResourceState::from(auth_resource.get()) {
                    ResourceState::Loading => view! {
                        <div class="hidden"></div>
                    }.into_any(),

                    // ── Unauthenticated / Error ──────────────────────────────
                    // LoginPanel renders with ZERO reactive dependencies on any
                    // LocalResource. The passkey nudge check lives in
                    // AuthenticatedDashboard only. This prevents its resolution
                    // from re-invalidating this branch and unmounting LoginPanel
                    // (which reset email_mode → Passkey tab, appearing as a
                    // page reload to the user).
                    ResourceState::Ready(false) | ResourceState::Error(_) => view! {
                        <LoginPanel on_authenticated=on_auth />
                    }.into_any(),

                    // ── Authenticated ────────────────────────────────────────
                    ResourceState::Ready(true) => view! {
                        <AuthenticatedDashboard
                            active_tab=active_tab
                            set_active_tab=set_active_tab
                        />
                    }.into_any(),
                }}
            </Suspense>
        </main>
    }
}

/// Authenticated dashboard shell, extracted so that the passkey nudge
/// LocalResource is scoped here — inside the authenticated branch only.
/// Previously it lived at the Admin component level; when it resolved,
/// the reactive write to show_passkey_nudge invalidated the entire
/// `match ResourceState` closure, destroying and recreating LoginPanel
/// and resetting email_mode to false on every post-hydration tick.
#[component]
fn AuthenticatedDashboard(
    active_tab: ReadSignal<AdminModuleType>,
    set_active_tab: WriteSignal<AdminModuleType>,
) -> impl IntoView {
    let modal_state = expect_context::<ReadSignal<ModalState>>();
    let set_modal_state = expect_context::<WriteSignal<ModalState>>();

    // Passkey nudge — client-only, safe to live here because this component
    // only mounts when the user is authenticated.
    let show_passkey_nudge = RwSignal::new(false);
    let passkey_check = LocalResource::new(move || async move {
        check_has_passkey().await.unwrap_or_else(|_| true)
    });
    Effect::new(move |_| {
        if let Some(false) = passkey_check.get() {
            show_passkey_nudge.set(true);
        }
    });

    // ── Dynamic module registry ───────────────────────────────────────────────
    // Loaded once on mount via server fn. Falls back to empty on error.
    // SSR-safe: Resource::new_blocking ensures the module list is in the HTML
    // payload so WASM reads it synchronously with no Suspense flash.
    let modules_resource = Resource::new_blocking(
        || (),
        |_| async move { get_admin_modules().await.unwrap_or_default() },
    );

    view! {
        <div class="flex-1 flex flex-col md:flex-row gap-6 md:gap-8 pb-24">
            // Sidebar — dynamic from module registry
            <aside class="w-full md:w-48 shrink-0">
                <Suspense fallback=move || view! { <div class="h-64 animate-pulse bg-surface-container-high rounded" /> }>
                    {move || {
                        match ResourceState::from_option(modules_resource.get()) {
                            ResourceState::Loading => view! { <div class="hidden"></div> }.into_any(),
                            ResourceState::Ready(modules) => {
                                let on_logout = Callback::new(move |_: ()| {
                                    leptos::task::spawn_local(async move {
                                        let _ = revoke_session().await;
                                        if let Some(w) = web_sys::window() {
                                            let _ = w.location().replace("/admin");
                                        }
                                    });
                                });
                                view! {
                                    <shared_ui::components::admin_module_sidebar::AdminModuleSidebar
                                        modules=modules
                                        active_tab=active_tab
                                        set_active_tab=set_active_tab
                                        on_logout=on_logout
                                        theme=SidebarTheme::Anchor
                                        brand_label="SYSTEM_CMS".to_string()
                                    />
                                }.into_any()
                            }
                            ResourceState::Error(_) => unreachable!(),
                        }
                    }}
                </Suspense>
            </aside>

            // Main Content Area
            <section class="flex-1 bg-surface-container-highest p-1 blueprint-overlay min-h-[600px]">
                <div class="bg-surface-container-lowest h-full p-8 md:p-12 relative flex flex-col">
                    <Show when=move || show_passkey_nudge.get()>
                        <PasskeyNudge />
                    </Show>
                    // Header
                    <div class="flex justify-between items-end border-b-2 border-outline-variant pb-6 mb-8">
                        <div>
                            <div class="inline-block bg-secondary-container/20 px-3 py-1 mb-4">
                                <span class="font-label text-[0.6875rem] text-secondary font-bold tracking-tighter">
                                    "DATABASE // LIVE"
                                </span>
                            </div>
                            <h2 class="text-3xl font-extrabold text-primary tracking-tight uppercase">
                                {move || active_tab.get().to_string()}
                            </h2>
                        </div>
                        <button
                            on:click=move |_| {
                                let state = match active_tab.get() {
                                    AdminModuleType::Settings       => ModalState::Settings,
                                    AdminModuleType::Services       => ModalState::Service(None),
                                    AdminModuleType::CaseStudies    => ModalState::CaseStudy(None),
                                    AdminModuleType::Highlights     => ModalState::Highlight(None),
                                    AdminModuleType::Blog           => ModalState::Post(None),
                                    AdminModuleType::ResumeProfiles => ModalState::Profile(None),
                                    AdminModuleType::ResumeEntries  => ModalState::BaseEntry(None, None),
                                    AdminModuleType::LandingPages   => ModalState::LandingPage(None),
                                    AdminModuleType::Footer         => ModalState::FooterItem(None),
                                    AdminModuleType::PageHeaders    => ModalState::PageHeader(None),
                                    AdminModuleType::Contacts       => ModalState::Contact(None),
                                    AdminModuleType::Leads          => ModalState::Lead(None),
                                    AdminModuleType::LeadOptions    => ModalState::LeadOption(None),
                                    AdminModuleType::Security       => ModalState::Passkey,
                                    _                               => ModalState::None,
                                };
                                set_modal_state.set(state);
                            }
                            class="bg-primary text-on-primary px-8 py-4 jetbrains text-xs font-bold tracking-[0.2em] uppercase hover:bg-primary-container transition-colors"
                        >
                            {move || match active_tab.get() {
                                AdminModuleType::Settings  => "EDIT VALUES",
                                AdminModuleType::Dashboard => "REFRESH",
                                _                          => "NEW ENTRY +",
                            }}
                        </button>
                    </div>

                    // Datagrid View — dispatch on AdminModuleType enum
                    <div class="flex-1 overflow-x-auto">
                        {move || match active_tab.get() {
                            AdminModuleType::Dashboard      => view! { <DashboardView /> }.into_any(),
                            AdminModuleType::Webforms       => view! { <WebformsTable /> }.into_any(),
                            AdminModuleType::Services       => view! { <ServiceTable /> }.into_any(),
                            AdminModuleType::CaseStudies    => view! { <CaseStudyTable /> }.into_any(),
                            AdminModuleType::Highlights     => view! { <HighlightTable /> }.into_any(),
                            AdminModuleType::Contacts       => view! { <contacts::ContactTable /> }.into_any(),
                            AdminModuleType::LeadOptions    => view! { <LeadOptionTable /> }.into_any(),
                            AdminModuleType::Navigation     => view! { <NavTable /> }.into_any(),
                            AdminModuleType::Footer         => view! { <FooterTable /> }.into_any(),
                            AdminModuleType::PageHeaders    => view! { <PageHeaderTable /> }.into_any(),
                            AdminModuleType::Settings       => view! { <SettingsReadView /> }.into_any(),
                            AdminModuleType::Blog           => view! { <PostTable /> }.into_any(),
                            AdminModuleType::ResumeProfiles => view! { <ResumeProfileTable /> }.into_any(),
                            AdminModuleType::ResumeEntries  => view! { <BaseResumeEntryTable /> }.into_any(),
                            AdminModuleType::LandingPages   => view! { <LandingPageTable /> }.into_any(),
                            AdminModuleType::Security       => view! { <PasskeyTable /> }.into_any(),
                            AdminModuleType::Leads          => view! { <leads::LeadTable /> }.into_any(),
                            _ => view! {
                                <div class="h-64 flex items-center justify-center border-2 border-dashed border-outline-variant text-outline">
                                    <span class="jetbrains text-sm">
                                        "MODULE_OFFLINE"
                                    </span>
                                </div>
                            }.into_any(),
                        }}
                    </div>
                </div>
                <AdminEditorModal />
            </section>
        </div>
    }
}



#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct DashboardStats {
    pub mempool_requests_24h: i64,
    pub total_signups: i64,
    pub total_mailing_list: i64,
    pub recent_page_views: i64,
}

#[server(GetDashboardStats, "/api")]
pub async fn get_dashboard_stats() -> Result<DashboardStats, ServerFnError> {
    use axum::Extension;
    use leptos_axum::extract;
    let Extension(state) = extract::<Extension<crate::state::AppState>>().await?;
    let Extension(tenant) = extract::<Extension<crate::state::TenantContext>>().await?;

    let mempool: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM api_requests_log WHERE endpoint = 'mempool_api' AND created_at > NOW() - INTERVAL '24 hours'").fetch_one(&state.pool).await.unwrap_or(0);
    // users.tenant_id was removed in the RBAC migration (m20260504_000002_remove_is_admin_from_user).
    // Count identities by joining user_account → account to resolve the tenant association.
    let signups: i64 = sqlx::query_scalar(
        "SELECT COUNT(DISTINCT ua.user_id) FROM user_account ua \
         JOIN account a ON a.id = ua.account_id \
         WHERE a.tenant_id IS NOT DISTINCT FROM $1",
    )
    .bind(tenant.0)
    .fetch_one(&state.pool)
    .await
    .unwrap_or(0);
    let mailing: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM mailing_list WHERE tenant_id IS NOT DISTINCT FROM $1",
    )
    .bind(tenant.0)
    .fetch_one(&state.pool)
    .await
    .unwrap_or(0);
    let views: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM page_views WHERE created_at > NOW() - INTERVAL '24 hours' AND tenant_id IS NOT DISTINCT FROM $1",
    )
    .bind(tenant.0)
    .fetch_one(&state.pool)
    .await
    .unwrap_or(0);

    Ok(DashboardStats {
        mempool_requests_24h: mempool,
        total_signups: signups,
        total_mailing_list: mailing,
        recent_page_views: views,
    })
}

#[component]
fn PageHeaderTable() -> impl IntoView {
    use crate::components::dynamic_header::get_all_page_headers;
    let refresh = expect_context::<ReadSignal<i32>>();
    let set_modal_state =
        expect_context::<WriteSignal<crate::components::admin_modal::ModalState>>();
    let headers_resource = Resource::new(move || refresh.get(), |_| get_all_page_headers());

    view! {
        <Transition fallback=move || view! { <div class="jetbrains text-sm text-outline">"QUERYING_DB..."</div> }>
            {move || {
                let res = headers_resource.get();
                view! {
            <table class="w-full text-left jetbrains text-sm">
                <thead>
                    <tr class="text-outline border-b border-outline-variant/30">
                        <th class="py-4 px-4 font-normal tracking-widest uppercase">"Route"</th>
                        <th class="py-4 px-4 font-normal tracking-widest uppercase">"Badge"</th>
                        <th class="py-4 px-4 font-normal tracking-widest uppercase">"Title"</th>
                        <th class="py-4 px-4 font-normal tracking-widest uppercase">"Actions"</th>
                    </tr>
                </thead>
                <tbody class="divide-y divide-outline-variant/20">
                    {match ResourceState::from(res) {
                        ResourceState::Ready(headers) => headers.into_iter().map(|h| {
                            let h_clone = h.clone();
                            view! {
                            <tr class="hover:bg-surface-container-high transition-colors group">
                                <td class="py-4 px-4 font-bold text-outline">{h.route_path.clone()}</td>
                                <td class="py-4 px-4 text-outline-variant">{h.badge_text.clone().unwrap_or_default()}</td>
                                <td class="py-4 px-4 text-on-surface">{h.title.clone()}</td>
                                <td class="py-4 px-4">
                                    <div class="flex space-x-4 opacity-0 group-hover:opacity-100 transition-opacity">
                                        <button on:click=move |_| set_modal_state.set(crate::components::admin_modal::ModalState::PageHeader(Some(h_clone.clone()))) class="text-secondary hover:underline uppercase text-xs">"Edit"</button>
                                    </div>
                                </td>
                            </tr>
                            }
                        }).collect::<Vec<_>>().into_any(),
                        ResourceState::Loading => view! { <tr class="hidden"></tr> }.into_any(),
                        ResourceState::Error(_) => view! { <tr><td colspan="4" class="py-8 text-center text-error">"ERR_NO_DATA"</td></tr> }.into_any(),
                    }}
                </tbody>
            </table>
            }.into_any()
            }}
        </Transition>
    }
}

#[component]
fn DashboardView() -> impl IntoView {
    let refresh = expect_context::<ReadSignal<i32>>();
    let stats_resource = Resource::new(move || refresh.get(), |_| get_dashboard_stats());

    view! {
        <Transition fallback=move || view! { <div class="jetbrains text-sm text-outline">"COMPILING_TELEMETRY..."</div> }>
            {move || match ResourceState::from(stats_resource.get()) {
                ResourceState::Ready(stats) => view! {
                    <div class="grid grid-cols-2 gap-8">
                        <div class="p-8 border border-outline-variant/30 flex flex-col justify-between">
                            <span class="jetbrains text-[0.65rem] uppercase text-outline tracking-wider mb-4">"Mempool Network Fetches (24H)"</span>
                            <span class="text-5xl font-extrabold text-[#f7931a]">{stats.mempool_requests_24h}</span>
                        </div>
                        <div class="p-8 border border-outline-variant/30 flex flex-col justify-between">
                            <span class="jetbrains text-[0.65rem] uppercase text-outline tracking-wider mb-4">"Page Views (24H)"</span>
                            <span class="text-5xl font-extrabold text-primary">{stats.recent_page_views}</span>
                        </div>
                        <div class="p-8 border border-outline-variant/30 flex flex-col justify-between">
                            <span class="jetbrains text-[0.65rem] uppercase text-outline tracking-wider mb-4">"Total Contacts"</span>
                            <span class="text-5xl font-extrabold text-secondary">{stats.total_mailing_list}</span>
                        </div>
                        <div class="p-8 border border-outline-variant/30 flex flex-col justify-between">
                            <span class="jetbrains text-[0.65rem] uppercase text-outline tracking-wider mb-4">"Admin Identities Registered"</span>
                            <span class="text-5xl font-extrabold text-on-surface">{stats.total_signups}</span>
                        </div>
                    </div>
                }.into_any(),
                ResourceState::Loading => view! { <div class="hidden"></div> }.into_any(),
                ResourceState::Error(_) => view! { <div class="text-error">"Failed to load settings"</div> }.into_any()
            }}
        </Transition>
    }
}

#[component]
fn SettingsReadView() -> impl IntoView {
    let refresh = expect_context::<ReadSignal<i32>>();
    let settings_res = Resource::new(
        move || refresh.get(),
        |_| crate::pages::landing::get_site_settings(),
    );

    view! {
        <Transition fallback=move || view! { <div class="jetbrains text-sm text-outline">"LOADING SETTINGS..."</div> }>
            {move || match ResourceState::from(settings_res.get()) {
                ResourceState::Ready(s) => view! {
                    <div class="space-y-0">
                    <div class="grid grid-cols-3 border-b-2 border-outline-variant/30 pb-2 mb-4">
                        <div class="font-label text-[0.65rem] uppercase tracking-widest text-outline">"KEY"</div>
                        <div class="col-span-2 font-label text-[0.65rem] uppercase tracking-widest text-outline">"VALUE"</div>
                    </div>

                        // Hero
                        <div class="grid grid-cols-3 py-2 border-b border-outline-variant/10 hover:bg-surface-container/30">
                            <div class="text-outline-variant uppercase tracking-widest text-xs">"CURRENT FOCUS"</div>
                            <div class="col-span-2 text-on-surface font-medium">{s.current_focus.clone()}</div>
                        </div>
                        <div class="grid grid-cols-3 py-2 border-b border-outline-variant/10 hover:bg-surface-container/30">
                            <div class="text-outline-variant uppercase tracking-widest text-xs">"STATUS"</div>
                            <div class="col-span-2 text-on-surface font-medium"><div class="inline-flex items-center gap-2"><div class="w-2 h-2 rounded-full" style=format!("background-color: {};", s.status_color)></div>{s.status.clone()}</div></div>
                        </div>
                        <div class="grid grid-cols-3 py-2 border-b border-outline-variant/10 hover:bg-surface-container/30">
                            <div class="text-outline-variant uppercase tracking-widest text-xs">"SUBTITLE"</div>
                            <div class="col-span-2 text-on-surface font-medium truncate">{s.hero_subtitle.clone()}</div>
                        </div>
                        <div class="grid grid-cols-3 py-2 border-b border-outline-variant/10 hover:bg-surface-container/30">
                            <div class="text-outline-variant uppercase tracking-widest text-xs">"QUOTE"</div>
                            <div class="col-span-2 text-on-surface font-medium truncate">{s.hero_quote.clone()}</div>
                        </div>

                        // Global
                        <div class="grid grid-cols-3 py-2 border-b border-primary/20 hover:bg-surface-container/30 mt-4">
                            <div class="text-primary uppercase tracking-widest text-xs">"SITE TITLE"</div>
                            <div class="col-span-2 text-on-surface font-bold">{s.site_title.clone()}</div>
                        </div>
                        <div class="grid grid-cols-3 py-2 border-b border-outline-variant/10 hover:bg-surface-container/30">
                            <div class="text-primary uppercase tracking-widest text-xs">"WEBHOOK URL"</div>
                            <div class="col-span-2 text-on-surface font-mono text-xs">{s.webhook_url.clone()}</div>
                        </div>
                        <div class="grid grid-cols-3 py-2 border-b border-outline-variant/10 hover:bg-surface-container/30">
                            <div class="text-primary uppercase tracking-widest text-xs">"ADMIN NOTIFICATION"</div>
                            <div class="col-span-2 text-on-surface font-mono text-xs">{s.admin_email.clone()}</div>
                        </div>

                        // Lead Capture
                        <div class="grid grid-cols-3 py-2 border-b border-outline-variant/10 hover:bg-surface-container/30 mt-4">
                            <div class="text-secondary uppercase tracking-widest text-xs">"LC TITLE"</div>
                            <div class="col-span-2 text-on-surface font-medium">{s.lc_title.clone()}</div>
                        </div>
                        <div class="grid grid-cols-3 py-2 border-b border-outline-variant/10 hover:bg-surface-container/30">
                            <div class="text-secondary uppercase tracking-widest text-xs">"LC DESC"</div>
                            <div class="col-span-2 text-on-surface font-medium truncate">{s.lc_desc.clone()}</div>
                        </div>
                        <div class="grid grid-cols-3 py-2 border-b border-outline-variant/10 hover:bg-surface-container/30">
                            <div class="text-secondary uppercase tracking-widest text-xs">"LC BTN"</div>
                            <div class="col-span-2 text-on-surface font-medium">{s.lc_btn.clone()}</div>
                        </div>


                        // Landing Pages Settings
                        <div class="grid grid-cols-3 py-2 border-b border-outline-variant/10 hover:bg-surface-container/30 mt-4">
                            <div class="text-outline uppercase tracking-widest text-xs">"BOOKING URL"</div>
                            <div class="col-span-2 text-on-surface font-medium truncate">{s.booking_url.clone()}</div>
                        </div>
                        <div class="grid grid-cols-3 py-2 border-b border-outline-variant/10 hover:bg-surface-container/30 mt-4">
                            <div class="text-outline uppercase tracking-widest text-xs">"TERMS HTML (MD)"</div>
                            <div class="col-span-2 text-on-surface font-mono text-xs truncate max-h-24 overflow-hidden">{s.terms_html.clone()}</div>
                        </div>
                        <div class="grid grid-cols-3 py-2 border-b border-outline-variant/10 hover:bg-surface-container/30 mt-4">
                            <div class="text-outline uppercase tracking-widest text-xs">"PRIVACY HTML (MD)"</div>
                            <div class="col-span-2 text-on-surface font-mono text-xs truncate max-h-24 overflow-hidden">{s.privacy_html.clone()}</div>
                        </div>

                        // Social Media Links
                        <div class="grid grid-cols-3 py-2 border-b border-outline-variant/10 hover:bg-surface-container/30 mt-4">
                            <div class="text-outline uppercase tracking-widest text-xs">"GITHUB URL"</div>
                            <div class="col-span-2 text-on-surface font-medium truncate">{s.github_url.clone()}</div>
                        </div>
                        <div class="grid grid-cols-3 py-2 border-b border-outline-variant/10 hover:bg-surface-container/30">
                            <div class="text-outline uppercase tracking-widest text-xs">"X (TWITTER) URL"</div>
                            <div class="col-span-2 text-on-surface font-medium truncate">{s.x_url.clone()}</div>
                        </div>
                        <div class="grid grid-cols-3 py-2 border-b border-outline-variant/10 hover:bg-surface-container/30">
                            <div class="text-outline uppercase tracking-widest text-xs">"LINKEDIN URL"</div>
                            <div class="col-span-2 text-on-surface font-medium truncate">{s.linkedin_url.clone()}</div>
                        </div>

                        // Global B2B Settings
                        <div class="grid grid-cols-3 py-2 border-b border-outline-variant/10 hover:bg-surface-container/30 mt-4 bg-tertiary/10 p-2">
                            <div class="text-tertiary uppercase tracking-widest text-xs font-bold">"B2B CONSULTING MODE"</div>
                            <div class="col-span-2 text-on-surface font-medium">
                                {if s.b2b_enabled { "ENABLED (PUBLIC)" } else { "STEALTH (HIDDEN)" }}
                            </div>
                        </div>
                </div>
                }.into_any(),
                ResourceState::Loading => view! { <div class="hidden"></div> }.into_any(),
                ResourceState::Error(_) => view! { <div class="text-error">"Failed to load settings"</div> }.into_any()
            }}
        </Transition>
    }
}

#[component]
fn ResumeProfileTable() -> impl IntoView {
    use crate::resume_engine::{delete_resume_profile, download_resume, get_entry_collections};
    let refresh = expect_context::<ReadSignal<i32>>();
    let set_refresh = expect_context::<WriteSignal<i32>>();
    let set_modal_state =
        expect_context::<WriteSignal<crate::components::admin_modal::ModalState>>();

    let items_res = Resource::new(move || refresh.get(), |_| get_entry_collections());

    view! {
        <Transition fallback=move || view! { <div class="jetbrains text-sm text-outline">"LOADING..."</div> }>
            {move || {
                let res = items_res.get();
                view! {
            <table class="w-full text-left border-collapse">
                <thead>
                    <tr class="border-b-2 border-outline-variant/30">
                        <th class="py-4 font-label text-[0.65rem] uppercase tracking-widest text-outline">"ID"</th>
                        <th class="py-4 font-label text-[0.65rem] uppercase tracking-widest text-outline">"PROFILE NAME"</th>
                        <th class="py-4 font-label text-[0.65rem] uppercase tracking-widest text-outline text-right">"ACTIONS"</th>
                    </tr>
                </thead>
                <tbody class="jetbrains text-sm">
                    {match ResourceState::from(res) {
                        ResourceState::Ready(items) => items.into_iter().map(|item| {
                            let id_val = item.id;
                            let clone_item = item.clone();
                            view! {
                                <tr class="border-b border-outline-variant/10 hover:bg-surface-container/50 transition-colors">
                                    <td class="py-4 text-outline-variant font-medium">{id_val.to_string()}</td>
                                    <td class="py-4 font-bold text-on-surface">{item.name.clone()}</td>
                                    <td class="py-4 text-right space-x-4">
                                        <button
                                            on:click=move |_| {
                                                leptos::task::spawn_local(async move {
                                                    if let Ok(bytes) = download_resume(id_val).await {
                                                        use web_sys::js_sys::{Array, Uint8Array};
                                                        use web_sys::{Blob, BlobPropertyBag, Url};

                                                        let uint8_arr = Uint8Array::from(bytes.as_slice());
                                                        let parts = Array::new();
                                                        parts.push(&uint8_arr);

                                                        let props = BlobPropertyBag::new();
                                                        props.set_type("application/pdf");

                                                        if let Ok(blob) = Blob::new_with_u8_array_sequence_and_options(&parts, &props) {
                                                            if let Ok(url) = Url::create_object_url_with_blob(&blob) {
                                                                if let Some(window) = web_sys::window() {
                                                                    let _ = window.open_with_url_and_target(&url, "_blank");
                                                                }
                                                            }
                                                        }
                                                    }
                                                });
                                            }
                                            class="text-primary hover:text-primary-container font-medium tracking-wide"
                                        >"[PREVIEW]"</button>
                                        <button
                                            on:click=move |_| {
                                                leptos::task::spawn_local(async move {
                                                    if let Ok(bytes) = download_resume(id_val).await {
                                                        use web_sys::js_sys::{Array, Uint8Array};
                                                        use web_sys::{Blob, BlobPropertyBag, Url};

                                                        let uint8_arr = Uint8Array::from(bytes.as_slice());
                                                        let parts = Array::new();
                                                        parts.push(&uint8_arr);

                                                        let props = BlobPropertyBag::new();
                                                        props.set_type("application/pdf");

                                                        if let Ok(blob) = Blob::new_with_u8_array_sequence_and_options(&parts, &props) {
                                                            if let Ok(url) = Url::create_object_url_with_blob(&blob) {
                                                                let document = web_sys::window().unwrap().document().unwrap();
                                                                if let Ok(a) = document.create_element("a") {
                                                                    let _ = a.set_attribute("href", &url);
                                                                    let _ = a.set_attribute("download", &format!("Profile_{}_Resume.pdf", id_val));
                                                                    use web_sys::wasm_bindgen::JsCast;
                                                                    let html_a = a.unchecked_into::<web_sys::HtmlElement>();
                                                                    html_a.click();
                                                                }
                                                            }
                                                        }
                                                    }
                                                });
                                            }
                                            class="text-primary hover:text-primary-container font-medium tracking-wide"
                                        >"[DOWNLOAD]"</button>
                                        <button on:click=move |_| set_modal_state.set(crate::components::admin_modal::ModalState::Profile(Some(clone_item.clone()))) class="text-secondary hover:text-on-secondary-fixed-variant font-medium tracking-wide">"[EDIT]"</button>
                                        <button
                                            on:click=move |_| {
                                                leptos::task::spawn_local(async move {
                                                    let _ = delete_resume_profile(id_val).await;
                                                    set_refresh.set(refresh.get_untracked() + 1);
                                                });
                                            }
                                            class="text-error hover:text-error/80 font-medium tracking-wide"
                                        >"[DEL]"</button>
                                    </td>
                                </tr>
                            }
                        }).collect::<Vec<_>>().into_any(),
                        ResourceState::Loading => view! { <tr class="hidden"></tr> }.into_any(),
                        ResourceState::Error(_) => view! { <tr><td colspan="3" class="py-8 text-center text-error">"ERR_NO_DATA"</td></tr> }.into_any(),
                    }}
                </tbody>
            </table>
            }.into_any()
            }}
        </Transition>
    }
}

#[component]
fn BaseResumeEntryTable() -> impl IntoView {
    use crate::resume_engine::{delete_base_entry, get_all_base_entries, ResumeCategory};
    let refresh = expect_context::<ReadSignal<i32>>();
    let set_refresh = expect_context::<WriteSignal<i32>>();
    let set_modal_state =
        expect_context::<WriteSignal<crate::components::admin_modal::ModalState>>();

    let items_res = Resource::new(move || refresh.get(), |_| get_all_base_entries());

    view! {
        <Transition fallback=move || view! { <div class="jetbrains text-sm text-outline">"LOADING DATA..."</div> }>
            {move || match ResourceState::from(items_res.get()) {
                ResourceState::Ready(items) => {
                    if items.is_empty() {
                        view! { <div class="py-8 text-center text-outline-variant">"NO ENTRIES IN DATABASE"</div> }.into_any()
                    } else {
                        let categories = vec![
                            ResumeCategory::Work,
                            ResumeCategory::Education,
                            ResumeCategory::Certification,
                            ResumeCategory::Project,
                            ResumeCategory::Skill,
                            ResumeCategory::Language,
                            ResumeCategory::Volunteer,
                            ResumeCategory::Extracurricular,
                            ResumeCategory::Hobby,
                        ];

                        categories.into_iter().map(|cat| {
                            let cat_items: Vec<_> = items.iter().filter(|i| i.category == cat).cloned().collect();
                            if cat_items.is_empty() {
                                view! { <div class="hidden"></div> }.into_any()
                            } else {
                                let category_str = cat.to_string();
                                view! {
                                    <div class="mb-12">
                                        <div class="flex justify-between items-center mb-4">
                                            <div class="inline-block bg-secondary-container/20 px-3 py-1 border border-secondary/30">
                                                <span class="font-label text-[0.6875rem] text-secondary font-bold tracking-tighter uppercase">{category_str.clone()} " ENTRIES"</span>
                                            </div>
                                            <button
                                                on:click=move |_| set_modal_state.set(crate::components::admin_modal::ModalState::BaseEntry(None, Some(cat)))
                                                class="bg-surface-container-high hover:bg-surface-container-highest text-primary px-3 py-1 text-xs font-bold font-label uppercase transition-colors border border-outline-variant/30 flex items-center gap-2"
                                            >
                                                <span class="material-symbols-outlined text-[0.8rem]">"add"</span>
                                                {format!("NEW {}", category_str.clone())}
                                            </button>
                                        </div>
                                        <table class="w-full text-left border-collapse">
                                            <thead>
                                                <tr class="border-b-2 border-outline-variant/30">
                                                    <th class="py-4 font-label text-[0.65rem] uppercase tracking-widest text-outline w-16">"ID"</th>
                                                    <th class="py-4 font-label text-[0.65rem] uppercase tracking-widest text-outline">"TITLE"</th>
                                                    <th class="py-4 font-label text-[0.65rem] uppercase tracking-widest text-outline text-right">"ACTIONS"</th>
                                                </tr>
                                            </thead>
                                            <tbody class="jetbrains text-sm">
                                                {cat_items.into_iter().map(|item| {
                                                    let id_val = item.id;
                                                    let clone_item = item.clone();
                                                    view! {
                                                        <tr class="border-b border-outline-variant/10 hover:bg-surface-container/50 transition-colors">
                                                            <td class="py-4 text-outline-variant font-medium">{id_val.to_string()}</td>
                                                            <td class="py-4 font-bold text-on-surface truncate">{item.title.clone()}</td>
                                                            <td class="py-4 text-right space-x-4">
                                                                <button on:click=move |_| set_modal_state.set(crate::components::admin_modal::ModalState::BaseEntry(Some(clone_item.clone()), Some(clone_item.category))) class="text-secondary hover:text-on-secondary-fixed-variant font-medium tracking-wide">"[EDIT]"</button>
                                                                <button
                                                                    on:click=move |_| {
                                                                        leptos::task::spawn_local(async move {
                                                                            let _ = delete_base_entry(id_val).await;
                                                                            set_refresh.set(refresh.get_untracked() + 1);
                                                                        });
                                                                    }
                                                                    class="text-error hover:text-error/80 font-medium tracking-wide"
                                                                >"[DEL]"</button>
                                                            </td>
                                                        </tr>
                                                    }
                                                }).collect::<Vec<_>>()}
                                            </tbody>
                                        </table>
                                    </div>
                                }.into_any()
                            }
                        }).collect::<Vec<_>>().into_any()
                    }
                },
                ResourceState::Loading => view! { <div class="hidden"></div> }.into_any(),
                ResourceState::Error(_) => view! { <div class="py-8 text-center text-error">"ERR_NO_DATA"</div> }.into_any(),
            }}
        </Transition>
    }
}



#[component]
fn LeadOptionTable() -> impl IntoView {
    use crate::pages::landing::{delete_lead_option, get_all_lead_options};
    let refresh = expect_context::<ReadSignal<i32>>();
    let set_refresh = expect_context::<WriteSignal<i32>>();
    let set_modal_state =
        expect_context::<WriteSignal<crate::components::admin_modal::ModalState>>();

    let items_res = Resource::new(move || refresh.get(), |_| get_all_lead_options());

    view! {
        <Transition fallback=move || view! { <div class="jetbrains text-sm text-outline">"LOADING DATA..."</div> }>
            {move || {
                let res = items_res.get();
                view! {
            <table class="w-full text-left border-collapse">
                <thead>
                    <tr class="border-b-2 border-outline-variant/30">
                        <th class="py-4 font-label text-[0.65rem] uppercase tracking-widest text-outline">"ID"</th>
                        <th class="py-4 font-label text-[0.65rem] uppercase tracking-widest text-outline">"Order"</th>
                        <th class="py-4 font-label text-[0.65rem] uppercase tracking-widest text-outline">"Key"</th>
                        <th class="py-4 font-label text-[0.65rem] uppercase tracking-widest text-outline">"Label"</th>
                        <th class="py-4 font-label text-[0.65rem] uppercase tracking-widest text-outline">"Status"</th>
                        <th class="py-4 font-label text-[0.65rem] uppercase tracking-widest text-outline text-right">"Actions"</th>
                    </tr>
                </thead>
                <tbody class="jetbrains text-sm">
                    {match ResourceState::from(res) {
                        ResourceState::Ready(items) => items.into_iter().map(|item| {
                            let id_val = item.id;
                            let clone_item = item.clone();
                            view! {
                                <tr class="border-b border-outline-variant/10 hover:bg-surface-container/50 transition-colors">
                                    <td class="py-4 text-outline-variant font-medium">{id_val.to_string()}</td>
                                    <td class="py-4 text-on-surface font-medium">{item.display_order}</td>
                                    <td class="py-4 text-on-surface font-mono text-xs">{item.value_key.clone()}</td>
                                    <td class="py-4 font-bold text-on-surface truncate">{item.label.clone()}</td>
                                    <td class="py-4 font-medium">
                                        <div class="inline-flex items-center gap-2">
                                            <div class="w-1.5 h-1.5 rounded-full" style=if item.is_active { "background-color: #4ade80" } else { "background-color: #f87171" }></div>
                                            {if item.is_active { "ACTIVE" } else { "INACTIVE" }}
                                        </div>
                                    </td>
                                    <td class="py-4 text-right space-x-4">
                                        <button on:click=move |_| set_modal_state.set(crate::components::admin_modal::ModalState::LeadOption(Some(clone_item.clone()))) class="text-secondary hover:text-on-secondary-fixed-variant font-medium tracking-wide">"[EDIT]"</button>
                                        <button
                                            on:click=move |_| {
                                                leptos::task::spawn_local(async move {
                                                    let _ = delete_lead_option(id_val).await;
                                                    set_refresh.set(refresh.get_untracked() + 1);
                                                });
                                            }
                                            class="text-error hover:text-error/80 font-medium tracking-wide"
                                        >"[DEL]"</button>
                                    </td>
                                </tr>
                            }
                        }).collect::<Vec<_>>().into_any(),
                        ResourceState::Loading => view! { <tr class="hidden"></tr> }.into_any(),
                        ResourceState::Error(_) => view! { <tr><td colspan="6" class="py-8 text-center text-error">"ERR_NO_DATA"</td></tr> }.into_any(),
                    }}
                </tbody>
            </table>
            }.into_any()
            }}
        </Transition>
    }
}



#[component]
fn PostTable() -> impl IntoView {
    use crate::pages::blog::get_posts;
    let refresh = expect_context::<ReadSignal<i32>>();
    let set_refresh = expect_context::<WriteSignal<i32>>();
    let set_modal_state = expect_context::<WriteSignal<ModalState>>();
    let posts_resource = Resource::new(move || refresh.get(), |_| get_posts());

    view! {
        <Transition fallback=move || view! { <div class="jetbrains text-sm text-outline">"QUERYING_DB..."</div> }>
            {move || {
                let res = posts_resource.get();
                view! {
            <table class="w-full text-left jetbrains text-sm">
                <thead>
                    <tr class="text-outline border-b border-outline-variant/30">
                        <th class="py-4 px-4 font-normal tracking-widest uppercase">"ID"</th>
                        <th class="py-4 px-4 font-normal tracking-widest uppercase">"Slug"</th>
                        <th class="py-4 px-4 font-normal tracking-widest uppercase">"Title"</th>
                        <th class="py-4 px-4 font-normal tracking-widest uppercase">"Actions"</th>
                    </tr>
                </thead>
                <tbody class="divide-y divide-outline-variant/20">
                    {match ResourceState::from(res) {
                        ResourceState::Ready(posts) => posts.into_iter().map(|p| {
                            let p_clone = p.clone();
                            let del_id = p.id.clone();
                            view! {
                            <tr class="hover:bg-surface-container-high transition-colors group">
                                <td class="py-4 px-4 text-outline-variant">"#" {p.id.clone()}</td>
                                <td class="py-4 px-4 font-bold text-outline">{p.subtitle.clone().unwrap_or_default()}</td>
                                <td class="py-4 px-4 text-primary font-bold">{p.title.clone()}</td>
                                <td class="py-4 px-4">
                                    <div class="flex space-x-4 opacity-0 group-hover:opacity-100 transition-opacity">
                                        <button
                                            on:click=move |_| set_modal_state.set(ModalState::Post(Some(p_clone.clone())))
                                            class="text-secondary hover:underline uppercase text-xs"
                                        >
                                            "Edit"
                                        </button>
                                        <button
                                            on:click=move |_| {
                                                let target_id = del_id.clone();
                                                leptos::task::spawn_local(async move {
                                                    if let Ok(_) = crate::pages::blog::delete_post(target_id).await {
                                                        set_refresh.set(refresh.get_untracked() + 1);
                                                    }
                                                });
                                            }
                                            class="text-error hover:underline uppercase text-xs"
                                        >
                                            "Drop"
                                        </button>
                                    </div>
                                </td>
                            </tr>
                            }
                        }).collect::<Vec<_>>().into_any(),
                        ResourceState::Loading => view! { <tr class="hidden"></tr> }.into_any(),
                        ResourceState::Error(_) => view! { <tr><td colspan="4" class="py-8 text-center text-error">"ERR_NO_DATA"</td></tr> }.into_any(),
                    }}
                </tbody>
            </table>
            }.into_any()
            }}
        </Transition>
    }
}

#[component]
fn PasskeyTable() -> impl IntoView {
    use crate::auth::get_users;
    let refresh = expect_context::<ReadSignal<i32>>();
    let set_refresh = expect_context::<WriteSignal<i32>>();
    let users_resource = Resource::new(move || refresh.get(), |_| get_users());

    view! {
        <Transition fallback=move || view! { <div class="jetbrains text-sm text-outline">"QUERYING_DB..."</div> }>
            {move || {
                let res = users_resource.get();
                view! {
            <table class="w-full text-left jetbrains text-sm">
                <thead>
                    <tr class="text-outline border-b border-outline-variant/30">
                        <th class="py-4 px-4 font-normal tracking-widest uppercase">"ID"</th>
                        <th class="py-4 px-4 font-normal tracking-widest uppercase">"Username"</th>
                        <th class="py-4 px-4 font-normal tracking-widest uppercase">"Created At"</th>
                        <th class="py-4 px-4 font-normal tracking-widest uppercase">"Actions"</th>
                    </tr>
                </thead>
                <tbody class="divide-y divide-outline-variant/20">
                    {match ResourceState::from(res) {
                        ResourceState::Ready(users) => users.into_iter().map(|u| view! {
                            <tr class="hover:bg-surface-container-high transition-colors group">
                                <td class="py-4 text-outline-variant">"#" {u.id}</td>
                                <td class="py-4 px-4 font-bold text-primary">{u.username}</td>
                                <td class="py-4 px-4 text-outline">{u.created_at}</td>
                                <td class="py-4 px-4">
                                    <div class="flex space-x-4 opacity-0 group-hover:opacity-100 transition-opacity">
                                        <button
                                            on:click=move |_| {
                                                let id = u.id;
                                                leptos::task::spawn_local(async move {
                                                    if let Ok(_) = crate::auth::delete_user(id).await {
                                                        set_refresh.set(refresh.get_untracked() + 1);
                                                    }
                                                });
                                            }
                                            class="text-error hover:underline uppercase text-xs"
                                        >
                                            "Revoke"
                                        </button>
                                    </div>
                                </td>
                            </tr>
                        }).collect::<Vec<_>>().into_any(),
                        ResourceState::Loading => view! { <tr class="hidden"></tr> }.into_any(),
                        ResourceState::Error(_) => view! { <tr><td colspan="4" class="py-8 text-center text-error">"ERR_NO_DATA"</td></tr> }.into_any(),
                    }}
                </tbody>
            </table>
            }.into_any()
            }}
        </Transition>
    }
}

#[component]
pub fn LandingPageTable() -> impl IntoView {
    use crate::pages::dynamic_landing::{delete_landing_page, get_all_landing_pages};
    let refresh = expect_context::<ReadSignal<i32>>();
    let set_refresh = expect_context::<WriteSignal<i32>>();
    let set_modal_state =
        expect_context::<WriteSignal<crate::components::admin_modal::ModalState>>();
    let pages_resource = Resource::new(move || refresh.get(), |_| get_all_landing_pages());

    view! {
        <Transition fallback=move || view! { <div class="jetbrains text-sm text-outline">"QUERYING_DB..."</div> }>
            {move || {
                let res = pages_resource.get();
                view! {
            <table class="w-full text-left jetbrains text-sm">
                <thead>
                    <tr class="text-outline border-b border-outline-variant/30">
                        <th class="py-4 px-4 font-normal tracking-widest uppercase">"Slug"</th>
                        <th class="py-4 px-4 font-normal tracking-widest uppercase">"Title"</th>
                        <th class="py-4 px-4 font-normal tracking-widest uppercase">"Actions"</th>
                    </tr>
                </thead>
                <tbody class="divide-y divide-outline-variant/20">
                    {match ResourceState::from(res) {
                        ResourceState::Ready(pages) => pages.into_iter().map(|p| {
                            let p_clone = p.clone();
                            let p_clone_2 = p.clone();
                            view! {
                            <tr class="hover:bg-surface-container-high transition-colors group">
                                <td class="py-4 font-bold text-primary">"/" {p.slug}</td>
                                <td class="py-4 px-4 text-outline">{p.title}</td>
                                <td class="py-4 px-4">
                                    <div class="flex space-x-4 opacity-0 group-hover:opacity-100 transition-opacity">
                                        <button
                                            on:click=move |_| set_modal_state.set(crate::components::admin_modal::ModalState::LandingPage(Some(p_clone.clone())))
                                            class="text-secondary hover:underline uppercase text-xs"
                                        >
                                            "Edit"
                                        </button>
                                        <button
                                            on:click=move |_| {
                                                let target_slug = p_clone_2.slug.clone();
                                                leptos::task::spawn_local(async move {
                                                    if let Ok(_) = delete_landing_page(target_slug).await {
                                                        set_refresh.set(refresh.get_untracked() + 1);
                                                    }
                                                });
                                            }
                                            class="text-error hover:underline uppercase text-xs"
                                        >
                                            "Drop"
                                        </button>
                                    </div>
                                </td>
                            </tr>
                        }}).collect::<Vec<_>>().into_any(),
                        ResourceState::Loading => view! { <tr class="hidden"></tr> }.into_any(),
                        ResourceState::Error(_) => view! { <tr><td colspan="3" class="py-8 text-center text-error">"ERR_NO_DATA"</td></tr> }.into_any(),
                    }}
                </tbody>
            </table>
            }.into_any()
            }}
        </Transition>
    }
}

#[component]
pub fn NavTable() -> impl IntoView {
    use crate::components::nav::{delete_nav_item, get_all_nav_items};
    let refresh = expect_context::<ReadSignal<i32>>();
    let set_refresh = expect_context::<WriteSignal<i32>>();
    let set_modal_state =
        expect_context::<WriteSignal<crate::components::admin_modal::ModalState>>();
    let nav_resource = Resource::new(move || refresh.get(), |_| get_all_nav_items());

    view! {
        <Transition fallback=move || view! { <div class="jetbrains text-sm text-outline">"QUERYING_DB..."</div> }>
            {move || {
                let res = nav_resource.get();
                view! {
            <table class="w-full text-left jetbrains text-sm">
                <thead>
                    <tr class="text-outline border-b border-outline-variant/30">
                        <th class="py-4 px-4 font-normal tracking-widest uppercase">"Weight"</th>
                        <th class="py-4 px-4 font-normal tracking-widest uppercase">"Label"</th>
                        <th class="py-4 px-4 font-normal tracking-widest uppercase">"Binding"</th>
                        <th class="py-4 px-4 font-normal tracking-widest uppercase">"Actions"</th>
                    </tr>
                </thead>
                <tbody class="divide-y divide-outline-variant/20">
                    {match ResourceState::from(res) {
                        ResourceState::Ready(items) => items.into_iter().map(|n| {
                            let n_clone = n.clone();
                            view! {
                            <tr class="hover:bg-surface-container-high transition-colors group">
                                <td class="py-4 px-4 text-outline-variant">{n.display_order}</td>
                                <td class="py-4 px-4 font-medium">"#" {n.id.to_string()}</td>
                                <td class="py-4 px-4 font-bold text-primary">
                                    {if let Some(_pid) = n.parent_id { format!("↳ {}", n.label) } else { n.label.clone() }}
                                </td>
                                <td class="py-4 px-4 text-outline">{n.href.clone().unwrap_or_default()}</td>
                                <td class="py-4 px-4">
                                    <div class="flex space-x-4 opacity-0 group-hover:opacity-100 transition-opacity">
                                        <button
                                            on:click=move |_| set_modal_state.set(crate::components::admin_modal::ModalState::NavItem(Some(n_clone.clone())))
                                            class="text-secondary hover:underline uppercase text-xs"
                                        >
                                            "Edit"
                                        </button>
                                        <button
                                            on:click=move |_| {
                                                let id = n.id;
                                                leptos::task::spawn_local(async move {
                                                    if let Ok(_) = delete_nav_item(id).await {
                                                        set_refresh.set(refresh.get_untracked() + 1);
                                                    }
                                                });
                                            }
                                            class="text-error hover:underline uppercase text-xs"
                                        >
                                            "Drop"
                                        </button>
                                    </div>
                                </td>
                            </tr>
                        }}).collect::<Vec<_>>().into_any(),
                        ResourceState::Loading => view! { <tr class="hidden"></tr> }.into_any(),
                        ResourceState::Error(_) => view! { <tr><td colspan="4" class="py-8 text-center text-error">"ERR_NO_DATA"</td></tr> }.into_any(),
                    }}
                </tbody>
            </table>
            }.into_any()
            }}
        </Transition>
    }
}

#[component]
pub fn FooterTable() -> impl IntoView {
    use crate::components::footer::{delete_footer_item, get_all_footer_items};
    let refresh = expect_context::<ReadSignal<i32>>();
    let set_refresh = expect_context::<WriteSignal<i32>>();
    let set_modal_state =
        expect_context::<WriteSignal<crate::components::admin_modal::ModalState>>();
    let footer_resource = Resource::new(move || refresh.get(), |_| get_all_footer_items());

    view! {
        <Transition fallback=move || view! { <div class="jetbrains text-sm text-outline">"QUERYING_DB..."</div> }>
            {move || {
                let res = footer_resource.get();
                view! {
            <table class="w-full text-left jetbrains text-sm">
                <thead>
                    <tr class="text-outline border-b border-outline-variant/30">
                        <th class="py-4 px-4 font-normal tracking-widest uppercase">"Weight"</th>
                        <th class="py-4 px-4 font-bold text-primary">"Label"</th>
                        <th class="py-4 px-4 text-outline">"Link / Dropdown"</th>
                        <th class="py-4 px-4 font-normal tracking-widest uppercase">"Actions"</th>
                    </tr>
                </thead>
                <tbody class="divide-y divide-outline-variant/20">
                    {match ResourceState::from(res) {
                        ResourceState::Ready(items) => items.into_iter().map(|n| {
                            let n_clone = n.clone();
                            view! {
                            <tr class="hover:bg-surface-container-high transition-colors group">
                                <td class="py-4 px-4 text-outline-variant">{n.display_order}</td>
                                <td class="py-4 font-bold text-primary">
                                    {n.label.clone()}
                                </td>
                                <td class="py-4 px-4 text-outline">{n.href.unwrap_or_else(|| "DROPDOWN [null]".to_string())}</td>
                                <td class="py-4 px-4">
                                    <div class="flex space-x-4 opacity-0 group-hover:opacity-100 transition-opacity">
                                        <button
                                            on:click=move |_| set_modal_state.set(crate::components::admin_modal::ModalState::FooterItem(Some(n_clone.clone())))
                                            class="text-secondary hover:underline uppercase text-xs"
                                        >
                                            "Edit"
                                        </button>
                                        <button
                                            on:click=move |_| {
                                                let id = n.id;
                                                leptos::task::spawn_local(async move {
                                                    if let Ok(_) = delete_footer_item(id).await {
                                                        set_refresh.set(refresh.get_untracked() + 1);
                                                    }
                                                });
                                            }
                                            class="text-error hover:underline uppercase text-xs"
                                        >
                                            "Drop"
                                        </button>
                                    </div>
                                </td>
                            </tr>
                        }}).collect::<Vec<_>>().into_any(),
                        ResourceState::Loading => view! { <tr class="hidden"></tr> }.into_any(),
                        ResourceState::Error(_) => view! { <tr><td colspan="4" class="py-8 text-center text-error">"ERR_NO_DATA"</td></tr> }.into_any(),
                    }}
                </tbody>
            </table>
            }.into_any()
            }}
        </Transition>
    }
}

#[component]
pub fn ServiceTable() -> impl IntoView {
    use crate::b2b::{delete_service, get_services};
    let refresh = expect_context::<ReadSignal<i32>>();
    let set_refresh = expect_context::<WriteSignal<i32>>();
    let set_modal_state =
        expect_context::<WriteSignal<crate::components::admin_modal::ModalState>>();
    let data_res = Resource::new(move || refresh.get(), |_| get_services(false));

    view! {
        <Transition fallback=move || view! { <div class="jetbrains text-sm text-outline">"QUERYING_DB..."</div> }>
            {move || {
                let res = data_res.get();
                view! {
            <table class="w-full text-left jetbrains text-sm">
                <thead><tr class="text-outline border-b border-outline-variant/30">
                    <th class="py-4 px-4 font-normal tracking-widest uppercase">"Weight"</th>
                    <th class="py-4 px-4 font-normal tracking-widest uppercase">"Title"</th>
                    <th class="py-4 px-4 font-normal tracking-widest uppercase">"Visible"</th>
                    <th class="py-4 px-4 font-normal tracking-widest uppercase">"Actions"</th>
                </tr></thead>
                <tbody class="divide-y divide-outline-variant/20">
                    {match ResourceState::from(res) {
                        ResourceState::Ready(items) => items.into_iter().map(|item| { let c = item.clone(); view! {
                            <tr class="hover:bg-surface-container-high transition-colors group">
                                <td class="py-4 px-4 text-outline-variant">{item.display_order}</td>
                                <td class="py-4 px-4 font-bold text-primary">{item.title}</td>
                                <td class="py-4 px-4 text-outline">{if item.is_visible { "YES" } else { "NO" }}</td>
                                <td class="py-4 px-4">
                                    <div class="flex space-x-4 opacity-0 group-hover:opacity-100 transition-opacity">
                                        <button on:click=move |_| set_modal_state.set(crate::components::admin_modal::ModalState::Service(Some(c.clone()))) class="text-secondary hover:underline uppercase text-xs">"Edit"</button>
                                        <button on:click=move |_| { let id = item.id; leptos::task::spawn_local(async move { if let Ok(_) = delete_service(id).await { set_refresh.set(refresh.get_untracked() + 1); } }); } class="text-error hover:underline uppercase text-xs">"Drop"</button>
                                    </div>
                                </td>
                            </tr>
                        }}).collect::<Vec<_>>().into_any(),
                        ResourceState::Loading => view! { <tr class="hidden"></tr> }.into_any(),
                        ResourceState::Error(_) => view! { <tr><td colspan="4" class="py-8 text-center text-error">"ERR_NO_DATA"</td></tr> }.into_any(),
                    }}
                </tbody>
            </table>
            }.into_any()
            }}
        </Transition>
    }
}

#[component]
pub fn CaseStudyTable() -> impl IntoView {
    use crate::b2b::{delete_case_study, get_case_studies};
    let refresh = expect_context::<ReadSignal<i32>>();
    let set_refresh = expect_context::<WriteSignal<i32>>();
    let set_modal_state =
        expect_context::<WriteSignal<crate::components::admin_modal::ModalState>>();
    let data_res = Resource::new(move || refresh.get(), |_| get_case_studies(false));

    view! {
        <Transition fallback=move || view! { <div class="jetbrains text-sm text-outline">"QUERYING_DB..."</div> }>
            {move || {
                let res = data_res.get();
                view! {
            <table class="w-full text-left jetbrains text-sm">
                <thead><tr class="text-outline border-b border-outline-variant/30">
                    <th class="py-4 px-4 font-normal tracking-widest uppercase">"Weight"</th>
                    <th class="py-4 px-4 font-normal tracking-widest uppercase">"Client"</th>
                    <th class="py-4 px-4 font-normal tracking-widest uppercase">"Visible"</th>
                    <th class="py-4 px-4 font-normal tracking-widest uppercase">"Actions"</th>
                </tr></thead>
                <tbody class="divide-y divide-outline-variant/20">
                    {match ResourceState::from(res) {
                        ResourceState::Ready(items) => items.into_iter().map(|item| { let c = item.clone(); view! {
                            <tr class="hover:bg-surface-container-high transition-colors group">
                                <td class="py-4 px-4 text-outline-variant">{item.display_order}</td>
                                <td class="py-4 font-bold text-primary">{item.client_name}</td>
                                <td class="py-4 px-4 text-outline">{if item.is_visible { "YES" } else { "NO" }}</td>
                                <td class="py-4 px-4">
                                    <div class="flex space-x-4 opacity-0 group-hover:opacity-100 transition-opacity">
                                        <button on:click=move |_| set_modal_state.set(crate::components::admin_modal::ModalState::CaseStudy(Some(c.clone()))) class="text-secondary hover:underline uppercase text-xs">"Edit"</button>
                                        <button on:click=move |_| { let id = item.id; leptos::task::spawn_local(async move { if let Ok(_) = delete_case_study(id).await { set_refresh.set(refresh.get_untracked() + 1); } }); } class="text-error hover:underline uppercase text-xs">"Drop"</button>
                                    </div>
                                </td>
                            </tr>
                        }}).collect::<Vec<_>>().into_any(),
                        ResourceState::Loading => view! { <tr class="hidden"></tr> }.into_any(),
                        ResourceState::Error(_) => view! { <tr><td colspan="4" class="py-8 text-center text-error">"ERR_NO_DATA"</td></tr> }.into_any(),
                    }}
                </tbody>
            </table>
            }.into_any()
            }}
        </Transition>
    }
}

#[component]
pub fn HighlightTable() -> impl IntoView {
    use crate::b2b::{delete_highlight, get_highlights};
    let refresh = expect_context::<ReadSignal<i32>>();
    let set_refresh = expect_context::<WriteSignal<i32>>();
    let set_modal_state =
        expect_context::<WriteSignal<crate::components::admin_modal::ModalState>>();
    let data_res = Resource::new(move || refresh.get(), |_| get_highlights(false));

    view! {
        <Transition fallback=move || view! { <div class="jetbrains text-sm text-outline">"QUERYING_DB..."</div> }>
            {move || {
                let res = data_res.get();
                view! {
            <table class="w-full text-left jetbrains text-sm">
                <thead><tr class="text-outline border-b border-outline-variant/30">
                    <th class="py-4 px-4 font-normal tracking-widest uppercase">"Weight"</th>
                    <th class="py-4 px-4 font-normal tracking-widest uppercase">"Title"</th>
                    <th class="py-4 px-4 font-normal tracking-widest uppercase">"Visible"</th>
                    <th class="py-4 px-4 font-normal tracking-widest uppercase">"Actions"</th>
                </tr></thead>
                <tbody class="divide-y divide-outline-variant/20">
                    {match ResourceState::from(res) {
                        ResourceState::Ready(items) => items.into_iter().map(|item| { let c = item.clone(); view! {
                            <tr class="hover:bg-surface-container-high transition-colors group">
                                <td class="py-4 px-4 text-outline-variant">{item.display_order}</td>
                                <td class="py-4 font-bold text-primary">{item.title}</td>
                                <td class="py-4 px-4 text-outline">{if item.is_visible { "YES" } else { "NO" }}</td>
                                <td class="py-4 px-4">
                                    <div class="flex space-x-4 opacity-0 group-hover:opacity-100 transition-opacity">
                                        <button on:click=move |_| set_modal_state.set(crate::components::admin_modal::ModalState::Highlight(Some(c.clone()))) class="text-secondary hover:underline uppercase text-xs">"Edit"</button>
                                        <button on:click=move |_| { let id = item.id; leptos::task::spawn_local(async move { if let Ok(_) = delete_highlight(id).await { set_refresh.set(refresh.get_untracked() + 1); } }); } class="text-error hover:underline uppercase text-xs">"Drop"</button>
                                    </div>
                                </td>
                            </tr>
                        }}).collect::<Vec<_>>().into_any(),
                        ResourceState::Loading => view! { <tr class="hidden"></tr> }.into_any(),
                        ResourceState::Error(_) => view! { <tr><td colspan="4" class="py-8 text-center text-error">"ERR_NO_DATA"</td></tr> }.into_any(),
                    }}
                </tbody>
            </table>
            }.into_any()
            }}
        </Transition>
    }
}

// PasskeyRegistrationNudge was removed. PasskeyNudge (shared-ui) is used instead.
// The old implementation used js_sys::eval + setTimeout(500) + manual DOM binding
// which raced against WASM hydration and CDN script load timing. See Bug 3 in the
// engineering brief (2026-05-17) for the full root-cause analysis.



