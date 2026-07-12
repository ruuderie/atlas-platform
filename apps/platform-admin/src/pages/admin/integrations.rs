use leptos::prelude::*;
use uuid::Uuid;

use crate::api::developer::*;
use crate::app::GlobalToast;
use crate::components::active_network_picker::ActiveNetworkPicker;

#[component]
pub fn Integrations() -> impl IntoView {
    let active_network = use_context::<ReadSignal<Option<Uuid>>>().expect("active network context");
    let toast = use_context::<GlobalToast>().expect("GlobalToast not found");

    let active_tab = RwSignal::new("services".to_string());
    let selected_delivery_id = RwSignal::new(None::<String>);
    let refetch_trigger = RwSignal::new(0);
    let panel_tab = RwSignal::new("payload".to_string());

    let show_key_modal = RwSignal::new(false);
    let show_webhook_modal = RwSignal::new(false);
    let show_revoke_modal = RwSignal::new(None::<String>);
    let show_delete_hook = RwSignal::new(None::<Uuid>);
    let new_key_name = RwSignal::new(String::new());
    let new_key_scope = RwSignal::new("read:leads".to_string());
    let generated_secret_key = RwSignal::new(None::<String>);
    let new_hook_url = RwSignal::new(String::new());
    let new_hook_event = RwSignal::new("lead.created".to_string());

    let deliveries_res = LocalResource::new(move || {
        let n = active_network.get();
        let _ = refetch_trigger.get();
        async move {
            if let Some(tenant_id) = n {
                list_webhook_deliveries(tenant_id).await.unwrap_or_default()
            } else {
                vec![]
            }
        }
    });

    let api_keys_res = LocalResource::new(move || {
        let n = active_network.get();
        let _ = refetch_trigger.get();
        async move {
            if let Some(tenant) = n {
                list_api_tokens(tenant).await.ok()
            } else {
                None
            }
        }
    });

    let endpoints_res = LocalResource::new(move || {
        let n = active_network.get();
        let _ = refetch_trigger.get();
        async move {
            if let Some(tenant) = n {
                list_webhook_endpoints(tenant).await.ok()
            } else {
                None
            }
        }
    });

    let submit_key_generation = move |_| {
        let name = new_key_name.get();
        if name.trim().is_empty() {
            toast.show_toast("Error", "Client name is required.", "error");
            return;
        }
        let tenant = active_network.get();
        let scope = new_key_scope.get();
        let t_toast = toast.clone();
        leptos::task::spawn_local(async move {
            let Some(tenant_id) = tenant else {
                t_toast.show_toast("Error", "Select a tenant network first.", "error");
                return;
            };
            let scopes: Vec<&str> = scope.split(',').map(|s| s.trim()).filter(|s| !s.is_empty()).collect();
            let req = CreateApiTokenRequest {
                name: Some(name.trim().to_string()),
                scopes: serde_json::json!(scopes),
            };
            match create_api_token(tenant_id, req).await {
                Ok(resp) => {
                    generated_secret_key.set(Some(resp.secret));
                    t_toast.show_toast("Success", "API credential created.", "success");
                    refetch_trigger.update(|v| *v += 1);
                }
                Err(e) => t_toast.show_toast("Error", &format!("Failed: {}", e), "error"),
            }
        });
    };

    let submit_webhook = move |_| {
        let url = new_hook_url.get();
        if url.trim().is_empty() {
            toast.show_toast("Error", "Target URL is required.", "error");
            return;
        }
        let tenant = active_network.get();
        let event = new_hook_event.get();
        let t_toast = toast.clone();
        leptos::task::spawn_local(async move {
            let Some(tenant_id) = tenant else {
                t_toast.show_toast("Error", "Select a tenant network first.", "error");
                return;
            };
            let req = CreateWebhookRequest {
                target_url: url.trim().to_string(),
                subscribed_events: serde_json::json!([event]),
            };
            match create_webhook_endpoint(tenant_id, req).await {
                Ok(_) => {
                    show_webhook_modal.set(false);
                    new_hook_url.set(String::new());
                    t_toast.show_toast("Success", "Webhook endpoint registered.", "success");
                    refetch_trigger.update(|v| *v += 1);
                }
                Err(e) => t_toast.show_toast("Error", &format!("Failed: {}", e), "error"),
            }
        });
    };

    let handle_revoke_key = move |id: String| {
        let tenant = active_network.get();
        let t_toast = toast.clone();
        leptos::task::spawn_local(async move {
            if let Some(tenant_id) = tenant {
                if let Ok(parsed_id) = Uuid::parse_str(&id) {
                    match revoke_api_token(tenant_id, parsed_id).await {
                        Ok(_) => {
                            t_toast.show_toast("Success", "Credential revoked.", "success");
                            refetch_trigger.update(|v| *v += 1);
                        }
                        Err(e) => t_toast.show_toast("Error", &format!("Failed: {}", e), "error"),
                    }
                }
            }
            show_revoke_modal.set(None);
        });
    };

    let handle_delete_hook = move |id: Uuid| {
        let tenant = active_network.get();
        let t_toast = toast.clone();
        leptos::task::spawn_local(async move {
            if let Some(tenant_id) = tenant {
                match delete_webhook_endpoint(tenant_id, id).await {
                    Ok(_) => {
                        t_toast.show_toast("Success", "Endpoint deleted.", "success");
                        refetch_trigger.update(|v| *v += 1);
                    }
                    Err(e) => t_toast.show_toast("Error", &format!("Failed: {}", e), "error"),
                }
            }
            show_delete_hook.set(None);
        });
    };

    let selected_delivery = Signal::derive(move || {
        let sid = selected_delivery_id.get();
        sid.and_then(|id| {
            deliveries_res
                .get()
                .and_then(|deliveries| deliveries.into_iter().find(|w| w.id.to_string() == id))
        })
    });

    view! {
        <div class="main-canvas">
            <div class="page-header">
                <div>
                    <h1 class="page-title">"Integrations & Webhooks"</h1>
                    <p class="page-subtitle">"Tenant API credentials · outbound webhooks · delivery ledger · platform service config"</p>
                </div>
                <div class="page-actions">
                    <button
                        class="btn btn-primary btn-sm"
                        on:click=move |_| {
                            if active_network.get().is_none() {
                                toast.show_toast("Select a network", "Pick an active network first.", "warn");
                                return;
                            }
                            new_key_name.set(String::new());
                            new_key_scope.set("read:leads".to_string());
                            generated_secret_key.set(None);
                            show_key_modal.set(true);
                        }
                    >
                        "+ Create credential"
                    </button>
                </div>
            </div>

            <div class="card" style="padding:10px 14px;border-left:3px solid var(--cobalt);margin-bottom:4px">
                <p class="muted" style="font-size:11.5px;margin:0">
                    <strong style="color:var(--text-primary)">"Live data path. "</strong>
                    "Tokens, endpoints, and deliveries require an active network. Platform service cards reflect env/config — connection checks are not faked."
                </p>
            </div>

            <ActiveNetworkPicker/>

            <div class="kpi-row">
                <div class="kpi-card">
                    <span class="kpi-label">"Platform services"</span>
                    <span class="kpi-value">"Config"</span>
                </div>
                <div class="kpi-card">
                    <span class="kpi-label">"Credentials"</span>
                    <span class="kpi-value">
                        {move || api_keys_res.get().and_then(|k| k).map(|v| v.len().to_string()).unwrap_or_else(|| "—".into())}
                    </span>
                </div>
                <div class="kpi-card">
                    <span class="kpi-label">"Endpoints"</span>
                    <span class="kpi-value">
                        {move || endpoints_res.get().and_then(|k| k).map(|v| v.len().to_string()).unwrap_or_else(|| "—".into())}
                    </span>
                </div>
                <div class="kpi-card">
                    <span class="kpi-label">"Deliveries"</span>
                    <span class="kpi-value">
                        {move || deliveries_res.get().map(|d| d.len().to_string()).unwrap_or_else(|| "—".into())}
                    </span>
                </div>
            </div>

            <div class="tab-bar">
                <button class=move || if active_tab.get() == "services" { "tab active" } else { "tab" }
                    on:click=move |_| active_tab.set("services".to_string())>"Platform services"</button>
                <button class=move || if active_tab.get() == "credentials" { "tab active" } else { "tab" }
                    on:click=move |_| active_tab.set("credentials".to_string())>"API credentials"</button>
                <button class=move || if active_tab.get() == "endpoints" { "tab active" } else { "tab" }
                    on:click=move |_| active_tab.set("endpoints".to_string())>"Webhook endpoints"</button>
                <button class=move || if active_tab.get() == "deliveries" { "tab active" } else { "tab" }
                    on:click=move |_| active_tab.set("deliveries".to_string())>"Delivery log"</button>
            </div>

            <Show when=move || active_tab.get() == "services">
                <div class="card" style="padding:12px 16px;border-left:3px solid var(--amber);margin-bottom:12px">
                    <p class="text-xs" style="color:var(--amber);font-weight:600;margin:0 0 2px">"Platform-level services"</p>
                    <p class="muted" style="font-size:11px;margin:0">
                        "Configured via environment / secrets at deploy time. Status is not a live ping — do not treat UI as a health check."
                    </p>
                </div>
                <div class="grid-cards">
                    <div class="card-item">
                        <div class="card-info">
                            <div class="card-name">"Stripe Connect"</div>
                            <div class="card-desc">"Billing & payouts. Key presence is env config."</div>
                        </div>
                        <span class="tag" style="font-size:9.5px">"Config"</span>
                    </div>
                    <div class="card-item">
                        <div class="card-info">
                            <div class="card-name">"SendGrid / Outbox"</div>
                            <div class="card-desc">"Transactional email via OutboxWorker. Failures surface in audit / outbox."</div>
                        </div>
                        <a href="/logs" class="btn btn-ghost btn-sm">"Audit logs"</a>
                    </div>
                    <div class="card-item">
                        <div class="card-info">
                            <div class="card-name">"Maps / Geo (G-01)"</div>
                            <div class="card-desc">"PostGIS service areas. Empty key = disabled."</div>
                        </div>
                        <a href="/admin/compliance" class="btn btn-ghost btn-sm">"Geo zones"</a>
                    </div>
                    <div class="card-item">
                        <div class="card-info">
                            <div class="card-name">"WebAuthn"</div>
                            <div class="card-desc">"Passkeys — manage registry on Team."</div>
                        </div>
                        <a href="/team#passkeys" class="btn btn-ghost btn-sm">"Passkeys"</a>
                    </div>
                </div>
            </Show>

            <Show when=move || active_tab.get() != "services" && active_network.get().is_none()>
                <div class="card" style="padding:32px;text-align:center">
                    <div style="font-size:14px;font-weight:600;margin-bottom:6px">"Select a tenant network"</div>
                    <p class="muted" style="font-size:12px;max-width:420px;margin:0 auto">
                        "API credentials and webhooks are scoped to a tenant. Use the Active network control above."
                    </p>
                </div>
            </Show>

            <Show when=move || active_tab.get() == "credentials" && active_network.get().is_some()>
                <div class="section">
                    <div class="section-hdr">
                        <span class="section-title">"API credentials"</span>
                        <button class="btn btn-primary btn-sm" on:click=move |_| {
                            generated_secret_key.set(None);
                            show_key_modal.set(true);
                        }>"+ New credential"</button>
                    </div>
                    <table>
                        <thead>
                            <tr>
                                <th>"Label"</th>
                                <th>"ID"</th>
                                <th>"Scopes"</th>
                                <th>"Created"</th>
                                <th></th>
                            </tr>
                        </thead>
                        <tbody>
                            {move || {
                                let keys = api_keys_res.get().flatten().unwrap_or_default();
                                if keys.is_empty() {
                                    view! {
                                        <tr><td colspan="5" class="p-6 text-center muted">
                                            "No credentials. Create a scoped token for this tenant. The full secret is shown once."
                                        </td></tr>
                                    }.into_any()
                                } else {
                                    keys.into_iter().map(|key| {
                                        let kid = key.id.to_string();
                                        let kid_clone = kid.clone();
                                        let short_id = format!("{}…", &kid[..8.min(kid.len())]);
                                        let label = key.display_label();
                                        let scopes = key.scopes_display();
                                        let created = key.created_at.clone().unwrap_or_else(|| "—".into());
                                        view! {
                                            <tr>
                                                <td><strong>{label}</strong></td>
                                                <td class="mono muted">{short_id}</td>
                                                <td class="mono muted">{scopes}</td>
                                                <td class="muted">{created}</td>
                                                <td>
                                                    <button
                                                        class="btn btn-ghost btn-sm"
                                                        style="color:var(--red)"
                                                        on:click=move |_| show_revoke_modal.set(Some(kid_clone.clone()))
                                                    >"Revoke"</button>
                                                </td>
                                            </tr>
                                        }
                                    }).collect_view().into_any()
                                }
                            }}
                        </tbody>
                    </table>
                </div>
            </Show>

            <Show when=move || active_tab.get() == "endpoints" && active_network.get().is_some()>
                <div class="section">
                    <div class="section-hdr">
                        <span class="section-title">"Webhook endpoints"</span>
                        <button class="btn btn-primary btn-sm" on:click=move |_| show_webhook_modal.set(true)>"+ Register endpoint"</button>
                    </div>
                    <table>
                        <thead>
                            <tr>
                                <th>"URL"</th>
                                <th>"Events"</th>
                                <th>"Status"</th>
                                <th>"Created"</th>
                                <th></th>
                            </tr>
                        </thead>
                        <tbody>
                            {move || {
                                let hooks = endpoints_res.get().flatten().unwrap_or_default();
                                if hooks.is_empty() {
                                    view! {
                                        <tr><td colspan="5" class="p-6 text-center muted">
                                            "No endpoints. Register an HTTPS URL and event subscription for this tenant."
                                        </td></tr>
                                    }.into_any()
                                } else {
                                    hooks.into_iter().map(|h| {
                                        let hid = h.id;
                                        let url = h.target_url.clone();
                                        let events = h.events_display();
                                        let active = h.is_active;
                                        let created = h.created_at.clone().unwrap_or_else(|| "—".into());
                                        view! {
                                            <tr>
                                                <td class="mono" style="color:var(--cobalt);max-width:280px;overflow:hidden;text-overflow:ellipsis">{url}</td>
                                                <td class="mono muted">{events}</td>
                                                <td>
                                                    <span class=if active { "tag tag-ok" } else { "tag" }>
                                                        {if active { "Active" } else { "Off" }}
                                                    </span>
                                                </td>
                                                <td class="muted">{created}</td>
                                                <td>
                                                    <button
                                                        class="btn btn-ghost btn-sm"
                                                        style="color:var(--red)"
                                                        on:click=move |_| show_delete_hook.set(Some(hid))
                                                    >"Delete"</button>
                                                </td>
                                            </tr>
                                        }
                                    }).collect_view().into_any()
                                }
                            }}
                        </tbody>
                    </table>
                </div>
            </Show>

            <Show when=move || active_tab.get() == "deliveries" && active_network.get().is_some()>
                <div class="section">
                    <div class="section-hdr">
                        <span class="section-title">"Recent deliveries"</span>
                        <button class="btn btn-ghost btn-sm" on:click=move |_| refetch_trigger.update(|v| *v += 1)>"Refresh"</button>
                    </div>
                    <table>
                        <thead>
                            <tr>
                                <th>"Time"</th>
                                <th>"Event"</th>
                                <th>"Status"</th>
                                <th>"HTTP"</th>
                                <th>"Attempts"</th>
                                <th></th>
                            </tr>
                        </thead>
                        <tbody>
                            <Suspense fallback=move || view! { <tr><td colspan="6" class="p-6 text-center muted">"Loading…"</td></tr> }>
                            {move || {
                                let deliveries = deliveries_res.get().unwrap_or_default();
                                if deliveries.is_empty() {
                                    view! {
                                        <tr><td colspan="6" class="p-6 text-center muted">
                                            "No deliveries yet. Outbound attempts appear after events fire."
                                        </td></tr>
                                    }.into_any()
                                } else {
                                    deliveries.into_iter().map(|w| {
                                        let wid = w.id.to_string();
                                        let wid_btn = wid.clone();
                                        let status_code = w.response_status.unwrap_or(0);
                                        let status_class = if (200..300).contains(&status_code) { "tag tag-ok" } else { "tag tag-error" };
                                        let status_label = w.status.clone();
                                        let delivered_at = w.created_at.clone().unwrap_or_else(|| "—".into());
                                        let event_type = w.event_type.clone();
                                        let attempts = w.attempts;
                                        view! {
                                            <tr style="cursor:pointer" on:click=move |_| {
                                                panel_tab.set("payload".to_string());
                                                selected_delivery_id.set(Some(wid.clone()));
                                            }>
                                                <td class="muted">{delivered_at}</td>
                                                <td class="mono" style="color:var(--cobalt)">{event_type}</td>
                                                <td><span class=status_class>{status_label}</span></td>
                                                <td class="muted">{status_code}</td>
                                                <td class="muted">{attempts}</td>
                                                <td>
                                                    <button class="btn btn-ghost btn-sm" on:click=move |e| {
                                                        e.stop_propagation();
                                                        selected_delivery_id.set(Some(wid_btn.clone()));
                                                    }>"Details"</button>
                                                </td>
                                            </tr>
                                        }
                                    }).collect_view().into_any()
                                }
                            }}
                            </Suspense>
                        </tbody>
                    </table>
                </div>
            </Show>

            // Delivery detail drawer
            <div
                class=move || if selected_delivery_id.get().is_some() { "panel-backdrop open" } else { "panel-backdrop" }
                on:click=move |_| selected_delivery_id.set(None)
            ></div>
            <div class=move || if selected_delivery_id.get().is_some() { "detail-panel open" } else { "detail-panel" }>
                {move || selected_delivery.get().map(|evt| {
                    let delivery_id = StoredValue::new(evt.id.to_string());
                    let status_code = evt.response_status.unwrap_or(0);
                    let status_class = if (200..300).contains(&status_code) { "tag tag-ok" } else { "tag tag-error" };
                    let event_type = StoredValue::new(evt.event_type.clone());
                    let payload_str = StoredValue::new(serde_json::to_string_pretty(&evt.payload).unwrap_or_else(|_| "No payload".into()));
                    let attempts = evt.attempts;
                    let delivered_at = StoredValue::new(evt.created_at.clone().unwrap_or_else(|| "—".into()));
                    let response_body = StoredValue::new(evt.response_body.clone().unwrap_or_else(|| "—".into()));
                    view! {
                        <div class="panel-header">
                            <div class="panel-header-top">
                                <div class="panel-identity">
                                    <div class="panel-title-text mono">{move || delivery_id.get_value()}</div>
                                    <div class="panel-subtitle-text">{move || event_type.get_value()}</div>
                                </div>
                                <button class="panel-close" on:click=move |_| selected_delivery_id.set(None)>"✕"</button>
                            </div>
                            <div class="tab-bar">
                                <button class=move || if panel_tab.get() == "payload" { "tab active" } else { "tab" }
                                    on:click=move |_| panel_tab.set("payload".to_string())>"Payload"</button>
                                <button class=move || if panel_tab.get() == "overview" { "tab active" } else { "tab" }
                                    on:click=move |_| panel_tab.set("overview".to_string())>"Delivery"</button>
                            </div>
                        </div>
                        <div class="panel-content">
                            <Show when=move || panel_tab.get() == "payload">
                                <pre style="font-family:monospace;font-size:11px;background:#05070B;padding:14px;border-radius:6px;color:#00D2FF;overflow-x:auto;border:1px solid var(--border-default)">
                                    {move || payload_str.get_value()}
                                </pre>
                            </Show>
                            <Show when=move || panel_tab.get() == "overview">
                                <div class="detail-grid">
                                    <div class="detail-field">
                                        <div class="detail-label">"HTTP"</div>
                                        <div class="detail-value"><span class=status_class>{status_code.to_string()}</span></div>
                                    </div>
                                    <div class="detail-field">
                                        <div class="detail-label">"Attempts"</div>
                                        <div class="detail-value mono">{attempts}</div>
                                    </div>
                                    <div class="detail-field">
                                        <div class="detail-label">"Created"</div>
                                        <div class="detail-value">{move || delivered_at.get_value()}</div>
                                    </div>
                                    <div class="detail-field">
                                        <div class="detail-label">"Response body"</div>
                                        <div class="detail-value mono">{move || response_body.get_value()}</div>
                                    </div>
                                </div>
                            </Show>
                        </div>
                    }
                })}
            </div>

            // Create credential modal
            <Show when=move || show_key_modal.get()>
                <div class="fixed inset-0 z-[100] bg-black/80 backdrop-blur-sm flex items-center justify-center p-4">
                    <div class="bg-surface w-full max-w-lg p-6 rounded-2xl border border-white/10 shadow-2xl relative text-on-surface">
                        <button class="absolute top-4 right-4 text-slate-400 hover:text-white" on:click=move |_| show_key_modal.set(false)>"✕"</button>
                        <h3 class="text-xl font-semibold mb-2">"Create API credential"</h3>
                        <Show when=move || generated_secret_key.get().is_none() fallback=move || {
                            let key = generated_secret_key.get().unwrap_or_default();
                            view! {
                                <div class="mt-4 p-4 rounded-xl bg-emerald-500/10 border border-emerald-500/20 space-y-4">
                                    <p class="text-xs text-on-surface-variant">"SAVE THIS SECRET. IT WILL NOT BE SHOWN AGAIN."</p>
                                    <div class="flex items-center gap-2 bg-surface-dim p-3 rounded-lg border border-white/5 font-mono text-sm text-emerald-400 justify-between">
                                        <span class="truncate pr-4">{key.clone()}</span>
                                        <button class="btn btn-ghost btn-sm" on:click=move |_| {
                                            let _ = web_sys::window().unwrap().navigator().clipboard().write_text(&key);
                                            toast.show_toast("Success", "Key copied.", "success");
                                        }>"Copy"</button>
                                    </div>
                                    <div class="flex justify-end pt-2">
                                        <button class="btn btn-primary" on:click=move |_| {
                                            show_key_modal.set(false);
                                            new_key_name.set(String::new());
                                            refetch_trigger.update(|v| *v += 1);
                                        }>"Done"</button>
                                    </div>
                                </div>
                            }.into_any()
                        }>
                            <div class="space-y-4 mt-4">
                                <div class="n-form-row">
                                    <label class="n-form-label">"Client name"</label>
                                    <input type="text" class="n-form-input" placeholder="e.g. Harbor CRM sync"
                                        prop:value=new_key_name
                                        on:input=move |ev| new_key_name.set(event_target_value(&ev))/>
                                </div>
                                <div class="n-form-row">
                                    <label class="n-form-label">"Scope"</label>
                                    <select class="n-form-select" on:change=move |ev| new_key_scope.set(event_target_value(&ev))>
                                        <option value="read:leads">"read:leads"</option>
                                        <option value="read:leads,write:leads">"read:leads, write:leads"</option>
                                        <option value="read:accounts">"read:accounts"</option>
                                    </select>
                                </div>
                                <div class="flex justify-end gap-3 pt-4 border-t border-white/5">
                                    <button class="btn btn-ghost" on:click=move |_| show_key_modal.set(false)>"Cancel"</button>
                                    <button class="btn btn-primary" on:click=submit_key_generation>"Create"</button>
                                </div>
                            </div>
                        </Show>
                    </div>
                </div>
            </Show>

            // Register webhook modal
            <Show when=move || show_webhook_modal.get()>
                <div class="fixed inset-0 z-[100] bg-black/80 backdrop-blur-sm flex items-center justify-center p-4">
                    <div class="bg-surface w-full max-w-lg p-6 rounded-2xl border border-white/10 shadow-2xl relative text-on-surface">
                        <button class="absolute top-4 right-4" on:click=move |_| show_webhook_modal.set(false)>"✕"</button>
                        <h3 class="text-xl font-semibold mb-4">"Register webhook endpoint"</h3>
                        <div class="space-y-4">
                            <div class="n-form-row">
                                <label class="n-form-label">"Target URL (HTTPS)"</label>
                                <input type="url" class="n-form-input" placeholder="https://…"
                                    prop:value=new_hook_url
                                    on:input=move |ev| new_hook_url.set(event_target_value(&ev))/>
                            </div>
                            <div class="n-form-row">
                                <label class="n-form-label">"Event"</label>
                                <select class="n-form-select" on:change=move |ev| new_hook_event.set(event_target_value(&ev))>
                                    <option value="lead.created">"lead.created"</option>
                                    <option value="lead.updated">"lead.updated"</option>
                                    <option value="contract.signed">"contract.signed"</option>
                                </select>
                            </div>
                            <div class="flex justify-end gap-3 pt-4 border-t border-white/5">
                                <button class="btn btn-ghost" on:click=move |_| show_webhook_modal.set(false)>"Cancel"</button>
                                <button class="btn btn-primary" on:click=submit_webhook>"Register"</button>
                            </div>
                        </div>
                    </div>
                </div>
            </Show>

            // Revoke credential confirm
            <Show when=move || show_revoke_modal.get().is_some()>
                {let target = show_revoke_modal.get().unwrap_or_default();
                 let target_clone = target.clone();
                 view! {
                    <div class="fixed inset-0 z-[100] bg-black/80 backdrop-blur-sm flex items-center justify-center p-4">
                        <div class="bg-surface w-full max-w-md p-6 rounded-2xl border border-white/10 shadow-2xl relative text-on-surface">
                            <h3 class="text-xl font-semibold mb-2">"Revoke credential"</h3>
                            <p class="text-xs text-on-surface-variant mb-4">
                                "Clients using " <code class="font-mono">{target}</code> " will fail immediately."
                            </p>
                            <div class="flex justify-end gap-3">
                                <button class="btn btn-ghost" on:click=move |_| show_revoke_modal.set(None)>"Cancel"</button>
                                <button class="btn btn-primary" style="background:var(--red);border-color:var(--red)"
                                    on:click=move |_| handle_revoke_key(target_clone.clone())>"Revoke"</button>
                            </div>
                        </div>
                    </div>
                 }}
            </Show>

            // Delete webhook confirm
            <Show when=move || show_delete_hook.get().is_some()>
                {let hid = show_delete_hook.get().unwrap_or_else(Uuid::nil);
                 view! {
                    <div class="fixed inset-0 z-[100] bg-black/80 backdrop-blur-sm flex items-center justify-center p-4">
                        <div class="bg-surface w-full max-w-md p-6 rounded-2xl border border-white/10 shadow-2xl relative text-on-surface">
                            <h3 class="text-xl font-semibold mb-2">"Delete endpoint"</h3>
                            <p class="text-xs text-on-surface-variant mb-4">"Remove this webhook endpoint? Deliveries will stop."</p>
                            <div class="flex justify-end gap-3">
                                <button class="btn btn-ghost" on:click=move |_| show_delete_hook.set(None)>"Cancel"</button>
                                <button class="btn btn-primary" style="background:var(--red);border-color:var(--red)"
                                    on:click=move |_| handle_delete_hook(hid)>"Delete"</button>
                            </div>
                        </div>
                    </div>
                 }}
            </Show>
        </div>
    }
}
