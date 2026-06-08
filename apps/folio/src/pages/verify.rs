use leptos::prelude::*;
use leptos_router::hooks::use_query_map;
use leptos_router::components::Redirect;

/// Handles ?token=... magic-link callbacks.
/// Verifies the token, then redirects to /dashboard on success.
#[component]
pub fn Verify() -> impl IntoView {
    let query   = use_query_map();
    let token   = move || query.read().get("token").cloned().unwrap_or_default();
    let result  = Resource::new(token, |t| crate::auth::verify_magic_link(t));

    view! {
        <div class="verify-page">
            <Suspense fallback=|| view! { <p class="verify-msg">"Verifying…"</p> }>
                {move || result.get().map(|r| match r {
                    Ok(_)  => view! { <Redirect path="/dashboard"/> }.into_any(),
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
