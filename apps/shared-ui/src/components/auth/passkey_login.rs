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

        is_submitting.set(true);
        let api_url = api_base_url.clone();
        
        leptos::task::spawn_local(async move {
            let client = Client::new();
            
            // 1. Start Login
            // credentials_include is required so the browser sends the HttpOnly
            // SameSite=Strict session cookie on this cross-origin fetch. Without it
            // the backend auth middleware cannot identify the caller and the WebAuthn
            // origin lookup fails, surfacing as "Network error".
            let start_url = format!("{}/start-login", api_url);
            let start_payload = json!({ "email": email_val });
            
            #[cfg(target_arch = "wasm32")]
            let start_result = client
                .post(&start_url)
                .json(&start_payload)
                .fetch_credentials_include()
                .send()
                .await;
            #[cfg(not(target_arch = "wasm32"))]
            let start_result = client.post(&start_url).json(&start_payload).send().await;

            let start_res = match start_result {
                Ok(res) if res.status().is_success() => res,
                Ok(res) => {
                    let text = res.text().await.unwrap_or_default();
                    leptos::logging::warn!("Passkey start-login failed: {}", text);
                    on_error.run("No passkeys found for this account. Try signing in with a magic link.".to_string());
                    is_submitting.set(false);
                    return;
                }
                Err(_) => {
                    on_error.run("Network error communicating with server.".to_string());
                    is_submitting.set(false);
                    return;
                }
            };
            
            let session_id_opt = start_res.headers()
                .get("x-passkey-session")
                .and_then(|v| v.to_str().ok())
                .map(|s| s.to_string());

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

            // 3. Finish Login — also needs credentials_include so the
            // passkey_session cookie set by start-login is echoed back for
            // state correlation on the server.
            let finish_url = format!("{}/finish-login", api_url);
            let finish_payload = json!({
                "email": email_val,
                "response": assertion
            });

            #[cfg(target_arch = "wasm32")]
            let mut req = client
                .post(&finish_url)
                .json(&finish_payload)
                .fetch_credentials_include();
            #[cfg(not(target_arch = "wasm32"))]
            let mut req = client.post(&finish_url).json(&finish_payload);

            if let Some(ref sess_id) = session_id_opt {
                req = req.header("x-passkey-session", sess_id);
            }
            let finish_result = req.send().await;

            match finish_result {
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
                    let status = res.status();
                    let text = res.text().await.unwrap_or_default();
                    leptos::logging::warn!("Passkey finish-login failed: {} - {}", status, text);
                    let err_msg = if text.trim().is_empty() {
                        format!("Passkey verification failed (HTTP {}). Please try again.", status)
                    } else {
                        format!("Passkey verification failed: {}", text)
                    };
                    on_error.run(err_msg);
                }
                Err(_) => on_error.run("Network error during verification. Please check your connection.".to_string()),
            }
            
            is_submitting.set(false);
        });
    };

    view! {
        <button
            type="button"
            on:click=handle_passkey_login
            disabled=move || is_submitting.get()
            style="
                display: flex;
                align-items: center;
                justify-content: center;
                gap: 10px;
                width: 100%;
                background: #1B365D;
                color: #faf9f5;
                border: none;
                border-radius: 6px;
                padding: 12px 20px;
                font-size: 14px;
                font-weight: 500;
                cursor: pointer;
                transition: background 0.15s, opacity 0.15s;
                text-align: center;
                opacity: 1;
            "
        >
            // Fingerprint icon inline SVG — no external icon font dependency
            {move || if is_submitting.get() {
                view! {
                    <span>
                        "Authenticating…"
                    </span>
                }.into_any()
            } else {
                view! {
                    <span style="display: flex; align-items: center; gap: 8px;">
                        <svg
                            xmlns="http://www.w3.org/2000/svg"
                            width="16" height="16"
                            viewBox="0 0 24 24"
                            fill="none"
                            stroke="currentColor"
                            stroke-width="1.8"
                            stroke-linecap="round"
                            stroke-linejoin="round"
                            aria-hidden="true"
                        >
                            <path d="M12 10a2 2 0 0 0-2 2c0 1.02-.1 2.51-.26 4"/>
                            <path d="M14 13.12c0 2.38 0 6.38-1 8.88"/>
                            <path d="M17.29 21.02c.12-.6.43-2.3.5-3.02"/>
                            <path d="M2 12a10 10 0 0 1 18-6"/>
                            <path d="M2 17c1 .5 2.5 1 4 1"/>
                            <path d="M21.8 16c.2-2 .131-5.354 0-6"/>
                            <path d="M5 19.5C5.5 18 6 15 6 12a6 6 0 0 1 .34-2"/>
                            <path d="M8.65 22c.21-.66.45-1.32.57-2"/>
                            <path d="M9 6.8a6 6 0 0 1 9 5.2v2"/>
                        </svg>
                        "Sign in with Passkey"
                    </span>
                }.into_any()
            }}
        </button>
    }
}
