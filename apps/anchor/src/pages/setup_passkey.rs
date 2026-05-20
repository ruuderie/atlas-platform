use crate::auth::exchange_setup_token;
use leptos::prelude::*;
use leptos_router::hooks::use_query_map;

#[component]
pub fn SetupPasskey() -> impl IntoView {
    let query = use_query_map();
    let token = move || query.with(|q| q.get("token").unwrap_or_default());

    let is_exchanging = RwSignal::new(false);
    let error = RwSignal::new(Option::<String>::None);
    let is_authenticated = RwSignal::new(false);

    let setup = move |_| {
        let t = token();
        if t.is_empty() {
            error.set(Some("No setup token found in URL.".to_string()));
            return;
        }

        is_exchanging.set(true);
        error.set(None);

        leptos::task::spawn_local(async move {
            match exchange_setup_token(t).await {
                Ok(_) => {
                    is_authenticated.set(true);
                }
                Err(e) => {
                    error.set(Some(e.to_string()));
                }
            }
            is_exchanging.set(false);
        });
    };

    view! {
        <div class="min-h-[80vh] flex items-center justify-center bg-surface p-4">
            <div class="max-w-md w-full bg-surface-container rounded-2xl p-8 shadow-sm">
                {move || if is_authenticated.get() {
                    view! {
                        <div class="space-y-6">
                            <div class="text-center mb-6">
                                <div class="w-16 h-16 bg-primary/10 text-primary rounded-full flex items-center justify-center mx-auto mb-4">
                                    <span class="material-symbols-outlined text-3xl">"passkey"</span>
                                </div>
                                <h1 class="text-2xl font-bold text-on-surface">"Register Passkey"</h1>
                                <p class="text-on-surface-variant mt-2 text-sm">
                                    "Your session is established. Register a passkey to securely log in later."
                                </p>
                            </div>

                            <div class="bg-surface-container-high p-6 rounded-2xl shadow-sm border border-outline-variant/30 mt-6">
                                <h3 class="text-xl font-bold text-on-surface mb-2">"Passkeys"</h3>
                                <p class="text-sm text-on-surface-variant mb-6">
                                    "Use a passkey (Face ID, Touch ID, or a hardware key) to sign in securely without a password."
                                </p>

                                <button
                                    type="button"
                                    id="register-passkey-btn"
                                    class="inline-flex justify-center items-center py-2.5 px-4 font-bold rounded-xl bg-primary text-on-primary hover:bg-primary/90 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-primary transition-all shadow-sm"
                                >
                                    <span class="material-symbols-outlined mr-2">"add_circle"</span>
                                    "Add a Passkey"
                                </button>
                                <div id="passkey-message" class="mt-4 text-sm font-medium"></div>
                            </div>

                            <script src="https://unpkg.com/@simplewebauthn/browser/dist/bundle/index.umd.min.js"></script>
                            <script>
                                "setTimeout(() => {
                                    const btn = document.getElementById('register-passkey-btn');
                                    const msg = document.getElementById('passkey-message');
                                    if(btn) {
                                        btn.addEventListener('click', async () => {
                                            // Idempotency guard: prevent double-registration if button
                                            // is clicked again before the first attempt completes.
                                            if (sessionStorage.getItem('passkey-reg-in-progress')) {
                                                msg.innerText = 'Registration already in progress.';
                                                return;
                                            }

                                            // Resolve the backend API base URL.
                                            // window.__ENV__.API_BASE_URL is injected by the Leptos
                                            // shell at SSR time. Fallback to relative path (dev only).
                                            const apiBase = (window.__ENV__ && window.__ENV__.API_BASE_URL)
                                                ? window.__ENV__.API_BASE_URL.replace(/\\/$/, '')
                                                : '';

                                            try {
                                                sessionStorage.setItem('passkey-reg-in-progress', '1');
                                                btn.disabled = true;
                                                msg.innerText = 'Initiating...';

                                                // NOTE: credentials:'include' sends the HttpOnly
                                                // SameSite=Strict session cookie cross-origin.
                                                const startRes = await fetch(apiBase + '/api/passkeys/start-register', {
                                                    method: 'POST',
                                                    credentials: 'include',
                                                    headers: { 'Content-Type': 'application/json' }
                                                });
                                                if (!startRes.ok) throw new Error('Failed to start registration: ' + await startRes.text());
                                                const options = await startRes.json();

                                                msg.innerText = 'Please follow browser prompts...';
                                                const { startRegistration } = window.SimpleWebAuthnBrowser;
                                                const credential = await startRegistration(options);

                                                msg.innerText = 'Verifying...';
                                                const finishRes = await fetch(apiBase + '/api/passkeys/finish-register', {
                                                    method: 'POST',
                                                    credentials: 'include',
                                                    headers: { 'Content-Type': 'application/json' },
                                                    body: JSON.stringify(credential)
                                                });

                                                if (finishRes.ok) {
                                                    msg.innerText = 'Passkey registered successfully!';
                                                    msg.className = 'mt-4 text-sm font-medium text-green-600';
                                                } else {
                                                    throw new Error(await finishRes.text());
                                                }
                                            } catch (err) {
                                                console.error(err);
                                                msg.innerText = err.message || 'Registration failed';
                                                msg.className = 'mt-4 text-sm font-medium text-red-600';
                                            } finally {
                                                sessionStorage.removeItem('passkey-reg-in-progress');
                                                btn.disabled = false;
                                            }
                                        });
                                    }
                                }, 500);"
                            </script>

                            <a href="/admin" class="block text-center w-full py-3 bg-primary text-on-primary rounded-xl font-medium mt-6 hover:bg-primary/90 transition-colors">
                                "Continue to Dashboard"
                            </a>
                        </div>
                    }.into_any()
                } else {
                    view! {
                        <div class="text-center mb-8">
                            <div class="w-16 h-16 bg-primary/10 text-primary rounded-full flex items-center justify-center mx-auto mb-4">
                                <span class="material-symbols-outlined text-3xl">"lock_open"</span>
                            </div>
                            <h1 class="text-2xl font-bold text-on-surface">"Secure Your Account"</h1>
                            <p class="text-on-surface-variant mt-2 text-sm">
                                "Your administrator has provisioned this account. Click below to securely connect your passkey."
                            </p>
                        </div>
                        <div class="space-y-4">
                            {move || error.get().map(|e| view! {
                                <div class="p-3 bg-error/10 text-error rounded-xl text-sm border border-error/20">
                                    {e}
                                </div>
                            })}
                            <button
                                class="w-full py-3 bg-primary hover:bg-primary/90 text-on-primary rounded-xl font-medium transition-colors disabled:opacity-50 flex items-center justify-center gap-2"
                                disabled=move || is_exchanging.get() || token().is_empty()
                                on:click=setup
                            >
                                {move || if is_exchanging.get() {
                                    view! { <span class="material-symbols-outlined animate-spin">"progress_activity"</span> }.into_any()
                                } else {
                                    view! { <span class="material-symbols-outlined">"arrow_forward"</span> }.into_any()
                                }}
                                {move || if is_exchanging.get() { "Verifying..." } else { "Begin Setup" }}
                            </button>
                        </div>
                    }.into_any()
                }}
            </div>
        </div>
    }
}
