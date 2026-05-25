use leptos::prelude::*;
use shared_ui::components::crm_stage_bar::{CrmStageBar, CrmStatusOption};
use shared_ui::components::crm_timeline_generic::{
    CrmTimelineGeneric, NoteModel, ActivityModel, ActivityType, ActivityStatus, FileModel
};
use shared_ui::components::properties_editor::PropertiesEditor;
use shared_ui::utils::ResourceState;
use shared_ui::components::file_attachments::{FileAttachments, RecordDocumentModel};

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
    pub avatar_url: Option<String>,
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
        "SELECT id, customer_id, name, first_name, last_name, email, phone, whatsapp, telegram, twitter, instagram, facebook, properties, avatar_url, created_at, updated_at \
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
                avatar_url: row.try_get("avatar_url").unwrap_or(None),
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
    avatar_url: Option<String>,
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
        "UPDATE contact SET name = $1, first_name = $2, last_name = $3, email = $4, phone = $5, whatsapp = $6, telegram = $7, twitter = $8, instagram = $9, facebook = $10, properties = $11, avatar_url = $12, updated_at = NOW() \
         WHERE id = $13 AND tenant_id = $14"
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
    .bind(avatar_url)
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
    attachments: Vec<String>,
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

        // 3. Construct MultiPart Body
        let mut multipart = MultiPart::mixed().singlepart(
            SinglePart::builder()
                .header(header::ContentType::TEXT_HTML)
                .body(body_html.clone()),
        );

        // 4. Download S3 attachments and append to Multipart
        if !attachments.is_empty() {
            let access_key = std::env::var("R2_ACCESS_KEY_ID").unwrap_or_default();
            let secret = std::env::var("R2_SECRET_ACCESS_KEY").unwrap_or_default();
            let endpoint = std::env::var("R2_ENDPOINT").unwrap_or_default();
            let bucket_name = "atlas-tenant-vault".to_string();

            if !access_key.is_empty() && !endpoint.is_empty() {
                let credentials = aws_sdk_s3::config::Credentials::new(
                    access_key, secret, None, None, "cloudflare"
                );
                let s3_config = aws_sdk_s3::config::Builder::new()
                    .credentials_provider(credentials)
                    .region(aws_sdk_s3::config::Region::new("auto"))
                    .endpoint_url(endpoint)
                    .build();

                let client = aws_sdk_s3::Client::from_conf(s3_config);
                for file_key in &attachments {
                    if let Ok(resp) = client.get_object().bucket(&bucket_name).key(file_key).send().await {
                        if let Ok(data) = resp.body.collect().await {
                            let bytes = data.into_bytes().to_vec();
                            let filename = file_key.split('/').last().unwrap_or("attachment").to_string();
                            let ext = filename.split('.').last().unwrap_or("").to_lowercase();
                            let mime = match ext.as_str() {
                                "pdf" => "application/pdf",
                                "png" => "image/png",
                                "jpg" | "jpeg" => "image/jpeg",
                                "gif" => "image/gif",
                                "txt" => "text/plain",
                                "html" => "text/html",
                                "doc" | "docx" => "application/msword",
                                _ => "application/octet-stream",
                            };
                            if let Ok(m_parsed) = mime.parse() {
                                let part = lettre::message::Attachment::new(filename)
                                    .body(bytes, m_parsed);
                                multipart = multipart.singlepart(part);
                            }
                        }
                    }
                }
            }
        }

        // 5. Construct Email Message
        let email = Message::builder()
            .from(from_email.parse().map_err(|e| ServerFnError::new(format!("Invalid FROM email: {}", e)))?)
            .to(to_email.parse().map_err(|e| ServerFnError::new(format!("Invalid TO email: {}", e)))?)
            .subject(&subject)
            .multipart(multipart)
            .map_err(|e| ServerFnError::new(format!("Failed to build email message: {}", e)))?;

        // 6. Send email
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

        // 7. Insert activity record
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
pub async fn get_contact_notes(contact_id: uuid::Uuid) -> Result<Vec<NoteModel>, ServerFnError> {
    use axum::Extension;
    use leptos_axum::extract;
    use crate::auth::check_session;
    let Extension(state) = extract::<Extension<crate::state::AppState>>().await?;
    let Extension(tenant) = extract::<Extension<crate::state::TenantContext>>().await?;
    
    let user_id = match check_session().await {
        Ok(true) => {
            let uid: uuid::Uuid = sqlx::query_scalar("SELECT id FROM \"user\" LIMIT 1").fetch_one(&state.pool).await?;
            uid
        }
        _ => return Err(ServerFnError::ServerError("Unauthorized".into())),
    };

    let rows = sqlx::query(
        "SELECT id, content, created_by, entity_type, entity_id, tenant_id, is_private, created_at, updated_at \
         FROM notes \
         WHERE entity_type = 'Contact' AND entity_id = $1 AND tenant_id = $2 \
           AND (is_private = false OR created_by = $3) \
         ORDER BY created_at DESC"
    )
    .bind(contact_id)
    .bind(tenant.0)
    .bind(user_id)
    .fetch_all(&state.pool)
    .await?;

    use sqlx::Row;
    let mut notes = Vec::new();
    for row in rows {
        let note_id: uuid::Uuid = row.get("id");
        
        let file_rows = sqlx::query(
            "SELECT f.id, f.name, f.size, f.mime_type, f.hash_sha256, f.storage_type, f.storage_path, f.views, f.downloads, f.bandwidth_used, f.bandwidth_used_paid, f.date_upload, f.date_last_view, f.is_anonymous, f.user_id \
             FROM file f \
             JOIN file_association fa ON f.id = fa.file_id \
             WHERE fa.associated_entity_type = 'Note' AND fa.associated_entity_id = $1"
        )
        .bind(note_id)
        .fetch_all(&state.pool)
        .await?;

        let mut files = Vec::new();
        for f_row in file_rows {
            let storage_type_str: String = f_row.get("storage_type");
            let file_id_str: String = f_row.get("id");
            let file_id = uuid::Uuid::parse_str(&file_id_str).unwrap_or_default();
            let date_upload: chrono::DateTime<chrono::Utc> = f_row.get("date_upload");
            let date_last_view: Option<chrono::DateTime<chrono::Utc>> = f_row.try_get("date_last_view").unwrap_or(None);
            let user_id_str: Option<String> = f_row.try_get("user_id").unwrap_or(None);
            let user_uuid = user_id_str.and_then(|s| uuid::Uuid::parse_str(&s).ok());

            files.push(FileModel {
                id: file_id,
                name: f_row.get("name"),
                size: f_row.get("size"),
                mime_type: f_row.get("mime_type"),
                hash_sha256: f_row.get("hash_sha256"),
                storage_type: storage_type_str,
                storage_path: f_row.get("storage_path"),
                views: f_row.get("views"),
                downloads: f_row.get("downloads"),
                bandwidth_used: f_row.get("bandwidth_used"),
                bandwidth_used_paid: f_row.get("bandwidth_used_paid"),
                date_upload: date_upload.to_rfc3339(),
                date_last_view: date_last_view.map(|dt| dt.to_rfc3339()),
                is_anonymous: f_row.get("is_anonymous"),
                user_id: user_uuid,
            });
        }

        let created_at: chrono::DateTime<chrono::Utc> = row.get("created_at");
        let updated_at: chrono::DateTime<chrono::Utc> = row.get("updated_at");

        notes.push(NoteModel {
            id: note_id,
            content: row.get("content"),
            created_by: row.get("created_by"),
            entity_type: row.get("entity_type"),
            entity_id: row.get("entity_id"),
            tenant_id: row.try_get("tenant_id").unwrap_or(None),
            is_private: row.get("is_private"),
            created_at: created_at.to_rfc3339(),
            updated_at: updated_at.to_rfc3339(),
            files,
        });
    }

    Ok(notes)
}

#[server(AddContactNote, "/api")]
pub async fn add_contact_note(
    contact_id: uuid::Uuid, 
    content: String, 
    is_private: bool, 
    files: Vec<FileModel>
) -> Result<(), ServerFnError> {
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

    let note_id = uuid::Uuid::new_v4();

    sqlx::query(
        "INSERT INTO notes (id, content, created_by, entity_type, entity_id, tenant_id, is_private, created_at, updated_at) \
         VALUES ($1, $2, $3, 'Contact', $4, $5, $6, NOW(), NOW())"
    )
    .bind(note_id)
    .bind(content)
    .bind(user_id)
    .bind(contact_id)
    .bind(tenant.0)
    .bind(is_private)
    .execute(&state.pool)
    .await?;

    for file in files {
        let file_exists: bool = sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM file WHERE id = $1)")
            .bind(file.id.to_string())
            .fetch_one(&state.pool)
            .await?;

        if !file_exists {
            sqlx::query(
                "INSERT INTO file (id, name, size, mime_type, hash_sha256, storage_type, storage_path, views, downloads, bandwidth_used, bandwidth_used_paid, date_upload, is_anonymous, user_id) \
                 VALUES ($1, $2, $3, $4, $5, $6, $7, 0, 0, 0, 0, NOW(), false, $8)"
            )
            .bind(file.id.to_string())
            .bind(file.name)
            .bind(0i64)
            .bind("application/octet-stream")
            .bind("")
            .bind("S3")
            .bind(file.storage_path)
            .bind(user_id.to_string())
            .execute(&state.pool)
            .await?;
        }

        sqlx::query(
            "INSERT INTO file_association (id, file_id, associated_entity_type, associated_entity_id, created_at) \
             VALUES ($1, $2, 'Note', $3, NOW())"
        )
        .bind(uuid::Uuid::new_v4())
        .bind(file.id.to_string())
        .bind(note_id)
        .execute(&state.pool)
        .await?;
    }

    Ok(())
}

#[server(DeleteContactNote, "/api")]
pub async fn delete_contact_note(note_id: uuid::Uuid) -> Result<(), ServerFnError> {
    use axum::Extension;
    use leptos_axum::extract;
    use crate::auth::check_session;
    let Extension(state) = extract::<Extension<crate::state::AppState>>().await?;
    let Extension(tenant) = extract::<Extension<crate::state::TenantContext>>().await?;
    
    if !check_session().await.unwrap_or(false) {
        return Err(ServerFnError::ServerError("Unauthorized".into()));
    }

    sqlx::query("DELETE FROM file_association WHERE associated_entity_type = 'Note' AND associated_entity_id = $1")
        .bind(note_id)
        .execute(&state.pool)
        .await?;

    sqlx::query("DELETE FROM notes WHERE id = $1 AND tenant_id = $2")
        .bind(note_id)
        .bind(tenant.0)
        .execute(&state.pool)
        .await?;

    Ok(())
}

#[server(GetContactActivities, "/api")]
pub async fn get_contact_activities(contact_id: uuid::Uuid) -> Result<Vec<ActivityModel>, ServerFnError> {
    use axum::Extension;
    use leptos_axum::extract;
    let Extension(state) = extract::<Extension<crate::state::AppState>>().await?;
    let Extension(tenant) = extract::<Extension<crate::state::TenantContext>>().await?;

    let rows = sqlx::query(
        "SELECT id, tenant_id, account_id, deal_id, customer_id, lead_id, contact_id, case_id, activity_type, title, description, status, due_date, completed_at, associated_entities, created_by, assigned_to, created_at, updated_at \
         FROM activity \
         WHERE contact_id = $1 AND tenant_id = $2 \
         ORDER BY created_at DESC"
    )
    .bind(contact_id)
    .bind(tenant.0)
    .fetch_all(&state.pool)
    .await?;

    use sqlx::Row;
    let mut activities = Vec::new();
    for row in rows {
        let act_id: uuid::Uuid = row.get("id");

        let file_rows = sqlx::query(
            "SELECT f.id, f.name, f.size, f.mime_type, f.hash_sha256, f.storage_type, f.storage_path, f.views, f.downloads, f.bandwidth_used, f.bandwidth_used_paid, f.date_upload, f.date_last_view, f.is_anonymous, f.user_id \
             FROM file f \
             JOIN activity_attachment aa ON f.id = aa.file_id \
             WHERE aa.activity_id = $1"
        )
        .bind(act_id)
        .fetch_all(&state.pool)
        .await?;

        let mut files = Vec::new();
        for f_row in file_rows {
            let storage_type_str: String = f_row.get("storage_type");
            let file_id_str: String = f_row.get("id");
            let file_id = uuid::Uuid::parse_str(&file_id_str).unwrap_or_default();
            let date_upload: chrono::DateTime<chrono::Utc> = f_row.get("date_upload");
            let date_last_view: Option<chrono::DateTime<chrono::Utc>> = f_row.try_get("date_last_view").unwrap_or(None);
            let user_id_str: Option<String> = f_row.try_get("user_id").unwrap_or(None);
            let user_uuid = user_id_str.and_then(|s| uuid::Uuid::parse_str(&s).ok());

            files.push(FileModel {
                id: file_id,
                name: f_row.get("name"),
                size: f_row.get("size"),
                mime_type: f_row.get("mime_type"),
                hash_sha256: f_row.get("hash_sha256"),
                storage_type: storage_type_str,
                storage_path: f_row.get("storage_path"),
                views: f_row.get("views"),
                downloads: f_row.get("downloads"),
                bandwidth_used: f_row.get("bandwidth_used"),
                bandwidth_used_paid: f_row.get("bandwidth_used_paid"),
                date_upload: date_upload.to_rfc3339(),
                date_last_view: date_last_view.map(|dt| dt.to_rfc3339()),
                is_anonymous: f_row.get("is_anonymous"),
                user_id: user_uuid,
            });
        }

        let activity_type_str: String = row.get("activity_type");
        let status_str: String = row.get("status");
        
        let activity_type = match activity_type_str.as_str() {
            "Log" => ActivityType::Log,
            "Task" => ActivityType::Task,
            "Event" => ActivityType::Event,
            _ => ActivityType::Log,
        };

        let status = match status_str.as_str() {
            "Open" => ActivityStatus::Open,
            "Pending" => ActivityStatus::Pending,
            "Completed" => ActivityStatus::Completed,
            _ => ActivityStatus::Open,
        };

        let due_date: Option<chrono::DateTime<chrono::Utc>> = row.try_get("due_date").unwrap_or(None);
        let completed_at: Option<chrono::DateTime<chrono::Utc>> = row.try_get("completed_at").unwrap_or(None);
        let created_at: chrono::DateTime<chrono::Utc> = row.get("created_at");
        let updated_at: chrono::DateTime<chrono::Utc> = row.get("updated_at");

        activities.push(ActivityModel {
            id: act_id,
            tenant_id: row.try_get("tenant_id").unwrap_or(None),
            account_id: row.try_get("account_id").unwrap_or(None),
            deal_id: row.try_get("deal_id").unwrap_or(None),
            customer_id: row.try_get("customer_id").unwrap_or(None),
            lead_id: row.try_get("lead_id").unwrap_or(None),
            contact_id: row.try_get("contact_id").unwrap_or(None),
            case_id: row.try_get("case_id").unwrap_or(None),
            activity_type,
            title: row.get("title"),
            description: row.try_get("description").unwrap_or(None),
            status,
            due_date: due_date.map(|dt| dt.to_rfc3339()),
            completed_at: completed_at.map(|dt| dt.to_rfc3339()),
            associated_entities: Vec::new(),
            created_by: row.get("created_by"),
            assigned_to: row.try_get("assigned_to").unwrap_or(None),
            created_at: created_at.to_rfc3339(),
            updated_at: updated_at.to_rfc3339(),
            files,
        });
    }

    Ok(activities)
}

#[server(AddContactActivity, "/api")]
pub async fn add_contact_activity(
    contact_id: uuid::Uuid,
    activity_type: ActivityType,
    title: String,
    description: Option<String>,
    status: ActivityStatus,
    due_date: Option<String>,
    completed_at: Option<String>,
    files: Vec<FileModel>
) -> Result<(), ServerFnError> {
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

    let activity_type_str = match activity_type {
        ActivityType::Log => "Log",
        ActivityType::Task => "Task",
        ActivityType::Event => "Event",
    };

    let status_str = match status {
        ActivityStatus::Open => "Open",
        ActivityStatus::Pending => "Pending",
        ActivityStatus::Completed => "Completed",
    };

    let parsed_due_date = due_date.and_then(|d| chrono::DateTime::parse_from_rfc3339(&d).ok()).map(|dt| dt.with_timezone(&chrono::Utc));
    let parsed_completed_at = completed_at.and_then(|d| chrono::DateTime::parse_from_rfc3339(&d).ok()).map(|dt| dt.with_timezone(&chrono::Utc));

    let act_id = uuid::Uuid::new_v4();

    sqlx::query(
        "INSERT INTO activity (id, tenant_id, contact_id, activity_type, title, description, status, due_date, completed_at, associated_entities, created_by, created_at, updated_at) \
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, '[]', $10, NOW(), NOW())"
    )
    .bind(act_id)
    .bind(tenant.0)
    .bind(contact_id)
    .bind(activity_type_str)
    .bind(title)
    .bind(description)
    .bind(status_str)
    .bind(parsed_due_date)
    .bind(parsed_completed_at)
    .bind(user_id)
    .execute(&state.pool)
    .await?;

    for file in files {
        let file_exists: bool = sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM file WHERE id = $1)")
            .bind(file.id.to_string())
            .fetch_one(&state.pool)
            .await?;

        if !file_exists {
            sqlx::query(
                "INSERT INTO file (id, name, size, mime_type, hash_sha256, storage_type, storage_path, views, downloads, bandwidth_used, bandwidth_used_paid, date_upload, is_anonymous, user_id) \
                 VALUES ($1, $2, $3, $4, $5, $6, $7, 0, 0, 0, 0, NOW(), false, $8)"
            )
            .bind(file.id.to_string())
            .bind(file.name)
            .bind(0i64)
            .bind("application/octet-stream")
            .bind("")
            .bind("S3")
            .bind(file.storage_path)
            .bind(user_id.to_string())
            .execute(&state.pool)
            .await?;
        }

        sqlx::query(
            "INSERT INTO activity_attachment (id, activity_id, file_id, created_at) \
             VALUES ($1, $2, $3, NOW())"
        )
        .bind(uuid::Uuid::new_v4())
        .bind(act_id)
        .bind(file.id.to_string())
        .execute(&state.pool)
        .await?;
    }

    Ok(())
}

#[server(UpdateContactActivityStatus, "/api")]
pub async fn update_contact_activity_status(
    activity_id: uuid::Uuid,
    status: ActivityStatus
) -> Result<(), ServerFnError> {
    use axum::Extension;
    use leptos_axum::extract;
    use crate::auth::check_session;
    let Extension(state) = extract::<Extension<crate::state::AppState>>().await?;
    let Extension(tenant) = extract::<Extension<crate::state::TenantContext>>().await?;

    if !check_session().await.unwrap_or(false) {
        return Err(ServerFnError::ServerError("Unauthorized".into()));
    }

    let status_str = match status {
        ActivityStatus::Open => "Open",
        ActivityStatus::Pending => "Pending",
        ActivityStatus::Completed => "Completed",
    };

    if status == ActivityStatus::Completed {
        sqlx::query("UPDATE activity SET status = $1, completed_at = NOW(), updated_at = NOW() WHERE id = $2 AND tenant_id = $3")
            .bind(status_str)
            .bind(activity_id)
            .bind(tenant.0)
            .execute(&state.pool)
            .await?;
    } else {
        sqlx::query("UPDATE activity SET status = $1, completed_at = NULL, updated_at = NOW() WHERE id = $2 AND tenant_id = $3")
            .bind(status_str)
            .bind(activity_id)
            .bind(tenant.0)
            .execute(&state.pool)
            .await?;
    }

    Ok(())
}

#[server(DeleteContactActivity, "/api")]
pub async fn delete_contact_activity(activity_id: uuid::Uuid) -> Result<(), ServerFnError> {
    use axum::Extension;
    use leptos_axum::extract;
    use crate::auth::check_session;
    let Extension(state) = extract::<Extension<crate::state::AppState>>().await?;
    let Extension(tenant) = extract::<Extension<crate::state::TenantContext>>().await?;

    if !check_session().await.unwrap_or(false) {
        return Err(ServerFnError::ServerError("Unauthorized".into()));
    }

    sqlx::query("DELETE FROM activity_attachment WHERE activity_id = $1")
        .bind(activity_id)
        .execute(&state.pool)
        .await?;

    sqlx::query("DELETE FROM activity WHERE id = $1 AND tenant_id = $2")
        .bind(activity_id)
        .bind(tenant.0)
        .execute(&state.pool)
        .await?;

    Ok(())
}

#[server(GetContactAttachments, "/api")]
pub async fn get_contact_attachments(contact_id: uuid::Uuid) -> Result<Vec<RecordDocumentModel>, ServerFnError> {
    use axum::Extension;
    use leptos_axum::extract;
    let Extension(state) = extract::<Extension<crate::state::AppState>>().await?;
    let Extension(tenant) = extract::<Extension<crate::state::TenantContext>>().await?;

    let rows = sqlx::query(
        "SELECT f.id as file_id, f.name, f.storage_path, f.created_at \
         FROM files f \
         INNER JOIN file_associations fa ON f.id = fa.file_id \
         WHERE fa.associated_entity_type = 'Contact' AND fa.associated_entity_id = $1 \
         ORDER BY f.created_at DESC"
    )
    .bind(contact_id)
    .fetch_all(&state.pool)
    .await?;

    use sqlx::Row;
    let docs = rows.into_iter().map(|row| {
        let created_at: chrono::DateTime<chrono::Utc> = row.get("created_at");
        let file_id_str: String = row.get("file_id");
        let id = uuid::Uuid::parse_str(&file_id_str).unwrap_or_default();
        RecordDocumentModel {
            id,
            tenant_id: tenant.0.unwrap_or_default(),
            target_record_id: contact_id,
            file_url: row.get("storage_path"),
            file_name: row.get("name"),
            uploaded_at: created_at.format("%Y-%m-%d %H:%M").to_string(),
        }
    }).collect();

    Ok(docs)
}

#[server(AddContactAttachment, "/api")]
pub async fn add_contact_attachment(contact_id: uuid::Uuid, file_name: String, file_url: String) -> Result<(), ServerFnError> {
    use axum::Extension;
    use leptos_axum::extract;
    let Extension(state) = extract::<Extension<crate::state::AppState>>().await?;
    let Extension(tenant) = extract::<Extension<crate::state::TenantContext>>().await?;

    let file_id = uuid::Uuid::new_v4();
    let file_id_str = file_id.to_string();

    sqlx::query(
        "INSERT INTO files (id, name, size, mime_type, hash_sha256, storage_type, storage_path, views, downloads, bandwidth_used, bandwidth_used_paid, created_at, updated_at, is_anonymous) \
         VALUES ($1, $2, 0, 'application/octet-stream', '', 'S', $3, 0, 0, 0, 0, NOW(), NOW(), false)"
    )
    .bind(&file_id_str)
    .bind(&file_name)
    .bind(&file_url)
    .execute(&state.pool)
    .await?;

    sqlx::query(
        "INSERT INTO file_associations (id, file_id, associated_entity_type, associated_entity_id) \
         VALUES ($1, $2, 'Contact', $3)"
    )
    .bind(uuid::Uuid::new_v4())
    .bind(&file_id_str)
    .bind(contact_id)
    .execute(&state.pool)
    .await?;

    Ok(())
}

#[server(DeleteContactAttachment, "/api")]
pub async fn delete_contact_attachment(doc_id: uuid::Uuid) -> Result<(), ServerFnError> {
    use axum::Extension;
    use leptos_axum::extract;
    let Extension(state) = extract::<Extension<crate::state::AppState>>().await?;

    sqlx::query(
        "DELETE FROM files WHERE id = $1"
    )
    .bind(doc_id.to_string())
    .execute(&state.pool)
    .await?;

    Ok(())
}

#[server(GetAttachmentDownloadUrl, "/api")]
pub async fn get_attachment_download_url(file_key: String) -> Result<String, ServerFnError> {
    let access_key = std::env::var("R2_ACCESS_KEY_ID").unwrap_or_default();
    let secret = std::env::var("R2_SECRET_ACCESS_KEY").unwrap_or_default();
    let endpoint = std::env::var("R2_ENDPOINT").unwrap_or_default();
    let bucket_name = "atlas-tenant-vault".to_string();

    if access_key.is_empty() || endpoint.is_empty() {
        return Err(ServerFnError::ServerError("R2 unconfigured".into()));
    }

    let credentials = aws_sdk_s3::config::Credentials::new(
        access_key, secret, None, None, "cloudflare"
    );
    let s3_config = aws_sdk_s3::config::Builder::new()
        .credentials_provider(credentials)
        .region(aws_sdk_s3::config::Region::new("auto"))
        .endpoint_url(endpoint)
        .build();

    let client = aws_sdk_s3::Client::from_conf(s3_config);
    let expires_in = std::time::Duration::from_secs(3600);
    let presigning_config = aws_sdk_s3::presigning::PresigningConfig::expires_in(expires_in)
        .map_err(|e| ServerFnError::<leptos::server_fn::error::NoCustomError>::ServerError(e.to_string()))?;
    
    let presigned_req = client
        .get_object()
        .bucket(&bucket_name)
        .key(&file_key)
        .presigned(presigning_config)
        .await
        .map_err(|e| ServerFnError::<leptos::server_fn::error::NoCustomError>::ServerError(e.to_string()))?;
        
    Ok(presigned_req.uri().to_string())
}

mod contacts_views;
pub use contacts_views::ContactTable;
