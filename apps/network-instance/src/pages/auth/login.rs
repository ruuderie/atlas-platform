use leptos::prelude::*;
use leptos_router::hooks::use_navigate;
use shared_ui::auth::atlas_auth::use_atlas_auth;
use shared_ui::components::auth::passkey_login::PasskeyLoginButton;
use shared_ui::components::ui::button::Button;
use shared_ui::components::ui::input::{Input, InputType};

#[component]
pub fn Login() -> impl IntoView {
    let auth = use_atlas_auth();
    let _navigate = use_navigate();

    let handle_passkey_success = move |_token: String| {
        // Auth is cookie-based — the backend already set the session cookie.
        // Just navigate to the dashboard.
        window().location().set_href("/dashboard").unwrap();
    };

    let handle_passkey_error = move |err: String| {
        auth.error.set(Some(err));
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

                    <div class="mt-8 space-y-6">
                        {move || auth.error.get().map(|msg| view! {
                            <div class="bg-error/10 border border-error/20 text-error px-4 py-3 rounded-xl text-sm font-medium animate-slide-up">
                                {msg}
                            </div>
                        })}

                        <div class="space-y-4 min-h-[140px]">
                            {move || if auth.use_email.get() {
                                view! {
                                    <div class="animate-fade-scale space-y-4">
                                        <div class="space-y-1.5">
                                            <label class="text-[10px] font-bold text-on-surface-variant uppercase tracking-wider">"Email address"</label>
                                            <input type="email"
                                                class="appearance-none block w-full px-4 py-3 border border-outline-variant/50 rounded-xl placeholder-outline-variant focus:outline-none focus:ring-2 focus:ring-[#004289] focus:border-transparent transition-all sm:text-sm font-medium text-on-surface bg-surface-container-lowest"
                                                placeholder="name@company.com"
                                                prop:value=move || auth.email.get()
                                                on:input=move |ev| auth.email.set(event_target_value(&ev))
                                            />
                                        </div>
                                        <button
                                            class="group relative w-full flex justify-center py-3.5 px-4 border border-transparent text-sm font-bold rounded-xl text-white bg-slate-800 hover:bg-slate-900 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-slate-900 transition-all disabled:opacity-70 shadow-sm"
                                            on:click=move |_| { auth.dispatch_login.dispatch(()); }
                                            disabled=move || auth.email.get().is_empty() || auth.is_loading.get() || (auth.countdown.get() > 0)
                                        >
                                            {move || if auth.is_loading.get() {
                                                "Sending...".to_string()
                                            } else if auth.countdown.get() > 0 {
                                                format!("Resend in {}s", auth.countdown.get())
                                            } else if auth.error.get() == Some("Magic link sent! Check your email.".to_string()) {
                                                "Resend Magic Link".to_string()
                                            } else {
                                                "Send Magic Link".to_string()
                                            }}
                                        </button>

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
                                                api_base_url=crate::get_api_base_url() + "/api/auth/passkeys"
                                                email=RwSignal::new("".to_string())
                                                on_success=handle_passkey_success.clone()
                                                on_error=handle_passkey_error.clone()
                                            />
                                        </div>
                                        <div class="text-center pt-2">
                                            <button
                                                type="button"
                                                class="text-xs font-bold text-outline hover:text-[#004289] transition-colors uppercase tracking-widest"
                                                on:click=move |_| auth.use_email.set(true)
                                            >
                                                "Use Email Instead"
                                            </button>
                                        </div>
                                    </div>
                                }.into_any()
                            }}
                        </div>

                        <div class="text-center mt-6">
                            <p class="text-sm text-on-surface-variant font-medium">
                                "Don't have an account? "
                                <a href="/auth/register" class="font-bold text-[#004289] hover:text-[#00336b] hover:underline transition-colors">"Register now"</a>
                            </p>
                        </div>
                    </div>
                </div>
            </div>
        </crate::components::layout::MainLayout>
    }
}
