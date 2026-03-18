use super::client::{api_url, create_client, with_credentials, ApiErrorResponse};
use super::models::{DirectoryModel, CreateDirectory};
use reqwest::StatusCode;

pub async fn get_directories() -> Result<Vec<DirectoryModel>, String> {
    let client = create_client();
    let url = api_url("/directories"); // public route

    let req = client.get(&url);
    let req = with_credentials(req);

    if let Ok(res) = req.send().await {
        if res.status() == StatusCode::OK {
            if let Ok(dirs) = res.json::<Vec<DirectoryModel>>().await { return Ok(dirs); }
        }
    }
    if crate::api::client::is_demo_mode() { Ok(vec![
        DirectoryModel { 
            id: "d1".into(), name: "Global Operations".into(), directory_type_id: "dt1".into(), domain: "global.local".into(), description: "Main tenant".into(), 
            created_at: "2026-03-18".into(), updated_at: "2026-03-18".into(), enabled_modules: 5, theme: None, site_status: "Active".into(), subdomain: None, custom_domain: None 
        },
        DirectoryModel { 
            id: "d2".into(), name: "APAC Region".into(), directory_type_id: "dt1".into(), domain: "apac.local".into(), description: "Asia operations".into(), 
            created_at: "2026-03-18".into(), updated_at: "2026-03-18".into(), enabled_modules: 5, theme: None, site_status: "Active".into(), subdomain: None, custom_domain: None 
        },
    ]) } else { Err("Network Error: Backend unreachable".into()) }
}

pub async fn create_directory(data: CreateDirectory) -> Result<DirectoryModel, String> {
    let client = create_client();
    let url = api_url("/api/directories"); // auth route

    let req = client.post(&url).json(&data);
    let req = with_credentials(req);

    let res = req.send().await.map_err(|e| e.to_string())?;

    if res.status() == StatusCode::CREATED || res.status() == StatusCode::OK {
        let dir = res.json::<DirectoryModel>().await.map_err(|e| e.to_string())?;
        Ok(dir)
    } else {
        let err: ApiErrorResponse = res.json().await.unwrap_or(ApiErrorResponse {
            message: Some("Failed to create directory".into()),
            error: None,
        });
        Err(err.message.unwrap_or_else(|| err.error.unwrap_or_else(|| "Unknown error".into())))
    }
}
