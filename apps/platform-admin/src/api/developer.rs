use serde::{Deserialize, Serialize};
use uuid::Uuid;
use super::client::{api_url, create_client, with_credentials, api_request};

// --- Models ---

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct ApiToken {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub token_hash: String,
    pub scopes: serde_json::Value,
    pub expires_at: Option<String>,
    pub created_at: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CreateApiTokenRequest {
    pub scopes: serde_json::Value,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CreateApiTokenResponse {
    pub id: Uuid,
    pub token: String,
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
    let url = api_url(&format!("/api/admin/developer/tenant/{}/api-tokens", tenant_id));
    let client = create_client();
    let req = client.get(&url);
    api_request(req).await
}

pub async fn create_api_token(tenant_id: Uuid, request: CreateApiTokenRequest) -> Result<CreateApiTokenResponse, String> {
    let url = api_url(&format!("/api/admin/developer/tenant/{}/api-tokens", tenant_id));
    let client = create_client();
    let req = client.post(&url).json(&request);
    api_request(req).await
}

pub async fn revoke_api_token(tenant_id: Uuid, token_id: Uuid) -> Result<(), String> {
    let url = api_url(&format!("/api/admin/developer/tenant/{}/api-tokens/{}", tenant_id, token_id));
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
    let url = api_url(&format!("/api/admin/developer/tenant/{}/webhooks", tenant_id));
    let client = create_client();
    let req = client.get(&url);
    api_request(req).await
}

pub async fn create_webhook_endpoint(tenant_id: Uuid, request: CreateWebhookRequest) -> Result<WebhookEndpoint, String> {
    let url = api_url(&format!("/api/admin/developer/tenant/{}/webhooks", tenant_id));
    let client = create_client();
    let req = client.post(&url).json(&request);
    api_request(req).await
}

pub async fn delete_webhook_endpoint(tenant_id: Uuid, endpoint_id: Uuid) -> Result<(), String> {
    let url = api_url(&format!("/api/admin/developer/tenant/{}/webhooks/{}", tenant_id, endpoint_id));
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
    let url = api_url(&format!("/api/admin/developer/tenant/{}/webhook-deliveries", tenant_id));
    let client = create_client();
    let req = client.get(&url);
    api_request(req).await
}
