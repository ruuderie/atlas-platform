use leptos::prelude::*;
use shared_ui::components::crm_stage_bar::{CrmStageBar, CrmStatusOption};
use shared_ui::components::crm_timeline::{CrmTimeline, CrmNote, CrmActivity};
use shared_ui::utils::ResourceState;

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq)]
pub struct LeadRecord {
    pub id: uuid::Uuid,
    pub name: String,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub company: Option<String>,
    pub title: Option<String>,
    pub lead_status: Option<String>,
    pub message: Option<String>,
    pub source: Option<String>,
    pub is_converted: bool,
    pub created_at: String,
}

#[server(GetLeads, "/api")]
pub async fn get_leads() -> Result<Vec<LeadRecord>, ServerFnError> {
    use axum::Extension;
    use leptos_axum::extract;
    use crate::auth::check_session;

    if !check_session().await.unwrap_or(false) {
        return Err(ServerFnError::ServerError("Unauthorized".into()));
    }

    let Extension(state) = extract::<Extension<crate::state::AppState>>().await?;
    let Extension(tenant) = extract::<Extension<crate::state::TenantContext>>().await?;

    let rows = sqlx::query(
        "SELECT id, name, first_name, last_name, email, phone, company, title, lead_status, message, source, is_converted, created_at \
         FROM lead \
         WHERE tenant_id = $1 \
         ORDER BY created_at DESC"
    )
    .bind(tenant.0)
    .fetch_all(&state.pool)
    .await?;

    let items = rows
        .into_iter()
        .map(|row| {
            use sqlx::Row;
            let created_at: chrono::DateTime<chrono::Utc> = row.get("created_at");
            LeadRecord {
                id: row.get("id"),
                name: row.get("name"),
                first_name: row.try_get("first_name").unwrap_or(None),
                last_name: row.try_get("last_name").unwrap_or(None),
                email: row.try_get("email").unwrap_or(None),
                phone: row.try_get("phone").unwrap_or(None),
                company: row.try_get("company").unwrap_or(None),
                title: row.try_get("title").unwrap_or(None),
                lead_status: row.try_get("lead_status").unwrap_or(None),
                message: row.try_get("message").unwrap_or(None),
                source: row.try_get("source").unwrap_or(None),
                is_converted: row.get("is_converted"),
                created_at: created_at.format("%Y-%m-%d %H:%M:%S").to_string(),
            }
        })
        .collect();

    Ok(items)
}

#[server(DeleteLead, "/api")]
pub async fn delete_lead(id: uuid::Uuid) -> Result<(), ServerFnError> {
    use axum::Extension;
    use leptos_axum::extract;
    use crate::auth::check_session;

    if !check_session().await.unwrap_or(false) {
        return Err(ServerFnError::ServerError("Unauthorized".into()));
    }

    let Extension(state) = extract::<Extension<crate::state::AppState>>().await?;
    let Extension(tenant) = extract::<Extension<crate::state::TenantContext>>().await?;

    sqlx::query("DELETE FROM lead WHERE id = $1 AND tenant_id = $2")
        .bind(id)
        .bind(tenant.0)
        .execute(&state.pool)
        .await?;

    Ok(())
}

#[server(ConvertLead, "/api")]
pub async fn convert_lead(id: uuid::Uuid) -> Result<(), ServerFnError> {
    use axum::Extension;
    use leptos_axum::extract;
    use crate::auth::check_session;

    if !check_session().await.unwrap_or(false) {
        return Err(ServerFnError::ServerError("Unauthorized".into()));
    }

    let Extension(state) = extract::<Extension<crate::state::AppState>>().await?;
    let Extension(tenant) = extract::<Extension<crate::state::TenantContext>>().await?;

    // 1. Fetch Lead
    let lead_row = sqlx::query(
        "SELECT name, first_name, last_name, email, phone, whatsapp, telegram, twitter, instagram, facebook, billing_address, shipping_address, is_converted \
         FROM lead WHERE id = $1 AND tenant_id = $2"
    )
    .bind(id)
    .bind(tenant.0)
    .fetch_optional(&state.pool)
    .await?;

    let Some(row) = lead_row else {
        return Err(ServerFnError::ServerError("Lead not found".into()));
    };

    use sqlx::Row;
    let is_converted: bool = row.get("is_converted");
    if is_converted {
        return Err(ServerFnError::ServerError("Lead already converted".into()));
    }

    let name: String = row.get("name");
    let first_name: Option<String> = row.try_get("first_name").unwrap_or(None);
    let last_name: Option<String> = row.try_get("last_name").unwrap_or(None);
    let email: Option<String> = row.try_get("email").unwrap_or(None);
    let phone: Option<String> = row.try_get("phone").unwrap_or(None);
    let whatsapp: Option<String> = row.try_get("whatsapp").unwrap_or(None);
    let telegram: Option<String> = row.try_get("telegram").unwrap_or(None);
    let twitter: Option<String> = row.try_get("twitter").unwrap_or(None);
    let instagram: Option<String> = row.try_get("instagram").unwrap_or(None);
    let facebook: Option<String> = row.try_get("facebook").unwrap_or(None);
    let billing_address: Option<serde_json::Value> = row.try_get("billing_address").unwrap_or(None);
    let shipping_address: Option<serde_json::Value> = row.try_get("shipping_address").unwrap_or(None);

    // Start SQLx transaction
    let mut tx = state.pool.begin().await?;

    // 2. Check for duplicate Contact by email or phone
    let mut duplicate_contact_id: Option<uuid::Uuid> = None;
    if let Some(ref email_str) = email {
        if !email_str.is_empty() {
            duplicate_contact_id = sqlx::query_scalar(
                "SELECT id FROM contact WHERE tenant_id = $1 AND email = $2 LIMIT 1"
            )
            .bind(tenant.0)
            .bind(email_str)
            .fetch_optional(&mut *tx)
            .await?;
        }
    }

    if duplicate_contact_id.is_none() {
        if let Some(ref phone_str) = phone {
            if !phone_str.is_empty() {
                duplicate_contact_id = sqlx::query_scalar(
                    "SELECT id FROM contact WHERE tenant_id = $1 AND phone = $2 LIMIT 1"
                )
                .bind(tenant.0)
                .bind(phone_str)
                .fetch_optional(&mut *tx)
                .await?;
            }
        }
    }

    let contact_id = if let Some(cid) = duplicate_contact_id {
        cid
    } else {
        let new_contact_id = uuid::Uuid::new_v4();
        sqlx::query(
            "INSERT INTO contact (id, customer_id, name, first_name, last_name, email, phone, whatsapp, telegram, twitter, instagram, facebook, billing_address, shipping_address, tenant_id, created_at, updated_at) \
             VALUES ($1, NULL, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, NOW(), NOW())"
        )
        .bind(new_contact_id)
        .bind(name)
        .bind(first_name)
        .bind(last_name)
        .bind(email)
        .bind(phone)
        .bind(whatsapp)
        .bind(telegram)
        .bind(twitter)
        .bind(instagram)
        .bind(facebook)
        .bind(billing_address)
        .bind(shipping_address)
        .bind(tenant.0)
        .execute(&mut *tx)
        .await?;
        new_contact_id
    };

    // 3. Update Lead Status
    sqlx::query(
        "UPDATE lead SET is_converted = true, converted_to_contact = true, converted_contact_id = $1, lead_status = 'Converted', updated_at = NOW() \
         WHERE id = $2 AND tenant_id = $3"
    )
    .bind(contact_id)
    .bind(id)
    .bind(tenant.0)
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;

    Ok(())
}

#[server(GetLeadCrmStatuses, "/api")]
pub async fn get_lead_crm_statuses() -> Result<Vec<CrmStatusOption>, ServerFnError> {
    use axum::Extension;
    use leptos_axum::extract;
    use sqlx::Row;
    let Extension(state) = extract::<Extension<crate::state::AppState>>().await?;
    let Extension(tenant) = extract::<Extension<crate::state::TenantContext>>().await?;

    let rows = sqlx::query(
        "SELECT status_key, label, color, sort_order, is_system \
         FROM crm_status_option \
         WHERE tenant_id = $1 AND object_type = 'Lead' \
         ORDER BY sort_order ASC"
    )
    .bind(tenant.0)
    .fetch_all(&state.pool)
    .await?;

    let options = rows
        .into_iter()
        .map(|row| CrmStatusOption {
            status_key: row.get("status_key"),
            label: row.get("label"),
            color: row.get("color"),
            sort_order: row.get("sort_order"),
            is_system: row.get("is_system"),
        })
        .collect();

    Ok(options)
}

#[server(UpdateLeadStage, "/api")]
pub async fn update_lead_stage(id: uuid::Uuid, stage: String) -> Result<(), ServerFnError> {
    use axum::Extension;
    use leptos_axum::extract;
    use crate::auth::check_session;

    if !check_session().await.unwrap_or(false) {
        return Err(ServerFnError::ServerError("Unauthorized".into()));
    }

    let Extension(state) = extract::<Extension<crate::state::AppState>>().await?;
    let Extension(tenant) = extract::<Extension<crate::state::TenantContext>>().await?;

    sqlx::query(
        "UPDATE lead SET lead_status = $1, updated_at = NOW() \
         WHERE id = $2 AND tenant_id = $3"
    )
    .bind(stage)
    .bind(id)
    .bind(tenant.0)
    .execute(&state.pool)
    .await?;

    Ok(())
}

#[server(UpdateLeadDetails, "/api")]
pub async fn update_lead_details(
    id: uuid::Uuid,
    name: String,
    first_name: Option<String>,
    last_name: Option<String>,
    email: Option<String>,
    phone: Option<String>,
    company: Option<String>,
    title: Option<String>,
    source: Option<String>,
    message: Option<String>,
) -> Result<(), ServerFnError> {
    use axum::Extension;
    use leptos_axum::extract;
    use crate::auth::check_session;

    if !check_session().await.unwrap_or(false) {
        return Err(ServerFnError::ServerError("Unauthorized".into()));
    }

    let Extension(state) = extract::<Extension<crate::state::AppState>>().await?;
    let Extension(tenant) = extract::<Extension<crate::state::TenantContext>>().await?;

    sqlx::query(
        "UPDATE lead SET name = $1, first_name = $2, last_name = $3, email = $4, phone = $5, company = $6, title = $7, source = $8, message = $9, updated_at = NOW() \
         WHERE id = $10 AND tenant_id = $11"
    )
    .bind(name)
    .bind(first_name)
    .bind(last_name)
    .bind(email)
    .bind(phone)
    .bind(company)
    .bind(title)
    .bind(source)
    .bind(message)
    .bind(id)
    .bind(tenant.0)
    .execute(&state.pool)
    .await?;

    Ok(())
}

#[server(GetLeadNotes, "/api")]
pub async fn get_lead_notes(lead_id: uuid::Uuid) -> Result<Vec<CrmNote>, ServerFnError> {
    use axum::Extension;
    use leptos_axum::extract;
    let Extension(state) = extract::<Extension<crate::state::AppState>>().await?;

    let rows = sqlx::query(
        "SELECT id, content, created_at \
         FROM notes \
         WHERE entity_type = 'Lead' AND entity_id = $1 \
         ORDER BY created_at DESC"
    )
    .bind(lead_id)
    .fetch_all(&state.pool)
    .await?;

    use sqlx::Row;
    let notes = rows
        .into_iter()
        .map(|row| {
            let created_at: chrono::DateTime<chrono::Utc> = row.get("created_at");
            CrmNote {
                id: row.get("id"),
                content: row.get("content"),
                created_at: created_at.format("%Y-%m-%d %H:%M").to_string(),
            }
        })
        .collect();

    Ok(notes)
}

#[server(AddLeadNote, "/api")]
pub async fn add_lead_note(lead_id: uuid::Uuid, content: String) -> Result<(), ServerFnError> {
    use axum::Extension;
    use leptos_axum::extract;
    use crate::auth::check_session;
    let Extension(state) = extract::<Extension<crate::state::AppState>>().await?;
    
    if !check_session().await.unwrap_or(false) {
        return Err(ServerFnError::ServerError("Unauthorized".into()));
    }
    
    let user_id: uuid::Uuid = sqlx::query_scalar("SELECT id FROM \"user\" LIMIT 1")
        .fetch_one(&state.pool)
        .await?;

    sqlx::query(
        "INSERT INTO notes (id, content, created_by, entity_type, entity_id, created_at, updated_at) \
         VALUES ($1, $2, $3, 'Lead', $4, NOW(), NOW())"
    )
    .bind(uuid::Uuid::new_v4())
    .bind(content)
    .bind(user_id)
    .bind(lead_id)
    .execute(&state.pool)
    .await?;

    Ok(())
}

#[server(GetLeadActivities, "/api")]
pub async fn get_lead_activities(lead_id: uuid::Uuid) -> Result<Vec<CrmActivity>, ServerFnError> {
    use axum::Extension;
    use leptos_axum::extract;
    let Extension(state) = extract::<Extension<crate::state::AppState>>().await?;

    let rows = sqlx::query(
        "SELECT id, activity_type, description, created_at \
         FROM activity \
         WHERE lead_id = $1 \
         ORDER BY created_at DESC"
    )
    .bind(lead_id)
    .fetch_all(&state.pool)
    .await?;

    use sqlx::Row;
    let activities = rows
        .into_iter()
        .map(|row| {
            let created_at: chrono::DateTime<chrono::Utc> = row.get("created_at");
            CrmActivity {
                id: row.get("id"),
                activity_type: row.get("activity_type"),
                description: row.try_get("description").unwrap_or_else(|_| Some("".to_string())).unwrap_or_default(),
                created_at: created_at.format("%Y-%m-%d %H:%M").to_string(),
            }
        })
        .collect();

    Ok(activities)
}

#[server(LogLeadActivity, "/api")]
pub async fn log_lead_activity(lead_id: uuid::Uuid, activity_type: String, description: String) -> Result<(), ServerFnError> {
    use axum::Extension;
    use leptos_axum::extract;
    use crate::auth::check_session;
    let Extension(state) = extract::<Extension<crate::state::AppState>>().await?;
    let Extension(tenant) = extract::<Extension<crate::state::TenantContext>>().await?;
    
    if !check_session().await.unwrap_or(false) {
        return Err(ServerFnError::ServerError("Unauthorized".into()));
    }

    let user_id: uuid::Uuid = sqlx::query_scalar("SELECT id FROM \"user\" LIMIT 1")
        .fetch_one(&state.pool)
        .await?;

    sqlx::query(
        "INSERT INTO activity (id, lead_id, activity_type, title, description, status, created_by, tenant_id, created_at, updated_at) \
         VALUES ($1, $2, $3, $4, $5, 'Completed', $6, $7, NOW(), NOW())"
    )
    .bind(uuid::Uuid::new_v4())
    .bind(lead_id)
    .bind(activity_type.clone())
    .bind(format!("Logged {}", activity_type))
    .bind(description)
    .bind(user_id)
    .bind(tenant.0)
    .execute(&state.pool)
    .await?;

    Ok(())
}

#[component]
pub fn LeadTable() -> impl IntoView {
    let refresh = expect_context::<ReadSignal<i32>>();
    let set_refresh = expect_context::<WriteSignal<i32>>();
    let leads_res = Resource::new(move || refresh.get(), |_| get_leads());
    let statuses_res = Resource::new(|| (), |_| get_lead_crm_statuses());

    let (selected_lead, set_selected_lead) = signal::<Option<LeadRecord>>(None);

    view! {
        <Transition fallback=move || view! { <div class="jetbrains text-sm text-outline">"QUERYING_DB..."</div> }>
            {move || {
                let res = leads_res.get();
                let statuses = statuses_res.get().and_then(|r| r.ok()).unwrap_or_default();
                view! {
                    <div class="relative w-full flex flex-col lg:flex-row gap-6">
                        // Table container
                        <div class="flex-1 overflow-x-auto bg-surface-container-lowest border border-outline-variant/30 rounded-xl p-6 shadow-sm">
                            <table class="w-full text-left jetbrains text-sm">
                                <thead>
                                    <tr class="text-outline border-b border-outline-variant/30 uppercase text-xs tracking-wider">
                                        <th class="py-4 px-4 font-semibold">"Name"</th>
                                        <th class="py-4 px-4 font-semibold">"Contact"</th>
                                        <th class="py-4 px-4 font-semibold">"Company / Title"</th>
                                        <th class="py-4 px-4 font-semibold">"Status"</th>
                                        <th class="py-4 px-4 font-semibold">"Source"</th>
                                        <th class="py-4 px-4 font-semibold">"Created"</th>
                                        <th class="py-4 px-4 font-semibold text-right">"Actions"</th>
                                    </tr>
                                </thead>
                                <tbody class="divide-y divide-outline-variant/20">
                                    {match ResourceState::from(res) {
                                        ResourceState::Ready(items) => {
                                            if items.is_empty() {
                                                view! {
                                                    <tr>
                                                        <td colspan="7" class="py-12 text-center text-outline-variant">
                                                            "NO_ACTIVE_LEADS"
                                                        </td>
                                                    </tr>
                                                }.into_any()
                                            } else {
                                                items.into_iter().map(|lead| {
                                                    let c = lead.clone();
                                                    let email_disp = lead.email.clone().unwrap_or_else(|| "-".to_string());
                                                    let phone_disp = lead.phone.clone().unwrap_or_else(|| "-".to_string());
                                                    let company_disp = lead.company.clone().unwrap_or_else(|| "-".to_string());
                                                    let title_disp = lead.title.clone().unwrap_or_else(|| "-".to_string());
                                                    let status_disp = lead.lead_status.clone().unwrap_or_else(|| "New".to_string());
                                                    let source_disp = lead.source.clone().unwrap_or_else(|| "Unknown".to_string());
                                                    
                                                    // Dynamic pipeline-based status badge styling
                                                    let matched_color = statuses.iter()
                                                        .find(|s| s.status_key.to_lowercase() == status_disp.to_lowercase())
                                                        .map(|s| s.color.as_str())
                                                        .unwrap_or("slate");
                                                        
                                                    let badge_classes = match matched_color {
                                                        "blue" => "bg-blue-500/10 text-blue-500 border-blue-500/20",
                                                        "purple" => "bg-purple-500/10 text-purple-500 border-purple-500/20",
                                                        "indigo" => "bg-indigo-500/10 text-indigo-500 border-indigo-500/20",
                                                        "orange" => "bg-orange-500/10 text-orange-500 border-orange-500/20",
                                                        "emerald" => "bg-emerald-500/10 text-emerald-500 border-emerald-500/20",
                                                        "rose" => "bg-rose-500/10 text-rose-500 border-rose-500/20",
                                                        _ => "bg-slate-500/10 text-slate-400 border-slate-500/20",
                                                    };

                                                    view! {
                                                        <tr 
                                                            class=move || format!(
                                                                "hover:bg-surface-container-high transition-all duration-150 cursor-pointer {}",
                                                                if selected_lead.get().map(|s| s.id) == Some(c.id) { "bg-surface-container-high border-l-4 border-primary" } else { "" }
                                                            )
                                                            on:click=move |_| set_selected_lead.set(Some(c.clone()))
                                                        >
                                                            <td class="py-4 px-4 font-bold text-primary">{lead.name}</td>
                                                            <td class="py-4 px-4">
                                                                <div class="text-xs text-outline">{email_disp}</div>
                                                                <div class="text-[10px] text-outline-variant">{phone_disp}</div>
                                                            </td>
                                                            <td class="py-4 px-4 text-xs">
                                                                <div class="font-semibold">{company_disp}</div>
                                                                <div class="text-outline-variant text-[10px]">{title_disp}</div>
                                                            </td>
                                                            <td class="py-4 px-4">
                                                                <span class=format!("px-2 py-0.5 border rounded text-[10px] font-bold {}", badge_classes)>
                                                                    {status_disp}
                                                                </span>
                                                            </td>
                                                            <td class="py-4 px-4 text-outline text-xs">{source_disp}</td>
                                                            <td class="py-4 px-4 text-outline-variant text-xs">{lead.created_at.chars().take(10).collect::<String>()}</td>
                                                            <td class="py-4 px-4 text-right">
                                                                <button 
                                                                    on:click=move |e| {
                                                                        e.stop_propagation();
                                                                        let id = lead.id;
                                                                        leptos::task::spawn_local(async move {
                                                                            if let Ok(_) = delete_lead(id).await {
                                                                                set_refresh.set(refresh.get_untracked() + 1);
                                                                                if selected_lead.get().map(|s| s.id) == Some(id) {
                                                                                    set_selected_lead.set(None);
                                                                                }
                                                                            }
                                                                        });
                                                                    } 
                                                                    class="text-error hover:underline text-xs tracking-wider uppercase font-bold"
                                                                >
                                                                    "Drop"
                                                                </button>
                                                            </td>
                                                        </tr>
                                                    }
                                                }).collect::<Vec<_>>().into_any()
                                            }
                                        }
                                        ResourceState::Loading => view! { <tr class="hidden"></tr> }.into_any(),
                                        ResourceState::Error(_) => view! { <tr><td colspan="7" class="py-8 text-center text-error">"ERR_NO_DATA"</td></tr> }.into_any(),
                                    }}
                                </tbody>
                            </table>
                        </div>

                        // Detail Overlay Modal / Split CRM Panel
                        <Show when=move || selected_lead.get().is_some()>
                            <LeadCrmPane 
                                lead_record=selected_lead.get().unwrap() 
                                stages=statuses.clone()
                                on_close=Callback::new(move |_: ()| set_selected_lead.set(None))
                            />
                        </Show>
                    </div>
                }
            }.into_any()
            }
        </Transition>
    }
}

#[component]
fn LeadCrmPane(
    lead_record: LeadRecord,
    stages: Vec<CrmStatusOption>,
    on_close: Callback<()>,
) -> impl IntoView {
    let refresh = expect_context::<ReadSignal<i32>>();
    let set_refresh = expect_context::<WriteSignal<i32>>();
    
    // Internal signals for notes, activities and stages
    let (current_stage, set_current_stage) = signal(lead_record.lead_status.clone().unwrap_or_else(|| "New".to_string()));
    
    let lead_id = lead_record.id;
    let notes_res = Resource::new(move || refresh.get(), move |_| get_lead_notes(lead_id));
    let activities_res = Resource::new(move || refresh.get(), move |_| get_lead_activities(lead_id));

    // Field signals for properties editing
    let (name, set_name) = signal(lead_record.name.clone());
    let (first_name, set_first_name) = signal(lead_record.first_name.clone().unwrap_or_default());
    let (last_name, set_last_name) = signal(lead_record.last_name.clone().unwrap_or_default());
    let (email, set_email) = signal(lead_record.email.clone().unwrap_or_default());
    let (phone, set_phone) = signal(lead_record.phone.clone().unwrap_or_default());
    let (company, set_company) = signal(lead_record.company.clone().unwrap_or_default());
    let (title, set_title) = signal(lead_record.title.clone().unwrap_or_default());
    let (source, set_source) = signal(lead_record.source.clone().unwrap_or_default());
    let (message, set_message) = signal(lead_record.message.clone().unwrap_or_default());
    
    let (edit_mode, set_edit_mode) = signal(false);

    let handle_stage_change = move |new_stage: String| {
        set_current_stage.set(new_stage.clone());
        let stage_cl = new_stage.clone();
        leptos::task::spawn_local(async move {
            if let Ok(_) = update_lead_stage(lead_id, stage_cl.clone()).await {
                // Log automatic timeline activity
                let _ = log_lead_activity(lead_id, "stage_change".to_string(), format!("Stage updated to {}", stage_cl)).await;
                set_refresh.set(refresh.get_untracked() + 1);
            }
        });
    };

    let handle_save_details = move |_| {
        let n = name.get();
        let fn_val = Some(first_name.get()).filter(|s| !s.is_empty());
        let ln_val = Some(last_name.get()).filter(|s| !s.is_empty());
        let em_val = Some(email.get()).filter(|s| !s.is_empty());
        let ph_val = Some(phone.get()).filter(|s| !s.is_empty());
        let co_val = Some(company.get()).filter(|s| !s.is_empty());
        let ti_val = Some(title.get()).filter(|s| !s.is_empty());
        let so_val = Some(source.get()).filter(|s| !s.is_empty());
        let me_val = Some(message.get()).filter(|s| !s.is_empty());

        leptos::task::spawn_local(async move {
            if let Ok(_) = update_lead_details(
                lead_id, n, fn_val, ln_val, em_val, ph_val, co_val, ti_val, so_val, me_val
            ).await {
                set_edit_mode.set(false);
                set_refresh.set(refresh.get_untracked() + 1);
            }
        });
    };

    let add_note_cb = Callback::new(move |text: String| {
        leptos::task::spawn_local(async move {
            if let Ok(_) = add_lead_note(lead_id, text).await {
                set_refresh.set(refresh.get_untracked() + 1);
            }
        });
    });

    let log_activity_cb = Callback::new(move |(act_type, desc): (String, String)| {
        leptos::task::spawn_local(async move {
            if let Ok(_) = log_lead_activity(lead_id, act_type, desc).await {
                set_refresh.set(refresh.get_untracked() + 1);
            }
        });
    });

    view! {
        <div class="w-full lg:w-[480px] shrink-0 bg-surface-container p-6 rounded-xl border border-outline-variant/30 flex flex-col max-h-[85vh] overflow-y-auto shadow-lg relative animate-slide-in">
            // Header actions
            <div class="flex items-center justify-between border-b border-outline-variant/30 pb-4 mb-6">
                <div>
                    <span class="text-[9px] font-bold tracking-widest text-outline-variant uppercase jetbrains">"LEAD_CRM_PROFILE"</span>
                    <h3 class="text-lg font-bold text-on-surface flex items-center gap-2 mt-0.5">
                        {move || name.get()}
                    </h3>
                </div>
                <div class="flex gap-2">
                    <Show when=move || !lead_record.is_converted>
                        <button
                            on:click=move |_| {
                                leptos::task::spawn_local(async move {
                                    if let Ok(_) = convert_lead(lead_id).await {
                                        let _ = log_lead_activity(lead_id, "conversion".to_string(), "Lead converted to contact successfully.".to_string()).await;
                                        set_refresh.set(refresh.get_untracked() + 1);
                                        on_close.run(());
                                    }
                                });
                            }
                            class="bg-emerald-600 text-white px-3 py-1.5 rounded jetbrains text-[10px] font-bold uppercase tracking-wider hover:bg-emerald-700 transition-colors flex items-center gap-1 shadow-sm"
                        >
                            <span class="material-symbols-outlined text-xs">"person_add"</span>
                            "Convert"
                        </button>
                    </Show>
                    <button on:click=move |_| on_close.run(()) class="p-1 hover:bg-surface-container-high rounded text-outline hover:text-on-surface">
                        <span class="material-symbols-outlined text-sm">"close"</span>
                    </button>
                </div>
            </div>

            // Chevron Pipeline Stage Bar
            <div class="mb-6">
                <label class="block text-[10px] jetbrains uppercase text-outline mb-2">"Pipeline Stage"</label>
                <CrmStageBar
                    stages=stages
                    current_stage=current_stage.into()
                    on_stage_change=handle_stage_change
                />
            </div>

            // Details Section
            <div class="space-y-4 mb-6">
                <div class="flex justify-between items-center border-b border-outline-variant/15 pb-2">
                    <span class="text-[10px] jetbrains font-bold uppercase text-outline">"Information details"</span>
                    <button
                        on:click=move |_| set_edit_mode.update(|m| *m = !*m)
                        class="text-primary hover:underline text-[10px] jetbrains font-bold uppercase tracking-wider"
                    >
                        {move || if edit_mode.get() { "Cancel" } else { "Edit Details" }}
                    </button>
                </div>

                <Show
                    when=move || edit_mode.get()
                    fallback=move || view! {
                        <div class="grid grid-cols-2 gap-4 text-xs font-mono bg-surface-container-lowest p-4 rounded-lg border border-outline-variant/10">
                            <div>
                                <span class="text-outline-variant text-[10px] block uppercase">"First Name"</span>
                                <span class="text-on-surface font-semibold">{move || if first_name.get().is_empty() { "-".to_string() } else { first_name.get() }}</span>
                            </div>
                            <div>
                                <span class="text-outline-variant text-[10px] block uppercase">"Last Name"</span>
                                <span class="text-on-surface font-semibold">{move || if last_name.get().is_empty() { "-".to_string() } else { last_name.get() }}</span>
                            </div>
                            <div>
                                <span class="text-outline-variant text-[10px] block uppercase">"Email"</span>
                                <span class="text-on-surface font-semibold break-all">{move || if email.get().is_empty() { "-".to_string() } else { email.get() }}</span>
                            </div>
                            <div>
                                <span class="text-outline-variant text-[10px] block uppercase">"Phone"</span>
                                <span class="text-on-surface font-semibold">{move || if phone.get().is_empty() { "-".to_string() } else { phone.get() }}</span>
                            </div>
                            <div>
                                <span class="text-outline-variant text-[10px] block uppercase">"Company"</span>
                                <span class="text-on-surface font-semibold">{move || if company.get().is_empty() { "-".to_string() } else { company.get() }}</span>
                            </div>
                            <div>
                                <span class="text-outline-variant text-[10px] block uppercase">"Title"</span>
                                <span class="text-on-surface font-semibold">{move || if title.get().is_empty() { "-".to_string() } else { title.get() }}</span>
                            </div>
                            <div>
                                <span class="text-outline-variant text-[10px] block uppercase">"Source"</span>
                                <span class="text-on-surface font-semibold">{move || if source.get().is_empty() { "-".to_string() } else { source.get() }}</span>
                            </div>
                            <div>
                                <span class="text-outline-variant text-[10px] block uppercase">"Created At"</span>
                                <span class="text-on-surface font-semibold">{lead_record.created_at.clone()}</span>
                            </div>
                            <div class="col-span-2 border-t border-outline-variant/10 pt-2 mt-1">
                                <span class="text-outline-variant text-[10px] block uppercase">"Original Submission Quote"</span>
                                <span class="text-on-surface leading-relaxed text-xs font-sans mt-0.5 block whitespace-pre-wrap">{move || if message.get().is_empty() { "-".to_string() } else { message.get() }}</span>
                            </div>
                        </div>
                    }
                >
                    <div class="space-y-3 bg-surface-container-lowest p-4 rounded-lg border border-outline-variant/20">
                        <div class="grid grid-cols-2 gap-3">
                            <div>
                                <label class="block text-[10px] jetbrains uppercase text-outline mb-1">"Full Name *"</label>
                                <input 
                                    type="text" 
                                    prop:value=name
                                    on:input=move |ev| set_name.set(event_target_value(&ev))
                                    class="w-full bg-surface-container border border-outline-variant/30 px-3 py-1.5 text-xs text-on-surface focus:outline-none focus:border-primary rounded"
                                />
                            </div>
                            <div>
                                <label class="block text-[10px] jetbrains uppercase text-outline mb-1">"Source"</label>
                                <input 
                                    type="text" 
                                    prop:value=source
                                    on:input=move |ev| set_source.set(event_target_value(&ev))
                                    class="w-full bg-surface-container border border-outline-variant/30 px-3 py-1.5 text-xs text-on-surface focus:outline-none focus:border-primary rounded"
                                />
                            </div>
                            <div>
                                <label class="block text-[10px] jetbrains uppercase text-outline mb-1">"First Name"</label>
                                <input 
                                    type="text" 
                                    prop:value=first_name
                                    on:input=move |ev| set_first_name.set(event_target_value(&ev))
                                    class="w-full bg-surface-container border border-outline-variant/30 px-3 py-1.5 text-xs text-on-surface focus:outline-none focus:border-primary rounded"
                                />
                            </div>
                            <div>
                                <label class="block text-[10px] jetbrains uppercase text-outline mb-1">"Last Name"</label>
                                <input 
                                    type="text" 
                                    prop:value=last_name
                                    on:input=move |ev| set_last_name.set(event_target_value(&ev))
                                    class="w-full bg-surface-container border border-outline-variant/30 px-3 py-1.5 text-xs text-on-surface focus:outline-none focus:border-primary rounded"
                                />
                            </div>
                            <div>
                                <label class="block text-[10px] jetbrains uppercase text-outline mb-1">"Email"</label>
                                <input 
                                    type="email" 
                                    prop:value=email
                                    on:input=move |ev| set_email.set(event_target_value(&ev))
                                    class="w-full bg-surface-container border border-outline-variant/30 px-3 py-1.5 text-xs text-on-surface focus:outline-none focus:border-primary rounded"
                                />
                            </div>
                            <div>
                                <label class="block text-[10px] jetbrains uppercase text-outline mb-1">"Phone"</label>
                                <input 
                                    type="text" 
                                    prop:value=phone
                                    on:input=move |ev| set_phone.set(event_target_value(&ev))
                                    class="w-full bg-surface-container border border-outline-variant/30 px-3 py-1.5 text-xs text-on-surface focus:outline-none focus:border-primary rounded"
                                />
                            </div>
                            <div>
                                <label class="block text-[10px] jetbrains uppercase text-outline mb-1">"Company"</label>
                                <input 
                                    type="text" 
                                    prop:value=company
                                    on:input=move |ev| set_company.set(event_target_value(&ev))
                                    class="w-full bg-surface-container border border-outline-variant/30 px-3 py-1.5 text-xs text-on-surface focus:outline-none focus:border-primary rounded"
                                />
                            </div>
                            <div>
                                <label class="block text-[10px] jetbrains uppercase text-outline mb-1">"Title"</label>
                                <input 
                                    type="text" 
                                    prop:value=title
                                    on:input=move |ev| set_title.set(event_target_value(&ev))
                                    class="w-full bg-surface-container border border-outline-variant/30 px-3 py-1.5 text-xs text-on-surface focus:outline-none focus:border-primary rounded"
                                />
                            </div>
                        </div>
                        <div>
                            <label class="block text-[10px] jetbrains uppercase text-outline mb-1">"Message / Quote Details"</label>
                            <textarea 
                                prop:value=message
                                on:input=move |ev| set_message.set(event_target_value(&ev))
                                rows="3"
                                class="w-full bg-surface-container border border-outline-variant/30 px-3 py-2 text-xs text-on-surface focus:outline-none focus:border-primary rounded resize-none"
                            ></textarea>
                        </div>
                        <div class="flex justify-end">
                            <button
                                on:click=handle_save_details
                                class="bg-primary text-on-primary px-4 py-2 text-xs jetbrains font-bold uppercase tracking-wider hover:bg-primary-container rounded"
                            >
                                "Save Changes"
                            </button>
                        </div>
                    </div>
                </Show>
            </div>

            // Timeline (Notes & Activities)
            <div class="border-t border-outline-variant/30 pt-6">
                <CrmTimeline
                    notes=Signal::derive(move || notes_res.get().and_then(|r| r.ok()).unwrap_or_default())
                    activities=Signal::derive(move || activities_res.get().and_then(|r| r.ok()).unwrap_or_default())
                    on_add_note=add_note_cb
                    on_log_activity=log_activity_cb
                />
            </div>
        </div>
    }
}
