use leptos::prelude::*;
use reqwest::Client;
use serde_json::json;
use crate::auth::passkey::start_authentication;

#[component]
pub fn PasskeyLoginButton(
    #[prop(into)] api_base_url: String,
    email: RwSignal<String>,
    #[prop(into)] on_success: Callback<String>,
    #[prop(into)] on_error: Callback<String>,
) -> impl IntoView {
    let is_submitting = RwSignal::new(false);
    
    let handle_passkey_login = move |ev: leptos::ev::MouseEvent| {
        ev.prevent_default();
        if is_submitting.get() { return; }
        
        let email_val = email.get();
        if email_val.is_empty() {
            on_error.run("Please enter your email to use passkey login.".to_string());
            return;
        }

        is_submitting.set(true);
        let api_url = api_base_url.clone();
        
        leptos::task::spawn_local(async move {
            let client = Client::new();
            
            // 1. Start Login
            let start_url = format!("{}/start-login", api_url);
            let start_payload = json!({ "email": email_val });
            
            let start_res = match client.post(&start_url).json(&start_payload).send().await {
                Ok(res) if res.status().is_success() => res,
                Ok(res) => {
                    let text = res.text().await.unwrap_or_default();
                    on_error.run(format!("No passkeys found: {}", text));
                    is_submitting.set(false);
                    return;
                }
                Err(_) => {
                    on_error.run("Network error communicating with server.".to_string());
                    is_submitting.set(false);
                    return;
                }
            };
            
            let options = match start_res.json::<serde_json::Value>().await {
                Ok(opt) => opt,
                Err(_) => {
                    on_error.run("Invalid server response.".to_string());
                    is_submitting.set(false);
                    return;
                }
            };

            // 2. Browser WebAuthn API
            let assertion = match start_authentication(&options).await {
                Ok(ass) => ass,
                Err(e) => {
                    on_error.run(e);
                    is_submitting.set(false);
                    return;
                }
            };

            // 3. Finish Login
            let finish_url = format!("{}/finish-login", api_url);
            let finish_payload = json!({
                "email": email_val,
                "response": assertion
            });
            
            match client.post(&finish_url).json(&finish_payload).send().await {
                Ok(res) if res.status().is_success() => {
                    if let Ok(json) = res.json::<serde_json::Value>().await {
                        if let Some(token) = json.get("token").and_then(|t| t.as_str()) {
                            on_success.run(token.to_string());
                        } else {
                            on_error.run("Invalid token received.".to_string());
                        }
                    }
                }
                Ok(res) => {
                    let text = res.text().await.unwrap_or_default();
                    on_error.run(format!("Passkey auth failed: {}", text));
                }
                Err(_) => on_error.run("Network error during verification.".to_string()),
            }
            
            is_submitting.set(false);
        });
    };

    view! {
        <button 
            type="button" 
            on:click=handle_passkey_login
            disabled=move || is_submitting.get()
            class="group relative w-full flex justify-center items-center py-3.5 px-4 mb-4 border border-outline-variant/60 text-sm font-bold rounded-xl text-on-surface bg-surface-container-lowest hover:bg-surface-container-low focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-primary transition-all disabled:opacity-70 shadow-sm"
        >
            <span class="material-symbols-outlined mr-3 text-primary">"fingerprint"</span>
            {move || if is_submitting.get() { "Authenticating..." } else { "Sign in with Passkey" }}
        </button>
    }
}
