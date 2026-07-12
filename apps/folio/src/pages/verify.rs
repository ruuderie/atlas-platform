use leptos::prelude::*;
use leptos_router::components::Redirect;
use leptos_router::hooks::use_query_map;

/// Handles ?token=... magic-link callbacks.
///
/// Routing after a successful verify:
///   1. No passkey yet          → /auth/passkey-setup  (first-ever login)
///   2. Passkey set, onboarding incomplete → /onboarding
///   3. Fully set up            → /dashboard
#[component]
pub fn Verify() -> impl IntoView {
    let query = use_query_map();
    let token = move || {
        query
            .read()
            .get("token")
            .map(|s| s.clone())
            .unwrap_or_default()
    };
    let result: Resource<Result<crate::auth::SessionInfo, _>> =
        Resource::new(token, |t| crate::auth::verify_magic_link(t));

    // Same-tab handoff: stash verified email before redirect so WizardShell
    // can skip OTP even if the first /api/folio/me probe is 403.
    Effect::new(move |_| {
        if let Some(Ok(info)) = result.get() {
            crate::auth::stash_verified_email(&info.email);
        }
    });

    view! {
        <div class="verify-page">
            <Suspense fallback=|| view! { <p class="verify-msg">"Verifying…"</p> }>
                {move || result.get().map(|r| match r {
                    Ok(info) => {
                        crate::auth::stash_verified_email(&info.email);
                        let dest = if !info.has_passkey {
                            // First login — set up passkey before anything else.
                            "/auth/passkey-setup"
                        } else if !info.onboarding_complete {
                            // Passkey set but wizard not finished.
                            "/onboarding"
                        } else {
                            "/dashboard"
                        };
                        view! { <Redirect path=dest/> }.into_any()
                    },
                    Err(e) => view! {
                        <div class="verify-error">
                            <p>"Login link invalid or expired."</p>
                            <p class="error-detail">{e.to_string()}</p>
                            <a href="/login">"Try again"</a>
                        </div>
                    }.into_any(),
                })}
            </Suspense>
        </div>
    }
}
