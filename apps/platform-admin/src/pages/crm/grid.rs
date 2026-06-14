use leptos::prelude::*;
use leptos::ev;
use crate::components::milestone_modal::MilestoneModal;
use crate::api::crm::{get_users, get_leads, get_accounts, get_deals, get_contacts, create_lead, create_account, create_contact};
use crate::api::models::{CreateLead, CreateAccount, CreateContact};

#[component]
pub fn CrmGrid() -> impl IntoView {
    let active_tab = RwSignal::new("leads".to_string());
    let new_record_name = RwSignal::new("".to_string());
    let new_record_email = RwSignal::new("".to_string());
    let new_record_type = RwSignal::new("Lead".to_string());
    let (trigger_fetch, set_trigger_fetch) = signal(0);
    
    let toast = use_context::<crate::app::GlobalToast>().expect("toast context");
    
    let users_res = LocalResource::new(move || { trigger_fetch.get(); async move { get_users().await.unwrap_or_default() }});
    let leads_res = LocalResource::new(move || { trigger_fetch.get(); async move { get_leads().await.unwrap_or_default() }});
    let accounts_res = LocalResource::new(move || { trigger_fetch.get(); async move { get_accounts().await.unwrap_or_default() }});
    let contacts_res = LocalResource::new(move || { trigger_fetch.get(); async move { get_contacts().await.unwrap_or_default() }});
    let deals_res = LocalResource::new(move || { trigger_fetch.get(); async move { get_deals().await.unwrap_or_default() }});

    let handle_impersonate = move |user_id: String| {
        let toast = toast.clone();
        leptos::task::spawn_local(async move {
            match crate::api::auth::impersonate_user(&user_id).await {
                Ok(session) => {
                    let url = format!("http://network.localhost:3000/?impersonate_code={}", session.token.as_deref().unwrap_or(""));
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

    let selected_user = RwSignal::new(None::<Vec<String>>);
    let selected_lead = RwSignal::new(None::<Vec<String>>);
    let selected_account = RwSignal::new(None::<Vec<String>>);
    let selected_contact = RwSignal::new(None::<Vec<String>>);

    let (show_milestone_modal, set_show_milestone_modal) = signal(false);

    let page_title = move || match active_tab.get().as_str() {
        "users" => "User Access & Auth",
        "leads" => "Leads",
        "accounts" => "Accounts",
        "contacts" => "Contacts",
        "deals" => "Deals",
        _ => "CRM",
    };

    let page_subtitle = move || match active_tab.get().as_str() {
        "users" => "Manage admin accounts and API permissions.",
        "leads" => "1,847 total · 142 new this week · G-31 Canonical Lead Store · Platform-wide",
        "accounts" => "Platform-wide organization and individual accounts · Party Model (G-31)",
        "contacts" => "Individual contact details and communications history.",
        "deals" => "Sales pipelines, opportunities, and upsell events.",
        _ => "",
    };

    view! {
        <div class="main-area">
            // ── Page Header ──
            <div class="page-header">
                <div>
                    <h1 class="page-title">{page_title}</h1>
                    <p class="page-subtitle">{page_subtitle}</p>
                </div>
                <div class="page-actions">
                    <button class="btn btn-ghost btn-sm">
                        <svg viewBox="0 0 14 14" width="12" height="12" fill="none" stroke="currentColor" stroke-width="1.5">
                            <path d="M2 10L7 2l5 8H2z"/>
                        </svg>
                        "Import"
                    </button>
                    <button class="btn btn-primary btn-sm">
                        <svg viewBox="0 0 14 14" width="12" height="12" fill="currentColor">
                            <path d="M7 2a1 1 0 0 1 1 1v3h3a1 1 0 1 1 0 2H8v3a1 1 0 1 1-2 0V8H3a1 1 0 1 1 0-2h3V3a1 1 0 0 1 1-1z"/>
                        </svg>
                        "New Record"
                    </button>
                </div>
            </div>

            // ── Custom CRM Tabs ──
            <div class="tab-bar">
                <button class=move || format!("tab {}", if active_tab.get() == "users" { "active" } else { "" }) on:click=move |_| active_tab.set("users".to_string())>"Users"</button>
                <button class=move || format!("tab {}", if active_tab.get() == "leads" { "active" } else { "" }) on:click=move |_| active_tab.set("leads".to_string())>"Leads"</button>
                <button class=move || format!("tab {}", if active_tab.get() == "accounts" { "active" } else { "" }) on:click=move |_| active_tab.set("accounts".to_string())>"Accounts"</button>
                <button class=move || format!("tab {}", if active_tab.get() == "contacts" { "active" } else { "" }) on:click=move |_| active_tab.set("contacts".to_string())>"Contacts"</button>
                <button class=move || format!("tab {}", if active_tab.get() == "deals" { "active" } else { "" }) on:click=move |_| active_tab.set("deals".to_string())>"Deals"</button>
            </div>

            // ── Filter Bar ──
            <div class="filter-bar" style="margin: 20px 24px 0 24px;">
                <div class="stage-pills">
                    <button class="pill active">"All"</button>
                    <button class="pill">"Recent"</button>
                </div>
                <div class="filter-sep"></div>
                <div class="filter-search">
                    <input type="text" placeholder="Search records..."/>
                </div>
            </div>

            // ── Data Grids ──
            <div class="table-container" style="margin: 20px 24px;">
                <Suspense fallback=move || view! { <div class="p-8 text-center text-on-surface-variant">"Loading CRM data..."</div> }>
                    {move || match active_tab.get().as_str() {
                        "users" => view! {
                            <table>
                                <thead>
                                    <tr>
                                        <th>"ID"</th>
                                        <th>"Name"</th>
                                        <th>"Email"</th>
                                        <th>"Role"</th>
                                        <th>"Status"</th>
                                    </tr>
                                </thead>
                                <tbody>
                                    {move || users_res.get().unwrap_or_default().into_iter().map(|u| {
                                        let id = u.id.to_string();
                                        let name = format!("{} {}", u.first_name, u.last_name);
                                        let email = u.email.clone();
                                        let role = if u.is_admin { "Admin".to_string() } else { "User".to_string() };
                                        let status = "Active".to_string();
                                        let row = vec![id.clone(), name.clone(), email.clone(), role.clone(), status.clone()];
                                        view! {
                                            <tr on:click=move |_| selected_user.set(Some(row.clone()))>
                                                <td class="mono">{id}</td>
                                                <td>{name}</td>
                                                <td>{email}</td>
                                                <td><span class="tag tag-org">{role}</span></td>
                                                <td><span class="tag tag-active">{status}</span></td>
                                            </tr>
                                        }
                                    }).collect_view()}
                                </tbody>
                            </table>
                        }.into_any(),

                        "leads" => view! {
                            <table>
                                <thead>
                                    <tr>
                                        <th>"Lead ID"</th>
                                        <th>"Name"</th>
                                        <th>"Email"</th>
                                        <th>"Status"</th>
                                        <th>"Converted"</th>
                                    </tr>
                                </thead>
                                <tbody>
                                    {move || leads_res.get().unwrap_or_default().into_iter().map(|l| {
                                        let id = l.id.clone();
                                        let name = l.name.clone();
                                        let email = l.email.clone().unwrap_or_else(|| "-".to_string());
                                        let status = l.status.clone().unwrap_or_else(|| "New".to_string());
                                        let converted = l.is_converted.to_string();
                                        let row = vec![id.clone(), name.clone(), email.clone(), status.clone(), converted.clone()];
                                        view! {
                                            <tr on:click=move |_| selected_lead.set(Some(row.clone()))>
                                                <td class="mono">{id}</td>
                                                <td>{name}</td>
                                                <td>{email}</td>
                                                <td><span class="tag tag-org">{status}</span></td>
                                                <td><span class="tag tag-active">{converted}</span></td>
                                            </tr>
                                        }
                                    }).collect_view()}
                                </tbody>
                            </table>
                        }.into_any(),

                        "accounts" => view! {
                            <table>
                                <thead>
                                    <tr>
                                        <th>"Account ID"</th>
                                        <th>"Name"</th>
                                    </tr>
                                </thead>
                                <tbody>
                                    {move || accounts_res.get().unwrap_or_default().into_iter().map(|a| {
                                        let id = a.id.clone();
                                        let name = a.name.clone();
                                        let row = vec![id.clone(), name.clone()];
                                        view! {
                                            <tr on:click=move |_| selected_account.set(Some(row.clone()))>
                                                <td class="mono">{id}</td>
                                                <td>{name}</td>
                                            </tr>
                                        }
                                    }).collect_view()}
                                </tbody>
                            </table>
                        }.into_any(),

                        "contacts" => view! {
                            <table>
                                <thead>
                                    <tr>
                                        <th>"Contact ID"</th>
                                        <th>"Name"</th>
                                        <th>"Email"</th>
                                        <th>"Phone"</th>
                                        <th>"Created At"</th>
                                    </tr>
                                </thead>
                                <tbody>
                                    {move || contacts_res.get().unwrap_or_default().into_iter().map(|c| {
                                        let id = c.id.clone();
                                        let name = c.name.clone();
                                        let email = c.email.clone().unwrap_or_else(|| "-".to_string());
                                        let phone = c.phone.clone().unwrap_or_else(|| "-".to_string());
                                        let created = c.created_at.clone();
                                        let row = vec![id.clone(), name.clone(), email.clone(), phone.clone(), created.clone()];
                                        view! {
                                            <tr on:click=move |_| selected_contact.set(Some(row.clone()))>
                                                <td class="mono">{id}</td>
                                                <td>{name}</td>
                                                <td>{email}</td>
                                                <td>{phone}</td>
                                                <td class="muted">{created}</td>
                                            </tr>
                                        }
                                    }).collect_view()}
                                </tbody>
                            </table>
                        }.into_any(),

                        "deals" => view! {
                            <table>
                                <thead>
                                    <tr>
                                        <th>"Deal ID"</th>
                                        <th>"Name"</th>
                                        <th>"Customer ID"</th>
                                        <th>"Amount"</th>
                                        <th>"Status"</th>
                                        <th>"Stage"</th>
                                    </tr>
                                </thead>
                                <tbody>
                                    {move || deals_res.get().unwrap_or_default().into_iter().map(|d| {
                                        let id = d.id.clone();
                                        let name = d.name.clone();
                                        let customer = d.customer_id.clone();
                                        let amount = format!("${:.2}", d.amount);
                                        let status = d.status.clone();
                                        let stage = d.stage.clone();
                                        let stage_for_click = stage.clone();
                                        view! {
                                            <tr on:click=move |_| {
                                                if stage_for_click.contains("Negotiation") || stage_for_click.contains("Won") {
                                                    set_show_milestone_modal.set(true);
                                                }
                                            }>
                                                <td class="mono">{id}</td>
                                                <td>{name}</td>
                                                <td class="mono">{customer}</td>
                                                <td class="mono green">{amount}</td>
                                                <td><span class="tag tag-active">{status}</span></td>
                                                <td>{stage}</td>
                                            </tr>
                                        }
                                    }).collect_view()}
                                </tbody>
                            </table>
                        }.into_any(),

                        _ => view! { <div></div> }.into_any(),
                    }}
                </Suspense>
            </div>

            // ── Sliding Drawer Backdrops & Panels ──
            
            // ── Users Details Drawer ──
            <div class=move || format!("panel-backdrop {}", if selected_user.get().is_some() { "open" } else { "" }) on:click=move |_| selected_user.set(None)></div>
            <div class=move || format!("detail-panel {}", if selected_user.get().is_some() { "open" } else { "" })>
                <div class="panel-header">
                    <div class="panel-header-top">
                        <div class="panel-lead-identity">
                            <div class="panel-lead-name">{move || selected_user.get().and_then(|u| u.get(1).cloned()).unwrap_or_default()}</div>
                            <div class="panel-lead-co">{move || selected_user.get().and_then(|u| u.get(2).cloned()).unwrap_or_default()}</div>
                        </div>
                        <button class="panel-close" on:click=move |_| selected_user.set(None)>
                            <svg viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.5">
                                <line x1="4" y1="4" x2="12" y2="12"/><line x1="12" y1="4" x2="4" y2="12"/>
                            </svg>
                        </button>
                    </div>
                    <div class="panel-actions">
                        <a href=move || format!("/crm/user/{}", selected_user.get().and_then(|u| u.get(0).cloned()).unwrap_or_default()) class="btn btn-ghost btn-sm" style="text-decoration:none">"View Full Record"</a>
                        <button 
                            on:click=move |_| {
                                if let Some(user_id) = selected_user.get().and_then(|u| u.get(0).cloned()) {
                                    handle_impersonate(user_id);
                                }
                            }
                            class="btn btn-primary btn-sm"
                        >
                            "Login As User"
                        </button>
                    </div>
                    <div class="panel-tabs">
                        <button class="panel-tab active">"Overview"</button>
                    </div>
                </div>
                <div class="panel-content" style="padding: 16px 20px;">
                    <div class="detail-grid">
                        <span class="detail-section-label">"User Info"</span>
                        <div class="detail-field">
                            <div class="detail-label">"Role"</div>
                            <div class="detail-value">{move || selected_user.get().and_then(|u| u.get(3).cloned()).unwrap_or_default()}</div>
                        </div>
                        <div class="detail-field">
                            <div class="detail-label">"Status"</div>
                            <div class="detail-value">{move || selected_user.get().and_then(|u| u.get(4).cloned()).unwrap_or_default()}</div>
                        </div>
                    </div>
                </div>
            </div>

            // ── Leads Details Drawer ──
            <div class=move || format!("panel-backdrop {}", if selected_lead.get().is_some() { "open" } else { "" }) on:click=move |_| selected_lead.set(None)></div>
            <div class=move || format!("detail-panel {}", if selected_lead.get().is_some() { "open" } else { "" })>
                <div class="panel-header">
                    <div class="panel-header-top">
                        <div class="panel-lead-identity">
                            <div class="panel-lead-name">{move || selected_lead.get().and_then(|l| l.get(1).cloned()).unwrap_or_default()}</div>
                            <div class="panel-lead-co">{move || selected_lead.get().and_then(|l| l.get(2).cloned()).unwrap_or_default()}</div>
                        </div>
                        <button class="panel-close" on:click=move |_| selected_lead.set(None)>
                            <svg viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.5">
                                <line x1="4" y1="4" x2="12" y2="12"/><line x1="12" y1="4" x2="4" y2="12"/>
                            </svg>
                        </button>
                    </div>
                    <div class="panel-actions">
                        <a href=move || format!("/crm/lead/{}", selected_lead.get().and_then(|l| l.get(0).cloned()).unwrap_or_default()) class="btn btn-ghost btn-sm" style="text-decoration:none">"View Full Record"</a>
                    </div>
                    <div class="panel-tabs">
                        <button class="panel-tab active">"Overview"</button>
                    </div>
                </div>
                <div class="panel-content" style="padding: 16px 20px;">
                    <div class="detail-grid">
                        <span class="detail-section-label">"Lead Info"</span>
                        <div class="detail-field">
                            <div class="detail-label">"Pipeline Stage"</div>
                            <div class="detail-value">{move || selected_lead.get().and_then(|l| l.get(3).cloned()).unwrap_or_default()}</div>
                        </div>
                        <div class="detail-field">
                            <div class="detail-label">"Converted"</div>
                            <div class="detail-value">{move || selected_lead.get().and_then(|l| l.get(4).cloned()).unwrap_or_default()}</div>
                        </div>
                    </div>
                </div>
            </div>

            // ── Accounts Details Drawer ──
            <div class=move || format!("panel-backdrop {}", if selected_account.get().is_some() { "open" } else { "" }) on:click=move |_| selected_account.set(None)></div>
            <div class=move || format!("detail-panel {}", if selected_account.get().is_some() { "open" } else { "" })>
                <div class="panel-header">
                    <div class="panel-header-top">
                        <div class="panel-lead-identity">
                            <div class="panel-lead-name">{move || selected_account.get().and_then(|a| a.get(1).cloned()).unwrap_or_default()}</div>
                            <div class="panel-lead-co">{move || selected_account.get().and_then(|a| a.get(0).cloned()).unwrap_or_default()}</div>
                        </div>
                        <button class="panel-close" on:click=move |_| selected_account.set(None)>
                            <svg viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.5">
                                <line x1="4" y1="4" x2="12" y2="12"/><line x1="12" y1="4" x2="4" y2="12"/>
                            </svg>
                        </button>
                    </div>
                    <div class="panel-actions">
                        <a href=move || format!("/crm/account/{}", selected_account.get().and_then(|a| a.get(0).cloned()).unwrap_or_default()) class="btn btn-ghost btn-sm" style="text-decoration:none">"View Full Record"</a>
                    </div>
                    <div class="panel-tabs">
                        <button class="panel-tab active">"Overview"</button>
                    </div>
                </div>
                <div class="panel-content" style="padding: 16px 20px;">
                    <div class="detail-grid">
                        <span class="detail-section-label">"Account Info"</span>
                        <div class="detail-field">
                            <div class="detail-label">"Account Name"</div>
                            <div class="detail-value">{move || selected_account.get().and_then(|a| a.get(1).cloned()).unwrap_or_default()}</div>
                        </div>
                    </div>
                </div>
            </div>

            // ── Contacts Details Drawer ──
            <div class=move || format!("panel-backdrop {}", if selected_contact.get().is_some() { "open" } else { "" }) on:click=move |_| selected_contact.set(None)></div>
            <div class=move || format!("detail-panel {}", if selected_contact.get().is_some() { "open" } else { "" })>
                <div class="panel-header">
                    <div class="panel-header-top">
                        <div class="panel-lead-identity">
                            <div class="panel-lead-name">{move || selected_contact.get().and_then(|c| c.get(1).cloned()).unwrap_or_default()}</div>
                            <div class="panel-lead-co">{move || selected_contact.get().and_then(|c| c.get(2).cloned()).unwrap_or_default()}</div>
                        </div>
                        <button class="panel-close" on:click=move |_| selected_contact.set(None)>
                            <svg viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.5">
                                <line x1="4" y1="4" x2="12" y2="12"/><line x1="12" y1="4" x2="4" y2="12"/>
                            </svg>
                        </button>
                    </div>
                    <div class="panel-actions">
                        <a href=move || format!("/crm/contact/{}", selected_contact.get().and_then(|c| c.get(0).cloned()).unwrap_or_default()) class="btn btn-ghost btn-sm" style="text-decoration:none">"View Full Record"</a>
                    </div>
                    <div class="panel-tabs">
                        <button class="panel-tab active">"Overview"</button>
                    </div>
                </div>
                <div class="panel-content" style="padding: 16px 20px;">
                    <div class="detail-grid">
                        <span class="detail-section-label">"Contact Info"</span>
                        <div class="detail-field">
                            <div class="detail-label">"Phone"</div>
                            <div class="detail-value">{move || selected_contact.get().and_then(|c| c.get(3).cloned()).unwrap_or_default()}</div>
                        </div>
                    </div>
                </div>
            </div>

            <MilestoneModal 
                open=show_milestone_modal
                on_close=Callback::new(move |_| set_show_milestone_modal.set(false))
                on_activate=Callback::new(move |_| {
                    leptos::logging::log!("Upsell Event: Proposal Auto-Gen Activated");
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
