use reqwest::StatusCode;
use crate::api::client::{api_url, create_client, with_credentials};
use crate::api::models::{FileModel, CreateFileInput, UpdateFileInput};

pub async fn create_file(data: CreateFileInput) -> Result<FileModel, String> {
    let client = create_client();
    let url = api_url("/api/admin/files");
    let req = with_credentials(client.post(&url)).json(&data);
    let res = req.send().await.map_err(|e| e.to_string())?;
    
    if res.status() == StatusCode::CREATED || res.status() == StatusCode::OK {
        res.json::<FileModel>().await.map_err(|e| e.to_string())
    } else {
        Err(format!("Failed to create file: {}", res.status()))
    }
}

pub async fn update_file(id: &str, data: UpdateFileInput) -> Result<FileModel, String> {
    let client = create_client();
    let url = api_url(&format!("/api/admin/files/{}", id));
    let req = with_credentials(client.put(&url)).json(&data);
    let res = req.send().await.map_err(|e| e.to_string())?;
    
    if res.status() == StatusCode::OK {
        res.json::<FileModel>().await.map_err(|e| e.to_string())
    } else {
        Err(format!("Failed to update file: {}", res.status()))
    }
}

pub async fn get_file(id: &str) -> Result<FileModel, String> {
    let client = create_client();
    let url = api_url(&format!("/api/admin/files/{}", id));
    let req = with_credentials(client.get(&url));
    let res = req.send().await.map_err(|e| e.to_string())?;
    
    if res.status() == StatusCode::OK {
        res.json::<FileModel>().await.map_err(|e| e.to_string())
    } else {
        Err(format!("Failed to fetch file: {}", res.status()))
    }
}

pub async fn delete_file(id: &str) -> Result<(), String> {
    let client = create_client();
    let url = api_url(&format!("/api/admin/files/{}", id));
    let req = with_credentials(client.delete(&url));
    let res = req.send().await.map_err(|e| e.to_string())?;
    
    if res.status() == StatusCode::NO_CONTENT || res.status() == StatusCode::OK {
        Ok(())
    } else {
        Err(format!("Failed to delete file: {}", res.status()))
    }
}

pub async fn get_user_files(user_id: &str) -> Result<Vec<FileModel>, String> {
    let client = create_client();
    let url = api_url(&format!("/api/admin/files/user/{}", user_id));
    let req = with_credentials(client.get(&url));
    let res = req.send().await.map_err(|e| e.to_string())?;
    
    if res.status() == StatusCode::OK {
        res.json::<Vec<FileModel>>().await.map_err(|e| e.to_string())
    } else {
        Err(format!("Failed to fetch user files: {}", res.status()))
    }
}

pub async fn associate_file(file_id: &str, entity_type: &str, entity_id: &str) -> Result<(), String> {
    let client = create_client();
    let url = api_url(&format!("/api/admin/files/{}/associate/{}/{}", file_id, entity_type, entity_id));
    let req = with_credentials(client.post(&url));
    let res = req.send().await.map_err(|e| e.to_string())?;
    
    if res.status() == StatusCode::CREATED || res.status() == StatusCode::OK {
        Ok(())
    } else {
        Err(format!("Failed to associate file: {}", res.status()))
    }
}

pub async fn disassociate_file(file_id: &str, entity_type: &str, entity_id: &str) -> Result<(), String> {
    let client = create_client();
    let url = api_url(&format!("/api/admin/files/{}/associate/{}/{}", file_id, entity_type, entity_id));
    let req = with_credentials(client.delete(&url));
    let res = req.send().await.map_err(|e| e.to_string())?;
    
    if res.status() == StatusCode::NO_CONTENT || res.status() == StatusCode::OK {
        Ok(())
    } else {
        Err(format!("Failed to disassociate file: {}", res.status()))
    }
}

pub async fn get_associated_files(entity_type: &str, entity_id: &str) -> Result<Vec<FileModel>, String> {
    let client = create_client();
    let url = api_url(&format!("/api/admin/files/associated/{}/{}", entity_type, entity_id));
    let req = with_credentials(client.get(&url));
    let res = req.send().await.map_err(|e| e.to_string())?;
    
    if res.status() == StatusCode::OK {
        res.json::<Vec<FileModel>>().await.map_err(|e| e.to_string())
    } else {
        Err(format!("Failed to fetch associated files: {}", res.status()))
    }
}

// ── R2 Presigned Upload ───────────────────────────────────────────────────────

#[derive(serde::Serialize)]
struct PresignReq<'a> {
    filename: &'a str,
    content_type: &'a str,
    folder: &'a str,
}

#[derive(serde::Deserialize)]
pub struct PresignResponse {
    pub upload_url: String,
    pub file_key: String,
    pub public_url: String,
}

/// Get a presigned R2 PUT URL scoped to the admin user.
/// Calls `POST /api/admin/upload-presign`.
pub async fn get_admin_presign(filename: &str, content_type: &str, folder: &str) -> Result<PresignResponse, String> {
    let client = create_client();
    let url = api_url("api/admin/upload-presign");
    let req = with_credentials(client.post(&url).json(&PresignReq { filename, content_type, folder }));
    let res = req.send().await.map_err(|e| e.to_string())?;
    if res.status().is_success() {
        res.json::<PresignResponse>().await.map_err(|e| e.to_string())
    } else {
        Err(res.text().await.unwrap_or_default())
    }
}

/// PUT file bytes directly to an R2 presigned URL (no backend proxy).
pub async fn put_to_presigned_url(upload_url: &str, bytes: Vec<u8>, content_type: &str) -> Result<(), String> {
    let client = reqwest::Client::new();
    let res = client
        .put(upload_url)
        .header("Content-Type", content_type)
        .body(bytes)
        .send()
        .await
        .map_err(|e| e.to_string())?;
    if res.status().is_success() { Ok(()) } else { Err(format!("R2 PUT {}", res.status())) }
}

/// Associate an uploaded file (by file_key/id) with a User entity as their avatar.
/// This uses the existing file_association system — no DB migration needed.
/// After this call, `GET /api/admin/files/associated/User/{user_id}` returns the avatar.
pub async fn set_user_avatar(user_id: &str, file_id: &str) -> Result<(), String> {
    associate_file(file_id, "User", user_id).await
}

/// Alias for `create_file` — creates a file record after a successful R2 upload.
/// Returns `FileModel` which has `.id` for subsequent association calls.
pub async fn create_file_record(data: crate::api::models::CreateFileInput) -> Result<crate::api::models::FileModel, String> {
    create_file(data).await
}
