use leptos::prelude::*;
use shared_ui::components::crm_stage_bar::{CrmStageBar, CrmStatusOption};
use shared_ui::components::crm_timeline_generic::{
    CrmTimelineGeneric, NoteModel, ActivityModel, ActivityType, ActivityStatus, FileModel
};
use shared_ui::utils::ResourceState;
use crate::pages::admin::contacts::send_crm_email;
use shared_ui::components::file_attachments::{FileAttachments, RecordDocumentModel};

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
    pub avatar_url: Option<String>,
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
        "SELECT id, name, first_name, last_name, email, phone, company, title, lead_status, message, source, is_converted, created_at, avatar_url \
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
                avatar_url: row.try_get("avatar_url").unwrap_or(None),
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
        "UPDATE lead SET name = $1, first_name = $2, last_name = $3, email = $4, phone = $5, company = $6, title = $7, source = $8, message = $9, avatar_url = $10, updated_at = NOW() \
         WHERE id = $11 AND tenant_id = $12"
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
    .bind(avatar_url)
    .bind(id)
    .bind(tenant.0)
    .execute(&state.pool)
    .await?;

    Ok(())
}

#[server(GetLeadAttachments, "/api")]
pub async fn get_lead_attachments(lead_id: uuid::Uuid) -> Result<Vec<RecordDocumentModel>, ServerFnError> {
    use axum::Extension;
    use leptos_axum::extract;
    let Extension(state) = extract::<Extension<crate::state::AppState>>().await?;
    let Extension(tenant) = extract::<Extension<crate::state::TenantContext>>().await?;

    let rows = sqlx::query(
        "SELECT f.id as file_id, f.name, f.storage_path, f.created_at \
         FROM files f \
         INNER JOIN file_associations fa ON f.id = fa.file_id \
         WHERE fa.associated_entity_type = 'Lead' AND fa.associated_entity_id = $1 \
         ORDER BY f.created_at DESC"
    )
    .bind(lead_id)
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
            target_record_id: lead_id,
            file_url: row.get("storage_path"),
            file_name: row.get("name"),
            uploaded_at: created_at.format("%Y-%m-%d %H:%M").to_string(),
        }
    }).collect();

    Ok(docs)
}

#[server(AddLeadAttachment, "/api")]
pub async fn add_lead_attachment(lead_id: uuid::Uuid, file_name: String, file_url: String) -> Result<(), ServerFnError> {
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
         VALUES ($1, $2, 'Lead', $3)"
    )
    .bind(uuid::Uuid::new_v4())
    .bind(&file_id_str)
    .bind(lead_id)
    .execute(&state.pool)
    .await?;

    Ok(())
}

#[server(DeleteLeadAttachment, "/api")]
pub async fn delete_lead_attachment(doc_id: uuid::Uuid) -> Result<(), ServerFnError> {
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

#[server(AddLead, "/api")]
pub async fn add_lead(
    name: String,
    first_name: Option<String>,
    last_name: Option<String>,
    email: Option<String>,
    phone: Option<String>,
    company: Option<String>,
    title: Option<String>,
    lead_status: Option<String>,
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

    let lead_id = uuid::Uuid::new_v4();
    let status_str = lead_status.unwrap_or_else(|| "new".to_string());

    sqlx::query(
        "INSERT INTO lead (id, name, first_name, last_name, email, phone, company, title, lead_status, message, source, is_converted, tenant_id, created_at, updated_at) \
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, false, $12, NOW(), NOW())"
    )
    .bind(lead_id)
    .bind(name)
    .bind(first_name)
    .bind(last_name)
    .bind(validated_email)
    .bind(validated_phone)
    .bind(company)
    .bind(title)
    .bind(status_str)
    .bind(message)
    .bind(source)
    .bind(tenant.0)
    .execute(&state.pool)
    .await?;

    // 3. Log lead creation activity in the existing activity table
    #[cfg(feature = "ssr")]
    {
        let user_id: uuid::Uuid = sqlx::query_scalar("SELECT id FROM \"user\" LIMIT 1")
            .fetch_one(&state.pool)
            .await?;

        sqlx::query(
            "INSERT INTO activity (id, tenant_id, lead_id, activity_type, title, description, status, associated_entities, created_by, created_at, updated_at) \
             VALUES ($1, $2, $3, 'Other', $4, $5, 'Completed', '[]'::json, $6, NOW(), NOW())"
        )
        .bind(uuid::Uuid::new_v4())
        .bind(tenant.0)
        .bind(lead_id)
        .bind("Lead Captured".to_string())
        .bind("System: Lead entered manually via the CRM admin portal.".to_string())
        .bind(user_id)
        .execute(&state.pool)
        .await?;
    }

    Ok(())
}


#[server(GetLeadNotes, "/api")]
pub async fn get_lead_notes(lead_id: uuid::Uuid) -> Result<Vec<NoteModel>, ServerFnError> {
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
         WHERE entity_type = 'Lead' AND entity_id = $1 AND tenant_id = $2 \
           AND (is_private = false OR created_by = $3) \
         ORDER BY created_at DESC"
    )
    .bind(lead_id)
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

#[server(AddLeadNote, "/api")]
pub async fn add_lead_note(
    lead_id: uuid::Uuid, 
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
         VALUES ($1, $2, $3, 'Lead', $4, $5, $6, NOW(), NOW())"
    )
    .bind(note_id)
    .bind(content)
    .bind(user_id)
    .bind(lead_id)
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

#[server(DeleteLeadNote, "/api")]
pub async fn delete_lead_note(note_id: uuid::Uuid) -> Result<(), ServerFnError> {
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

#[server(GetLeadActivities, "/api")]
pub async fn get_lead_activities(lead_id: uuid::Uuid) -> Result<Vec<ActivityModel>, ServerFnError> {
    use axum::Extension;
    use leptos_axum::extract;
    let Extension(state) = extract::<Extension<crate::state::AppState>>().await?;
    let Extension(tenant) = extract::<Extension<crate::state::TenantContext>>().await?;

    let rows = sqlx::query(
        "SELECT id, tenant_id, account_id, deal_id, customer_id, lead_id, contact_id, case_id, activity_type, title, description, status, due_date, completed_at, associated_entities, created_by, assigned_to, created_at, updated_at \
         FROM activity \
         WHERE lead_id = $1 AND tenant_id = $2 \
         ORDER BY created_at DESC"
    )
    .bind(lead_id)
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

#[server(AddLeadActivity, "/api")]
pub async fn add_lead_activity(
    lead_id: uuid::Uuid,
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
        "INSERT INTO activity (id, tenant_id, lead_id, activity_type, title, description, status, due_date, completed_at, associated_entities, created_by, created_at, updated_at) \
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, '[]', $10, NOW(), NOW())"
    )
    .bind(act_id)
    .bind(tenant.0)
    .bind(lead_id)
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

#[server(UpdateLeadActivityStatus, "/api")]
pub async fn update_lead_activity_status(
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

#[server(DeleteLeadActivity, "/api")]
pub async fn delete_lead_activity(activity_id: uuid::Uuid) -> Result<(), ServerFnError> {
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

#[component]
pub fn LeadTable() -> impl IntoView {
    let refresh = expect_context::<ReadSignal<i32>>();
    let set_refresh = expect_context::<WriteSignal<i32>>();
    let leads_res = Resource::new(move || refresh.get(), |_| get_leads());
    let statuses_res = Resource::new(|| (), |_| get_lead_crm_statuses());

    let location = leptos_router::hooks::use_location();
    let navigate = leptos_router::hooks::use_navigate();

    // Parse lead ID from URL path: e.g., "/admin/leads/123-456"
    let id_from_url = move || {
        let path = location.pathname.get();
        let parts: Vec<&str> = path.split('/').filter(|s| !s.is_empty()).collect();
        if parts.len() >= 3 && parts[1] == "leads" {
            uuid::Uuid::parse_str(parts[2]).ok()
        } else {
            None
        }
    };

    let (selected_lead, set_selected_lead) = signal::<Option<LeadRecord>>(None);

    Effect::new(move |_| {
        if let Some(Ok(items)) = leads_res.get() {
            if let Some(target_id) = id_from_url() {
                if let Some(matched) = items.iter().find(|l| l.id == target_id) {
                    if selected_lead.get_untracked().map(|l| l.id) != Some(target_id) {
                        set_selected_lead.set(Some(matched.clone()));
                    }
                } else {
                    set_selected_lead.set(None);
                }
            } else {
                set_selected_lead.set(None);
            }
        }
    });

    view! {
        <Transition fallback=move || view! { <div class="jetbrains text-sm text-outline">"QUERYING_DB..."</div> }>
            {move || {
                let navigate = navigate.clone();
                let res = leads_res.get();
                let statuses = statuses_res.get().and_then(|r| r.ok()).unwrap_or_default();
                view! {
                    <div class="relative w-full">
                        <Show
                            when=move || selected_lead.get().is_none()
                            fallback={
                                let navigate = navigate.clone();
                                let statuses = statuses.clone();
                                move || {
                                    let navigate = navigate.clone();
                                    let statuses = statuses.clone();
                                    view! {
                                        {move || selected_lead.get().map(|lead| {
                                            let navigate = navigate.clone();
                                            view! {
                                                <LeadCrmPane 
                                                    lead_record=lead
                                                    stages=statuses.clone()
                                                    on_close=Callback::new(move |_: ()| {
                                                        let _ = navigate("/admin/leads", Default::default());
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
                                            <th class="py-4 px-4 font-semibold">"Company / Title"</th>
                                            <th class="py-4 px-4 font-semibold">"Status"</th>
                                            <th class="py-4 px-4 font-semibold">"Source"</th>
                                            <th class="py-4 px-4 font-semibold">"Created"</th>
                                            <th class="py-4 px-4 font-semibold text-right">"Actions"</th>
                                        </tr>
                                    </thead>
                                    <tbody class="divide-y divide-outline-variant/20">
                                        {match ResourceState::from(res.clone()) {
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
                                                        let navigate = navigate.clone();
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
                                                                class="hover:bg-surface-container-high transition-all duration-150 cursor-pointer"
                                                                on:click={
                                                                    let navigate = navigate.clone();
                                                                    move |_| {
                                                                        let _ = navigate(&format!("/admin/leads/{}", c.id), Default::default());
                                                                    }
                                                                }
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
                                                                        on:click={
                                                                            let navigate = navigate.clone();
                                                                            move |e| {
                                                                                e.stop_propagation();
                                                                                let id = lead.id;
                                                                                let navigate = navigate.clone();
                                                                                leptos::task::spawn_local(async move {
                                                                                    if let Ok(_) = delete_lead(id).await {
                                                                                        set_refresh.set(refresh.get_untracked() + 1);
                                                                                        if selected_lead.get().map(|s| s.id) == Some(id) {
                                                                                            let _ = navigate("/admin/leads", Default::default());
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
                                            ResourceState::Error(_) => view! { <tr><td colspan="7" class="py-8 text-center text-error">"ERR_NO_DATA"</td></tr> }.into_any(),
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
fn LeadCrmPane(
    lead_record: LeadRecord,
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
    
    // Internal signals for notes, activities and stages
    let (current_stage, set_current_stage) = signal(lead_record.lead_status.clone().unwrap_or_else(|| "New".to_string()));
    
    let lead_id = lead_record.id;
    let notes_res = Resource::new(move || refresh.get(), move |_| get_lead_notes(lead_id));
    let activities_res = Resource::new(move || refresh.get(), move |_| get_lead_activities(lead_id));
    let attachments_res = Resource::new(move || refresh.get(), move |_| get_lead_attachments(lead_id));

    // Avatar Url State
    let (avatar_url_signal, set_avatar_url_signal) = signal(lead_record.avatar_url.clone());
    let avatar_input_ref = NodeRef::<leptos::html::Input>::new();
    
    let trigger_avatar_upload = move |_| {
        if let Some(input) = avatar_input_ref.get() {
            input.click();
        }
    };

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
                let _ = add_lead_activity(
                    lead_id,
                    ActivityType::Log,
                    "Logged: Stage Change".to_string(),
                    Some(format!("Stage updated to {}", stage_cl)),
                    ActivityStatus::Completed,
                    None,
                    Some(chrono::Utc::now().to_rfc3339()),
                    Vec::new()
                ).await;
                set_refresh.set(refresh.get_untracked() + 1);
            }
        });
    };

    let (save_error, set_save_error) = signal::<Option<String>>(None);

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
        let co_val = Some(company.get()).filter(|s| !s.is_empty());
        let ti_val = Some(title.get()).filter(|s| !s.is_empty());
        let so_val = Some(source.get()).filter(|s| !s.is_empty());
        let me_val = Some(message.get()).filter(|s| !s.is_empty());
        let av_opt = avatar_url_signal.get_untracked();

        leptos::task::spawn_local(async move {
            match update_lead_details(
                lead_id, n, fn_opt, ln_opt, em_val, ph_val, co_val, ti_val, so_val, me_val, av_opt
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

    let add_note_cb = Callback::new(move |(content, is_private, files): (String, bool, Vec<FileModel>)| {
        let set_refresh = set_refresh.clone();
        let refresh = refresh.clone();
        leptos::task::spawn_local(async move {
            if let Ok(_) = add_lead_note(lead_id, content, is_private, files).await {
                set_refresh.set(refresh.get_untracked() + 1);
            }
        });
    });

    let log_activity_cb = Callback::new(move |(act_type, title, desc, status, due_date, completed_at, files): (ActivityType, String, Option<String>, ActivityStatus, Option<String>, Option<String>, Vec<FileModel>)| {
        let set_refresh = set_refresh.clone();
        let refresh = refresh.clone();
        leptos::task::spawn_local(async move {
            if let Ok(_) = add_lead_activity(lead_id, act_type, title, desc, status, due_date, completed_at, files).await {
                set_refresh.set(refresh.get_untracked() + 1);
            }
        });
    });

    let update_activity_status_cb = Callback::new(move |(act_id, status): (uuid::Uuid, ActivityStatus)| {
        let set_refresh = set_refresh.clone();
        let refresh = refresh.clone();
        leptos::task::spawn_local(async move {
            if let Ok(_) = update_lead_activity_status(act_id, status).await {
                set_refresh.set(refresh.get_untracked() + 1);
            }
        });
    });

    let delete_note_cb = Callback::new(move |note_id: uuid::Uuid| {
        let set_refresh = set_refresh.clone();
        let refresh = refresh.clone();
        leptos::task::spawn_local(async move {
            if let Ok(_) = delete_lead_note(note_id).await {
                set_refresh.set(refresh.get_untracked() + 1);
            }
        });
    });

    let delete_activity_cb = Callback::new(move |act_id: uuid::Uuid| {
        let set_refresh = set_refresh.clone();
        let refresh = refresh.clone();
        leptos::task::spawn_local(async move {
            if let Ok(_) = delete_lead_activity(act_id).await {
                set_refresh.set(refresh.get_untracked() + 1);
            }
        });
    });

    let add_attachment_cb = Callback::new(move |(file_name, file_url): (String, String)| {
        leptos::task::spawn_local(async move {
            if let Ok(_) = add_lead_attachment(lead_id, file_name, file_url).await {
                set_refresh.set(refresh.get_untracked() + 1);
            }
        });
    });

    let delete_attachment_cb = Callback::new(move |doc_id: uuid::Uuid| {
        leptos::task::spawn_local(async move {
            if let Ok(_) = delete_lead_attachment(doc_id).await {
                set_refresh.set(refresh.get_untracked() + 1);
            }
        });
    });

    let download_attachment_cb = Callback::new(move |file_key: String| {
        leptos::task::spawn_local(async move {
            if let Ok(download_url) = crate::pages::admin::contacts::get_attachment_download_url(file_key).await {
                #[cfg(not(feature = "ssr"))]
                if let Some(win) = web_sys::window() {
                    let _ = win.open_with_url_and_target(&download_url, "_blank");
                }
            }
        });
    });

    let handle_avatar_change = {
        let set_refresh = set_refresh.clone();
        let refresh = refresh.clone();
        let name = name.clone();
        let first_name = first_name.clone();
        let last_name = last_name.clone();
        let email = email.clone();
        let phone = phone.clone();
        let company = company.clone();
        let title = title.clone();
        let source = source.clone();
        let message = message.clone();
        let set_avatar_url_signal = set_avatar_url_signal.clone();
        move |ev: web_sys::Event| {
            #[cfg(not(feature = "ssr"))]
            {
                use leptos::wasm_bindgen::JsCast;
                let target = ev.target().and_then(|t| t.dyn_into::<web_sys::HtmlInputElement>().ok());
                if let Some(input) = target {
                    if let Some(files) = input.files() {
                        if let Some(file) = files.get(0) {
                            let name_val = name.get_untracked();
                            let fn_val = Some(first_name.get_untracked()).filter(|s| !s.is_empty());
                            let ln_val = Some(last_name.get_untracked()).filter(|s| !s.is_empty());
                            let em_val = Some(email.get_untracked()).filter(|s| !s.is_empty());
                            let ph_val = Some(phone.get_untracked()).filter(|s| !s.is_empty());
                            let co_val = Some(company.get_untracked()).filter(|s| !s.is_empty());
                            let ti_val = Some(title.get_untracked()).filter(|s| !s.is_empty());
                            let so_val = Some(source.get_untracked()).filter(|s| !s.is_empty());
                            let me_val = Some(message.get_untracked()).filter(|s| !s.is_empty());
                            let set_refresh = set_refresh.clone();
                            let refresh = refresh.clone();
                            let set_avatar_url_signal = set_avatar_url_signal.clone();
                            
                            leptos::task::spawn_local(async move {
                                if let Ok((_, key)) = shared_ui::components::file_attachments::upload_file_to_s3(file).await {
                                    if let Ok(_) = update_lead_details(
                                        lead_id, name_val, fn_val, ln_val, em_val, ph_val, co_val, ti_val, so_val, me_val, Some(key.clone())
                                    ).await {
                                        set_avatar_url_signal.set(Some(key));
                                        set_refresh.set(refresh.get_untracked() + 1);
                                    }
                                }
                            });
                        }
                    }
                }
            }
        }
    };

    view! {
        <div class="w-full bg-background flex flex-col animate-slide-in font-sans text-on-surface">
            // Breadcrumb navigation header
            <div class="flex items-center gap-2 mb-6 text-xs font-mono text-outline-variant">
                <button 
                    on:click=move |_| on_close.run(()) 
                    class="hover:text-primary transition-colors flex items-center gap-1 font-bold uppercase tracking-wider"
                >
                    <span class="material-symbols-outlined text-[14px]">"arrow_back"</span>
                    "Back to Leads"
                </button>
            </div>

            // Salesforce-style layout container
            <div class="flex flex-col lg:flex-row gap-6 w-full items-start">
                
                // LEFT COLUMN (65% width) - Core info and status
                <div class="w-full lg:w-[65%] space-y-6 flex flex-col">
                    
                    // Main Highlight Panel / Avatar & Quick Details
                    <div class="bg-surface-container p-6 rounded-2xl border border-outline-variant/30 shadow-xs flex flex-col md:flex-row md:items-center justify-between gap-4">
                        <div class="flex items-center gap-4">
                            <input 
                                type="file" 
                                node_ref=avatar_input_ref
                                on:change=handle_avatar_change
                                class="hidden"
                            />
                            <div 
                                on:click=trigger_avatar_upload
                                class="w-14 h-14 rounded-full bg-primary/10 text-primary flex items-center justify-center shrink-0 border border-primary/20 relative group cursor-pointer overflow-hidden"
                            >
                                <Show 
                                    when=move || avatar_url_signal.get().is_some()
                                    fallback=move || {
                                        let name_val = name.get();
                                        let initials: String = name_val.split_whitespace()
                                            .filter_map(|s| s.chars().next())
                                            .take(2)
                                            .collect::<String>()
                                            .to_uppercase();
                                        view! {
                                            <span class="font-bold text-lg">{initials}</span>
                                        }
                                    }
                                >
                                    <img 
                                        src=move || avatar_url_signal.get().unwrap_or_default()
                                        class="w-full h-full object-cover animate-fade-in"
                                    />
                                </Show>
                                <div class="absolute inset-0 bg-black/40 flex items-center justify-center opacity-0 group-hover:opacity-100 transition-opacity">
                                    <span class="material-symbols-outlined text-white text-[18px]">"photo_camera"</span>
                                </div>
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
                            <Show when=move || !lead_record.is_converted>
                                <button
                                    on:click=move |_| {
                                        leptos::task::spawn_local(async move {
                                            if let Ok(_) = convert_lead(lead_id).await {
                                                 let _ = add_lead_activity(
                                                     lead_id,
                                                     ActivityType::Log,
                                                     "Logged: Conversion".to_string(),
                                                     Some("Lead converted to contact successfully.".to_string()),
                                                     ActivityStatus::Completed,
                                                     None,
                                                     Some(chrono::Utc::now().to_rfc3339()),
                                                     Vec::new()
                                                 ).await;
                                                set_refresh.set(refresh.get_untracked() + 1);
                                                on_close.run(());
                                            }
                                        });
                                    }
                                    class="bg-emerald-600 text-white px-3 py-1.5 rounded-lg jetbrains text-[10px] font-bold uppercase tracking-wider hover:bg-emerald-700 transition-colors flex items-center gap-1 shadow-xs"
                                >
                                    <span class="material-symbols-outlined text-xs">"person_add"</span>
                                    "Convert"
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
                        <label class="block text-[10px] font-bold uppercase text-outline-variant tracking-wider font-mono mb-3">"Lead Status / Pipeline Stage"</label>
                        <CrmStageBar
                            stages=stages
                            current_stage=current_stage.into()
                            on_stage_change=handle_stage_change
                        />
                    </div>

                    // Details Section
                    <div class="bg-surface-container p-6 rounded-2xl border border-outline-variant/30 shadow-xs space-y-4">
                        <div class="flex justify-between items-center border-b border-outline-variant/15 pb-2">
                            <span class="text-[10px] jetbrains font-bold uppercase text-outline">"Lead Details"</span>
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
                                        <span class="text-outline-variant text-[10px] block uppercase">"Converted"</span>
                                        <span class="text-on-surface font-semibold">{move || if lead_record.is_converted { "Yes".to_string() } else { "No".to_string() }}</span>
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
                                    <div>
                                        <label class="block text-[10px] jetbrains uppercase text-outline mb-1">"Source"</label>
                                        <input 
                                            type="text" 
                                            prop:value=source
                                            on:input=move |ev| set_source.set(event_target_value(&ev))
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
                </div>

                // RIGHT COLUMN (35% width) - Activity Feed & Timeline
                <div class="w-full lg:w-[35%] space-y-6">
                    <div class="bg-surface-container p-6 rounded-2xl border border-outline-variant/30 shadow-xs flex flex-col">
                        <label class="block text-[10px] font-bold uppercase text-outline-variant tracking-wider font-mono mb-4">"Timeline (Notes & Activities)"</label>
                        <CrmTimelineGeneric
                            notes=Signal::derive(move || notes_res.get().and_then(|r| r.ok()).unwrap_or_default())
                            activities=Signal::derive(move || activities_res.get().and_then(|r| r.ok()).unwrap_or_default())
                            on_add_note=add_note_cb
                            on_add_activity=log_activity_cb
                            on_update_activity_status=update_activity_status_cb
                            on_delete_note=delete_note_cb
                            on_delete_activity=delete_activity_cb
                        />
                    </div>
                    <FileAttachments
                        entity_type="Lead".to_string()
                        entity_id=lead_id
                        files=Signal::derive(move || attachments_res.get().and_then(|r| r.ok()).unwrap_or_default())
                        on_upload=add_attachment_cb
                        on_delete=delete_attachment_cb
                        on_download=download_attachment_cb
                    />
                </div>

            </div>

            <shared_ui::components::email_composer::EmailComposer
                open=composer_open
                to_email=email
                templates=default_templates.clone()
                record_files=Signal::derive(move || attachments_res.get().and_then(|r| r.ok()).unwrap_or_default())
                on_close=Callback::new(move |_: ()| set_composer_open.set(false))
                on_send=Callback::new({
                    let set_refresh = set_refresh.clone();
                    let refresh = refresh.clone();
                    let to_email = email.clone();
                    move |(subj, bdy, atts): (String, String, Vec<String>)| {
                        let set_refresh = set_refresh.clone();
                        let refresh = refresh.clone();
                        let to_addr = to_email.get();
                        leptos::task::spawn_local(async move {
                            if let Ok(_) = send_crm_email(to_addr, subj, bdy, None, Some(lead_id), atts).await {
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
