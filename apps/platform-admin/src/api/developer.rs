use super::client::{api_request, api_url, create_client, with_credentials};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// --- Models (aligned to backend developer_console + api_token entity) ---

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct ApiToken {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub scopes: serde_json::Value,
    pub expires_at: Option<String>,
    pub created_at: Option<String>,
    /// Not persisted on backend today — optional for forward compat.
    #[serde(default)]
    pub name: Option<String>,
}

impl ApiToken {
    pub fn display_label(&self) -> String {
        self.name
            .clone()
            .unwrap_or_else(|| format!("Token · {}", &self.id.to_string()[..8]))
    }

    pub fn scopes_display(&self) -> String {
        match &self.scopes {
            serde_json::Value::Array(arr) => arr
                .iter()
                .filter_map(|v| v.as_str())
                .collect::<Vec<_>>()
                .join(", "),
            other => other.to_string(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CreateApiTokenRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    pub scopes: serde_json::Value,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CreateApiTokenResponse {
    pub id: Uuid,
    /// Full secret — only returned at creation time (`token` from backend).
    #[serde(alias = "token")]
    pub secret: String,
    pub scopes: serde_json::Value,
    pub expires_at: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct WebhookEndpoint {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub target_url: String,
    pub secret_key: String,
    pub subscribed_events: serde_json::Value,
    pub is_active: bool,
    pub created_at: Option<String>,
}

impl WebhookEndpoint {
    pub fn events_display(&self) -> String {
        match &self.subscribed_events {
            serde_json::Value::Array(arr) => arr
                .iter()
                .filter_map(|v| v.as_str())
                .collect::<Vec<_>>()
                .join(", "),
            other => other.to_string(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CreateWebhookRequest {
    pub target_url: String,
    pub subscribed_events: serde_json::Value,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct WebhookDelivery {
    pub id: Uuid,
    pub endpoint_id: Uuid,
    pub tenant_id: Uuid,
    pub event_type: String,
    pub payload: serde_json::Value,
    pub status: String,
    pub next_retry_at: Option<String>,
    pub attempts: i32,
    pub response_status: Option<i32>,
    pub response_body: Option<String>,
    pub created_at: Option<String>,
}

// --- Methods ---

pub async fn list_api_tokens(tenant_id: Uuid) -> Result<Vec<ApiToken>, String> {
    let url = api_url(&format!(
        "/api/admin/developer/tenant/{}/api-tokens",
        tenant_id
    ));
    let client = create_client();
    let req = with_credentials(client.get(&url));
    api_request(req).await
}

pub async fn create_api_token(
    tenant_id: Uuid,
    request: CreateApiTokenRequest,
) -> Result<CreateApiTokenResponse, String> {
    let url = api_url(&format!(
        "/api/admin/developer/tenant/{}/api-tokens",
        tenant_id
    ));
    let client = create_client();
    let req = with_credentials(client.post(&url).json(&request));
    api_request(req).await
}

pub async fn revoke_api_token(tenant_id: Uuid, token_id: Uuid) -> Result<(), String> {
    let url = api_url(&format!(
        "/api/admin/developer/tenant/{}/api-tokens/{}",
        tenant_id, token_id
    ));
    let client = create_client();
    let req = with_credentials(client.delete(&url));
    let res = req.send().await.map_err(|e| e.to_string())?;
    if res.status().is_success() {
        Ok(())
    } else {
        Err(format!("Failed to revoke token: {}", res.status()))
    }
}

pub async fn list_webhook_endpoints(tenant_id: Uuid) -> Result<Vec<WebhookEndpoint>, String> {
    let url = api_url(&format!(
        "/api/admin/developer/tenant/{}/webhooks",
        tenant_id
    ));
    let client = create_client();
    let req = with_credentials(client.get(&url));
    api_request(req).await
}

pub async fn create_webhook_endpoint(
    tenant_id: Uuid,
    request: CreateWebhookRequest,
) -> Result<WebhookEndpoint, String> {
    let url = api_url(&format!(
        "/api/admin/developer/tenant/{}/webhooks",
        tenant_id
    ));
    let client = create_client();
    let req = with_credentials(client.post(&url).json(&request));
    api_request(req).await
}

pub async fn delete_webhook_endpoint(tenant_id: Uuid, endpoint_id: Uuid) -> Result<(), String> {
    let url = api_url(&format!(
        "/api/admin/developer/tenant/{}/webhooks/{}",
        tenant_id, endpoint_id
    ));
    let client = create_client();
    let req = with_credentials(client.delete(&url));
    let res = req.send().await.map_err(|e| e.to_string())?;
    if res.status().is_success() {
        Ok(())
    } else {
        Err(format!("Failed to delete webhook: {}", res.status()))
    }
}

pub async fn list_webhook_deliveries(tenant_id: Uuid) -> Result<Vec<WebhookDelivery>, String> {
    let url = api_url(&format!(
        "/api/admin/developer/tenant/{}/webhook-deliveries",
        tenant_id
    ));
    let client = create_client();
    let req = with_credentials(client.get(&url));
    api_request(req).await
}
