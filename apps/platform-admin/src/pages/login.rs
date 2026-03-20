use leptos::prelude::*;
use leptos_router::hooks::use_navigate;
use shared_ui::components::ui::button::Button;
use shared_ui::components::ui::input::{Input, InputType};
use crate::api::auth::login;
use crate::api::models::{UserLogin, UserInfo};

#[component]
pub fn Login() -> impl IntoView {
    let email = RwSignal::new("".to_string());
    let password = RwSignal::new("".to_string());
    let error_message = RwSignal::new(None::<String>);
    let is_loading = RwSignal::new(false);
    let set_user = use_context::<WriteSignal<Option<UserInfo>>>().expect("set_user context");
    let toast = use_context::<crate::app::GlobalToast>().expect("toast context");

    let navigate = use_navigate();
    let navigate_demo = navigate.clone();

    let handle_login = move |_| {
        crate::api::client::set_demo_mode(false);
        let navigate = navigate.clone();
        error_message.set(None);
        is_loading.set(true);

        let credentials = UserLogin {
            email: email.get(),
            password: password.get(),
        };

        leptos::task::spawn_local(async move {
            match login(credentials).await {
                Ok(res) => {
                    set_user.set(res.user);
                    navigate("/", Default::default());
                }
                Err(err) => {
                    toast.message.set(Some(err.clone()));
                    error_message.set(Some(err));
                }
            }
            is_loading.set(false);
        });
    };

    let handle_demo = move |_| {
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
    };

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
                    <div class="space-y-4">
                        <div class="space-y-1.5">
                            <label class="text-[10px] font-bold text-on-surface-variant uppercase tracking-wider">"Email / Node ID"</label>
                            <Input 
                                r#type=InputType::Email 
                                placeholder="operator@foundry.local".to_string() 
                                bind_value=email 
                            />
                        </div>
                        
                        <div class="space-y-1.5">
                            <label class="text-[10px] font-bold text-on-surface-variant uppercase tracking-wider">"Access Token"</label>
                            <Input 
                                r#type=InputType::Password 
                                placeholder="••••••••".to_string() 
                                bind_value=password 
                            />
                        </div>

                        {move || error_message.get().map(|msg| view! {
                            <div class="p-3 bg-error-container/30 text-error text-xs rounded border border-error/20">
                                {msg}
                            </div>
                        })}

                        <Button 
                            class="w-full mt-4 btn-primary-gradient text-on-primary border-none shadow-[0_0_20px_rgba(123,208,255,0.2)] hover:shadow-[0_0_25px_rgba(123,208,255,0.4)] transition-all font-bold".to_string() 
                            on:click=handle_login 
                        >
                            {move || if is_loading.get() { "Authenticating..." } else { "Initialize Session" }}
                        </Button>
                    </div>

                    <div class="relative py-2">
                        <div class="absolute inset-0 flex items-center"><span class="w-full border-t border-outline-variant/20"></span></div>
                        <div class="relative flex justify-center text-xs uppercase"><span class="bg-surface px-2 text-on-surface-variant">"Or"</span></div>
                    </div>

                    <Button 
                        variant=shared_ui::components::ui::button::ButtonVariant::Outline
                        class="w-full bg-transparent border-outline-variant/20 text-on-surface-variant hover:bg-surface-bright/10 hover:text-on-surface transition-all".to_string() 
                        on:click=handle_demo 
                    >
                        "Explore Demo Mode"
                    </Button>
                </div>
            </div>
        </div>
    }
}
