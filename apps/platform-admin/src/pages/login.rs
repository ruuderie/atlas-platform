use leptos::prelude::*;
use leptos_router::hooks::use_navigate;
use shared_ui::components::ui::button::Button;
use shared_ui::components::ui::input::{Input, InputType};
use crate::api::models::UserInfo;
use crate::api::setup::{get_setup_status, purge_admin};
use shared_ui::components::auth::passkey_login::PasskeyLoginButton;

#[derive(Clone, PartialEq)]
enum LoginState {
    EmailEntry,
    PasskeyPrompt,
    SendingToken,
    SetupTokenSent,
}

#[component]
pub fn Login() -> impl IntoView {
    let email = RwSignal::new("".to_string());
    let login_state = RwSignal::new(LoginState::EmailEntry);
    let error_message = RwSignal::new(None::<String>);
    let is_loading = RwSignal::new(false);
    let set_user = use_context::<WriteSignal<Option<UserInfo>>>().expect("set_user context");
    let toast = use_context::<crate::app::GlobalToast>().expect("toast context");

    let navigate = use_navigate();
    let navigate_demo = navigate.clone();
    let navigate_pk = navigate.clone();
    let navigate_setup = navigate.clone();

    // Check if system needs setup
    leptos::task::spawn_local(async move {
        if let Ok(status) = get_setup_status().await {
            if status.needs_setup {
                navigate_setup("/setup", Default::default());
            }
        }
    });

    let handle_check_flow = Callback::new(move |_| {
        error_message.set(None);
        is_loading.set(true);
        let current_email = email.get();

        leptos::task::spawn_local(async move {
            let flow_url = format!("{}/api/auth/flow/{}", crate::api::client::api_url(""), urlencoding::encode(&current_email));
            match crate::api::client::api_request::<serde_json::Value>(reqwest::Client::new().get(&flow_url)).await {
                Ok(res) => {
                    if let Some(true) = res.get("has_passkey").and_then(|v| v.as_bool()) {
                        login_state.set(LoginState::PasskeyPrompt);
                    } else {
                        // Force a setup token bypass
                        login_state.set(LoginState::SendingToken);
                    }
                }
                Err(_) => {
                    error_message.set(Some("User lookup failed. Try again.".to_string()));
                }
            }
            is_loading.set(false);
        });
    });

    let handle_send_recovery = Callback::new(move |_| {
        error_message.set(None);
        is_loading.set(true);
        login_state.set(LoginState::SendingToken);
    });

    Effect::new(move |_| {
        if login_state.get() == LoginState::SendingToken {
            let current_email = email.get();
            leptos::task::spawn_local(async move {
                let req_url = crate::api::client::api_url("/api/auth/magic-link/request");
                if let Err(_) = crate::api::client::api_request::<serde_json::Value>(
                    reqwest::Client::new().post(&req_url).json(&serde_json::json!({ "email": current_email }))
                ).await {
                    error_message.set(Some("Failed to dispatch Setup Token. Check infrastructure routing.".to_string()));
                    login_state.set(LoginState::EmailEntry);
                } else {
                    login_state.set(LoginState::SetupTokenSent);
                }
                is_loading.set(false);
            });
        }
    });

    let set_user_pk = set_user.clone();
    let toast_pk = toast.clone();
    
    let handle_passkey_success = Callback::new(move |_token: String| {
        let set_user = set_user_pk.clone();
        let navigate = navigate_pk.clone();
        let toast = toast_pk.clone();
        leptos::task::spawn_local(async move {
            crate::api::client::set_auth_token(&_token);
            if let Ok(user) = crate::api::auth::validate_session().await {
                set_user.set(Some(user));
                navigate("/", Default::default());
            } else {
                toast.message.set(Some("Validated passkey, but session handshake failed.".to_string()));
            }
        });
    });

    let handle_passkey_error = Callback::new(move |err: String| {
        error_message.set(Some(err));
    });

    let handle_demo = Callback::new(move |_| {
        crate::api::client::set_demo_mode(true);
        let navigate = navigate_demo.clone();
        set_user.set(Some(UserInfo {
            id: "demo-user-1".to_string(),
            first_name: "Demo".to_string(),
            last_name: "Admin".to_string(),
            email: "operator@foundry.local".to_string(),
            is_admin: true,
        }));
        navigate("/", Default::default());
    });

    let navigate_purge = navigate.clone();
    let toast_purge = toast.clone();
    let handle_purge_admin = Callback::new(move |_| {
        is_loading.set(true);
        let navigate = navigate_purge.clone();
        let toast = toast_purge.clone();
        leptos::task::spawn_local(async move {
            match purge_admin().await {
                Ok(_) => {
                    navigate("/setup", Default::default());
                }
                Err(e) => {
                    toast.message.set(Some(e.clone()));
                    is_loading.set(false);
                }
            }
        });
    });

    view! {
        <div class="relative flex items-center justify-center min-h-screen bg-surface font-sans overflow-hidden">
            // Grid background
            <div class="absolute inset-0 opacity-50" style="background-image:url(\"data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' width='14' height='24'%3E%3Crect x='0' y='0' width='1' height='24' fill='%232b468020'/%3E%3Crect x='0' y='0' width='14' height='1' fill='%232b468020'/%3E%3C/svg%3E\");background-size:14px 24px;"></div>
            // Glow
            <div class="absolute top-0 left-1/2 -translate-x-1/2 -translate-y-1/2 w-[600px] h-[400px] bg-primary/20 rounded-full blur-[100px] pointer-events-none"></div>

            <div class="relative z-10 w-full max-w-md p-6">
                <div class="mb-10 text-center">
                    <div class="inline-flex items-center justify-center w-16 h-16 rounded-2xl bg-surface-container border border-outline-variant/20 shadow-[0_0_15px_rgba(123,208,255,0.15)] mb-6 backdrop-blur-md">
                        <span class="material-symbols-outlined text-3xl text-primary">"hub"</span>
                    </div>
                    <h1 class="text-3xl font-light tracking-tight text-on-surface mb-2 font-['Inter']">"The Intelligence Layer"</h1>
                    <p class="text-on-surface-variant text-sm tracking-wide">"Enterprise Operations Control Center"</p>
                </div>

                <div class="p-8 rounded-2xl bg-surface-container/30 border border-outline-variant/10 shadow-2xl backdrop-blur-xl space-y-6">
                    {move || error_message.get().map(|msg| view! {
                        <div class="p-3 bg-error-container/30 text-error text-xs rounded border border-error/20 mb-4 animate-slide-up">
                            {msg}
                        </div>
                    })}

                    <div class="space-y-4 min-h-[140px]">
                        {move || match login_state.get() {
                            LoginState::EmailEntry => view! {
                                <div class="animate-fade-scale space-y-4">
                                    <div class="space-y-1.5">
                                        <label class="text-[10px] font-bold text-on-surface-variant uppercase tracking-wider">"Email / Node ID"</label>
                                        <Input 
                                            r#type=InputType::Email 
                                            placeholder="operator@foundry.local".to_string() 
                                            bind_value=email 
                                        />
                                    </div>
                                    <Button 
                                        class="w-full mt-4 btn-primary-gradient text-on-primary border-none shadow-[0_0_20px_rgba(123,208,255,0.2)] hover:shadow-[0_0_25px_rgba(123,208,255,0.4)] transition-all font-bold".to_string() 
                                        on:click=move |ev| handle_check_flow.run(ev) 
                                        disabled=email.get().is_empty()
                                    >
                                        {move || if is_loading.get() { "Evaluating Node..." } else { "Continue" }}
                                    </Button>
                                </div>
                            }.into_any(),

                            LoginState::PasskeyPrompt => view! {
                                <div class="animate-fade-scale space-y-4">
                                    <div class="text-center pb-2">
                                        <p class="text-sm font-medium text-on-surface-variant">
                                            "Biometrics found for "
                                            <span class="text-on-surface font-bold">{email.get()}</span>
                                        </p>
                                    </div>
                                    <div class="py-2">
                                        <PasskeyLoginButton 
                                            api_base_url=crate::api::client::api_url("/api/auth/webauthn")
                                            email=email
                                            on_success=handle_passkey_success
                                            on_error=handle_passkey_error
                                        />
                                    </div>
                                    <div class="text-center pt-2">
                                        <button type="button" class="text-xs font-bold text-on-surface-variant hover:text-primary transition-colors underline" on:click=move |ev| handle_send_recovery.run(ev)>
                                            "Device lost? Send a Session Recovery Token"
                                        </button>
                                    </div>
                                </div>
                            }.into_any(),

                            LoginState::SendingToken => view! {
                                <div class="animate-fade-scale space-y-4 flex flex-col items-center justify-center py-6">
                                    <span class="material-symbols-outlined text-4xl text-primary animate-spin">"sync"</span>
                                    <p class="text-sm text-on-surface-variant mt-4 font-medium animate-pulse">"Establishing secure email relay..."</p>
                                </div>
                            }.into_any(),

                            LoginState::SetupTokenSent => view! {
                                <div class="animate-fade-scale space-y-4 flex flex-col items-center justify-center text-center py-4">
                                    <div class="w-16 h-16 rounded-full bg-primary/20 flex items-center justify-center mb-2">
                                        <span class="material-symbols-outlined text-3xl text-primary block">"mail"</span>
                                    </div>
                                    <h3 class="text-lg font-bold text-on-surface">"Transmission Sent"</h3>
                                    <p class="text-sm text-on-surface-variant">"A single-use Setup Token has been dispatched to "</p>
                                    <p class="text-sm text-primary font-bold">{email.get()}</p>
                                    <p class="text-xs text-on-surface-variant mt-4 opacity-75">"Check your inbox to authenticate and strictly configure a new physical passkey."</p>
                                    
                                    <button class="mt-6 text-xs text-on-surface-variant underline hover:text-primary" on:click=move |_| { error_message.set(None); login_state.set(LoginState::EmailEntry); }>
                                        "Return to Email Input"
                                    </button>
                                </div>
                            }.into_any(),
                        }}
                    </div>

                    <div class="relative py-2 mt-4">
                        <div class="absolute inset-0 flex items-center"><span class="w-full border-t border-outline-variant/20"></span></div>
                    </div>

                    <Button 
                        variant=shared_ui::components::ui::button::ButtonVariant::Outline
                        class="w-full bg-transparent border-outline-variant/20 text-on-surface-variant hover:bg-surface-bright/10 hover:text-on-surface transition-all".to_string() 
                        on:click=move |ev| handle_demo.run(ev) 
                    >
                        "Explore Demo Mode"
                    </Button>

                    <div class=if cfg!(debug_assertions) { "mt-4" } else { "hidden" }>
                        <Button 
                            variant=shared_ui::components::ui::button::ButtonVariant::Outline
                            class="w-full bg-error-container/10 border-error/30 text-error hover:bg-error-container/30 hover:text-error transition-all".to_string() 
                            on:click=move |ev| handle_purge_admin.run(ev) 
                        >
                            "Purge Admin (Dev)"
                        </Button>
                    </div>
                </div>
            </div>
        </div>
    }
}
