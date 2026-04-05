use leptos::prelude::*;
use leptos_router::hooks::use_navigate;
use shared_ui::components::ui::button::Button;
use shared_ui::components::ui::input::{Input, InputType};
use shared_ui::components::auth::passkey_manager::ManagePasskeys;
use crate::api::setup::{initialize_system, get_setup_status, SetupInitializeRequest};
use crate::api::models::UserInfo;

#[component]
pub fn Setup() -> impl IntoView {
    let step = RwSignal::new(1); // 1 = Admin Info, 2 = Passkey
    let first_name = RwSignal::new("Platform".to_string());
    let last_name = RwSignal::new("Admin".to_string());
    let email = RwSignal::new("".to_string());
    
    let error_message = RwSignal::new(None::<String>);
    let is_loading = RwSignal::new(false);
    
    let auth_token = RwSignal::new("".to_string());
    
    let set_user = use_context::<WriteSignal<Option<UserInfo>>>().expect("set_user context");
    let navigate = use_navigate();
    let navigate_ok = navigate.clone();
    let navigate_setup_check = navigate.clone();
    
    let query = leptos_router::hooks::use_query_map();
    let url_token = move || query.with(|q| q.get("token").unwrap_or_default());

    // Check if system needs setup
    leptos::task::spawn_local(async move {
        if let Ok(status) = get_setup_status().await {
            if !status.needs_setup {
                navigate_setup_check("/login", Default::default());
            }
        }
    });

    let handle_initialize = Callback::new(move |_| {
        error_message.set(None);
        is_loading.set(true);

        let req_data = SetupInitializeRequest {
            email: email.get(),
            first_name: first_name.get(),
            last_name: last_name.get(),
            init_token: Some(url_token()),
        };

        leptos::task::spawn_local(async move {
            match initialize_system(req_data).await {
                Ok(res) => {
                    crate::api::client::set_auth_token(&res.token);
                    set_user.set(res.user);
                    auth_token.set(res.token);
                    step.set(2);
                }
                Err(err) => {
                    error_message.set(Some(err));
                }
            }
            is_loading.set(false);
        });
    });

    let handle_finish = Callback::new(move |_| {
        navigate_ok("/", Default::default());
    });

    view! {
        <div class="relative flex items-center justify-center min-h-screen bg-surface font-sans overflow-hidden">
            // Grid background
            <div class="absolute inset-0 opacity-50" style="background-image:url(\"data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' width='14' height='24'%3E%3Crect x='0' y='0' width='1' height='24' fill='%232b468020'/%3E%3Crect x='0' y='0' width='14' height='1' fill='%232b468020'/%3E%3C/svg%3E\");background-size:14px 24px;"></div>
            // Glow
            <div class="absolute top-0 left-1/2 -translate-x-1/2 -translate-y-1/2 w-[800px] h-[500px] bg-primary/20 rounded-full blur-[120px] pointer-events-none"></div>

            <div class="relative z-10 w-full max-w-xl p-6">
                <div class="mb-10 text-center">
                    <div class="inline-flex items-center justify-center w-16 h-16 rounded-2xl bg-surface-container border border-outline-variant/20 shadow-[0_0_15px_rgba(123,208,255,0.15)] mb-6 backdrop-blur-md">
                        <span class="material-symbols-outlined text-3xl text-primary">"rocket_launch"</span>
                    </div>
                    <h1 class="text-3xl font-light tracking-tight text-on-surface mb-2 font-['Inter']">"System Initialization"</h1>
                    <p class="text-on-surface-variant text-sm tracking-wide">"Complete setup to activate the intelligence layer"</p>
                </div>

                <div class="p-8 rounded-2xl bg-surface-container/30 border border-outline-variant/10 shadow-2xl backdrop-blur-xl">
                    <Show 
                        when=move || !url_token().is_empty() 
                        fallback=|| view! {
                            <div class="text-center animate-fade-in py-4">
                                <div class="inline-flex items-center justify-center w-16 h-16 rounded-full bg-error-container/20 border border-error/30 mb-6">
                                    <span class="material-symbols-outlined text-3xl text-error">"lock"</span>
                                </div>
                                <h2 class="text-xl font-bold text-on-surface mb-3">"Security Token Required"</h2>
                                <p class="text-on-surface-variant text-sm mb-6 leading-relaxed">
                                    "Platform initialization is firmly locked. Please launch the setup flow using the exact secure URL provided by your CI/CD deployment environment."
                                </p>
                                <div class="p-4 bg-surface-container/50 rounded-xl border border-outline-variant/30 text-xs text-left font-mono text-on-surface-variant break-all">
                                    "Format: https://<your-domain>/setup?token=YOUR_CI_TOKEN"
                                </div>
                            </div>
                        }
                    >
                    <div class="mb-8 flex justify-between items-center px-8 relative">
                        <div class="absolute top-1/2 left-12 right-12 h-[2px] bg-outline-variant/20 -z-10 translate-y-[-50%]"></div>
                        <div class=move || { if step.get() >= 1 { "w-8 h-8 rounded-full bg-primary text-on-primary flex items-center justify-center font-bold text-sm shadow-[0_0_10px_rgba(123,208,255,0.5)] transition-all" } else { "w-8 h-8 rounded-full bg-surface-container-highest text-on-surface-variant flex items-center justify-center font-bold text-sm border border-outline-variant/30 transition-all" } }>
                            "1"
                        </div>
                        <div class=move || { if step.get() >= 2 { "w-8 h-8 rounded-full bg-primary text-on-primary flex items-center justify-center font-bold text-sm shadow-[0_0_10px_rgba(123,208,255,0.5)] transition-all" } else { "w-8 h-8 rounded-full bg-surface-container-highest text-on-surface-variant flex items-center justify-center font-bold text-sm border border-outline-variant/30 transition-all" } }>
                            "2"
                        </div>
                    </div>

                    {move || error_message.get().map(|msg| view! {
                        <div class="p-3 bg-error-container/30 text-error text-xs rounded border border-error/20 mb-6 animate-slide-up">
                            {msg}
                        </div>
                    })}

                    {move || if step.get() == 1 {
                        view! {
                            <div class="space-y-6 animate-fade-scale">
                                <div class="text-center mb-6">
                                    <h2 class="text-xl font-bold text-on-surface">"Create Master Admin"</h2>
                                    <p class="text-sm text-on-surface-variant mt-1">"This account will have full access to the platform."</p>
                                </div>
                                <div class="grid grid-cols-2 gap-4">
                                    <div class="space-y-1.5">
                                        <label class="text-[10px] font-bold text-on-surface-variant uppercase tracking-wider">"First Name"</label>
                                        <Input r#type=InputType::Text placeholder="Platform".to_string() bind_value=first_name />
                                    </div>
                                    <div class="space-y-1.5">
                                        <label class="text-[10px] font-bold text-on-surface-variant uppercase tracking-wider">"Last Name"</label>
                                        <Input r#type=InputType::Text placeholder="Admin".to_string() bind_value=last_name />
                                    </div>
                                </div>
                                <div class="space-y-1.5">
                                    <label class="text-[10px] font-bold text-on-surface-variant uppercase tracking-wider">"Admin Email"</label>
                                    <Input r#type=InputType::Email placeholder="admin@foundry.local".to_string() bind_value=email />
                                </div>

                                <Button 
                                    class="w-full mt-6 btn-primary-gradient text-on-primary border-none shadow-[0_0_20px_rgba(123,208,255,0.2)] hover:shadow-[0_0_25px_rgba(123,208,255,0.4)] transition-all font-bold".to_string() 
                                    on:click=move |ev| handle_initialize.run(ev) 
                                >
                                    {move || if is_loading.get() { "Initializing..." } else { "Create Admin & Continue" }}
                                </Button>
                            </div>
                        }.into_any()
                    } else {
                        view! {
                            <div class="space-y-6 animate-fade-scale">
                                <div class="text-center mb-2">
                                    <h2 class="text-xl font-bold text-on-surface">"Secure Your Account"</h2>
                                    <p class="text-sm text-on-surface-variant mt-1">"Create a passkey (Face ID/Touch ID) to log in securely without needing your password in the future."</p>
                                </div>
                                
                                <ManagePasskeys 
                                    api_base_url=crate::api::client::api_url("/api/passkeys") 
                                    auth_token=auth_token.get() 
                                />

                                <div class="pt-6 border-t border-outline-variant/10 mt-6 flex justify-between items-center">
                                    <span class="text-xs text-on-surface-variant">"You can manage passkeys later in Settings."</span>
                                    <Button 
                                        class="bg-surface/50 border-outline-variant/30 text-on-surface hover:bg-surface-bright/50 transition-all".to_string() 
                                        on:click=move |ev| handle_finish.run(ev) 
                                    >
                                        "Finish Setup"
                                    </Button>
                                </div>
                            </div>
                        }.into_any()
                    }}
                    </Show>
                </div>
            </div>
        </div>
    }
}
