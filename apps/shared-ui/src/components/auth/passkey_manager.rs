use leptos::prelude::*;
use reqwest::Client;
use serde_json::json;
use crate::auth::passkey::start_registration;

#[component]
pub fn ManagePasskeys(
    #[prop(into)] api_base_url: Signal<String>,
    #[prop(into)] auth_token: String,
) -> impl IntoView {
    let is_submitting = RwSignal::new(false);
    let message = RwSignal::new(String::new());
    let is_error = RwSignal::new(false);
    
    let handle_register = move |_| {
        if is_submitting.get() { return; }
        
        is_submitting.set(true);
        message.set("Initiating registration...".to_string());
        is_error.set(false);
        
        let api_url = api_base_url.get();
        let token = auth_token.clone();
        
        leptos::task::spawn_local(async move {
            let client = Client::new();
            
            // 1. Start Registration
            let start_url = format!("{}/start-register", api_url);
            
            let start_res = match client.post(&start_url)
                .header("Authorization", format!("Bearer {}", token))
                .send().await {
                Ok(res) if res.status().is_success() => res,
                Ok(res) => {
                    let text = res.text().await.unwrap_or_default();
                    message.set(format!("Failed to start registration: {}", text));
                    is_error.set(true);
                    is_submitting.set(false);
                    return;
                }
                Err(_) => {
                    message.set("Network error communicating with server.".to_string());
                    is_error.set(true);
                    is_submitting.set(false);
                    return;
                }
            };
            
            let options = match start_res.json::<serde_json::Value>().await {
                Ok(opt) => opt,
                Err(_) => {
                    message.set("Invalid server response.".to_string());
                    is_error.set(true);
                    is_submitting.set(false);
                    return;
                }
            };

            // 2. Browser WebAuthn API
            let credential = match start_registration(&options).await {
                Ok(cred) => cred,
                Err(e) => {
                    message.set(e);
                    is_error.set(true);
                    is_submitting.set(false);
                    return;
                }
            };

            // 3. Finish Registration
            let finish_url = format!("{}/finish-register", api_url);
            
            match client.post(&finish_url)
                .header("Authorization", format!("Bearer {}", token))
                .json(&credential).send().await {
                Ok(res) if res.status().is_success() => {
                    message.set("Passkey registered successfully!".to_string());
                    is_error.set(false);
                }
                Ok(res) => {
                    let text = res.text().await.unwrap_or_default();
                    message.set(format!("Registration failed: {}", text));
                    is_error.set(true);
                }
                Err(_) => {
                    message.set("Network error during verification.".to_string());
                    is_error.set(true);
                }
            }
            
            is_submitting.set(false);
        });
    };

    view! {
        <div class="bg-surface-container-high p-6 rounded-2xl shadow-sm border border-outline-variant/30 mt-6">
            <h3 class="text-xl font-bold text-on-surface mb-2">"Passkeys"</h3>
            <p class="text-sm text-on-surface-variant mb-6">
                "Use a passkey (Face ID, Touch ID, or a hardware key) to sign in securely without a password."
            </p>
            
            {move || if !message.get().is_empty() {
                let color_class = if is_error.get() { "bg-error-container/30 text-error border-error/20" } else { "bg-tertiary/10 text-tertiary border-tertiary/20" };
                view! {
                    <div class=format!("px-4 py-3 rounded-xl text-sm font-medium mb-4 border {}", color_class)>
                        {message.get()}
                    </div>
                }.into_any()
            } else { view! { <span/> }.into_any() }}
            
            <button 
                type="button" 
                on:click=handle_register
                disabled=move || is_submitting.get()
                class="inline-flex justify-center items-center py-2.5 px-4 font-bold rounded-xl bg-primary text-on-primary hover:bg-primary-dim focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-primary transition-all disabled:opacity-70 shadow-[0_0_15px_rgba(123,208,255,0.15)] hover:shadow-[0_0_20px_rgba(123,208,255,0.3)]"
            >
                <span class="material-symbols-outlined mr-2">"add_circle"</span>
                {move || if is_submitting.get() { "Registering..." } else { "Add a Passkey" }}
            </button>
        </div>
    }
}
