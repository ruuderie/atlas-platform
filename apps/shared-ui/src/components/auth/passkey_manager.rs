use leptos::prelude::*;
use reqwest::Client;
use crate::auth::passkey::start_registration;
use crate::auth::atlas_auth::server_fns::{
    start_passkey_registration, finish_passkey_registration,
    get_passkeys, revoke_passkey, PasskeyInfo
};

#[component]
pub fn ManagePasskeys(
    #[prop(into)] api_base_url: Signal<String>,
    #[prop(into)] auth_token: String,
    #[prop(default = false)] auto_register: bool,
) -> impl IntoView {
    let is_submitting = RwSignal::new(false);
    let message = RwSignal::new(String::new());
    let is_error = RwSignal::new(false);
    
    let api_url_sig = api_base_url;
    let auth_token_str = auth_token;

    let auth_token_for_list = auth_token_str.clone();
    let auth_token_for_revoke = auth_token_str.clone();
    let auth_token_for_register = auth_token_str;

    let (refetch_trigger, set_refetch_trigger) = signal(0);

    let passkeys_res = LocalResource::new(move || {
        let _ = refetch_trigger.get();
        let api_url = api_url_sig.get();
        let auth_token_val = auth_token_for_list.clone();
        async move {
            if auth_token_val.is_empty() {
                // Use Server Function
                leptos::logging::log!("ManagePasskeys: Fetching passkeys via Server Function");
                match get_passkeys().await {
                    Ok(pks) => pks,
                    Err(e) => {
                        leptos::logging::error!("ManagePasskeys: Server Function get_passkeys failed: {:?}", e);
                        Vec::new()
                    }
                }
            } else {
                // Direct CSR HTTP request flow
                leptos::logging::log!("ManagePasskeys: Fetching passkeys via CSR HTTP request to {}", api_url);
                let client = Client::new();
                #[cfg(target_arch = "wasm32")]
                let req = client.get(&api_url).fetch_credentials_include();
                #[cfg(not(target_arch = "wasm32"))]
                let req = client.get(&api_url);

                match req.send().await {
                    Ok(res) => {
                        let status = res.status();
                        if status.is_success() {
                            let text = match res.text().await {
                                Ok(t) => t,
                                Err(e) => {
                                    leptos::logging::error!("ManagePasskeys: Failed to get response text: {:?}", e);
                                    return Vec::new();
                                }
                            };
                            match serde_json::from_str::<Vec<PasskeyInfo>>(&text) {
                                Ok(pks) => {
                                    leptos::logging::log!("ManagePasskeys: Successfully deserialized {} passkeys", pks.len());
                                    pks
                                }
                                Err(e) => {
                                    leptos::logging::error!(
                                        "ManagePasskeys: JSON deserialization failed: {:?}. Response body was: {}",
                                        e,
                                        text
                                    );
                                    Vec::new()
                                }
                            }
                        } else {
                            let err_body = res.text().await.unwrap_or_default();
                            leptos::logging::error!(
                                "ManagePasskeys: HTTP error status {} fetching passkeys: {}",
                                status,
                                err_body
                            );
                            Vec::new()
                        }
                    }
                    Err(e) => {
                        leptos::logging::error!("ManagePasskeys: Network request failed to send: {:?}", e);
                        Vec::new()
                    }
                }
            }
        }
    });

    let revoke_action = Action::new_local(move |id: &uuid::Uuid| {
        let id = *id;
        let api_url = api_url_sig.get();
        let auth_token_val = auth_token_for_revoke.clone();
        let message = message.clone();
        let is_error = is_error.clone();
        let set_refetch_trigger = set_refetch_trigger.clone();
        async move {
            message.set("Revoking passkey...".to_string());
            is_error.set(false);
            
            let success = if auth_token_val.is_empty() {
                revoke_passkey(id).await.is_ok()
            } else {
                let delete_url = format!("{}/{}", api_url, id);
                let client = Client::new();
                #[cfg(target_arch = "wasm32")]
                let req = client.delete(&delete_url).fetch_credentials_include();
                #[cfg(not(target_arch = "wasm32"))]
                let req = client.delete(&delete_url);

                match req.send().await {
                    Ok(res) if res.status().is_success() => true,
                    _ => false,
                }
            };

            if success {
                message.set("Passkey revoked successfully.".to_string());
                set_refetch_trigger.update(|n| *n += 1);
            } else {
                message.set("Failed to revoke passkey.".to_string());
                is_error.set(true);
            }
        }
    });

    let do_register = Action::new_local(move |_: &()| {
        let is_submitting = is_submitting.clone();
        let message = message.clone();
        let is_error = is_error.clone();
        let api_url = api_url_sig.get();
        let auth_token_val = auth_token_for_register.clone();
        let set_refetch_trigger = set_refetch_trigger.clone();

        async move {
            if is_submitting.get() { return; }
        
            is_submitting.set(true);
            is_error.set(false);

            if auth_token_val.is_empty() {
                // Same-Origin Server Function Proxy Flow
                message.set("Initiating registration...".to_string());
                let options = match start_passkey_registration().await {
                    Ok(opt) => opt,
                    Err(e) => {
                        message.set(format!("Failed to start registration: {}", e));
                        is_error.set(true);
                        is_submitting.set(false);
                        return;
                    }
                };

                // Browser WebAuthn API
                let credential = match start_registration(&options).await {
                    Ok(cred) => cred,
                    Err(e) => {
                        message.set(e);
                        is_error.set(true);
                        is_submitting.set(false);
                        return;
                    }
                };

                message.set("Verifying credential...".to_string());
                match finish_passkey_registration(credential).await {
                    Ok(_) => {
                        message.set("Passkey registered successfully!".to_string());
                        is_error.set(false);
                        set_refetch_trigger.update(|n| *n += 1);
                    }
                    Err(e) => {
                        message.set(format!("Registration failed: {}", e));
                        is_error.set(true);
                    }
                }
            } else {
                // Direct CSR HTTP request flow
                message.set("Initiating registration...".to_string());
                let client = Client::new();

                // 1. Start Registration
                let start_url = format!("{}/start-register", api_url);

                #[cfg(target_arch = "wasm32")]
                let start_res = match client.post(&start_url)
                    .fetch_credentials_include()
                    .send().await {
                    Ok(res) if res.status().is_success() => res,
                    Ok(res) => {
                        let text: String = res.text().await.unwrap_or_default();
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
                #[cfg(not(target_arch = "wasm32"))]
                let start_res = match client.post(&start_url).send().await {
                    Ok(res) if res.status().is_success() => res,
                    Ok(res) => {
                        let text: String = res.text().await.unwrap_or_default();
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

                // Finish Registration
                let finish_url = format!("{}/finish-register", api_url);

                #[cfg(target_arch = "wasm32")]
                let finish_result = client.post(&finish_url)
                    .fetch_credentials_include()
                    .json(&credential).send().await;
                #[cfg(not(target_arch = "wasm32"))]
                let finish_result = client.post(&finish_url)
                    .json(&credential).send().await;

                match finish_result {
                    Ok(res) if res.status().is_success() => {
                        message.set("Passkey registered successfully!".to_string());
                        is_error.set(false);
                        set_refetch_trigger.update(|n| *n += 1);
                    }
                    Ok(res) => {
                        let text: String = res.text().await.unwrap_or_default();
                        message.set(format!("Registration failed: {}", text));
                        is_error.set(true);
                    }
                    Err(_) => {
                        message.set("Network error during verification.".to_string());
                        is_error.set(true);
                    }
                }
            }
            
            is_submitting.set(false);
        }
    });

    // Auto trigger if requested
    Effect::new(move |_| {
        if auto_register {
            do_register.dispatch(());
        }
    });

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
                on:click=move |_| { do_register.dispatch(()); }
                disabled=move || is_submitting.get()
                class="inline-flex justify-center items-center py-2.5 px-4 font-bold rounded-xl bg-primary text-on-primary hover:bg-primary-dim focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-primary transition-all disabled:opacity-70 shadow-[0_0_15px_rgba(123,208,255,0.15)] hover:shadow-[0_0_20px_rgba(123,208,255,0.3)]"
            >
                <span class="material-symbols-outlined mr-2">"add_circle"</span>
                {move || if is_submitting.get() { "Registering..." } else { "Add a Passkey" }}
            </button>

            // Stored Passkeys List
            <div class="mt-6 border-t border-outline-variant/20 pt-6">
                <h4 class="text-sm font-bold text-on-surface mb-4">"Registered Passkeys"</h4>
                {move || {
                    let pks = passkeys_res.get().unwrap_or_default();
                    if pks.is_empty() {
                        view! {
                            <p class="text-xs text-on-surface-variant italic py-2">
                                "No passkeys registered yet."
                            </p>
                        }.into_any()
                    } else {
                        view! {
                            <div class="space-y-3">
                                <For
                                    each=move || pks.clone()
                                    key=|pk| pk.id
                                    children=move |pk| {
                                        let pk_id = pk.id;
                                        let pk_name = pk.name.clone();
                                        let sign_count = pk.sign_count;
                                        
                                        // Format date: YYYY-MM-DD
                                        let date_str = pk.created_at.format("%Y-%m-%d %H:%M").to_string();
                                        let last_used = pk.last_used_at
                                            .map(|dt| dt.format("%Y-%m-%d %H:%M").to_string())
                                            .unwrap_or_else(|| "Never".to_string());
                                            
                                        view! {
                                            <div class="flex items-center justify-between p-4 bg-surface border border-outline-variant/30 rounded-xl hover:border-outline-variant/60 transition-all">
                                                <div class="flex items-center gap-3">
                                                    <span class="material-symbols-outlined text-2xl text-primary">"fingerprint"</span>
                                                    <div>
                                                        <div class="text-sm font-bold text-on-surface">{pk_name}</div>
                                                        <div class="text-[10px] text-on-surface-variant/80 mt-1 flex flex-wrap gap-x-3 gap-y-1">
                                                            <span>"Registered: " <strong class="text-on-surface">{date_str}</strong></span>
                                                            <span>"Used: " <strong class="text-on-surface">{sign_count} " times"</strong></span>
                                                            <span>"Last Active: " <strong class="text-on-surface">{last_used}</strong></span>
                                                        </div>
                                                    </div>
                                                </div>
                                                <button
                                                    type="button"
                                                    on:click=move |_| { revoke_action.dispatch(pk_id); }
                                                    class="inline-flex items-center justify-center py-1.5 px-3 font-semibold text-xs rounded-lg border border-error/40 text-error hover:bg-error/10 focus:outline-none transition-all"
                                                >
                                                    <span class="material-symbols-outlined text-sm mr-1">"delete"</span>
                                                    "Revoke"
                                                </button>
                                            </div>
                                        }
                                    }
                                />
                            </div>
                        }.into_any()
                    }
                }}
            </div>
        </div>
    }
}
