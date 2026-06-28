// apps/folio/src/pages/auth/passkey_setup.rs
//
// Passkey Setup — /auth/passkey-setup
//
// Shown immediately after a magic link exchange (first login).
// Clean, focused, no nav chrome. Guides the user through WebAuthn registration
// and then redirects to the first-run onboarding wizard.

use leptos::prelude::*;
use leptos_router::hooks::use_navigate;

#[derive(Clone, PartialEq)]
enum PasskeyState {
    Idle,
    Working,
    Done,
}

#[component]
pub fn PasskeySetup() -> impl IntoView {
    let nav       = StoredValue::new(use_navigate());
    let state     = RwSignal::new(PasskeyState::Idle);
    let error_msg = RwSignal::new(Option::<String>::None);

    let handle_skip = move |_| {
        nav.with_value(|n| n("/onboarding", Default::default()));
    };

    let handle_setup = move |_| {
        let state_s = state;
        let error_s = error_msg;
        state_s.set(PasskeyState::Working);
        error_s.set(None);

        leptos::task::spawn_local(async move {
            // Attempt WebAuthn credential creation via JS interop
            match crate::utils::passkey_js::create_passkey().await {
                Ok(_credential_json) => {
                    state_s.set(PasskeyState::Done);
                    nav.with_value(|n| n("/onboarding", Default::default()));
                }
                Err(e) => {
                    state_s.set(PasskeyState::Idle);
                    error_s.set(Some(format!("Biometric prompt cancelled or failed: {e}")));
                }
            }
        });
    };

    view! {
        <div
            id="passkey-setup-page"
            style="min-height:100vh;display:flex;align-items:center;justify-content:center;\
                   background:linear-gradient(135deg,#0f1117 0%,#1a1033 50%,#0f1117 100%);padding:24px;"
        >
            <div
                style="background:rgba(255,255,255,.03);border:1px solid rgba(255,255,255,.08);\
                       border-radius:24px;padding:48px;max-width:480px;width:100%;\
                       box-shadow:0 24px 80px rgba(0,0,0,.6);backdrop-filter:blur(12px);\
                       text-align:center;position:relative;"
            >
                // Atlas wordmark
                <div style="margin-bottom:32px;">
                    <span style="font-size:13px;font-weight:600;letter-spacing:.15em;\
                                 text-transform:uppercase;color:rgba(165,180,252,.7);">
                        "Atlas Platform"
                    </span>
                </div>

                // Icon — changes based on state
                <div style="font-size:56px;margin-bottom:24px;line-height:1;">
                    {move || match state.get() {
                        PasskeyState::Done    => "\u{2705}",
                        PasskeyState::Working => "\u{1F510}",
                        PasskeyState::Idle    => "\u{1F511}",
                    }}
                </div>

                <h1 style="font-size:24px;font-weight:700;color:#f1f5f9;margin:0 0 12px;">
                    {move || match state.get() {
                        PasskeyState::Done    => "You're all set!",
                        PasskeyState::Working => "Waiting for biometric\u{2026}",
                        PasskeyState::Idle    => "Secure your account",
                    }}
                </h1>

                <p style="font-size:15px;color:#94a3b8;line-height:1.65;margin:0 0 32px;">
                    {move || match state.get() {
                        PasskeyState::Done    => "Passkey registered. Taking you to setup\u{2026}".to_string(),
                        PasskeyState::Working => "Approve with your device biometric when prompted.".to_string(),
                        PasskeyState::Idle    => "Create a passkey using Touch ID, Face ID, or Windows Hello. No password needed.".to_string(),
                    }}
                </p>

                // What is a passkey? chips — only shown at idle
                <Show when=move || state.get() == PasskeyState::Idle>
                    <div style="display:flex;flex-wrap:wrap;gap:8px;justify-content:center;margin-bottom:32px;">
                        {[
                            ("\u{1F6E1}\u{FE0F}", "Phishing-proof"),
                            ("\u{26A1}", "One-tap login"),
                            ("\u{1F4F5}", "No passwords"),
                            ("\u{1F512}", "Device-bound"),
                        ].map(|(icon, label)| view! {
                            <div style="display:flex;align-items:center;gap:6px;\
                                        background:rgba(255,255,255,.05);border:1px solid rgba(255,255,255,.08);\
                                        border-radius:20px;padding:6px 12px;font-size:12px;color:#94a3b8;">
                                <span>{icon}</span>
                                <span>{label}</span>
                            </div>
                        }).collect_view()}
                    </div>
                </Show>

                // Error message
                <Show when=move || error_msg.get().is_some()>
                    <div style="background:rgba(239,68,68,.1);border:1px solid rgba(239,68,68,.3);\
                                border-radius:10px;padding:12px 16px;margin-bottom:20px;\
                                font-size:13px;color:#fca5a5;text-align:left;">
                        {move || error_msg.get().unwrap_or_default()}
                    </div>
                </Show>

                // CTA button
                <Show when=move || state.get() != PasskeyState::Done>
                    <button
                        id="passkey-setup-btn"
                        disabled=move || state.get() == PasskeyState::Working
                        on:click=handle_setup.clone()
                        style="width:100%;padding:16px;border-radius:12px;\
                               background:linear-gradient(135deg,#4f46e5,#7c3aed);\
                               color:#fff;font-size:16px;font-weight:600;\
                               border:none;cursor:pointer;\
                               box-shadow:0 8px 24px rgba(99,102,241,.35);\
                               transition:opacity .2s;margin-bottom:16px;"
                    >
                        {move || if state.get() == PasskeyState::Working {
                            "Setting up\u{2026}"
                        } else {
                            "Set up my passkey \u{2192}"
                        }}
                    </button>
                </Show>

                // Skip link
                <Show when=move || state.get() == PasskeyState::Idle>
                    <button
                        id="passkey-skip-btn"
                        on:click=handle_skip.clone()
                        style="background:none;border:none;color:#64748b;font-size:13px;\
                               cursor:pointer;text-decoration:underline;padding:4px;"
                    >
                        "Skip for now \u{2014} I'll set this up later"
                    </button>
                </Show>

                <p style="font-size:11px;color:#475569;margin-top:24px;line-height:1.5;">
                    "Your passkey never leaves your device. Atlas Platform never sees your biometric data."
                </p>
            </div>
        </div>
    }
}
