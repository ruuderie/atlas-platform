// apps/folio/src/components/wizard_shell.rs
//
// WizardShell — shared split-panel layout for all 9 persona onboarding wizards.
//
// # Layout
//
//   ┌──────────────────────────────────────────────────────────────────────┐
//   │  Topnav (56px sticky — logo + persona tag + step counter + exit)     │
//   ├──────────────────┬───────────────────────────────────────────────────┤
//   │  Context panel   │  Form panel                                        │
//   │  (360px, dark)   │  (flex: 1, light, scrollable)                     │
//   │                  │  ┌─────────────────────────────────────────────┐  │
//   │  - Persona pill  │  │  Step content (slot)                        │  │
//   │  - Headline      │  │                                             │  │
//   │  - Feature list  │  │                                             │  │
//   │  - Divider       │  │                                             │  │
//   │  - Step nav      │  └─────────────────────────────────────────────┘  │
//   │    (numbered,    │  ┌─────────────────────────────────────────────┐  │
//   │     done/active) │  │  Footer (sticky bottom — Back + Continue)   │  │
//   │                  │  └─────────────────────────────────────────────┘  │
//   └──────────────────┴───────────────────────────────────────────────────┘
//
// # Invite Code Context
//
// If the user arrives via /join/:code → /onboard/:role?code=XXX, the `invite_code`
// signal is pre-populated from the GET /api/folio/invite/resolve/:code response.
// The context panel shows the resolved entity (unit, landlord, etc.) instead of the
// generic persona marketing copy.

use leptos::prelude::*;
use serde::{Deserialize, Serialize};

// ── Invite code context types ─────────────────────────────────────────────────

/// Returned by GET /api/folio/invite/resolve/:code.
/// Intentionally contains NO PII — safe to display to unauthenticated users.
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct ResolvedInviteCode {
    pub code: String,
    pub role: String,
    pub label: Option<String>,
    pub invite_message: Option<String>,
    pub context: InviteCodeContext,
    pub expires_at: Option<String>,
    pub uses_remaining: Option<i32>,
    pub is_valid: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct InviteCodeContext {
    pub asset: Option<ContextEntity>,
    pub landlord: Option<ContextEntity>,
    pub broker: Option<ContextEntity>,
    pub asset_count: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct ContextEntity {
    pub name: String,
    pub address: Option<String>,
}

// ── Server function ───────────────────────────────────────────────────────────

/// Resolve an invite code from the server. Safe to call with empty string
/// (returns None without making a request).
#[server(ResolveInviteCode, "/api")]
pub async fn resolve_invite_code(
    code: String,
) -> Result<Option<ResolvedInviteCode>, server_fn::error::ServerFnError> {
    if code.trim().is_empty() {
        return Ok(None);
    }
    crate::atlas_client::fetch::<ResolvedInviteCode>(&format!(
        "/api/folio/invite/resolve/{}",
        code.trim()
    ))
    .await
    .map(Some)
    .or_else(|_| Ok(None)) // 404/410 → treat as no code
}

// ── Step descriptor ───────────────────────────────────────────────────────────

/// A single step in a wizard flow.
#[derive(Clone, PartialEq)]
pub struct WizardStepDesc {
    pub id: &'static str,
    pub label: &'static str,
    pub skippable: bool,
}

/// Per-step left-rail copy (stitch `ctx` array). When provided, the shell
/// renders step-aware tag + icon + headline/body/bullets instead of a static
/// persona pill.
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct WizardCtxStep {
    pub glyph: &'static str,
    pub headline: &'static str,
    pub body: &'static str,
    pub bullets: &'static [&'static str],
}

/// Verified identity established by WizardShell (session, peek, or OTP).
/// Provided to wizard children via context — use [`VerifiedEmailField`].
#[derive(Clone, Copy)]
pub struct WizardAuthCtx {
    pub email: Signal<Option<String>>,
}

fn accept_verified_email(
    email: String,
    verified_email: RwSignal<Option<String>>,
    session_email_sig: Option<RwSignal<Option<String>>>,
    pre_auth_done: RwSignal<bool>,
) {
    let email = email.trim().to_string();
    if email.is_empty() {
        return;
    }
    crate::auth::stash_verified_email(&email);
    verified_email.set(Some(email.clone()));
    if let Some(sig) = session_email_sig {
        sig.set(Some(email));
    }
    pre_auth_done.set(true);
}

// ── WizardShell component ─────────────────────────────────────────────────────

/// Shared split-panel wizard shell.
///
/// Callers supply:
/// - `steps` — ordered list of step descriptors
/// - `current_idx` — reactive index into `steps`
/// - `persona_pill` — e.g. "Landlord" or "Property Manager"
/// - `persona_icon` — Material Symbol name (filled), e.g. "apartment"
/// - `accent_color` — CSS color for pill / icon accent, e.g. "#0284c7"
/// - `panel_bg`    — dark panel background, e.g. "#0e1c36"
/// - `ctx_headline` — context panel headline (ignored when `ctx_steps` is set)
/// - `ctx_body`    — context panel body text / feature list (ignored when `ctx_steps` is set)
/// - `ctx_steps`   — optional per-step left-rail copy (stitch parity)
/// - `progress_label` — sidebar progress heading (default "Setup progress")
/// - `nav_detail` — optional topnav suffix when authenticated (e.g. "5 steps, ~4 min")
/// - `invite_code`  — resolved invite code signal (or None)
/// - `on_next` / `on_prev` — navigation callbacks
/// - `is_last_step` — if true, Next becomes primary CTA
/// - `step_content` — the form content for the current step (Children)
/// - `session_email` — optional signal that will be populated with the verified email
///   after OTP auth. Wizards can read this to pre-fill / skip their email field.
#[component]
pub fn WizardShell(
    steps: Vec<WizardStepDesc>,
    current_idx: RwSignal<usize>,
    persona_pill: &'static str,
    persona_icon: &'static str,
    accent_color: &'static str,
    panel_bg: &'static str,
    ctx_headline: &'static str,
    #[prop(into)] ctx_body: ViewFn,
    /// Stitch-style per-step context. When set, replaces persona pill + static headline/body.
    #[prop(optional)]
    ctx_steps: Option<Vec<WizardCtxStep>>,
    #[prop(optional)]
    progress_label: Option<&'static str>,
    /// Shown in topnav after persona name when authenticated, e.g. "5 steps, ~4 min".
    #[prop(optional)]
    nav_detail: Option<&'static str>,
    #[prop(optional)] invite_code: Option<RwSignal<Option<ResolvedInviteCode>>>,
    /// Set this signal to receive the verified email from the OTP pre-step.
    /// If the user was already authenticated, it is populated from the session.
    #[prop(optional)]
    session_email: Option<RwSignal<Option<String>>>,
    on_next: Callback<()>,
    on_prev: Callback<()>,
    is_last_step: Signal<bool>,
    next_label: Signal<&'static str>,
    children: ChildrenFn,
) -> impl IntoView {
    let total = steps.len();
    let steps_store = StoredValue::new(steps);
    let children_store = StoredValue::new(children);
    let ctx_steps_store = StoredValue::new(ctx_steps);
    let progress_label = progress_label.unwrap_or("Setup progress");
    let nav_detail = nav_detail;

    // ── Pre-auth phase ─────────────────────────────────────────────────────────
    // For unauthenticated users (cold QR scan / direct mail), we show an email + OTP
    // entry step before the wizard. On success the session cookie is set by the backend.

    // Whether the pre-auth step has been completed (either via OTP or existing session)
    let pre_auth_done: RwSignal<bool> = RwSignal::new(false);
    let verified_email: RwSignal<Option<String>> = RwSignal::new(None);
    provide_context(WizardAuthCtx {
        email: verified_email.into(),
    });

    // Local OTP flow state
    let otp_email: RwSignal<String> = RwSignal::new(String::new());
    let otp_code: RwSignal<String> = RwSignal::new(String::new());
    let otp_sent: RwSignal<bool> = RwSignal::new(false);
    let otp_sending: RwSignal<bool> = RwSignal::new(false);
    let otp_verifying: RwSignal<bool> = RwSignal::new(false);
    let otp_error: RwSignal<Option<String>> = RwSignal::new(None);

    // Client-only session probe. Fresh magic-link users often have a valid cookie
    // while /api/folio/me is still 403 — peek_auth_session covers that case.
    // sessionStorage is a same-tab assist after magic-link verify (never alone).
    let session_email_sig = session_email;
    let session_probe = LocalResource::new(|| async {
        match crate::auth::get_session().await {
            Ok(info) => Some(info.email),
            Err(_) => match crate::auth::peek_auth_session().await {
                Ok(peek) => Some(peek.email),
                Err(_) => {
                    // Brief retry — cookie from Set-Cookie may not be on the first
                    // server-fn round-trip after magic-link redirect.
                    match crate::auth::peek_auth_session().await {
                        Ok(peek) => Some(peek.email),
                        Err(_) => crate::auth::read_stashed_verified_email(),
                    }
                }
            },
        }
    });
    Effect::new(move |_| {
        if let Some(Some(email)) = session_probe.get() {
            accept_verified_email(email, verified_email, session_email_sig, pre_auth_done);
        }
    });

    // Send OTP action
    let send_action = Action::new(move |email: &String| {
        let email = email.clone();
        async move { crate::pages::onboarding::otp_client::send_otp(email).await }
    });

    let send_email_clone = otp_email;
    let on_send = move |_| {
        let email = send_email_clone.get();
        if email.trim().is_empty() {
            return;
        }
        otp_error.set(None);
        otp_sending.set(true);
        send_action.dispatch(email);
    };

    Effect::new(move |_| {
        if let Some(result) = send_action.value().get() {
            otp_sending.set(false);
            match result {
                Ok(_) => {
                    otp_sent.set(true);
                }
                Err(e) => {
                    otp_error.set(Some(format!("Could not send code: {e}")));
                }
            }
        }
    });

    // Verify OTP action
    let verify_action = Action::new(move |(email, code): &(String, String)| {
        let email = email.clone();
        let code = code.clone();
        async move { crate::pages::onboarding::otp_client::verify_otp(email, code).await }
    });

    let verify_email_clone = otp_email;
    let verify_code_clone = otp_code;
    let on_verify = move |_| {
        let email = verify_email_clone.get();
        let code = verify_code_clone.get();
        if code.trim().is_empty() {
            return;
        }
        otp_error.set(None);
        otp_verifying.set(true);
        verify_action.dispatch((email, code));
    };

    Effect::new(move |_| {
        if let Some(result) = verify_action.value().get() {
            otp_verifying.set(false);
            match result {
                Ok(resp) => {
                    accept_verified_email(
                        resp.email,
                        verified_email,
                        session_email_sig,
                        pre_auth_done,
                    );
                }
                Err(e) => {
                    otp_error.set(Some(format!("Incorrect code — please try again. ({e})")));
                }
            }
        }
    });

    let panel_style = format!(
        "background:{panel_bg}; color:#fff; overflow-y:auto; position:relative; \
         display:flex; flex-direction:column; padding:40px 32px;",
    );

    let pill_style = format!(
        "display:inline-flex; align-items:center; gap:6px; \
         background:rgba(255,255,255,.08); border:1px solid rgba(255,255,255,.15); \
         color:#fff; font-size:11px; font-weight:700; padding:5px 12px; \
         border-radius:20px; margin-bottom:20px;"
    );

    view! {
        <style>
            {WIZARD_CSS}
            {PRE_AUTH_CSS}
        </style>

        // ── Topnav ────────────────────────────────────────────────────────────
        <header class="wiz-nav">
            <div class="wiz-logo">
                <div class="wiz-logo-mark">
                    <span class="ms msf" style="font-size:16px; color:#fff;">"apartment"</span>
                </div>
                <span class="wiz-logo-name">"Folio"</span>
            </div>
            <div class="wiz-nav-center">
                {move || {
                    if !pre_auth_done.get() {
                        return view! {
                            <span>{persona_pill}" Setup \u{b7} "<strong>"Verify email"</strong></span>
                        }.into_any();
                    }
                    if let Some(detail) = nav_detail {
                        view! {
                            <span><strong>{persona_pill}" Setup"</strong>" \u{b7} "{detail}</span>
                        }.into_any()
                    } else {
                        let idx = current_idx.get();
                        view! {
                            <span>
                                {persona_pill}" Setup \u{b7} "
                                <strong>{format!("Step {} of {}", idx + 1, total)}</strong>
                            </span>
                        }.into_any()
                    }
                }}
            </div>
            <a href="/dashboard" class="wiz-exit">
                <span class="ms">"close"</span>
                "Exit"
            </a>
        </header>

        // ── Split body ────────────────────────────────────────────────────────
        <div class="wiz-split">

            // ── Context panel ─────────────────────────────────────────────────
            <aside class="wiz-ctx" style=panel_style>
                <div class="wiz-ctx-glow" style=format!(
                    "background: radial-gradient(ellipse at 70% 0%, {}47 0%, transparent 60%), \
                                 radial-gradient(ellipse at 20% 95%, rgba(16,185,129,.16) 0%, transparent 50%);",
                    accent_color
                )></div>
                <div class="wiz-ctx-inner">

                    // If an invite code is resolved, show entity card instead of generic copy
                    {move || {
                        let code_opt = invite_code.and_then(|sig| sig.get());
                        if let Some(code) = code_opt {
                            view! {
                                <div class="wiz-invite-resolved">
                                    <div class="wiz-invite-badge">
                                        <span class="ms msf" style=format!("font-size:14px; color:{};", accent_color)>
                                            {persona_icon}
                                        </span>
                                        {persona_pill}
                                    </div>
                                    {if let Some(label) = &code.label {
                                        view! { <div class="wiz-invite-label">{label.clone()}</div> }.into_any()
                                    } else {
                                        view! { <span></span> }.into_any()
                                    }}
                                    {if let Some(asset) = &code.context.asset {
                                        let addr = asset.address.clone().unwrap_or_default();
                                        view! {
                                            <div class="wiz-invite-entity">
                                                <span class="ms msf wiz-ent-ico">"location_on"</span>
                                                <div>
                                                    <div class="wiz-ent-name">{asset.name.clone()}</div>
                                                    <div class="wiz-ent-addr">{addr}</div>
                                                </div>
                                            </div>
                                        }.into_any()
                                    } else { view! { <span></span> }.into_any() }}
                                    {if let Some(landlord) = &code.context.landlord {
                                        view! {
                                            <div class="wiz-invite-entity">
                                                <span class="ms msf wiz-ent-ico">"corporate_fare"</span>
                                                <div>
                                                    <div class="wiz-ent-name">{landlord.name.clone()}</div>
                                                </div>
                                            </div>
                                        }.into_any()
                                    } else { view! { <span></span> }.into_any() }}
                                    {if let Some(msg) = &code.invite_message {
                                        view! {
                                            <div class="wiz-invite-msg">
                                                <span class="ms wiz-msg-ico">"format_quote"</span>
                                                {msg.clone()}
                                            </div>
                                        }.into_any()
                                    } else { view! { <span></span> }.into_any() }}
                                    <div class="wiz-code-pill">
                                        <span class="ms" style="font-size:13px;">"qr_code_2"</span>
                                        {code.code.clone()}
                                    </div>
                                </div>
                            }.into_any()
                        } else if ctx_steps_store.with_value(|c| c.is_some()) {
                            let idx = if pre_auth_done.get() { current_idx.get() } else { 0 };
                            let step = ctx_steps_store.with_value(|c| {
                                c.as_ref().and_then(|steps| steps.get(idx).copied())
                            });
                            if let Some(step) = step {
                                let tag = if pre_auth_done.get() {
                                    format!("Step {} of {}", idx + 1, total)
                                } else {
                                    "Verify email".to_string()
                                };
                                view! {
                                    <div>
                                        <div class="wiz-ctx-tag">{tag}</div>
                                        <div class="wiz-ctx-icon">
                                            <span class="ms msf">{step.glyph}</span>
                                        </div>
                                        <h2 class="wiz-ctx-h">{step.headline}</h2>
                                        <p class="wiz-ctx-p">{step.body}</p>
                                        <ul class="wiz-ctx-list">
                                            {step.bullets.iter().map(|b| view! {
                                                <li><span class="ms msf">"check_circle"</span>{*b}</li>
                                            }).collect_view()}
                                        </ul>
                                    </div>
                                }.into_any()
                            } else {
                                view! { <span></span> }.into_any()
                            }
                        } else {
                            // Generic persona copy from caller
                            view! {
                                <div>
                                    <div style=pill_style.clone()>
                                        <span class="ms msf" style=format!("font-size:14px; color:{};", accent_color)>
                                            {persona_icon}
                                        </span>
                                        {persona_pill}
                                    </div>
                                    <h2 class="wiz-ctx-h">{ctx_headline}</h2>
                                    {ctx_body.run()}
                                </div>
                            }.into_any()
                        }
                    }}

                    // Step navigation (always shown)
                    <div class="wiz-ctx-div"></div>
                    <div class="wiz-nav-label">{progress_label}</div>
                    <div class="wiz-ctx-steps">
                        {steps_store.with_value(|steps| {
                            steps.iter().enumerate().map(|(i, step)| {
                                let label = step.label;
                                let is_done    = move || pre_auth_done.get() && current_idx.get() > i;
                                let is_active  = move || pre_auth_done.get() && current_idx.get() == i;
                                view! {
                                    <div class=move || {
                                        if is_done() { "wiz-ctx-si done" }
                                        else if is_active() { "wiz-ctx-si active" }
                                        else { "wiz-ctx-si" }
                                    }>
                                        <div class="wiz-ctx-num">
                                            {move || if is_done() { "✓".to_string() }
                                             else { (i + 1).to_string() }}
                                        </div>
                                        <span>{label}</span>
                                    </div>
                                }
                            }).collect_view()
                        })}
                    </div>
                </div>
            </aside>

            // ── Form panel ────────────────────────────────────────────────────
            <main class="wiz-fp">
                <Show
                    when=move || pre_auth_done.get()
                    fallback=move || {
                        // Same column as wizard steps (stitch `.fi`) — not a floating modal.
                        view! {
                            <div class="wiz-fi wiz-anim">
                                <Show when=move || !otp_sent.get() fallback=move || {
                                    view! {
                                        <div class="wiz-s-badge">
                                            <span class="ms" style="font-size:13px;">"mark_email_read"</span>
                                            "Verify email"
                                        </div>
                                        <h1 class="wiz-s-title">"Check your email"</h1>
                                        <p class="wiz-s-sub">
                                            "We sent a 6-digit code to "
                                            <strong>{move || otp_email.get()}</strong>
                                            ". Enter it below to continue."
                                        </p>
                                        <div class="wiz-card">
                                            <div class="wiz-ct">"Verification code"</div>
                                            <div class="wiz-f">
                                                <label class="wiz-label" for="otp-code-input">"Code"</label>
                                                <input
                                                    id="otp-code-input"
                                                    type="text"
                                                    inputmode="numeric"
                                                    autocomplete="one-time-code"
                                                    placeholder="000 000"
                                                    maxlength="7"
                                                    class="wiz-inp pre-auth-code"
                                                    prop:value=move || otp_code.get()
                                                    on:input=move |ev| {
                                                        otp_code.set(event_target_value(&ev));
                                                    }
                                                    on:keydown=move |ev| {
                                                        if ev.key() == "Enter" {
                                                            let email = verify_email_clone.get();
                                                            let code  = verify_code_clone.get();
                                                            if !code.trim().is_empty() {
                                                                otp_error.set(None);
                                                                otp_verifying.set(true);
                                                                verify_action.dispatch((email, code));
                                                            }
                                                        }
                                                    }
                                                />
                                            </div>
                                            {move || otp_error.get().map(|e| view! {
                                                <div class="pre-auth-error">{e}</div>
                                            })}
                                            <button
                                                id="otp-verify-btn"
                                                class="wiz-btn wiz-btn-primary pre-auth-cta"
                                                on:click=on_verify
                                                disabled=move || otp_verifying.get()
                                            >
                                                {move || if otp_verifying.get() {
                                                    "Verifying…".to_string()
                                                } else {
                                                    "Verify & Continue".to_string()
                                                }}
                                                <Show when=move || !otp_verifying.get()>
                                                    <span class="ms">"arrow_forward"</span>
                                                </Show>
                                            </button>
                                            <button
                                                class="pre-auth-link"
                                                on:click=move |_| {
                                                    otp_sent.set(false);
                                                    otp_code.set(String::new());
                                                    otp_error.set(None);
                                                }
                                            >
                                                "← Change email"
                                            </button>
                                        </div>
                                    }.into_any()
                                }>
                                    <div class="wiz-s-badge">
                                        <span class="ms" style="font-size:13px;">"mail"</span>
                                        "Verify email"
                                    </div>
                                    <h1 class="wiz-s-title">"Verify your email"</h1>
                                    <p class="wiz-s-sub">
                                        "Enter your email and we’ll send a one-time code so we know it’s you."
                                    </p>
                                    <div class="wiz-card">
                                        <div class="wiz-ct">"Email"</div>
                                        <div class="wiz-f">
                                            <label class="wiz-label" for="otp-email-input">"Email address"</label>
                                            <input
                                                id="otp-email-input"
                                                type="email"
                                                autocomplete="email"
                                                placeholder="you@example.com"
                                                class="wiz-inp"
                                                prop:value=move || otp_email.get()
                                                on:input=move |ev| {
                                                    otp_email.set(event_target_value(&ev));
                                                }
                                                on:keydown=move |ev| {
                                                    if ev.key() == "Enter" {
                                                        let email = send_email_clone.get();
                                                        if !email.trim().is_empty() {
                                                            otp_error.set(None);
                                                            otp_sending.set(true);
                                                            send_action.dispatch(email);
                                                        }
                                                    }
                                                }
                                            />
                                        </div>
                                        {move || otp_error.get().map(|e| view! {
                                            <div class="pre-auth-error">{e}</div>
                                        })}
                                        <button
                                            id="otp-send-btn"
                                            class="wiz-btn wiz-btn-primary pre-auth-cta"
                                            on:click=on_send
                                            disabled=move || otp_sending.get()
                                        >
                                            {move || if otp_sending.get() {
                                                "Sending…".to_string()
                                            } else {
                                                "Send Code".to_string()
                                            }}
                                            <Show when=move || !otp_sending.get()>
                                                <span class="ms">"arrow_forward"</span>
                                            </Show>
                                        </button>
                                    </div>
                                    <p class="pre-auth-footer-note">
                                        "Already have an account? "
                                        <a href="/login" class="pre-auth-link-inline">"Sign in →"</a>
                                    </p>
                                </Show>
                            </div>
                        }
                    }
                >
                    // ── Normal wizard content (authenticated) ─────────────
                    <div class="wiz-fi">
                        {children_store.with_value(|children| children())}
                    </div>
                    // ── Sticky footer ─────────────────────────────────────
                    <footer class="wiz-ftr">
                        <div class="wiz-ftr-in">
                            <div class="wiz-step-ind">
                                "Step " <strong>{move || current_idx.get() + 1}</strong>
                                " of " <strong>{total}</strong>
                            </div>
                            <div class="wiz-btn-g">
                                <Show when=move || { current_idx.get() > 0 }>
                                    <button class="wiz-btn wiz-btn-ghost" on:click=move |_| on_prev.run(())>
                                        <span class="ms">"arrow_back"</span>
                                        "Back"
                                    </button>
                                </Show>
                                <button
                                    class=move || {
                                        if is_last_step.get() { "wiz-btn wiz-btn-success" }
                                        else { "wiz-btn wiz-btn-primary" }
                                    }
                                    on:click=move |_| on_next.run(())
                                >
                                    <Show when=move || is_last_step.get()>
                                        <span class="ms msf">"rocket_launch"</span>
                                    </Show>
                                    {move || next_label.get()}
                                    <Show when=move || !is_last_step.get()>
                                        <span class="ms">"arrow_forward"</span>
                                    </Show>
                                </button>
                            </div>
                        </div>
                    </footer>
                </Show>
            </main>

        </div>
    }
}

// ── CSS ───────────────────────────────────────────────────────────────────────

const WIZARD_CSS: &str = r#"
@import url('https://fonts.googleapis.com/css2?family=Inter:wght@300;400;500;600;700;800;900&display=swap');
@import url('https://fonts.googleapis.com/css2?family=Material+Symbols+Outlined:opsz,wght,FILL,GRAD@20..48,100..700,0..1,-50..200&display=swap');

.ms {
    font-family: 'Material Symbols Outlined';
    font-weight: normal; font-style: normal;
    line-height: 1; letter-spacing: normal;
    text-transform: none; display: inline-block;
    white-space: nowrap; direction: ltr;
    -webkit-font-smoothing: antialiased;
    font-variation-settings: 'FILL' 0, 'wght' 400;
}
.msf { font-variation-settings: 'FILL' 1, 'wght' 400; }

html, body { height: 100%; }
body { font-family: 'Inter', sans-serif; margin: 0; }

/* Topnav — stitch wiz_landlord_onboard tokens */
.wiz-nav {
    background: #fff; border-bottom: 1px solid #e5e7eb;
    height: 56px; padding: 0 24px;
    display: flex; align-items: center; justify-content: space-between;
    position: sticky; top: 0; z-index: 100; flex-shrink: 0;
}
.wiz-logo { display: flex; align-items: center; gap: 10px; }
.wiz-logo-mark {
    width: 32px; height: 32px; background: #111827;
    border-radius: 8px; display: flex; align-items: center; justify-content: center;
}
.wiz-logo-name { font-size: 15px; font-weight: 700; letter-spacing: -.02em; color: #111827; }
.wiz-nav-center { font-size: 13px; color: #6b7280; text-align: center; }
.wiz-nav-center strong { color: #111827; }
.wiz-exit {
    display: flex; align-items: center; gap: 6px;
    font-size: 13px; color: #6b7280; text-decoration: none;
    padding: 6px 10px; border-radius: 8px; transition: .15s;
}
.wiz-exit:hover { background: #f4f6f9; color: #111827; }

/* Split */
.wiz-split {
    display: grid; grid-template-columns: 360px 1fr;
    height: calc(100vh - 56px); overflow: hidden;
}

/* Context panel */
.wiz-ctx { position: relative; min-width: 0; }
.wiz-ctx-glow {
    position: absolute; inset: 0; pointer-events: none;
}
.wiz-ctx-inner {
    position: relative; z-index: 1;
    display: flex; flex-direction: column; height: 100%;
}
.wiz-ctx-tag {
    font-size: 11px; font-weight: 700; text-transform: uppercase;
    letter-spacing: .1em; color: rgba(255,255,255,.4); margin-bottom: 28px;
}
.wiz-ctx-icon {
    width: 52px; height: 52px;
    background: rgba(255,255,255,.07); border: 1px solid rgba(255,255,255,.12);
    border-radius: 14px; display: flex; align-items: center; justify-content: center;
    margin-bottom: 18px;
}
.wiz-ctx-icon .ms { font-size: 24px; color: rgba(255,255,255,.9); }
.wiz-ctx-h {
    font-size: 20px; font-weight: 800; line-height: 1.3;
    letter-spacing: -.02em; margin-bottom: 8px;
}
.wiz-ctx-p {
    font-size: 13px; line-height: 1.7;
    color: rgba(255,255,255,.5); margin-bottom: 28px;
}
.wiz-ctx-list {
    list-style: none; margin: 0; padding: 0;
    display: flex; flex-direction: column; gap: 10px;
}
.wiz-ctx-list li {
    display: flex; align-items: flex-start; gap: 9px;
    font-size: 13px; color: rgba(255,255,255,.6); line-height: 1.5;
}
.wiz-ctx-list .ms {
    font-size: 17px; color: #10b981; flex-shrink: 0; margin-top: 1px;
}
.wiz-ctx-div { height: 1px; background: rgba(255,255,255,.08); margin: 28px 0; }
.wiz-nav-label {
    font-size: 11px; font-weight: 600; text-transform: uppercase;
    letter-spacing: .1em; color: rgba(255,255,255,.3); margin-bottom: 14px;
}
.wiz-ctx-steps { display: flex; flex-direction: column; gap: 7px; }
.wiz-ctx-si {
    display: flex; align-items: center; gap: 11px;
    font-size: 13px; color: rgba(255,255,255,.3); transition: .2s;
}
.wiz-ctx-si.done  { color: rgba(255,255,255,.6); }
.wiz-ctx-si.active { color: #fff; font-weight: 600; }
.wiz-ctx-num {
    width: 22px; height: 22px; border-radius: 50%;
    border: 1.5px solid rgba(255,255,255,.2);
    display: flex; align-items: center; justify-content: center;
    font-size: 11px; font-weight: 700; flex-shrink: 0; transition: .2s;
}
.wiz-ctx-si.done .wiz-ctx-num {
    background: #10b981; border-color: #10b981;
}
.wiz-ctx-si.active .wiz-ctx-num {
    background: #fff; border-color: #fff; color: #111827;
}

/* Invite resolved card */
.wiz-invite-resolved { display: flex; flex-direction: column; gap: 12px; }
.wiz-invite-badge {
    display: inline-flex; align-items: center; gap: 6px;
    background: rgba(255,255,255,.08); border: 1px solid rgba(255,255,255,.15);
    font-size: 11px; font-weight: 700; padding: 5px 12px;
    border-radius: 20px; width: fit-content;
}
.wiz-invite-label { font-size: 18px; font-weight: 800; letter-spacing: -.02em; }
.wiz-invite-entity {
    display: flex; align-items: flex-start; gap: 10px;
    background: rgba(255,255,255,.05); border: 1px solid rgba(255,255,255,.08);
    border-radius: 10px; padding: 12px;
}
.wiz-ent-ico { font-size: 17px; color: rgba(255,255,255,.4); margin-top: 1px; flex-shrink: 0; }
.wiz-ent-name { font-size: 14px; font-weight: 600; }
.wiz-ent-addr { font-size: 12px; color: rgba(255,255,255,.5); margin-top: 2px; }
.wiz-invite-msg {
    display: flex; align-items: flex-start; gap: 8px;
    font-size: 13px; color: rgba(255,255,255,.55); line-height: 1.6;
    font-style: italic;
}
.wiz-msg-ico { font-size: 16px; color: rgba(255,255,255,.25); flex-shrink: 0; margin-top: 2px; }
.wiz-code-pill {
    display: inline-flex; align-items: center; gap: 6px;
    background: rgba(255,255,255,.08); border: 1px solid rgba(255,255,255,.1);
    border-radius: 20px; padding: 5px 12px;
    font-size: 12px; font-weight: 700; font-family: monospace;
    letter-spacing: .05em; width: fit-content;
}

/* Form panel */
.wiz-fp {
    overflow-y: auto; display: flex; flex-direction: column;
    background: #f4f6f9;
    min-height: 0;
    min-width: 0;
}
.wiz-fi {
    flex: 1; max-width: 620px; width: 100%;
    margin: 0 auto; padding: 44px 28px 120px;
    box-sizing: border-box;
}

/* Footer */
.wiz-ftr {
    position: sticky; bottom: 0;
    background: #fff; border-top: 1px solid #e5e7eb;
    padding: 14px 28px;
}
.wiz-ftr-in {
    max-width: 620px; width: 100%; margin: 0 auto;
    display: flex; align-items: center; justify-content: space-between; gap: 12px;
}
.wiz-step-ind { font-size: 13px; color: #6b7280; }
.wiz-step-ind strong { color: #111827; }
.wiz-btn-g { display: flex; gap: 10px; }
.wiz-btn {
    display: inline-flex; align-items: center; gap: 7px;
    font-size: 14px; font-weight: 600;
    padding: 9px 18px; border-radius: 8px; border: none;
    cursor: pointer; transition: .15s; font-family: 'Inter', sans-serif;
}
.wiz-btn .ms { font-size: 18px; }
.wiz-btn-ghost {
    background: none; color: #6b7280;
    border: 1.5px solid #d1d5db;
}
.wiz-btn-ghost:hover { background: #f4f6f9; color: #111827; }
.wiz-btn-primary { background: #111827; color: #fff; }
.wiz-btn-primary:hover { background: #1f2937; }
.wiz-btn-primary:active,
.wiz-btn-success:active,
.wiz-btn-ghost:active {
    transform: scale(0.97);
    transition: transform 100ms ease-out;
}
.wiz-btn-success { background: #10b981; color: #fff; }
.wiz-btn-success:hover { background: #059669; }

/* Form primitives */
.wiz-card {
    background: #fff; border: 1px solid #e5e7eb;
    border-radius: 12px; padding: 22px;
    margin-bottom: 14px;
    box-shadow: 0 1px 3px rgba(0,0,0,.08), 0 1px 2px rgba(0,0,0,.05);
}
.wiz-ct {
    font-size: 11px; font-weight: 700; text-transform: uppercase;
    letter-spacing: .07em; color: #6b7280; margin-bottom: 18px;
}
.wiz-ct-hint {
    font-size: 10px; color: #9ca3af; text-transform: none;
    letter-spacing: 0; font-weight: 400; margin-left: 4px;
}
.wiz-f { margin-bottom: 16px; }
.wiz-f:last-child { margin-bottom: 0; }
.wiz-label {
    display: block; font-size: 11px; font-weight: 700;
    text-transform: uppercase; letter-spacing: .06em;
    color: #6b7280; margin-bottom: 5px;
}
.wiz-label-hint {
    font-size: 10px; color: #9ca3af; text-transform: none;
    letter-spacing: 0; font-weight: 400;
}
.wiz-inp {
    width: 100%; background: #f4f6f9;
    border: 1.5px solid #d1d5db; border-radius: 8px;
    padding: 10px 13px; font-size: 14px;
    font-family: 'Inter', sans-serif; color: #111827;
    outline: none; transition: .15s; box-sizing: border-box;
}
.wiz-inp:focus {
    border-color: #6366f1;
    box-shadow: 0 0 0 3px rgba(99,102,241,.1);
}
.wiz-inp::placeholder { color: #9ca3af; }
.wiz-inp--readonly {
    color: #374151;
    background: #eef1f5;
    border-color: #e5e7eb;
    cursor: default;
}
.wiz-inp--readonly:focus {
    border-color: #e5e7eb;
    box-shadow: none;
}
.wiz-label-row {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 8px;
}
.wiz-verified-pill {
    display: inline-flex;
    align-items: center;
    gap: 4px;
    font-size: 10px;
    font-weight: 700;
    text-transform: uppercase;
    letter-spacing: .06em;
    color: #059669;
    background: rgba(16,185,129,.1);
    border: 1px solid rgba(16,185,129,.35);
    border-radius: 20px;
    padding: 2px 8px;
}
select.wiz-inp {
    appearance: none;
    background-image: url("data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' width='12' height='12' viewBox='0 0 24 24'%3E%3Cpath fill='%236b7280' d='M7 10l5 5 5-5z'/%3E%3C/svg%3E");
    background-repeat: no-repeat;
    background-position: right 12px center;
    padding-right: 34px;
    cursor: pointer;
}
.wiz-inp-row { display: grid; grid-template-columns: 1fr 1fr; gap: 14px; }
.wiz-toggle {
    position: relative; width: 42px; height: 23px; flex-shrink: 0;
    background: #d1d5db; border-radius: 12px; border: none; cursor: pointer;
    transition: .2s; padding: 0;
}
.wiz-toggle.on { background: #6366f1; }
.wiz-toggle::after {
    content: ''; position: absolute; width: 17px; height: 17px; background: #fff;
    border-radius: 50%; top: 3px; left: 3px; box-shadow: 0 1px 3px rgba(0,0,0,.2);
    transition: .2s;
}
.wiz-toggle.on::after { transform: translateX(19px); }
.wiz-og { display: grid; gap: 10px; }
.wiz-og2 { grid-template-columns: 1fr 1fr; }
.wiz-og3 { grid-template-columns: 1fr 1fr 1fr; }
.wiz-oc {
    border: 1.5px solid #d1d5db; border-radius: 8px; padding: 13px 15px;
    cursor: pointer; transition: .15s; display: flex; flex-direction: column; gap: 7px;
    background: #fff; text-align: left; font-family: inherit; color: inherit;
}
.wiz-oc:hover { border-color: #6366f1; background: rgba(99,102,241,.03); }
.wiz-oc.sel { border-color: #6366f1; background: rgba(99,102,241,.06); }
.wiz-oc .ms { font-size: 22px; color: #6b7280; }
.wiz-oc.sel .ms { color: #6366f1; }
.wiz-oc-row { display: flex; align-items: center; gap: 8px; width: 100%; }
.wiz-oc-chk {
    width: 18px; height: 18px; border: 1.5px solid #d1d5db; border-radius: 50%;
    margin-left: auto; display: flex; align-items: center; justify-content: center;
    transition: .15s; flex-shrink: 0;
}
.wiz-oc.sel .wiz-oc-chk { background: #6366f1; border-color: #6366f1; }
.wiz-oc.sel .wiz-oc-chk::after {
    content: ''; width: 6px; height: 6px; background: #fff; border-radius: 50%;
}
.wiz-oc-label { font-size: 13px; font-weight: 600; }
.wiz-oc-desc { font-size: 12px; color: #6b7280; line-height: 1.4; }
.wiz-tr {
    display: flex; align-items: center; justify-content: space-between;
    padding: 13px 0; border-bottom: 1px solid #e5e7eb;
}
.wiz-tr:last-child { border-bottom: none; padding-bottom: 0; }
.wiz-tr:first-child { padding-top: 0; }
.wiz-tr-label { font-size: 14px; font-weight: 500; color: #111827; }
.wiz-tr-desc { font-size: 12px; color: #6b7280; margin-top: 1px; }

/* Avatar upload */
.wiz-av-up {
    display: flex; align-items: center; gap: 18px; padding: 18px;
    background: #f4f6f9; border: 1.5px dashed #d1d5db; border-radius: 12px;
    cursor: pointer; transition: .15s;
}
.wiz-av-up:hover { border-color: #6366f1; background: rgba(99,102,241,.03); }
.wiz-av-circle {
    width: 60px; height: 60px;
    background: linear-gradient(135deg,#6366f1,#4f46e5);
    border-radius: 50%; display: flex; align-items: center; justify-content: center;
    font-size: 22px; font-weight: 800; color: #fff; flex-shrink: 0;
}
.wiz-av-label { font-size: 14px; font-weight: 600; margin-bottom: 2px; color: #111827; }
.wiz-av-hint { font-size: 12px; color: #6b7280; }

/* Completion */
.wiz-done-card {
    background: linear-gradient(135deg,#0f1117 0%,#1a1b2e 100%);
    color: #fff; border-radius: 12px; padding: 36px 28px; text-align: center;
    position: relative; overflow: hidden; margin-bottom: 14px;
}
.wiz-done-card::before {
    content: ''; position: absolute; inset: 0;
    background: radial-gradient(ellipse at 50% 0%,rgba(99,102,241,.3) 0%,transparent 65%);
}
.wiz-done-inner { position: relative; z-index: 1; }
.wiz-done-ico {
    width: 68px; height: 68px;
    background: rgba(16,185,129,.12); border: 2px solid rgba(16,185,129,.35);
    border-radius: 50%; display: flex; align-items: center; justify-content: center;
    margin: 0 auto 18px;
}
.wiz-done-ico .ms { font-size: 32px; color: #10b981; }
.wiz-done-h { font-size: 22px; font-weight: 800; letter-spacing: -.02em; margin-bottom: 6px; }
.wiz-done-p { font-size: 13px; color: rgba(255,255,255,.55); line-height: 1.6; }
.wiz-stats {
    display: grid; grid-template-columns: repeat(3,1fr); gap: 10px;
    margin-top: 22px;
}
.wiz-stat {
    background: rgba(255,255,255,.06); border: 1px solid rgba(255,255,255,.1);
    border-radius: 10px; padding: 14px; text-align: center;
}
.wiz-stat-v { font-size: 20px; font-weight: 800; }
.wiz-stat-l { font-size: 11px; color: rgba(255,255,255,.45); margin-top: 2px; }

.wiz-na-row {
    display: flex; align-items: center; gap: 12px; padding: 14px 16px;
    border: 1.5px solid #d1d5db; border-radius: 8px; margin-bottom: 10px;
    cursor: pointer; transition: .15s; text-decoration: none; color: inherit;
}
.wiz-na-row:hover { border-color: #6366f1; background: rgba(99,102,241,.03); }
.wiz-na-row:last-child { margin-bottom: 0; }
.wiz-na-text { flex: 1; min-width: 0; }
.wiz-na-label { font-size: 14px; font-weight: 600; color: #111827; }
.wiz-na-desc { font-size: 12px; color: #6b7280; margin-top: 1px; }
.wiz-na-arrow { margin-left: auto; color: #9ca3af; flex-shrink: 0; }
.wiz-pay-option {
    display: flex; align-items: center; gap: 12px; padding: 14px;
    border: 1.5px solid #e5e7eb; border-radius: 8px;
}
.wiz-s-badge {
    display: inline-flex; align-items: center; gap: 6px;
    background: rgba(99,102,241,.08); color: #6366f1;
    font-size: 11px; font-weight: 700; text-transform: uppercase;
    letter-spacing: .08em; padding: 4px 10px; border-radius: 20px;
    margin-bottom: 14px;
}
.wiz-s-badge-done {
    background: rgba(16,185,129,.1); color: #059669;
}
.wiz-s-title {
    font-size: 26px; font-weight: 800;
    letter-spacing: -.03em; margin-bottom: 6px; color: #111827;
}
.wiz-s-sub {
    font-size: 14px; color: #6b7280;
    line-height: 1.6; margin-bottom: 32px;
}
.wiz-error-banner {
    display: flex; align-items: center; gap: 8px;
    background: #fef2f2; border: 1px solid #fecaca; color: #dc2626;
    border-radius: 8px; padding: 10px 14px; margin-bottom: 16px; font-size: 13px;
}

/* Animation */
@keyframes wiz-slide {
    from { opacity: 0; transform: translateY(8px); }
    to   { opacity: 1; transform: translateY(0); }
}
.wiz-anim { animation: wiz-slide .2s ease; }

@media (prefers-reduced-motion: reduce) {
    .wiz-anim {
        animation: none;
        transition: opacity 200ms ease;
    }
    .wiz-btn-primary:active,
    .wiz-btn-success:active,
    .wiz-btn-ghost:active,
    .pre-auth-cta:active {
        transform: none;
    }
}

/* Responsive */
@media (max-width: 900px) {
    .wiz-split {
        grid-template-columns: 1fr;
        height: auto; overflow: visible;
    }
    .wiz-ctx { display: none; }
    .wiz-fp { min-height: calc(100svh - 56px); }
    .wiz-fi { padding: 28px 18px 120px; max-width: 100%; }
    .wiz-ftr { padding: 12px 18px; }
    .wiz-og3 { grid-template-columns: 1fr 1fr; }
    .wiz-inp { font-size: 16px; }
}
@media (max-width: 520px) {
    .wiz-inp-row, .wiz-og2, .wiz-og3 { grid-template-columns: 1fr; }
    .wiz-s-title { font-size: 22px; }
    .wiz-stats { grid-template-columns: 1fr 1fr; }
}
"#;

// ── Pre-auth CSS ──────────────────────────────────────────────────────────────
// Lives in the same `.wiz-fi` column as wizard steps — no floating modal.

const PRE_AUTH_CSS: &str = r#"
.pre-auth-cta {
    width: 100%;
    justify-content: center;
    margin-top: 8px;
    padding: 12px 18px;
}
.pre-auth-cta:disabled {
    opacity: .6;
    cursor: default;
}
.pre-auth-code {
    font-size: 22px;
    font-weight: 700;
    letter-spacing: .2em;
    text-align: center;
    font-variant-numeric: tabular-nums;
}
.pre-auth-error {
    font-size: 13px;
    color: #dc2626;
    background: #fef2f2;
    border: 1px solid #fecaca;
    border-radius: 8px;
    padding: 8px 12px;
    margin-top: 4px;
}
.pre-auth-footer-note {
    margin-top: 18px;
    font-size: 13px;
    color: #6b7280;
}
.pre-auth-link-inline {
    color: #6366f1;
    font-weight: 600;
    text-decoration: none;
}
.pre-auth-link-inline:hover { text-decoration: underline; }
.pre-auth-link {
    display: block;
    width: 100%;
    margin-top: 12px;
    padding: 0;
    background: none;
    border: none;
    font-size: 13px;
    font-weight: 600;
    color: #6b7280;
    cursor: pointer;
    font-family: inherit;
    text-align: center;
}
.pre-auth-link:hover { color: #374151; }
"#;
