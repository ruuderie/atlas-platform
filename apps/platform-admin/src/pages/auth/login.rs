use crate::api::models::UserInfo;
use crate::api::setup::{get_setup_status, purge_admin};
use leptos::prelude::*;
use leptos_router::hooks::use_navigate;
use shared_ui::auth::atlas_auth::use_atlas_auth;
use shared_ui::components::auth::passkey_login::PasskeyLoginButton;
use shared_ui::components::ui::input::{Input, InputType};

#[component]
pub fn Login() -> impl IntoView {
    let auth = use_atlas_auth();
    let set_user = use_context::<WriteSignal<Option<UserInfo>>>().expect("set_user context");
    let toast = use_context::<crate::app::GlobalToast>().expect("toast context");
    let navigate = use_navigate();
    let is_purging = RwSignal::new(false);

    // Setup status check
    let navigate_setup = navigate.clone();
    leptos::task::spawn_local(async move {
        if let Ok(status) = get_setup_status().await {
            if status.needs_setup {
                navigate_setup("/setup", Default::default());
            }
        }
    });

    let set_user_pk = set_user.clone();
    let navigate_pk = navigate.clone();
    let toast_pk = toast.clone();
    let handle_passkey_success = Callback::new(move |token: String| {
        let set_user = set_user_pk.clone();
        let navigate = navigate_pk.clone();
        let toast = toast_pk.clone();
        leptos::task::spawn_local(async move {
            crate::api::client::set_auth_token(&token);
            if let Ok(_) = shared_ui::auth::atlas_auth::set_session_cookie(token).await {
                if let Ok(user) = crate::api::auth::get_session().await {
                    set_user.set(Some(user));
                    navigate("/", Default::default());
                    return;
                }
            }
            toast.show_toast(
                "Auth",
                "Validated passkey, but session handshake failed.",
                "error",
            );
        });
    });

    let handle_passkey_error = Callback::new(move |err: String| {
        auth.error.set(Some(err));
    });

    let navigate_purge = navigate.clone();
    let toast_purge = toast.clone();
    let handle_purge_admin = Callback::new(move |_| {
        is_purging.set(true);
        let navigate = navigate_purge.clone();
        let toast = toast_purge.clone();
        leptos::task::spawn_local(async move {
            match purge_admin().await {
                Ok(_) => {
                    navigate("/setup", Default::default());
                }
                Err(e) => {
                    toast.show_toast("Auth Error", &e, "error");
                    is_purging.set(false);
                }
            }
        });
    });

    let is_sent = Signal::derive(move || {
        auth.error.get().as_deref() == Some("Magic link sent! Check your email.")
    });

    let handle_start_over = move |ev: leptos::ev::MouseEvent| {
        ev.prevent_default();
        auth.error.set(None);
        auth.email.set("".to_string());
    };

    let handle_resend = move |ev: leptos::ev::MouseEvent| {
        ev.prevent_default();
        if !auth.is_loading.get_untracked() && auth.countdown.get_untracked() == 0 {
            auth.dispatch_login.dispatch(());
        }
    };

    view! {
        <style>
            "
            .login-shell {
              --bg: #080A13;
              --surface: #0E1120;
              --elevated: #161B2E;
              --elevated2: #1C2236;
              --border: rgba(255,255,255,0.07);
              --border-strong: rgba(255,255,255,0.13);
              --text: #E2E5F0;
              --muted: #7A829A;
              --dim: #44495D;
              --cobalt: #0A84FF;
              --cobalt-dim: rgba(10,132,255,0.10);
              --cobalt-border: rgba(10,132,255,0.35);
              --green: #06966A;
              --green-dim: rgba(6,150,105,0.12);
              --red: #E5484D;
              --red-dim: rgba(229,72,77,0.12);
              --amber: #F5A623;

              display: grid;
              grid-template-columns: 1fr 480px;
              min-height: 100vh;
              background: var(--bg);
              color: var(--text);
              font-family: 'Inter', sans-serif;
              font-size: 13px;
              line-height: 1.5;
            }
            @media (max-width: 1024px) {
              .login-shell {
                grid-template-columns: 1fr;
              }
            }
            .login-brand {
              background: var(--surface);
              border-right: 1px solid var(--border);
              display: flex;
              flex-direction: column;
              padding: 40px 48px;
              position: relative;
              overflow: hidden;
            }
            @media (max-width: 1024px) {
              .login-brand {
                display: none;
              }
            }
            .login-brand::before {
              content: '';
              position: absolute;
              top: -160px;
              left: -160px;
              width: 560px;
              height: 560px;
              background: radial-gradient(circle, rgba(10,132,255,0.07) 0%, transparent 68%);
              pointer-events: none;
            }
            .login-brand::after {
              content: '';
              position: absolute;
              bottom: -100px;
              right: -80px;
              width: 380px;
              height: 380px;
              background: radial-gradient(circle, rgba(10,132,255,0.04) 0%, transparent 70%);
              pointer-events: none;
            }
            .login-brand-logo {
              display: flex;
              align-items: center;
              gap: 12px;
              margin-bottom: auto;
              position: relative;
              z-index: 1;
            }
            .login-brand-mark {
              width: 34px;
              height: 34px;
              background: var(--cobalt);
              border-radius: 7px;
              display: flex;
              align-items: center;
              justify-content: center;
              font-weight: 900;
              font-size: 15px;
              color: #fff;
              flex-shrink: 0;
              box-shadow: 0 0 0 1px rgba(10,132,255,0.4), 0 4px 16px rgba(10,132,255,0.25);
            }
            .login-brand-wordmark {
              font-size: 17px;
              font-weight: 700;
              letter-spacing: -0.3px;
              color: #fff;
            }
            .login-brand-badge {
              font-size: 9px;
              font-weight: 700;
              text-transform: uppercase;
              letter-spacing: 0.1em;
              color: var(--muted);
              border: 1px solid var(--border);
              border-radius: 3px;
              padding: 2px 6px;
              margin-left: auto;
            }
            .login-brand-hero {
              margin-top: auto;
              margin-bottom: 44px;
              position: relative;
              z-index: 1;
            }
            .login-brand-h {
              font-size: 42px;
              font-weight: 900;
              letter-spacing: -1px;
              line-height: 1.08;
              margin-bottom: 16px;
              color: #fff;
            }
            .login-brand-h em {
              font-style: normal;
              color: var(--cobalt);
            }
            .login-brand-sub {
              font-size: 14px;
              color: var(--muted);
              line-height: 1.7;
              max-width: 420px;
            }
            .login-brand-stats {
              display: flex;
              gap: 0;
              margin-top: 36px;
              padding-top: 28px;
              border-top: 1px solid var(--border);
            }
            .login-brand-stat {
              flex: 1;
              padding: 0 20px 0 0;
            }
            .login-brand-stat + .login-brand-stat {
              border-left: 1px solid var(--border);
              padding-left: 20px;
            }
            .login-brand-stat-val {
              font-size: 24px;
              font-weight: 800;
              letter-spacing: -0.5px;
              color: var(--cobalt);
            }
            .login-brand-stat-label {
              font-size: 10.5px;
              color: var(--dim);
              margin-top: 2px;
            }
            .login-brand-footer {
              font-size: 11px;
              color: var(--dim);
              position: relative;
              z-index: 1;
            }
            .login-brand-footer a {
              color: var(--dim);
              text-decoration: none;
            }
            .login-brand-footer a:hover {
              color: var(--muted);
            }
            .login-form-panel {
              display: flex;
              align-items: center;
              justify-content: center;
              padding: 40px;
              background: var(--bg);
            }
            .login-form-box {
              width: 100%;
              max-width: 360px;
            }
            .login-step-hdr {
              margin-bottom: 32px;
            }
            .login-step-title {
              font-size: 24px;
              font-weight: 800;
              letter-spacing: -0.5px;
              margin-bottom: 6px;
              color: #fff;
            }
            .login-step-sub {
              font-size: 12.5px;
              color: var(--muted);
              line-height: 1.6;
            }
            .login-alert {
              display: flex;
              padding: 9px 12px;
              border-radius: 6px;
              font-size: 12px;
              margin-bottom: 16px;
              align-items: flex-start;
              gap: 8px;
            }
            .login-alert-error {
              background: var(--red-dim);
              border: 1px solid rgba(229,72,77,0.3);
              color: var(--red);
            }
            .login-alert-success {
              background: var(--green-dim);
              border: 1px solid rgba(6,150,105,0.3);
              color: var(--green);
            }
            .login-alert-icon {
              flex-shrink: 0;
              font-size: 14px;
              margin-top: 1px;
            }
            .login-f-group {
              margin-bottom: 16px;
            }
            .login-f-label {
              display: block;
              font-size: 11px;
              font-weight: 600;
              color: var(--muted);
              margin-bottom: 5px;
              letter-spacing: 0.02em;
            }
            .login-f-input-wrapper input {
              width: 100% !important;
              background: var(--elevated) !important;
              border: 1px solid var(--border-strong) !important;
              border-radius: 7px !important;
              padding: 10px 13px !important;
              font-size: 13px !important;
              color: var(--text) !important;
              font-family: inherit !important;
              outline: none !important;
              height: auto !important;
              transition: border-color 0.15s, box-shadow 0.15s !important;
            }
            .login-f-input-wrapper input:focus {
              border-color: var(--cobalt) !important;
              box-shadow: 0 0 0 3px var(--cobalt-dim) !important;
            }
            .login-btn-primary {
              width: 100%;
              padding: 11px;
              background: var(--cobalt);
              border: none;
              border-radius: 7px;
              color: #fff;
              font-size: 13.5px;
              font-weight: 600;
              font-family: inherit;
              cursor: pointer;
              transition: background 0.15s, transform 0.1s;
              display: flex;
              align-items: center;
              justify-content: center;
              gap: 8px;
            }
            .login-btn-primary:hover {
              background: #0070E0;
            }
            .login-btn-primary:active {
              transform: scale(0.98);
            }
            .login-btn-primary:disabled {
              opacity: 0.5;
              cursor: not-allowed;
            }
            .login-btn-secondary {
              width: 100%;
              padding: 10px;
              background: transparent;
              border: 1px solid var(--border-strong);
              border-radius: 7px;
              color: var(--muted);
              font-size: 12.5px;
              font-weight: 500;
              font-family: inherit;
              cursor: pointer;
              transition: all 0.15s;
              display: flex;
              align-items: center;
              justify-content: center;
              gap: 8px;
            }
            .login-btn-secondary:hover {
              background: var(--elevated);
              color: var(--text);
              border-color: rgba(255,255,255,0.18);
            }
            .login-divider {
              display: flex;
              align-items: center;
              gap: 12px;
              margin: 18px 0;
            }
            .login-divider span {
              font-size: 11px;
              color: var(--dim);
            }
            .login-divider::before, .login-divider::after {
              content: '';
              flex: 1;
              height: 1px;
              background: var(--border);
            }
            .login-security-note {
              display: flex;
              align-items: center;
              gap: 7px;
              margin-top: 20px;
              padding: 8px 12px;
              background: var(--elevated);
              border: 1px solid var(--border);
              border-radius: 6px;
              font-size: 11px;
              color: var(--dim);
            }
            .login-security-note svg {
              flex-shrink: 0;
              opacity: 0.5;
            }
            .login-form-footer {
              font-size: 11.5px;
              color: var(--dim);
              text-align: center;
              margin-top: 24px;
              line-height: 1.8;
            }
            .login-form-footer a {
              color: var(--cobalt);
              text-decoration: none;
            }
            .login-form-footer a:hover {
              text-decoration: underline;
            }
            .login-ml-sent {
              text-align: center;
              padding: 8px 0;
            }
            .login-ml-sent-icon {
              font-size: 40px;
              margin-bottom: 16px;
              display: block;
            }
            .login-ml-sent-title {
              font-size: 18px;
              font-weight: 700;
              letter-spacing: -0.3px;
              margin-bottom: 8px;
              color: #fff;
            }
            .login-ml-sent-sub {
              font-size: 12.5px;
              color: var(--muted);
              line-height: 1.7;
              margin-bottom: 24px;
            }
            .login-ml-sent-email {
              display: inline-block;
              padding: 5px 12px;
              background: var(--elevated);
              border: 1px solid var(--border-strong);
              border-radius: 5px;
              font-size: 12.5px;
              color: var(--cobalt);
              margin-bottom: 24px;
            }
            .login-btn-danger {
              width: 100%;
              padding: 10px;
              background: rgba(229, 72, 77, 0.1);
              border: 1px solid rgba(229, 72, 77, 0.3);
              border-radius: 7px;
              color: var(--red);
              font-size: 12px;
              font-weight: 600;
              cursor: pointer;
              transition: all 0.15s;
            }
            .login-btn-danger:hover:not(:disabled) {
              background: rgba(229, 72, 77, 0.2);
            }
            .login-btn-danger:disabled {
              opacity: 0.5;
              cursor: not-allowed;
            }
            "
        </style>

        <div class="login-shell">
            // Left panel: Brand
            <div class="login-brand">
                <div class="login-brand-logo">
                    <div class="login-brand-mark">"A"</div>
                    <span class="login-brand-wordmark">"Atlas Platform"</span>
                    <span class="login-brand-badge">"Admin"</span>
                </div>

                <div class="login-brand-hero">
                    <div class="login-brand-h">"Command your "<br/><em>"entire operation."</em></div>
                    <div class="login-brand-sub">
                        "Atlas Platform Admin gives you full visibility over tenants, billing, CRM, scorecards, and payment rails — from a single authenticated workspace."
                    </div>
                    <div class="login-brand-stats">
                        <div class="login-brand-stat">
                            <div class="login-brand-stat-val">"$2.1B+"</div>
                            <div class="login-brand-stat-label">"GMV processed"</div>
                        </div>
                        <div class="login-brand-stat">
                            <div class="login-brand-stat-val">"48+"</div>
                            <div class="login-brand-stat-label">"Active tenants"</div>
                        </div>
                        <div class="login-brand-stat">
                            <div class="login-brand-stat-val">"₿ Native"</div>
                            <div class="login-brand-stat-label">"Payment rails"</div>
                        </div>
                    </div>
                </div>

                <div class="login-brand-footer">
                    "© 2026 Atlas Platform"
                </div>
            </div>

            // Right panel: Form
            <div class="login-form-panel">
                <div class="login-form-box">

                    <Show
                        when=move || is_sent.get()
                        fallback=move || view! {
                            <div>
                                <div class="login-step-hdr">
                                    <h2 class="login-step-title">"Admin Sign In"</h2>
                                    <p class="login-step-sub">"Super-admin access only. All sessions are logged and audited."</p>
                                </div>

                                {move || auth.error.get().map(|msg| view! {
                                    <div class="login-alert login-alert-error">
                                        <span class="login-alert-icon">"✕"</span>
                                        <span>{msg}</span>
                                    </div>
                                })}

                                {move || if auth.use_email.get() {
                                    view! {
                                        <div class="space-y-4">
                                            <div class="login-f-group">
                                                <label class="login-f-label">"Work Email"</label>
                                                <div class="login-f-input-wrapper">
                                                    <Input
                                                        r#type=InputType::Email
                                                        placeholder="admin@atlasplatform.io".to_string()
                                                        bind_value=auth.email
                                                    />
                                                </div>
                                            </div>

                                            <button
                                                class="login-btn-primary"
                                                on:click=move |ev| {
                                                    ev.prevent_default();
                                                    if auth.is_loading.get_untracked()
                                                        || auth.countdown.get_untracked() != 0
                                                        || auth.email.get_untracked().trim().is_empty()
                                                    {
                                                        return;
                                                    }
                                                    auth.dispatch_login.dispatch(());
                                                }
                                                disabled=move || auth.email.get().is_empty() || auth.is_loading.get() || (auth.countdown.get() > 0)
                                            >
                                                <svg width="14" height="14" viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" class="mr-1 inline">
                                                    <path d="M2 4l6 5 6-5M2 4h12v9a1 1 0 01-1 1H3a1 1 0 01-1-1V4z"/>
                                                </svg>
                                                {move || if auth.is_loading.get() {
                                                    "Evaluating Node...".to_string()
                                                } else if auth.countdown.get() > 0 {
                                                    format!("Resend in {}s", auth.countdown.get())
                                                } else {
                                                    "Send Magic Link".to_string()
                                                }}
                                            </button>

                                            <div class="login-divider"><span>"or sign in with"</span></div>

                                            <button
                                                class="login-btn-secondary"
                                                on:click=move |ev| { ev.prevent_default(); auth.use_email.set(false); auth.error.set(None); }
                                            >
                                                "← Back to Passkey"
                                            </button>
                                        </div>
                                    }.into_any()
                                } else {
                                    view! {
                                        <div class="space-y-4">
                                            <div class="py-2">
                                                <PasskeyLoginButton
                                                    api_base_url=crate::api::client::api_url("/api/passkeys")
                                                    email=RwSignal::new("".to_string())
                                                    on_success=handle_passkey_success.clone()
                                                    on_error=handle_passkey_error.clone()
                                                />
                                            </div>

                                            <div class="login-divider"><span>"or sign in with"</span></div>

                                            <button
                                                class="login-btn-secondary"
                                                on:click=move |ev| { ev.prevent_default(); auth.use_email.set(true); }
                                            >
                                                "Use Email Instead"
                                            </button>
                                        </div>
                                    }.into_any()
                                }}

                                <div class="login-security-note">
                                    <svg width="12" height="12" viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.5">
                                        <path d="M8 1L2 4v4c0 3.5 2.5 6.5 6 7.5C14 14.5 14 11.5 14 8V4L8 1z"/>
                                    </svg>
                                    <span>"Passwordless only. No passwords are stored. Sessions expire in 24h."</span>
                                </div>

                                <div class=if cfg!(debug_assertions) { "mt-6" } else { "hidden" }>
                                    <button
                                        class="login-btn-danger"
                                        on:click=move |ev| handle_purge_admin.run(ev)
                                        disabled=move || is_purging.get()
                                    >
                                        {move || if is_purging.get() { "Purging..." } else { "Purge Admin (Dev)" }}
                                    </button>
                                </div>
                            </div>
                        }
                    >
                        <div class="login-ml-sent">
                            <span class="login-ml-sent-icon">"✉️"</span>
                            <h2 class="login-ml-sent-title">"Check your inbox"</h2>
                            <p class="login-ml-sent-sub">"We sent a secure sign-in link to"</p>
                            <span class="login-ml-sent-email">{move || auth.email.get()}</span>
                            <p class="login-ml-sent-sub">
                                "The link expires in "<strong style="color:var(--text)">"15 minutes"</strong>" and can only be used once. After clicking it you'll be signed in automatically."
                            </p>

                            <div class="login-form-footer">
                                "Wrong address? "<a href="#" on:click=handle_start_over>"← Start over"</a>
                                " · "
                                <a href="#" on:click=handle_resend>"Resend link"</a>
                            </div>

                            <div class="login-security-note">
                                <svg width="12" height="12" viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.5">
                                    <circle cx="8" cy="8" r="6"/><path d="M8 5v4M8 11v.5"/>
                                </svg>
                                <span>"Link is single-use and tied to this browser session."</span>
                            </div>
                        </div>
                    </Show>

                </div>
            </div>
        </div>
    }
}
