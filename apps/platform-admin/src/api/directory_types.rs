use super::client::{api_url, create_client, with_credentials, ApiErrorResponse};
use super::models::DirectoryTypeModel;
use reqwest::StatusCode;

pub async fn get_directory_types() -> Result<Vec<DirectoryTypeModel>, String> {
    let client = create_client();
    let url = api_url("/api/admin/directory-types");

    let req = client.get(&url);
    let req = with_credentials(req);

    let res = req.send().await.map_err(|e| e.to_string())?;
    
    if res.status() == StatusCode::OK {
        res.json::<Vec<DirectoryTypeModel>>().await.map_err(|e| e.to_string())
    } else {
        Err(format!("Failed to fetch directory types: {}", res.status()))
    }
}
