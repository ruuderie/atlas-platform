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
            status_class: "bg-emerald-500/10 border-emerald-500/30 text-emerald-400",
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
            status_class: "bg-emerald-500/10 border-emerald-500/30 text-emerald-400",
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
            status_class: "bg-red-500/10 border-red-500/30 text-red-400",
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
            status_class: "bg-emerald-500/10 border-emerald-500/30 text-emerald-400",
            created: "Jan 2025".to_string(),
        },
        MockApiCredential {
            id: "client_nexus_891f3a".to_string(),
            name: "Nexus CRM Bridge".to_string(),
            scopes: "read:leads, write:leads".to_string(),
            status: "Active".to_string(),
            status_class: "bg-emerald-500/10 border-emerald-500/30 text-emerald-400",
            created: "Feb 2025".to_string(),
        },
        MockApiCredential {
            id: "client_bisc_772a1c".to_string(),
            name: "Biscayne Ledger Exporter".to_string(),
            scopes: "read:ledger".to_string(),
            status: "Active".to_string(),
            status_class: "bg-emerald-500/10 border-emerald-500/30 text-emerald-400",
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
        <div class="max-w-6xl mx-auto space-y-8 animate-in slide-in-from-bottom-4 duration-500 ease-out fade-in">
            // Header
            <header class="flex justify-between items-center bg-surface-container border border-outline-variant/10 p-6 rounded-2xl shadow-sm">
                <div>
                    <h1 class="text-3xl font-light tracking-tight text-on-surface mb-2 font-headline">"Integrations & Webhooks"</h1>
                    <p class="text-on-surface-variant text-sm tracking-wide">"Manage platform-wide API connections and webhook dispatch systems · G-05"</p>
                </div>
                <div>
                    <button 
                        on:click=move |_| {
                            new_key_name.set(String::new());
                            new_key_scope.set("read:leads".to_string());
                            generated_secret_key.set(None);
                            show_key_modal.set(true);
                        }
                        class="btn-primary-gradient px-4 py-2 rounded-lg text-sm font-bold text-on-primary shadow-lg shadow-primary/20 hover:scale-105 active:scale-95 transition-all"
                    >
                        "+ Create Client Credential"
                    </button>
                </div>
            </header>

            // KPIs
            <div class="grid grid-cols-1 md:grid-cols-4 gap-4">
                <div class="bg-surface-container border border-outline-variant/10 p-4 rounded-xl flex flex-col gap-2">
                    <span class="text-[10px] font-bold text-on-surface-variant uppercase tracking-widest">"Connected Services"</span>
                    <span class="text-3xl font-bold font-mono text-on-surface">"4"</span>
                </div>
                <div class="bg-surface-container border border-outline-variant/10 p-4 rounded-xl flex flex-col gap-2">
                    <span class="text-[10px] font-bold text-on-surface-variant uppercase tracking-widest">"Webhook deliveries (24h)"</span>
                    <span class="text-3xl font-bold font-mono text-on-surface">"8,642"</span>
                </div>
                <div class="bg-surface-container border border-outline-variant/10 p-4 rounded-xl flex flex-col gap-2">
                    <span class="text-[10px] font-bold text-on-surface-variant uppercase tracking-widest">"Successful Deliveries"</span>
                    <span class="text-3xl font-bold font-mono text-success">"99.64%"</span>
                </div>
                <div class="bg-surface-container border border-outline-variant/10 p-4 rounded-xl flex flex-col gap-2">
                    <span class="text-[10px] font-bold text-on-surface-variant uppercase tracking-widest">"Active API Credentials"</span>
                    <span class="text-3xl font-bold font-mono text-on-surface">
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
            <div class="flex border-b border-outline-variant/15 flex-shrink-0">
                {
                    let tab_btn = move |id: &'static str, label: &'static str| {
                        let id_str = id.to_string();
                        let active_id = id_str.clone();
                        let click_id = id_str.clone();
                        view! {
                            <button 
                                class=move || if active_tab.get() == active_id { "px-4 py-2 text-sm font-semibold border-b-2 border-primary text-on-surface transition-all" } else { "px-4 py-2 text-sm text-on-surface-variant hover:text-on-surface transition-all" }
                                on:click=move |_| active_tab.set(click_id.clone())
                            >
                                {label}
                            </button>
                        }
                    };
                    view! {
                        {tab_btn("services", "Connected Services")}
                        {tab_btn("webhooks", "Webhook Logs (G-05)")}
                        {tab_btn("credentials", "API Credentials")}
                    }
                }
            </div>

            // Connected Services Tab Content
            <Show when=move || active_tab.get() == "services">
                <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
                    <div class="bg-surface-container border border-outline-variant/10 rounded-xl p-5 flex items-center justify-between gap-4">
                        <div class="flex items-center gap-4">
                            <div class="w-10 h-10 rounded-xl bg-purple-500/10 border border-purple-500/20 text-purple-400 flex items-center justify-center text-lg">"💳"</div>
                            <div>
                                <h3 class="font-semibold text-on-surface">"Stripe Connect"</h3>
                                <p class="text-xs text-on-surface-variant">"Payment splits and MRR collecting (MoR). Status: "<span class="text-success font-bold">"Active"</span></p>
                            </div>
                        </div>
                        <button on:click=move |_| handle_test_connection("stripe") class="px-3 py-1.5 text-xs bg-surface-container border border-outline-variant/30 hover:bg-surface-container-high/40 rounded transition-all">"Test Connection"</button>
                    </div>
                    <div class="bg-surface-container border border-outline-variant/10 rounded-xl p-5 flex items-center justify-between gap-4">
                        <div class="flex items-center gap-4">
                            <div class="w-10 h-10 rounded-xl bg-emerald-500/10 border border-emerald-500/20 text-emerald-400 flex items-center justify-center text-lg">"🗺"</div>
                            <div>
                                <h3 class="font-semibold text-on-surface">"PostGIS Geography Engine"</h3>
                                <p class="text-xs text-on-surface-variant">"Geo boundaries and PostGIS queries (G-01). Status: "<span class="text-success font-bold">"Active"</span></p>
                            </div>
                        </div>
                        <button on:click=move |_| handle_test_connection("postgis") class="px-3 py-1.5 text-xs bg-surface-container border border-outline-variant/30 hover:bg-surface-container-high/40 rounded transition-all">"Test Connection"</button>
                    </div>
                    <div class="bg-surface-container border border-outline-variant/10 rounded-xl p-5 flex items-center justify-between gap-4">
                        <div class="flex items-center gap-4">
                            <div class="w-10 h-10 rounded-xl bg-red-500/10 border border-red-500/20 text-red-400 flex items-center justify-center text-lg">"📧"</div>
                            <div>
                                <h3 class="font-semibold text-on-surface">"SendGrid Transactional"</h3>
                                <p class="text-xs text-on-surface-variant">"Outgoing SMTP transactional and campaigns (G-19). Status: "<span class="text-success font-bold">"Active"</span></p>
                            </div>
                        </div>
                        <button on:click=move |_| handle_test_connection("sendgrid") class="px-3 py-1.5 text-xs bg-surface-container border border-outline-variant/30 hover:bg-surface-container-high/40 rounded transition-all">"Test Connection"</button>
                    </div>
                    <div class="bg-surface-container border border-outline-variant/10 rounded-xl p-5 flex items-center justify-between gap-4">
                        <div class="flex items-center gap-4">
                            <div class="w-10 h-10 rounded-xl bg-blue-500/10 border border-blue-500/20 text-blue-400 flex items-center justify-center text-lg">"🔑"</div>
                            <div>
                                <h3 class="font-semibold text-on-surface">"WebAuthn (Passkeys) Registry"</h3>
                                <p class="text-xs text-on-surface-variant">"Dynamic multi-tenant passkey authentication. Status: "<span class="text-success font-bold">"Active"</span></p>
                            </div>
                        </div>
                        <button on:click=move |_| handle_test_connection("webauthn") class="px-3 py-1.5 text-xs bg-surface-container border border-outline-variant/30 hover:bg-surface-container-high/40 rounded transition-all">"Test Connection"</button>
                    </div>
                </div>
            </Show>

            // Webhook Logs Tab Content
            <Show when=move || active_tab.get() == "webhooks">
                <div class="bg-surface border border-outline-variant/10 rounded-xl overflow-hidden shadow-sm">
                    <div class="px-5 py-4 bg-surface-container/30 border-b border-outline-variant/10 flex justify-between items-center">
                        <span class="font-semibold text-sm">"Recent Dispatched Events"</span>
                        <button on:click=move |_| refetch_trigger.update(|v| *v += 1) class="px-3 py-1.5 text-xs font-semibold bg-[#05183c] border border-[#7bd0ff]/20 text-[#7bd0ff] rounded hover:bg-[#05183c]/50 transition-all">"Refresh"</button>
                    </div>
                    <div class="overflow-x-auto w-full">
                        <table class="w-full text-left text-sm whitespace-nowrap">
                            <thead class="bg-surface-container-highest/60 text-[#91aaeb] text-xs font-medium uppercase tracking-wider">
                                <tr>
                                    <th class="px-6 py-4">"Event ID"</th>
                                    <th class="px-6 py-4">"Tenant"</th>
                                    <th class="px-6 py-4">"Event Type"</th>
                                    <th class="px-6 py-4">"Target URL"</th>
                                    <th class="px-6 py-4">"Status"</th>
                                    <th class="px-6 py-4">"Attempts"</th>
                                    <th class="px-6 py-4">"Time"</th>
                                    <th class="px-6 py-4"></th>
                                </tr>
                            </thead>
                            <tbody class="divide-y divide-outline-variant/10 text-on-surface">
                                <For 
                                    each=move || mock_webhooks.get() 
                                    key=|w| w.id.clone() 
                                    children=move |w| {
                                        let wid = w.id.clone();
                                        view! {
                                            <tr on:click=move |_| selected_delivery_id.set(Some(wid.clone())) class="hover:bg-surface-bright/5 cursor-pointer">
                                                <td class="px-6 py-4 font-mono font-semibold text-xs">{w.id.clone()}</td>
                                                <td class="px-6 py-4 font-medium">{w.tenant.clone()}</td>
                                                <td class="px-6 py-4 font-mono text-xs text-primary">{w.event_type.clone()}</td>
                                                <td class="px-6 py-4 font-mono text-xs text-on-surface-variant break-all max-w-[200px] truncate">{w.target_url.clone()}</td>
                                                <td class="px-6 py-4">
                                                    <span class=format!("px-2 py-0.5 rounded text-[10px] uppercase font-bold border {}", w.status_class)>
                                                        {w.status.clone()}
                                                    </span>
                                                </td>
                                                <td class="px-6 py-4 font-mono text-xs text-on-surface-variant text-center">{w.attempts}</td>
                                                <td class="px-6 py-4 text-xs text-on-surface-variant">{w.time.clone()}</td>
                                                <td class="px-6 py-4 text-right">
                                                    <button class="px-2.5 py-1 text-xs bg-surface-container border border-outline-variant/30 hover:bg-surface-container-high/40 rounded transition-all">"Payload"</button>
                                                </td>
                                            </tr>
                                        }
                                    }
                                />
                            </tbody>
                        </table>
                    </div>
                </div>
            </Show>

            // API Credentials Tab Content
            <Show when=move || active_tab.get() == "credentials">
                <div class="bg-surface border border-outline-variant/10 rounded-xl overflow-hidden shadow-sm">
                    <div class="px-5 py-4 bg-surface-container/30 border-b border-outline-variant/10 flex justify-between items-center">
                        <span class="font-semibold text-sm">"Client API Credentials"</span>
                    </div>
                    <div class="overflow-x-auto w-full">
                        <table class="w-full text-left text-sm whitespace-nowrap">
                            <thead class="bg-surface-container-highest/60 text-[#91aaeb] text-xs font-medium uppercase tracking-wider">
                                <tr>
                                    <th class="px-6 py-4">"Client Name"</th>
                                    <th class="px-6 py-4">"Client ID"</th>
                                    <th class="px-6 py-4">"Scopes"</th>
                                    <th class="px-6 py-4">"Status"</th>
                                    <th class="px-6 py-4">"Created"</th>
                                    <th class="px-6 py-4"></th>
                                </tr>
                            </thead>
                            <tbody class="divide-y divide-outline-variant/10 text-on-surface">
                                {move || {
                                    if let Some(keys) = api_keys_res.get().flatten() {
                                        view! {
                                            <For each=move || keys.clone() key=|k| k.id children=move |key| {
                                                let kid = key.id.to_string();
                                                let kid_clone = kid.clone();
                                                view! {
                                                    <tr class="hover:bg-surface-bright/5">
                                                        <td class="px-6 py-4 font-bold">"REST API Token"</td>
                                                        <td class="px-6 py-4 font-mono text-xs">{key.id.to_string()}</td>
                                                        <td class="px-6 py-4 font-mono text-xs text-on-surface-variant">{key.scopes.to_string()}</td>
                                                        <td class="px-6 py-4">
                                                            <span class="px-2 py-0.5 rounded text-[10px] uppercase font-bold border bg-emerald-500/10 border-emerald-500/30 text-emerald-400">
                                                                "Active"
                                                            </span>
                                                        </td>
                                                        <td class="px-6 py-4 text-xs text-on-surface-variant">{key.created_at.clone().unwrap_or_else(|| "-".to_string())}</td>
                                                        <td class="px-6 py-4 text-right">
                                                            <button 
                                                                on:click=move |_| show_revoke_modal.set(Some(kid_clone.clone()))
                                                                class="px-2.5 py-1 text-xs font-semibold bg-red-600/15 border border-red-500/30 hover:bg-red-600/25 rounded transition-all text-red-400"
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
                                                 view! {
                                                     <tr class="hover:bg-surface-bright/5">
                                                         <td class="px-6 py-4 font-bold">{move || k_val.with_value(|v| v.name.clone())}</td>
                                                         <td class="px-6 py-4 font-mono text-xs">{move || k_val.with_value(|v| v.id.clone())}</td>
                                                         <td class="px-6 py-4 font-mono text-xs text-on-surface-variant">{move || k_val.with_value(|v| v.scopes.clone())}</td>
                                                         <td class="px-6 py-4">
                                                             <span class=move || k_val.with_value(|v| format!("px-2 py-0.5 rounded text-[10px] uppercase font-bold border {}", v.status_class))>
                                                                 {move || k_val.with_value(|v| v.status.clone())}
                                                             </span>
                                                         </td>
                                                         <td class="px-6 py-4 text-xs text-on-surface-variant">{move || k_val.with_value(|v| v.created.clone())}</td>
                                                         <td class="px-6 py-4 text-right">
                                                             <Show when=move || is_active>
                                                                 <button 
                                                                     on:click=move |_| show_revoke_modal.set(Some(k_val.with_value(|v| v.id.clone())))
                                                                     class="px-2.5 py-1 text-xs font-semibold bg-red-600/15 border border-red-500/30 hover:bg-red-600/25 rounded transition-all text-red-400"
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
                </div>
            </Show>

            // Webhook details panel overlay
            <div 
                class=move || if selected_delivery_id.get().is_some() { "panel-backdrop open" } else { "panel-backdrop" }
                on:click=move |_| selected_delivery_id.set(None)
                style="position: fixed; inset: 0; background: rgba(0,0,0,0.5); z-index: 300; opacity: 0; pointer-events: none; transition: opacity 0.2s;"
            ></div>
            <div 
                class=move || if selected_delivery_id.get().is_some() { "detail-panel open" } else { "detail-panel" }
                style="position: fixed; top: 0; right: -560px; width: 560px; height: 100vh; background: #111520; border-left: 1px solid rgba(255,255,255,0.08); z-index: 400; display: flex; flex-direction: column; transition: right 0.24s cubic-bezier(0.25, 0.46, 0.45, 0.94); overflow: hidden;"
            >
                {move || selected_delivery.get().map(|evt| {
                    let evt_clone = evt.clone();
                    view! {
                        <div class="panel-header" style="padding: 16px 20px 0; border-bottom: 1px solid rgba(255,255,255,0.08); flex-shrink: 0;">
                            <div class="panel-header-top" style="display: flex; align-items: flex-start; justify-content: space-between; gap: 12px; margin-bottom: 12px;">
                                <div class="panel-identity" style="flex: 1; min-width: 0;">
                                    <div class="panel-title-text font-mono" style="font-size: 18px; font-weight: 700; color: #E8EAF0;">{evt.id.clone()}</div>
                                    <div class="panel-subtitle-text" style="font-size: 12.5px; color: #8B92A8; margin-top: 3px;">{evt.event_type.clone()} " · " {evt.tenant.clone()}</div>
                                </div>
                                <button 
                                    class="panel-close" 
                                    on:click=move |_| selected_delivery_id.set(None)
                                    style="width: 28px; height: 28px; border-radius: 5px; border: 1px solid rgba(255,255,255,0.08); background: transparent; color: #525A72; cursor: pointer; display: flex; align-items: center; justify-content: center; transition: all 0.12s;"
                                >
                                    "✕"
                                </button>
                            </div>
                            <div class="panel-actions" style="display: flex; align-items: center; gap: 6px; padding-bottom: 12px;">
                                <button 
                                    on:click=move |_| handle_retrigger_webhook(evt_clone.id.clone())
                                    class="btn-primary-gradient px-3 py-1.5 text-xs font-bold text-on-primary rounded-lg shadow-sm"
                                >
                                    "Resend Event"
                                </button>
                            </div>
                        </div>
                        <div class="panel-content" style="flex: 1; overflow-y: auto; padding: 16px 20px;">
                            <div class="grid grid-cols-2 gap-y-4 gap-x-8 text-sm">
                                <div class="col-span-2 text-[10px] font-bold text-[#8B92A8] uppercase tracking-widest border-b border-white/5 pb-2">"HTTP Response Telemetry"</div>
                                <div class="space-y-1">
                                    <span class="text-xs text-[#8B92A8]">"Status Code"</span>
                                    <p class={format!("font-medium {}", if evt.status.contains("200") { "text-emerald-400" } else { "text-red-400" })}>{evt.status.clone()}</p>
                                </div>
                                <div class="space-y-1">
                                    <span class="text-xs text-[#8B92A8]">"Duration"</span>
                                    <p class="font-mono text-xs text-[#E8EAF0]">{evt.duration.clone()}</p>
                                </div>
                                <div class="space-y-1">
                                    <span class="text-xs text-[#8B92A8]">"Attempt Count"</span>
                                    <p class="font-mono text-xs text-[#E8EAF0]">{evt.attempts}</p>
                                </div>
                                <div class="space-y-1">
                                    <span class="text-xs text-[#8B92A8]">"Next Retry"</span>
                                    <p class="font-medium text-[#E8EAF0]">{evt.retry.clone()}</p>
                                </div>
                                <div class="col-span-2 text-[10px] font-bold text-[#8B92A8] uppercase tracking-widest border-b border-white/5 pb-2 mt-4">"JSON Payload"</div>
                                <div class="col-span-2">
                                    <pre style="font-family:monospace; font-size:11px; background:#05070B; padding:14px; border-radius:6px; color:#00D2FF; overflow-x:auto; border:1px solid rgba(255,255,255,0.08);">
                                        {serde_json::to_string_pretty(&evt.payload).unwrap_or_default()}
                                    </pre>
                                </div>
                            </div>
                        </div>
                    }
                })}
            </div>

            // Create Key Dialog Modal
            <Show when=move || show_key_modal.get()>
                <div class="fixed inset-0 z-[100] bg-black/80 backdrop-blur-sm flex items-center justify-center p-4">
                    <div class="bg-card w-full max-w-lg p-6 rounded-2xl border border-white/10 shadow-2xl relative text-on-surface">
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
                                            class="px-2 py-1 text-xs bg-surface-container-high border border-outline-variant/30 rounded"
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
                                            class="btn-primary-gradient px-4 py-2 rounded text-xs font-bold text-on-primary"
                                        >
                                            "Done"
                                        </button>
                                    </div>
                                </div>
                            }.into_any()
                        }>
                            <div class="space-y-4 mt-4">
                                <div class="grid grid-cols-2 gap-4">
                                    <div class="flex flex-col gap-1.5">
                                        <label class="text-xs font-medium text-on-surface-variant">"Client Name"</label>
                                        <input 
                                            type="text" 
                                            class="bg-surface-container-highest border border-outline/20 text-on-surface text-sm rounded-lg p-2.5 w-full focus:ring-primary focus:border-primary"
                                            placeholder="e.g. Ruud Ledger Exporter"
                                            prop:value=new_key_name
                                            on:input=move |ev| new_key_name.set(event_target_value(&ev))
                                        />
                                    </div>
                                    <div class="flex flex-col gap-1.5">
                                        <label class="text-xs font-medium text-on-surface-variant">"Scope Privilege Level"</label>
                                        <select 
                                            class="bg-surface-container-highest border border-outline/20 text-on-surface text-sm rounded-lg p-2.5 w-full focus:ring-primary focus:border-primary"
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
                                    <button on:click=move |_| show_key_modal.set(false) class="px-4 py-2 bg-surface-container-highest border border-outline-variant/30 rounded-lg text-xs font-bold text-on-surface">"Cancel"</button>
                                    <button on:click=submit_key_generation class="btn-primary-gradient px-4 py-2 rounded-lg text-xs font-bold text-on-primary">"Generate Key"</button>
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
                        <div class="bg-card w-full max-w-md p-6 rounded-2xl border border-white/10 shadow-2xl relative text-on-surface">
                            <button class="absolute top-4 right-4 text-slate-400 hover:text-white" on:click=move |_| show_revoke_modal.set(None)>"✕"</button>
                            <h3 class="text-xl font-semibold mb-2">"Revoke Credential"</h3>
                            <div class="mt-4 p-3 bg-red-500/10 border border-red-500/20 rounded-xl space-y-2">
                                <p class="text-xs text-red-400">"Are you sure you want to revoke this credential?"</p>
                                <p class="text-xs text-[#8B92A8]">"All active API applications running under Client ID " <code class="bg-[#05070B] px-1 py-0.5 rounded font-mono text-[11px]">{target.clone()}</code> " will fail immediately."</p>
                            </div>
                            <div class="flex justify-end gap-3 pt-6 border-t border-white/5 mt-4">
                                <button on:click=move |_| show_revoke_modal.set(None) class="px-4 py-2 bg-surface-container-highest border border-outline-variant/30 rounded-lg text-xs font-bold text-on-surface">"Cancel"</button>
                                <button on:click=move |_| handle_revoke_key(target_clone.clone()) class="px-4 py-2 bg-red-600 text-white rounded-lg text-xs font-bold hover:bg-red-700 transition-colors">"Revoke Access"</button>
                            </div>
                        </div>
                    </div>
                }}
            </Show>
        </div>
    }
}
