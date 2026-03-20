use leptos::prelude::*;
use leptos_router::hooks::use_navigate;
use serde_json::json;

use crate::auth::set_auth_token;

#[component]
pub fn Login() -> impl IntoView {
    let email = RwSignal::new("".to_string());
    let password = RwSignal::new("".to_string());
    let error = RwSignal::new("".to_string());
    let is_submitting = RwSignal::new(false);
    
    let navigate = use_navigate();

    let handle_login = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        if is_submitting.get() { return; }
        
        error.set("".to_string());
        if email.get().is_empty() || password.get().is_empty() {
            error.set("Please enter your email and password.".to_string());
            return;
        }

        is_submitting.set(true);
        let email_val = email.get();
        let pass_val = password.get();

        leptos::task::spawn_local(async move {
            let client = reqwest::Client::new();
            let url = "http://127.0.0.1:8000/api/auth/login";
            let payload = json!({
                "email": email_val,
                "password": pass_val,
            });

            match client.post(url).json(&payload).send().await {
                Ok(res) => {
                    if res.status().is_success() {
                        if let Ok(json) = res.json::<serde_json::Value>().await {
                            if let Some(token) = json.get("token").and_then(|t| t.as_str()) {
                                set_auth_token(token);
                                // Refresh logic could go here or hard navigation
                                window().location().set_href("/dashboard").unwrap();
                            } else {
                                error.set("Invalid response from server".to_string());
                            }
                        }
                    } else {
                        if res.status() == reqwest::StatusCode::UNAUTHORIZED {
                            error.set("Invalid email or password.".to_string());
                        } else {
                            error.set("Failed to login. Please try again.".to_string());
                        }
                    }
                }
                Err(_) => {
                    error.set("Network error. Could not reach server.".to_string());
                }
            }
            is_submitting.set(false);
        });
    };

    view! {
        <crate::components::layout::MainLayout>
            <div class="min-h-[80vh] flex items-center justify-center py-12 px-4 sm:px-6 lg:px-8 bg-surface-container-lowest">
                <div class="max-w-md w-full space-y-8 bg-white p-10 rounded-2xl shadow-premium border border-outline-variant/30">
                    <div>
                        <div class="w-16 h-16 bg-[#004289]/10 rounded-2xl flex items-center justify-center mx-auto mb-6">
                            <span class="material-symbols-outlined text-[#004289] text-3xl">"login"</span>
                        </div>
                        <h2 class="text-center text-3xl font-extrabold font-headline text-on-surface">"Welcome back"</h2>
                        <p class="mt-2 text-center text-sm text-on-surface-variant font-medium">
                            "Sign in to manage your listings and profile"
                        </p>
                    </div>
                    
                    <form class="mt-8 space-y-6" on:submit=handle_login>
                        {move || if !error.get().is_empty() {
                            view! {
                                <div class="bg-error/10 border border-error/20 text-error px-4 py-3 rounded-xl text-sm font-medium animate-slide-up">
                                    {error.get()}
                                </div>
                            }.into_any()
                        } else { view! { <span/> }.into_any() }}

                        <div class="space-y-4">
                            <div>
                                <label for="email-address" class="block text-xs font-bold text-on-surface-variant uppercase tracking-wider mb-2">"Email address"</label>
                                <input id="email-address" name="email" type="email" autocomplete="email" required
                                    class="appearance-none block w-full px-4 py-3 border border-outline-variant/50 rounded-xl placeholder-outline-variant focus:outline-none focus:ring-2 focus:ring-[#004289] focus:border-transparent transition-all sm:text-sm font-medium text-on-surface bg-surface-container-lowest"
                                    placeholder="name@company.com"
                                    prop:value=email
                                    on:input=move |ev| email.set(event_target_value(&ev))
                                />
                            </div>
                            <div>
                                <div class="flex items-center justify-between mb-2">
                                    <label for="password" class="block text-xs font-bold text-on-surface-variant uppercase tracking-wider">"Password"</label>
                                    <a href="#" class="text-xs font-bold text-[#004289] hover:underline">"Forgot password?"</a>
                                </div>
                                <input id="password" name="password" type="password" autocomplete="current-password" required
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
                                {move || if is_submitting.get() { "Signing in..." } else { "Sign in" }}
                            </button>
                        </div>
                        
                        <div class="text-center mt-6">
                            <p class="text-sm text-on-surface-variant font-medium">
                                "Don't have an account? "
                                <a href="/auth/register" class="font-bold text-[#004289] hover:text-[#00336b] hover:underline transition-colors">"Register now"</a>
                            </p>
                        </div>
                    </form>
                </div>
            </div>
        </crate::components::layout::MainLayout>
    }
}
