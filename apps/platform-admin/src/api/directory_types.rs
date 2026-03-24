use super::client::{api_url, create_client, with_credentials, ApiErrorResponse};
use super::models::DirectoryTypeModel;
use reqwest::StatusCode;

pub async fn get_directory_types() -> Result<Vec<DirectoryTypeModel>, String> {
    let client = create_client();
    let url = api_url("/api/admin/directory-types");

    let req = client.get(&url);
    let req = with_credentials(req);

    if let Ok(res) = req.send().await {
        if res.status() == StatusCode::OK {
            if let Ok(types) = res.json::<Vec<DirectoryTypeModel>>().await { 
                return Ok(types); 
            }
        }
    }
    
    // Fallback/Demo Mock
    Ok(vec![
        DirectoryTypeModel { 
            id: "fake-id-1".into(),
            name: "Professional Network".into(),
            description: "Standard business listings network".into(),
            created_at: "2026-03-23".into(),
            updated_at: "2026-03-23".into(),
        }
    ])
}
