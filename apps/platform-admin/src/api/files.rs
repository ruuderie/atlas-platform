use reqwest::StatusCode;
use crate::api::client::{api_url, create_client, with_credentials};
use crate::api::models::{FileModel, CreateFileInput, UpdateFileInput};

pub async fn create_file(data: CreateFileInput) -> Result<FileModel, String> {
    let client = create_client();
    let url = api_url("/api/files");
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
    let url = api_url(&format!("/api/files/{}", id));
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
    let url = api_url(&format!("/api/files/{}", id));
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
    let url = api_url(&format!("/api/files/{}", id));
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
    let url = api_url(&format!("/api/files/user/{}", user_id));
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
    let url = api_url(&format!("/api/files/{}/associate/{}/{}", file_id, entity_type, entity_id));
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
    let url = api_url(&format!("/api/files/{}/associate/{}/{}", file_id, entity_type, entity_id));
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
    let url = api_url(&format!("/api/files/associated/{}/{}", entity_type, entity_id));
    let req = with_credentials(client.get(&url));
    let res = req.send().await.map_err(|e| e.to_string())?;
    
    if res.status() == StatusCode::OK {
        res.json::<Vec<FileModel>>().await.map_err(|e| e.to_string())
    } else {
        Err(format!("Failed to fetch associated files: {}", res.status()))
    }
}
