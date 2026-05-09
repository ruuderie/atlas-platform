use leptos::prelude::*;
use leptos::task::spawn_local;
use crate::auth::api_base_url;

#[allow(unused_variables)]
#[component]
pub fn LoginModal(
    /// Triggered when authentication succeeds so the parent can refresh state
    #[prop(into)] on_success: Callback<(), ()>,
    /// Controls modal visibility
    #[prop(into)] is_open: Signal<bool>,
    /// Callback to close the modal
    #[prop(into)] on_close: Callback<(), ()>,
) -> impl IntoView {
    let (use_email, set_use_email) = signal(false);
    let (email_input, set_email_input) = signal(String::new());
    let (is_loading, set_is_loading) = signal(false);
    let (auth_message, set_auth_message) = signal(String::new());
    let (is_error, set_is_error) = signal(false);

    let handle_magic_link = move |_| {
        let email = email_input.get_untracked();
        if email.is_empty() {
            set_auth_message.set("Email is required.".to_string());
            set_is_error.set(true);
            return;
        }
        set_is_loading.set(true);
        set_auth_message.set(String::new());
        set_is_error.set(false);

        // Standard Leptos spawn_local for the network request
        spawn_local(async move {
            let url = format!("{}/api/auth/magic-link/request", api_base_url());
            let client = reqwest::Client::new();
            let res = client.post(&url)
                .json(&serde_json::json!({ "email": email }))
                .send()
                .await;

            set_is_loading.set(false);
            match res {
                Ok(r) if r.status().is_success() => {
                    set_is_error.set(false);
                    set_auth_message.set("Magic link sent! Check your email.".to_string());
                }
                _ => {
                    set_is_error.set(true);
                    set_auth_message.set("Failed to send magic link.".to_string());
                }
            }
        });
    };

    // To avoid hydration slot issues with passkeys, we use native JS events for the WebAuthn flow
    // similar to the anchor app. We inject the SimpleWebAuthn browser script.
    
    // When the modal opens, reset state
    Effect::new(move |_| {
        if is_open.get() {
            set_use_email.set(false);
            set_auth_message.set(String::new());
            set_email_input.set(String::new());
            set_is_loading.set(false);
        }
    });

    view! {
        <Show when=move || is_open.get() fallback=move || view! { <span/> }>
            <div class="fixed inset-0 z-50 flex items-center justify-center p-4 bg-black/60 backdrop-blur-sm animate-fade-in">
                <div class="bg-white rounded-2xl shadow-premium w-full max-w-md overflow-hidden relative animate-slide-up">
                    <button 
                        class="absolute top-4 right-4 text-on-surface-variant hover:text-on-surface transition-colors"
                        on:click=move |_| on_close.run(())
                    >
                        <span class="material-symbols-outlined">"close"</span>
                    </button>
                    
                    <div class="p-8">
                        <div class="mb-8 text-center">
                            <h2 class="text-2xl font-headline font-extrabold text-[#004289] mb-2">"Welcome Back"</h2>
                            <p class="text-on-surface-variant text-sm">"Sign in to manage your account and alerts."</p>
                        </div>

                        <Show when=move || use_email.get() fallback=move || view! {
                            <div class="space-y-4">
                                <button
                                    id="network-login-passkey-btn"
                                    class="w-full bg-[#004289] text-white py-4 rounded-xl font-bold text-sm hover:bg-[#003366] transition-all transform hover:scale-[1.02] flex items-center justify-center gap-3 shadow-md"
                                >
                                    <span class="material-symbols-outlined">"passkey"</span>
                                    "Sign In with Passkey"
                                </button>

                                <div id="network-passkey-message" class="text-sm text-center text-error font-medium h-4"></div>

                                <button
                                    class="w-full py-4 text-[#004289] font-bold text-sm hover:bg-surface-container transition-colors rounded-xl border border-outline-variant/30"
                                    on:click=move |_| set_use_email.set(true)
                                >
                                    "Use Email Instead"
                                </button>

                                // The passkey login button uses native JS events to avoid Leptos SSR
                                // hydration mismatches — WebAuthn browser API calls can only run
                                // client-side and must never be in the Leptos reactive graph.
                                //
                                // We dynamically insert the SimpleWebAuthn script and bind the
                                // click listener inside its `onload` callback, which eliminates the
                                // race condition of a setTimeout-based approach on slow connections.
                                <script>
                                {format!("
                                (function() {{
                                    var existing = document.querySelector('script[data-simplewebauthn]');
                                    var scriptEl = existing || document.createElement('script');
                                    if (!existing) {{
                                        scriptEl.src = 'https://unpkg.com/@simplewebauthn/browser/dist/bundle/index.umd.min.js';
                                        scriptEl.setAttribute('data-simplewebauthn', 'true');
                                        document.head.appendChild(scriptEl);
                                    }}

                                    function bindBtn() {{
                                        var btn = document.getElementById('network-login-passkey-btn');
                                        var msg = document.getElementById('network-passkey-message');
                                        if (!btn || btn.dataset.bound) return;
                                        btn.dataset.bound = 'true';
                                        btn.addEventListener('click', async function() {{
                                            try {{
                                                btn.disabled = true;
                                                btn.classList.add('opacity-70');
                                                msg.innerText = 'Initiating...';

                                                // Empty email triggers the discoverable-credentials
                                                // (usernameless) WebAuthn flow — the authenticator
                                                // will present stored passkeys for this RP origin.
                                                var startRes = await fetch('{0}/api/passkeys/start-login', {{
                                                    method: 'POST',
                                                    headers: {{ 'Content-Type': 'application/json' }},
                                                    body: JSON.stringify({{ email: '' }})
                                                }});
                                                if (!startRes.ok) {
                                                    var errText = await startRes.text();
                                                    throw new Error('Failed to start login: ' + errText);
                                                }
                                                var options = await startRes.json();

                                                msg.innerText = 'Please follow browser prompts...';
                                                var startAuthentication = window.SimpleWebAuthnBrowser.startAuthentication;
                                                var credential = await startAuthentication(options);

                                                msg.innerText = 'Verifying...';
                                                var finishRes = await fetch('{0}/api/passkeys/finish-login', {{
                                                    method: 'POST',
                                                    headers: {{ 'Content-Type': 'application/json' }},
                                                    body: JSON.stringify({{ email: '', response: credential }})
                                                }});
                                                if (!finishRes.ok) {
                                                    var errText = await finishRes.text();
                                                    throw new Error('Verification failed: ' + errText);
                                                }

                                                msg.style.color = 'green';
                                                msg.innerText = 'Success! Reloading...';
                                                window.location.reload();
                                            }} catch (e) {{
                                                console.error(e);
                                                msg.innerText = e.message || 'Passkey authentication failed';
                                                btn.disabled = false;
                                                btn.classList.remove('opacity-70');
                                            }}
                                        }});
                                    }}

                                    if (existing && window.SimpleWebAuthnBrowser) {{
                                        bindBtn();
                                    }} else {{
                                        scriptEl.onload = bindBtn;
                                    }}
                                }})();
                                ", api_base_url())}
                                </script>
                            </div>
                        }>
                            <div class="space-y-4">
                                <div class="space-y-2">
                                    <label class="text-sm font-bold text-on-surface">"Email Address"</label>
                                    <input 
                                        type="email" 
                                        placeholder="name@example.com"
                                        class="w-full bg-surface-container border border-outline-variant/30 rounded-xl px-4 py-3 text-on-surface focus:outline-none focus:border-[#004289] focus:ring-1 focus:ring-[#004289] transition-all"
                                        on:input=move |ev| set_email_input.set(event_target_value(&ev))
                                        prop:value=move || email_input.get()
                                    />
                                </div>
                                <button 
                                    class="w-full bg-[#004289] text-white py-4 rounded-xl font-bold text-sm hover:bg-[#003366] transition-all disabled:opacity-50 flex items-center justify-center gap-2"
                                    on:click=handle_magic_link
                                    disabled=move || is_loading.get()
                                >
                                    <Show when=move || is_loading.get() fallback=move || view! { "Send Login Link" }>
                                        <span class="material-symbols-outlined animate-spin text-[18px]">"progress_activity"</span>
                                        "Sending..."
                                    </Show>
                                </button>
                                
                                <Show when=move || !auth_message.get().is_empty()>
                                    <div class=move || format!("text-sm text-center font-medium {}", if is_error.get() { "text-error" } else { "text-emerald-600" })>
                                        {move || auth_message.get()}
                                    </div>
                                </Show>

                                <button 
                                    class="w-full py-4 text-on-surface-variant font-bold text-sm hover:text-[#004289] transition-colors"
                                    on:click=move |_| set_use_email.set(false)
                                >
                                    "Back to Passkey"
                                </button>
                            </div>
                        </Show>
                    </div>
                </div>
            </div>
        </Show>
    }
}
