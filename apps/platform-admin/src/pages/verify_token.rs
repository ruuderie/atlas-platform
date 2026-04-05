use leptos::prelude::*;
use leptos_router::hooks::{use_navigate, use_params_map};

#[component]
pub fn VerifyToken() -> impl IntoView {
    let params = use_params_map();
    let toast = use_context::<crate::app::GlobalToast>().expect("toast context");
    let set_user = use_context::<WriteSignal<Option<crate::api::models::UserInfo>>>().expect("set user context");
    let navigate = use_navigate();
    let is_loading = RwSignal::new(true);
    let error_msg = RwSignal::new(None::<String>);

    Effect::new(move |_| {
        if let Some(token) = params.read().get("token") {
            let active_token = token.clone();
            let nav = navigate.clone();
            let toast_clone = toast.clone();
            leptos::task::spawn_local(async move {
                let verify_url = crate::api::client::api_url("/api/auth/magic-link/verify");
                match crate::api::client::api_request::<serde_json::Value>(
                    reqwest::Client::new().post(&verify_url).json(&serde_json::json!({ "token": active_token }))
                ).await {
                    Ok(res) => {
                        // The user is successfully authenticated and old passkeys are deleted.
                        // Force a refresh of the session globally.
                        if let Ok(user) = crate::api::auth::validate_session().await {
                            set_user.set(Some(user));
                            toast_clone.message.set(Some("Token consumed! Please register a new Passkey immediately.".to_string()));
                            nav("/settings", Default::default());
                        } else {
                            error_msg.set(Some("Handshake failed. Try again.".to_string()));
                            is_loading.set(false);
                        }
                    }
                    Err(_) => {
                        error_msg.set(Some("Invalid, expired, or previously consumed token.".to_string()));
                        is_loading.set(false);
                    }
                }
            });
        }
    });

    view! {
        <div class="min-h-screen bg-surface flex flex-col items-center justify-center p-6">
            <div class="w-full max-w-md p-8 rounded-2xl bg-surface-container/30 border border-outline-variant/10 shadow-2xl backdrop-blur-xl text-center">
                {move || if is_loading.get() {
                    view! {
                        <span class="material-symbols-outlined text-4xl text-primary animate-spin mb-4">"sync"</span>
                        <h2 class="text-xl font-bold text-on-surface">"Verifying Setup Token..."</h2>
                    }.into_any()
                } else {
                    view! {
                        <span class="material-symbols-outlined text-4xl text-error mb-4">"error"</span>
                        <h2 class="text-xl font-bold text-error">"Verification Failed"</h2>
                        <p class="text-on-surface-variant mt-2">{error_msg.get()}</p>
                        <a href="/login" class="mt-6 inline-block text-sm text-primary underline">"Return to Login Flow"</a>
                    }.into_any()
                }}
            </div>
        </div>
    }
}
