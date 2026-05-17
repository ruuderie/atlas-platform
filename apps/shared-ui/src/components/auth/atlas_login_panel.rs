// Build: 2026-05-14 — forces anchor image rebuild via CI path detection.
// ARCHITECTURE & HYDRATION INVARIANT — READ BEFORE EDITING
// ============================================================================
// AtlasLoginPanel operates in standard document flow (`flex: 1`).
// It expects to be rendered inside a layout shell that provides clearance for
// the global navigation bar (e.g., `<main pt-24>`).
//
// HYDRATION WARNING:
// If the buttons/tabs on this panel suddenly become unclickable, DO NOT try
// to "fix" it by making this component `position:fixed` or `z-index:70`. 
// The unclickability is almost certainly caused by a WASM/HTML hydration 
// mismatch due to aggressive CDN caching (e.g., Cloudflare serving an old 
// anchor.js bundle against new HTML).
// 
// To fix unclickable buttons, bust the WASM cache by incrementing the
// `output-name` in Cargo.toml (e.g., "anchor-v3").
// ============================================================================
use leptos::prelude::*;
use leptos_router::hooks::use_query_map;

use crate::auth::atlas_auth::{use_atlas_auth, verify_magic_link};
use crate::components::auth::passkey_login::PasskeyLoginButton;

#[derive(Clone, PartialEq)]
enum TokenFailure {
    /// Token string exists in DB but the 15-minute window has passed.
    Expired,
    /// Token was already consumed by a prior successful login click.
    AlreadyUsed,
    /// Token string not found in DB — likely a malformed or copied link.
    NotFound,
    /// Unexpected backend or network error.
    ServerError,
}

#[derive(Clone, PartialEq)]
enum TokenState {
    /// Resource not yet resolved — WASM is still verifying.
    Pending,
    Success,
    Failed(TokenFailure),
}

/// Full auth UI component — owns passkey, magic-link send, AND token verification.
/// Every app using this gets the complete auth flow for free.
#[component]
pub fn AtlasLoginPanel(
    #[prop(into, default = "Atlas".into())]
    app_title: String,
    #[prop(into, optional)]
    on_authenticated: Option<Callback<()>>,
    /// Clean URL to navigate to after token exchange. Defaults to /admin.
    #[prop(into, default = "/admin".into())]
    success_path: String,
) -> impl IntoView {
    let query = use_query_map();
    let token_in_url = query.with_untracked(|q| q.get("token").filter(|t| !t.is_empty()));
    let has_token = token_in_url.is_some();

    if has_token {
        token_view(token_in_url, success_path, on_authenticated).into_any()
    } else {
        login_view(app_title, on_authenticated).into_any()
    }
}

// ── Token verification branch ────────────────────────────────────────────────
fn token_view(
    token: Option<String>,
    success_path: String,
    on_authenticated: Option<Callback<()>>,
) -> impl IntoView {
    // IMPORTANT: Use LocalResource (client-only) — NOT Resource.
    // Resource::new runs during SSR, which would consume the single-use token
    // server-side before WASM loads. LocalResource skips SSR entirely: WASM
    // verifies the token, the Set-Cookie header is proxied, then we navigate.
    let resource = LocalResource::new(move || {
        let tok = token.clone();
        async move {
            match tok {
                Some(t) if !t.is_empty() => match verify_magic_link(t).await {
                    Ok(_) => TokenState::Success,
                    Err(e) => {
                        // Map the structured error_code prefix the backend returns
                        // into a typed TokenFailure variant.
                        let msg = e.to_string();
                        let failure = if msg.contains("token_already_used") {
                            TokenFailure::AlreadyUsed
                        } else if msg.contains("token_expired") {
                            TokenFailure::Expired
                        } else if msg.contains("token_not_found") {
                            TokenFailure::NotFound
                        } else {
                            TokenFailure::ServerError
                        };
                        TokenState::Failed(failure)
                    }
                },
                _ => TokenState::Failed(TokenFailure::NotFound),
            }
        }
    });

    // CRITICAL: use window.location.replace NOT leptos_router navigate.
    //
    // navigate() is a client-side navigation — the Admin component stays mounted
    // and its auth_resource (keyed on `|| ()`) never re-runs. auth_state stays
    // AuthState::No → the dashboard never renders, even though the session cookie
    // is now set.
    //
    // window.location.replace() forces a full page reload. The server runs a fresh
    // check_session() with the new HttpOnly session cookie in the request, the
    // resource resolves to Authenticated, and the dashboard renders.
    //
    // The on_authenticated callback (if any) still fires first so callers can
    // perform any synchronous side-effects before the navigation happens.
    let clean2 = success_path.clone();
    let on_ok  = on_authenticated.clone();
    Effect::new(move |_| {
        if matches!(resource.get(), Some(TokenState::Success)) {
            if let Some(cb) = &on_ok { cb.run(()); }
            #[cfg(feature = "hydrate")]
            if let Some(w) = web_sys::window() {
                let _ = w.location().replace(&clean2);
            }
        }
    });

    // INVARIANT: This panel flows in the document. Do not use position:fixed.
    // See file header for hydration/cache-busting instructions if buttons break.
    let page = "flex:1;width:100%;display:flex;align-items:center;justify-content:center;background:#f5f4ed;padding:48px 16px;";
    let card = "width:100%;max-width:420px;height:440px;background:#faf9f5;border:1px solid #e8e6dc;border-radius:8px;padding:48px 40px;box-shadow:0 4px 24px rgba(0,0,0,0.05);box-sizing:border-box;";

    view! {
        <Suspense fallback=move || view! {
            <div style=page>
                <div style="text-align:center;">
                    <svg width="32" height="32" viewBox="0 0 24 24"
                        style="animation:atlas-spin 0.8s linear infinite;margin:0 auto 16px;">
                        <circle cx="12" cy="12" r="10" fill="none"
                            stroke="#1B365D" stroke-width="3"
                            stroke-dasharray="31.4" stroke-dashoffset="10"/>
                    </svg>
                    <style>"@keyframes atlas-spin{to{transform:rotate(360deg)}}"</style>
                    <p style="font-size:13px;color:#6b6a64;margin:0;">"Verifying sign-in link\u{2026}"</p>
                </div>
            </div>
        }>
            {move || match resource.get() {
                // Resource not yet resolved — show nothing (Suspense fallback covers this)
                None => view!{ <div/> }.into_any(),

                Some(TokenState::Pending) => view!{ <div/> }.into_any(),

                Some(TokenState::Success) => view! {
                    <div style=page>
                        <div style=card>
                            <div style="border-left:3px solid #1B365D;padding-left:10px;margin-bottom:20px;">
                                <p style="font-size:11px;font-weight:600;letter-spacing:.08em;text-transform:uppercase;color:#1B365D;margin:0 0 4px;">"Signed in"</p>
                                <h1 style="font-size:22px;font-weight:500;color:#141413;margin:0;">"You're in \u{2014} redirecting\u{2026}"</h1>
                            </div>
                            <p style="font-size:14px;color:#504e49;line-height:1.6;margin:0;">
                                "Your sign-in link was accepted. Taking you to the dashboard."
                            </p>
                        </div>
                    </div>
                }.into_any(),

                Some(TokenState::Failed(reason)) => {
                    // Each failure reason gets a distinct heading and explanation.
                    let (heading, detail, hint) = match reason {
                        TokenFailure::AlreadyUsed => (
                            "This link has already been used",
                            "Sign-in links are single-use. This link was consumed by a previous login attempt.",
                            "If that wasn't you, your account is secure — each link can only be activated once.",
                        ),
                        TokenFailure::Expired => (
                            "This link has expired",
                            "Sign-in links expire after 15 minutes. This one is no longer valid.",
                            "Request a new link below and click it within 15 minutes.",
                        ),
                        TokenFailure::NotFound => (
                            "Sign-in link not recognised",
                            "This link doesn't match any active sign-in request. It may have been copied incorrectly.",
                            "Request a fresh link below.",
                        ),
                        TokenFailure::ServerError => (
                            "Something went wrong",
                            "We couldn't verify your sign-in link due to a server error.",
                            "Please try again in a moment.",
                        ),
                    };
                    view! {
                        <div style=page>
                            <div style=card>
                                <div style="border-left:3px solid #c0392b;padding-left:10px;margin-bottom:24px;">
                                    <p style="font-size:11px;font-weight:600;letter-spacing:.08em;text-transform:uppercase;color:#922b21;margin:0 0 4px;">"Sign-in link invalid"</p>
                                    <h1 style="font-size:22px;font-weight:500;color:#141413;margin:0;">{heading}</h1>
                                </div>
                                <p style="font-size:14px;color:#504e49;line-height:1.6;margin:0 0 8px;">{detail}</p>
                                <p style="font-size:13px;color:#6b6a64;line-height:1.55;margin:0 0 28px;">{hint}</p>
                                <button type="button"
                                    on:click=move |_| {
                                        #[cfg(feature = "hydrate")]
                                        if let Some(w) = web_sys::window() {
                                            let _ = w.location().replace("/admin?mode=email");
                                        }
                                    }
                                    style="display:block;width:100%;box-sizing:border-box;background:#1B365D;color:#faf9f5;border:none;border-radius:6px;padding:12px 20px;font-size:14px;font-weight:500;text-align:center;cursor:pointer;"
                                >
                                    "Request a new sign-in link"
                                </button>
                            </div>
                        </div>
                    }.into_any()
                },
            }}
        </Suspense>
    }
}

// ── Normal login panel (no token) ────────────────────────────────────────────
fn login_view(app_title: String, on_authenticated: Option<Callback<()>>) -> impl IntoView {
    let auth           = use_atlas_auth();
    let email_sig      = auth.email;
    let is_loading_sig = auth.is_loading;
    let countdown_sig  = auth.countdown;
    let ml_sent_sig    = auth.magic_link_sent;
    let error_sig      = auth.error;
    let dispatch_login = auth.dispatch_login;

    // Default to email/magic-link tab when arriving via ?mode=email — e.g. after a
    // hard-reload from the "Request a new sign-in link" button on the expired-token card.
    let query = use_query_map();
    let initial_email_mode = query.with_untracked(|q| q.get("mode").as_deref() == Some("email"));
    let email_mode = RwSignal::new(initial_email_mode);

    let on_pk = on_authenticated.clone();
    let handle_passkey_success = Callback::new(move |_: String| {
        if let Some(cb) = &on_pk { cb.run(()); }
        else { let _ = web_sys::window().unwrap().location().reload(); }
    });
    let handle_passkey_error = Callback::new(move |err: String| {
        error_sig.set(Some(err));
    });

    // INVARIANT: This panel flows in the document. Do not use position:fixed.
    // See file header for hydration/cache-busting instructions if buttons break.
    let page = "flex:1;width:100%;display:flex;align-items:center;justify-content:center;background:#f5f4ed;padding:48px 16px;";
    let card = "width:100%;max-width:420px;height:440px;background:#faf9f5;border:1px solid #e8e6dc;border-radius:8px;padding:48px 40px;box-shadow:0 4px 24px rgba(0,0,0,0.05);box-sizing:border-box;";


    view! {
        <div style=page>
          <div style=card>
            // Header
            <div style="border-left:3px solid #1B365D;padding-left:10px;margin-bottom:32px;">
                <p style="font-size:11px;font-weight:600;letter-spacing:.08em;text-transform:uppercase;color:#6b6a64;margin:0 0 4px;">"Secure Login"</p>
                <h1 style="font-size:26px;font-weight:500;line-height:1.2;color:#141413;margin:0;">{app_title}</h1>
            </div>
            // Tabs
            // Note: type="button" prevents form submission on click.
            // The primary pre-hydration guard is Resource::new_blocking in admin.rs
            // which eliminates the Suspense DOM swap that caused the full-page refresh.
            <div style="display:flex;gap:8px;margin-bottom:32px;">
                <button type="button"
                    on:click=move |ev| { ev.prevent_default(); email_mode.set(false); }
                    style=move || format!("padding:6px 14px;border-radius:4px;font-size:12px;font-weight:500;border:none;pointer-events:auto;cursor:pointer;{}",
                        if !email_mode.get() {"background:#1B365D;color:#faf9f5;"} else {"background:#e8e6dc;color:#3d3d3a;"})
                >"Passkey"</button>
                <button type="button"
                    on:click=move |ev| { ev.prevent_default(); email_mode.set(true); }
                    style=move || format!("padding:6px 14px;border-radius:4px;font-size:12px;font-weight:500;border:none;pointer-events:auto;cursor:pointer;{}",
                        if email_mode.get() {"background:#1B365D;color:#faf9f5;"} else {"background:#e8e6dc;color:#3d3d3a;"})
                >"Magic Link"</button>
            </div>
            // Content
            {move || if email_mode.get() {
                // Magic link send flow
                view! {
                    <div>
                    {move || if ml_sent_sig.get() {
                        view! {
                            <div style="border-left:3px solid #1B365D;background:#EEF2F7;padding:20px 20px 20px 18px;">
                                <p style="font-size:13px;font-weight:600;color:#1B365D;margin:0 0 8px;">"Check your inbox"</p>
                                <p style="font-size:13px;color:#504e49;line-height:1.55;margin:0 0 4px;">
                                    "A sign-in link was sent to "
                                    <span style="color:#1B365D;font-weight:500;">{move || email_sig.get()}</span>"."
                                </p>
                                <p style="font-size:12px;color:#6b6a64;margin:0 0 16px;">"Expires in 15 minutes. Check spam if it doesn't arrive."</p>
                                <button type="button"
                                    on:click=move |_| { ml_sent_sig.set(false); error_sig.set(None); }
                                    style="background:none;border:none;cursor:pointer;padding:0;font-size:12px;color:#6b6a64;text-decoration:underline;"
                                >"← Try a different email"</button>
                            </div>
                        }.into_any()
                    } else {
                        view! {
                            <div>
                                <label for="atlas-ml-email" style="display:block;font-size:11px;font-weight:600;letter-spacing:.06em;text-transform:uppercase;color:#6b6a64;margin-bottom:8px;">"Email address"</label>
                                <input id="atlas-ml-email" type="email" placeholder="you@example.com"
                                    on:input=move |ev| email_sig.set(event_target_value(&ev))
                                    prop:value=move || email_sig.get()
                                    style="display:block;width:100%;box-sizing:border-box;background:#faf9f5;border:1px solid #e8e6dc;border-radius:6px;padding:10px 12px;font-size:14px;color:#141413;outline:none;margin-bottom:8px;"
                                />
                                <p style="font-size:12px;color:#6b6a64;margin:0 0 20px;line-height:1.45;">"We'll send a one-time sign-in link to this address."</p>
                                {move || error_sig.get().map(|e| view! {
                                    <div style="border-left:3px solid #c0392b;background:#fdf3f2;padding:10px 12px 10px 14px;margin-bottom:16px;font-size:13px;color:#922b21;line-height:1.45;">{e}</div>
                                })}
                                <button id="atlas-send-link-btn" type="button"
                                    on:click=move |_| {
                                        // Synchronous guard: re-check signal state before dispatching.
                                        // prop:disabled prevents NEW clicks, but events already queued
                                        // in the browser event loop can still fire before the DOM
                                        // reflects the disabled state. Checking the signal value
                                        // (not the DOM property) closes this race window.
                                        if is_loading_sig.get_untracked()
                                            || countdown_sig.get_untracked() != 0
                                            || email_sig.get_untracked().trim().is_empty()
                                        {
                                            return;
                                        }
                                        let _ = dispatch_login.dispatch(());
                                    }
                                    prop:disabled=move || is_loading_sig.get() || (countdown_sig.get() != 0) || email_sig.get().trim().is_empty()
                                    style=move || format!(
                                        "display:flex;align-items:center;justify-content:center;gap:8px;width:100%;background:#1B365D;color:#faf9f5;border:none;border-radius:6px;padding:12px 20px;font-size:14px;font-weight:500;cursor:{};opacity:{};transition:opacity .15s;",
                                        if is_loading_sig.get()||countdown_sig.get()>0||email_sig.get().trim().is_empty() {"not-allowed"} else {"pointer"},
                                        if is_loading_sig.get()||countdown_sig.get()>0||email_sig.get().trim().is_empty() {"0.6"} else {"1.0"})
                                >
                                    {move || if is_loading_sig.get() { "Sending\u{2026}".to_string() }
                                             else if countdown_sig.get() > 0 { format!("Resend in {}s", countdown_sig.get()) }
                                             else { "Send sign-in link".to_string() }}
                                </button>
                                <style>"@keyframes atlas-spin{to{transform:rotate(360deg)}}"</style>
                            </div>
                        }.into_any()
                    }}
                    </div>
                }.into_any()
            } else {
                // Passkey flow
                view! {
                    <div>
                        <p style="font-size:14px;color:#504e49;line-height:1.55;margin:0 0 24px;">
                            "Use a registered passkey — Face ID, Touch ID, or a hardware key — to sign in instantly."
                        </p>
                        {move || error_sig.get().map(|e| view! {
                            <div style="border-left:3px solid #c0392b;background:#fdf3f2;padding:10px 12px 10px 14px;margin-bottom:16px;font-size:13px;color:#922b21;">{e}</div>
                        })}
                        <PasskeyLoginButton
                            api_base_url="/api/passkeys".to_string()
                            email=RwSignal::new("".to_string())
                            on_success=handle_passkey_success
                            on_error=handle_passkey_error
                        />
                        <div style="border-top:1px solid #e8e6dc;margin:24px 0 16px;"/>
                        <p style="font-size:12px;color:#6b6a64;line-height:1.55;margin:0;">
                            "Don't have a passkey yet? Sign in with a magic link first, then register one from your security settings."
                        </p>
                    </div>
                }.into_any()
            }}
          </div>
        </div>
    }
}
