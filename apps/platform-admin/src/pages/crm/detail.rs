use leptos::prelude::*;
use leptos_router::hooks::use_params_map;

use crate::components::upsell_banner::UpsellBanner;
use crate::components::recommended_partners::RecommendedPartners;

use serde::{Serialize, Deserialize};
use crate::api::crm::{
    get_user_by_id, get_lead_by_id, get_account_by_id, get_deal_by_id, get_contact_by_id,
    update_contact, convert_lead, update_lead, get_contact_notes, add_contact_note,
    get_contact_activities, log_contact_activity, get_crm_status_options
};
use crate::api::models::{UserInfo, LeadModel, AccountModel, DealModel, ContactModel, CreateContact};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum EntityDetail {
    User(UserInfo),
    Lead(LeadModel),
    Contact(ContactModel),
    Account(AccountModel),
    Deal(DealModel),
    Unknown,
}

#[component]
pub fn CrmDetail() -> impl IntoView {
    let params = use_params_map();
    let toast = use_context::<crate::app::GlobalToast>().expect("toast context");
    
    let entity_type = move || params.get().get("entity").unwrap_or_default().to_string();
    let record_id = move || params.get().get("id").unwrap_or_default().to_string();

    let (trigger_refresh, set_trigger_refresh) = signal(0);
    let active_tab = RwSignal::new("overview".to_string());
    let note_content = RwSignal::new("".to_string());

    let details_res = LocalResource::new(
        move || {
            trigger_refresh.get();
            let entity = entity_type();
            let id = record_id();
            async move {
                match entity.as_str() {
                    "user" => get_user_by_id(&id).await.map(EntityDetail::User).unwrap_or(EntityDetail::Unknown),
                    "lead" => get_lead_by_id(&id).await.map(EntityDetail::Lead).unwrap_or(EntityDetail::Unknown),
                    "contact" => get_contact_by_id(&id).await.map(EntityDetail::Contact).unwrap_or(EntityDetail::Unknown),
                    "account" => get_account_by_id(&id).await.map(EntityDetail::Account).unwrap_or(EntityDetail::Unknown),
                    "deal" => get_deal_by_id(&id).await.map(EntityDetail::Deal).unwrap_or(EntityDetail::Unknown),
                    _ => EntityDetail::Unknown,
                }
            }
        }
    );

    let status_options_res = LocalResource::new(move || async move {
        get_crm_status_options("Lead").await.unwrap_or_default()
    });

    let notes_res = LocalResource::new(move || {
        trigger_refresh.get();
        let entity = entity_type();
        let id = record_id();
        async move {
            if entity == "contact" || entity == "lead" {
                get_contact_notes(&id).await.unwrap_or_default()
            } else {
                Vec::new()
            }
        }
    });

    let activities_res = LocalResource::new(move || {
        trigger_refresh.get();
        let entity = entity_type();
        let id = record_id();
        async move {
            if entity == "contact" || entity == "lead" {
                get_contact_activities(&id).await.unwrap_or_default()
            } else {
                Vec::new()
            }
        }
    });

    let handle_convert_lead = move |_| {
        let id = record_id();
        let toast = toast.clone();
        let navigate = leptos_router::hooks::use_navigate();
        leptos::task::spawn_local(async move {
            match convert_lead(&id).await {
                Ok(contact) => {
                    toast.message.set(Some("Lead qualified and converted to Contact!".to_string()));
                    navigate(&format!("/crm/contact/{}", contact.id), Default::default());
                }
                Err(e) => {
                    toast.message.set(Some(format!("Failed to convert lead: {}", e)));
                }
            }
        });
    };

    let handle_add_note = move |_| {
        let id = record_id();
        let content = note_content.get();
        if content.is_empty() { return; }
        let toast = toast.clone();
        leptos::task::spawn_local(async move {
            match add_contact_note(&id, &content).await {
                Ok(_) => {
                    toast.message.set(Some("Note added successfully!".to_string()));
                    note_content.set("".to_string());
                    set_trigger_refresh.update(|v| *v += 1);
                }
                Err(e) => {
                    toast.message.set(Some(format!("Failed to add note: {}", e)));
                }
            }
        });
    };

    let record_name = move || match details_res.get() {
        Some(EntityDetail::Lead(ref l)) => l.name.clone(),
        Some(EntityDetail::Contact(ref c)) => c.name.clone(),
        Some(EntityDetail::User(ref u)) => format!("{} {}", u.first_name, u.last_name),
        Some(EntityDetail::Account(ref a)) => a.name.clone(),
        Some(EntityDetail::Deal(ref d)) => d.name.clone(),
        _ => "Record Details".to_string(),
    };

    let avatar_initials = move || {
        let name = record_name();
        name.split_whitespace().map(|w| w.chars().next().unwrap_or('?')).collect::<String>().chars().take(2).collect::<String>().to_uppercase()
    };

    view! {
        <div class="main-area" style="overflow-y: auto;">
            <Suspense fallback=move || view! { <div class="p-8 text-center text-on-surface-variant">"Loading details..."</div> }>
                // ── Record Header ──
                <div class="rec-hdr">
                    <div class="breadcrumb">
                        <a href="/crm">"CRM"</a>" › "
                        <span class="capitalize">{move || entity_type() + "s"}</span>" › "
                        {record_name}
                    </div>
                    <div class="rec-identity">
                        <div class="rec-left">
                            <div class="lead-avatar">{avatar_initials}</div>
                            <div>
                                <div class="rec-name">
                                    {record_name}
                                    <span class="tag" style="color:var(--cobalt);border-color:var(--cobalt)">"Qualifying"</span>
                                    <span class="tag" style="color:var(--amber);border-color:var(--amber)">"Not Converted"</span>
                                    <span class="source-badge" style="color:var(--violet);border-color:var(--violet);background:var(--violet-dim)">"⚙ FMCSA Import"</span>
                                </div>
                                <div class="rec-meta">
                                    "atlas_" {move || entity_type()} " · G-31 · " {move || record_id()} " · VP Operations · Rio de Janeiro, Brazil"
                                </div>
                                <div class="rec-actions-row">
                                    <button class="btn btn-ghost btn-sm">"✉ Email"</button>
                                    <button class="btn btn-ghost btn-sm">"📞 Log Call"</button>
                                    <button class="btn btn-ghost btn-sm">"💬 WhatsApp"</button>
                                    <button class="btn btn-convert btn-sm" on:click=handle_convert_lead>"⇉ Convert Lead"</button>
                                </div>
                            </div>
                        </div>
                        <div style="text-align:right;font-size:11px;color:var(--text-muted)">
                            <div>"Lead Owner: Maria Fernandes"</div>
                            <div style="margin-top:3px">"G-27 Score: "<span style="color:#88CC00;font-weight:600">"7.2"</span></div>
                        </div>
                    </div>
                </div>

                // ── Status Flow ──
                <div class="status-flow">
                    <div class="sf-step done"><div class="sf-pill">"New"</div></div>
                    <div class="sf-arrow">"→"</div>
                    <div class="sf-step done"><div class="sf-pill">"Contacted"</div></div>
                    <div class="sf-arrow">"→"</div>
                    <div class="sf-step current"><div class="sf-pill">"Qualifying"</div></div>
                    <div class="sf-arrow">"→"</div>
                    <div class="sf-step future"><div class="sf-pill">"Qualified"</div></div>
                    <div class="sf-arrow">"→"</div>
                    <div class="sf-step terminal-won" style="opacity:0.4"><div class="sf-pill">"Converted"</div></div>
                </div>

                // ── KPI Strip ──
                <div class="kpi-strip">
                    <div class="kpi"><div class="kpi-label">"Annual Revenue"</div><div class="kpi-value mono" style="color:var(--green)">"$42M"</div><div class="kpi-sub">"FMCSA verified"</div></div>
                    <div class="kpi"><div class="kpi-label">"Employees"</div><div class="kpi-value mono">"340"</div><div class="kpi-sub">"Fleet: 87 units"</div></div>
                    <div class="kpi"><div class="kpi-label">"Activities"</div><div class="kpi-value mono">"5"</div><div class="kpi-sub">"2 calls · 1 meeting"</div></div>
                    <div class="kpi"><div class="kpi-label">"G-27 Score"</div><div class="kpi-value" style="font-size:15px;color:#88CC00">"7.2"</div><div class="kpi-sub">"Above Bar"</div></div>
                </div>

                // ── Tabs Bar ──
                <div class="tab-bar">
                    <button class=move || format!("tab {}", if active_tab.get() == "overview" { "active" } else { "" }) on:click=move |_| active_tab.set("overview".to_string())>"Overview"</button>
                    <button class=move || format!("tab {}", if active_tab.get() == "details" { "active" } else { "" }) on:click=move |_| active_tab.set("details".to_string())>"All Fields · G-31"</button>
                    <button class=move || format!("tab {}", if active_tab.get() == "activity" { "active" } else { "" }) on:click=move |_| active_tab.set("activity".to_string())>"Activity · G-29"</button>
                </div>

                // ── Tab Content Area ──
                <div class="content-body">
                    {move || match active_tab.get().as_str() {
                        "overview" => view! {
                            <div class="col-7-5">
                                // Left Column
                                <div>
                                    <div class="convert-panel">
                                        <div class="convert-panel-hdr">"⇉ This lead is ready to convert"</div>
                                        <div class="convert-panel-body">
                                            "João Silva has been qualified: 3 calls connected, on-site demo completed, CFO introduced. Converting will atomically create an Account, a Contact, and an Opportunity."
                                        </div>
                                        <button class="btn btn-convert btn-sm" on:click=handle_convert_lead>"Qualification Conversion →"</button>
                                    </div>

                                    // Composer
                                    <div class="card">
                                        <div class="composer">
                                            <div class="composer-tabs">
                                                <button class="c-tab active">"Note"</button>
                                            </div>
                                            <textarea 
                                                class="w-full bg-surface-elevated border border-border-default rounded p-2 text-sm text-text-primary"
                                                placeholder="Log activity on this record..."
                                                prop:value=move || note_content.get()
                                                on:input=move |e| note_content.set(event_target_value(&e))
                                            ></textarea>
                                            <div class="composer-footer">
                                                <button class="btn btn-primary btn-sm" on:click=handle_add_note>"Save Note"</button>
                                            </div>
                                        </div>
                                    </div>

                                    // Activity feed
                                    <div class="card">
                                        <div class="card-hdr">
                                            <span class="card-title">"Activity Timeline · G-29"</span>
                                        </div>
                                        <div>
                                            {move || notes_res.get().unwrap_or_default().into_iter().map(|n| view! {
                                                <div class="activity-item">
                                                    <div class="act-icon" style="background:var(--cobalt-dim);color:var(--cobalt)">"📝"</div>
                                                    <div class="act-body">
                                                        <div class="act-title">"Internal Note added"</div>
                                                        <div class="act-meta">"System User · " {n.created_at}</div>
                                                        <div class="act-desc">{n.content}</div>
                                                    </div>
                                                </div>
                                            }).collect_view()}
                                            {move || activities_res.get().unwrap_or_default().into_iter().map(|a| view! {
                                                <div class="activity-item">
                                                    <div class="act-icon" style="background:var(--green-dim);color:var(--green)">"🤝"</div>
                                                    <div class="act-body">
                                                        <div class="act-title">{a.activity_type}</div>
                                                        <div class="act-meta">"System User · " {a.created_at}</div>
                                                        <div class="act-desc">{a.description}</div>
                                                    </div>
                                                </div>
                                            }).collect_view()}
                                        </div>
                                    </div>
                                </div>

                                // Right Column
                                <div>
                                    <div class="card">
                                        <div class="card-hdr"><span class="card-title">"Lead Info"</span></div>
                                        <div class="stat-row"><span class="s-label">"Status"</span><span class="s-value cobalt">"Qualifying"</span></div>
                                        <div class="stat-row"><span class="s-label">"Source"</span><span class="s-value"><span class="pill" style="color:var(--violet);border-color:var(--violet)">"FMCSA Import"</span></span></div>
                                        <div class="stat-row"><span class="s-label">"Lead Owner"</span><span class="s-value">"Maria Fernandes"</span></div>
                                    </div>

                                    <div class="card">
                                        <div class="card-hdr"><span class="card-title">"Data Quality"</span></div>
                                        <div class="stat-row"><span class="s-label">"Email verified"</span><span class="s-value green">"✓"</span></div>
                                        <div class="stat-row"><span class="s-label">"Phone verified"</span><span class="s-value green">"✓"</span></div>
                                        <div class="stat-row"><span class="s-label">"Completeness"</span><span class="s-value green">"94%"</span></div>
                                    </div>
                                </div>
                            </div>
                        }.into_any(),

                        "details" => view! {
                            <div class="card">
                                <div class="card-hdr"><span class="card-title">"All Fields · G-31"</span></div>
                                <div class="field-grid">
                                    <div class="field-row"><span class="f-label">"ID"</span><span class="f-value mono">{move || record_id()}</span></div>
                                    <div class="field-row"><span class="f-label">"Name"</span><span class="f-value">{record_name}</span></div>
                                    <div class="field-row"><span class="f-label">"Type"</span><span class="f-value capitalize">{move || entity_type()}</span></div>
                                </div>
                            </div>
                        }.into_any(),

                        _ => view! { <div></div> }.into_any(),
                    }}
                </div>
            </Suspense>
        </div>
    }
}
