use leptos::prelude::*;
use leptos_router::hooks::use_query_map;

use crate::auth::atlas_auth::use_atlas_auth;
use crate::components::auth::passkey_login::PasskeyLoginButton;

/// Platform-standard login panel — Kami design system.
///
/// Uses the Kami neutral vocabulary (parchment base, ivory card, ink-blue #1B365D accent)
/// with typography inherited from the host application so it is visually consistent
/// regardless of which Atlas app embeds it.
///
/// # Usage
/// ```rust
/// view! { <AtlasLoginPanel app_title="Atlas Admin" /> }
/// ```
///
/// Mode switching is driven by `?mode=email` in the URL — SSR-rendered, no WASM required.
#[component]
pub fn AtlasLoginPanel(
    /// Title shown in the panel heading. Any casing is fine.
    #[prop(into, default = "Atlas".into())]
    app_title: String,
    /// Called after a successful passkey login. Defaults to `window.location.reload()`.
    #[prop(into, optional)]
    on_authenticated: Option<Callback<()>>,
) -> impl IntoView {
    let auth = use_atlas_auth();
    let query = use_query_map();

    // URL-param mode: ?mode=email → email form is SSR-rendered immediately.
    let email_mode = move || query.with(|q| q.get("mode").as_deref() == Some("email"));

    // Passkey callbacks
    let on_auth = on_authenticated.clone();
    let handle_passkey_success = Callback::new(move |_token: String| {
        if let Some(cb) = &on_auth {
            cb.run(());
        } else {
            let _ = web_sys::window().unwrap().location().reload();
        }
    });

    let handle_passkey_error = Callback::new(move |err: String| {
        auth.error.set(Some(err));
    });

    let magic_link_sent = move || auth.magic_link_sent.get();

    view! {
        // Outer: inherit font-family from host app so the panel feels native
        <div style="
            min-height: 100vh;
            display: flex;
            align-items: center;
            justify-content: center;
            background: #f5f4ed;
            padding: 48px 16px;
        ">
            // Card
            <div style="
                width: 100%;
                max-width: 420px;
                background: #faf9f5;
                border: 1px solid #e8e6dc;
                border-radius: 8px;
                padding: 48px 40px;
                box-shadow: 0 4px 24px rgba(0,0,0,0.05);
            ">

                // ── Header ────────────────────────────────────────────────────
                <div style="
                    border-left: 3px solid #1B365D;
                    border-radius: 2px;
                    padding-left: 10px;
                    margin-bottom: 32px;
                ">
                    <p style="
                        font-size: 11px;
                        font-weight: 600;
                        letter-spacing: 0.08em;
                        text-transform: uppercase;
                        color: #6b6a64;
                        margin: 0 0 4px 0;
                    ">
                        "Secure Login"
                    </p>
                    <h1 style="
                        font-size: 26px;
                        font-weight: 500;
                        line-height: 1.2;
                        color: #141413;
                        margin: 0;
                    ">
                        {app_title.clone()}
                    </h1>
                </div>

                // ── Mode tabs ─────────────────────────────────────────────────
                <div style="display: flex; gap: 8px; margin-bottom: 32px;">
                    <a
                        href="?"
                        style=move || format!(
                            "display: inline-block; padding: 6px 14px; border-radius: 4px; \
                             font-size: 12px; font-weight: 500; text-decoration: none; \
                             transition: background 0.15s, color 0.15s; \
                             {}",
                            if !email_mode() {
                                "background: #1B365D; color: #faf9f5;"
                            } else {
                                "background: #e8e6dc; color: #3d3d3a;"
                            }
                        )
                    >
                        "Passkey"
                    </a>
                    <a
                        href="?mode=email"
                        style=move || format!(
                            "display: inline-block; padding: 6px 14px; border-radius: 4px; \
                             font-size: 12px; font-weight: 500; text-decoration: none; \
                             transition: background 0.15s, color 0.15s; \
                             {}",
                            if email_mode() {
                                "background: #1B365D; color: #faf9f5;"
                            } else {
                                "background: #e8e6dc; color: #3d3d3a;"
                            }
                        )
                    >
                        "Magic Link"
                    </a>
                </div>

                // ── Email / Magic Link flow ────────────────────────────────────
                {move || if email_mode() {
                    view! {
                        <div>
                            {move || if magic_link_sent() {
                                // ── Post-send confirmation ────────────────────
                                view! {
                                    <div style="
                                        border-left: 3px solid #1B365D;
                                        border-radius: 2px;
                                        background: #EEF2F7;
                                        padding: 20px 20px 20px 18px;
                                    ">
                                        <p style="
                                            font-size: 13px;
                                            font-weight: 600;
                                            color: #1B365D;
                                            margin: 0 0 8px 0;
                                        ">
                                            "Check your inbox"
                                        </p>
                                        <p style="
                                            font-size: 13px;
                                            color: #504e49;
                                            line-height: 1.55;
                                            margin: 0 0 4px 0;
                                        ">
                                            "A sign-in link was sent to "
                                            <span style="color: #1B365D; font-weight: 500;">
                                                {move || auth.email.get()}
                                            </span>
                                            "."
                                        </p>
                                        <p style="
                                            font-size: 12px;
                                            color: #6b6a64;
                                            margin: 0 0 16px 0;
                                        ">
                                            "Expires in 15 minutes. Check spam if it doesn't arrive."
                                        </p>
                                        <button
                                            type="button"
                                            on:click=move |_| {
                                                auth.magic_link_sent.set(false);
                                                auth.error.set(None);
                                            }
                                            style="
                                                background: none; border: none; cursor: pointer; padding: 0;
                                                font-size: 12px; color: #6b6a64;
                                                text-decoration: underline;
                                                text-underline-offset: 2px;
                                                transition: color 0.15s;
                                            "
                                        >
                                            "← Try a different email"
                                        </button>
                                    </div>
                                }.into_any()
                            } else {
                                // ── Email input ───────────────────────────────
                                view! {
                                    <div>
                                        <label
                                            for="atlas-magic-link-email"
                                            style="
                                                display: block;
                                                font-size: 11px;
                                                font-weight: 600;
                                                letter-spacing: 0.06em;
                                                text-transform: uppercase;
                                                color: #6b6a64;
                                                margin-bottom: 8px;
                                            "
                                        >
                                            "Email address"
                                        </label>

                                        <input
                                            id="atlas-magic-link-email"
                                            type="email"
                                            placeholder="you@example.com"
                                            on:input=move |ev| auth.email.set(event_target_value(&ev))
                                            prop:value=auth.email
                                            style="
                                                display: block;
                                                width: 100%;
                                                box-sizing: border-box;
                                                background: #faf9f5;
                                                border: 1px solid #e8e6dc;
                                                border-radius: 6px;
                                                padding: 10px 12px;
                                                font-size: 14px;
                                                color: #141413;
                                                outline: none;
                                                transition: border-color 0.15s;
                                                margin-bottom: 8px;
                                            "
                                        />

                                        <p style="
                                            font-size: 12px;
                                            color: #6b6a64;
                                            margin: 0 0 20px 0;
                                            line-height: 1.45;
                                        ">
                                            "We'll send a one-time sign-in link to this address."
                                        </p>

                                        // Error state
                                        {move || auth.error.get().map(|e| view! {
                                            <div style="
                                                border-left: 3px solid #c0392b;
                                                border-radius: 2px;
                                                background: #fdf3f2;
                                                padding: 10px 12px 10px 14px;
                                                margin-bottom: 16px;
                                                font-size: 13px;
                                                color: #922b21;
                                                line-height: 1.45;
                                            ">
                                                {e}
                                            </div>
                                        })}

                                        // CTA button with loading spinner
                                        <button
                                            id="atlas-send-link-btn"
                                            type="button"
                                            on:click=move |_| { auth.dispatch_login.dispatch(()); }
                                            disabled=move || {
                                                auth.is_loading.get()
                                                || auth.countdown.get() > 0
                                                || auth.email.get().is_empty()
                                            }
                                            style=move || format!("
                                                display: flex;
                                                align-items: center;
                                                justify-content: center;
                                                gap: 8px;
                                                width: 100%;
                                                background: #1B365D;
                                                color: #faf9f5;
                                                border: none;
                                                border-radius: 6px;
                                                padding: 12px 20px;
                                                font-size: 14px;
                                                font-weight: 500;
                                                cursor: {};
                                                transition: background 0.15s, opacity 0.15s;
                                                text-align: center;
                                                opacity: {};
                                            ",
                                            if auth.is_loading.get() || auth.countdown.get() > 0 || auth.email.get().is_empty() {
                                                "not-allowed"
                                            } else {
                                                "pointer"
                                            },
                                            if auth.is_loading.get() || auth.countdown.get() > 0 || auth.email.get().is_empty() {
                                                "0.6"
                                            } else {
                                                "1"
                                            })
                                        >
                                            // Spinner shown while loading
                                            {move || if auth.is_loading.get() {
                                                view! {
                                                    <svg
                                                        width="14" height="14"
                                                        viewBox="0 0 24 24"
                                                        style="animation: atlas-spin 0.8s linear infinite; flex-shrink: 0;"
                                                    >
                                                        <circle cx="12" cy="12" r="10" fill="none" stroke="#faf9f5" stroke-width="3" stroke-dasharray="31.4" stroke-dashoffset="10"/>
                                                    </svg>
                                                }.into_any()
                                            } else {
                                                view! { <span/> }.into_any()
                                            }}
                                            {move || if auth.is_loading.get() {
                                                "Sending…".to_string()
                                            } else if auth.countdown.get() > 0 {
                                                format!("Resend in {}s", auth.countdown.get())
                                            } else {
                                                "Send sign-in link".to_string()
                                            }}
                                        </button>

                                        // Spinner keyframe injected once
                                        <style>
                                            "@keyframes atlas-spin { to { transform: rotate(360deg); } }"
                                        </style>
                                    </div>
                                }.into_any()
                            }}
                        </div>
                    }.into_any()

                // ── Passkey flow ───────────────────────────────────────────────
                } else {
                    view! {
                        <div>
                            <p style="
                                font-size: 14px;
                                color: #504e49;
                                line-height: 1.55;
                                margin: 0 0 24px 0;
                            ">
                                "Use a registered passkey — Face ID, Touch ID, or a hardware key — to sign in instantly without a password."
                            </p>

                            // Passkey error
                            {move || auth.error.get().map(|e| view! {
                                <div style="
                                    border-left: 3px solid #c0392b;
                                    border-radius: 2px;
                                    background: #fdf3f2;
                                    padding: 10px 12px 10px 14px;
                                    margin-bottom: 16px;
                                    font-size: 13px;
                                    color: #922b21;
                                    line-height: 1.45;
                                ">
                                    {e}
                                </div>
                            })}

                            <PasskeyLoginButton
                                api_base_url="/api/passkeys".to_string()
                                email=RwSignal::new("".to_string())
                                on_success=handle_passkey_success
                                on_error=handle_passkey_error
                            />

                            // Divider
                            <div style="
                                border-top: 1px solid #e8e6dc;
                                margin: 24px 0 16px;
                            "/>

                            <p style="
                                font-size: 12px;
                                color: #6b6a64;
                                line-height: 1.55;
                                margin: 0;
                            ">
                                "Don't have a passkey yet? Sign in with a magic link first, then register one from your security settings."
                            </p>
                        </div>
                    }.into_any()
                }}
            </div>
        </div>
    }
}
