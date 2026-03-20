use leptos::prelude::*;
use leptos_router::hooks::use_navigate;
use serde_json::json;
use crate::app::DirectoryConfig;

#[component]
pub fn Register() -> impl IntoView {
    let email = RwSignal::new("".to_string());
    let password = RwSignal::new("".to_string());
    let first_name = RwSignal::new("".to_string());
    let last_name = RwSignal::new("".to_string());
    let phone_num = RwSignal::new("".to_string());

    let error = RwSignal::new("".to_string());
    let is_submitting = RwSignal::new(false);
    let success = RwSignal::new(false);
    
    let config = use_context::<DirectoryConfig>().expect("DirectoryConfig must be provided");
    let directory_id = RwSignal::new(config.id.clone());

    view! {
        <crate::components::layout::MainLayout>
            <div class="min-h-[80vh] flex items-center justify-center py-12 px-4 sm:px-6 lg:px-8 bg-surface-container-lowest">
                {move || if success.get() {
                    view! {
                        <div class="max-w-md w-full space-y-8 bg-white p-10 rounded-2xl shadow-premium border border-outline-variant/30 text-center animate-fade-scale">
                            <div class="w-20 h-20 bg-emerald-100 rounded-full flex items-center justify-center mx-auto mb-6 text-emerald-600">
                                <span class="material-symbols-outlined text-4xl" data-icon="check_circle">"check_circle"</span>
                            </div>
                            <h2 class="text-3xl font-extrabold font-headline text-on-surface">"Registration Successful"</h2>
                            <p class="mt-4 text-on-surface-variant font-medium leading-relaxed">
                                "Your account has been created. Please log in to continue."
                            </p>
                            <div class="mt-8">
                                <a href="/auth/login" class="inline-flex justify-center w-full py-3.5 px-4 border border-transparent text-sm font-bold rounded-xl text-white bg-[#004289] hover:bg-[#00336b] transition-colors shadow-sm shadow-[#004289]/20">
                                    "Go to Login"
                                </a>
                            </div>
                        </div>
                    }.into_any()
                } else {
                    view! {
                        <div class="max-w-md w-full space-y-8 bg-white p-10 rounded-2xl shadow-premium border border-outline-variant/30">
                            <div>
                                <div class="w-16 h-16 bg-[#004289]/10 rounded-2xl flex items-center justify-center mx-auto mb-6">
                                    <span class="material-symbols-outlined text-[#004289] text-3xl">"person_add"</span>
                                </div>
                                <h2 class="text-center text-3xl font-extrabold font-headline text-on-surface">"Create an account"</h2>
                                <p class="mt-2 text-center text-sm text-on-surface-variant font-medium">
                                    "Join as a service provider today"
                                </p>
                            </div>
                            
                            <form class="mt-8 space-y-6" on:submit=move |ev: leptos::ev::SubmitEvent| {
                                ev.prevent_default();
                                if is_submitting.get() { return; }
                                
                                error.set("".to_string());
                                if email.get().is_empty() || password.get().is_empty() || first_name.get().is_empty() || last_name.get().is_empty() {
                                    error.set("Please fill out all required fields.".to_string());
                                    return;
                                }

                                is_submitting.set(true);
                                let email_val = email.get();
                                let pass_val = password.get();
                                let fname_val = first_name.get();
                                let lname_val = last_name.get();
                                let phone_val = phone_num.get();
                                let dir_id = directory_id.get();

                                leptos::task::spawn_local(async move {
                                    let client = reqwest::Client::new();
                                    let url = "http://127.0.0.1:8000/api/auth/register";
                                    let payload = json!({
                                        "directory_id": dir_id,
                                        "first_name": fname_val,
                                        "last_name": lname_val,
                                        "email": email_val,
                                        "password": pass_val,
                                        "phone": phone_val,
                                        "username": email_val, // username is email for now
                                    });

                                    match client.post(url).json(&payload).send().await {
                                        Ok(res) => {
                                            if res.status().is_success() {
                                                success.set(true);
                                            } else if res.status() == reqwest::StatusCode::CONFLICT {
                                                error.set("An account with this email already exists.".to_string());
                                            } else {
                                                error.set("Failed to register. Please try again.".to_string());
                                            }
                                        }
                                        Err(_) => {
                                            error.set("Network error. Could not reach server.".to_string());
                                        }
                                    }
                                    is_submitting.set(false);
                                });
                            }>
                                {move || if !error.get().is_empty() {
                                    view! {
                                        <div class="bg-error/10 border border-error/20 text-error px-4 py-3 rounded-xl text-sm font-medium animate-slide-up">
                                            {error.get()}
                                        </div>
                                    }.into_any()
                                } else { view! { <span/> }.into_any() }}

                                <div class="space-y-4">
                                    <div class="grid grid-cols-2 gap-4">
                                        <div>
                                            <label class="block text-xs font-bold text-on-surface-variant uppercase tracking-wider mb-2">"First name"</label>
                                            <input type="text" required
                                                class="appearance-none block w-full px-4 py-3 border border-outline-variant/50 rounded-xl placeholder-outline-variant focus:outline-none focus:ring-2 focus:ring-[#004289] focus:border-transparent transition-all sm:text-sm font-medium text-on-surface bg-surface-container-lowest"
                                                placeholder="John"
                                                prop:value=first_name
                                                on:input=move |ev| first_name.set(event_target_value(&ev))
                                            />
                                        </div>
                                        <div>
                                            <label class="block text-xs font-bold text-on-surface-variant uppercase tracking-wider mb-2">"Last name"</label>
                                            <input type="text" required
                                                class="appearance-none block w-full px-4 py-3 border border-outline-variant/50 rounded-xl placeholder-outline-variant focus:outline-none focus:ring-2 focus:ring-[#004289] focus:border-transparent transition-all sm:text-sm font-medium text-on-surface bg-surface-container-lowest"
                                                placeholder="Doe"
                                                prop:value=last_name
                                                on:input=move |ev| last_name.set(event_target_value(&ev))
                                            />
                                        </div>
                                    </div>
                                    <div>
                                        <label class="block text-xs font-bold text-on-surface-variant uppercase tracking-wider mb-2">"Email address"</label>
                                        <input type="email" autocomplete="email" required
                                            class="appearance-none block w-full px-4 py-3 border border-outline-variant/50 rounded-xl placeholder-outline-variant focus:outline-none focus:ring-2 focus:ring-[#004289] focus:border-transparent transition-all sm:text-sm font-medium text-on-surface bg-surface-container-lowest"
                                            placeholder="name@company.com"
                                            prop:value=email
                                            on:input=move |ev| email.set(event_target_value(&ev))
                                        />
                                    </div>
                                    <div>
                                        <label class="block text-xs font-bold text-on-surface-variant uppercase tracking-wider mb-2">"Phone Number (Opt)"</label>
                                        <input type="tel" autocomplete="tel"
                                            class="appearance-none block w-full px-4 py-3 border border-outline-variant/50 rounded-xl placeholder-outline-variant focus:outline-none focus:ring-2 focus:ring-[#004289] focus:border-transparent transition-all sm:text-sm font-medium text-on-surface bg-surface-container-lowest"
                                            placeholder="+1 (555) 000-0000"
                                            prop:value=phone_num
                                            on:input=move |ev| phone_num.set(event_target_value(&ev))
                                        />
                                    </div>
                                    <div>
                                        <label class="block text-xs font-bold text-on-surface-variant uppercase tracking-wider mb-2">"Password"</label>
                                        <input type="password" autocomplete="new-password" required minlength="8"
                                            class="appearance-none block w-full px-4 py-3 border border-outline-variant/50 rounded-xl placeholder-outline-variant focus:outline-none focus:ring-2 focus:ring-[#004289] focus:border-transparent transition-all sm:text-sm font-medium text-on-surface bg-surface-container-lowest"
                                            placeholder="••••••••"
                                            prop:value=password
                                            on:input=move |ev| password.set(event_target_value(&ev))
                                        />
                                    </div>
                                </div>

                                <div>
                                    <button type="submit" disabled=is_submitting
                                        class="group relative w-full flex justify-center py-3.5 px-4 border border-transparent text-sm font-bold rounded-xl text-white bg-[#004289] hover:bg-[#00336b] focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-[#004289] transition-all disabled:opacity-70 shadow-sm shadow-[#004289]/20"
                                    >
                                        {move || if is_submitting.get() { "Creating account..." } else { "Create account" }}
                                    </button>
                                </div>
                                
                                <div class="text-center mt-6">
                                    <p class="text-sm text-on-surface-variant font-medium">
                                        "Already have an account? "
                                        <a href="/auth/login" class="font-bold text-[#004289] hover:text-[#00336b] hover:underline transition-colors">"Sign in"</a>
                                    </p>
                                </div>
                            </form>
                        </div>
                    }.into_any()
                }}
            </div>
        </crate::components::layout::MainLayout>
    }
}
