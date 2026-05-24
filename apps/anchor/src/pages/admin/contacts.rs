use leptos::prelude::*;
use shared_ui::components::crm_stage_bar::{CrmStageBar, CrmStatusOption};
use shared_ui::components::crm_timeline::{CrmTimeline, CrmNote, CrmActivity};
use shared_ui::components::properties_editor::PropertiesEditor;
use shared_ui::utils::ResourceState;

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq)]
pub struct ContactRecord {
    pub id: uuid::Uuid,
    pub customer_id: Option<uuid::Uuid>,
    pub name: String,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub whatsapp: Option<String>,
    pub telegram: Option<String>,
    pub twitter: Option<String>,
    pub instagram: Option<String>,
    pub facebook: Option<String>,
    pub properties: Option<serde_json::Value>,
    pub created_at: String,
    pub updated_at: String,
}

#[server(GetContacts, "/api")]
pub async fn get_contacts() -> Result<Vec<ContactRecord>, ServerFnError> {
    use axum::Extension;
    use leptos_axum::extract;
    use crate::auth::check_session;

    if !check_session().await.unwrap_or(false) {
        return Err(ServerFnError::ServerError("Unauthorized".into()));
    }

    let Extension(state) = extract::<Extension<crate::state::AppState>>().await?;
    let Extension(tenant) = extract::<Extension<crate::state::TenantContext>>().await?;

    let rows = sqlx::query(
        "SELECT id, customer_id, name, first_name, last_name, email, phone, whatsapp, telegram, twitter, instagram, facebook, properties, created_at, updated_at \
         FROM contact \
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
            let updated_at: chrono::DateTime<chrono::Utc> = row.get("updated_at");
            ContactRecord {
                id: row.get("id"),
                customer_id: row.try_get("customer_id").unwrap_or(None),
                name: row.get("name"),
                first_name: row.try_get("first_name").unwrap_or(None),
                last_name: row.try_get("last_name").unwrap_or(None),
                email: row.try_get("email").unwrap_or(None),
                phone: row.try_get("phone").unwrap_or(None),
                whatsapp: row.try_get("whatsapp").unwrap_or(None),
                telegram: row.try_get("telegram").unwrap_or(None),
                twitter: row.try_get("twitter").unwrap_or(None),
                instagram: row.try_get("instagram").unwrap_or(None),
                facebook: row.try_get("facebook").unwrap_or(None),
                properties: row.try_get("properties").unwrap_or(None),
                created_at: created_at.format("%Y-%m-%d %H:%M:%S").to_string(),
                updated_at: updated_at.format("%Y-%m-%d %H:%M:%S").to_string(),
            }
        })
        .collect();

    Ok(items)
}

#[server(DeleteContact, "/api")]
pub async fn delete_contact(id: uuid::Uuid) -> Result<(), ServerFnError> {
    use axum::Extension;
    use leptos_axum::extract;
    use crate::auth::check_session;

    if !check_session().await.unwrap_or(false) {
        return Err(ServerFnError::ServerError("Unauthorized".into()));
    }

    let Extension(state) = extract::<Extension<crate::state::AppState>>().await?;
    let Extension(tenant) = extract::<Extension<crate::state::TenantContext>>().await?;

    sqlx::query("DELETE FROM contact WHERE id = $1 AND tenant_id = $2")
        .bind(id)
        .bind(tenant.0)
        .execute(&state.pool)
        .await?;

    Ok(())
}

#[server(GetContactCrmStatuses, "/api")]
pub async fn get_contact_crm_statuses() -> Result<Vec<CrmStatusOption>, ServerFnError> {
    use axum::Extension;
    use leptos_axum::extract;
    use sqlx::Row;
    let Extension(state) = extract::<Extension<crate::state::AppState>>().await?;
    let Extension(tenant) = extract::<Extension<crate::state::TenantContext>>().await?;

    let rows = sqlx::query(
        "SELECT status_key, label, color, sort_order, is_system \
         FROM crm_status_option \
         WHERE tenant_id = $1 AND object_type = 'Contact' \
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

#[server(UpdateContactDetails, "/api")]
pub async fn update_contact_details(
    id: uuid::Uuid,
    name: String,
    first_name: Option<String>,
    last_name: Option<String>,
    email: Option<String>,
    phone: Option<String>,
    whatsapp: Option<String>,
    telegram: Option<String>,
    twitter: Option<String>,
    instagram: Option<String>,
    facebook: Option<String>,
    properties: Option<serde_json::Value>,
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
        "UPDATE contact SET name = $1, first_name = $2, last_name = $3, email = $4, phone = $5, whatsapp = $6, telegram = $7, twitter = $8, instagram = $9, facebook = $10, properties = $11, updated_at = NOW() \
         WHERE id = $12 AND tenant_id = $13"
    )
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
    .bind(properties)
    .bind(id)
    .bind(tenant.0)
    .execute(&state.pool)
    .await?;

    Ok(())
}

#[server(AddContact, "/api")]
pub async fn add_contact(
    name: String,
    first_name: Option<String>,
    last_name: Option<String>,
    email: Option<String>,
    phone: Option<String>,
    whatsapp: Option<String>,
    telegram: Option<String>,
    twitter: Option<String>,
    instagram: Option<String>,
    facebook: Option<String>,
    properties: Option<serde_json::Value>,
) -> Result<(), ServerFnError> {
    use axum::Extension;
    use leptos_axum::extract;
    use crate::auth::check_session;

    if !check_session().await.unwrap_or(false) {
        return Err(ServerFnError::ServerError("Unauthorized".into()));
    }

    let Extension(state) = extract::<Extension<crate::state::AppState>>().await?;
    let Extension(tenant) = extract::<Extension<crate::state::TenantContext>>().await?;

    let mut validated_phone = phone.clone();
    let mut validated_email = email.clone();

    #[cfg(feature = "ssr")]
    {
        // 1. Telephone E.164 Standardization & Validation
        if let Some(ref p) = phone {
            let trimmed = p.trim();
            if !trimmed.is_empty() {
                let cleaned: String = trimmed.chars()
                    .filter(|c| c.is_ascii_digit() || *c == '+')
                    .collect();
                if cleaned.starts_with('+') && cleaned.len() >= 8 && cleaned.len() <= 16 {
                    let after_plus = &cleaned[1..];
                    if after_plus.chars().all(|c| c.is_ascii_digit()) && !after_plus.starts_with('0') {
                        validated_phone = Some(cleaned);
                    } else {
                        return Err(ServerFnError::ServerError("Invalid phone format. Please enter a valid international number in E.164 format (e.g., +15551234567).".into()));
                    }
                } else {
                    return Err(ServerFnError::ServerError("Invalid phone format. Please enter a valid international number in E.164 format (e.g., +15551234567).".into()));
                }
            }
        }

        // 2. Email Verification with DNS resolution check
        if let Some(ref e) = email {
            let trimmed = e.trim();
            if !trimmed.is_empty() {
                let parts: Vec<&str> = trimmed.split('@').collect();
                if parts.len() != 2 || parts[0].is_empty() || parts[1].is_empty() {
                    return Err(ServerFnError::ServerError("Invalid email address format (e.g. user@domain.com).".into()));
                }
                let domain = parts[1].to_lowercase();
                if !domain.contains('.') || domain.starts_with('.') || domain.ends_with('.') {
                    return Err(ServerFnError::ServerError("Invalid email address format (e.g. user@domain.com).".into()));
                }
                
                // Block test list
                let blocked = ["test.com", "example.com", "tempmail.com", "mailinator.com", "junk.com", "trashmail.com"];
                if blocked.contains(&domain.as_str()) {
                    return Err(ServerFnError::ServerError(format!("The domain '{}' is blocked or reserved for testing.", domain).into()));
                }

                // DNS resolving check
                let host_to_resolve = format!("{}:80", domain);
                match tokio::net::lookup_host(host_to_resolve.as_str()).await {
                    Ok(mut addrs) => {
                        if addrs.next().is_none() {
                            return Err(ServerFnError::ServerError(format!("The email domain '{}' does not resolve to any active hosts.", domain).into()));
                        }
                    }
                    Err(_) => {
                        return Err(ServerFnError::ServerError(format!("The email domain '{}' is offline or has no active DNS registration.", domain).into()));
                    }
                }
                validated_email = Some(trimmed.to_string());
            }
        }
    }

    let contact_id = uuid::Uuid::new_v4();

    sqlx::query(
        "INSERT INTO contact (id, name, first_name, last_name, email, phone, whatsapp, telegram, twitter, instagram, facebook, properties, tenant_id, created_at, updated_at) \
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, NOW(), NOW())"
    )
    .bind(contact_id)
    .bind(name)
    .bind(first_name)
    .bind(last_name)
    .bind(validated_email)
    .bind(validated_phone)
    .bind(whatsapp)
    .bind(telegram)
    .bind(twitter)
    .bind(instagram)
    .bind(facebook)
    .bind(properties)
    .bind(tenant.0)
    .execute(&state.pool)
    .await?;

    // 3. Log dynamic activity log into the existing activity table
    #[cfg(feature = "ssr")]
    {
        let user_id: uuid::Uuid = sqlx::query_scalar("SELECT id FROM \"user\" LIMIT 1")
            .fetch_one(&state.pool)
            .await?;

        sqlx::query(
            "INSERT INTO activity (id, tenant_id, contact_id, activity_type, title, description, status, associated_entities, created_by, created_at, updated_at) \
             VALUES ($1, $2, $3, 'Other', $4, $5, 'Completed', '[]'::json, $6, NOW(), NOW())"
        )
        .bind(uuid::Uuid::new_v4())
        .bind(tenant.0)
        .bind(contact_id)
        .bind("Contact Created".to_string())
        .bind("System: Profile created manually via the CRM admin portal.".to_string())
        .bind(user_id)
        .execute(&state.pool)
        .await?;
    }

    Ok(())
}

#[server(SendCrmEmail, "/api")]
pub async fn send_crm_email(
    to_email: String,
    subject: String,
    body_html: String,
    contact_id: Option<uuid::Uuid>,
    lead_id: Option<uuid::Uuid>,
) -> Result<(), ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        use axum::Extension;
        use leptos_axum::extract;
        use crate::auth::check_session;
        use lettre::message::{header, MultiPart, SinglePart};
        use lettre::transport::smtp::authentication::Credentials;
        use lettre::{AsyncSmtpTransport, AsyncTransport, Message, Tokio1Executor};

        if !check_session().await.unwrap_or(false) {
            return Err(ServerFnError::ServerError("Unauthorized".into()));
        }

        let Extension(state) = extract::<Extension<crate::state::AppState>>().await?;
        let Extension(tenant) = extract::<Extension<crate::state::TenantContext>>().await?;

        // 1. Fetch Tenant Settings for SMTP override (if any)
        let settings_rows = sqlx::query(
            "SELECT key, value FROM tenant_setting WHERE tenant_id = $1"
        )
        .bind(tenant.0)
        .fetch_all(&state.pool)
        .await?;

        let mut custom_host = None;
        let mut custom_port = None;
        let mut custom_username = None;
        let mut custom_token = None;
        let mut custom_from = None;

        use sqlx::Row;
        for row in settings_rows {
            let key: String = row.get("key");
            let value: String = row.get("value");
            match key.as_str() {
                "smtp_server" => custom_host = Some(value),
                "smtp_port" => custom_port = Some(value.parse().unwrap_or(587)),
                "smtp_username" => custom_username = Some(value),
                "smtp_token" => custom_token = Some(value),
                "smtp_from" => custom_from = Some(value),
                _ => {}
            }
        }

        // 2. Fallback to System environment variables if no Custom settings
        let host = custom_host.unwrap_or_else(|| std::env::var("SMTP_SERVER").unwrap_or_else(|_| "localhost".to_string()));
        let port = custom_port.unwrap_or_else(|| std::env::var("SMTP_PORT").unwrap_or_else(|_| "587".to_string()).parse().unwrap_or(587));
        let username = custom_username.unwrap_or_else(|| std::env::var("SMTP_USERNAME").unwrap_or_default());
        let token = custom_token.unwrap_or_else(|| std::env::var("SMTP_TOKEN").unwrap_or_default());
        let from_email = custom_from.unwrap_or_else(|| std::env::var("SMTP_FROM").unwrap_or_else(|_| "noreply@atlas-platform.local".to_string()));

        // 3. Construct Message
        let email = Message::builder()
            .from(from_email.parse().map_err(|e| ServerFnError::new(format!("Invalid FROM email: {}", e)))?)
            .to(to_email.parse().map_err(|e| ServerFnError::new(format!("Invalid TO email: {}", e)))?)
            .subject(&subject)
            .multipart(
                MultiPart::alternative().singlepart(
                    SinglePart::builder()
                        .header(header::ContentType::TEXT_HTML)
                        .body(body_html.clone()),
                ),
            )
            .map_err(|e| ServerFnError::new(format!("Failed to build email message: {}", e)))?;

        // 4. Send email
        if host == "localhost" || host.is_empty() {
            leptos::logging::log!("SMTP Host not configured. Mocking email send to: {}", to_email);
        } else {
            let creds = Credentials::new(username, token);
            let mailer: AsyncSmtpTransport<Tokio1Executor> = if port == 465 {
                AsyncSmtpTransport::<Tokio1Executor>::relay(&host)
                    .map_err(|e| ServerFnError::new(format!("Invalid SMTP relay host: {}", e)))?
                    .port(port)
                    .credentials(creds)
                    .build()
            } else {
                AsyncSmtpTransport::<Tokio1Executor>::starttls_relay(&host)
                    .map_err(|e| ServerFnError::new(format!("Invalid SMTP STARTTLS host: {}", e)))?
                    .port(port)
                    .credentials(creds)
                    .build()
            };

            mailer.send(email).await.map_err(|e| ServerFnError::new(format!("SMTP delivery failed: {}", e)))?;
        }

        // 5. Insert activity record
        let user_id: uuid::Uuid = sqlx::query_scalar("SELECT id FROM \"user\" LIMIT 1")
            .fetch_one(&state.pool)
            .await?;

        sqlx::query(
            "INSERT INTO activity (id, tenant_id, contact_id, lead_id, activity_type, title, description, status, created_by, created_at, updated_at) \
             VALUES ($1, $2, $3, $4, 'Email', $5, $6, 'Completed', $7, NOW(), NOW())"
        )
        .bind(uuid::Uuid::new_v4())
        .bind(tenant.0)
        .bind(contact_id)
        .bind(lead_id)
        .bind(format!("Email Sent: {}", subject))
        .bind(body_html)
        .bind(user_id)
        .execute(&state.pool)
        .await?;
    }
    Ok(())
}


#[server(GetContactNotes, "/api")]
pub async fn get_contact_notes(contact_id: uuid::Uuid) -> Result<Vec<CrmNote>, ServerFnError> {
    use axum::Extension;
    use leptos_axum::extract;
    let Extension(state) = extract::<Extension<crate::state::AppState>>().await?;

    let rows = sqlx::query(
        "SELECT id, content, created_at \
         FROM notes \
         WHERE entity_type = 'Contact' AND entity_id = $1 \
         ORDER BY created_at DESC"
    )
    .bind(contact_id)
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

#[server(AddContactNote, "/api")]
pub async fn add_contact_note(contact_id: uuid::Uuid, content: String) -> Result<(), ServerFnError> {
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
         VALUES ($1, $2, $3, 'Contact', $4, NOW(), NOW())"
    )
    .bind(uuid::Uuid::new_v4())
    .bind(content)
    .bind(user_id)
    .bind(contact_id)
    .execute(&state.pool)
    .await?;

    Ok(())
}

#[server(GetContactActivities, "/api")]
pub async fn get_contact_activities(contact_id: uuid::Uuid) -> Result<Vec<CrmActivity>, ServerFnError> {
    use axum::Extension;
    use leptos_axum::extract;
    let Extension(state) = extract::<Extension<crate::state::AppState>>().await?;

    let rows = sqlx::query(
        "SELECT id, activity_type, description, created_at \
         FROM activity \
         WHERE contact_id = $1 \
         ORDER BY created_at DESC"
    )
    .bind(contact_id)
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

#[server(LogContactActivity, "/api")]
pub async fn log_contact_activity(contact_id: uuid::Uuid, activity_type: String, description: String) -> Result<(), ServerFnError> {
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
        "INSERT INTO activity (id, contact_id, activity_type, title, description, status, created_by, tenant_id, created_at, updated_at) \
         VALUES ($1, $2, $3, $4, $5, 'Completed', $6, $7, NOW(), NOW())"
    )
    .bind(uuid::Uuid::new_v4())
    .bind(contact_id)
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
pub fn ContactTable() -> impl IntoView {
    let refresh = expect_context::<ReadSignal<i32>>();
    let set_refresh = expect_context::<WriteSignal<i32>>();
    let contacts_res = Resource::new(move || refresh.get(), |_| get_contacts());
    let statuses_res = Resource::new(|| (), |_| get_contact_crm_statuses());

    let location = leptos_router::hooks::use_location();
    let navigate = leptos_router::hooks::use_navigate();

    // Parse contact ID from URL path: e.g., "/admin/contacts/123-456"
    let id_from_url = move || {
        let path = location.pathname.get();
        let parts: Vec<&str> = path.split('/').filter(|s| !s.is_empty()).collect();
        if parts.len() >= 3 && parts[1] == "contacts" {
            uuid::Uuid::parse_str(parts[2]).ok()
        } else {
            None
        }
    };

    let (selected_contact, set_selected_contact) = signal::<Option<ContactRecord>>(None);

    Effect::new(move |_| {
        if let Some(Ok(items)) = contacts_res.get() {
            if let Some(target_id) = id_from_url() {
                if let Some(matched) = items.iter().find(|c| c.id == target_id) {
                    if selected_contact.get_untracked().map(|c| c.id) != Some(target_id) {
                        set_selected_contact.set(Some(matched.clone()));
                    }
                } else {
                    set_selected_contact.set(None);
                }
            } else {
                set_selected_contact.set(None);
            }
        }
    });

    view! {
        <Transition fallback=move || view! { <div class="jetbrains text-sm text-outline">"QUERYING_DB..."</div> }>
            {move || {
                let navigate = navigate.clone();
                let res = contacts_res.get();
                let statuses = statuses_res.get().and_then(|r| r.ok()).unwrap_or_default();
                view! {
                    <div class="relative w-full">
                        <Show
                            when=move || selected_contact.get().is_none()
                            fallback={
                                let navigate = navigate.clone();
                                let statuses = statuses.clone();
                                move || {
                                    let navigate = navigate.clone();
                                    let statuses = statuses.clone();
                                    view! {
                                        {move || selected_contact.get().map(|contact| {
                                            let navigate = navigate.clone();
                                            view! {
                                                <ContactCrmPane 
                                                    contact_record=contact
                                                    stages=statuses.clone()
                                                    on_close=Callback::new(move |_: ()| {
                                                        let _ = navigate("/admin/contacts", Default::default());
                                                    })
                                                />
                                            }
                                        })}
                                    }
                                }
                            }
                        >
                            // Table container
                            <div class="overflow-x-auto bg-surface-container-lowest border border-outline-variant/30 rounded-xl p-6 shadow-sm">
                                <table class="w-full text-left jetbrains text-sm">
                                    <thead>
                                        <tr class="text-outline border-b border-outline-variant/30 uppercase text-xs tracking-wider">
                                            <th class="py-4 px-4 font-semibold">"Name"</th>
                                            <th class="py-4 px-4 font-semibold">"Contact"</th>
                                            <th class="py-4 px-4 font-semibold">"Social channels"</th>
                                            <th class="py-4 px-4 font-semibold">"Status"</th>
                                            <th class="py-4 px-4 font-semibold text-right">"Actions"</th>
                                        </tr>
                                    </thead>
                                    <tbody class="divide-y divide-outline-variant/20">
                                        {match ResourceState::from(res.clone()) {
                                            ResourceState::Ready(items) => {
                                                if items.is_empty() {
                                                    view! {
                                                        <tr>
                                                            <td colspan="5" class="py-12 text-center text-outline-variant">
                                                                "NO_ACTIVE_CONTACTS"
                                                            </td>
                                                        </tr>
                                                    }.into_any()
                                                } else {
                                                    items.into_iter().map(|contact| {
                                                        let c = contact.clone();
                                                        let navigate = navigate.clone();
                                                        let email_disp = contact.email.clone().unwrap_or_else(|| "-".to_string());
                                                        let phone_disp = contact.phone.clone().unwrap_or_else(|| "-".to_string());
                                                        
                                                        // Extract status from properties JSON
                                                        let status_disp = contact.properties.as_ref()
                                                            .and_then(|p| p.get("status"))
                                                            .and_then(|s| s.as_str())
                                                            .unwrap_or("prospect")
                                                            .to_string();
                                                        
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
                                                                class="hover:bg-surface-container-high transition-all duration-150 cursor-pointer"
                                                                on:click={
                                                                    let navigate = navigate.clone();
                                                                    move |_| {
                                                                        let _ = navigate(&format!("/admin/contacts/{}", c.id), Default::default());
                                                                    }
                                                                }
                                                            >
                                                                <td class="py-4 px-4 font-bold text-primary">{contact.name}</td>
                                                                <td class="py-4 px-4">
                                                                    <div class="text-xs text-outline">{email_disp}</div>
                                                                    <div class="text-[10px] text-outline-variant">{phone_disp}</div>
                                                                </td>
                                                                <td class="py-4 px-4 text-xs font-mono text-outline-variant">
                                                                    <div class="flex gap-2">
                                                                        <Show when=move || contact.twitter.is_some()>
                                                                            <span class="bg-surface-container px-1.5 py-0.5 rounded text-[10px]">"X"</span>
                                                                        </Show>
                                                                        <Show when=move || contact.whatsapp.is_some()>
                                                                            <span class="bg-emerald-500/10 text-emerald-600 px-1.5 py-0.5 rounded text-[10px]">"WA"</span>
                                                                        </Show>
                                                                        <Show when=move || contact.telegram.is_some()>
                                                                            <span class="bg-blue-500/10 text-blue-600 px-1.5 py-0.5 rounded text-[10px]">"TG"</span>
                                                                        </Show>
                                                                    </div>
                                                                </td>
                                                                <td class="py-4 px-4">
                                                                    <span class=format!("px-2 py-0.5 border rounded text-[10px] font-bold uppercase {}", badge_classes)>
                                                                        {status_disp}
                                                                    </span>
                                                                </td>
                                                                <td class="py-4 px-4 text-right">
                                                                    <button 
                                                                        on:click={
                                                                            let navigate = navigate.clone();
                                                                            move |e| {
                                                                                e.stop_propagation();
                                                                                let id = contact.id;
                                                                                let navigate = navigate.clone();
                                                                                leptos::task::spawn_local(async move {
                                                                                    if let Ok(_) = delete_contact(id).await {
                                                                                        set_refresh.set(refresh.get_untracked() + 1);
                                                                                        if selected_contact.get().map(|s| s.id) == Some(id) {
                                                                                            let _ = navigate("/admin/contacts", Default::default());
                                                                                        }
                                                                                    }
                                                                                });
                                                                            }
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
                                            ResourceState::Error(_) => view! { <tr><td colspan="5" class="py-8 text-center text-error">"ERR_NO_DATA"</td></tr> }.into_any(),
                                        }}
                                    </tbody>
                                </table>
                            </div>
                        </Show>
                    </div>
                }
            }.into_any()
            }
        </Transition>
    }
}

#[component]
fn ContactCrmPane(
    contact_record: ContactRecord,
    stages: Vec<CrmStatusOption>,
    on_close: Callback<()>,
) -> impl IntoView {
    let refresh = expect_context::<ReadSignal<i32>>();
    let set_refresh = expect_context::<WriteSignal<i32>>();

    let (composer_open, set_composer_open) = signal(false);

    let default_templates = vec![
        shared_ui::components::email_composer::EmailTemplate {
            name: "Intake Follow-Up".to_string(),
            subject: "Following up on your intake inquiry".to_string(),
            body: "<p>Hello,</p><p>Thank you for reaching out. We received your details and are currently reviewing your inquiry. We will get back to you shortly with next steps.</p><p>Best regards,<br/>The Operations Team</p>".to_string(),
        },
        shared_ui::components::email_composer::EmailTemplate {
            name: "Proposal Presentation".to_string(),
            subject: "Custom Proposal Presentation".to_string(),
            body: "<p>Hello,</p><p>We are excited to share our custom proposal based on our initial discussion. Please review the attached details and let us know if you have any questions or when you would be available for a quick walkthrough.</p><p>Best regards,<br/>The Consulting Team</p>".to_string(),
        },
    ];
    
    // Extract status from properties JSON
    let status_val = contact_record.properties.as_ref()
        .and_then(|p| p.get("status"))
        .and_then(|s| s.as_str())
        .unwrap_or("prospect")
        .to_string();

    let (current_stage, set_current_stage) = signal(status_val);
    
    let contact_id = contact_record.id;
    let notes_res = Resource::new(move || refresh.get(), move |_| get_contact_notes(contact_id));
    let activities_res = Resource::new(move || refresh.get(), move |_| get_contact_activities(contact_id));

    // Field signals for standard details editing
    let (name, set_name) = signal(contact_record.name.clone());
    let (first_name, set_first_name) = signal(contact_record.first_name.clone().unwrap_or_default());
    let (last_name, set_last_name) = signal(contact_record.last_name.clone().unwrap_or_default());
    let (email, set_email) = signal(contact_record.email.clone().unwrap_or_default());
    let (phone, set_phone) = signal(contact_record.phone.clone().unwrap_or_default());
    let (whatsapp, set_whatsapp) = signal(contact_record.whatsapp.clone().unwrap_or_default());
    let (telegram, set_telegram) = signal(contact_record.telegram.clone().unwrap_or_default());
    let (twitter, set_twitter) = signal(contact_record.twitter.clone().unwrap_or_default());
    let (instagram, set_instagram) = signal(contact_record.instagram.clone().unwrap_or_default());
    let (facebook, set_facebook) = signal(contact_record.facebook.clone().unwrap_or_default());
    
    // Properties JSON RwSignal for PropertiesEditor
    let properties_signal = RwSignal::new(contact_record.properties.clone());
    
    let (edit_mode, set_edit_mode) = signal(false);
    let (save_error, set_save_error) = signal::<Option<String>>(None);

    let handle_stage_change = move |new_stage: String| {
        set_current_stage.set(new_stage.clone());
        let stage_cl = new_stage.clone();
        
        // Update status in properties JSON payload
        let mut props = properties_signal.get_untracked().unwrap_or(serde_json::Value::Object(serde_json::Map::new()));
        if let serde_json::Value::Object(ref mut map) = props {
            map.insert("status".to_string(), serde_json::Value::String(stage_cl.clone()));
        }
        properties_signal.set(Some(props.clone()));
        
        let n = name.get();
        let fn_val = Some(first_name.get()).filter(|s| !s.is_empty());
        let ln_val = Some(last_name.get()).filter(|s| !s.is_empty());
        let em_val = Some(email.get()).filter(|s| !s.is_empty());
        let ph_val = Some(phone.get()).filter(|s| !s.is_empty());
        let wa_val = Some(whatsapp.get()).filter(|s| !s.is_empty());
        let tg_val = Some(telegram.get()).filter(|s| !s.is_empty());
        let tw_val = Some(twitter.get()).filter(|s| !s.is_empty());
        let ig_val = Some(instagram.get()).filter(|s| !s.is_empty());
        let fb_val = Some(facebook.get()).filter(|s| !s.is_empty());

        leptos::task::spawn_local(async move {
            if let Ok(_) = update_contact_details(
                contact_id, n, fn_val, ln_val, em_val, ph_val, wa_val, tg_val, tw_val, ig_val, fb_val, Some(props)
            ).await {
                // Log timeline activity
                let _ = log_contact_activity(contact_id, "stage_change".to_string(), format!("Status transitioned to {}", stage_cl)).await;
                set_refresh.set(refresh.get_untracked() + 1);
            }
        });
    };

    let handle_save_details = move |_| {
        let fn_val = first_name.get();
        let ln_val = last_name.get();
        if fn_val.is_empty() && ln_val.is_empty() {
            set_save_error.set(Some("First Name or Last Name is required".to_string()));
            return;
        }
        let n = format!("{} {}", fn_val, ln_val).trim().to_string();
        set_name.set(n.clone());
        set_save_error.set(None);

        let fn_opt = Some(fn_val).filter(|s| !s.is_empty());
        let ln_opt = Some(ln_val).filter(|s| !s.is_empty());
        let em_val = Some(email.get()).filter(|s| !s.is_empty());
        let ph_val = Some(phone.get()).filter(|s| !s.is_empty());
        let wa_val = Some(whatsapp.get()).filter(|s| !s.is_empty());
        let tg_val = Some(telegram.get()).filter(|s| !s.is_empty());
        let tw_val = Some(twitter.get()).filter(|s| !s.is_empty());
        let ig_val = Some(instagram.get()).filter(|s| !s.is_empty());
        let fb_val = Some(facebook.get()).filter(|s| !s.is_empty());
        
        // Include status in the saved properties JSON
        let mut props = properties_signal.get().unwrap_or(serde_json::Value::Object(serde_json::Map::new()));
        if let serde_json::Value::Object(ref mut map) = props {
            map.insert("status".to_string(), serde_json::Value::String(current_stage.get_untracked()));
        }

        leptos::task::spawn_local(async move {
            match update_contact_details(
                contact_id, n, fn_opt, ln_opt, em_val, ph_val, wa_val, tg_val, tw_val, ig_val, fb_val, Some(props)
            ).await {
                Ok(_) => {
                    set_edit_mode.set(false);
                    set_refresh.set(refresh.get_untracked() + 1);
                }
                Err(e) => {
                    set_save_error.set(Some(format!("Save failed: {}", e)));
                }
            }
        });
    };

    let add_note_cb = Callback::new(move |text: String| {
        leptos::task::spawn_local(async move {
            if let Ok(_) = add_contact_note(contact_id, text).await {
                set_refresh.set(refresh.get_untracked() + 1);
            }
        });
    });

    let log_activity_cb = Callback::new(move |(act_type, desc): (String, String)| {
        leptos::task::spawn_local(async move {
            if let Ok(_) = log_contact_activity(contact_id, act_type, desc).await {
                set_refresh.set(refresh.get_untracked() + 1);
            }
        });
    });

    view! {
        <div class="w-full bg-background flex flex-col animate-slide-in font-sans text-on-surface">
            // Breadcrumb navigation header
            <div class="flex items-center gap-2 mb-6 text-xs font-mono text-outline-variant">
                <button 
                    on:click=move |_| on_close.run(()) 
                    class="hover:text-primary transition-colors flex items-center gap-1 font-bold uppercase tracking-wider"
                >
                    <span class="material-symbols-outlined text-[14px]">"arrow_back"</span>
                    "Back to Contacts"
                </button>
            </div>

            // Salesforce-style layout container
            <div class="flex flex-col lg:flex-row gap-6 w-full items-start">
                
                // LEFT COLUMN (65% width) - Core info and status
                <div class="w-full lg:w-[65%] space-y-6 flex flex-col">
                    
                    // Main Highlight Panel / Avatar & Quick Details
                    <div class="bg-surface-container p-6 rounded-2xl border border-outline-variant/30 shadow-xs flex flex-col md:flex-row md:items-center justify-between gap-4">
                        <div class="flex items-center gap-4">
                            <div class="w-14 h-14 rounded-full bg-primary/10 text-primary flex items-center justify-center shrink-0 border border-primary/20">
                                <span class="material-symbols-outlined text-[28px]">"person"</span>
                            </div>
                            <div>
                                <h2 class="text-xl font-bold text-on-surface leading-tight">{move || name.get()}</h2>
                                <div class="flex flex-wrap items-center gap-x-4 gap-y-1 text-xs text-outline mt-1 font-mono">
                                    <div class="flex items-center gap-1">
                                        <span class="material-symbols-outlined text-[14px]">"mail"</span>
                                        <span>{move || if email.get().is_empty() { "-".to_string() } else { email.get() }}</span>
                                    </div>
                                    <div class="flex items-center gap-1">
                                        <span class="material-symbols-outlined text-[14px]">"call"</span>
                                        <span>{move || if phone.get().is_empty() { "-".to_string() } else { phone.get() }}</span>
                                    </div>
                                </div>
                            </div>
                        </div>

                        // Quick Actions Row
                        <div class="flex items-center gap-2 self-end md:self-auto">
                            <Show when=move || !email.get().is_empty()>
                                <button
                                    on:click=move |_| set_composer_open.set(true)
                                    class="bg-primary text-on-primary px-3 py-1.5 rounded-lg jetbrains text-[10px] font-bold uppercase tracking-wider hover:bg-primary-container transition-colors flex items-center gap-1 shadow-xs"
                                >
                                    <span class="material-symbols-outlined text-xs">"mail"</span>
                                    "Send Email"
                                </button>
                            </Show>
                            <button
                                on:click=move |_| set_edit_mode.update(|m| *m = !*m)
                                class="bg-surface-container-high border border-outline-variant/40 px-3 py-1.5 rounded-lg jetbrains text-[10px] font-bold uppercase tracking-wider text-on-surface hover:bg-surface-container-lowest transition-colors flex items-center gap-1 shadow-xs"
                            >
                                <span class="material-symbols-outlined text-xs">"edit"</span>
                                {move || if edit_mode.get() { "Cancel" } else { "Edit Details" }}
                            </button>
                        </div>
                    </div>

                    // Chevron Pipeline Stage Bar Card
                    <div class="bg-surface-container p-6 rounded-2xl border border-outline-variant/30 shadow-xs">
                        <label class="block text-[10px] font-bold uppercase text-outline-variant tracking-wider font-mono mb-3">"Relationship Status / Pipeline Stage"</label>
                        <CrmStageBar
                            stages=stages
                            current_stage=current_stage.into()
                            on_stage_change=handle_stage_change
                        />
                    </div>

                    // Details Section
                    <div class="bg-surface-container p-6 rounded-2xl border border-outline-variant/30 shadow-xs space-y-4">
                        <div class="flex justify-between items-center border-b border-outline-variant/15 pb-2">
                            <span class="text-[10px] jetbrains font-bold uppercase text-outline">"Information Details"</span>
                        </div>

                        <Show
                            when=move || edit_mode.get()
                            fallback=move || view! {
                                <div class="grid grid-cols-1 md:grid-cols-2 gap-4 text-xs font-mono bg-surface-container-lowest p-4 rounded-xl border border-outline-variant/10">
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
                                        <div class="flex items-center gap-2">
                                            <span class="text-on-surface font-semibold break-all">{move || if email.get().is_empty() { "-".to_string() } else { email.get() }}</span>
                                            <Show when=move || !email.get().is_empty()>
                                                <button 
                                                    on:click=move |_| set_composer_open.set(true)
                                                    class="text-primary hover:text-primary-container p-0.5 rounded transition-colors flex items-center justify-center"
                                                    title="Compose Email"
                                                >
                                                    <span class="material-symbols-outlined text-[14px]">"mail"</span>
                                                </button>
                                            </Show>
                                        </div>
                                    </div>
                                    <div>
                                        <span class="text-outline-variant text-[10px] block uppercase">"Phone"</span>
                                        <span class="text-on-surface font-semibold">{move || if phone.get().is_empty() { "-".to_string() } else { phone.get() }}</span>
                                    </div>
                                    <div>
                                        <span class="text-outline-variant text-[10px] block uppercase">"WhatsApp"</span>
                                        <span class="text-on-surface font-semibold">{move || if whatsapp.get().is_empty() { "-".to_string() } else { whatsapp.get() }}</span>
                                    </div>
                                    <div>
                                        <span class="text-outline-variant text-[10px] block uppercase">"Telegram"</span>
                                        <span class="text-on-surface font-semibold">{move || if telegram.get().is_empty() { "-".to_string() } else { telegram.get() }}</span>
                                    </div>
                                    <div>
                                        <span class="text-outline-variant text-[10px] block uppercase">"Twitter / X"</span>
                                        <span class="text-on-surface font-semibold">{move || if twitter.get().is_empty() { "-".to_string() } else { twitter.get() }}</span>
                                    </div>
                                    <div>
                                        <span class="text-outline-variant text-[10px] block uppercase">"Instagram"</span>
                                        <span class="text-on-surface font-semibold">{move || if instagram.get().is_empty() { "-".to_string() } else { instagram.get() }}</span>
                                    </div>
                                </div>
                            }
                        >
                            <div class="space-y-3 bg-surface-container-lowest p-4 rounded-xl border border-outline-variant/20">
                                <div class="grid grid-cols-1 md:grid-cols-2 gap-3">
                                    <div>
                                        <label class="block text-[10px] jetbrains uppercase text-outline mb-1">"First Name *"</label>
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
                                        <label class="block text-[10px] jetbrains uppercase text-outline mb-1">"WhatsApp"</label>
                                        <input 
                                            type="text" 
                                            prop:value=whatsapp
                                            on:input=move |ev| set_whatsapp.set(event_target_value(&ev))
                                            class="w-full bg-surface-container border border-outline-variant/30 px-3 py-1.5 text-xs text-on-surface focus:outline-none focus:border-primary rounded"
                                        />
                                    </div>
                                    <div>
                                        <label class="block text-[10px] jetbrains uppercase text-outline mb-1">"Telegram"</label>
                                        <input 
                                            type="text" 
                                            prop:value=telegram
                                            on:input=move |ev| set_telegram.set(event_target_value(&ev))
                                            class="w-full bg-surface-container border border-outline-variant/30 px-3 py-1.5 text-xs text-on-surface focus:outline-none focus:border-primary rounded"
                                        />
                                    </div>
                                    <div>
                                        <label class="block text-[10px] jetbrains uppercase text-outline mb-1">"Twitter / X"</label>
                                        <input 
                                            type="text" 
                                            prop:value=twitter
                                            on:input=move |ev| set_twitter.set(event_target_value(&ev))
                                            class="w-full bg-surface-container border border-outline-variant/30 px-3 py-1.5 text-xs text-on-surface focus:outline-none focus:border-primary rounded"
                                        />
                                    </div>
                                    <div>
                                        <label class="block text-[10px] jetbrains uppercase text-outline mb-1">"Instagram"</label>
                                        <input 
                                            type="text" 
                                            prop:value=instagram
                                            on:input=move |ev| set_instagram.set(event_target_value(&ev))
                                            class="w-full bg-surface-container border border-outline-variant/30 px-3 py-1.5 text-xs text-on-surface focus:outline-none focus:border-primary rounded"
                                        />
                                    </div>
                                </div>
                                <Show when=move || save_error.get().is_some()>
                                    <div class="bg-error/10 border-l-4 border-error p-3 jetbrains text-xs text-error font-medium">
                                        {move || save_error.get().unwrap_or_default()}
                                    </div>
                                </Show>
                                <div class="flex justify-end">
                                    <button
                                        on:click=handle_save_details
                                        class="bg-primary text-on-primary px-4 py-2 text-xs jetbrains font-bold uppercase tracking-wider hover:bg-primary-container rounded-lg"
                                    >
                                        "Save Changes"
                                    </button>
                                </div>
                            </div>
                        </Show>
                    </div>

                    // Reusable Headless Custom Properties Editor (JSON-based metadata)
                    <div class="bg-surface-container p-6 rounded-2xl border border-outline-variant/30 shadow-xs">
                        <label class="block text-[10px] font-bold uppercase text-outline-variant tracking-wider font-mono mb-3">"Custom Properties (JSON Metadata)"</label>
                        <PropertiesEditor
                            properties=properties_signal
                        />
                    </div>
                </div>

                // RIGHT COLUMN (35% width) - Activity Feed & Timeline
                <div class="w-full lg:w-[35%] space-y-6">
                    <div class="bg-surface-container p-6 rounded-2xl border border-outline-variant/30 shadow-xs flex flex-col">
                        <label class="block text-[10px] font-bold uppercase text-outline-variant tracking-wider font-mono mb-4">"Timeline (Notes & Activities)"</label>
                        <CrmTimeline
                            notes=Signal::derive(move || notes_res.get().and_then(|r| r.ok()).unwrap_or_default())
                            activities=Signal::derive(move || activities_res.get().and_then(|r| r.ok()).unwrap_or_default())
                            on_add_note=add_note_cb
                            on_log_activity=log_activity_cb
                        />
                    </div>
                </div>

            </div>

            <shared_ui::components::email_composer::EmailComposer
                open=composer_open
                to_email=email
                templates=default_templates.clone()
                on_close=Callback::new(move |_: ()| set_composer_open.set(false))
                on_send=Callback::new({
                    let set_refresh = set_refresh.clone();
                    let refresh = refresh.clone();
                    let to_email = email.clone();
                    move |(subj, bdy): (String, String)| {
                        let set_refresh = set_refresh.clone();
                        let refresh = refresh.clone();
                        let to_addr = to_email.get();
                        leptos::task::spawn_local(async move {
                            if let Ok(_) = send_crm_email(to_addr, subj, bdy, Some(contact_id), None).await {
                                set_composer_open.set(false);
                                set_refresh.set(refresh.get_untracked() + 1);
                            }
                        });
                    }
                })
            />
        </div>
    }
}
