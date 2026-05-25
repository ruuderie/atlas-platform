use leptos::prelude::*;
use shared_ui::components::crm_stage_bar::{CrmStageBar, CrmStatusOption};
use shared_ui::components::crm_timeline::{CrmTimeline, CrmNote, CrmActivity};
use shared_ui::components::properties_editor::PropertiesEditor;
use shared_ui::utils::ResourceState;
use shared_ui::components::email_composer::{EmailComposer, EmailTemplate};
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
    pub avatar_url: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[server(GetNetworkContacts, "/api")]
pub async fn get_contacts() -> Result<Vec<ContactRecord>, ServerFnError> {
    use axum::http::request::Parts;
    use crate::auth::api_base_url;

    let session_cookie = if let Some(req_parts) = use_context::<Parts>() {
        req_parts
            .headers
            .get("cookie")
            .and_then(|v| v.to_str().ok())
            .and_then(|cookies| {
                cookies.split(';').find_map(|part| {
                    let part = part.trim();
                    part.strip_prefix("session=").map(|t| t.to_string())
                })
            })
    } else {
        None
    };

    let Some(token) = session_cookie else {
        return Err(ServerFnError::new("Unauthorized"));
    };

    let url = format!("{}/api/contacts", api_base_url());
    let client = reqwest::Client::new();
    let res = client
        .get(&url)
        .header("Cookie", format!("session={}", token))
        .send()
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;

    if res.status().is_success() {
        let items: Vec<serde_json::Value> = res.json().await.map_err(|e| ServerFnError::new(e.to_string()))?;
        let formatted = items.into_iter().map(|item| {
            let id = uuid::Uuid::parse_str(item.get("id").and_then(|v| v.as_str()).unwrap_or_default()).unwrap_or_default();
            let customer_id = item.get("customer_id").and_then(|v| v.as_str()).and_then(|s| uuid::Uuid::parse_str(s).ok());
            let name = item.get("name").and_then(|v| v.as_str()).unwrap_or_default().to_string();
            let first_name = item.get("first_name").and_then(|v| v.as_str()).map(String::from);
            let last_name = item.get("last_name").and_then(|v| v.as_str()).map(String::from);
            let email = item.get("email").and_then(|v| v.as_str()).map(String::from);
            let phone = item.get("phone").and_then(|v| v.as_str()).map(String::from);
            let whatsapp = item.get("whatsapp").and_then(|v| v.as_str()).map(String::from);
            let telegram = item.get("telegram").and_then(|v| v.as_str()).map(String::from);
            let twitter = item.get("twitter").and_then(|v| v.as_str()).map(String::from);
            let instagram = item.get("instagram").and_then(|v| v.as_str()).map(String::from);
            let facebook = item.get("facebook").and_then(|v| v.as_str()).map(String::from);
            let properties = item.get("properties").cloned();
            let avatar_url = item.get("avatar_url").and_then(|v| v.as_str()).map(String::from);
            
            // Format dates
            let created_at_str = item.get("created_at").and_then(|v| v.as_str()).unwrap_or_default();
            let created_at = chrono::DateTime::parse_from_rfc3339(created_at_str)
                .map(|dt| dt.with_timezone(&chrono::Utc))
                .map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string())
                .unwrap_or_else(|_| created_at_str.to_string());
                
            let updated_at_str = item.get("updated_at").and_then(|v| v.as_str()).unwrap_or_default();
            let updated_at = chrono::DateTime::parse_from_rfc3339(updated_at_str)
                .map(|dt| dt.with_timezone(&chrono::Utc))
                .map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string())
                .unwrap_or_else(|_| updated_at_str.to_string());

            ContactRecord {
                id,
                customer_id,
                name,
                first_name,
                last_name,
                email,
                phone,
                whatsapp,
                telegram,
                twitter,
                instagram,
                facebook,
                properties,
                avatar_url,
                created_at,
                updated_at,
            }
        }).collect();
        Ok(formatted)
    } else {
        Err(ServerFnError::new("Failed to fetch contacts from backend"))
    }
}

#[server(DeleteNetworkContact, "/api")]
pub async fn delete_contact(id: uuid::Uuid) -> Result<(), ServerFnError> {
    use axum::http::request::Parts;
    use crate::auth::api_base_url;

    let session_cookie = if let Some(req_parts) = use_context::<Parts>() {
        req_parts
            .headers
            .get("cookie")
            .and_then(|v| v.to_str().ok())
            .and_then(|cookies| {
                cookies.split(';').find_map(|part| {
                    let part = part.trim();
                    part.strip_prefix("session=").map(|t| t.to_string())
                })
            })
    } else {
        None
    };

    let Some(token) = session_cookie else {
        return Err(ServerFnError::new("Unauthorized"));
    };

    let url = format!("{}/api/contacts/{}", api_base_url(), id);
    let client = reqwest::Client::new();
    let res = client
        .delete(&url)
        .header("Cookie", format!("session={}", token))
        .send()
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;

    if res.status().is_success() {
        Ok(())
    } else {
        Err(ServerFnError::new("Failed to delete contact"))
    }
}

#[server(GetNetworkContactCrmStatuses, "/api")]
pub async fn get_contact_crm_statuses() -> Result<Vec<CrmStatusOption>, ServerFnError> {
    use axum::http::request::Parts;
    use crate::auth::api_base_url;

    let session_cookie = if let Some(req_parts) = use_context::<Parts>() {
        req_parts
            .headers
            .get("cookie")
            .and_then(|v| v.to_str().ok())
            .and_then(|cookies| {
                cookies.split(';').find_map(|part| {
                    let part = part.trim();
                    part.strip_prefix("session=").map(|t| t.to_string())
                })
            })
    } else {
        None
    };

    let Some(token) = session_cookie else {
        return Err(ServerFnError::new("Unauthorized"));
    };

    let url = format!("{}/api/crm/status-options?object_type=Contact", api_base_url());
    let client = reqwest::Client::new();
    let res = client
        .get(&url)
        .header("Cookie", format!("session={}", token))
        .send()
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;

    if res.status().is_success() {
        let items: Vec<serde_json::Value> = res.json().await.map_err(|e| ServerFnError::new(e.to_string()))?;
        let options = items.into_iter().map(|item| {
            CrmStatusOption {
                status_key: item.get("status_key").and_then(|v| v.as_str()).unwrap_or_default().to_string(),
                label: item.get("label").and_then(|v| v.as_str()).unwrap_or_default().to_string(),
                color: item.get("color").and_then(|v| v.as_str()).unwrap_or_default().to_string(),
                sort_order: item.get("sort_order").and_then(|v| v.as_i64()).unwrap_or(0) as i32,
                is_system: item.get("is_system").and_then(|v| v.as_bool()).unwrap_or(false),
            }
        }).collect();
        Ok(options)
    } else {
        Err(ServerFnError::new("Failed to fetch contact crm statuses"))
    }
}

#[server(UpdateNetworkContactDetails, "/api")]
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
    use axum::http::request::Parts;
    use crate::auth::api_base_url;

    let session_cookie = if let Some(req_parts) = use_context::<Parts>() {
        req_parts
            .headers
            .get("cookie")
            .and_then(|v| v.to_str().ok())
            .and_then(|cookies| {
                cookies.split(';').find_map(|part| {
                    let part = part.trim();
                    part.strip_prefix("session=").map(|t| t.to_string())
                })
            })
    } else {
        None
    };

    let Some(token) = session_cookie else {
        return Err(ServerFnError::new("Unauthorized"));
    };

    let url = format!("{}/api/contacts/{}", api_base_url(), id);
    let client = reqwest::Client::new();
    
    let payload = serde_json::json!({
        "name": name,
        "first_name": first_name,
        "last_name": last_name,
        "email": email,
        "phone": phone,
        "whatsapp": whatsapp,
        "telegram": telegram,
        "twitter": twitter,
        "instagram": instagram,
        "facebook": facebook,
        "properties": properties,
        "avatar_url": avatar_url
    });

    let res = client
        .put(&url)
        .header("Cookie", format!("session={}", token))
        .json(&payload)
        .send()
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;

    if res.status().is_success() {
        Ok(())
    } else {
        Err(ServerFnError::new("Failed to update contact details"))
    }
}

#[server(SendNetworkCrmEmail, "/api")]
pub async fn send_network_crm_email(
    to_email: String,
    subject: String,
    body_html: String,
    contact_id: Option<uuid::Uuid>,
    lead_id: Option<uuid::Uuid>,
    attachments: Vec<String>,
) -> Result<(), ServerFnError> {
    use axum::http::request::Parts;
    use crate::auth::api_base_url;

    let session_cookie = if let Some(req_parts) = use_context::<Parts>() {
        req_parts
            .headers
            .get("cookie")
            .and_then(|v| v.to_str().ok())
            .and_then(|cookies| {
                cookies.split(';').find_map(|part| {
                    let part = part.trim();
                    part.strip_prefix("session=").map(|t| t.to_string())
                })
            })
    } else {
        None
    };

    let Some(token) = session_cookie else {
        return Err(ServerFnError::new("Unauthorized"));
    };

    let client = reqwest::Client::new();

    // 1. Resolve active tenant context from backend GET /api/users/me
    let me_url = format!("{}/api/users/me", api_base_url());
    let me_res = client
        .get(&me_url)
        .header("Cookie", format!("session={}", token))
        .send()
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;

    if !me_res.status().is_success() {
        return Err(ServerFnError::new("Unauthorized"));
    }

    let me_val: serde_json::Value = me_res.json().await.map_err(|e| ServerFnError::new(e.to_string()))?;
    let tenant_id_str = me_val.get("tenant_id").and_then(|v| v.as_str()).ok_or_else(|| ServerFnError::new("Missing tenant context"))?;
    let tenant_id = uuid::Uuid::parse_str(tenant_id_str).map_err(|e| ServerFnError::new(e.to_string()))?;

    // 2. Post email dispatch to `/api/communications/email`
    let email_url = format!("{}/api/communications/email", api_base_url());
    let email_payload = serde_json::json!({
        "tenant_id": tenant_id,
        "to_email": to_email,
        "subject": subject.clone(),
        "body_html": body_html.clone(),
        "attachments": attachments,
    });

    let email_res = client
        .post(&email_url)
        .header("Cookie", format!("session={}", token))
        .json(&email_payload)
        .send()
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;

    if !email_res.status().is_success() {
        let err_text = email_res.text().await.unwrap_or_default();
        return Err(ServerFnError::new(format!("Email delivery failed: {}", err_text)));
    }

    // 3. Log a completed activity record in the database for the contact or lead
    let activity_url = format!("{}/api/activities", api_base_url());
    let activity_payload = serde_json::json!({
        "contact_id": contact_id,
        "lead_id": lead_id,
        "activity_type": "Email",
        "title": format!("Email Sent: {}", subject),
        "description": body_html,
        "status": "Completed"
    });

    let _ = client
        .post(&activity_url)
        .header("Cookie", format!("session={}", token))
        .json(&activity_payload)
        .send()
        .await;

    Ok(())
}

#[server(GetNetworkContactAttachments, "/api")]
pub async fn get_contact_attachments(contact_id: uuid::Uuid) -> Result<Vec<RecordDocumentModel>, ServerFnError> {
    use axum::http::request::Parts;
    use crate::auth::api_base_url;

    let session_cookie = if let Some(req_parts) = use_context::<Parts>() {
        req_parts
            .headers
            .get("cookie")
            .and_then(|v| v.to_str().ok())
            .and_then(|cookies| {
                cookies.split(';').find_map(|part| {
                    let part = part.trim();
                    part.strip_prefix("session=").map(|t| t.to_string())
                })
            })
    } else {
        None
    };

    let Some(token) = session_cookie else {
        return Err(ServerFnError::new("Unauthorized"));
    };

    let url = format!("{}/api/admin/files/associated/Contact/{}", api_base_url(), contact_id);
    let client = reqwest::Client::new();
    let res = client
        .get(&url)
        .header("Cookie", format!("session={}", token))
        .send()
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;

    if res.status().is_success() {
        let items: Vec<serde_json::Value> = res.json().await.map_err(|e| ServerFnError::new(e.to_string()))?;
        let docs = items.into_iter().map(|item| {
            let id = uuid::Uuid::parse_str(item.get("id").and_then(|v| v.as_str()).unwrap_or_default()).unwrap_or_default();
            let file_url = item.get("storage_path").and_then(|v| v.as_str()).unwrap_or_default().to_string();
            let file_name = item.get("name").and_then(|v| v.as_str()).unwrap_or_default().to_string();
            
            let uploaded_at_str = item.get("date_upload").and_then(|v| v.as_str()).unwrap_or_default();
            let uploaded_at = chrono::DateTime::parse_from_rfc3339(uploaded_at_str)
                .map(|dt| dt.with_timezone(&chrono::Utc))
                .map(|dt| dt.format("%Y-%m-%d %H:%M").to_string())
                .unwrap_or_else(|_| uploaded_at_str.to_string());

            RecordDocumentModel {
                id,
                tenant_id: uuid::Uuid::default(),
                target_record_id: contact_id,
                file_url,
                file_name,
                uploaded_at,
            }
        }).collect();
        Ok(docs)
    } else {
        Err(ServerFnError::new("Failed to fetch contact documents"))
    }
}

#[server(AddNetworkContactAttachment, "/api")]
pub async fn add_contact_attachment(contact_id: uuid::Uuid, file_name: String, file_url: String) -> Result<(), ServerFnError> {
    use axum::http::request::Parts;
    use crate::auth::api_base_url;

    let session_cookie = if let Some(req_parts) = use_context::<Parts>() {
        req_parts
            .headers
            .get("cookie")
            .and_then(|v| v.to_str().ok())
            .and_then(|cookies| {
                cookies.split(';').find_map(|part| {
                    let part = part.trim();
                    part.strip_prefix("session=").map(|t| t.to_string())
                })
            })
    } else {
        None
    };

    let Some(token) = session_cookie else {
        return Err(ServerFnError::new("Unauthorized"));
    };

    let client = reqwest::Client::new();
    
    // 1. Create file record
    let create_url = format!("{}/api/admin/files", api_base_url());
    let create_payload = serde_json::json!({
        "name": file_name,
        "size": 0,
        "mime_type": "application/octet-stream",
        "hash_sha256": "",
        "storage_type": "S",
        "storage_path": file_url,
        "is_anonymous": false,
        "user_id": null
    });
    
    let create_res = client
        .post(&create_url)
        .header("Cookie", format!("session={}", token))
        .json(&create_payload)
        .send()
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;

    if !create_res.status().is_success() {
        return Err(ServerFnError::new("Failed to create file record"));
    }

    let created_file: serde_json::Value = create_res.json().await.map_err(|e| ServerFnError::new(e.to_string()))?;
    let file_id = created_file.get("id").and_then(|v| v.as_str()).ok_or_else(|| ServerFnError::new("Missing file ID"))?;

    // 2. Create file association
    let associate_url = format!("{}/api/admin/files/{}/associate/Contact/{}", api_base_url(), file_id, contact_id);
    let associate_res = client
        .post(&associate_url)
        .header("Cookie", format!("session={}", token))
        .send()
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;

    if associate_res.status().is_success() {
        Ok(())
    } else {
        Err(ServerFnError::new("Failed to associate file with contact"))
    }
}

#[server(DeleteNetworkContactAttachment, "/api")]
pub async fn delete_contact_attachment(contact_id: uuid::Uuid, doc_id: uuid::Uuid) -> Result<(), ServerFnError> {
    use axum::http::request::Parts;
    use crate::auth::api_base_url;

    let session_cookie = if let Some(req_parts) = use_context::<Parts>() {
        req_parts
            .headers
            .get("cookie")
            .and_then(|v| v.to_str().ok())
            .and_then(|cookies| {
                cookies.split(';').find_map(|part| {
                    let part = part.trim();
                    part.strip_prefix("session=").map(|t| t.to_string())
                })
            })
    } else {
        None
    };

    let Some(token) = session_cookie else {
        return Err(ServerFnError::new("Unauthorized"));
    };

    let url = format!("{}/api/admin/files/{}", api_base_url(), doc_id);
    let client = reqwest::Client::new();
    let res = client
        .delete(&url)
        .header("Cookie", format!("session={}", token))
        .send()
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;

    if res.status().is_success() {
        Ok(())
    } else {
        Err(ServerFnError::new("Failed to delete contact document"))
    }
}

#[server(GetNetworkAttachmentDownloadUrl, "/api")]
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

#[server(GetNetworkContactNotes, "/api")]
pub async fn get_contact_notes(contact_id: uuid::Uuid) -> Result<Vec<CrmNote>, ServerFnError> {
    use axum::http::request::Parts;
    use crate::auth::api_base_url;

    let session_cookie = if let Some(req_parts) = use_context::<Parts>() {
        req_parts
            .headers
            .get("cookie")
            .and_then(|v| v.to_str().ok())
            .and_then(|cookies| {
                cookies.split(';').find_map(|part| {
                    let part = part.trim();
                    part.strip_prefix("session=").map(|t| t.to_string())
                })
            })
    } else {
        None
    };

    let Some(token) = session_cookie else {
        return Err(ServerFnError::new("Unauthorized"));
    };

    let url = format!("{}/api/contacts/{}/notes", api_base_url(), contact_id);
    let client = reqwest::Client::new();
    let res = client
        .get(&url)
        .header("Cookie", format!("session={}", token))
        .send()
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;

    if res.status().is_success() {
        let items: Vec<serde_json::Value> = res.json().await.map_err(|e| ServerFnError::new(e.to_string()))?;
        let notes = items.into_iter().map(|item| {
            let created_at_str = item.get("created_at").and_then(|v| v.as_str()).unwrap_or_default();
            let created_at = chrono::DateTime::parse_from_rfc3339(created_at_str)
                .map(|dt| dt.with_timezone(&chrono::Utc))
                .map(|dt| dt.format("%Y-%m-%d %H:%M").to_string())
                .unwrap_or_else(|_| created_at_str.to_string());

            CrmNote {
                id: item.get("id").and_then(|v| v.as_str()).unwrap_or_default().to_string(),
                content: item.get("content").and_then(|v| v.as_str()).unwrap_or_default().to_string(),
                created_at,
            }
        }).collect();
        Ok(notes)
    } else {
        Err(ServerFnError::new("Failed to fetch contact notes"))
    }
}

#[server(AddNetworkContactNote, "/api")]
pub async fn add_contact_note(contact_id: uuid::Uuid, content: String) -> Result<(), ServerFnError> {
    use axum::http::request::Parts;
    use crate::auth::api_base_url;

    let session_cookie = if let Some(req_parts) = use_context::<Parts>() {
        req_parts
            .headers
            .get("cookie")
            .and_then(|v| v.to_str().ok())
            .and_then(|cookies| {
                cookies.split(';').find_map(|part| {
                    let part = part.trim();
                    part.strip_prefix("session=").map(|t| t.to_string())
                })
            })
    } else {
        None
    };

    let Some(token) = session_cookie else {
        return Err(ServerFnError::new("Unauthorized"));
    };

    let url = format!("{}/api/notes", api_base_url());
    let client = reqwest::Client::new();
    
    let payload = serde_json::json!({
        "entity_type": "Contact",
        "entity_id": contact_id,
        "content": content
    });

    let res = client
        .post(&url)
        .header("Cookie", format!("session={}", token))
        .json(&payload)
        .send()
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;

    if res.status().is_success() {
        Ok(())
    } else {
        Err(ServerFnError::new("Failed to add contact note"))
    }
}

#[server(GetNetworkContactActivities, "/api")]
pub async fn get_contact_activities(contact_id: uuid::Uuid) -> Result<Vec<CrmActivity>, ServerFnError> {
    use axum::http::request::Parts;
    use crate::auth::api_base_url;

    let session_cookie = if let Some(req_parts) = use_context::<Parts>() {
        req_parts
            .headers
            .get("cookie")
            .and_then(|v| v.to_str().ok())
            .and_then(|cookies| {
                cookies.split(';').find_map(|part| {
                    let part = part.trim();
                    part.strip_prefix("session=").map(|t| t.to_string())
                })
            })
    } else {
        None
    };

    let Some(token) = session_cookie else {
        return Err(ServerFnError::new("Unauthorized"));
    };

    let url = format!("{}/api/contacts/{}/activities", api_base_url(), contact_id);
    let client = reqwest::Client::new();
    let res = client
        .get(&url)
        .header("Cookie", format!("session={}", token))
        .send()
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;

    if res.status().is_success() {
        let items: Vec<serde_json::Value> = res.json().await.map_err(|e| ServerFnError::new(e.to_string()))?;
        let activities = items.into_iter().map(|item| {
            let created_at_str = item.get("created_at").and_then(|v| v.as_str()).unwrap_or_default();
            let created_at = chrono::DateTime::parse_from_rfc3339(created_at_str)
                .map(|dt| dt.with_timezone(&chrono::Utc))
                .map(|dt| dt.format("%Y-%m-%d %H:%M").to_string())
                .unwrap_or_else(|_| created_at_str.to_string());

            CrmActivity {
                id: item.get("id").and_then(|v| v.as_str()).unwrap_or_default().to_string(),
                activity_type: item.get("activity_type").and_then(|v| v.as_str()).unwrap_or_default().to_string(),
                description: item.get("description").and_then(|v| v.as_str()).unwrap_or_default().to_string(),
                created_at,
            }
        }).collect();
        Ok(activities)
    } else {
        Err(ServerFnError::new("Failed to fetch contact activities"))
    }
}

#[server(LogNetworkContactActivity, "/api")]
pub async fn log_contact_activity(contact_id: uuid::Uuid, activity_type: String, description: String) -> Result<(), ServerFnError> {
    use axum::http::request::Parts;
    use crate::auth::api_base_url;

    let session_cookie = if let Some(req_parts) = use_context::<Parts>() {
        req_parts
            .headers
            .get("cookie")
            .and_then(|v| v.to_str().ok())
            .and_then(|cookies| {
                cookies.split(';').find_map(|part| {
                    let part = part.trim();
                    part.strip_prefix("session=").map(|t| t.to_string())
                })
            })
    } else {
        None
    };

    let Some(token) = session_cookie else {
        return Err(ServerFnError::new("Unauthorized"));
    };

    let url = format!("{}/api/activities", api_base_url());
    let client = reqwest::Client::new();
    
    let payload = serde_json::json!({
        "contact_id": contact_id,
        "activity_type": activity_type.clone(),
        "title": format!("Logged {}", activity_type),
        "description": description,
        "status": "Completed"
    });

    let res = client
        .post(&url)
        .header("Cookie", format!("session={}", token))
        .json(&payload)
        .send()
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;

    if res.status().is_success() {
        Ok(())
    } else {
        Err(ServerFnError::new("Failed to log contact activity"))
    }
}

#[component]
pub fn ContactTable() -> impl IntoView {
    let (refresh, set_refresh) = signal(0);
    let contacts_res = Resource::new(move || refresh.get(), |_| get_contacts());
    let statuses_res = Resource::new(|| (), |_| get_contact_crm_statuses());

    let (selected_contact, set_selected_contact) = signal::<Option<ContactRecord>>(None);

    view! {
        <Transition fallback=move || view! { <div class="jetbrains text-sm text-outline">"QUERYING_DB..."</div> }>
            {move || {
                let res = contacts_res.get();
                let statuses = statuses_res.get().and_then(|r| r.ok()).unwrap_or_default();
                view! {
                    <div class="relative w-full">
                        <Show
                            when=move || selected_contact.get().is_none()
                            fallback={
                                let statuses = statuses.clone();
                                move || {
                                    let statuses = statuses.clone();
                                    view! {
                                        {move || selected_contact.get().map(|contact| {
                                            view! {
                                                <ContactCrmPane 
                                                    contact_record=contact
                                                    stages=statuses.clone()
                                                    on_close=Callback::new(move |_: ()| set_selected_contact.set(None))
                                                    set_refresh=set_refresh
                                                    refresh=refresh
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
                                                                on:click=move |_| set_selected_contact.set(Some(c.clone()))
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
                                                                        on:click=move |e| {
                                                                            e.stop_propagation();
                                                                            let id = contact.id;
                                                                            leptos::task::spawn_local(async move {
                                                                                if let Ok(_) = delete_contact(id).await {
                                                                                    set_refresh.set(refresh.get_untracked() + 1);
                                                                                    if selected_contact.get().map(|s| s.id) == Some(id) {
                                                                                        set_selected_contact.set(None);
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
    set_refresh: WriteSignal<i32>,
    refresh: ReadSignal<i32>,
) -> impl IntoView {
    // Extract status from properties JSON
    let status_val = contact_record.properties.as_ref()
        .and_then(|p| p.get("status"))
        .and_then(|s| s.as_str())
        .unwrap_or("prospect")
        .to_string();

    let (current_stage, set_current_stage) = signal(status_val);
    
    let (composer_open, set_composer_open) = signal(false);

    let default_templates = vec![
        EmailTemplate {
            name: "Intake Follow-Up".to_string(),
            subject: "Following up on your intake inquiry".to_string(),
            body: "<p>Hello,</p><p>Thank you for reaching out. We received your details and are currently reviewing your inquiry. We will get back to you shortly with next steps.</p><p>Best regards,<br/>The Operations Team</p>".to_string(),
        },
        EmailTemplate {
            name: "Proposal Presentation".to_string(),
            subject: "Custom Proposal Presentation".to_string(),
            body: "<p>Hello,</p><p>We are excited to share our custom proposal based on our initial discussion. Please review the attached details and let us know if you have any questions or when you would be available for a quick walkthrough.</p><p>Best regards,<br/>The Consulting Team</p>".to_string(),
        },
    ];
    
    let contact_id = contact_record.id;
    let notes_res = Resource::new(move || refresh.get(), move |_| get_contact_notes(contact_id));
    let activities_res = Resource::new(move || refresh.get(), move |_| get_contact_activities(contact_id));
    let attachments_res = Resource::new(move || refresh.get(), move |_| get_contact_attachments(contact_id));

    // Avatar Url State
    let (avatar_url_signal, set_avatar_url_signal) = signal(contact_record.avatar_url.clone());
    let avatar_input_ref = NodeRef::<leptos::html::Input>::new();
    
    let trigger_avatar_upload = move |_| {
        if let Some(input) = avatar_input_ref.get() {
            input.click();
        }
    };

    let add_attachment_cb = Callback::new(move |(file_name, file_url): (String, String)| {
        leptos::task::spawn_local(async move {
            if let Ok(_) = add_contact_attachment(contact_id, file_name, file_url).await {
                set_refresh.set(refresh.get_untracked() + 1);
            }
        });
    });
    let delete_attachment_cb = Callback::new(move |doc_id: uuid::Uuid| {
        leptos::task::spawn_local(async move {
            if let Ok(_) = delete_contact_attachment(contact_id, doc_id).await {
                set_refresh.set(refresh.get_untracked() + 1);
            }
        });
    });
    let download_attachment_cb = Callback::new(move |file_key: String| {
        leptos::task::spawn_local(async move {
            if let Ok(download_url) = get_attachment_download_url(file_key).await {
                #[cfg(not(feature = "ssr"))]
                if let Some(win) = web_sys::window() {
                    let _ = win.open_with_url_and_target(&download_url, "_blank");
                }
            }
        });
    });

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
    
    let handle_avatar_change = {
        let set_refresh = set_refresh.clone();
        let refresh = refresh.clone();
        let name = name.clone();
        let first_name = first_name.clone();
        let last_name = last_name.clone();
        let email = email.clone();
        let phone = phone.clone();
        let whatsapp = whatsapp.clone();
        let telegram = telegram.clone();
        let twitter = twitter.clone();
        let instagram = instagram.clone();
        let facebook = facebook.clone();
        let properties_signal = properties_signal.clone();
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
                            let fn_val = Some(first_name.get_untracked()).filter(|s: &String| !s.is_empty());
                            let ln_val = Some(last_name.get_untracked()).filter(|s: &String| !s.is_empty());
                            let em_val = Some(email.get_untracked()).filter(|s: &String| !s.is_empty());
                            let ph_val = Some(phone.get_untracked()).filter(|s: &String| !s.is_empty());
                            let wa_val = Some(whatsapp.get_untracked()).filter(|s: &String| !s.is_empty());
                            let tg_val = Some(telegram.get_untracked()).filter(|s: &String| !s.is_empty());
                            let tw_val = Some(twitter.get_untracked()).filter(|s: &String| !s.is_empty());
                            let ig_val = Some(instagram.get_untracked()).filter(|s: &String| !s.is_empty());
                            let fb_val = Some(facebook.get_untracked()).filter(|s: &String| !s.is_empty());
                            let props = properties_signal.get_untracked();
                            let set_refresh = set_refresh.clone();
                            let refresh = refresh.clone();
                            let set_avatar_url_signal = set_avatar_url_signal.clone();
                            
                            leptos::task::spawn_local(async move {
                                if let Ok((_, key)) = shared_ui::components::file_attachments::upload_file_to_s3(file).await {
                                    if let Ok(_) = update_contact_details(
                                        contact_id, name_val, fn_val, ln_val, em_val, ph_val, wa_val, tg_val, tw_val, ig_val, fb_val, props, Some(key.clone())
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
        let avatar_val = avatar_url_signal.get_untracked();

        leptos::task::spawn_local(async move {
            if let Ok(_) = update_contact_details(
                contact_id, n, fn_val, ln_val, em_val, ph_val, wa_val, tg_val, tw_val, ig_val, fb_val, Some(props), avatar_val
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
        let avatar_val = avatar_url_signal.get_untracked();
        
        // Include status in the saved properties JSON
        let mut props = properties_signal.get().unwrap_or(serde_json::Value::Object(serde_json::Map::new()));
        if let serde_json::Value::Object(ref mut map) = props {
            map.insert("status".to_string(), serde_json::Value::String(current_stage.get_untracked()));
        }

        leptos::task::spawn_local(async move {
            match update_contact_details(
                contact_id, n, fn_opt, ln_opt, em_val, ph_val, wa_val, tg_val, tw_val, ig_val, fb_val, Some(props), avatar_val
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
                            <button
                                // Use _set_facebook or ignore unused variable if needed, but here we just toggle edit_mode
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
                    <FileAttachments
                        entity_type="Contact".to_string()
                        entity_id=contact_id
                        files=Signal::derive(move || attachments_res.get().and_then(|r| r.ok()).unwrap_or_default())
                        on_upload=add_attachment_cb
                        on_delete=delete_attachment_cb
                        on_download=download_attachment_cb
                    />
                </div>

            </div>

            <EmailComposer
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
                            if let Ok(_) = send_network_crm_email(to_addr, subj, bdy, Some(contact_id), None, atts).await {
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
