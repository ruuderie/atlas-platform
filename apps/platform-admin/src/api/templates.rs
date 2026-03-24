use super::client::{api_url, create_client, with_credentials, ApiErrorResponse};
use super::models::TemplateModel;
use reqwest::StatusCode;

pub async fn get_templates() -> Result<Vec<TemplateModel>, String> {
    let client = create_client();
    let url = api_url("/api/admin/templates");

    let req = client.get(&url);
    let req = with_credentials(req);

    let res = req.send().await.map_err(|e| e.to_string())?;
    
    if res.status() == StatusCode::OK {
        res.json::<Vec<TemplateModel>>().await.map_err(|e| e.to_string())
    } else {
        Err(format!("Failed to fetch templates: {}", res.status()))
    }
}
