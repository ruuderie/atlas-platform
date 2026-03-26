use leptos::prelude::*;
use leptos::ev;
use shared_ui::components::data_table::DataTable;
use shared_ui::components::tabs::{Tabs, TabButton};
use shared_ui::components::ui::tabs::{TabsContent, TabsList};
use shared_ui::components::ui::button::{Button, ButtonVariant};

use crate::components::milestone_modal::MilestoneModal;

use crate::api::crm::{get_users, get_leads, get_accounts, get_deals, create_lead, create_account};
use crate::api::models::{UserInfo, LeadModel, AccountModel, DealModel, CreateLead, CreateAccount};

#[component]
pub fn CrmGrid() -> impl IntoView {
    let new_record_name = RwSignal::new("".to_string());
    let new_record_email = RwSignal::new("".to_string());
    let new_record_type = RwSignal::new("Lead".to_string());
    let (trigger_fetch, set_trigger_fetch) = signal(0);
    
    let toast = use_context::<crate::app::GlobalToast>().expect("toast context");
    
    let users_res = LocalResource::new(move || { trigger_fetch.get(); async move { get_users().await.unwrap_or_default() }});
    let leads_res = LocalResource::new(move || { trigger_fetch.get(); async move { get_leads().await.unwrap_or_default() }});
    let accounts_res = LocalResource::new(move || { trigger_fetch.get(); async move { get_accounts().await.unwrap_or_default() }});
    let deals_res = LocalResource::new(move || { trigger_fetch.get(); async move { get_deals().await.unwrap_or_default() }});

    let handle_save_record = move |_: ev::MouseEvent| {
        let name = new_record_name.get();
        let email = new_record_email.get();
        let rtype = new_record_type.get();
        
        leptos::task::spawn_local(async move {
            if rtype == "Lead" {
                let data = CreateLead { name, email: Some(email) };
                if let Err(e) = create_lead(data).await { toast.message.set(Some(e)); }
            } else if rtype == "Account" {
                let data = CreateAccount { name };
                if let Err(e) = create_account(data).await { toast.message.set(Some(e)); }
            }
            set_trigger_fetch.update(|v| *v += 1);
            new_record_name.set("".to_string());
            new_record_email.set("".to_string());
        });
    };

    let handle_impersonate = move |user_id: String| {
        let toast = toast.clone();
        leptos::task::spawn_local(async move {
            match crate::api::auth::impersonate_user(&user_id).await {
                Ok(session) => {
                    let url = format!("http://directory.localhost:3000/?impersonate_token={}", session.token);
                    if let Some(window) = web_sys::window() {
                        let _ = window.location().assign(&url);
                    }
                }
                Err(e) => {
                    toast.message.set(Some(format!("Impersonation failed: {}", e)));
                }
            }
        });
    };

    let user_headers = vec!["ID".to_string(), "Name".to_string(), "Email".to_string(), "Role".to_string(), "Status".to_string()];
    let lead_headers = vec!["Lead ID".to_string(), "Name".to_string(), "Email".to_string(), "Status".to_string(), "Converted?".to_string()];
    let account_headers = vec!["Account ID".to_string(), "Name".to_string()];
    let deal_headers = vec!["Deal ID".to_string(), "Name".to_string(), "Customer ID".to_string(), "Amount".to_string(), "Status".to_string(), "Stage".to_string()];

    let user_data = Signal::derive(move || {
        users_res.get().unwrap_or_default().into_iter().map(|u| {
            vec![
                u.id, 
                format!("{} {}", u.first_name, u.last_name), 
                u.email, 
                if u.is_admin { "Admin".into() } else { "User".into() }, 
                "Active".into()
            ]
        }).collect::<Vec<Vec<String>>>()
    });

    let lead_data = Signal::derive(move || {
        leads_res.get().unwrap_or_default().into_iter().map(|l| {
            vec![
                l.id,
                l.name,
                l.email.unwrap_or_else(|| "-".to_string()),
                l.status.unwrap_or_else(|| "New".to_string()),
                l.is_converted.to_string(),
            ]
        }).collect::<Vec<Vec<String>>>()
    });

    let account_data = Signal::derive(move || {
        accounts_res.get().unwrap_or_default().into_iter().map(|a| {
            vec![
                a.id,
                a.name,
            ]
        }).collect::<Vec<Vec<String>>>()
    });

    let deal_data = Signal::derive(move || {
        deals_res.get().unwrap_or_default().into_iter().map(|d| {
            vec![
                d.id,
                d.name,
                d.customer_id,
                format!("${:.2}", d.amount),
                d.status,
                d.stage,
            ]
        }).collect::<Vec<Vec<String>>>()
    });

    let selected_user = RwSignal::new(None::<Vec<String>>);
    let selected_lead = RwSignal::new(None::<Vec<String>>);
    let selected_account = RwSignal::new(None::<Vec<String>>);

    let (show_milestone_modal, set_show_milestone_modal) = signal(false);

    view! {
        <div class="flex flex-col min-h-[calc(100vh-128px)]">
            // ── Header ──
            <header class="bg-surface-container-low px-8 pt-6 -mx-8 -mt-8">
                <div class="flex justify-between items-end mb-6">
                    <div>
                        <h1 class="text-2xl font-bold font-headline text-on-surface tracking-tight">"Sales & Pipelines"</h1>
                        <p class="text-on-surface-variant text-sm mt-1">"Manage leads, contacts, and active deal flow."</p>
                    </div>
                    <a href="/crm/new" class="btn-primary-gradient px-6 py-2.5 rounded-md text-on-primary font-bold text-sm flex items-center gap-2 shadow-lg shadow-primary/10 hover:scale-[1.02] active:scale-95 transition-all">
                        <span class="material-symbols-outlined text-[18px]">"add_circle"</span>
                        "New Record"
                    </a>
                </div>
            </header>

            // ── Tabs + Content ──
            <Suspense fallback=move || view! { <div class="p-8 text-center text-on-surface-variant">"Loading CRM data..."</div> }>
                <Tabs default_value="leads".to_string()>
                    <div class="bg-surface-container-low -mx-8 px-8 border-b border-outline-variant/20">
                        <TabsList class="inline-flex gap-8".to_string()>
                            <TabButton label="Users" value="users" active_value=Signal::derive(|| "".to_string()) on_select=move |_| {} />
                            <TabButton label="Leads" value="leads" active_value=Signal::derive(|| "".to_string()) on_select=move |_| {} />
                            <TabButton label="Accounts & Contacts" value="accounts_contacts" active_value=Signal::derive(|| "".to_string()) on_select=move |_| {} />
                            <TabButton label="Deals" value="deals" active_value=Signal::derive(|| "".to_string()) on_select=move |_| {} />
                        </TabsList>
                    </div>

                    <div class="flex-1 flex flex-col mt-0">
                        // ── Users Tab ──
                        <TabsContent value="users".to_string()>
                            <div class="flex flex-col xl:flex-row gap-0 items-stretch -mx-8">
                                <div class="flex-1 min-w-0 overflow-x-auto border-r border-outline-variant/10 bg-surface-container">
                                    <DataTable 
                                        headers=user_headers.clone() 
                                        data=user_data 
                                        on_row_click=Callback::new(move |row: Vec<String>| selected_user.set(Some(row)))
                                    />
                                </div>
                                <Show when=move || selected_user.get().is_some() fallback=|| view! { <div class="w-full xl:w-[400px] flex items-center justify-center p-8 text-on-surface-variant bg-surface-container-low">"Select a user to view details"</div> }>
                                    <div class="w-full xl:w-[400px] shrink-0 bg-surface-container-low overflow-y-auto p-8 space-y-6">
                                        <div class="flex items-start justify-between mb-6">
                                            <div class="flex items-center gap-4">
                                                <div class="w-16 h-16 rounded-xl bg-surface-bright flex items-center justify-center border border-outline-variant/40">
                                                    <span class="text-2xl font-bold text-primary">{move || selected_user.get().and_then(|u| u.get(1).map(|n| n.chars().take(2).collect::<String>().to_uppercase())).unwrap_or_default()}</span>
                                                </div>
                                                <div>
                                                    <h2 class="text-xl font-bold text-on-surface">{move || selected_user.get().and_then(|u| u.get(1).cloned()).unwrap_or_default()}</h2>
                                                    <p class="text-on-surface-variant text-sm">{move || selected_user.get().and_then(|u| u.get(2).cloned()).unwrap_or_default()}</p>
                                                </div>
                                            </div>
                                        </div>
                                        <div class="grid grid-cols-2 gap-4">
                                            <div class="p-3 rounded-lg bg-surface-container/50 border border-outline-variant/10">
                                                <div class="text-[10px] font-bold text-secondary uppercase tracking-widest mb-1">"Role"</div>
                                                <div class="text-sm font-semibold text-primary">{move || selected_user.get().and_then(|u| u.get(3).cloned()).unwrap_or_default()}</div>
                                            </div>
                                            <div class="p-3 rounded-lg bg-surface-container/50 border border-outline-variant/10">
                                                <div class="text-[10px] font-bold text-secondary uppercase tracking-widest mb-1">"Status"</div>
                                                <div class="text-sm font-semibold text-on-surface">{move || selected_user.get().and_then(|u| u.get(4).cloned()).unwrap_or_default()}</div>
                                            </div>
                                        </div>
                                        <div class="mt-auto pt-6 flex flex-col gap-3">
                                            <a href=move || format!("/crm/user/{}", selected_user.get().and_then(|u| u.get(0).cloned()).unwrap_or_default()) class="w-full py-3 bg-surface-bright text-on-surface font-bold text-sm rounded-md border border-outline/30 hover:bg-surface-tint/10 transition-colors flex items-center justify-center gap-2">
                                                "View Full Details"
                                                <span class="material-symbols-outlined text-sm">"open_in_new"</span>
                                            </a>
                                            <button 
                                                on:click=move |_| {
                                                    if let Some(user_id) = selected_user.get().and_then(|u| u.get(0).cloned()) {
                                                        handle_impersonate(user_id);
                                                    }
                                                }
                                                class="w-full py-3 bg-[#0d0d0d] text-white font-bold text-sm rounded-md shadow-sm border border-white/10 hover:bg-[#1a1a1a] transition-colors flex items-center justify-center gap-2"
                                            >
                                                "Login As User"
                                                <span class="material-symbols-outlined text-sm">"login"</span>
                                            </button>
                                        </div>
                                    </div>
                                </Show>
                            </div>
                        </TabsContent>

                        // ── Leads Tab ──
                        <TabsContent value="leads".to_string()>
                            <div class="flex flex-col xl:flex-row gap-0 items-stretch -mx-8">
                                <div class="flex-1 min-w-0 overflow-x-auto border-r border-outline-variant/10 bg-surface-container">
                                    <DataTable 
                                        headers=lead_headers.clone() 
                                        data=lead_data 
                                        on_row_click=Callback::new(move |row: Vec<String>| selected_lead.set(Some(row)))
                                    />
                                </div>
                                <Show when=move || selected_lead.get().is_some() fallback=|| view! { <div class="w-full xl:w-[400px] flex items-center justify-center p-8 text-on-surface-variant bg-surface-container-low">"Select a lead to view details"</div> }>
                                    <div class="w-full xl:w-[400px] shrink-0 bg-surface-container-low overflow-y-auto p-8 space-y-6">
                                        <div class="flex items-start justify-between mb-6">
                                            <div class="flex items-center gap-4">
                                                <div class="w-16 h-16 rounded-xl bg-surface-bright flex items-center justify-center border border-outline-variant/40">
                                                    <span class="text-2xl font-bold text-primary">{move || selected_lead.get().and_then(|l| l.get(1).map(|n| n.chars().take(2).collect::<String>().to_uppercase())).unwrap_or_default()}</span>
                                                </div>
                                                <div>
                                                    <h2 class="text-xl font-bold text-on-surface">{move || selected_lead.get().and_then(|l| l.get(1).cloned()).unwrap_or_default()}</h2>
                                                    <p class="text-on-surface-variant text-sm">{move || selected_lead.get().and_then(|l| l.get(2).cloned()).unwrap_or_default()}</p>
                                                </div>
                                            </div>
                                        </div>
                                        <div class="grid grid-cols-2 gap-4">
                                            <div class="p-3 rounded-lg bg-surface-container/50 border border-outline-variant/10">
                                                <div class="text-[10px] font-bold text-secondary uppercase tracking-widest mb-1">"Pipeline Stage"</div>
                                                <div class="text-sm font-semibold text-primary">{move || selected_lead.get().and_then(|l| l.get(3).cloned()).unwrap_or_default()}</div>
                                            </div>
                                            <div class="p-3 rounded-lg bg-surface-container/50 border border-outline-variant/10">
                                                <div class="text-[10px] font-bold text-secondary uppercase tracking-widest mb-1">"Converted"</div>
                                                <div class="text-sm font-semibold text-on-surface">{move || selected_lead.get().and_then(|l| l.get(4).cloned()).unwrap_or_default()}</div>
                                            </div>
                                        </div>
                                        // Contact Info
                                        <section>
                                            <h3 class="text-[10px] font-bold text-secondary uppercase tracking-widest mb-4 flex items-center gap-2">
                                                <span class="material-symbols-outlined text-sm">"contact_page"</span>
                                                "Contact Information"
                                            </h3>
                                            <div class="space-y-4">
                                                <div class="flex items-center gap-4">
                                                    <div class="w-8 h-8 rounded-full bg-surface-container flex items-center justify-center text-on-surface-variant">
                                                        <span class="material-symbols-outlined text-sm">"mail"</span>
                                                    </div>
                                                    <div>
                                                        <div class="text-xs text-on-surface-variant">"Primary Email"</div>
                                                        <div class="text-sm text-on-surface">{move || selected_lead.get().and_then(|l| l.get(2).cloned()).unwrap_or_default()}</div>
                                                    </div>
                                                </div>
                                            </div>
                                        </section>
                                        // Activity
                                        <section>
                                            <h3 class="text-[10px] font-bold text-secondary uppercase tracking-widest mb-4 flex items-center gap-2">
                                                <span class="material-symbols-outlined text-sm">"history"</span>
                                                "Recent Activity"
                                            </h3>
                                            <div class="space-y-6 relative before:content-[''] before:absolute before:left-[15px] before:top-2 before:bottom-2 before:w-[1px] before:bg-outline-variant/30">
                                                <div class="relative pl-10">
                                                    <div class="absolute left-0 top-1 w-8 h-8 rounded-full bg-surface-container flex items-center justify-center z-10">
                                                        <span class="material-symbols-outlined text-[14px] text-tertiary">"check_circle"</span>
                                                    </div>
                                                    <div class="text-xs text-on-surface">"Lead created"</div>
                                                    <div class="text-[10px] text-on-surface-variant">"Via platform admin"</div>
                                                </div>
                                            </div>
                                        </section>
                                        <div class="mt-auto pt-6">
                                            <a href=move || format!("/crm/lead/{}", selected_lead.get().and_then(|l| l.get(0).cloned()).unwrap_or_default()) class="w-full py-3 bg-surface-bright text-on-surface font-bold text-sm rounded-md border border-outline/30 hover:bg-surface-tint/10 transition-colors flex items-center justify-center gap-2">
                                                "View Full Details"
                                                <span class="material-symbols-outlined text-sm">"open_in_new"</span>
                                            </a>
                                        </div>
                                    </div>
                                </Show>
                            </div>
                        </TabsContent>

                        // ── Accounts Tab ──
                        <TabsContent value="accounts_contacts".to_string()>
                            <div class="flex flex-col xl:flex-row gap-0 items-stretch -mx-8">
                                <div class="flex-1 min-w-0 overflow-x-auto border-r border-outline-variant/10 bg-surface-container">
                                    <DataTable 
                                        headers=account_headers.clone() 
                                        data=account_data 
                                        on_row_click=Callback::new(move |row: Vec<String>| selected_account.set(Some(row)))
                                    />
                                </div>
                                <Show when=move || selected_account.get().is_some() fallback=|| view! { <div class="w-full xl:w-[400px] flex items-center justify-center p-8 text-on-surface-variant bg-surface-container-low">"Select an account to view details"</div> }>
                                    <div class="w-full xl:w-[400px] shrink-0 bg-surface-container-low overflow-y-auto p-8 space-y-6">
                                        <div class="flex items-center gap-4">
                                            <div class="w-16 h-16 rounded-xl bg-surface-bright flex items-center justify-center border border-outline-variant/40">
                                                <span class="text-2xl font-bold text-primary">{move || selected_account.get().and_then(|a| a.get(1).map(|n| n.chars().take(2).collect::<String>().to_uppercase())).unwrap_or_default()}</span>
                                            </div>
                                            <div>
                                                <h2 class="text-xl font-bold text-on-surface">{move || selected_account.get().and_then(|a| a.get(1).cloned()).unwrap_or_default()}</h2>
                                                <p class="text-on-surface-variant text-sm">{move || selected_account.get().and_then(|a| a.get(0).cloned()).unwrap_or_default()}</p>
                                            </div>
                                        </div>
                                        <div class="mt-auto pt-6">
                                            <a href=move || format!("/crm/account/{}", selected_account.get().and_then(|a| a.get(0).cloned()).unwrap_or_default()) class="w-full py-3 bg-surface-bright text-on-surface font-bold text-sm rounded-md border border-outline/30 hover:bg-surface-tint/10 transition-colors flex items-center justify-center gap-2">
                                                "View Full Details"
                                                <span class="material-symbols-outlined text-sm">"open_in_new"</span>
                                            </a>
                                        </div>
                                    </div>
                                </Show>
                            </div>
                        </TabsContent>

                        // ── Deals Tab ──
                        <TabsContent value="deals".to_string()>
                            <div class="overflow-x-auto bg-surface-container -mx-8">
                                <DataTable 
                                    headers=deal_headers.clone() 
                                    data=deal_data 
                                    on_row_click=Callback::new(move |row: Vec<String>| {
                                        let stage = row.get(5).cloned().unwrap_or_default();
                                        if stage.contains("Negotiation") || stage.contains("Won") {
                                            set_show_milestone_modal.set(true);
                                        }
                                    })
                                />
                            </div>
                        </TabsContent>
                    </div>
                </Tabs>
            </Suspense>
            
            <MilestoneModal 
                is_open=show_milestone_modal
                on_close=Callback::new(move |_| set_show_milestone_modal.set(false))
                on_activate=Callback::new(move |_| {
                    leptos::tracing::info!("Upsell Event: Proposal Auto-Gen Activated");
                    set_show_milestone_modal.set(false);
                })
                title="Deal is heating up!".to_string()
                description="This deal is nearing the finish line. Do you want to automatically generate a tailored proposal?".to_string()
                feature_name="Atlas Proposal Auto-Gen".to_string()
                price_text="$49 / month".to_string()
            />
        </div>
    }
}
