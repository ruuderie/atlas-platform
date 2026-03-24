use super::client::{api_url, create_client, with_credentials, ApiErrorResponse};
use super::models::TemplateModel;
use reqwest::StatusCode;

pub async fn get_templates() -> Result<Vec<TemplateModel>, String> {
    let client = create_client();
    let url = api_url("/api/admin/templates");

    let req = client.get(&url);
    let req = with_credentials(req);

    if let Ok(res) = req.send().await {
        if res.status() == StatusCode::OK {
            if let Ok(data) = res.json::<Vec<TemplateModel>>().await { 
                return Ok(data); 
            }
        }
    }
    
    // Fallback/Demo Mock
    Ok(vec![
        TemplateModel { 
            id: "fake-tpl-1".into(),
            directory_id: "fake-dir-1".into(),
            category_id: "fake-cat-1".into(),
            name: "Premium Plumber Details".into(),
            description: "Advanced listing fields for verified plumbers.".into(),
            template_type: "Listing Extension".into(),
            is_active: true,
            created_at: "2026-03-23".into(),
            updated_at: "2026-03-23".into(),
        }
    ])
}
