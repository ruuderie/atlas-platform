use super::client::{api_url, create_client, with_credentials, api_request};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};

// ── Models ─────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppMenu {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub menu_type: String,
    pub label: String,
    pub href: Option<String>,
    pub parent_id: Option<Uuid>,
    pub display_order: i32,
    pub is_visible: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateMenuPayload {
    pub menu_type: String,
    pub label: String,
    pub href: Option<String>,
    pub parent_id: Option<Uuid>,
    pub display_order: Option<i32>,
    pub is_visible: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateMenuPayload {
    pub label: Option<String>,
    pub href: Option<String>,
    pub parent_id: Option<Uuid>,
    pub display_order: Option<i32>,
    pub is_visible: Option<bool>,
}

// ── API functions ─────────────────────────────────────────────────────────────

/// `GET /api/menus/{tenant_id}` — all menus including hidden items (admin).
pub async fn list_menus(tenant_id: Uuid) -> Result<Vec<AppMenu>, String> {
    let client = create_client();
    let url = api_url(&format!("/api/menus/{}", tenant_id));
    let req = with_credentials(client.get(&url));
    api_request::<Vec<AppMenu>>(req).await
}

/// `POST /api/menus/{tenant_id}` — create a new menu item.
pub async fn create_menu(tenant_id: Uuid, payload: CreateMenuPayload) -> Result<AppMenu, String> {
    let client = create_client();
    let url = api_url(&format!("/api/menus/{}", tenant_id));
    let req = with_credentials(client.post(&url).json(&payload));
    api_request::<AppMenu>(req).await
}

/// `PUT /api/menus/{tenant_id}/{menu_id}` — update a menu item.
pub async fn update_menu(
    tenant_id: Uuid,
    menu_id: Uuid,
    payload: UpdateMenuPayload,
) -> Result<AppMenu, String> {
    let client = create_client();
    let url = api_url(&format!("/api/menus/{}/{}", tenant_id, menu_id));
    let req = with_credentials(client.put(&url).json(&payload));
    api_request::<AppMenu>(req).await
}

/// `DELETE /api/menus/{tenant_id}/{menu_id}` — delete a menu item (cascades to children).
pub async fn delete_menu(tenant_id: Uuid, menu_id: Uuid) -> Result<(), String> {
    let client = create_client();
    let url = api_url(&format!("/api/menus/{}/{}", tenant_id, menu_id));
    let req = with_credentials(client.delete(&url));
    let res = req.send().await.map_err(|e| e.to_string())?;
    if res.status().is_success() {
        Ok(())
    } else {
        Err(format!("Delete failed: HTTP {}", res.status()))
    }
}
