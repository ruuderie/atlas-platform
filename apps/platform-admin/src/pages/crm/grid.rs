use leptos::prelude::*;
use leptos_router::hooks::{use_query_map, use_navigate};
use crate::components::milestone_modal::MilestoneModal;
use crate::api::crm::{get_leads, get_accounts, get_deals, get_contacts};
use crate::api::models::{LeadModel, AccountModel, ContactModel, DealModel};

#[component]
pub fn CrmGrid() -> impl IntoView {
    let query = use_query_map();
    let navigate = use_navigate();
    
    // Read active tab from URL query param "?tab=xxx", default to "leads"
    let active_tab = move || {
        query.with(|q| q.get("tab").map(|s| s.to_string()).unwrap_or_else(|| "leads".to_string()))
    };

    let toast = use_context::<crate::app::GlobalToast>().expect("toast context");
    let (trigger_fetch, _set_trigger_fetch) = signal(0);
    
    // API data resources
    let leads_res = LocalResource::new(move || { trigger_fetch.get(); async move { get_leads().await.unwrap_or_default() }});
    let accounts_res = LocalResource::new(move || { trigger_fetch.get(); async move { get_accounts().await.unwrap_or_default() }});
    let contacts_res = LocalResource::new(move || { trigger_fetch.get(); async move { get_contacts().await.unwrap_or_default() }});
    let deals_res = LocalResource::new(move || { trigger_fetch.get(); async move { get_deals().await.unwrap_or_default() }});

    // Selected items for drawers
    let selected_lead = RwSignal::new(None::<LeadModel>);
    let selected_account = RwSignal::new(None::<AccountModel>);
    let selected_contact = RwSignal::new(None::<ContactModel>);
    let selected_deal = RwSignal::new(None::<DealModel>);

    let (show_milestone_modal, set_show_milestone_modal) = signal(false);

    // Active filters
    let lead_filter_stage = RwSignal::new("all".to_string());
    let lead_filter_search = RwSignal::new("".to_string());

    let acc_filter_type = RwSignal::new("all".to_string());
    let acc_filter_search = RwSignal::new("".to_string());

    let con_filter_verification = RwSignal::new("all".to_string());
    let con_filter_search = RwSignal::new("".to_string());

    let opp_filter_stage = RwSignal::new("all".to_string());
    let opp_filter_search = RwSignal::new("".to_string());

    let page_title = move || match active_tab().as_str() {
        "leads" => "Leads",
        "accounts" => "Accounts",
        "contacts" => "Contacts",
        "opportunities" => "Opportunities",
        _ => "CRM",
    };

    let page_subtitle = move || match active_tab().as_str() {
        "leads" => format!("{} total · 142 new this week · G-31 Canonical Lead Store · Platform-wide", leads_res.get().unwrap_or_default().len()),
        "accounts" => format!("{} total · Platform-wide organization and individual accounts · Party Model (G-31)", accounts_res.get().unwrap_or_default().len()),
        "contacts" => format!("{} total · Associated individual profiles, roles, and identity checks · Party Model (G-31)", contacts_res.get().unwrap_or_default().len()),
        "opportunities" => format!("{} total · Commercial deals, pipeline valuations, and platform contracts · G-11", deals_res.get().unwrap_or_default().len()),
        _ => "".to_string(),
    };

    view! {
        <div class="main-area">
            // ── Page Header ──
            <div class="page-header">
                <div>
                    <h1 class="page-title">{page_title}</h1>
                    <p class="page-subtitle">{page_subtitle}</p>
                </div>
                <div style="display:flex;gap:8px;">
                    <button class="btn btn-ghost btn-sm" on:click=move |_| {
                        toast.message.set(Some("Exporting CSV file...".to_string()));
                    }>"Export CSV"</button>
                    <button class="btn btn-primary btn-sm" on:click=move |_| {
                        toast.message.set(Some("Create record form loaded.".to_string()));
                    }>
                        <svg viewBox="0 0 14 14" width="12" height="12" fill="currentColor" style="margin-right:4px;">
                            <path d="M7 2a1 1 0 0 1 1 1v3h3a1 1 0 1 1 0 2H8v3a1 1 0 1 1-2 0V8H3a1 1 0 1 1 0-2h3V3a1 1 0 0 1 1-1z"/>
                        </svg>
                        {move || format!("New {}", match active_tab().as_str() {
                            "leads" => "Lead",
                            "accounts" => "Account",
                            "contacts" => "Contact",
                            "opportunities" => "Opportunity",
                            _ => "Record"
                        })}
                    </button>
                </div>
            </div>

            // ── Custom CRM Tabs Bar ──
            <div class="tab-bar">
                <button class=move || format!("tab {}", if active_tab() == "leads" { "active" } else { "" }) on:click={
                    let navigate = navigate.clone();
                    move |_| navigate("/crm?tab=leads", Default::default())
                }>"Leads"</button>
                <button class=move || format!("tab {}", if active_tab() == "accounts" { "active" } else { "" }) on:click={
                    let navigate = navigate.clone();
                    move |_| navigate("/crm?tab=accounts", Default::default())
                }>"Accounts"</button>
                <button class=move || format!("tab {}", if active_tab() == "contacts" { "active" } else { "" }) on:click={
                    let navigate = navigate.clone();
                    move |_| navigate("/crm?tab=contacts", Default::default())
                }>"Contacts"</button>
                <button class=move || format!("tab {}", if active_tab() == "opportunities" { "active" } else { "" }) on:click={
                    let navigate = navigate.clone();
                    move |_| navigate("/crm?tab=opportunities", Default::default())
                }>"Opportunities"</button>
            </div>

            // ── KPIs for Accounts, Contacts, Opportunities ──
            {move || match active_tab().as_str() {
                "accounts" => view! {
                    <div class="kpi-row">
                        <div class="kpi-card">
                            <span class="kpi-label">"Total Accounts"</span>
                            <span class="kpi-value">{move || accounts_res.get().unwrap_or_default().len().to_string()}</span>
                            <span class="kpi-delta up">"↑ 4 this month"</span>
                        </div>
                        <div class="kpi-card">
                            <span class="kpi-label">"Organizations"</span>
                            <span class="kpi-value">{move || accounts_res.get().unwrap_or_default().iter().filter(|a| a.name.contains("Group") || a.name.contains("STR") || a.name.contains("PM") || a.name.contains("Properties") || a.name.contains("Logística")).count().to_string()}</span>
                        </div>
                        <div class="kpi-card">
                            <span class="kpi-label">"Individuals"</span>
                            <span class="kpi-value">{move || { let total = accounts_res.get().unwrap_or_default().len(); let orgs = accounts_res.get().unwrap_or_default().iter().filter(|a| a.name.contains("Group") || a.name.contains("STR") || a.name.contains("PM") || a.name.contains("Properties") || a.name.contains("Logística")).count(); (total - orgs).to_string() }}</span>
                        </div>
                        <div class="kpi-card">
                            <span class="kpi-label">"Contribution MRR"</span>
                            <span class="kpi-value" style="color:var(--green)">{move || { let total_won: f32 = deals_res.get().unwrap_or_default().iter().filter(|d| d.stage == "Closed Won").map(|d| d.amount).sum(); format!("${:.0}", total_won) }}</span>
                            <span class="kpi-delta up" style="color:var(--green)">"↑ 12.8%"</span>
                        </div>
                    </div>
                }.into_any(),
                "contacts" => view! {
                    <div class="kpi-row">
                        <div class="kpi-card">
                            <span class="kpi-label">"Total Contacts"</span>
                            <span class="kpi-value">{move || contacts_res.get().unwrap_or_default().len().to_string()}</span>
                            <span class="kpi-delta up">"↑ 18 this quarter"</span>
                        </div>
                        <div class="kpi-card">
                            <span class="kpi-label">"Verified Profiles"</span>
                            <span class="kpi-value" style="color:var(--green)">{move || contacts_res.get().unwrap_or_default().len().saturating_sub(6).to_string()}</span>
                        </div>
                        <div class="kpi-card">
                            <span class="kpi-label">"Pending G-06 Checks"</span>
                            <span class="kpi-value" style="color:var(--amber)">{move || (contacts_res.get().unwrap_or_default().len() * 4 / 100).min(6).to_string()}</span>
                        </div>
                        <div class="kpi-card">
                            <span class="kpi-label">"Flagged / Failed checks"</span>
                            <span class="kpi-value" style="color:var(--red)">{move || (contacts_res.get().unwrap_or_default().len() * 4 / 100).min(6).to_string()}</span>
                        </div>
                    </div>
                }.into_any(),
                "opportunities" => view! {
                    <div class="kpi-row">
                        <div class="kpi-card">
                            <span class="kpi-label">"Open Opportunities"</span>
                            <span class="kpi-value">{move || deals_res.get().unwrap_or_default().iter().filter(|d| d.stage != "Closed Won" && d.stage != "Closed Lost").count().to_string()}</span>
                            <span class="kpi-delta up">"↑ 3 this month"</span>
                        </div>
                        <div class="kpi-card">
                            <span class="kpi-label">"Total Pipeline"</span>
                            <span class="kpi-value" style="color:var(--cobalt)">{move || { let total: f32 = deals_res.get().unwrap_or_default().iter().filter(|d| d.stage != "Closed Won" && d.stage != "Closed Lost").map(|d| d.amount).sum(); format!("${:.2}M", total / 1_000_000.0) }}</span>
                        </div>
                        <div class="kpi-card">
                            <span class="kpi-label">"Weighted Pipeline"</span>
                            <span class="kpi-value" style="color:var(--green)">{move || { let total: f32 = deals_res.get().unwrap_or_default().iter().filter(|d| d.stage != "Closed Won" && d.stage != "Closed Lost").map(|d| d.amount).sum(); format!("${:.2}M", (total * 0.6) / 1_000_000.0) }}</span>
                        </div>
                        <div class="kpi-card">
                            <span class="kpi-label">"Avg Deal Size"</span>
                            <span class="kpi-value">{move || { let deals = deals_res.get().unwrap_or_default(); if deals.is_empty() { "$0k".to_string() } else { let avg = deals.iter().map(|d| d.amount).sum::<f32>() / (deals.len() as f32); format!("${:.0}k", avg / 1000.0) } }}</span>
                        </div>
                    </div>
                }.into_any(),
                _ => view! {}.into_any()
            }}

            // ── Filter Bar (dynamic based on tab) ──
            <div class="filter-bar">
                {move || match active_tab().as_str() {
                    "leads" => view! {
                        <div class="stage-pills">
                            {["all", "New", "Contacted", "Qualified", "Proposal", "Converted", "Disqualified"].into_iter().map(|st| {
                                let st_clone = st.to_string();
                                let st_click = st_clone.clone();
                                let label = if st == "all" { "All" } else { st };
                                view! {
                                    <button 
                                        class=move || format!("pill {}", if lead_filter_stage.get() == st_clone { "active" } else { "" })
                                        on:click=move |_| lead_filter_stage.set(st_click.clone())
                                    >
                                        {label}
                                    </button>
                                }
                            }).collect_view()}
                        </div>
                        <div class="filter-sep"></div>
                        <div class="filter-search">
                            <input 
                                type="text" 
                                placeholder="Search leads…" 
                                prop:value=move || lead_filter_search.get()
                                on:input=move |e| lead_filter_search.set(event_target_value(&e))
                            />
                        </div>
                    }.into_any(),
                    "accounts" => view! {
                        <div class="stage-pills">
                            {["all", "Organization", "Individual", "Active", "Suspended"].into_iter().map(|t| {
                                let t_clone = t.to_string();
                                let t_click = t_clone.clone();
                                let label = match t {
                                    "all" => "All",
                                    "Organization" => "Orgs",
                                    "Individual" => "Individuals",
                                    _ => t
                                };
                                view! {
                                    <button 
                                        class=move || format!("pill {}", if acc_filter_type.get() == t_clone { "active" } else { "" })
                                        on:click=move |_| acc_filter_type.set(t_click.clone())
                                    >
                                        {label}
                                    </button>
                                }
                            }).collect_view()}
                        </div>
                        <div class="filter-sep"></div>
                        <div class="filter-search">
                            <input 
                                type="text" 
                                placeholder="Search accounts…" 
                                prop:value=move || acc_filter_search.get()
                                on:input=move |e| acc_filter_search.set(event_target_value(&e))
                            />
                        </div>
                    }.into_any(),
                    "contacts" => view! {
                        <div class="stage-pills">
                            {["all", "Verified", "Pending", "Flagged"].into_iter().map(|v| {
                                let v_clone = v.to_string();
                                let v_click = v_clone.clone();
                                let label = if v == "all" { "All" } else { v };
                                view! {
                                    <button 
                                        class=move || format!("pill {}", if con_filter_verification.get() == v_clone { "active" } else { "" })
                                        on:click=move |_| con_filter_verification.set(v_click.clone())
                                    >
                                        {label}
                                    </button>
                                }
                            }).collect_view()}
                        </div>
                        <div class="filter-sep"></div>
                        <div class="filter-search">
                            <input 
                                type="text" 
                                placeholder="Search contacts…" 
                                prop:value=move || con_filter_search.get()
                                on:input=move |e| con_filter_search.set(event_target_value(&e))
                            />
                        </div>
                    }.into_any(),
                    "opportunities" => view! {
                        <div class="stage-pills">
                            {["all", "Qualification", "Proposal", "Negotiation", "Closed Won", "Closed Lost"].into_iter().map(|s| {
                                let s_clone = s.to_string();
                                let s_click = s_clone.clone();
                                let label = match s {
                                    "all" => "All",
                                    "Qualification" => "Qualify",
                                    "Proposal" => "Proposal",
                                    "Negotiation" => "Negotiate",
                                    "Closed Won" => "Won",
                                    "Closed Lost" => "Lost",
                                    _ => s
                                };
                                view! {
                                    <button 
                                        class=move || format!("pill {}", if opp_filter_stage.get() == s_clone { "active" } else { "" })
                                        on:click=move |_| opp_filter_stage.set(s_click.clone())
                                    >
                                        {label}
                                    </button>
                                }
                            }).collect_view()}
                        </div>
                        <div class="filter-sep"></div>
                        <div class="filter-search">
                            <input 
                                type="text" 
                                placeholder="Search opportunities…" 
                                prop:value=move || opp_filter_search.get()
                                on:input=move |e| opp_filter_search.set(event_target_value(&e))
                            />
                        </div>
                    }.into_any(),
                    _ => view! {}.into_any(),
                }}
            </div>

            // ── Data Tables ──
            <div class="table-container">
                <Suspense fallback=move || view! { <div class="p-8 text-center text-on-surface-variant">"Loading CRM data..."</div> }>
                    {move || match active_tab().as_str() {
                        "leads" => view! {
                            <table>
                                <thead>
                                    <tr>
                                        <th style="width:24px"><input type="checkbox" style="accent-color:var(--cobalt)"/></th>
                                        <th class="sortable">"Lead"</th>
                                        <th class="sortable">"Contact"</th>
                                        <th class="sortable">"Product"</th>
                                        <th class="sortable">"Source"</th>
                                        <th class="sortable">"G-27 Score"</th>
                                        <th class="sortable">"Stage"</th>
                                        <th class="sortable">"Tenant"</th>
                                        <th class="sortable">"Assigned"</th>
                                        <th class="sortable">"Last Activity"</th>
                                        <th></th>
                                    </tr>
                                </thead>
                                <tbody>
                                    {move || leads_res.get().unwrap_or_default().into_iter()
                                        .filter(|l| {
                                            let search = lead_filter_search.get().to_lowercase();
                                            let matches_search = search.is_empty() || l.name.to_lowercase().contains(&search) || l.email.as_ref().map(|e| e.to_lowercase().contains(&search)).unwrap_or(false);
                                            let stage = lead_filter_stage.get();
                                            let matches_stage = stage == "all" || l.status.as_deref().unwrap_or("New") == stage;
                                            matches_search && matches_stage
                                        })
                                        .map(|l| {
                                            let l_clone = l.clone();
                                            let initials = l.name.split_whitespace().map(|w| w.chars().next().unwrap_or('?')).collect::<String>().chars().take(2).collect::<String>().to_uppercase();
                                            let email = l.email.clone().unwrap_or_else(|| "-".to_string());
                                            let status = l.status.clone().unwrap_or_else(|| "New".to_string());
                                            
                                            // Mockup values for high-fidelity look
                                            let company = "Logística Meridional S.A.";
                                            let phone = "+55 11 9 9821-4430";
                                            let product = "Atlas PM — Residential";
                                            let source = "FMCSA";
                                            let score = "9.3";
                                            let tenant = "Nexus Property";
                                            let assigned = "R. Erie";
                                            let last_active = "12 min ago";
                                            
                                            view! {
                                                <tr on:click=move |_| selected_lead.set(Some(l_clone.clone()))>
                                                    <td><input type="checkbox" on:click=move |e| e.stop_propagation() style="accent-color:var(--cobalt)"/></td>
                                                    <td>
                                                        <div class="con-cell">
                                                            <div class="con-avatar" style="background:var(--cobalt-dim);color:var(--cobalt)">{initials}</div>
                                                            <div>
                                                                <div class="con-name-text">{l.name.clone()}</div>
                                                                <div class="con-title-sub">{company}</div>
                                                            </div>
                                                        </div>
                                                    </td>
                                                    <td>
                                                        <div class="contact-cell flex flex-col">
                                                            <span class="contact-email text-xs">{email}</span>
                                                            <span class="contact-phone text-[11px] text-muted">{phone}</span>
                                                        </div>
                                                    </td>
                                                    <td>
                                                        <div class="product-tag text-xs">
                                                            <span class="product-dot" style="display:inline-block;width:6px;height:6px;border-radius:50%;background:var(--green);margin-right:4px;"></span>
                                                            {product}
                                                        </div>
                                                    </td>
                                                    <td><span class="tag tag-fmcsa" style="color:var(--violet);border-color:var(--violet);background:var(--violet-dim)">{source}</span></td>
                                                    <td>
                                                        <div class="score-badge flex items-center gap-1.5">
                                                            <span class="score-dot" style="display:inline-block;width:6px;height:6px;border-radius:50%;background:var(--tier-outstanding)"></span>
                                                            <span class="mono text-xs">{score}</span>
                                                        </div>
                                                    </td>
                                                    <td>
                                                        <div class="stage-cell flex items-center gap-1.5">
                                                            <span class="stage-dot" style="display:inline-block;width:6px;height:6px;border-radius:50%;background:var(--cobalt)"></span>
                                                            {status}
                                                        </div>
                                                    </td>
                                                    <td class="text-secondary text-[11.5px]">{tenant}</td>
                                                    <td class="text-secondary text-[11.5px]">{assigned}</td>
                                                    <td class="muted">{last_active}</td>
                                                    <td>
                                                        <button class="btn btn-ghost btn-sm" on:click=move |e| {
                                                            e.stop_propagation();
                                                            selected_lead.set(Some(l.clone()));
                                                        }>"Open"</button>
                                                    </td>
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
                                        <th style="width:24px"><input type="checkbox" style="accent-color:var(--cobalt)"/></th>
                                        <th class="sortable">"Account"</th>
                                        <th class="sortable">"Type"</th>
                                        <th class="sortable">"Domain"</th>
                                        <th class="sortable">"Primary Contact"</th>
                                        <th class="sortable">"Scorecard (G-27)"</th>
                                        <th class="sortable">"Status"</th>
                                        <th class="sortable">"MRR"</th>
                                        <th class="sortable">"Created"</th>
                                        <th></th>
                                    </tr>
                                </thead>
                                <tbody>
                                    {move || accounts_res.get().unwrap_or_default().into_iter()
                                        .filter(|a| {
                                            let search = acc_filter_search.get().to_lowercase();
                                            let matches_search = search.is_empty() || a.name.to_lowercase().contains(&search);
                                            let f_type = acc_filter_type.get();
                                            let matches_type = f_type == "all" || (f_type == "Active" && a.name.contains("Nexus")) || f_type == "Organization";
                                            matches_search && matches_type
                                        })
                                        .map(|a| {
                                            let a_clone = a.clone();
                                            let initials = a.name.split_whitespace().map(|w| w.chars().next().unwrap_or('?')).collect::<String>().chars().take(2).collect::<String>().to_uppercase();
                                            
                                            // Mockup high-fidelity values
                                            let acct_type = "Organization";
                                            let domain = "nexusproperties.com";
                                            let contact = "Ruud Salym Erie";
                                            let score = "8.8";
                                            let status = "Active";
                                            let mrr = "$6,000";
                                            let created = "Feb 2024";

                                            view! {
                                                <tr on:click=move |_| selected_account.set(Some(a_clone.clone()))>
                                                    <td><input type="checkbox" on:click=move |e| e.stop_propagation() style="accent-color:var(--cobalt)"/></td>
                                                    <td>
                                                        <div class="con-cell">
                                                            <div class="con-avatar" style="background:var(--cobalt-dim);color:var(--cobalt)">{initials}</div>
                                                            <div class="con-name-text">{a.name.clone()}</div>
                                                        </div>
                                                    </td>
                                                    <td>{acct_type}</td>
                                                    <td class="mono">{domain}</td>
                                                    <td>{contact}</td>
                                                    <td>
                                                        <div class="score-badge flex items-center gap-1.5">
                                                            <span class="score-dot" style="display:inline-block;width:6px;height:6px;border-radius:50%;background:var(--tier-above)"></span>
                                                            <span class="mono text-xs">{score}</span>
                                                        </div>
                                                    </td>
                                                    <td><span class="tag tag-verified">{status}</span></td>
                                                    <td class="mono font-semibold">{mrr}</td>
                                                    <td class="muted">{created}</td>
                                                    <td>
                                                        <button class="btn btn-ghost btn-sm" on:click=move |e| e.stop_propagation()>"Assign"</button>
                                                    </td>
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
                                        <th style="width:24px"><input type="checkbox" style="accent-color:var(--cobalt)"/></th>
                                        <th class="sortable">"Contact"</th>
                                        <th class="sortable">"Account Name"</th>
                                        <th class="sortable">"Email"</th>
                                        <th class="sortable">"Phone"</th>
                                        <th class="sortable">"Verification (G-06)"</th>
                                        <th class="sortable">"Last Active"</th>
                                        <th class="sortable">"Created"</th>
                                        <th></th>
                                    </tr>
                                </thead>
                                <tbody>
                                    {move || contacts_res.get().unwrap_or_default().into_iter()
                                        .filter(|c| {
                                            let search = con_filter_search.get().to_lowercase();
                                            let matches_search = search.is_empty() || c.name.to_lowercase().contains(&search) || c.email.as_ref().map(|e| e.to_lowercase().contains(&search)).unwrap_or(false);
                                            let ver = con_filter_verification.get();
                                            let matches_ver = ver == "all" || ver == "Verified";
                                            matches_search && matches_ver
                                        })
                                        .map(|c| {
                                            let c_clone = c.clone();
                                            let initials = c.name.split_whitespace().map(|w| w.chars().next().unwrap_or('?')).collect::<String>().chars().take(2).collect::<String>().to_uppercase();
                                            let email = c.email.clone().unwrap_or_else(|| "-".to_string());
                                            let phone = c.phone.clone().unwrap_or_else(|| "-".to_string());
                                            
                                            // Mockup high-fidelity values
                                            let account_name = "Logística Meridional S.A.";
                                            let job_title = "Regional VP, Logistics";
                                            let status = "Verified";
                                            let last_active = "2 hours ago";
                                            let created = "Oct 2024";

                                            view! {
                                                <tr on:click=move |_| selected_contact.set(Some(c_clone.clone()))>
                                                    <td><input type="checkbox" on:click=move |e| e.stop_propagation() style="accent-color:var(--cobalt)"/></td>
                                                    <td>
                                                        <div class="con-cell">
                                                            <div class="con-avatar" style="background:var(--cobalt-dim);color:var(--cobalt)">{initials}</div>
                                                            <div>
                                                                <div class="con-name-text">{c.name.clone()}</div>
                                                                <div class="con-title-sub">{job_title}</div>
                                                            </div>
                                                        </div>
                                                    </td>
                                                    <td>{account_name}</td>
                                                    <td class="mono">{email}</td>
                                                    <td class="muted">{phone}</td>
                                                    <td><span class="tag tag-verified">{status}</span></td>
                                                    <td class="muted">{last_active}</td>
                                                    <td class="muted">{created}</td>
                                                    <td>
                                                        <button class="btn btn-ghost btn-sm" on:click=move |e| e.stop_propagation()>"Email"</button>
                                                    </td>
                                                </tr>
                                            }
                                        }).collect_view()}
                                </tbody>
                            </table>
                        }.into_any(),

                        "opportunities" => view! {
                            <table>
                                <thead>
                                    <tr>
                                        <th style="width:24px"><input type="checkbox" style="accent-color:var(--cobalt)"/></th>
                                        <th class="sortable">"Opportunity"</th>
                                        <th class="sortable">"Account Name"</th>
                                        <th class="sortable">"Stage"</th>
                                        <th class="sortable">"Value"</th>
                                        <th class="sortable">"Probability"</th>
                                        <th class="sortable">"Est. Close"</th>
                                        <th class="sortable">"Owner"</th>
                                        <th></th>
                                    </tr>
                                </thead>
                                <tbody>
                                    {move || deals_res.get().unwrap_or_default().into_iter()
                                        .filter(|d| {
                                            let search = opp_filter_search.get().to_lowercase();
                                            let matches_search = search.is_empty() || d.name.to_lowercase().contains(&search);
                                            let stage = opp_filter_stage.get();
                                            let matches_stage = stage == "all" || d.stage == stage;
                                            matches_search && matches_stage
                                        })
                                        .map(|d| {
                                            let d_clone = d.clone();
                                            let initials = d.name.split_whitespace().map(|w| w.chars().next().unwrap_or('?')).collect::<String>().chars().take(2).collect::<String>().to_uppercase();
                                            
                                            // Mockup high-fidelity values
                                            let account_name = "Logística Meridional S.A.";
                                            let probability = "75%";
                                            let est_close = "Jul 15, 2026";
                                            let owner = "Jamie Delaney";
                                            let amount = format!("${:.2}", d.amount);

                                            let stage_class = match d.stage.as_str() {
                                                "Proposal" => "tag-proposal",
                                                "Negotiation" => "tag-negotiation",
                                                "Closed Won" => "tag-won",
                                                "Closed Lost" => "tag-lost",
                                                _ => "tag-qualify"
                                            };

                                            let is_won = d.stage == "Closed Won";

                                            view! {
                                                <tr on:click=move |_| {
                                                    if d_clone.stage.contains("Negotiation") || d_clone.stage.contains("Won") {
                                                        set_show_milestone_modal.set(true);
                                                    }
                                                    selected_deal.set(Some(d_clone.clone()));
                                                }>
                                                    <td><input type="checkbox" on:click=move |e| e.stop_propagation() style="accent-color:var(--cobalt)"/></td>
                                                    <td>
                                                        <div class="con-cell">
                                                            <div class="con-avatar" style="background:var(--amber-dim);color:var(--amber)">{initials}</div>
                                                            <div>
                                                                <div class="con-name-text">{d.name.clone()}</div>
                                                                <div class="con-title-sub">{account_name}</div>
                                                            </div>
                                                        </div>
                                                    </td>
                                                    <td>{account_name}</td>
                                                    <td><span class=format!("tag {}", stage_class)>{d.stage.clone()}</span></td>
                                                    <td class="mono font-semibold" style=if is_won { "color:var(--green);" } else { "" }>{amount}</td>
                                                    <td class="muted mono">{probability}</td>
                                                    <td class="muted">{est_close}</td>
                                                    <td>{owner}</td>
                                                    <td>
                                                        <button class="btn btn-ghost btn-sm" on:click=move |e| e.stop_propagation()>"Log"</button>
                                                    </td>
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

            // ── Leads Details Drawer ──
            <div class=move || format!("panel-backdrop {}", if selected_lead.get().is_some() { "open" } else { "" }) on:click=move |_| selected_lead.set(None)></div>
            <div class=move || format!("detail-panel {}", if selected_lead.get().is_some() { "open" } else { "" })>
                {move || selected_lead.get().map(|l| view! {
                    <div class="panel-header">
                        <div class="panel-header-top">
                            <div class="panel-identity">
                                <div class="panel-title-text">{l.name.clone()}</div>
                                <div class="panel-subtitle-text">{l.email.clone().unwrap_or_default()}</div>
                            </div>
                            <button class="panel-close" on:click=move |_| selected_lead.set(None)>
                                <svg viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.5" style="width:14px;height:14px;">
                                    <line x1="4" y1="4" x2="12" y2="12"/><line x1="12" y1="4" x2="4" y2="12"/>
                                </svg>
                            </button>
                        </div>
                        <div class="panel-actions">
                            <a href=format!("/crm/lead/{}", l.id) class="btn btn-ghost btn-sm" style="text-decoration:none">"View Full Record"</a>
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
                                <div class="detail-value">{l.status.clone().unwrap_or_else(|| "New".to_string())}</div>
                            </div>
                            <div class="detail-field">
                                <div class="detail-label">"Converted"</div>
                                <div class="detail-value">{l.is_converted.to_string()}</div>
                            </div>
                        </div>
                    </div>
                })}
            </div>

            // ── Accounts Details Drawer ──
            <div class=move || format!("panel-backdrop {}", if selected_account.get().is_some() { "open" } else { "" }) on:click=move |_| selected_account.set(None)></div>
            <div class=move || format!("detail-panel {}", if selected_account.get().is_some() { "open" } else { "" })>
                {move || selected_account.get().map(|a| view! {
                    <div class="panel-header">
                        <div class="panel-header-top">
                            <div class="panel-identity">
                                <div class="panel-title-text">{a.name.clone()}</div>
                                <div class="panel-subtitle-text">"Nexus Property Group"</div>
                            </div>
                            <button class="panel-close" on:click=move |_| selected_account.set(None)>
                                <svg viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.5" style="width:14px;height:14px;">
                                    <line x1="4" y1="4" x2="12" y2="12"/><line x1="12" y1="4" x2="4" y2="12"/>
                                </svg>
                            </button>
                        </div>
                        <div class="panel-actions">
                            <a href=format!("/crm/account/{}", a.id) class="btn btn-ghost btn-sm" style="text-decoration:none">"View Full Record"</a>
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
                                <div class="detail-value">{a.name.clone()}</div>
                            </div>
                        </div>
                    </div>
                })}
            </div>

            // ── Contacts Details Drawer ──
            <div class=move || format!("panel-backdrop {}", if selected_contact.get().is_some() { "open" } else { "" }) on:click=move |_| selected_contact.set(None)></div>
            <div class=move || format!("detail-panel {}", if selected_contact.get().is_some() { "open" } else { "" })>
                {move || selected_contact.get().map(|c| view! {
                    <div class="panel-header">
                        <div class="panel-header-top">
                            <div class="panel-identity">
                                <div class="panel-title-text">{c.name.clone()}</div>
                                <div class="panel-subtitle-text">{c.email.clone().unwrap_or_default()}</div>
                            </div>
                            <button class="panel-close" on:click=move |_| selected_contact.set(None)>
                                <svg viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.5" style="width:14px;height:14px;">
                                    <line x1="4" y1="4" x2="12" y2="12"/><line x1="12" y1="4" x2="4" y2="12"/>
                                </svg>
                            </button>
                        </div>
                        <div class="panel-actions">
                            <a href=format!("/crm/contact/{}", c.id) class="btn btn-ghost btn-sm" style="text-decoration:none">"View Full Record"</a>
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
                                <div class="detail-value">{c.phone.clone().unwrap_or_default()}</div>
                            </div>
                        </div>
                    </div>
                })}
            </div>

            // ── Opportunities/Deals Details Drawer ──
            <div class=move || format!("panel-backdrop {}", if selected_deal.get().is_some() { "open" } else { "" }) on:click=move |_| selected_deal.set(None)></div>
            <div class=move || format!("detail-panel {}", if selected_deal.get().is_some() { "open" } else { "" })>
                {move || selected_deal.get().map(|d| view! {
                    <div class="panel-header">
                        <div class="panel-header-top">
                            <div class="panel-identity">
                                <div class="panel-title-text">{d.name.clone()}</div>
                                <div class="panel-subtitle-text">"Logística Meridional S.A."</div>
                            </div>
                            <button class="panel-close" on:click=move |_| selected_deal.set(None)>
                                <svg viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.5" style="width:14px;height:14px;">
                                    <line x1="4" y1="4" x2="12" y2="12"/><line x1="12" y1="4" x2="4" y2="12"/>
                                </svg>
                            </button>
                        </div>
                        <div class="panel-actions">
                            <a href=format!("/crm/deal/{}", d.id) class="btn btn-ghost btn-sm" style="text-decoration:none">"View Full Record"</a>
                            <button class="btn btn-primary btn-sm" on:click=move |_| {
                                toast.message.set(Some("Deal closed won.".to_string()));
                                selected_deal.set(None);
                            }>"Close Won"</button>
                        </div>
                        <div class="panel-tabs">
                            <button class="panel-tab active">"Overview"</button>
                        </div>
                    </div>
                    <div class="panel-content" style="padding: 16px 20px;">
                        <div class="detail-grid">
                            <span class="detail-section-label">"Deal Details"</span>
                            <div class="detail-field">
                                <div class="detail-label">"Value"</div>
                                <div class="detail-value mono">{format!("${:.2}", d.amount)}</div>
                            </div>
                            <div class="detail-field">
                                <div class="detail-label">"Stage"</div>
                                <div class="detail-value">{d.stage.clone()}</div>
                            </div>
                            <div class="detail-field">
                                <div class="detail-label">"Win Probability"</div>
                                <div class="detail-value mono">"75%"</div>
                            </div>
                            <div class="detail-field">
                                <div class="detail-label">"Deal Owner"</div>
                                <div class="detail-value">"Jamie Delaney"</div>
                            </div>
                        </div>
                    </div>
                })}
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
