use crate::pages::admin::contacts::send_crm_email;
use leptos::prelude::*;
use shared_ui::components::crm_stage_bar::{CrmStageBar, CrmStatusOption};
use shared_ui::components::crm_timeline_generic::{
    ActivityModel, ActivityStatus, ActivityType, CrmTimelineGeneric, FileModel, NoteModel,
};
use shared_ui::components::file_attachments::{FileAttachments, RecordDocumentModel};
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
    pub avatar_url: Option<String>,
}

#[server(GetLeads, "/api")]
pub async fn get_leads() -> Result<Vec<LeadRecord>, ServerFnError> {
    use crate::auth::check_session;
    use axum::Extension;
    use leptos_axum::extract;

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
    use crate::auth::check_session;
    use axum::Extension;
    use leptos_axum::extract;

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
    use crate::auth::check_session;
    use axum::Extension;
    use leptos_axum::extract;

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
    let shipping_address: Option<serde_json::Value> =
        row.try_get("shipping_address").unwrap_or(None);

    // Start SQLx transaction
    let mut tx = state.pool.begin().await?;

    // 2. Check for duplicate Contact by email or phone
    let mut duplicate_contact_id: Option<uuid::Uuid> = None;
    if let Some(ref email_str) = email {
        if !email_str.is_empty() {
            duplicate_contact_id = sqlx::query_scalar(
                "SELECT id FROM contact WHERE tenant_id = $1 AND email = $2 LIMIT 1",
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
                    "SELECT id FROM contact WHERE tenant_id = $1 AND phone = $2 LIMIT 1",
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
         ORDER BY sort_order ASC",
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
    use crate::auth::check_session;
    use axum::Extension;
    use leptos_axum::extract;

    if !check_session().await.unwrap_or(false) {
        return Err(ServerFnError::ServerError("Unauthorized".into()));
    }

    let Extension(state) = extract::<Extension<crate::state::AppState>>().await?;
    let Extension(tenant) = extract::<Extension<crate::state::TenantContext>>().await?;

    sqlx::query(
        "UPDATE lead SET lead_status = $1, updated_at = NOW() \
         WHERE id = $2 AND tenant_id = $3",
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
    use crate::auth::check_session;
    use axum::Extension;
    use leptos_axum::extract;

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
pub async fn get_lead_attachments(
    lead_id: uuid::Uuid,
) -> Result<Vec<RecordDocumentModel>, ServerFnError> {
    use axum::Extension;
    use leptos_axum::extract;
    let Extension(state) = extract::<Extension<crate::state::AppState>>().await?;
    let Extension(tenant) = extract::<Extension<crate::state::TenantContext>>().await?;

    let rows = sqlx::query(
        "SELECT f.id as file_id, f.name, f.storage_path, f.created_at \
         FROM files f \
         INNER JOIN file_associations fa ON f.id = fa.file_id \
         WHERE fa.associated_entity_type = 'Lead' AND fa.associated_entity_id = $1 \
         ORDER BY f.created_at DESC",
    )
    .bind(lead_id)
    .fetch_all(&state.pool)
    .await?;

    use sqlx::Row;
    let docs = rows
        .into_iter()
        .map(|row| {
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
        })
        .collect();

    Ok(docs)
}

#[server(AddLeadAttachment, "/api")]
pub async fn add_lead_attachment(
    lead_id: uuid::Uuid,
    file_name: String,
    file_url: String,
) -> Result<(), ServerFnError> {
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

    sqlx::query("DELETE FROM files WHERE id = $1")
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
    use crate::auth::check_session;
    use axum::Extension;
    use leptos_axum::extract;

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
                let cleaned: String = trimmed
                    .chars()
                    .filter(|c| c.is_ascii_digit() || *c == '+')
                    .collect();
                if cleaned.starts_with('+') && cleaned.len() >= 8 && cleaned.len() <= 16 {
                    let after_plus = &cleaned[1..];
                    if after_plus.chars().all(|c| c.is_ascii_digit())
                        && !after_plus.starts_with('0')
                    {
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
                    return Err(ServerFnError::ServerError(
                        "Invalid email address format (e.g. user@domain.com).".into(),
                    ));
                }
                let domain = parts[1].to_lowercase();
                if !domain.contains('.') || domain.starts_with('.') || domain.ends_with('.') {
                    return Err(ServerFnError::ServerError(
                        "Invalid email address format (e.g. user@domain.com).".into(),
                    ));
                }

                // Block test list
                let blocked = [
                    "test.com",
                    "example.com",
                    "tempmail.com",
                    "mailinator.com",
                    "junk.com",
                    "trashmail.com",
                ];
                if blocked.contains(&domain.as_str()) {
                    return Err(ServerFnError::ServerError(
                        format!(
                            "The domain '{}' is blocked or reserved for testing.",
                            domain
                        )
                        .into(),
                    ));
                }

                // DNS resolving check
                let host_to_resolve = format!("{}:80", domain);
                match tokio::net::lookup_host(host_to_resolve.as_str()).await {
                    Ok(mut addrs) => {
                        if addrs.next().is_none() {
                            return Err(ServerFnError::ServerError(
                                format!(
                                    "The email domain '{}' does not resolve to any active hosts.",
                                    domain
                                )
                                .into(),
                            ));
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
    use crate::auth::check_session;
    use axum::Extension;
    use leptos_axum::extract;
    let Extension(state) = extract::<Extension<crate::state::AppState>>().await?;
    let Extension(tenant) = extract::<Extension<crate::state::TenantContext>>().await?;

    let user_id = match check_session().await {
        Ok(true) => {
            let uid: uuid::Uuid = sqlx::query_scalar("SELECT id FROM \"user\" LIMIT 1")
                .fetch_one(&state.pool)
                .await?;
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
            let date_last_view: Option<chrono::DateTime<chrono::Utc>> =
                f_row.try_get("date_last_view").unwrap_or(None);
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
    files: Vec<FileModel>,
) -> Result<(), ServerFnError> {
    use crate::auth::check_session;
    use axum::Extension;
    use leptos_axum::extract;
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
        let file_exists: bool =
            sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM file WHERE id = $1)")
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
    use crate::auth::check_session;
    use axum::Extension;
    use leptos_axum::extract;
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
            let date_last_view: Option<chrono::DateTime<chrono::Utc>> =
                f_row.try_get("date_last_view").unwrap_or(None);
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

        let due_date: Option<chrono::DateTime<chrono::Utc>> =
            row.try_get("due_date").unwrap_or(None);
        let completed_at: Option<chrono::DateTime<chrono::Utc>> =
            row.try_get("completed_at").unwrap_or(None);
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
    files: Vec<FileModel>,
) -> Result<(), ServerFnError> {
    use crate::auth::check_session;
    use axum::Extension;
    use leptos_axum::extract;
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

    let parsed_due_date = due_date
        .and_then(|d| chrono::DateTime::parse_from_rfc3339(&d).ok())
        .map(|dt| dt.with_timezone(&chrono::Utc));
    let parsed_completed_at = completed_at
        .and_then(|d| chrono::DateTime::parse_from_rfc3339(&d).ok())
        .map(|dt| dt.with_timezone(&chrono::Utc));

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
        let file_exists: bool =
            sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM file WHERE id = $1)")
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
             VALUES ($1, $2, $3, NOW())",
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
    status: ActivityStatus,
) -> Result<(), ServerFnError> {
    use crate::auth::check_session;
    use axum::Extension;
    use leptos_axum::extract;
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
    use crate::auth::check_session;
    use axum::Extension;
    use leptos_axum::extract;
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

mod leads_views;
pub use leads_views::LeadTable;
