use leptos::prelude::*;
use leptos_router::hooks::use_query_map;

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

/// Returned by the `auth_resource` async block.
/// Carries the magic-link flag through to view-layer derived closures.
#[derive(Clone, PartialEq, serde::Serialize, serde::Deserialize)]
enum AuthStep {
    /// Session validated (or magic link verified). `from_magic_link` triggers the
    /// passkey-registration nudge shown immediately after a magic-link login.
    Authenticated { from_magic_link: bool },
    /// No valid session — render the login form.
    Unauthenticated,
}

/// Three-state auth enum for the view match.
/// `Pending` is only reached when the resource hasn't resolved yet;
/// in practice the `<Suspense>` fallback covers that state.
#[derive(Clone, PartialEq)]
enum AuthState {
    Pending,
    No,
    Yes,
}

/// Self-contained login UI rendered when the session is unauthenticated.
///
/// Owns its own reactive signals (`use_email`, `username`, loading/error/countdown)
/// so that toggling between passkey and email flows is fully isolated from the
/// outer `Admin` auth-state match closure.
///
/// # Why `move || if` instead of `<Show>`
/// During SSR, `use_email` is `false` → only the passkey branch is emitted into HTML.
/// `<Show fallback=...>` cannot insert the email-form nodes post-hydration because
/// those nodes were never part of the SSR output — Leptos 0.6's hydration cursor
/// panics or silently fails reconciling them.  A `move || if/else` expression is a
/// single `dyn_child` slot: Leptos replaces the entire slot's content on each toggle,
/// which works correctly whether or not both branches were rendered during SSR.
#[component]
fn LoginPanel() -> impl IntoView {
    let (use_email, set_use_email)     = signal(false);
    let (username, set_username)       = signal(String::new());
    let (is_loading, set_is_loading)   = signal(false);
    let (auth_error, set_auth_error)   = signal(String::new());
    let (countdown, set_countdown)     = signal(0i32);

    let login_action = Action::new(move |_: &()| async move {
        let uname = username.get_untracked();
        if uname.is_empty() {
            set_auth_error.set("Email is required.".to_string());
            return;
        }
        set_is_loading.set(true);
        set_auth_error.set(String::new());
        match request_magic_link(uname).await {
            Ok(_) => {
                set_auth_error.set("Magic link sent! Check your email.".to_string());
                set_countdown.set(60);
                #[cfg(feature = "hydrate")]
                leptos::task::spawn_local(async move {
                    use std::time::Duration;
                    while countdown.get_untracked() > 0 {
                        let (tx, rx) = futures::channel::oneshot::channel::<()>();
                        set_timeout_with_handle(
                            move || { let _ = tx.send(()); },
                            Duration::from_secs(1),
                        ).expect("failed to set timeout");
                        rx.await.unwrap();
                        set_countdown.update(|c| *c -= 1);
                    }
                });
            }
            Err(e) => set_auth_error.set(format!("Login failed: {:?}", e)),
        }
        set_is_loading.set(false);
    });

    view! {
        <div class="flex-1 flex justify-center items-center">
            <div class="w-full max-w-lg bg-surface-container-highest p-1 blueprint-overlay">
                <div class="bg-surface-container-lowest p-12">
                    <div class="inline-block bg-secondary-container/20 px-3 py-1 mb-6">
                        <span class="font-label text-[0.6875rem] text-secondary font-bold tracking-tighter">
                            "SECURE_ZONE // 0xAUTH"
                        </span>
                    </div>
                    <h2 class="text-4xl font-extrabold text-primary mb-8 tracking-tight">"SYSTEM_CMS"</h2>
                    <div class="space-y-12">
                        // dyn_child if/else: Leptos replaces the whole slot on toggle —
                        // both branches are hydration-safe regardless of which was SSR'd.
                        {move || if use_email.get() {
                            view! {
                                <div class="space-y-6">
                                    <div class="relative w-full group">
                                        <label class="jetbrains text-[0.65rem] uppercase tracking-[0.1em] text-outline text-left block mb-2">
                                            "Email Address"
                                        </label>
                                        <input
                                            type="email"
                                            placeholder="you@example.com"
                                            on:input=move |ev| set_username.set(event_target_value(&ev))
                                            prop:value=username
                                            class="w-full bg-transparent border-none border-b-2 border-outline-variant focus:border-primary focus:ring-0 px-0 py-4 jetbrains text-lg text-on-surface transition-all placeholder:text-outline-variant/50"
                                        />
                                    </div>
                                    <div class="space-y-4 pt-2">
                                        <Show when=move || !auth_error.get().is_empty()>
                                            <div class="bg-error/10 border-l-4 border-error p-4 text-error jetbrains text-sm font-medium">
                                                {move || auth_error.get()}
                                            </div>
                                        </Show>
                                        <button
                                            on:click=move |_| { login_action.dispatch(()); }
                                            disabled=move || is_loading.get() || (countdown.get() > 0)
                                            class="w-full bg-primary text-white py-6 jetbrains font-bold text-sm tracking-[0.2em] uppercase hover:bg-primary-container disabled:opacity-50 disabled:cursor-not-allowed transition-colors flex items-center justify-center gap-3"
                                        >
                                            <Show when=move || is_loading.get()>
                                                <span class="material-symbols-outlined animate-spin text-base">"progress_activity"</span>
                                            </Show>
                                            <span class="inline-block translate-y-[1px]">
                                                {move || if countdown.get() > 0 {
                                                    format!("Resend in {}s", countdown.get())
                                                } else if auth_error.get() == "Magic link sent! Check your email." {
                                                    "Resend Magic Link".to_string()
                                                } else {
                                                    "Send Magic Link".to_string()
                                                }}
                                            </span>
                                        </button>
                                        <div class="text-center pt-2">
                                            <button
                                                type="button"
                                                class="text-xs font-bold text-outline hover:text-primary transition-colors uppercase tracking-widest"
                                                on:click=move |_| { set_use_email.set(false); set_auth_error.set(String::new()); }
                                            >
                                                "\u{2190} Back to Passkey"
                                            </button>
                                        </div>
                                    </div>
                                </div>
                            }.into_any()
                        } else {
                            view! {
                                <div class="space-y-6">
                                    <button
                                        id="login-passkey-btn"
                                        class="w-full bg-primary text-white py-6 jetbrains font-bold text-sm tracking-[0.2em] uppercase hover:bg-primary-container transition-colors flex items-center justify-center gap-3"
                                    >
                                        <span class="material-symbols-outlined text-base">"passkey"</span>
                                        <span class="inline-block translate-y-[1px]">"Sign In with Passkey"</span>
                                    </button>
                                    <div id="passkey-message" class="text-sm text-center text-error font-medium h-4"></div>
                                    <script inner_html=r#"
                                    (function() {
                                        var existing = document.querySelector('script[data-simplewebauthn]');
                                        var scriptEl = existing || document.createElement('script');
                                        if (!existing) {
                                            scriptEl.src = 'https://unpkg.com/@simplewebauthn/browser/dist/bundle/index.umd.min.js';
                                            scriptEl.setAttribute('data-simplewebauthn', 'true');
                                            document.head.appendChild(scriptEl);
                                        }
                                        function bindBtn() {
                                            var btn = document.getElementById('login-passkey-btn');
                                            var msg = document.getElementById('passkey-message');
                                            if (!btn || btn.dataset.bound) return;
                                            btn.dataset.bound = 'true';
                                            btn.addEventListener('click', async function() {
                                                try {
                                                    btn.disabled = true; msg.innerText = 'Initiating...';
                                                    var startRes = await fetch('/api/passkeys/start-login', {
                                                        method: 'POST', headers: {'Content-Type':'application/json'},
                                                        body: JSON.stringify({ email: '' })
                                                    });
                                                    if (!startRes.ok) throw new Error('Failed to start login');
                                                    var options = await startRes.json();
                                                    msg.innerText = 'Please follow browser prompts...';
                                                    var credential = await window.SimpleWebAuthnBrowser.startAuthentication(options);
                                                    msg.innerText = 'Verifying...';
                                                    var finishRes = await fetch('/api/passkeys/finish-login', {
                                                        method: 'POST', headers: {'Content-Type':'application/json'},
                                                        body: JSON.stringify(credential)
                                                    });
                                                    if (finishRes.ok) {
                                                        msg.innerText = 'Success! Redirecting...';
                                                        msg.className = 'text-sm text-center text-green-500 font-medium h-4';
                                                        window.location.reload();
                                                    } else { throw new Error(await finishRes.text()); }
                                                } catch(err) {
                                                    console.error(err);
                                                    msg.innerText = err.message || 'Login failed';
                                                    msg.className = 'text-sm text-center text-error font-medium h-4';
                                                } finally { btn.disabled = false; }
                                            });
                                        }
                                        if (existing && window.SimpleWebAuthnBrowser) { bindBtn(); }
                                        else { scriptEl.onload = bindBtn; }
                                    })();
                                    "#></script>
                                    <div class="text-center pt-2">
                                        <button
                                            type="button"
                                            class="text-xs font-bold text-outline hover:text-primary transition-colors uppercase tracking-widest"
                                            on:click=move |_| set_use_email.set(true)
                                        >
                                            "Use Email Instead"
                                        </button>
                                    </div>
                                </div>
                            }.into_any()
                        }}
                    </div>
                </div>
            </div>
        </div>
    }
}

#[component]
pub fn Admin() -> impl IntoView {
    let query = use_query_map();

    // ── Auth resource ─────────────────────────────────────────────────────────
    // Uses create_resource (not create_effect + spawn_local) so it integrates
    // directly with the Leptos reactive scheduler and resolves correctly even
    // when the component mounts inside the outer App <Suspense>.
    //
    // Reactive key: the ?token= query param. If a magic-link token is present
    // it is verified first; otherwise we fall through to session validation.
    // The resource re-runs automatically if the URL query string changes.
    let auth_resource = Resource::new(
        move || query.with(|q| q.get("token")),
        |token| async move {
            if let Some(t) = token {
                if !t.is_empty() {
                    // Magic link: verify token, set HttpOnly cookie server-side,
                    // then treat as authenticated. Auto-login — no extra click.
                    if verify_magic_link(t).await.is_ok() {
                        return AuthStep::Authenticated { from_magic_link: true };
                    }
                }
            }
            match check_session().await {
                Ok(true) => AuthStep::Authenticated { from_magic_link: false },
                _        => AuthStep::Unauthenticated,
            }
        },
    );

    // Derived closures — read the resource; <Suspense> handles the None/pending case.
    let auth_state    = move || match auth_resource.get() {
        Some(AuthStep::Authenticated { .. }) => AuthState::Yes,
        Some(AuthStep::Unauthenticated)      => AuthState::No,
        None                                 => AuthState::Pending,
    };
    let show_passkey_nudge = move || matches!(
        auth_resource.get(),
        Some(AuthStep::Authenticated { from_magic_link: true })
    );

    let (active_tab, set_active_tab) = signal("DASHBOARD");

    let (modal_state, set_modal_state) = signal(ModalState::None);
    provide_context(modal_state);
    provide_context(set_modal_state);

    let (refresh, set_refresh) = signal(0i32);
    provide_context(refresh);
    provide_context(set_refresh);



    view! {
        <main class="min-h-screen bg-surface-container-low text-on-surface flex flex-col pt-24 px-4 md:px-[8.5rem]">
            // Suspense is INSIDE <main> (not wrapping it) so there is only ever
            // one <main> element in the DOM — the hydration walker never sees a
            // structural mismatch. Leptos serializes auth_resource during SSR
            // because it is read inside this Suspense boundary, so WASM picks up
            // the resolved value immediately without a refetch.
            <Suspense fallback=move || view! {
                <div class="flex-1 flex justify-center items-center">
                    <span class="material-symbols-outlined animate-spin text-4xl text-primary">"progress_activity"</span>
                </div>
            }>
            {move || match auth_state() {
                // Pending: Suspense fallback renders instead — this arm is kept
                // only for type completeness and is practically unreachable once
                // the SSR-serialized resource value is available on the WASM side.
                AuthState::Pending => view! {
                    <div class="flex-1 flex justify-center items-center">
                        <span class="material-symbols-outlined animate-spin text-4xl text-primary">"progress_activity"</span>
                    </div>
                }.into_any(),

                // ── Unauthenticated ─────────────────────────────────────────────
                // LoginPanel owns all auth-form signals in its own component scope,
                // so the reactive toggle between passkey and email views is fully
                // isolated from this outer auth-state match closure.
                AuthState::No => view! {
                    <LoginPanel />
                }.into_any(),

                // ── Authenticated ───────────────────────────────────────────────
                AuthState::Yes => view! {
                    <div class="flex-1 flex flex-col md:flex-row gap-12 pb-24">
                        // Sidebar
                        <aside class="w-full md:w-64 shrink-0 space-y-2">
                            <div class="mb-12">
                            {["DASHBOARD", "WEBFORMS", "SERVICES", "CASE STUDIES", "HIGHLIGHTS", "MAILING LIST", "SETTINGS", "LEAD OPTIONS", "NAVIGATION", "FOOTER", "PAGE HEADERS", "BLOG", "RESUME PROFILES", "RESUME ENTRIES", "LANDING PAGES", "SECURITY"].iter().map(|&t| {
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
                                    ).collect::<Vec<_>>()}
                                </div>

                            <button
                                on:click=move |_| {
                                    leptos::task::spawn_local(async move {
                                        // RevokeSession clears the HttpOnly cookie server-side.
                                        let _ = revoke_session().await;
                                        // Refetch auth_resource — it will re-run check_session,
                                        // find no valid cookie, and resolve to Unauthenticated.
                                        auth_resource.refetch();
                                    });
                                }
                                class="text-error text-xs jetbrains font-bold uppercase tracking-widest hover:underline"
                            >
                                "[ TERMINATE SESSION ]"
                            </button>
                        </aside>

                        // Main Content Area
                        <section class="flex-1 bg-surface-container-highest p-1 blueprint-overlay min-h-[600px]">
                            <div class="bg-surface-container-lowest h-full p-8 md:p-12 relative flex flex-col">
                                <Show when=show_passkey_nudge>
                                    <PasskeyRegistrationNudge />
                                </Show>
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
                                                "WEBFORMS" => ModalState::None, // Requires dedicated Form Builder UI
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
                                        "DASHBOARD" => view! { <DashboardView /> }.into_any(),
                                        "WEBFORMS" => view! { <WebformsTable /> }.into_any(),
                                        "SERVICES" => view! { <ServiceTable /> }.into_any(),
                                        "CASE STUDIES" => view! { <CaseStudyTable /> }.into_any(),
                                        "HIGHLIGHTS" => view! { <HighlightTable /> }.into_any(),
                                        "MAILING LIST" => view! { <MailingListTable /> }.into_any(),
                                        "LEAD OPTIONS" => view! { <LeadOptionTable /> }.into_any(),
                                        "NAVIGATION" => view! { <NavTable /> }.into_any(),
                                        "FOOTER" => view! { <FooterTable /> }.into_any(),
                                        "PAGE HEADERS" => view! { <PageHeaderTable /> }.into_any(),
                                        "SETTINGS" => view! { <SettingsReadView /> }.into_any(),
                                        "BLOG" => view! { <PostTable /> }.into_any(),
                                        "RESUME PROFILES" => view! { <ResumeProfileTable /> }.into_any(),
                                        "RESUME ENTRIES" => view! { <BaseResumeEntryTable /> }.into_any(),
                                        "LANDING PAGES" => view! { <LandingPageTable /> }.into_any(),
                                        "SECURITY" => view! { <PasskeyTable /> }.into_any(),
                                        _ => view! {
                                            <div class="h-64 flex items-center justify-center border-2 border-dashed border-outline-variant text-outline">
                                                <span class="jetbrains text-sm">"MODULE_OFFLINE"</span>
                                            </div>
                                        }.into_any(),
                                    }}
                                </div>
                            </div>
                            <AdminEditorModal />
                        </section>
                    </div>
                }.into_any(),
            }}
            </Suspense>
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
    // users.tenant_id was removed in the RBAC migration (m20260504_000002_remove_is_admin_from_user).
    // Count identities by joining user_account → account to resolve the tenant association.
    let signups: i64 = sqlx::query_scalar(
        "SELECT COUNT(DISTINCT ua.user_id) FROM user_account ua \
         JOIN account a ON a.id = ua.account_id \
         WHERE a.tenant_id IS NOT DISTINCT FROM $1"
    )
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
                    {match res {
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
                        }).collect::<Vec<_>>().into_any(),
                        _ => view! { <tr><td colspan="4" class="py-8 text-center text-error">"ERR_NO_DATA"</td></tr> }.into_any(),
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
                }.into_any(),
                _ => view! { <div class="text-error">"Failed to load settings"</div> }.into_any()
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
                        <div class="grid grid-cols-3 py-2 border-b border-secondary/20 hover:bg-surface-container/30 mt-4">
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
                _ => view! { <div class="text-error">"Failed to load settings"</div> }.into_any()
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
                    {match res {
                        Some(Ok(items)) => items.into_iter().map(|item| {
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
                        _ => view! { <tr><td colspan="3" class="py-8 text-center text-error">"ERR_NO_DATA"</td></tr> }.into_any(),
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
            {move || match items_res.get() {
                Some(Ok(items)) => {
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
                _ => view! { <div class="py-8 text-center text-error">"ERR_NO_DATA"</div> }.into_any(),
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
                    {match res {
                        Some(Ok(items)) => items.into_iter().map(|item| {
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
                        _ => view! { <tr><td colspan="6" class="py-8 text-center text-error">"ERR_NO_DATA"</td></tr> }.into_any(),
                    }}
                </tbody>
            </table>
            }.into_any()
            }}
        </Transition>
    }
}

#[component]
fn MailingListTable() -> impl IntoView {
    let refresh = expect_context::<ReadSignal<i32>>();
    let list_resource = Resource::new(move || refresh.get(), |_| get_mailing_list());

    view! {
        <Transition fallback=move || view! { <div class="jetbrains text-sm text-outline">"QUERYING_DB..."</div> }>
            {move || {
                let res = list_resource.get();
                view! {
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
                    {match res {
                        Some(Ok(items)) => items.into_iter().map(|i| {
                            let prefs_title = i.preferences.clone();
                            let prefs_text = i.preferences;
                            view! {
                            <tr class="hover:bg-surface-container-high transition-colors group">
                                <td class="py-4 px-4 font-bold text-primary">{i.email}</td>
                                <td class="py-4 px-4 text-on-surface">{i.list_type}</td>
                                <td class="py-4 px-4 text-outline truncate max-w-[200px]" title=prefs_title>{prefs_text}</td>
                                <td class="py-4 px-4 text-outline-variant">{i.created_at}</td>
                                <td class="py-4 px-4">
                                    <div class="flex space-x-4 opacity-0 group-hover:opacity-100 transition-opacity">
                                        <button
                                            on:click=move |_| {
                                                let id = i.id;
                                                let r = refresh;
                                                leptos::task::spawn_local(async move {
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
                            }
                        }).collect::<Vec<_>>().into_any(),
                        _ => view! { <tr><td colspan="5" class="py-8 text-center text-error">"ERR_NO_DATA"</td></tr> }.into_any(),
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
                    {match res {
                        Some(Ok(posts)) => posts.into_iter().map(|p| {
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
                        _ => view! { <tr><td colspan="4" class="py-8 text-center text-error">"ERR_NO_DATA"</td></tr> }.into_any(),
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
                    {match res {
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
                        _ => view! { <tr><td colspan="4" class="py-8 text-center text-error">"ERR_NO_DATA"</td></tr> }.into_any(),
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
                    {match res {
                        Some(Ok(pages)) => pages.into_iter().map(|p| {
                            let p_clone = p.clone();
                            let p_clone_2 = p.clone();
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
                        _ => view! { <tr><td colspan="3" class="py-8 text-center text-error">"ERR_NO_DATA"</td></tr> }.into_any(),
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
                    {match res {
                        Some(Ok(items)) => items.into_iter().map(|n| {
                            let n_clone = n.clone();
                            view! {
                            <tr class="hover:bg-surface-container-high transition-colors group">
                                <td class="py-4 px-4 text-outline-variant">{n.display_order}</td>
                                <td class="py-4 px-4 text-outline font-medium">"#" {n.id.to_string()}</td>
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
                        _ => view! { <tr><td colspan="4" class="py-8 text-center text-error">"ERR_NO_DATA"</td></tr> }.into_any(),
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
                        <th class="py-4 px-4 font-normal tracking-widest uppercase">"Label"</th>
                        <th class="py-4 px-4 font-normal tracking-widest uppercase">"Binding"</th>
                        <th class="py-4 px-4 font-normal tracking-widest uppercase">"Actions"</th>
                    </tr>
                </thead>
                <tbody class="divide-y divide-outline-variant/20">
                    {match res {
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
                        _ => view! { <tr><td colspan="4" class="py-8 text-center text-error">"ERR_NO_DATA"</td></tr> }.into_any(),
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
                    {match res {
                        Some(Ok(items)) => items.into_iter().map(|item| { let c = item.clone(); view! {
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
                        _ => view! { <tr><td colspan="4" class="py-8 text-center text-error">"ERR_NO_DATA"</td></tr> }.into_any(),
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
                    {match res {
                        Some(Ok(items)) => items.into_iter().map(|item| { let c = item.clone(); view! {
                            <tr class="hover:bg-surface-container-high transition-colors group">
                                <td class="py-4 px-4 text-outline-variant">{item.display_order}</td>
                                <td class="py-4 px-4 font-bold text-primary">{item.client_name}</td>
                                <td class="py-4 px-4 text-outline">{if item.is_visible { "YES" } else { "NO" }}</td>
                                <td class="py-4 px-4">
                                    <div class="flex space-x-4 opacity-0 group-hover:opacity-100 transition-opacity">
                                        <button on:click=move |_| set_modal_state.set(crate::components::admin_modal::ModalState::CaseStudy(Some(c.clone()))) class="text-secondary hover:underline uppercase text-xs">"Edit"</button>
                                        <button on:click=move |_| { let id = item.id; leptos::task::spawn_local(async move { if let Ok(_) = delete_case_study(id).await { set_refresh.set(refresh.get_untracked() + 1); } }); } class="text-error hover:underline uppercase text-xs">"Drop"</button>
                                    </div>
                                </td>
                            </tr>
                        }}).collect::<Vec<_>>().into_any(),
                        _ => view! { <tr><td colspan="4" class="py-8 text-center text-error">"ERR_NO_DATA"</td></tr> }.into_any(),
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
                    {match res {
                        Some(Ok(items)) => items.into_iter().map(|item| { let c = item.clone(); view! {
                            <tr class="hover:bg-surface-container-high transition-colors group">
                                <td class="py-4 px-4 text-outline-variant">{item.display_order}</td>
                                <td class="py-4 px-4 font-bold text-primary">{item.title}</td>
                                <td class="py-4 px-4 text-outline">{if item.is_visible { "YES" } else { "NO" }}</td>
                                <td class="py-4 px-4">
                                    <div class="flex space-x-4 opacity-0 group-hover:opacity-100 transition-opacity">
                                        <button on:click=move |_| set_modal_state.set(crate::components::admin_modal::ModalState::Highlight(Some(c.clone()))) class="text-secondary hover:underline uppercase text-xs">"Edit"</button>
                                        <button on:click=move |_| { let id = item.id; leptos::task::spawn_local(async move { if let Ok(_) = delete_highlight(id).await { set_refresh.set(refresh.get_untracked() + 1); } }); } class="text-error hover:underline uppercase text-xs">"Drop"</button>
                                    </div>
                                </td>
                            </tr>
                        }}).collect::<Vec<_>>().into_any(),
                        _ => view! { <tr><td colspan="4" class="py-8 text-center text-error">"ERR_NO_DATA"</td></tr> }.into_any(),
                    }}
                </tbody>
            </table>
            }.into_any()
            }}
        </Transition>
    }
}

#[component]
fn PasskeyRegistrationNudge() -> impl IntoView {
    let (is_hidden, set_is_hidden) = signal(false);
    
    view! {
        <Show when=move || !is_hidden.get()>
            <div class="bg-primary/10 border border-primary p-6 mb-8 flex justify-between items-center w-full">
                <div>
                    <h3 class="text-primary font-bold text-lg">"Action Required: Set Up a Passkey"</h3>
                    <p class="text-sm text-on-surface-variant mt-1">"You logged in using an email link. For future logins, please set up a passkey."</p>
                </div>
                <div class="flex gap-4 items-center">
                    <button 
                        id="nudge-register-passkey-btn"
                        class="bg-primary text-white px-4 py-2 font-bold hover:bg-primary-container transition-colors disabled:opacity-50"
                    >
                        "Set Up Passkey"
                    </button>
                    <button 
                        class="text-outline hover:text-on-surface"
                        on:click=move |_| set_is_hidden.set(true)
                    >
                        <span class="material-symbols-outlined">"close"</span>
                    </button>
                </div>
                <div id="nudge-passkey-message" class="text-sm font-medium mt-2"></div>
            </div>
            <script src="https://unpkg.com/@simplewebauthn/browser/dist/bundle/index.umd.min.js"></script>
            <script>
            "setTimeout(() => {
                const btn = document.getElementById('nudge-register-passkey-btn');
                const msg = document.getElementById('nudge-passkey-message');
                if(btn && !btn.dataset.bound) {
                    btn.dataset.bound = 'true';
                    btn.addEventListener('click', async () => {
                        try {
                            btn.disabled = true;
                            msg.innerText = 'Initiating...';
                            
                            const startRes = await fetch('/api/passkeys/start-register', {
                                method: 'POST',
                                headers: { 'Content-Type': 'application/json' }
                            });
                            if (!startRes.ok) throw new Error('Failed to start registration');
                            const options = await startRes.json();
                            
                            msg.innerText = 'Please follow browser prompts...';
                            const { startRegistration } = window.SimpleWebAuthnBrowser;
                            const credential = await startRegistration(options);
                            
                            msg.innerText = 'Verifying...';
                            const finishRes = await fetch('/api/passkeys/finish-register', {
                                method: 'POST',
                                headers: { 'Content-Type': 'application/json' },
                                body: JSON.stringify(credential)
                            });
                            
                            if (finishRes.ok) {
                                msg.innerText = 'Passkey registered successfully!';
                                msg.className = 'text-sm font-medium text-green-500 mt-2';
                                setTimeout(() => btn.closest('.bg-primary\\\\/10').style.display = 'none', 2000);
                            } else {
                                throw new Error(await finishRes.text());
                            }
                        } catch (err) {
                            console.error(err);
                            msg.innerText = err.message || 'Registration failed';
                            msg.className = 'text-sm font-medium text-error mt-2';
                        } finally {
                            btn.disabled = false;
                        }
                    });
                }
            }, 500);"
            </script>
        </Show>
    }
}
