use crate::api::client::{api_get, api_url, create_client, api_request};
use crate::api::models::VerificationRequestModel;
use uuid::Uuid;

pub async fn get_verification_requests(
    tenant_id: Option<Uuid>,
    status: Option<String>,
) -> Result<Vec<VerificationRequestModel>, String> {
    let mut url = "api/admin/verification-requests".to_string();
    let mut params = Vec::new();
    if let Some(t) = tenant_id {
        params.push(format!("tenant_id={}", t));
    }
    if let Some(s) = status {
        params.push(format!("status={}", s));
    }
    if !params.is_empty() {
        url = format!("{}?{}", url, params.join("&"));
    }
    api_get(&url).await
}

pub async fn approve_verification_request(id: Uuid) -> Result<VerificationRequestModel, String> {
    let client = create_client();
    let url = api_url(&format!("api/admin/verification-requests/{}/approve", id));
    let req = client.post(&url);
    api_request(req).await
}

pub async fn reject_verification_request(id: Uuid, reason: String) -> Result<VerificationRequestModel, String> {
    let client = create_client();
    let url = api_url(&format!("api/admin/verification-requests/{}/reject", id));
    let req = client.post(&url).json(&serde_json::json!({ "reason": reason }));
    api_request(req).await
}
