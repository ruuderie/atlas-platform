use leptos::prelude::*;
use crate::components::auth::passkey_manager::ManagePasskeys;

/// A dismissable nudge banner that prompts the user to register a passkey after
/// magic-link login. Universal across all Atlas apps.
///
/// This component replaces the old `PasskeyRegistrationNudge` in anchor/admin.rs
/// which used `js_sys::eval` + `setTimeout(500)` + manual DOM binding — a pattern
/// that races against WASM hydration and CDN script load timing, silently failing
/// in most cases. See the 2026-05-17 engineering brief for full root-cause analysis.
///
/// The action is delegated to `ManagePasskeys` which uses:
/// - `Action::new_local` (correct for client-only async in Leptos 0.8)
/// - `#[cfg(target_arch = "wasm32")]` to gate `fetch_credentials_include()` on the reqwest client
/// - `crate::auth::passkey::start_registration` Rust wrapper (no raw JS eval)
/// - Reactive signal state for error/success feedback
///
/// # Usage
/// ```rust
/// use shared_ui::components::auth::passkey_nudge::PasskeyNudge;
///
/// // Wrap in a <Show> gated on a passkey check signal:
/// <Show when=move || show_nudge.get()>
///     <PasskeyNudge />
/// </Show>
/// ```
#[component]
pub fn PasskeyNudge() -> impl IntoView {
    let (is_hidden, set_is_hidden) = signal(false);

    view! {
        <Show when=move || !is_hidden.get()>
            <div class="bg-primary/10 border border-primary/40 rounded-2xl p-6 mb-8 w-full">
                <div class="flex justify-between items-start mb-4">
                    <div>
                        <h3 class="text-primary font-bold text-base">
                            "Action Required: Set Up a Passkey"
                        </h3>
                        <p class="text-sm text-on-surface-variant mt-1">
                            "You signed in via email link. Register a passkey (Face ID, \
                             Touch ID, or a hardware key) for faster, passwordless sign-in \
                             on future visits."
                        </p>
                    </div>
                    <button
                        type="button"
                        class="text-outline hover:text-on-surface ml-4 shrink-0 transition-colors"
                        on:click=move |_| set_is_hidden.set(true)
                        title="Dismiss"
                    >
                        <span class="material-symbols-outlined text-xl">"close"</span>
                    </button>
                </div>
                // ManagePasskeys owns the full WebAuthn registration flow:
                // Action::new_local + fetch_credentials_include() + reactive error/success UI.
                // api_base_url is passed as a relative path so this component works on any
                // Atlas app regardless of domain. auth_token is vestigial — the backend reads
                // the session cookie, not an Authorization header.
                <ManagePasskeys
                    api_base_url=Signal::derive(|| "/api/passkeys".to_string())
                    auth_token="".to_string()
                />
            </div>
        </Show>
    }
}
