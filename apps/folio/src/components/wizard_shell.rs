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
use leptos_router::hooks::{use_location, use_navigate, use_query_map};
use serde::{Deserialize, Serialize};

// ── Pre-auth gate mode ────────────────────────────────────────────────────────

/// How WizardShell should treat an unauthenticated (or not-yet-probed) visitor.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OnboardAuthMode {
    /// Session / peek / stash already proved identity — skip OTP.
    Skip,
    /// Warm acquisition (`ref` or `code`) — show editable OTP gate.
    Otp,
    /// Cold hit with no session — send to login (magic link / passkey).
    RedirectLogin,
}

/// Decide OTP vs skip vs login redirect.
///
/// Warm = non-empty `ref` (F&F referral) or `code` (NetworkInvite).
pub fn onboard_auth_mode(
    has_session_email: bool,
    ref_q: Option<&str>,
    code_q: Option<&str>,
) -> OnboardAuthMode {
    if has_session_email {
        return OnboardAuthMode::Skip;
    }
    let warm = ref_q.is_some_and(|s| !s.trim().is_empty())
        || code_q.is_some_and(|s| !s.trim().is_empty());
    if warm {
        OnboardAuthMode::Otp
    } else {
        OnboardAuthMode::RedirectLogin
    }
}

/// Build `/login?next=…` for cold onboard redirects.
pub fn login_next_path(current_path_with_query: &str) -> String {
    format!("/login?next={}", encode_next_param(current_path_with_query))
}

fn encode_next_param(s: &str) -> String {
    s.chars()
        .map(|c| match c {
            'A'..='Z' | 'a'..='z' | '0'..='9' | '-' | '_' | '.' | '~' => c.to_string(),
            _ => format!("%{:02X}", c as u32),
        })
        .collect()
}

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
    // Warm (`ref` / `code`): editable OTP when no session.
    // Cold: redirect to /login?next=… (magic link / passkey), never open OTP signup.
    // Magic-link returnees: session/peek/stash → skip OTP → VerifiedEmailField.

    let pre_auth_done: RwSignal<bool> = RwSignal::new(false);
    let verified_email: RwSignal<Option<String>> = RwSignal::new(None);
    provide_context(WizardAuthCtx {
        email: verified_email.into(),
    });

    let location = use_location();
    let query = use_query_map();
    let is_warm = Memo::new(move |_| {
        query.with(|q| {
            let ref_q = q.get("ref");
            let code_q = q.get("code");
            matches!(
                onboard_auth_mode(false, ref_q.as_deref(), code_q.as_deref()),
                OnboardAuthMode::Otp
            )
        })
    });
    let login_redirect = Memo::new(move |_| {
        let path = location.pathname.get();
        let search = location.search.get();
        let current = if search.is_empty() {
            path
        } else if search.starts_with('?') {
            format!("{path}{search}")
        } else {
            format!("{path}?{search}")
        };
        login_next_path(&current)
    });

    // Local OTP flow state
    let otp_email: RwSignal<String> = RwSignal::new(String::new());
    let otp_code: RwSignal<String> = RwSignal::new(String::new());
    let otp_sent: RwSignal<bool> = RwSignal::new(false);
    let otp_sending: RwSignal<bool> = RwSignal::new(false);
    let otp_verifying: RwSignal<bool> = RwSignal::new(false);
    let otp_error: RwSignal<Option<String>> = RwSignal::new(None);

    // Client-only session probe. Prefer peek (no Folio RBAC). Fresh magic-link
    // users get 403/404 from /api/folio/me, which Leptos surfaces as a noisy
    // check_session 500 — never fall back to it during onboarding pre-auth.
    // Cookie handoff from Axum /verify covers the same-tab case when peek is slow.
    let session_email_sig = session_email;
    let session_probe = LocalResource::new(|| async {
        let stash = crate::auth::read_stashed_verified_email();
        let peek = match crate::auth::peek_auth_session().await {
            Ok(peek) => Some(peek.email),
            Err(_) => match crate::auth::peek_auth_session().await {
                Ok(peek) => Some(peek.email),
                Err(_) => None,
            },
        };
        crate::auth::resolve_verified_email_probe(None, peek, stash)
    });
    Effect::new(move |_| {
        if let Some(Some(email)) = session_probe.get() {
            accept_verified_email(email, verified_email, session_email_sig, pre_auth_done);
        }
    });

    // Cold path: after the client probe settles with no email, navigate to login.
    // Do NOT put <Redirect/> inside the Show fallback — that swaps the view tree
    // during/after hydration and panics tachys (unreachable in hydration.rs).
    let navigate = use_navigate();
    Effect::new(move |_| {
        if pre_auth_done.get() || is_warm.get() {
            return;
        }
        // LocalResource resolved to None (no session/peek/stash).
        if matches!(session_probe.get(), Some(None)) {
            navigate(&login_redirect.get(), Default::default());
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

    // Styles live in style/wizard_shell.css (bundled via folio-v1.css).
    // Do NOT inject <style> / leptos_meta::Style here — a Style sibling of the
    // shell leaves a body `<!--<() />-->` marker that hydrate never consumes,
    // so the walker hits .wiz-logo while expecting nav title text.
    let nav_verify = format!("{persona_pill} Setup · Verify email");
    let nav_detail_full = nav_detail.map(|d| format!("{persona_pill} Setup · {d}"));
    let nav_title = Memo::new(move |_| {
        if !pre_auth_done.get() {
            return nav_verify.clone();
        }
        if let Some(detail) = nav_detail_full.clone() {
            return detail;
        }
        format!(
            "{persona_pill} Setup · Step {} of {}",
            current_idx.get() + 1,
            total
        )
    });

    view! {
        <div class="wiz-shell">
        // ── Topnav ────────────────────────────────────────────────────────────
        <header class="wiz-nav">
            <div class="wiz-logo">
                <div class="wiz-logo-mark">
                    <span class="ms msf" style="font-size:16px; color:#fff;">"apartment"</span>
                </div>
                <span class="wiz-logo-name">"Folio"</span>
            </div>
            <div class="wiz-nav-center">
                <span>{move || nav_title.get()}</span>
            </div>
            <a href="/dashboard" class="wiz-exit">
                <span class="ms">"close"</span>
                <span>"Exit"</span>
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
                        // Branch only on URL warmth (identical on SSR + first client paint).
                        // Never branch on LocalResource here — that desyncs hydration and
                        // panics tachys ("entered unreachable code" in hydration.rs).
                        if is_warm.get() {
                            // Warm acquisition — editable OTP gate (probe may still upgrade).
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
                            }.into_any()
                        } else {
                            // Cold: stable placeholder until Effect navigates or session probe skips OTP.
                            // Keep this DOM minimal and SSR-identical to avoid tachys hydration panics.
                            view! {
                                <div class="wiz-fi wiz-anim">
                                    <p class="wiz-s-sub">"Checking your session..."</p>
                                    <p class="wiz-s-sub" style="margin-top:8px;font-size:13px;color:#9ca3af;">
                                        "If you are signed in, your setup will continue automatically."
                                    </p>
                                    <p class="pre-auth-footer-note" style="margin-top:24px;">
                                        <a href="/login" class="pre-auth-link-inline">
                                            "Sign in"
                                        </a>
                                    </p>
                                </div>
                            }.into_any()
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
        </div>
    }
}

// Wizard + pre-auth CSS: style/wizard_shell.css (imported from style/main.css).
// Kept out of this component so SSR/hydrate never see a <style>/meta Style node.

#[cfg(test)]
mod auth_mode_tests {
    use super::{login_next_path, onboard_auth_mode, OnboardAuthMode};

    #[test]
    fn session_email_skips_otp() {
        assert_eq!(
            onboard_auth_mode(true, Some("alice"), Some("CODE")),
            OnboardAuthMode::Skip
        );
        assert_eq!(
            onboard_auth_mode(true, None, None),
            OnboardAuthMode::Skip
        );
    }

    #[test]
    fn warm_ref_shows_otp() {
        assert_eq!(
            onboard_auth_mode(false, Some("alice"), None),
            OnboardAuthMode::Otp
        );
    }

    #[test]
    fn warm_code_shows_otp() {
        assert_eq!(
            onboard_auth_mode(false, None, Some("OAK4B")),
            OnboardAuthMode::Otp
        );
    }

    #[test]
    fn blank_warm_params_are_cold() {
        assert_eq!(
            onboard_auth_mode(false, Some("  "), Some("")),
            OnboardAuthMode::RedirectLogin
        );
    }

    #[test]
    fn cold_redirects_to_login() {
        assert_eq!(
            onboard_auth_mode(false, None, None),
            OnboardAuthMode::RedirectLogin
        );
    }

    #[test]
    fn login_next_path_encodes_query() {
        let path = login_next_path("/onboarding?ref=alice");
        assert!(path.starts_with("/login?next="));
        assert!(path.contains("onboarding"));
        assert!(path.contains("ref"));
    }
}
