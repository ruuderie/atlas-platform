use leptos::prelude::*;
use crate::api::developer::*;
use crate::app::GlobalToast;
use uuid::Uuid;
use serde_json::json;

#[component]
pub fn DeveloperConsole() -> impl IntoView {
    let active_network = use_context::<ReadSignal<Option<Uuid>>>().expect("active network context");
    let toast = use_context::<GlobalToast>().expect("GlobalToast not found");

    // We will track the newly created API token string, which will only be displayed once
    let new_api_token = RwSignal::new(None::<String>);

    // Use a trigger to refetch resources
    let refetch_trigger = RwSignal::new(0);

    let api_keys_res = LocalResource::new(move || {
        let n = active_network.get();
        let _ = refetch_trigger.get();
        async move {
            if let Some(tenant) = n {
                list_api_tokens(tenant).await.unwrap_or_default()
            } else {
                vec![]
            }
        }
    });

    let webhooks_res = LocalResource::new(move || {
        let n = active_network.get();
        let _ = refetch_trigger.get();
        async move {
            if let Some(tenant) = n {
                list_webhook_endpoints(tenant).await.unwrap_or_default()
            } else {
                vec![]
            }
        }
    });

    let deliveries_res = LocalResource::new(move || {
        let n = active_network.get();
        let _ = refetch_trigger.get();
        async move {
            if let Some(tenant) = n {
                list_webhook_deliveries(tenant).await.unwrap_or_default()
            } else {
                vec![]
            }
        }
    });

    // Actions
    let create_key_action = Action::new_local(move |_: &()| {
        let t = toast.clone();
        let tenant = active_network.get();
        async move {
            if let Some(tenant_id) = tenant {
                // For now, give blanket scopes or prompt via modal. We just use ["*"]
                let req = CreateApiTokenRequest { scopes: json!(["*"]) };
                match create_api_token(tenant_id, req).await {
                    Ok(resp) => {
                        new_api_token.set(Some(resp.token));
                        t.show_toast("Success", "API Key created.", "success");
                        refetch_trigger.update(|v| *v += 1);
                    }
                    Err(e) => t.show_toast("Error", &format!("Failed: {}", e), "error"),
                }
            } else {
                t.show_toast("Error", "Please select a network first.", "error");
            }
        }
    });

    let new_webhook_url = RwSignal::new(String::new());

    let create_webhook_action = Action::new_local(move |_: &()| {
        let t = toast.clone();
        let tenant = active_network.get();
        let url = new_webhook_url.get();
        async move {
            if url.is_empty() {
                t.show_toast("Error", "URL required", "error");
                return;
            }
            if let Some(tenant_id) = tenant {
                let req = CreateWebhookRequest { target_url: url, subscribed_events: json!(["*"]) };
                match create_webhook_endpoint(tenant_id, req).await {
                    Ok(_) => {
                        new_webhook_url.set(String::new());
                        t.show_toast("Success", "Webhook Endopint created.", "success");
                        refetch_trigger.update(|v| *v += 1);
                    }
                    Err(e) => t.show_toast("Error", &format!("Failed: {}", e), "error"),
                }
            } else {
                t.show_toast("Error", "Please select a network first.", "error");
            }
        }
    });

    let revoke_token_action = Action::new_local(move |token_id: &Uuid| {
        let t = toast.clone();
        let tenant = active_network.get();
        let tid = *token_id;
        async move {
            if let Some(tenant_id) = tenant {
                if let Ok(_) = revoke_api_token(tenant_id, tid).await {
                    t.show_toast("Success", "Token revoked.", "success");
                    refetch_trigger.update(|v| *v += 1);
                }
            }
        }
    });

    let delete_webhook_action = Action::new_local(move |endpoint_id: &Uuid| {
        let t = toast.clone();
        let tenant = active_network.get();
        let eid = *endpoint_id;
        async move {
            if let Some(tenant_id) = tenant {
                if let Ok(_) = delete_webhook_endpoint(tenant_id, eid).await {
                    t.show_toast("Success", "Webhook deleted.", "success");
                    refetch_trigger.update(|v| *v += 1);
                }
            }
        }
    });


    view! {
        <div class="max-w-6xl mx-auto space-y-8 animate-in slide-in-from-bottom-4 duration-500 ease-out fade-in">
            <header class="flex justify-between items-center bg-surface-container border border-outline-variant/10 p-6 rounded-2xl shadow-sm">
                <div>
                    <h1 class="text-3xl font-light tracking-tight text-on-surface mb-2 font-['Inter']">"Developer Console"</h1>
                    <p class="text-on-surface-variant text-sm tracking-wide">"Manage API Keys and Webhooks for the selected network."</p>
                </div>
                <div>
                    <Show when=move || active_network.get().is_none()>
                        <div class="px-4 py-2 bg-error/10 text-error rounded-lg text-sm font-medium border border-error/20">
                            "Please select a Network in the top navigation to continue."
                        </div>
                    </Show>
                </div>
            </header>

            <Show when=move || active_network.get().is_some()>
                <div class="grid grid-cols-1 lg:grid-cols-2 gap-8">
                    // Left Column: API Keys
                    <div class="space-y-6">
                        <section class="p-6 rounded-2xl bg-surface-container border border-outline-variant/10 shadow-sm">
                            <div class="flex justify-between items-center mb-6">
                                <div>
                                    <h2 class="text-lg font-semibold text-on-surface">"API Keys"</h2>
                                    <p class="text-sm text-on-surface-variant">"Keys provide blanket access to your tenant data."</p>
                                </div>
                                <button on:click=move |_| { create_key_action.dispatch(()); } class="btn-primary-gradient px-4 py-2 rounded-lg text-sm font-bold text-on-primary shadow-lg shadow-primary/20 hover:scale-105 transition-all">
                                    "Generate Root Key"
                                </button>
                            </div>

                            <Show when=move || new_api_token.get().is_some()>
                                <div class="mb-6 p-4 rounded-xl bg-primary-container/30 border border-primary/40 text-on-surface">
                                    <p class="text-sm font-bold text-primary mb-2">"Key generated! Please copy it now, it will not be shown again:"</p>
                                    <div class="flex items-center gap-2">
                                        <input type="text" readonly class="w-full bg-surface-container-highest border border-outline/20 p-2 text-sm rounded font-mono" value=move || new_api_token.get().unwrap_or_default() />
                                        <button class="bg-surface-bright px-3 py-2 rounded hover:bg-surface-highest transition-colors flex items-center" on:click=move |_| new_api_token.set(None)>
                                            "Done"
                                        </button>
                                    </div>
                                </div>
                            </Show>

                            <Suspense fallback=move || view! { <div class="text-sm text-on-surface-variant">"Loading keys..."</div> }>
                                {move || {
                                    if let Some(keys) = api_keys_res.get() {
                                        if keys.is_empty() {
                                            view! { <div class="text-sm border border-dashed border-outline-variant/30 rounded-lg p-6 text-center text-on-surface-variant">"No API keys created yet."</div> }.into_any()
                                        } else {
                                            view! {
                                                <div class="bg-surface border border-outline-variant/10 overflow-hidden sm:rounded-md">
                                                    <ul role="list" class="divide-y divide-outline-variant/10">
                                                        <For each=move || keys.clone() key=|k| k.id children=move |key| {
                                                            let kid = key.id;
                                                            view! {
                                                                <li class="px-4 py-4 sm:px-6 flex justify-between items-center hover:bg-surface-bright/5">
                                                                    <div>
                                                                        <div class="flex items-center gap-2">
                                                                            <span class="text-sm font-medium text-on-surface">"Token"</span>
                                                                            <span class="px-2 py-0.5 rounded text-[10px] uppercase font-bold bg-secondary-container text-on-secondary-container">"Root"</span>
                                                                        </div>
                                                                        <div class="text-xs text-on-surface-variant mt-1 font-mono">"Hash: " {key.token_hash.chars().take(15).collect::<String>()} "..."</div>
                                                                    </div>
                                                                    <button on:click=move |_| { revoke_token_action.dispatch(kid); } class="text-error hover:text-error-container text-sm font-medium transition-colors">
                                                                        "Revoke"
                                                                    </button>
                                                                </li>
                                                            }
                                                        }/>
                                                    </ul>
                                                </div>
                                            }.into_any()
                                        }
                                    } else {
                                        view! { <div></div> }.into_any()
                                    }
                                }}
                            </Suspense>
                        </section>
                    </div>

                    // Right Column: Webhooks
                    <div class="space-y-6">
                        <section class="p-6 rounded-2xl bg-surface-container border border-outline-variant/10 shadow-sm">
                            <h2 class="text-lg font-semibold text-on-surface mb-2">"Webhooks"</h2>
                            <p class="text-sm text-on-surface-variant mb-6">"Notify your remote systems when events occur."</p>
                            
                            <form on:submit=move |e| { e.prevent_default(); create_webhook_action.dispatch(()); } class="flex items-end gap-4 mb-6">
                                <div class="flex-1">
                                    <label class="block text-xs font-medium text-on-surface-variant mb-1 uppercase tracking-wider">"Target URL"</label>
                                    <input type="url" 
                                        class="w-full bg-surface-container-highest border border-outline/20 text-on-surface text-sm rounded-lg focus:ring-primary focus:border-primary block p-3"
                                        placeholder="https://api.yourdomain.com/webhook"
                                        prop:value=move || new_webhook_url.get()
                                        on:input=move |ev| new_webhook_url.set(event_target_value(&ev))
                                        required
                                    />
                                </div>
                                <button type="submit" class="bg-surface-container-high border border-outline/20 px-4 py-3 rounded-lg text-sm font-bold text-on-surface shadow-sm hover:bg-surface-bright/20 transition-all">
                                    "Add Webhook"
                                </button>
                            </form>

                            <Suspense fallback=move || view! { <div class="text-sm text-on-surface-variant">"Loading webhooks..."</div> }>
                                {move || {
                                    if let Some(webhooks) = webhooks_res.get() {
                                        if webhooks.is_empty() {
                                            view! { <div class="text-sm border border-dashed border-outline-variant/30 rounded-lg p-6 text-center text-on-surface-variant">"No webhooks configured."</div> }.into_any()
                                        } else {
                                            view! {
                                                <div class="bg-surface border border-outline-variant/10 overflow-hidden sm:rounded-md">
                                                    <ul role="list" class="divide-y divide-outline-variant/10 text-sm">
                                                        <For each=move || webhooks.clone() key=|w| w.id children=move |hook| {
                                                            let hid = hook.id;
                                                            view! {
                                                                <li class="px-4 py-4 sm:px-6 flex flex-col gap-2 hover:bg-surface-bright/5">
                                                                    <div class="flex justify-between items-start">
                                                                        <div class="font-medium text-on-surface break-all pr-4">{hook.target_url}</div>
                                                                        <button on:click=move |_| { delete_webhook_action.dispatch(hid); } class="text-error hover:text-error-container font-medium shrink-0">"Delete"</button>
                                                                    </div>
                                                                    <div class="flex justify-between items-center">
                                                                        <div class="text-xs text-on-surface-variant font-mono bg-surface-container-highest px-2 py-1 rounded">"Secret: " {hook.secret_key}</div>
                                                                        <div>
                                                                            {if hook.is_active {
                                                                                view!{ <span class="px-2 py-0.5 rounded text-[10px] uppercase font-bold bg-success/20 text-success">"Active"</span> }.into_any()
                                                                            } else {
                                                                                view!{ <span class="px-2 py-0.5 rounded text-[10px] uppercase font-bold bg-error/20 text-error">"Disabled"</span> }.into_any()
                                                                            }}
                                                                        </div>
                                                                    </div>
                                                                </li>
                                                            }
                                                        }/>
                                                    </ul>
                                                </div>
                                            }.into_any()
                                        }
                                    } else {
                                        view! { <div></div> }.into_any()
                                    }
                                }}
                            </Suspense>
                        </section>
                    </div>
                </div>

                // Bottom Log View
                <div class="mt-8">
                    <section class="p-6 rounded-2xl bg-surface border border-outline-variant/10 shadow-sm">
                        <div class="flex justify-between items-center mb-4">
                            <h2 class="text-lg font-semibold text-on-surface">"Delivery Logs"</h2>
                            <button on:click=move |_| refetch_trigger.update(|v| *v += 1) class="text-sm font-medium hover:text-primary transition-colors flex items-center gap-1">
                                <span class="material-symbols-outlined text-sm">"refresh"</span>
                                "Refresh"
                            </button>
                        </div>
                        <Suspense fallback=move || view! { <div class="text-sm text-on-surface-variant">"Loading tracking..."</div> }>
                            {move || {
                                if let Some(logs) = deliveries_res.get() {
                                    if logs.is_empty() {
                                        view! { <div class="text-sm border border-dashed border-outline-variant/30 rounded-lg p-6 text-center text-on-surface-variant">"No deliveries logged yet."</div> }.into_any()
                                    } else {
                                        view! {
                                            <div class="overflow-x-auto w-full">
                                                <table class="w-full text-left text-sm whitespace-nowrap">
                                                    <thead class="bg-surface-container-highest text-on-surface-variant uppercase text-xs tracking-wider">
                                                        <tr>
                                                            <th class="px-4 py-3 font-medium rounded-tl-lg">"Status"</th>
                                                            <th class="px-4 py-3 font-medium">"Event"</th>
                                                            <th class="px-4 py-3 font-medium">"Attempts"</th>
                                                            <th class="px-4 py-3 font-medium">"HTTP Status"</th>
                                                            <th class="px-4 py-3 font-medium rounded-tr-lg w-full">"Response"</th>
                                                        </tr>
                                                    </thead>
                                                    <tbody class="divide-y divide-outline-variant/10">
                                                        <For each=move || logs.clone() key=|l| l.id children=move |log| {
                                                            let status_color = match log.status.as_str() {
                                                                "sent" => "text-success",
                                                                "failed" => "text-error",
                                                                _ => "text-on-surface-variant",
                                                            };
                                                            view! {
                                                                <tr class="hover:bg-surface-bright/5">
                                                                    <td class=format!("px-4 py-3 font-semibold uppercase text-[10px] {}", status_color)>{log.status}</td>
                                                                    <td class="px-4 py-3 text-on-surface font-mono">{log.event_type}</td>
                                                                    <td class="px-4 py-3 text-on-surface-variant">{log.attempts}</td>
                                                                    <td class="px-4 py-3 text-on-surface">{log.response_status.map(|s| s.to_string()).unwrap_or_else(|| "-".to_string())}</td>
                                                                    <td class="px-4 py-3 text-on-surface-variant truncate max-w-xs">{log.response_body.unwrap_or_else(|| "-".to_string())}</td>
                                                                </tr>
                                                            }
                                                        }/>
                                                    </tbody>
                                                </table>
                                            </div>
                                        }.into_any()
                                    }
                                } else {
                                    view! { <div></div> }.into_any()
                                }
                            }}
                        </Suspense>
                    </section>
                </div>
            </Show>
        </div>
    }
}
