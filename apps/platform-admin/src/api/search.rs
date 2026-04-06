use crate::api::client::{api_url, api_request, create_client};
use crate::api::models::SearchResult;
use uuid::Uuid;

pub async fn search_global(query: &str, tenant_id: Option<Uuid>) -> Result<Vec<SearchResult>, String> {
    if query.trim().is_empty() {
        return Ok(Vec::new());
    }

    let mut path = format!("/api/v1/search?q={}", urlencoding::encode(query));
    if let Some(tid) = tenant_id {
        path.push_str(&format!("&tenant_id={}", tid));
    }

    let url = api_url(&path);
    let client = create_client();
    let req = client.get(&url);

    api_request::<Vec<SearchResult>>(req).await
}
