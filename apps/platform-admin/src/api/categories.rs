use super::client::{api_url, create_client, with_credentials, ApiErrorResponse};
use super::models::CategoryModel;
use reqwest::StatusCode;

pub async fn get_categories(directory_id: Option<String>) -> Result<Vec<CategoryModel>, String> {
    let client = create_client();
    let url = match directory_id {
        Some(d) => api_url(&format!("/api/admin/categories?directory_id={}", d)),
        None => api_url("/api/admin/categories"),
    };

    let req = client.get(&url);
    let req = with_credentials(req);

    let res = req.send().await.map_err(|e| e.to_string())?;
    
    if res.status() == StatusCode::OK {
        res.json::<Vec<CategoryModel>>().await.map_err(|e| e.to_string())
    } else {
        Err(format!("Failed to fetch categories: {}", res.status()))
    }
}
