//! Read-only verified email row for persona onboarding wizards.
//! Reads [`crate::components::wizard_shell::WizardAuthCtx`] from WizardShell.

use leptos::prelude::*;

use crate::components::wizard_shell::WizardAuthCtx;

/// Prefill a local email signal from WizardShell auth context.
/// Must be rendered **inside** `<WizardShell>` children (context is provided there).
#[component]
pub fn SyncVerifiedEmail(email: RwSignal<String>) -> impl IntoView {
    let ctx = use_context::<WizardAuthCtx>();
    Effect::new(move |_| {
        if let Some(c) = ctx {
            if let Some(e) = c.email.get() {
                if !e.is_empty() {
                    email.set(e);
                }
            }
        }
    });
    ()
}

/// Email shown as locked + Verified once WizardShell has authenticated the user
/// (magic link, session peek, or OTP).
#[component]
pub fn VerifiedEmailField() -> impl IntoView {
    let ctx = use_context::<WizardAuthCtx>();

    view! {
        <div class="wiz-f">
            <label class="wiz-label wiz-label-row">
                <span>"Email"</span>
                <Show when=move || {
                    ctx.map(|c| c.email.get().filter(|e| !e.is_empty()).is_some())
                        .unwrap_or(false)
                }>
                    <span class="wiz-verified-pill">
                        <span class="ms msf" style="font-size:14px;">"verified"</span>
                        "Verified"
                    </span>
                </Show>
            </label>
            <input
                class="wiz-inp wiz-inp--readonly"
                type="email"
                readonly
                prop:value=move || {
                    ctx.and_then(|c| c.email.get())
                        .filter(|e| !e.is_empty())
                        .unwrap_or_else(|| "—".to_string())
                }
            />
        </div>
    }
}
