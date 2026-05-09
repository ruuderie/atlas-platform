use leptos::prelude::*;
use leptos_router::hooks::use_navigate;
use shared_ui::components::ui::button::Button;
use shared_ui::components::ui::input::{Input, InputType};
use crate::api::models::UserInfo;
use crate::api::setup::{get_setup_status, purge_admin};
use shared_ui::components::auth::passkey_login::PasskeyLoginButton;
use shared_ui::auth::atlas_auth::use_atlas_auth;

#[component]
pub fn Login() -> impl IntoView {
    let auth = use_atlas_auth();
    let set_user = use_context::<WriteSignal<Option<UserInfo>>>().expect("set_user context");
    let toast = use_context::<crate::app::GlobalToast>().expect("toast context");
    let navigate = use_navigate();
    let is_purging = RwSignal::new(false);
    
    // Setup status check
    let navigate_setup = navigate.clone();
    leptos::task::spawn_local(async move {
        if let Ok(status) = get_setup_status().await {
            if status.needs_setup {
                navigate_setup("/setup", Default::default());
            }
        }
    });

    let set_user_pk = set_user.clone();
    let navigate_pk = navigate.clone();
    let toast_pk = toast.clone();
    let handle_passkey_success = Callback::new(move |_token: String| {
        let set_user = set_user_pk.clone();
        let navigate = navigate_pk.clone();
        let toast = toast_pk.clone();
        leptos::task::spawn_local(async move {
            if let Ok(user) = crate::api::auth::validate_session().await {
                set_user.set(Some(user));
                navigate("/", Default::default());
            } else {
                toast.message.set(Some("Validated passkey, but session handshake failed.".to_string()));
            }
        });
    });

    let handle_passkey_error = Callback::new(move |err: String| {
        auth.error.set(Some(err));
    });

    let navigate_purge = navigate.clone();
    let toast_purge = toast.clone();
    let handle_purge_admin = Callback::new(move |_| {
        is_purging.set(true);
        let navigate = navigate_purge.clone();
        let toast = toast_purge.clone();
        leptos::task::spawn_local(async move {
            match purge_admin().await {
                Ok(_) => { navigate("/setup", Default::default()); }
                Err(e) => {
                    toast.message.set(Some(e.clone()));
                    is_purging.set(false);
                }
            }
        });
    });

    view! {
        <div class="relative flex items-center justify-center min-h-screen bg-surface font-sans overflow-hidden">
            <div class="absolute inset-0 opacity-50" style="background-image:url(\"data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' width='14' height='24'%3E%3Crect x='0' y='0' width='1' height='24' fill='%232b468020'/%3E%3Crect x='0' y='0' width='14' height='1' fill='%232b468020'/%3E%3C/svg%3E\");background-size:14px 24px;"></div>
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
                    {move || auth.error.get().map(|msg| view! {
                        <div class="p-3 bg-error-container/30 text-error text-xs rounded border border-error/20 mb-4 animate-slide-up">
                            {msg}
                        </div>
                    })}

                    <div class="space-y-4 min-h-[140px]">
                        {move || if auth.use_email.get() {
                            view! {
                                <div class="animate-fade-scale space-y-4">
                                    <div class="space-y-1.5">
                                        <label class="text-[10px] font-bold text-on-surface-variant uppercase tracking-wider">"Email / Node ID"</label>
                                        <Input 
                                            r#type=InputType::Email 
                                            placeholder="operator@foundry.local".to_string() 
                                            bind_value=auth.email 
                                        />
                                    </div>
                                    <Button 
                                        class="w-full mt-4 btn-primary-gradient text-on-primary border-none shadow-[0_0_20px_rgba(123,208,255,0.2)] hover:shadow-[0_0_25px_rgba(123,208,255,0.4)] transition-all font-bold".to_string() 
                                        on:click=move |_| { auth.dispatch_login.dispatch(()); } 
                                        attr:disabled=move || auth.email.get().is_empty() || auth.is_loading.get() || (auth.countdown.get() > 0)
                                    >
                                        {move || if auth.is_loading.get() { 
                                            "Evaluating Node...".to_string() 
                                        } else if auth.countdown.get() > 0 {
                                            format!("Resend in {}s", auth.countdown.get())
                                        } else if auth.error.get() == Some("Magic link sent! Check your email.".to_string()) {
                                            "Resend Magic Link".to_string()
                                        } else { 
                                            "Send Magic Link".to_string() 
                                        }}
                                    </Button>

                                    <div class="text-center pt-2">
                                        <button
                                            type="button"
                                            class="text-xs font-bold text-outline hover:text-primary transition-colors uppercase tracking-widest"
                                            on:click=move |_| { auth.use_email.set(false); auth.error.set(None); }
                                        >
                                            "\u{2190} Back to Passkey"
                                        </button>
                                    </div>
                                </div>
                            }.into_any()
                        } else {
                            view! {
                                <div class="animate-fade-scale space-y-4">
                                    <div class="text-center pb-2">
                                        <p class="text-sm font-medium text-on-surface-variant">
                                            "Biometric Authentication"
                                        </p>
                                    </div>
                                    <div class="py-2">
                                        <PasskeyLoginButton 
                                            api_base_url=crate::api::client::api_url("/api/passkeys")
                                            email=RwSignal::new("".to_string())
                                            on_success=handle_passkey_success.clone()
                                            on_error=handle_passkey_error.clone()
                                        />
                                    </div>
                                    <div class="text-center pt-2">
                                        <button
                                            type="button"
                                            class="text-xs font-bold text-outline hover:text-primary transition-colors uppercase tracking-widest"
                                            on:click=move |_| auth.use_email.set(true)
                                        >
                                            "Use Email Instead"
                                        </button>
                                    </div>
                                </div>
                            }.into_any()
                        }}
                    </div>

                    <div class=if cfg!(debug_assertions) { "mt-4" } else { "hidden" }>
                        <Button 
                            variant=shared_ui::components::ui::button::ButtonVariant::Outline
                            class="w-full bg-error-container/10 border-error/30 text-error hover:bg-error-container/30 hover:text-error transition-all".to_string() 
                            on:click=move |ev| handle_purge_admin.run(ev) 
                            attr:disabled=move || is_purging.get()
                        >
                            {move || if is_purging.get() { "Purging..." } else { "Purge Admin (Dev)" }}
                        </Button>
                    </div>
                </div>
            </div>
        </div>
    }
}
