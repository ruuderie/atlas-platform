use super::client::{api_url, create_client, with_credentials, api_request};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};

// ── Models ─────────────────────────────────────────────────────────────────────

/// Summary returned by `GET /api/pages/{tenant_id}` (admin, includes unpublished).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PageSummary {
    pub id: Uuid,
    pub slug: String,
    pub title: String,
    pub page_type: String,
    pub is_published: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Full page model returned by slug GET / create / update.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppPage {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub slug: String,
    pub title: String,
    pub description: String,
    pub page_type: String,
    pub hero_payload: Option<serde_json::Value>,
    pub blocks_payload: Option<serde_json::Value>,
    pub is_published: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreatePagePayload {
    pub slug: String,
    pub title: String,
    pub description: String,
    pub page_type: Option<String>,
    pub hero_payload: Option<serde_json::Value>,
    pub blocks_payload: Option<serde_json::Value>,
    pub is_published: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdatePagePayload {
    pub title: Option<String>,
    pub description: Option<String>,
    pub page_type: Option<String>,
    pub hero_payload: Option<serde_json::Value>,
    pub blocks_payload: Option<serde_json::Value>,
    pub is_published: Option<bool>,
}

// ── API functions ─────────────────────────────────────────────────────────────

/// `GET /api/pages/{tenant_id}` — all pages including unpublished (admin).
pub async fn list_pages(tenant_id: Uuid) -> Result<Vec<PageSummary>, String> {
    let client = create_client();
    let url = api_url(&format!("/api/pages/{}", tenant_id));
    let req = with_credentials(client.get(&url));
    api_request::<Vec<PageSummary>>(req).await
}

/// `GET /api/pages/{tenant_id}/{slug}` — get page including unpublished (admin).
pub async fn get_page(tenant_id: Uuid, slug: &str) -> Result<AppPage, String> {
    let client = create_client();
    let url = api_url(&format!("/api/pages/{}/{}", tenant_id, slug));
    let req = with_credentials(client.get(&url));
    api_request::<AppPage>(req).await
}

/// `POST /api/pages/{tenant_id}` — create a new page.
pub async fn create_page(tenant_id: Uuid, payload: CreatePagePayload) -> Result<AppPage, String> {
    let client = create_client();
    let url = api_url(&format!("/api/pages/{}", tenant_id));
    let req = with_credentials(client.post(&url).json(&payload));
    api_request::<AppPage>(req).await
}

/// `PUT /api/pages/{tenant_id}/{slug}` — update a page.
pub async fn update_page(
    tenant_id: Uuid,
    slug: &str,
    payload: UpdatePagePayload,
) -> Result<AppPage, String> {
    let client = create_client();
    let url = api_url(&format!("/api/pages/{}/{}", tenant_id, slug));
    let req = with_credentials(client.put(&url).json(&payload));
    api_request::<AppPage>(req).await
}

/// `DELETE /api/pages/{tenant_id}/{slug}` — delete a page.
pub async fn delete_page(tenant_id: Uuid, slug: &str) -> Result<(), String> {
    let client = create_client();
    let url = api_url(&format!("/api/pages/{}/{}", tenant_id, slug));
    let req = with_credentials(client.delete(&url));
    let res = req.send().await.map_err(|e| e.to_string())?;
    if res.status().is_success() {
        Ok(())
    } else {
        Err(format!("Delete failed: HTTP {}", res.status()))
    }
}
