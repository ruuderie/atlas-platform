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
                match shared_ui::auth::atlas_auth::server_fns::verify_magic_link(active_token).await {
                    Ok(token_str) => {
                        crate::api::client::set_auth_token(&token_str);
                        let _ = shared_ui::auth::atlas_auth::set_session_cookie(token_str).await;
                        // The user is successfully authenticated and old passkeys are deleted.
                        // Force a refresh of the session globally.
                        match crate::api::auth::get_session().await {
                            Ok(user) => {
                                set_user.set(Some(user));
                                toast_clone.show_toast("Auth", "Token consumed! Please register a new Passkey immediately.", "success");
                                nav("/settings", Default::default());
                            }
                            Err(e) => {
                                error_msg.set(Some(format!("Handshake failed: {}. Try again.", e)));
                                is_loading.set(false);
                            }
                        }
                    }
                    Err(e) => {
                        let e_str = e.to_string();
                        if e_str.contains("token_already_used") {
                            error_msg.set(Some("This magic link has already been used.".to_string()));
                        } else if e_str.contains("token_expired") {
                            error_msg.set(Some("This magic link has expired.".to_string()));
                        } else if e_str.contains("token_not_found") {
                            error_msg.set(Some("This magic link is invalid or does not exist.".to_string()));
                        } else {
                            error_msg.set(Some(format!("Verification failed: {}", e_str)));
                        }
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
