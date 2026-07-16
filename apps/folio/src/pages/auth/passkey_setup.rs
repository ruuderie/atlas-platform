// apps/folio/src/pages/auth/passkey_setup.rs
//
// Passkey Setup — /auth/passkey-setup
//
// Shown after magic-link verify when the account has no passkey yet.
// Dark-harmonized with pub_login_v3 register guidance.

use leptos::prelude::*;
use leptos_router::hooks::use_navigate;

#[derive(Clone, Copy, PartialEq, Eq)]
enum PasskeySetupState {
    Idle,
    Working,
    Done,
}

async fn resolve_after_passkey_dest(continue_label: RwSignal<String>) -> &'static str {
    crate::auth::invalidate_session_cache();
    match crate::auth::get_session().await {
        Ok(info) => {
            let path = crate::auth::after_passkey_setup_path(&info);
            continue_label.set(if path == "/onboarding" {
                "Taking you to setup…".into()
            } else {
                "Taking you to your dashboard…".into()
            });
            path
        }
        // Soft-fail: keep historical behavior if /me is briefly unavailable.
        Err(_) => "/onboarding",
    }
}

#[component]
pub fn PasskeySetup() -> impl IntoView {
    let nav = StoredValue::new(use_navigate());
    let state = RwSignal::new(PasskeySetupState::Idle);
    let error_msg = RwSignal::new(Option::<String>::None);
    let continue_label = RwSignal::new("Taking you to setup…".to_string());

    let handle_skip = move |_| {
        leptos::task::spawn_local(async move {
            let dest = resolve_after_passkey_dest(continue_label).await;
            nav.with_value(|n| n(dest, Default::default()));
        });
    };

    let handle_setup = move |_| {
        if state.get() == PasskeySetupState::Working {
            return;
        }
        state.set(PasskeySetupState::Working);
        error_msg.set(None);

        leptos::task::spawn_local(async move {
            match crate::utils::passkey_js::create_passkey().await {
                Ok(_credential_json) => {
                    state.set(PasskeySetupState::Done);
                    let dest = resolve_after_passkey_dest(continue_label).await;
                    nav.with_value(|n| n(dest, Default::default()));
                }
                Err(e) => {
                    state.set(PasskeySetupState::Idle);
                    // Don't prefix with "Biometric cancelled" — most failures here are
                    // API/proxy errors before the browser prompt appears.
                    error_msg.set(Some(e));
                }
            }
        });
    };

    view! {
        <div class="login-layout login-layout--solo">
            <main class="login-auth-panel">
                <div class="login-auth-inner">
                    <div class="login-mobile-logo" style="display:flex">
                        <span class="login-logo-mark">"F"</span>
                        <span class="login-logo-text">"Folio"</span>
                    </div>

                    <Show when=move || state.get() != PasskeySetupState::Done>
                        <h1 class="login-auth-h1">"Register a passkey"</h1>
                        <p class="login-auth-sub">
                            {move || if state.get() == PasskeySetupState::Working {
                                "Approve with Face ID, Touch ID, or your device PIN when prompted."
                            } else {
                                "Your device will prompt you for Face ID, Touch ID, or PIN — no passwords needed, ever."
                            }}
                        </p>

                        <div class="login-feature-list">
                            <div class="login-feature-row">
                                <div class="login-feature-icon">
                                    <span class="material-symbols-outlined login-icon-fill" style="font-size:16px">"fingerprint"</span>
                                </div>
                                <div>
                                    <div class="login-feature-title">"Biometric or PIN"</div>
                                    <div class="login-feature-sub">"Authenticate with what your device already knows — no typing required."</div>
                                </div>
                            </div>
                            <div class="login-feature-row">
                                <div class="login-feature-icon">
                                    <span class="material-symbols-outlined login-icon-fill" style="font-size:16px">"shield_lock"</span>
                                </div>
                                <div>
                                    <div class="login-feature-title">"Phishing-resistant"</div>
                                    <div class="login-feature-sub">"Your private key never leaves this device. Not even Folio can see it."</div>
                                </div>
                            </div>
                            <div class="login-feature-row">
                                <div class="login-feature-icon">
                                    <span class="material-symbols-outlined login-icon-fill" style="font-size:16px">"sync"</span>
                                </div>
                                <div>
                                    <div class="login-feature-title">"Synced automatically"</div>
                                    <div class="login-feature-sub">"Stored in iCloud Keychain or Google Password Manager — ready on all your devices."</div>
                                </div>
                            </div>
                        </div>

                        <div class="login-device-preview">
                            <div class="login-dp-label">"Device prompt preview"</div>
                            <div class="login-dp-modal">
                                <div class="login-dp-modal-icon">"F"</div>
                                <div class="login-dp-modal-title">"Create a passkey for Folio"</div>
                                <div class="login-dp-modal-domain">"folio.app"</div>
                            </div>
                        </div>

                        <Show when=move || error_msg.get().is_some()>
                            <p class="login-field-error login-field-error--show" style="margin-bottom:1rem;text-align:left">
                                {move || error_msg.get().unwrap_or_default()}
                            </p>
                        </Show>

                        <button
                            type="button"
                            id="passkey-setup-btn"
                            class="login-auth-btn login-auth-btn--green"
                            disabled=move || state.get() == PasskeySetupState::Working
                            on:click=handle_setup
                        >
                            <span class="material-symbols-outlined login-icon-fill" style="font-size:18px">"fingerprint"</span>
                            {move || if state.get() == PasskeySetupState::Working {
                                "Waiting for device…"
                            } else {
                                "Create passkey"
                            }}
                        </button>

                        <Show when=move || state.get() == PasskeySetupState::Idle>
                            <button
                                type="button"
                                id="passkey-skip-btn"
                                class="login-btn-text"
                                on:click=handle_skip
                            >
                                "Maybe later"
                            </button>
                        </Show>
                    </Show>

                    <Show when=move || state.get() == PasskeySetupState::Done>
                        <div class="login-screen--center" style="display:flex;flex-direction:column;align-items:center;text-align:center">
                            <div class="login-status-circle login-status-circle--green">
                                <span class="material-symbols-outlined login-icon-fill" style="font-size:40px">"key"</span>
                            </div>
                            <h1 class="login-auth-h1">"Passkey saved"</h1>
                            <p class="login-auth-sub">{move || continue_label.get()}</p>
                        </div>
                    </Show>

                    <p class="login-legal">
                        "Your passkey never leaves your device. Folio never sees your biometric data."
                    </p>
                </div>
            </main>
        </div>
    }
}
