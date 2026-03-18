use leptos::prelude::*;
use leptos_router::hooks::use_navigate;
use shared_ui::components::card::Card;
use shared_ui::components::ui::button::Button;
use shared_ui::components::ui::input::{Input, InputType};
use shared_ui::components::ui::label::Label;
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
                    // Update global user context
                    set_user.set(res.user);
                    // Redirect to dashboard
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
        <div class="relative flex items-center justify-center min-h-screen bg-slate-950 font-sans overflow-hidden">
            <div class="absolute inset-0 bg-[linear-gradient(to_right,#4f4f4f2e_1px,transparent_1px),linear-gradient(to_bottom,#4f4f4f2e_1px,transparent_1px)] bg-[size:14px_24px] [mask-image:radial-gradient(ellipse_60%_50%_at_50%_0%,#000_70%,transparent_100%)]"></div>
            
            <div class="absolute top-0 left-1/2 -translate-x-1/2 -translate-y-1/2 w-[600px] h-[400px] bg-blue-600/30 rounded-full blur-[100px] pointer-events-none"></div>

            <div class="relative z-10 w-full max-w-md p-6">
                <div class="mb-10 text-center">
                    <div class="inline-flex items-center justify-center w-16 h-16 rounded-2xl bg-white/5 border border-white/10 shadow-[0_0_15px_rgba(37,99,235,0.2)] mb-6 backdrop-blur-md">
                        <svg class="w-8 h-8 text-blue-400" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="1.5" d="M14 10l-2 1m0 0l-2-1m2 1v2.5M20 7l-2 1m2-1l-2-1m2 1v2.5M14 4l-2-1-2 1M4 7l2-1M4 7l2 1M4 7v2.5M12 21l-2-1m2 1l2-1m-2 1v-2.5M6 18l-2-1v-2.5M18 18l2-1v-2.5" />
                        </svg>
                    </div>
                    <h1 class="text-3xl font-light tracking-tight text-white mb-2 font-['Inter']">"Platform Admin"</h1>
                    <p class="text-slate-400 text-sm tracking-wide">"Enterprise Operations Control Center"</p>
                </div>

                <div class="p-8 rounded-2xl bg-white/[0.03] border border-white/[0.05] shadow-2xl backdrop-blur-xl space-y-6">
                    <div class="space-y-4">
                        <div class="space-y-1.5">
                            <label class="text-xs font-medium text-slate-300 uppercase tracking-wider">"Email / Node ID"</label>
                            <Input 
                                r#type=InputType::Email 
                                placeholder="operator@foundry.local".to_string() 
                                bind_value=email 
                            />
                        </div>
                        
                        <div class="space-y-1.5">
                            <label class="text-xs font-medium text-slate-300 uppercase tracking-wider">"Access Token"</label>
                            <Input 
                                r#type=InputType::Password 
                                placeholder="••••••••".to_string() 
                                bind_value=password 
                            />
                        </div>

                        {move || error_message.get().map(|msg| view! {
                            <div class="p-3 bg-red-500/10 text-red-400 text-xs rounded border border-red-500/20">
                                {msg}
                            </div>
                        })}

                        <Button 
                            class="w-full mt-4 bg-blue-600 hover:bg-blue-500 text-white border-none shadow-[0_0_20px_rgba(37,99,235,0.3)] hover:shadow-[0_0_25px_rgba(37,99,235,0.5)] transition-all".to_string() 
                            on:click=handle_login 
                        >
                            {move || if is_loading.get() { "Authenticating..." } else { "Initialize Session" }}
                        </Button>
                    </div>

                    <div class="relative py-2">
                        <div class="absolute inset-0 flex items-center"><span class="w-full border-t border-white/10"></span></div>
                        <div class="relative flex justify-center text-xs uppercase"><span class="bg-slate-950 px-2 text-slate-500">"Or"</span></div>
                    </div>

                    <Button 
                        variant=shared_ui::components::ui::button::ButtonVariant::Outline
                        class="w-full bg-transparent border-white/10 text-slate-300 hover:bg-white/5 hover:text-white transition-all".to_string() 
                        on:click=handle_demo 
                    >
                        "Explore Demo Mode"
                    </Button>
                </div>
            </div>
        </div>
    }
}
