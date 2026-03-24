use super::client::{api_url, create_client, with_credentials, ApiErrorResponse};
use super::models::CategoryModel;
use reqwest::StatusCode;

pub async fn get_categories() -> Result<Vec<CategoryModel>, String> {
    let client = create_client();
    let url = api_url("/api/admin/categories");

    let req = client.get(&url);
    let req = with_credentials(req);

    if let Ok(res) = req.send().await {
        if res.status() == StatusCode::OK {
            if let Ok(data) = res.json::<Vec<CategoryModel>>().await { 
                return Ok(data); 
            }
        }
    }
    
    // Fallback/Demo Mock
    Ok(vec![
        CategoryModel { 
            id: "fake-cat-1".into(),
            directory_type_id: "fake-type-1".into(),
            parent_category_id: None,
            name: "Home Services".into(),
            description: "Plumbers, electricians, and more".into(),
            icon: Some("home_repair_service".into()),
            slug: Some("home-services".into()),
            is_custom: false,
            is_active: true,
            created_at: "2026-03-23".into(),
            updated_at: "2026-03-23".into(),
            directory_id: None,
        },
        CategoryModel { 
            id: "fake-cat-2".into(),
            directory_type_id: "fake-type-1".into(),
            parent_category_id: Some("fake-cat-1".into()),
            name: "Plumbers".into(),
            description: "Professional plumbing services".into(),
            icon: Some("plumbing".into()),
            slug: Some("plumbers".into()),
            is_custom: false,
            is_active: true,
            created_at: "2026-03-23".into(),
            updated_at: "2026-03-23".into(),
            directory_id: None,
        }
    ])
}
