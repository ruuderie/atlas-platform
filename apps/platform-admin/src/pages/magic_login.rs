use leptos::prelude::*;
use leptos_router::hooks::use_navigate;
use crate::api::models::UserInfo;

#[component]
pub fn MagicLogin() -> impl IntoView {
    let query = leptos_router::hooks::use_query_map();
    let token = move || query.with(|q| q.get("token").unwrap_or_default());

    let (error, set_error) = signal(None::<String>);
    
    let set_user = use_context::<WriteSignal<Option<UserInfo>>>().expect("set_user context");
    let navigate = use_navigate();
    
    // We clone these for async task
    let navigate_target = navigate.clone();
    let set_user_task = set_user.clone();

    Effect::new(move |_| {
        let t = token();
        if t.is_empty() {
            set_error.set(Some("Invalid or missing Magic Link token.".to_string()));
            return;
        }

        leptos::task::spawn_local(async move {
            let req_url = crate::api::client::api_url("/magic-links/verify");
            match crate::api::client::api_request::<crate::api::models::SessionResponse>(
                reqwest::Client::new().post(&req_url).json(&serde_json::json!({ "token": t }))
            ).await {
                Ok(res) => {
                    // Set token and save user
                    crate::api::client::set_auth_token(&res.token);
                    if let Some(user_info) = res.user.clone() {
                        set_user_task.set(Some(user_info.clone()));
                        
                        // Check if they need passkey setup
                        let email_encoded = urlencoding::encode(&user_info.email);
                        let flow_url = crate::api::client::api_url(&format!("/api/auth/flow/{}", email_encoded));
                        
                        if let Ok(flow_res) = crate::api::client::api_request::<serde_json::Value>(reqwest::Client::new().get(&flow_url)).await {
                            if let Some(true) = flow_res.get("has_passkey").and_then(|v| v.as_bool()) {
                                navigate_target("/", Default::default());
                            } else {
                                navigate_target("/setup", Default::default());
                            }
                        } else {
                            // Fallback to Dashboard if flow inspection fails
                            navigate_target("/", Default::default());
                        }
                    } else {
                        navigate_target("/", Default::default());
                    }
                }
                Err(e) => {
                    set_error.set(Some(format!("Magic link consumption failed: {}", e)));
                }
            }
        });
    });

    view! {
        <div class="relative flex items-center justify-center min-h-screen bg-surface font-sans overflow-hidden">
            <div class="absolute inset-0 opacity-50" style="background-image:url(\"data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' width='14' height='24'%3E%3Crect x='0' y='0' width='1' height='24' fill='%232b468020'/%3E%3Crect x='0' y='0' width='14' height='1' fill='%232b468020'/%3E%3C/svg%3E\");background-size:14px 24px;"></div>
            <div class="absolute top-0 left-1/2 -translate-x-1/2 -translate-y-1/2 w-[600px] h-[400px] bg-primary/20 rounded-full blur-[100px] pointer-events-none"></div>

            <div class="relative z-10 w-full max-w-md p-6">
                <div class="p-8 rounded-2xl bg-surface-container/30 border border-outline-variant/10 shadow-2xl backdrop-blur-xl text-center space-y-6">
                    {move || match error.get() {
                        Some(err) => view! {
                            <div class="inline-flex items-center justify-center w-16 h-16 rounded-full bg-error-container/20 border border-error/30 mb-2">
                                <span class="material-symbols-outlined text-3xl text-error">"error"</span>
                            </div>
                            <h2 class="text-xl font-bold text-on-surface">"Link Invalid"</h2>
                            <p class="text-sm text-error">{err}</p>
                            <a href="/login" class="text-xs text-primary underline hover:text-primary-variant transition-colors mt-6 block">"Return to Login"</a>
                        }.into_any(),
                        None => view! {
                            <span class="material-symbols-outlined text-5xl text-primary animate-pulse">"vpn_key"</span>
                            <h2 class="text-xl font-bold text-on-surface">"Authenticating..."</h2>
                            <p class="text-sm text-on-surface-variant">"Verifying your magic link cipher and establishing handshake."</p>
                        }.into_any(),
                    }}
                </div>
            </div>
        </div>
    }
}
