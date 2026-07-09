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
    pub code:            String,
    pub role:            String,
    pub label:           Option<String>,
    pub invite_message:  Option<String>,
    pub context:         InviteCodeContext,
    pub expires_at:      Option<String>,
    pub uses_remaining:  Option<i32>,
    pub is_valid:        bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct InviteCodeContext {
    pub asset:       Option<ContextEntity>,
    pub landlord:    Option<ContextEntity>,
    pub broker:      Option<ContextEntity>,
    pub asset_count: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct ContextEntity {
    pub name:    String,
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
    crate::atlas_client::fetch::<ResolvedInviteCode>(
        &format!("/api/folio/invite/resolve/{}", code.trim()),
    )
    .await
    .map(Some)
    .or_else(|_| Ok(None))   // 404/410 → treat as no code
}

// ── Step descriptor ───────────────────────────────────────────────────────────

/// A single step in a wizard flow.
#[derive(Clone, PartialEq)]
pub struct WizardStepDesc {
    pub id:         &'static str,
    pub label:      &'static str,
    pub skippable:  bool,
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
/// - `ctx_headline` — context panel headline
/// - `ctx_body`    — context panel body text / feature list (any view)
/// - `invite_code`  — resolved invite code signal (or None)
/// - `on_next` / `on_prev` — navigation callbacks
/// - `is_last_step` — if true, Next becomes primary CTA
/// - `step_content` — the form content for the current step (Children)
#[component]
pub fn WizardShell(
    steps:         Vec<WizardStepDesc>,
    current_idx:   RwSignal<usize>,
    persona_pill:  &'static str,
    persona_icon:  &'static str,
    accent_color:  &'static str,
    panel_bg:      &'static str,
    ctx_headline:  &'static str,
    #[prop(into)] ctx_body: ViewFn,
    #[prop(optional)] invite_code: Option<RwSignal<Option<ResolvedInviteCode>>>,
    on_next:       Callback<()>,
    on_prev:       Callback<()>,
    is_last_step:  Signal<bool>,
    next_label:    Signal<&'static str>,
    children:      Children,
) -> impl IntoView {
    let total = steps.len();
    let steps_store = StoredValue::new(steps);

    let panel_style = format!(
        "background:{panel_bg}; color:#fff; width:360px; min-width:280px; \
         max-width:360px; overflow-y:auto; position:relative; display:flex; \
         flex-direction:column; padding:36px 28px; flex-shrink:0;",
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
                {persona_pill}" Setup \u{b7} "
                <strong>
                    {move || {
                        let idx = current_idx.get();
                        format!("Step {} of {}", idx + 1, total)
                    }}
                </strong>
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
                    "background: radial-gradient(ellipse at 80% 0%, {}44 0%, transparent 55%), \
                                 radial-gradient(ellipse at 10% 90%, rgba(16,185,129,.18) 0%, transparent 50%);",
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
                    <div class="wiz-nav-label">"Setup progress"</div>
                    <div class="wiz-ctx-steps">
                        {steps_store.with_value(|steps| {
                            steps.iter().enumerate().map(|(i, step)| {
                                let label = step.label;
                                let is_done    = move || current_idx.get() > i;
                                let is_active  = move || current_idx.get() == i;
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
                <div class="wiz-fi">
                    {children()}
                </div>
                // ── Sticky footer ─────────────────────────────────────────────
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
                                {move || next_label.get()}
                                <span class="ms">"arrow_forward"</span>
                            </button>
                        </div>
                    </div>
                </footer>
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

body { font-family: 'Inter', sans-serif; margin: 0; }

/* Topnav */
.wiz-nav {
    background: #fff; border-bottom: 1px solid #e2e8f0;
    height: 56px; padding: 0 24px;
    display: flex; align-items: center; justify-content: space-between;
    position: sticky; top: 0; z-index: 100; flex-shrink: 0;
}
.wiz-logo { display: flex; align-items: center; gap: 10px; }
.wiz-logo-mark {
    width: 32px; height: 32px; background: #0f172a;
    border-radius: 8px; display: flex; align-items: center; justify-content: center;
}
.wiz-logo-name { font-size: 15px; font-weight: 700; letter-spacing: -.02em; color: #0f172a; }
.wiz-nav-center { font-size: 13px; color: #64748b; }
.wiz-nav-center strong { color: #0f172a; }
.wiz-exit {
    display: flex; align-items: center; gap: 6px;
    font-size: 13px; color: #64748b; text-decoration: none;
    padding: 6px 10px; border-radius: 8px; transition: .15s;
}
.wiz-exit:hover { background: #f4f6fb; color: #0f172a; }

/* Split */
.wiz-split {
    display: grid; grid-template-columns: 360px 1fr;
    height: calc(100vh - 56px); overflow: hidden;
}

/* Context panel */
.wiz-ctx { position: relative; }
.wiz-ctx-glow {
    position: absolute; inset: 0; pointer-events: none;
}
.wiz-ctx-inner {
    position: relative; z-index: 1;
    display: flex; flex-direction: column; height: 100%;
}
.wiz-ctx-h {
    font-size: 20px; font-weight: 800;
    letter-spacing: -.02em; margin-bottom: 8px;
}
.wiz-ctx-div { height: 1px; background: rgba(255,255,255,.08); margin: 18px 0; }
.wiz-nav-label {
    font-size: 11px; font-weight: 700; text-transform: uppercase;
    letter-spacing: .1em; color: rgba(255,255,255,.3); margin-bottom: 14px;
}
.wiz-ctx-steps { display: flex; flex-direction: column; gap: 7px; }
.wiz-ctx-si {
    display: flex; align-items: center; gap: 11px;
    font-size: 13px; color: rgba(255,255,255,.3);
}
.wiz-ctx-si.done  { color: rgba(255,255,255,.6); }
.wiz-ctx-si.active { color: #fff; font-weight: 600; }
.wiz-ctx-num {
    width: 22px; height: 22px; border-radius: 50%;
    border: 1.5px solid rgba(255,255,255,.2);
    display: flex; align-items: center; justify-content: center;
    font-size: 11px; font-weight: 700; flex-shrink: 0;
}
.wiz-ctx-si.done .wiz-ctx-num {
    background: #10b981; border-color: #10b981;
}
.wiz-ctx-si.active .wiz-ctx-num {
    background: #fff; border-color: #fff; color: #0f172a;
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
    background: #f4f6fb;
}
.wiz-fi {
    flex: 1; max-width: 640px; width: 100%;
    margin: 0 auto; padding: 44px 28px 120px;
}

/* Footer */
.wiz-ftr {
    position: sticky; bottom: 0;
    background: #fff; border-top: 1px solid #e2e8f0;
    padding: 14px 28px;
}
.wiz-ftr-in {
    max-width: 640px; width: 100%; margin: 0 auto;
    display: flex; align-items: center; justify-content: space-between; gap: 12px;
}
.wiz-step-ind { font-size: 13px; color: #64748b; }
.wiz-step-ind strong { color: #0f172a; }
.wiz-btn-g { display: flex; gap: 10px; }
.wiz-btn {
    display: inline-flex; align-items: center; gap: 7px;
    font-size: 14px; font-weight: 600;
    padding: 9px 18px; border-radius: 8px; border: none;
    cursor: pointer; transition: .15s; font-family: 'Inter', sans-serif;
}
.wiz-btn .ms { font-size: 18px; }
.wiz-btn-ghost {
    background: none; color: #64748b;
    border: 1.5px solid #cbd5e1;
}
.wiz-btn-ghost:hover { background: #f4f6fb; color: #0f172a; }
.wiz-btn-primary { background: #0f172a; color: #fff; }
.wiz-btn-primary:hover { background: #1e293b; }
.wiz-btn-success { background: #10b981; color: #fff; }
.wiz-btn-success:hover { background: #059669; }

/* Form primitives */
.wiz-card {
    background: #fff; border: 1px solid #e2e8f0;
    border-radius: 12px; padding: 22px;
    margin-bottom: 14px; box-shadow: 0 1px 3px rgba(0,0,0,.07);
}
.wiz-ct {
    font-size: 11px; font-weight: 700; text-transform: uppercase;
    letter-spacing: .07em; color: #64748b; margin-bottom: 18px;
}
.wiz-f { margin-bottom: 16px; }
.wiz-f:last-child { margin-bottom: 0; }
.wiz-label {
    display: block; font-size: 11px; font-weight: 700;
    text-transform: uppercase; letter-spacing: .06em;
    color: #64748b; margin-bottom: 5px;
}
.wiz-inp {
    width: 100%; background: #f4f6fb;
    border: 1.5px solid #cbd5e1; border-radius: 8px;
    padding: 10px 13px; font-size: 14px;
    font-family: 'Inter', sans-serif; color: #0f172a;
    outline: none; transition: .15s; box-sizing: border-box;
}
.wiz-inp:focus {
    border-color: #0284c7;
    box-shadow: 0 0 0 3px rgba(2,132,199,.1);
}
.wiz-inp::placeholder { color: #94a3b8; }
.wiz-inp-row { display: grid; grid-template-columns: 1fr 1fr; gap: 14px; }
.wiz-s-badge {
    display: inline-flex; align-items: center; gap: 6px;
    font-size: 11px; font-weight: 700; text-transform: uppercase;
    letter-spacing: .08em; padding: 4px 10px; border-radius: 20px;
    margin-bottom: 14px;
}
.wiz-s-title {
    font-size: 26px; font-weight: 800;
    letter-spacing: -.03em; margin-bottom: 6px; color: #0f172a;
}
.wiz-s-sub {
    font-size: 14px; color: #64748b;
    line-height: 1.6; margin-bottom: 32px;
}

/* Animation */
@keyframes wiz-slide {
    from { opacity: 0; transform: translateY(8px); }
    to   { opacity: 1; transform: translateY(0); }
}
.wiz-anim { animation: wiz-slide .2s ease; }

/* Responsive */
@media (max-width: 900px) {
    .wiz-split {
        grid-template-columns: 1fr;
        height: auto; overflow: visible;
    }
    .wiz-ctx { display: none; }
    .wiz-fi { padding: 28px 18px 120px; max-width: 100%; }
    .wiz-ftr { padding: 12px 18px; }
}
@media (max-width: 520px) {
    .wiz-inp-row { grid-template-columns: 1fr; }
    .wiz-s-title { font-size: 22px; }
}
"#;
