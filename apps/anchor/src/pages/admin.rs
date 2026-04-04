use leptos::*;

use crate::auth::*;
use crate::components::admin_modal::*;

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen(module = "/public/webauthn.js")]
extern "C" {
    #[wasm_bindgen(catch)]
    async fn registerDevice(optionsJson: &str) -> Result<JsValue, JsValue>;

    #[wasm_bindgen(catch)]
    async fn authenticateDevice(optionsJson: &str) -> Result<JsValue, JsValue>;
}

#[component]
pub fn Admin() -> impl IntoView {
    let (is_authenticated, set_authenticated) = create_signal(false);
    let (active_tab, set_active_tab) = create_signal("DASHBOARD");
    let (username, set_username) = create_signal(String::new());
    let (setup_token, set_setup_token) = create_signal(String::new());
    let (is_loading, set_is_loading) = create_signal(false);
    let (auth_error, set_auth_error) = create_signal(String::new());

    let sys_init_res = create_resource(|| (), |_| is_system_initialized());

    let (modal_state, set_modal_state) = create_signal(ModalState::None);
    provide_context(modal_state);
    provide_context(set_modal_state);

    let (refresh, set_refresh) = create_signal(0);
    provide_context(refresh);
    provide_context(set_refresh);

    create_effect(move |_| {
        spawn_local(async move {
            if let Ok(true) = check_session().await {
                set_authenticated.set(true);
            }
        });
    });

    let login_action = create_action(move |_: &()| async move {
        let uname = username.get_untracked();
        if uname.is_empty() {
            set_auth_error.set("Identity Hash (Username) is required.".to_string());
            return;
        }
        set_is_loading.set(true);
        set_auth_error.set(String::new());

        match login_start(uname.clone()).await {
            Ok(_payload) => {
                #[cfg(target_arch = "wasm32")]
                {
                    if let Ok(val) = serde_json::from_str::<serde_json::Value>(&_payload) {
                        if let (Some(c_str), Some(o_str)) =
                            (val["challenge_id"].as_str(), val["options"].as_str())
                        {
                            if let Ok(challenge_id) = uuid::Uuid::parse_str(c_str) {
                                match authenticateDevice(o_str).await {
                                    Ok(cred_js) => {
                                        if let Some(cred_str) = cred_js.as_string() {
                                            match login_finish(uname, challenge_id, cred_str).await {
                                                Ok(_) => set_authenticated.set(true),
                                                Err(e) => set_auth_error.set(format!("Validation failed: {:?}", e)),
                                            }
                                        } else {
                                            set_auth_error.set("Invalid credential format from browser.".to_string());
                                        }
                                    },
                                    Err(_) => set_auth_error.set("Device challenge rejected. Cancelled or no matching passkey found.".to_string()),
                                }
                            } else {
                                set_auth_error.set("Internal error: Bad challenge ID".to_string());
                            }
                        } else {
                            set_auth_error
                                .set("Internal error: Malformed server payload".to_string());
                        }
                    } else {
                        set_auth_error.set("Internal error: JSON parse failed".to_string());
                    }
                }
            }
            Err(e) => set_auth_error.set(format!("Identity not recognized: {:?}", e)),
        }
        set_is_loading.set(false);
    });

    let register_action = create_action(move |_: &()| async move {
        let uname = username.get_untracked();
        if uname.is_empty() {
            set_auth_error.set("Identity Hash (Username) is required.".to_string());
            return;
        }
        set_is_loading.set(true);
        set_auth_error.set(String::new());

        let token = setup_token.get_untracked();
        let token_opt = if token.is_empty() { None } else { Some(token) };
        match register_start(uname.clone(), token_opt).await {
            Ok(_payload) => {
                #[cfg(target_arch = "wasm32")]
                {
                    if let Ok(val) = serde_json::from_str::<serde_json::Value>(&_payload) {
                        if let (Some(c_str), Some(o_str)) =
                            (val["challenge_id"].as_str(), val["options"].as_str())
                        {
                            if let Ok(challenge_id) = uuid::Uuid::parse_str(c_str) {
                                match registerDevice(o_str).await {
                                    Ok(cred_js) => {
                                        if let Some(cred_str) = cred_js.as_string() {
                                            match register_finish(uname, challenge_id, cred_str)
                                                .await
                                            {
                                                Ok(_) => set_authenticated.set(true),
                                                Err(e) => set_auth_error
                                                    .set(format!("Validation failed: {:?}", e)),
                                            }
                                        } else {
                                            set_auth_error.set(
                                                "Invalid credential format from browser."
                                                    .to_string(),
                                            );
                                        }
                                    }
                                    Err(_) => set_auth_error
                                        .set("Device setup rejected or cancelled.".to_string()),
                                }
                            } else {
                                set_auth_error.set("Internal error: Bad challenge ID".to_string());
                            }
                        } else {
                            set_auth_error
                                .set("Internal error: Malformed server payload".to_string());
                        }
                    } else {
                        set_auth_error.set("Internal error: JSON parse failed".to_string());
                    }
                }
            }
            Err(e) => set_auth_error.set(format!("Server refused registration: {:?}", e)),
        }
        set_is_loading.set(false);
    });

    view! {
        <main class="min-h-screen bg-surface-container-low text-on-surface flex flex-col pt-24 px-4 md:px-[8.5rem]">
            {move || if !is_authenticated.get() {
                view! {
                    <div class="flex-1 flex justify-center items-center">
                        <div class="w-full max-w-lg bg-surface-container-highest p-1 lg:p-1 blueprint-overlay">
                            <div class="bg-surface-container-lowest p-12">
                                <div class="inline-block bg-secondary-container/20 px-3 py-1 mb-6">
                                    <span class="font-label text-[0.6875rem] text-secondary font-bold tracking-tighter">"SECURE_ZONE // 0xAUTH"</span>
                                </div>
                                <h2 class="text-4xl font-extrabold text-primary mb-8 tracking-tight">"SYSTEM_CMS"</h2>

                                <div class="space-y-12">
                                    <div class="relative w-full group">
                                        <label class="jetbrains text-[0.65rem] uppercase tracking-[0.1em] text-outline text-left block mb-2">"Identity Hash"</label>
                                        <input
                                            type="text"
                                            placeholder="admin"
                                            on:input=move |ev| set_username.set(event_target_value(&ev))
                                            prop:value=username
                                            class="w-full bg-transparent border-none border-b-2 border-outline-variant focus:border-primary focus:ring-0 px-0 py-4 jetbrains text-lg text-on-surface transition-all placeholder:text-outline-variant/50"
                                        />
                                    </div>                                    <Suspense fallback=move || view! { <div class="hidden"></div> }>
                                        {move || {
                                            if !sys_init_res.get().unwrap_or(Ok(true)).unwrap_or(true) {
                                                view! {
                                                    <div class="relative w-full group mt-6">
                                                        <label class="jetbrains text-[0.65rem] uppercase tracking-[0.1em] text-outline text-left block mb-2">"Setup Token (First-Run Only)"</label>
                                                        <input
                                                            type="text"
                                                            placeholder="..."
                                                            on:input=move |ev| set_setup_token.set(event_target_value(&ev))
                                                            prop:value=setup_token
                                                            class="w-full bg-transparent border-none border-b-2 border-outline-variant focus:border-primary focus:ring-0 px-0 py-4 jetbrains text-lg text-on-surface transition-all placeholder:text-outline-variant/50"
                                                        />
                                                    </div>
                                                }.into_view()
                                            } else {
                                                view! { <div class="hidden"></div> }.into_view()
                                            }
                                        }}
                                    </Suspense>

                                    <div class="space-y-4 pt-6">
                                        <Show when=move || !auth_error.get().is_empty()>
                                            <div class="bg-error/10 border-l-4 border-error p-4 mb-4 text-error jetbrains text-sm font-medium">
                                                {move || auth_error.get()}
                                            </div>
                                        </Show>

                                        <button
                                            on:click=move |_| login_action.dispatch(())
                                            disabled=is_loading
                                            class="w-full bg-primary text-white py-6 jetbrains font-bold text-sm tracking-[0.2em] uppercase hover:bg-primary-container disabled:opacity-50 disabled:cursor-not-allowed transition-colors flex items-center justify-center gap-3"
                                        >
                                            <Show when=move || is_loading.get()>
                                                <span class="material-symbols-outlined animate-spin text-base">"progress_activity"</span>
                                            </Show>
                                            <span class="inline-block translate-y-[1px]">"Authenticate // Passkey"</span>
                                        </button>

                                        <Suspense fallback=move || view! { <div class="hidden"></div> }>
                                            {move || {
                                                if !sys_init_res.get().unwrap_or(Ok(true)).unwrap_or(true) {
                                                    view! {
                                                        <button
                                                            on:click=move |_| register_action.dispatch(())
                                                            disabled=is_loading
                                                            class="w-full border border-primary/20 text-primary py-4 jetbrains font-bold text-sm tracking-[0.2em] uppercase hover:bg-surface-container disabled:opacity-50 disabled:cursor-not-allowed transition-colors"
                                                        >
                                                            "Register Device"
                                                        </button>
                                                    }.into_view()
                                                } else {
                                                    view! { <div class="hidden"></div> }.into_view()
                                                }
                                            }}
                                        </Suspense>
                                    </div>
                                </div>
                            </div>
                        </div>
                    </div>
                }.into_view()
            } else {
                view! {
                    <div class="flex-1 flex flex-col md:flex-row gap-12 pb-24">
                        // Sidebar
                        <aside class="w-full md:w-64 shrink-0 space-y-2">
                            <div class="mb-12">
                                <span class="font-label text-[0.6875rem] text-outline font-bold tracking-widest uppercase block mb-4">"Navigation"</span>
                            {["DASHBOARD", "SERVICES", "CASE STUDIES", "HIGHLIGHTS", "MAILING LIST", "SETTINGS", "LEAD OPTIONS", "NAVIGATION", "FOOTER", "PAGE HEADERS", "BLOG", "RESUME PROFILES", "RESUME ENTRIES", "LANDING PAGES", "SECURITY"].iter().map(|&t| {
                                            let tab = t; // Capture `t` for the closure
                                            view! {
                                                <button
                                                    on:click=move |_| set_active_tab.set(tab)
                                                    class=move || format!(
                                                        "w-full text-left px-4 py-3 jetbrains text-sm font-bold tracking-wider transition-colors {}",
                                                        if active_tab.get() == tab {
                                                            "bg-primary text-on-primary"
                                                        } else {
                                                            "text-slate-500 hover:bg-surface-container-high dark:text-slate-400 dark:hover:bg-slate-800"
                                                        }
                                                    )
                                                >
                                                    {tab}
                                                </button>
                                            }
                                        }
                                    ).collect_view()}
                                </div>

                            <button
                                on:click=move |_| set_authenticated.set(false)
                                class="text-error text-xs jetbrains font-bold uppercase tracking-widest hover:underline"
                            >
                                "[ TERMINATE SESSION ]"
                            </button>
                        </aside>

                        // Main Content Area
                        <section class="flex-1 bg-surface-container-highest p-1 blueprint-overlay min-h-[600px]">
                            <div class="bg-surface-container-lowest h-full p-8 md:p-12 relative flex flex-col">
                                // Header
                                <div class="flex justify-between items-end border-b-2 border-outline-variant pb-6 mb-8">
                                    <div>
                                        <div class="inline-block bg-secondary-container/20 px-3 py-1 mb-4">
                                            <span class="font-label text-[0.6875rem] text-secondary font-bold tracking-tighter">"DATABASE // LIVE"</span>
                                        </div>
                                        <h2 class="text-3xl font-extrabold text-primary tracking-tight uppercase">
                                            {move || active_tab.get()}
                                        </h2>
                                    </div>
                                    <button
                                        on:click=move |_| {
                                            let state = match active_tab.get() {
                                                "SETTINGS" => ModalState::Settings,
                                                "SERVICES" => ModalState::Service(None),
                                                "CASE STUDIES" => ModalState::CaseStudy(None),
                                                "HIGHLIGHTS" => ModalState::Highlight(None),
                                                "BLOG" => ModalState::Post(None),
                                                "RESUME PROFILES" => ModalState::Profile(None),
                                                "RESUME ENTRIES" => ModalState::BaseEntry(None, None),
                                                "LANDING PAGES" => ModalState::LandingPage(None),
                                                "FOOTER" => ModalState::FooterItem(None),
                                                "PAGE HEADERS" => ModalState::PageHeader(None),
                                                "MAILING LIST" => ModalState::MailingList(None),
                                                "LEAD OPTIONS" => ModalState::LeadOption(None),
                                                "SECURITY" => ModalState::Passkey,
                                                _ => ModalState::None,
                                            };
                                            set_modal_state.set(state);
                                        }
                                        class="bg-primary text-on-primary px-8 py-4 jetbrains text-xs font-bold tracking-[0.2em] uppercase hover:bg-primary-container transition-colors"
                                    >
                                        {move || if active_tab.get() == "SETTINGS" { "EDIT VALUES" } else if active_tab.get() == "DASHBOARD" { "REFRESH" } else { "NEW ENTRY +" }}
                                    </button>
                                </div>

                                // Datagrid View
                                <div class="flex-1 overflow-x-auto">
                                    {move || match active_tab.get() {
                                        "DASHBOARD" => view! { <DashboardView /> }.into_view(),
                                        "SERVICES" => view! { <ServiceTable /> }.into_view(),
                                        "CASE STUDIES" => view! { <CaseStudyTable /> }.into_view(),
                                        "HIGHLIGHTS" => view! { <HighlightTable /> }.into_view(),
                                        "MAILING LIST" => view! { <MailingListTable /> }.into_view(),
                                        "LEAD OPTIONS" => view! { <LeadOptionTable /> }.into_view(),
                                        "NAVIGATION" => view! { <NavTable /> }.into_view(),
                                        "FOOTER" => view! { <FooterTable /> }.into_view(),
                                        "PAGE HEADERS" => view! { <PageHeaderTable /> }.into_view(),
                                        "SETTINGS" => view! { <SettingsReadView /> }.into_view(),
                                        "BLOG" => view! { <PostTable /> }.into_view(),
                                        "RESUME PROFILES" => view! { <ResumeProfileTable /> }.into_view(),
                                        "RESUME ENTRIES" => view! { <BaseResumeEntryTable /> }.into_view(),
                                        "LANDING PAGES" => view! { <LandingPageTable /> }.into_view(),
                                        "SECURITY" => view! { <PasskeyTable /> }.into_view(),
                                        _ => view! {
                                            <div class="h-64 flex items-center justify-center border-2 border-dashed border-outline-variant text-outline">
                                                <span class="jetbrains text-sm">"MODULE_OFFLINE"</span>
                                            </div>
                                        }.into_view(),
                                    }}
                                </div>
                            </div>
                            <AdminEditorModal />
                        </section>
                    </div>
                }.into_view()
            }}
        </main>
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
    let signups: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM users WHERE tenant_id IS NOT DISTINCT FROM $1")
        .bind(tenant.0)
        .fetch_one(&state.pool)
        .await
        .unwrap_or(0);
    let mailing: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM mailing_list WHERE tenant_id IS NOT DISTINCT FROM $1")
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
    let headers_resource = create_resource(move || refresh.get(), |_| get_all_page_headers());

    view! {
        <Transition fallback=move || view! { <div class="jetbrains text-sm text-outline">"QUERYING_DB..."</div> }>
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
                    {move || match headers_resource.get() {
                        Some(Ok(headers)) => headers.into_iter().map(|h| {
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
                        }).collect_view(),
                        _ => view! { <tr><td colspan="4" class="py-8 text-center text-error">"ERR_NO_DATA"</td></tr> }.into_view(),
                    }}
                </tbody>
            </table>
        </Transition>
    }
}

#[component]
fn DashboardView() -> impl IntoView {
    let refresh = expect_context::<ReadSignal<i32>>();
    let stats_resource = create_resource(move || refresh.get(), |_| get_dashboard_stats());

    view! {
        <Transition fallback=move || view! { <div class="jetbrains text-sm text-outline">"COMPILING_TELEMETRY..."</div> }>
            {move || match stats_resource.get() {
                Some(Ok(stats)) => view! {
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
                            <span class="jetbrains text-[0.65rem] uppercase text-outline tracking-wider mb-4">"Total Mailing List & Leads"</span>
                            <span class="text-5xl font-extrabold text-secondary">{stats.total_mailing_list}</span>
                        </div>
                        <div class="p-8 border border-outline-variant/30 flex flex-col justify-between">
                            <span class="jetbrains text-[0.65rem] uppercase text-outline tracking-wider mb-4">"Admin Identities Registered"</span>
                            <span class="text-5xl font-extrabold text-on-surface">{stats.total_signups}</span>
                        </div>
                    </div>
                }.into_view(),
                _ => view! { <div class="text-error">"Failed to load settings"</div> }.into_view()
            }}
        </Transition>
    }
}

#[component]
fn SettingsReadView() -> impl IntoView {
    let refresh = expect_context::<ReadSignal<i32>>();
    let settings_res = create_resource(
        move || refresh.get(),
        |_| crate::pages::landing::get_site_settings(),
    );

    view! {
        <Transition fallback=move || view! { <div class="jetbrains text-sm text-outline">"LOADING SETTINGS..."</div> }>
            {move || match settings_res.get() {
                Some(Ok(s)) => view! {
                    <div class="grid grid-cols-1 gap-4 text-left jetbrains text-sm">

                        <div class="grid grid-cols-3 border-b-2 border-outline-variant/30 pb-2 mb-4">
                            <div class="font-label text-[0.65rem] uppercase tracking-widest text-outline">"KEY"</div>
                            <div class="col-span-2 font-label text-[0.65rem] uppercase tracking-widest text-outline">"VALUE"</div>
                        </div>

                        // Hero
                        <div class="grid grid-cols-3 py-2 border-b border-outline-variant/10 hover:bg-surface-container/30">
                            <div class="text-outline-variant uppercase tracking-widest text-xs">"CURRENT FOCUS"</div>
                            <div class="col-span-2 text-on-surface font-medium">{&s.current_focus}</div>
                        </div>
                        <div class="grid grid-cols-3 py-2 border-b border-outline-variant/10 hover:bg-surface-container/30">
                            <div class="text-outline-variant uppercase tracking-widest text-xs">"STATUS"</div>
                            <div class="col-span-2 text-on-surface font-medium"><div class="inline-flex items-center gap-2"><div class="w-2 h-2 rounded-full" style=format!("background-color: {};", s.status_color)></div>{&s.status}</div></div>
                        </div>
                        <div class="grid grid-cols-3 py-2 border-b border-outline-variant/10 hover:bg-surface-container/30">
                            <div class="text-outline-variant uppercase tracking-widest text-xs">"SUBTITLE"</div>
                            <div class="col-span-2 text-on-surface font-medium truncate">{&s.hero_subtitle}</div>
                        </div>
                        <div class="grid grid-cols-3 py-2 border-b border-outline-variant/10 hover:bg-surface-container/30">
                            <div class="text-outline-variant uppercase tracking-widest text-xs">"QUOTE"</div>
                            <div class="col-span-2 text-on-surface font-medium truncate">{&s.hero_quote}</div>
                        </div>

                        // Global
                        <div class="grid grid-cols-3 py-2 border-b border-primary/20 hover:bg-surface-container/30 mt-4">
                            <div class="text-primary uppercase tracking-widest text-xs">"SITE TITLE"</div>
                            <div class="col-span-2 text-on-surface font-bold">{&s.site_title}</div>
                        </div>
                        <div class="grid grid-cols-3 py-2 border-b border-outline-variant/10 hover:bg-surface-container/30">
                            <div class="text-primary uppercase tracking-widest text-xs">"WEBHOOK URL"</div>
                            <div class="col-span-2 text-on-surface font-mono text-xs">{&s.webhook_url}</div>
                        </div>
                        <div class="grid grid-cols-3 py-2 border-b border-outline-variant/10 hover:bg-surface-container/30">
                            <div class="text-primary uppercase tracking-widest text-xs">"ADMIN NOTIFICATION"</div>
                            <div class="col-span-2 text-on-surface font-mono text-xs">{&s.admin_email}</div>
                        </div>

                        // Lead Capture
                        <div class="grid grid-cols-3 py-2 border-b border-secondary/20 hover:bg-surface-container/30 mt-4">
                            <div class="text-secondary uppercase tracking-widest text-xs">"LC TITLE"</div>
                            <div class="col-span-2 text-on-surface font-medium">{&s.lc_title}</div>
                        </div>
                        <div class="grid grid-cols-3 py-2 border-b border-outline-variant/10 hover:bg-surface-container/30">
                            <div class="text-secondary uppercase tracking-widest text-xs">"LC DESC"</div>
                            <div class="col-span-2 text-on-surface font-medium truncate">{&s.lc_desc}</div>
                        </div>
                        <div class="grid grid-cols-3 py-2 border-b border-outline-variant/10 hover:bg-surface-container/30">
                            <div class="text-secondary uppercase tracking-widest text-xs">"LC BTN"</div>
                            <div class="col-span-2 text-on-surface font-medium">{&s.lc_btn}</div>
                        </div>


                        // Landing Pages Settings
                        <div class="grid grid-cols-3 py-2 border-b border-outline-variant/10 hover:bg-surface-container/30 mt-4">
                            <div class="text-outline uppercase tracking-widest text-xs">"BOOKING URL"</div>
                            <div class="col-span-2 text-on-surface font-medium truncate">{&s.booking_url}</div>
                        </div>
                        <div class="grid grid-cols-3 py-2 border-b border-outline-variant/10 hover:bg-surface-container/30 mt-4">
                            <div class="text-outline uppercase tracking-widest text-xs">"TERMS HTML (MD)"</div>
                            <div class="col-span-2 text-on-surface font-mono text-xs truncate max-h-24 overflow-hidden">{&s.terms_html}</div>
                        </div>
                        <div class="grid grid-cols-3 py-2 border-b border-outline-variant/10 hover:bg-surface-container/30 mt-4">
                            <div class="text-outline uppercase tracking-widest text-xs">"PRIVACY HTML (MD)"</div>
                            <div class="col-span-2 text-on-surface font-mono text-xs truncate max-h-24 overflow-hidden">{&s.privacy_html}</div>
                        </div>

                        // Social Media Links
                        <div class="grid grid-cols-3 py-2 border-b border-outline-variant/10 hover:bg-surface-container/30 mt-4">
                            <div class="text-outline uppercase tracking-widest text-xs">"GITHUB URL"</div>
                            <div class="col-span-2 text-on-surface font-medium truncate">{&s.github_url}</div>
                        </div>
                        <div class="grid grid-cols-3 py-2 border-b border-outline-variant/10 hover:bg-surface-container/30">
                            <div class="text-outline uppercase tracking-widest text-xs">"X (TWITTER) URL"</div>
                            <div class="col-span-2 text-on-surface font-medium truncate">{&s.x_url}</div>
                        </div>
                        <div class="grid grid-cols-3 py-2 border-b border-outline-variant/10 hover:bg-surface-container/30">
                            <div class="text-outline uppercase tracking-widest text-xs">"LINKEDIN URL"</div>
                            <div class="col-span-2 text-on-surface font-medium truncate">{&s.linkedin_url}</div>
                        </div>

                        // Global B2B Settings
                        <div class="grid grid-cols-3 py-2 border-b border-outline-variant/10 hover:bg-surface-container/30 mt-4 bg-tertiary/10 p-2">
                            <div class="text-tertiary uppercase tracking-widest text-xs font-bold">"B2B CONSULTING MODE"</div>
                            <div class="col-span-2 text-on-surface font-medium">
                                {if s.b2b_enabled { "ENABLED (PUBLIC)" } else { "STEALTH (HIDDEN)" }}
                            </div>
                        </div>

                    </div>
                }.into_view(),
                _ => view! { <div class="text-error">"Failed to load settings"</div> }.into_view()
            }}
        </Transition>
    }
}

#[component]
fn ResumeProfileTable() -> impl IntoView {
    use crate::resume_engine::{delete_resume_profile, download_resume, get_resume_profiles};
    let refresh = expect_context::<ReadSignal<i32>>();
    let set_refresh = expect_context::<WriteSignal<i32>>();
    let set_modal_state =
        expect_context::<WriteSignal<crate::components::admin_modal::ModalState>>();

    let items_res = create_resource(move || refresh.get(), |_| get_resume_profiles());

    view! {
        <Transition fallback=move || view! { <div class="jetbrains text-sm text-outline">"LOADING..."</div> }>
            <table class="w-full text-left border-collapse">
                <thead>
                    <tr class="border-b-2 border-outline-variant/30">
                        <th class="py-4 font-label text-[0.65rem] uppercase tracking-widest text-outline">"ID"</th>
                        <th class="py-4 font-label text-[0.65rem] uppercase tracking-widest text-outline">"PROFILE NAME"</th>
                        <th class="py-4 font-label text-[0.65rem] uppercase tracking-widest text-outline text-right">"ACTIONS"</th>
                    </tr>
                </thead>
                <tbody class="jetbrains text-sm">
                    {move || match items_res.get() {
                        Some(Ok(items)) => items.into_iter().map(|item| {
                            let id_val = item.id;
                            let clone_item = item.clone();
                            view! {
                                <tr class="border-b border-outline-variant/10 hover:bg-surface-container/50 transition-colors">
                                    <td class="py-4 text-outline-variant font-medium">#{id_val}</td>
                                    <td class="py-4 font-bold text-on-surface">{&item.name}</td>
                                    <td class="py-4 text-right space-x-4">
                                        <button
                                            on:click=move |_| {
                                                spawn_local(async move {
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
                                                spawn_local(async move {
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
                                                                let document = leptos::document();
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
                                                spawn_local(async move {
                                                    let _ = delete_resume_profile(id_val).await;
                                                    set_refresh.set(refresh.get_untracked() + 1);
                                                });
                                            }
                                            class="text-error hover:text-error/80 font-medium tracking-wide"
                                        >"[DEL]"</button>
                                    </td>
                                </tr>
                            }
                        }).collect_view(),
                        _ => view! { <tr><td colspan="3" class="py-8 text-center text-error">"ERR_NO_DATA"</td></tr> }.into_view(),
                    }}
                </tbody>
            </table>
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

    let items_res = create_resource(move || refresh.get(), |_| get_all_base_entries());

    view! {
        <Transition fallback=move || view! { <div class="jetbrains text-sm text-outline">"LOADING DATA..."</div> }>
            {move || match items_res.get() {
                Some(Ok(items)) => {
                    if items.is_empty() {
                        view! { <div class="py-8 text-center text-outline-variant">"NO ENTRIES IN DATABASE"</div> }.into_view()
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
                                view! { <div class="hidden"></div> }.into_view()
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
                                                            <td class="py-4 text-outline-variant font-medium">#{id_val}</td>
                                                            <td class="py-4 font-bold text-on-surface truncate">{&item.title}</td>
                                                            <td class="py-4 text-right space-x-4">
                                                                <button on:click=move |_| set_modal_state.set(crate::components::admin_modal::ModalState::BaseEntry(Some(clone_item.clone()), Some(clone_item.category))) class="text-secondary hover:text-on-secondary-fixed-variant font-medium tracking-wide">"[EDIT]"</button>
                                                                <button
                                                                    on:click=move |_| {
                                                                        spawn_local(async move {
                                                                            let _ = delete_base_entry(id_val).await;
                                                                            set_refresh.set(refresh.get_untracked() + 1);
                                                                        });
                                                                    }
                                                                    class="text-error hover:text-error/80 font-medium tracking-wide"
                                                                >"[DEL]"</button>
                                                            </td>
                                                        </tr>
                                                    }
                                                }).collect_view()}
                                            </tbody>
                                        </table>
                                    </div>
                                }.into_view()
                            }
                        }).collect_view()
                    }
                },
                _ => view! { <div class="py-8 text-center text-error">"ERR_NO_DATA"</div> }.into_view(),
            }}
        </Transition>
    }
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq)]
pub struct MailingListRecord {
    pub id: i32,
    pub email: String,
    pub list_type: String,
    pub preferences: String,
    pub created_at: String,
}

#[server(GetMailingList, "/api")]
pub async fn get_mailing_list() -> Result<Vec<MailingListRecord>, ServerFnError> {
    use axum::Extension;
    use leptos_axum::extract;
    use sqlx::Row;
    let Extension(state) = extract::<Extension<crate::state::AppState>>().await?;
    let Extension(tenant) = extract::<Extension<crate::state::TenantContext>>().await?;

    let rows = sqlx::query("SELECT id, email, list_type, preferences::text as prefs, created_at FROM mailing_list WHERE tenant_id IS NOT DISTINCT FROM $1 ORDER BY created_at DESC")
        .bind(tenant.0)
        .fetch_all(&state.pool).await?;

    let mut records = Vec::new();
    for row in rows {
        let created_at: chrono::DateTime<chrono::Utc> = row.get("created_at");
        records.push(MailingListRecord {
            id: row.get("id"),
            email: row.get("email"),
            list_type: row.get("list_type"),
            preferences: row.get("prefs"),
            created_at: created_at.format("%Y-%m-%d %H:%M").to_string(),
        });
    }

    Ok(records)
}

#[server(DeleteMailingList, "/api")]
pub async fn delete_mailing_list(id: i32) -> Result<(), ServerFnError> {
    use crate::auth::check_session;
    use axum::Extension;
    use leptos_axum::extract;
    if !check_session().await.unwrap_or(false) {
        return Err(ServerFnError::ServerError("Unauthorized".into()));
    }
    let Extension(state) = extract::<Extension<crate::state::AppState>>().await?;
    let Extension(tenant) = extract::<Extension<crate::state::TenantContext>>().await?;
    sqlx::query("DELETE FROM mailing_list WHERE id = $1 AND tenant_id IS NOT DISTINCT FROM $2")
        .bind(id)
        .bind(tenant.0)
        .execute(&state.pool)
        .await?;
    Ok(())
}

#[component]
fn LeadOptionTable() -> impl IntoView {
    use crate::pages::landing::{delete_lead_option, get_all_lead_options};
    let refresh = expect_context::<ReadSignal<i32>>();
    let set_refresh = expect_context::<WriteSignal<i32>>();
    let set_modal_state =
        expect_context::<WriteSignal<crate::components::admin_modal::ModalState>>();

    let items_res = create_resource(move || refresh.get(), |_| get_all_lead_options());

    view! {
        <Transition fallback=move || view! { <div class="jetbrains text-sm text-outline">"LOADING DATA..."</div> }>
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
                    {move || match items_res.get() {
                        Some(Ok(items)) => items.into_iter().map(|item| {
                            let id_val = item.id;
                            let clone_item = item.clone();
                            view! {
                                <tr class="border-b border-outline-variant/10 hover:bg-surface-container/50 transition-colors">
                                    <td class="py-4 text-outline-variant font-medium">#{id_val}</td>
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
                                                spawn_local(async move {
                                                    let _ = delete_lead_option(id_val).await;
                                                    set_refresh.set(refresh.get_untracked() + 1);
                                                });
                                            }
                                            class="text-error hover:text-error/80 font-medium tracking-wide"
                                        >"[DEL]"</button>
                                    </td>
                                </tr>
                            }
                        }).collect_view(),
                        _ => view! { <tr><td colspan="6" class="py-8 text-center text-error">"ERR_NO_DATA"</td></tr> }.into_view(),
                    }}
                </tbody>
            </table>
        </Transition>
    }
}

#[component]
fn MailingListTable() -> impl IntoView {
    let refresh = expect_context::<ReadSignal<i32>>();
    let list_resource = create_resource(move || refresh.get(), |_| get_mailing_list());

    view! {
        <Transition fallback=move || view! { <div class="jetbrains text-sm text-outline">"QUERYING_DB..."</div> }>
            <table class="w-full text-left jetbrains text-sm">
                <thead>
                    <tr class="text-outline border-b border-outline-variant/30">
                        <th class="py-4 px-4 font-normal tracking-widest uppercase">"Email"</th>
                        <th class="py-4 px-4 font-normal tracking-widest uppercase">"Type"</th>
                        <th class="py-4 px-4 font-normal tracking-widest uppercase">"Preferences"</th>
                        <th class="py-4 px-4 font-normal tracking-widest uppercase">"Timestamp"</th>
                        <th class="py-4 px-4 font-normal tracking-widest uppercase">"Actions"</th>
                    </tr>
                </thead>
                <tbody class="divide-y divide-outline-variant/20">
                    {move || match list_resource.get() {
                        Some(Ok(items)) => items.into_iter().map(|i| view! {
                            <tr class="hover:bg-surface-container-high transition-colors group">
                                <td class="py-4 px-4 font-bold text-primary">{i.email}</td>
                                <td class="py-4 px-4 text-on-surface">{i.list_type}</td>
                                <td class="py-4 px-4 text-outline truncate max-w-[200px]" title=i.preferences.clone()>{i.preferences}</td>
                                <td class="py-4 px-4 text-outline-variant">{i.created_at}</td>
                                <td class="py-4 px-4">
                                    <div class="flex space-x-4 opacity-0 group-hover:opacity-100 transition-opacity">
                                        <button
                                            on:click=move |_| {
                                                let id = i.id;
                                                let r = refresh;
                                                spawn_local(async move {
                                                    if let Ok(_) = delete_mailing_list(id).await {
                                                        expect_context::<WriteSignal<i32>>().set(r.get_untracked() + 1);
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
                        }).collect_view(),
                        _ => view! { <tr><td colspan="5" class="py-8 text-center text-error">"ERR_NO_DATA"</td></tr> }.into_view(),
                    }}
                </tbody>
            </table>
        </Transition>
    }
}

#[component]
fn PostTable() -> impl IntoView {
    use crate::pages::blog::get_posts;
    let refresh = expect_context::<ReadSignal<i32>>();
    let set_refresh = expect_context::<WriteSignal<i32>>();
    let set_modal_state = expect_context::<WriteSignal<ModalState>>();
    let posts_resource = create_resource(move || refresh.get(), |_| get_posts());

    view! {
        <Transition fallback=move || view! { <div class="jetbrains text-sm text-outline">"QUERYING_DB..."</div> }>
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
                    {move || match posts_resource.get() {
                        Some(Ok(posts)) => posts.into_iter().map(|p| {
                            let p_clone = p.clone();
                            let del_id = p.id.parse::<i32>().unwrap_or(0);
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
                                                spawn_local(async move {
                                                    if let Ok(_) = crate::pages::blog::delete_post(del_id).await {
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
                        }).collect_view(),
                        _ => view! { <tr><td colspan="4" class="py-8 text-center text-error">"ERR_NO_DATA"</td></tr> }.into_view(),
                    }}
                </tbody>
            </table>
        </Transition>
    }
}

#[component]
fn PasskeyTable() -> impl IntoView {
    use crate::auth::get_users;
    let refresh = expect_context::<ReadSignal<i32>>();
    let set_refresh = expect_context::<WriteSignal<i32>>();
    let users_resource = create_resource(move || refresh.get(), |_| get_users());

    view! {
        <Transition fallback=move || view! { <div class="jetbrains text-sm text-outline">"QUERYING_DB..."</div> }>
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
                    {move || match users_resource.get() {
                        Some(Ok(users)) => users.into_iter().map(|u| view! {
                            <tr class="hover:bg-surface-container-high transition-colors group">
                                <td class="py-4 px-4 text-outline-variant">"#" {u.id}</td>
                                <td class="py-4 px-4 font-bold text-primary">{u.username}</td>
                                <td class="py-4 px-4 text-outline">{u.created_at}</td>
                                <td class="py-4 px-4">
                                    <div class="flex space-x-4 opacity-0 group-hover:opacity-100 transition-opacity">
                                        <button
                                            on:click=move |_| {
                                                let id = u.id;
                                                spawn_local(async move {
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
                        }).collect_view(),
                        _ => view! { <tr><td colspan="4" class="py-8 text-center text-error">"ERR_NO_DATA"</td></tr> }.into_view(),
                    }}
                </tbody>
            </table>
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
    let pages_resource = create_resource(move || refresh.get(), |_| get_all_landing_pages());

    view! {
        <Transition fallback=move || view! { <div class="jetbrains text-sm text-outline">"QUERYING_DB..."</div> }>
            <table class="w-full text-left jetbrains text-sm">
                <thead>
                    <tr class="text-outline border-b border-outline-variant/30">
                        <th class="py-4 px-4 font-normal tracking-widest uppercase">"Slug"</th>
                        <th class="py-4 px-4 font-normal tracking-widest uppercase">"Title"</th>
                        <th class="py-4 px-4 font-normal tracking-widest uppercase">"Actions"</th>
                    </tr>
                </thead>
                <tbody class="divide-y divide-outline-variant/20">
                    {move || match pages_resource.get() {
                        Some(Ok(pages)) => pages.into_iter().map(|p| {
                            let p_clone = p.clone();
                            view! {
                            <tr class="hover:bg-surface-container-high transition-colors group">
                                <td class="py-4 px-4 font-bold text-primary">"/" {p.slug}</td>
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
                                                let id = p.id;
                                                spawn_local(async move {
                                                    if let Ok(_) = delete_landing_page(id).await {
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
                        }}).collect_view(),
                        _ => view! { <tr><td colspan="3" class="py-8 text-center text-error">"ERR_NO_DATA"</td></tr> }.into_view(),
                    }}
                </tbody>
            </table>
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
    let nav_resource = create_resource(move || refresh.get(), |_| get_all_nav_items());

    view! {
        <Transition fallback=move || view! { <div class="jetbrains text-sm text-outline">"QUERYING_DB..."</div> }>
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
                    {move || match nav_resource.get() {
                        Some(Ok(items)) => items.into_iter().map(|n| {
                            let n_clone = n.clone();
                            view! {
                            <tr class="hover:bg-surface-container-high transition-colors group">
                                <td class="py-4 px-4 text-outline-variant">{n.display_order}</td>
                                <td class="py-4 px-4 text-outline font-medium">"#" {n.id}</td>
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
                                                spawn_local(async move {
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
                        }}).collect_view(),
                        _ => view! { <tr><td colspan="4" class="py-8 text-center text-error">"ERR_NO_DATA"</td></tr> }.into_view(),
                    }}
                </tbody>
            </table>
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
    let footer_resource = create_resource(move || refresh.get(), |_| get_all_footer_items());

    view! {
        <Transition fallback=move || view! { <div class="jetbrains text-sm text-outline">"QUERYING_DB..."</div> }>
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
                    {move || match footer_resource.get() {
                        Some(Ok(items)) => items.into_iter().map(|n| {
                            let n_clone = n.clone();
                            view! {
                            <tr class="hover:bg-surface-container-high transition-colors group">
                                <td class="py-4 px-4 text-outline-variant">{n.display_order}</td>
                                <td class="py-4 px-4 font-bold text-primary">
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
                                                spawn_local(async move {
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
                        }}).collect_view(),
                        _ => view! { <tr><td colspan="4" class="py-8 text-center text-error">"ERR_NO_DATA"</td></tr> }.into_view(),
                    }}
                </tbody>
            </table>
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
    let data_res = create_resource(move || refresh.get(), |_| get_services(false));

    view! {
        <Transition fallback=move || view! { <div class="jetbrains text-sm text-outline">"QUERYING_DB..."</div> }>
            <table class="w-full text-left jetbrains text-sm">
                <thead><tr class="text-outline border-b border-outline-variant/30">
                    <th class="py-4 px-4 font-normal tracking-widest uppercase">"Weight"</th>
                    <th class="py-4 px-4 font-normal tracking-widest uppercase">"Title"</th>
                    <th class="py-4 px-4 font-normal tracking-widest uppercase">"Visible"</th>
                    <th class="py-4 px-4 font-normal tracking-widest uppercase">"Actions"</th>
                </tr></thead>
                <tbody class="divide-y divide-outline-variant/20">
                    {move || match data_res.get() {
                        Some(Ok(items)) => items.into_iter().map(|item| { let c = item.clone(); view! {
                            <tr class="hover:bg-surface-container-high transition-colors group">
                                <td class="py-4 px-4 text-outline-variant">{item.display_order}</td>
                                <td class="py-4 px-4 font-bold text-primary">{item.title}</td>
                                <td class="py-4 px-4 text-outline">{if item.is_visible { "YES" } else { "NO" }}</td>
                                <td class="py-4 px-4">
                                    <div class="flex space-x-4 opacity-0 group-hover:opacity-100 transition-opacity">
                                        <button on:click=move |_| set_modal_state.set(crate::components::admin_modal::ModalState::Service(Some(c.clone()))) class="text-secondary hover:underline uppercase text-xs">"Edit"</button>
                                        <button on:click=move |_| { let id = item.id; spawn_local(async move { if let Ok(_) = delete_service(id).await { set_refresh.set(refresh.get_untracked() + 1); } }); } class="text-error hover:underline uppercase text-xs">"Drop"</button>
                                    </div>
                                </td>
                            </tr>
                        }}).collect_view(),
                        _ => view! { <tr><td colspan="4" class="py-8 text-center text-error">"ERR_NO_DATA"</td></tr> }.into_view(),
                    }}
                </tbody>
            </table>
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
    let data_res = create_resource(move || refresh.get(), |_| get_case_studies(false));

    view! {
        <Transition fallback=move || view! { <div class="jetbrains text-sm text-outline">"QUERYING_DB..."</div> }>
            <table class="w-full text-left jetbrains text-sm">
                <thead><tr class="text-outline border-b border-outline-variant/30">
                    <th class="py-4 px-4 font-normal tracking-widest uppercase">"Weight"</th>
                    <th class="py-4 px-4 font-normal tracking-widest uppercase">"Client"</th>
                    <th class="py-4 px-4 font-normal tracking-widest uppercase">"Visible"</th>
                    <th class="py-4 px-4 font-normal tracking-widest uppercase">"Actions"</th>
                </tr></thead>
                <tbody class="divide-y divide-outline-variant/20">
                    {move || match data_res.get() {
                        Some(Ok(items)) => items.into_iter().map(|item| { let c = item.clone(); view! {
                            <tr class="hover:bg-surface-container-high transition-colors group">
                                <td class="py-4 px-4 text-outline-variant">{item.display_order}</td>
                                <td class="py-4 px-4 font-bold text-primary">{item.client_name}</td>
                                <td class="py-4 px-4 text-outline">{if item.is_visible { "YES" } else { "NO" }}</td>
                                <td class="py-4 px-4">
                                    <div class="flex space-x-4 opacity-0 group-hover:opacity-100 transition-opacity">
                                        <button on:click=move |_| set_modal_state.set(crate::components::admin_modal::ModalState::CaseStudy(Some(c.clone()))) class="text-secondary hover:underline uppercase text-xs">"Edit"</button>
                                        <button on:click=move |_| { let id = item.id; spawn_local(async move { if let Ok(_) = delete_case_study(id).await { set_refresh.set(refresh.get_untracked() + 1); } }); } class="text-error hover:underline uppercase text-xs">"Drop"</button>
                                    </div>
                                </td>
                            </tr>
                        }}).collect_view(),
                        _ => view! { <tr><td colspan="4" class="py-8 text-center text-error">"ERR_NO_DATA"</td></tr> }.into_view(),
                    }}
                </tbody>
            </table>
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
    let data_res = create_resource(move || refresh.get(), |_| get_highlights(false));

    view! {
        <Transition fallback=move || view! { <div class="jetbrains text-sm text-outline">"QUERYING_DB..."</div> }>
            <table class="w-full text-left jetbrains text-sm">
                <thead><tr class="text-outline border-b border-outline-variant/30">
                    <th class="py-4 px-4 font-normal tracking-widest uppercase">"Weight"</th>
                    <th class="py-4 px-4 font-normal tracking-widest uppercase">"Title"</th>
                    <th class="py-4 px-4 font-normal tracking-widest uppercase">"Visible"</th>
                    <th class="py-4 px-4 font-normal tracking-widest uppercase">"Actions"</th>
                </tr></thead>
                <tbody class="divide-y divide-outline-variant/20">
                    {move || match data_res.get() {
                        Some(Ok(items)) => items.into_iter().map(|item| { let c = item.clone(); view! {
                            <tr class="hover:bg-surface-container-high transition-colors group">
                                <td class="py-4 px-4 text-outline-variant">{item.display_order}</td>
                                <td class="py-4 px-4 font-bold text-primary">{item.title}</td>
                                <td class="py-4 px-4 text-outline">{if item.is_visible { "YES" } else { "NO" }}</td>
                                <td class="py-4 px-4">
                                    <div class="flex space-x-4 opacity-0 group-hover:opacity-100 transition-opacity">
                                        <button on:click=move |_| set_modal_state.set(crate::components::admin_modal::ModalState::Highlight(Some(c.clone()))) class="text-secondary hover:underline uppercase text-xs">"Edit"</button>
                                        <button on:click=move |_| { let id = item.id; spawn_local(async move { if let Ok(_) = delete_highlight(id).await { set_refresh.set(refresh.get_untracked() + 1); } }); } class="text-error hover:underline uppercase text-xs">"Drop"</button>
                                    </div>
                                </td>
                            </tr>
                        }}).collect_view(),
                        _ => view! { <tr><td colspan="4" class="py-8 text-center text-error">"ERR_NO_DATA"</td></tr> }.into_view(),
                    }}
                </tbody>
            </table>
        </Transition>
    }
}
