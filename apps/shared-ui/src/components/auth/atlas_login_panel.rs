use leptos::prelude::*;
use leptos_router::hooks::use_query_map;

use crate::auth::atlas_auth::use_atlas_auth;
use crate::components::auth::passkey_login::PasskeyLoginButton;

/// Platform-standard login panel — handles both passkey and magic link flows.
///
/// # Usage
/// ```rust
/// <AtlasLoginPanel
///     app_title="SYSTEM_CMS"
///     on_authenticated=Callback::new(|_| { /* navigate or reload */ })
/// />
/// ```
///
/// Mode switching is driven by `?mode=email` in the URL so the email form
/// is SSR-rendered and immediately visible without waiting for WASM to hydrate.
///
/// # Props
/// - `app_title`       — displayed above the mode tabs (e.g. "SYSTEM_CMS", "ATLAS ADMIN")
/// - `on_authenticated` — called after a successful passkey login. Magic link logins
///                        redirect via the `?token=` query param handled by the parent page.
#[component]
pub fn AtlasLoginPanel(
    /// Short all-caps label shown as the panel heading.
    #[prop(into, default = "ATLAS".into())]
    app_title: String,
    /// Called when passkey authentication completes successfully. Typically reloads
    /// the page or navigates to the authenticated view.
    #[prop(into, optional)]
    on_authenticated: Option<Callback<()>>,
) -> impl IntoView {
    let auth = use_atlas_auth();
    let query = use_query_map();

    // URL-param–driven mode: ?mode=email → magic link form is SSR-rendered immediately.
    // No WASM hydration required to see the email input — the form is in the HTML.
    let email_mode = move || query.with(|q| q.get("mode").as_deref() == Some("email"));

    // Passkey callbacks — delegate to PasskeyLoginButton
    let on_auth = on_authenticated.clone();
    let handle_passkey_success = Callback::new(move |_token: String| {
        if let Some(cb) = &on_auth {
            cb.run(());
        } else {
            // Default: reload so the parent page's auth_resource re-checks the session
            let _ = web_sys::window().unwrap().location().reload();
        }
    });

    let handle_passkey_error = Callback::new(move |err: String| {
        auth.error.set(Some(err));
    });

    // Successful magic link send — read the dedicated typed signal, not auth.error
    let magic_link_sent = move || auth.magic_link_sent.get();

    view! {
        <div class="flex-1 flex justify-center items-center py-12">
            <div class="w-full max-w-lg bg-surface-container-highest p-1 blueprint-overlay">
                <div class="bg-surface-container-lowest p-12">

                    // ── Header ───────────────────────────────────────────────────────
                    <div class="inline-block bg-secondary-container/20 px-3 py-1 mb-6">
                        <span class="font-label text-[0.6875rem] text-secondary font-bold tracking-tighter">
                            "SECURE_ZONE // 0xAUTH"
                        </span>
                    </div>
                    <h2 class="text-4xl font-extrabold text-primary mb-2 tracking-tight">
                        {app_title.clone()}
                    </h2>

                    // ── Mode tabs ─────────────────────────────────────────────────────
                    // Plain <a> links — work before WASM loads, update the URL, and
                    // cause the SSR to re-render the correct form on navigation.
                    <div class="flex items-center gap-3 mb-10">
                        <a
                            href="?"
                            class=move || format!(
                                "text-[0.65rem] jetbrains font-bold uppercase tracking-widest px-3 py-1 border transition-colors {}",
                                if !email_mode() {
                                    "border-primary text-primary bg-primary/10"
                                } else {
                                    "border-outline-variant text-outline hover:border-primary hover:text-primary"
                                }
                            )
                        >
                            "Passkey"
                        </a>
                        <a
                            href="?mode=email"
                            class=move || format!(
                                "text-[0.65rem] jetbrains font-bold uppercase tracking-widest px-3 py-1 border transition-colors {}",
                                if email_mode() {
                                    "border-primary text-primary bg-primary/10"
                                } else {
                                    "border-outline-variant text-outline hover:border-primary hover:text-primary"
                                }
                            )
                        >
                            "Magic Link"
                        </a>
                    </div>

                    // ── Email / Magic Link flow ───────────────────────────────────────
                    {move || if email_mode() {
                        view! {
                            <div class="space-y-6">
                                {move || if magic_link_sent() {
                                    // ── Post-send confirmation ────────────────────────
                                    view! {
                                        <div class="border border-primary/30 bg-primary/5 p-8 text-center space-y-4">
                                            <span class="material-symbols-outlined text-4xl text-primary block">
                                                "mark_email_read"
                                            </span>
                                            <p class="text-on-surface font-bold jetbrains text-sm tracking-wide">
                                                "CHECK YOUR INBOX"
                                            </p>
                                            <p class="text-on-surface-variant text-sm leading-relaxed">
                                                "A login link has been sent to "
                                                <span class="text-primary font-bold">
                                                    {move || auth.email.get()}
                                                </span>
                                                ". Click the link in the email to sign in."
                                            </p>
                                            <p class="text-outline text-xs jetbrains">
                                                "Link expires in 15 minutes. Check your spam folder if it doesn't arrive."
                                            </p>
                                            <button
                                                type="button"
                                                on:click=move |_| {
                                                    auth.magic_link_sent.set(false);
                                                    auth.error.set(None);
                                                }
                                                class="text-xs font-bold text-outline hover:text-primary transition-colors uppercase tracking-widest mt-4 inline-block"
                                            >
                                                "← Try a different email"
                                            </button>
                                        </div>
                                    }.into_any()
                                } else {
                                    // ── Email input ───────────────────────────────────
                                    view! {
                                        <div class="space-y-6">
                                            <div class="space-y-2">
                                                <label class="jetbrains text-[0.65rem] uppercase tracking-[0.1em] text-outline text-left block">
                                                    "Email Address"
                                                </label>
                                                <input
                                                    id="atlas-magic-link-email"
                                                    type="email"
                                                    placeholder="admin@yourdomain.com"
                                                    on:input=move |ev| auth.email.set(event_target_value(&ev))
                                                    prop:value=auth.email
                                                    class="w-full bg-transparent border-b-2 border-outline-variant focus:border-primary focus:outline-none px-0 py-4 jetbrains text-lg text-on-surface transition-all placeholder:text-outline-variant/40"
                                                />
                                                <p class="text-outline text-xs jetbrains">
                                                    "We'll send a one-time sign-in link to this address."
                                                </p>
                                            </div>

                                            // Error feedback — only shown on actual errors, never on success
                                            {move || auth.error.get().map(|e| {
                                                view! {
                                                    <div class="border-l-4 border-error bg-error/10 p-4 text-sm jetbrains font-medium text-error">
                                                        {e}
                                                    </div>
                                                }
                                            })}

                                            <button
                                                on:click=move |_| { auth.dispatch_login.dispatch(()); }
                                                disabled=move || {
                                                    auth.is_loading.get()
                                                    || auth.countdown.get() > 0
                                                    || auth.email.get().is_empty()
                                                }
                                                class="w-full bg-primary text-white py-5 jetbrains font-bold text-sm tracking-[0.2em] uppercase hover:bg-primary-container disabled:opacity-40 disabled:cursor-not-allowed transition-colors flex items-center justify-center gap-3"
                                            >
                                                {move || if auth.is_loading.get() {
                                                    view! {
                                                        <span class="flex items-center gap-3">
                                                            <span class="material-symbols-outlined animate-spin text-base">
                                                                "progress_activity"
                                                            </span>
                                                            "Sending..."
                                                        </span>
                                                    }.into_any()
                                                } else if auth.countdown.get() > 0 {
                                                    view! {
                                                        <span>
                                                            {format!("Resend in {}s", auth.countdown.get())}
                                                        </span>
                                                    }.into_any()
                                                } else {
                                                    view! {
                                                        <span class="flex items-center gap-2">
                                                            <span class="material-symbols-outlined text-base">"send"</span>
                                                            "Send Magic Link"
                                                        </span>
                                                    }.into_any()
                                                }}
                                            </button>
                                        </div>
                                    }.into_any()
                                }}
                            </div>
                        }.into_any()

                    // ── Passkey flow ──────────────────────────────────────────────────
                    } else {
                        view! {
                            <div class="space-y-6">
                                <p class="text-on-surface-variant text-sm leading-relaxed">
                                    "Use a registered passkey (Face ID, Touch ID, or hardware key) to authenticate instantly — no password required."
                                </p>

                                // Error feedback (routed here by handle_passkey_error callback)
                                {move || auth.error.get().map(|e| view! {
                                    <div class="border-l-4 border-error bg-error/10 p-4 text-error text-sm jetbrains font-medium">
                                        {e}
                                    </div>
                                })}

                                <PasskeyLoginButton
                                    api_base_url="/api/passkeys".to_string()
                                    email=RwSignal::new("".to_string())
                                    on_success=handle_passkey_success
                                    on_error=handle_passkey_error
                                />

                                <div class="border-t border-outline-variant/30 pt-4 text-center space-y-1">
                                    <p class="text-outline text-xs jetbrains">
                                        "Don't have a passkey registered yet?"
                                    </p>
                                    <p class="text-outline text-xs jetbrains">
                                        "Sign in with a magic link first, then register one from Security settings."
                                    </p>
                                </div>
                            </div>
                        }.into_any()
                    }}
                </div>
            </div>
        </div>
    }
}
