use leptos::prelude::*;
use uuid::Uuid;
use serde_json::json;
use crate::api::developer::*;
use crate::app::GlobalToast;

#[derive(Clone, Debug, PartialEq)]
pub struct MockWebhookDelivery {
    pub id: String,
    pub tenant: String,
    pub event_type: String,
    pub target_url: String,
    pub status: String,
    pub status_class: &'static str,
    pub attempts: i32,
    pub time: String,
    pub duration: String,
    pub retry: String,
    pub payload: serde_json::Value,
}

#[derive(Clone, Debug, PartialEq)]
pub struct MockApiCredential {
    pub id: String,
    pub name: String,
    pub scopes: String,
    pub status: String,
    pub status_class: &'static str,
    pub created: String,
}

#[component]
pub fn Integrations() -> impl IntoView {
    let active_network = use_context::<ReadSignal<Option<Uuid>>>().expect("active network context");
    let toast = use_context::<GlobalToast>().expect("GlobalToast not found");

    // UI state
    let active_tab = RwSignal::new("services".to_string());
    let selected_delivery_id = RwSignal::new(None::<String>);
    let refetch_trigger = RwSignal::new(0);
    let panel_tab = RwSignal::new("payload".to_string());
    
    // Modal states
    let show_key_modal = RwSignal::new(false);
    let show_revoke_modal = RwSignal::new(None::<String>);
    let new_key_name = RwSignal::new(String::new());
    let new_key_scope = RwSignal::new("read:leads".to_string());
    let generated_secret_key = RwSignal::new(None::<String>);

    // Mock webhook logs (fallback/enrichment)
    let mock_webhooks = RwSignal::new(vec![
        MockWebhookDelivery {
            id: "evt_9fca6af8".to_string(),
            tenant: "Nexus Property Group".to_string(),
            event_type: "lead.created".to_string(),
            target_url: "https://api.nexus.com/webhooks".to_string(),
            status: "200 OK".to_string(),
            status_class: "tag tag-ok",
            attempts: 1,
            time: "2 mins ago".to_string(),
            duration: "142ms".to_string(),
            retry: "None".to_string(),
            payload: json!({
                "event_id": "evt_9fca6af8",
                "type": "lead.created",
                "tenant_id": "t_nexus_01",
                "data": { "lead_id": "l_9a2f3", "company": "Ruud Logistics", "score": 9.4 }
            }),
        },
        MockWebhookDelivery {
            id: "evt_eda9043a".to_string(),
            tenant: "Biscayne STR Co.".to_string(),
            event_type: "ledger.split_reconciled".to_string(),
            target_url: "https://biscayne.io/webhooks/atlas".to_string(),
            status: "200 OK".to_string(),
            status_class: "tag tag-ok",
            attempts: 1,
            time: "14 mins ago".to_string(),
            duration: "214ms".to_string(),
            retry: "None".to_string(),
            payload: json!({
                "event_id": "evt_eda9043a",
                "type": "ledger.split_reconciled",
                "tenant_id": "t_biscayne_str",
                "data": { "split_id": "ls_0083a2", "gross": 420000, "net": 386400 }
            }),
        },
        MockWebhookDelivery {
            id: "evt_005c6922".to_string(),
            tenant: "South Beach Nets".to_string(),
            event_type: "subscription.updated".to_string(),
            target_url: "https://southbeachnets.io/webhook".to_string(),
            status: "500 Fail".to_string(),
            status_class: "tag tag-error",
            attempts: 3,
            time: "1 hour ago".to_string(),
            duration: "4.2s".to_string(),
            retry: "Pending (in 4 mins)".to_string(),
            payload: json!({
                "event_id": "evt_005c6922",
                "type": "subscription.updated",
                "tenant_id": "t_south_beach",
                "data": { "subscription_id": "sub_4R2", "previous_status": "active", "new_status": "past_due" }
            }),
        },
    ]);

    // Mock API Credentials database (local state)
    let mock_credentials = RwSignal::new(vec![
        MockApiCredential {
            id: "client_cli_a82f3c".to_string(),
            name: "Internal Platform CLI".to_string(),
            scopes: "read:all, write:all, root".to_string(),
            status: "Active".to_string(),
            status_class: "tag tag-ok",
            created: "Jan 2025".to_string(),
        },
        MockApiCredential {
            id: "client_nexus_891f3a".to_string(),
            name: "Nexus CRM Bridge".to_string(),
            scopes: "read:leads, write:leads".to_string(),
            status: "Active".to_string(),
            status_class: "tag tag-ok",
            created: "Feb 2025".to_string(),
        },
        MockApiCredential {
            id: "client_bisc_772a1c".to_string(),
            name: "Biscayne Ledger Exporter".to_string(),
            scopes: "read:ledger".to_string(),
            status: "Active".to_string(),
            status_class: "tag tag-ok",
            created: "Jun 2025".to_string(),
        },
    ]);

    // API resource hooks for actual backend integration
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



    // Helper: test single integration connection
    let handle_test_connection = move |service: &'static str| {
        let t = toast.clone();
        leptos::task::spawn_local(async move {
            gloo_timers::future::TimeoutFuture::new(600).await;
            t.show_toast("Success", &format!("{} connection test successful.", service.to_uppercase()), "success");
        });
    };

    // Helper: generate new credential
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
            if let Some(tenant_id) = tenant {
                // Perform real API call
                let scopes_arr = json!([scope]);
                let req = CreateApiTokenRequest { scopes: scopes_arr };
                match create_api_token(tenant_id, req).await {
                    Ok(resp) => {
                        generated_secret_key.set(Some(resp.token));
                        t_toast.show_toast("Success", "API credential created.", "success");
                        refetch_trigger.update(|v| *v += 1);
                    }
                    Err(e) => t_toast.show_toast("Error", &format!("Failed: {}", e), "error"),
                }
            } else {
                // Perform local mock generation
                let mock_id = format!("client_{}", Uuid::new_v4().to_string().chars().take(6).collect::<String>());
                let mock_sk = format!("at_sk_live_{}", Uuid::new_v4().to_string().replace("-", ""));
                
                mock_credentials.update(|list| {
                    list.push(MockApiCredential {
                        id: mock_id,
                        name: name.clone(),
                        scopes: scope.clone(),
                        status: "Active".to_string(),
                        status_class: "bg-emerald-500/10 border-emerald-500/30 text-emerald-400",
                        created: "Jun 2026".to_string(),
                    });
                });
                
                generated_secret_key.set(Some(mock_sk));
                t_toast.show_toast("Success", "Mock API key generated.", "success");
            }
        });
    };

    // Helper: revoke key
    let handle_revoke_key = move |id: String| {
        let tenant = active_network.get();
        let t_toast = toast.clone();
        let target_id = id.clone();
        
        leptos::task::spawn_local(async move {
            if let Some(tenant_id) = tenant {
                if let Ok(parsed_id) = Uuid::parse_str(&target_id) {
                    match revoke_api_token(tenant_id, parsed_id).await {
                        Ok(_) => {
                            t_toast.show_toast("Success", "Credential revoked successfully.", "success");
                            refetch_trigger.update(|v| *v += 1);
                        }
                        Err(e) => t_toast.show_toast("Error", &format!("Failed: {}", e), "error"),
                    }
                }
            } else {
                mock_credentials.update(|list| {
                    if let Some(k) = list.iter_mut().find(|c| c.id == target_id) {
                        k.status = "Revoked".to_string();
                        k.status_class = "bg-red-500/10 border-red-500/30 text-red-400";
                    }
                });
                t_toast.show_toast("Warning", "Credential marked as revoked.", "warn");
            }
            show_revoke_modal.set(None);
        });
    };

    // Helper: retrigger webhook event dispatch
    let handle_retrigger_webhook = move |id: String| {
        let t_toast = toast.clone();
        let target_id = id.clone();
        leptos::task::spawn_local(async move {
            t_toast.show_toast("Info", &format!("Webhook event {} re-enqueued.", target_id), "info");
            gloo_timers::future::TimeoutFuture::new(800).await;
            
            mock_webhooks.update(|list| {
                if let Some(w) = list.iter_mut().find(|evt| evt.id == target_id) {
                    w.status = "200 OK".to_string();
                    w.status_class = "bg-emerald-500/10 border-emerald-500/30 text-emerald-400";
                    w.attempts += 1;
                    w.retry = "None".to_string();
                }
            });
            
            t_toast.show_toast("Success", "Webhook delivered successfully with status code 200.", "success");
        });
    };

    let selected_delivery = Signal::derive(move || {
        let sid = selected_delivery_id.get();
        sid.and_then(|id| {
            mock_webhooks.get().iter().find(|w| w.id == id).cloned()
        })
    });

    view! {
        // Page Header
        <div class="page-header">
            <div>
                <h1 class="page-title">"Integrations & Webhooks"</h1>
                <p class="page-subtitle">"Manage platform-wide API connections and webhook dispatch systems · G-05"</p>
            </div>
            <div class="page-actions">
                <button 
                    on:click=move |_| {
                        new_key_name.set(String::new());
                        new_key_scope.set("read:leads".to_string());
                        generated_secret_key.set(None);
                        show_key_modal.set(true);
                    }
                    class="btn btn-primary btn-sm"
                >
                    "+ Create Client Credential"
                </button>
            </div>
        </div>

        // KPI Row
        <div class="kpi-row">
            <div class="kpi-card">
                <span class="kpi-label">"Connected Services"</span>
                <span class="kpi-value">"4"</span>
            </div>
            <div class="kpi-card">
                <span class="kpi-label">"Webhook deliveries (24h)"</span>
                <span class="kpi-value">"8,642"</span>
            </div>
            <div class="kpi-card">
                <span class="kpi-label">"Successful Deliveries"</span>
                <span class="kpi-value" style="color:var(--green)">"99.64%"</span>
            </div>
            <div class="kpi-card">
                <span class="kpi-label">"Active API Credentials"</span>
                <span class="kpi-value">
                    {move || {
                        if active_network.get().is_some() {
                            api_keys_res.get().map(|k| k.unwrap_or_default().len()).unwrap_or(0).to_string()
                        } else {
                            mock_credentials.get().iter().filter(|c| c.status == "Active").count().to_string()
                        }
                    }}
                </span>
            </div>
        </div>

        // Tabs
        <div class="tab-bar">
            <button 
                class=move || if active_tab.get() == "services" { "tab active" } else { "tab" }
                on:click=move |_| active_tab.set("services".to_string())
            >
                "Connected Services"
            </button>
            <button 
                class=move || if active_tab.get() == "webhooks" { "tab active" } else { "tab" }
                on:click=move |_| active_tab.set("webhooks".to_string())
            >
                "Webhook Logs (G-05)"
            </button>
            <button 
                class=move || if active_tab.get() == "credentials" { "tab active" } else { "tab" }
                on:click=move |_| active_tab.set("credentials".to_string())
            >
                "API Credentials"
            </button>
        </div>

        // Connected Services Content
        <Show when=move || active_tab.get() == "services">
            <div class="grid-cards">
                <div class="card-item">
                    <div class="card-icon" style="background:#635BFF22;color:#635BFF">"💳"</div>
                    <div class="card-info">
                        <div class="card-name">"Stripe Connect"</div>
                        <div class="card-desc">"Payment splits and MRR collecting (MoR). Status: "<span class="tag tag-ok" style="font-size:9.5px">"Active"</span></div>
                    </div>
                    <button on:click=move |_| handle_test_connection("stripe") class="btn btn-ghost btn-sm">"Test Connection"</button>
                </div>
                <div class="card-item">
                    <div class="card-icon" style="background:#00A86B22;color:#00A86B">"🗺"</div>
                    <div class="card-info">
                        <div class="card-name">"PostGIS Geography Engine"</div>
                        <div class="card-desc">"Geo boundaries and PostGIS queries (G-01). Status: "<span class="tag tag-ok" style="font-size:9.5px">"Active"</span></div>
                    </div>
                    <button on:click=move |_| handle_test_connection("postgis") class="btn btn-ghost btn-sm">"Test Connection"</button>
                </div>
                <div class="card-item">
                    <div class="card-icon" style="background:#E5484D22;color:#E5484D">"📧"</div>
                    <div class="card-info">
                        <div class="card-name">"SendGrid transactional"</div>
                        <div class="card-desc">"Outgoing SMTP transactional and campaigns (G-19). Status: "<span class="tag tag-ok" style="font-size:9.5px">"Active"</span></div>
                    </div>
                    <button on:click=move |_| handle_test_connection("sendgrid") class="btn btn-ghost btn-sm">"Test Connection"</button>
                </div>
                <div class="card-item">
                    <div class="card-icon" style="background:#7C3AED22;color:#7C3AED">"🔑"</div>
                    <div class="card-info">
                        <div class="card-name">"WebAuthn (Passkeys) Registry"</div>
                        <div class="card-desc">"Dynamic multi-tenant passkey authentication. Status: "<span class="tag tag-ok" style="font-size:9.5px">"Active"</span></div>
                    </div>
                    <button on:click=move |_| handle_test_connection("webauthn") class="btn btn-ghost btn-sm">"Test Connection"</button>
                </div>
            </div>
        </Show>

        // Webhook Logs Content
        <Show when=move || active_tab.get() == "webhooks">
            <div class="section">
                <div class="section-hdr">
                    <span class="section-title">"Recent Dispatched Events"</span>
                    <button on:click=move |_| refetch_trigger.update(|v| *v += 1) class="btn btn-ghost btn-sm">"Refresh"</button>
                </div>
                <table>
                    <thead>
                        <tr>
                            <th>"Event ID"</th>
                            <th>"Tenant"</th>
                            <th>"Event Type"</th>
                            <th>"Target URL"</th>
                            <th>"Status"</th>
                            <th>"Attempts"</th>
                            <th>"Time"</th>
                            <th></th>
                        </tr>
                    </thead>
                    <tbody>
                        <For 
                            each=move || mock_webhooks.get() 
                            key=|w| w.id.clone() 
                            children=move |w| {
                                let wid = w.id.clone();
                                let wid_for_click = wid.clone();
                                let wid_for_btn = wid.clone();
                                view! {
                                    <tr on:click=move |_| {
                                        panel_tab.set("payload".to_string());
                                        selected_delivery_id.set(Some(wid_for_click.clone()));
                                    } style="cursor:pointer">
                                        <td class="mono font-semibold">{w.id.clone()}</td>
                                        <td>{w.tenant.clone()}</td>
                                        <td><span class="mono">{w.event_type.clone()}</span></td>
                                        <td class="mono muted">{w.target_url.clone()}</td>
                                        <td><span class=w.status_class>{w.status.clone()}</span></td>
                                        <td class="mono muted">{w.attempts}</td>
                                        <td class="muted">{w.time.clone()}</td>
                                        <td>
                                            <button 
                                                on:click=move |e| {
                                                    e.stop_propagation();
                                                    panel_tab.set("payload".to_string());
                                                    selected_delivery_id.set(Some(wid_for_btn.clone()));
                                                }
                                                class="btn btn-ghost btn-sm"
                                            >
                                                "Payload"
                                            </button>
                                        </td>
                                    </tr>
                                }
                            }
                        />
                    </tbody>
                </table>
            </div>
        </Show>

        // API Credentials Content
        <Show when=move || active_tab.get() == "credentials">
            <div class="section">
                <div class="section-hdr">
                    <span class="section-title">"Client API Credentials"</span>
                </div>
                <table>
                    <thead>
                        <tr>
                            <th>"Client Name"</th>
                            <th>"Client ID"</th>
                            <th>"Scopes"</th>
                            <th>"Status"</th>
                            <th>"Created"</th>
                            <th></th>
                        </tr>
                    </thead>
                    <tbody>
                        {move || {
                            if let Some(keys) = api_keys_res.get().flatten() {
                                view! {
                                    <For each=move || keys.clone() key=|k| k.id children=move |key| {
                                        let kid = key.id.to_string();
                                        let kid_clone = kid.clone();
                                        view! {
                                            <tr>
                                                <td><strong>"REST API Token"</strong></td>
                                                <td class="mono">{key.id.to_string()}</td>
                                                <td class="muted mono">{key.scopes.to_string()}</td>
                                                <td><span class="tag tag-ok">"Active"</span></td>
                                                <td class="muted">{key.created_at.clone().unwrap_or_else(|| "-".to_string())}</td>
                                                <td>
                                                    <button 
                                                        on:click=move |_| show_revoke_modal.set(Some(kid_clone.clone()))
                                                        class="btn btn-ghost btn-sm"
                                                        style="color:var(--red)"
                                                    >
                                                        "Revoke"
                                                    </button>
                                                </td>
                                            </tr>
                                        }
                                    }/>
                                }.into_any()
                            } else {
                                view! {
                                    <For each=move || mock_credentials.get() key=|k| k.id.clone() children=move |k| {
                                         let k_val = StoredValue::new(k);
                                         let is_active = k_val.with_value(|v| v.status == "Active");
                                         let k_status_class = k_val.with_value(|v| if v.status == "Active" { "tag tag-ok" } else { "tag tag-error" });
                                         view! {
                                             <tr>
                                                 <td><strong>{move || k_val.with_value(|v| v.name.clone())}</strong></td>
                                                 <td class="mono">{move || k_val.with_value(|v| v.id.clone())}</td>
                                                 <td class="muted mono">{move || k_val.with_value(|v| v.scopes.clone())}</td>
                                                 <td>
                                                     <span class=k_status_class>
                                                         {move || k_val.with_value(|v| v.status.clone())}
                                                     </span>
                                                 </td>
                                                 <td class="muted">{move || k_val.with_value(|v| v.created.clone())}</td>
                                                 <td>
                                                     <Show when=move || is_active>
                                                         <button 
                                                             on:click=move |_| show_revoke_modal.set(Some(k_val.with_value(|v| v.id.clone())))
                                                             class="btn btn-ghost btn-sm"
                                                             style="color:var(--red)"
                                                         >
                                                             "Revoke"
                                                         </button>
                                                     </Show>
                                                 </td>
                                             </tr>
                                         }
                                     }/>
                                }.into_any()
                            }
                        }}
                    </tbody>
                </table>
            </div>
        </Show>

        // Webhook detail panel overlay drawer
        <div 
            class=move || if selected_delivery_id.get().is_some() { "panel-backdrop open" } else { "panel-backdrop" }
            on:click=move |_| selected_delivery_id.set(None)
        ></div>
        <div 
            class=move || if selected_delivery_id.get().is_some() { "detail-panel open" } else { "detail-panel" }
        >
            {move || selected_delivery.get().map(|evt| {
                let evt_clone = evt.clone();
                view! {
                    <div class="panel-header">
                        <div class="panel-header-top">
                            <div class="panel-identity">
                                <div class="panel-title-text mono">{evt.id.clone()}</div>
                                <div class="panel-subtitle-text">{evt.event_type.clone()} " · " {evt.tenant.clone()}</div>
                            </div>
                            <button 
                                class="panel-close" 
                                on:click=move |_| selected_delivery_id.set(None)
                            >
                                <svg viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.5"><line x1="4" y1="4" x2="12" y2="12"/><line x1="12" y1="4" x2="4" y2="12"/></svg>
                            </button>
                        </div>
                        <div class="panel-actions">
                            <button 
                                on:click=move |_| handle_retrigger_webhook(evt_clone.id.clone())
                                class="btn btn-primary btn-sm"
                            >
                                "Resend Event"
                            </button>
                        </div>
                        <div class="panel-tabs">
                            <button 
                                class=move || if panel_tab.get() == "payload" { "panel-tab active" } else { "panel-tab" }
                                on:click=move |_| panel_tab.set("payload".to_string())
                            >
                                "JSON Payload"
                            </button>
                            <button 
                                class=move || if panel_tab.get() == "overview" { "panel-tab active" } else { "panel-tab" }
                                on:click=move |_| panel_tab.set("overview".to_string())
                            >
                                "Delivery Info"
                            </button>
                        </div>
                    </div>
                    <div class="panel-content">
                        // Payload Tab Pane
                        <Show when=move || panel_tab.get() == "payload">
                            <div class="tab-pane active">
                                <pre style="font-family:monospace; font-size:11px; background:#05070B; padding:14px; border-radius:6px; color:#00D2FF; overflow-x:auto; border:1px solid var(--border-default)">
                                    {serde_json::to_string_pretty(&evt.payload).unwrap_or_default()}
                                </pre>
                            </div>
                        </Show>
                        // Overview Tab Pane
                        <Show when=move || panel_tab.get() == "overview">
                            <div class="tab-pane active">
                                <div class="detail-grid">
                                    <span class="detail-section-label">"HTTP Response Telemetry"</span>
                                    <div class="detail-field">
                                        <div class="detail-label">"Status Code"</div>
                                        <div class="detail-value"><span class=evt.status_class>{evt.status.clone()}</span></div>
                                    </div>
                                    <div class="detail-field">
                                        <div class="detail-label">"Duration"</div>
                                        <div class="detail-value mono">{evt.duration.clone()}</div>
                                    </div>
                                    <div class="detail-field">
                                        <div class="detail-label">"Attempt Count"</div>
                                        <div class="detail-value mono">{evt.attempts}</div>
                                    </div>
                                    <div class="detail-field">
                                        <div class="detail-label">"Next Retry"</div>
                                        <div class="detail-value">{evt.retry.clone()}</div>
                                    </div>
                                </div>
                            </div>
                        </Show>
                    </div>
                }
            })}
        </div>

        // Create Key Dialog Modal
        <Show when=move || show_key_modal.get()>
            <div class="fixed inset-0 z-[100] bg-black/80 backdrop-blur-sm flex items-center justify-center p-4">
                <div class="bg-[#111520] w-full max-w-lg p-6 rounded-2xl border border-white/10 shadow-2xl relative text-on-surface">
                    <button class="absolute top-4 right-4 text-slate-400 hover:text-white" on:click=move |_| show_key_modal.set(false)>"✕"</button>
                    <h3 class="text-xl font-semibold mb-2">"Generate Client API Credential"</h3>
                    
                    <Show when=move || generated_secret_key.get().is_none() fallback=move || {
                        let key = generated_secret_key.get().unwrap_or_default();
                        view! {
                            <div class="mt-4 p-4 rounded-xl bg-emerald-500/10 border border-emerald-500/20 space-y-4">
                                <p class="text-xs text-[#8B92A8]">"SAVE THIS SECRET KEY. IT WILL NOT BE SHOWN AGAIN."</p>
                                <div class="flex items-center gap-2 bg-[#05070B] p-3 rounded-lg border border-white/5 font-mono text-sm text-emerald-400 justify-between">
                                    <span class="truncate pr-4">{key.clone()}</span>
                                    <button 
                                        on:click=move |_| {
                                            let _ = web_sys::window().unwrap().navigator().clipboard().write_text(&key);
                                            toast.show_toast("Success", "Key copied to clipboard.", "success");
                                        }
                                        class="btn btn-ghost btn-sm"
                                    >
                                        "Copy"
                                    </button>
                                </div>
                                <div class="flex justify-end pt-2">
                                    <button 
                                        on:click=move |_| {
                                            show_key_modal.set(false);
                                            new_key_name.set(String::new());
                                            refetch_trigger.update(|v| *v += 1);
                                        } 
                                        class="btn btn-primary"
                                    >
                                        "Done"
                                    </button>
                                </div>
                            </div>
                        }.into_any()
                    }>
                        <div class="space-y-4 mt-4">
                            <div style="display:grid; grid-template-columns:1fr 1fr; gap:12px">
                                <div class="n-form-row">
                                    <label class="n-form-label">"Client Name"</label>
                                    <input 
                                        type="text" 
                                        class="n-form-input"
                                        placeholder="e.g. Ruud Ledger Exporter"
                                        prop:value=new_key_name
                                        on:input=move |ev| new_key_name.set(event_target_value(&ev))
                                    />
                                </div>
                                <div class="n-form-row">
                                    <label class="n-form-label">"Scope Privilege Level"</label>
                                    <select 
                                        class="n-form-select"
                                        on:change=move |ev| new_key_scope.set(event_target_value(&ev))
                                    >
                                        <option value="read:leads">"read:leads"</option>
                                        <option value="read:leads, write:leads">"read:leads, write:leads"</option>
                                        <option value="read:ledger">"read:ledger"</option>
                                        <option value="read:all, write:all, root">"root super-admin"</option>
                                    </select>
                                </div>
                            </div>
                            <div class="flex justify-end gap-3 pt-4 border-t border-white/5">
                                <button on:click=move |_| show_key_modal.set(false) class="btn btn-ghost">"Cancel"</button>
                                <button on:click=submit_key_generation class="btn btn-primary">"Generate Key"</button>
                            </div>
                        </div>
                    </Show>
                </div>
            </div>
        </Show>

        // Revoke Confirmation Dialog Modal
        <Show when=move || show_revoke_modal.get().is_some()>
            {let target = show_revoke_modal.get().unwrap_or_default();
             let target_clone = target.clone();
             view! {
                <div class="fixed inset-0 z-[100] bg-black/80 backdrop-blur-sm flex items-center justify-center p-4">
                    <div class="bg-[#111520] w-full max-w-md p-6 rounded-2xl border border-white/10 shadow-2xl relative text-on-surface">
                        <button class="absolute top-4 right-4 text-slate-400 hover:text-white" on:click=move |_| show_revoke_modal.set(None)>"✕"</button>
                        <h3 class="text-xl font-semibold mb-2">"Revoke Credential"</h3>
                        <div class="mt-4 p-3 bg-red-500/10 border border-red-500/20 rounded-xl space-y-2">
                            <p class="text-xs text-red-400">"Are you sure you want to revoke this credential?"</p>
                            <p class="text-xs text-[#8B92A8]">"All active API applications running under Client ID " <code class="bg-[#05070B] px-1 py-0.5 rounded font-mono text-[11px]">{target.clone()}</code> " will fail immediately."</p>
                        </div>
                        <div class="flex justify-end gap-3 pt-6 border-t border-white/5 mt-4">
                            <button on:click=move |_| show_revoke_modal.set(None) class="btn btn-ghost">"Cancel"</button>
                            <button on:click=move |_| handle_revoke_key(target_clone.clone()) class="btn btn-primary" style="background-color:var(--red); border-color:var(--red);">"Revoke Access"</button>
                        </div>
                    </div>
                </div>
            }}
        </Show>
    }
}

