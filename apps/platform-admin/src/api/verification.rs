use crate::api::client::{api_get, api_request, api_url, create_client};
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

pub async fn reject_verification_request(
    id: Uuid,
    reason: String,
) -> Result<VerificationRequestModel, String> {
    let client = create_client();
    let url = api_url(&format!("api/admin/verification-requests/{}/reject", id));
    let req = client
        .post(&url)
        .json(&serde_json::json!({ "reason": reason }));
    api_request(req).await
}

pub async fn add_verification_notes(
    id: Uuid,
    notes: String,
) -> Result<VerificationRequestModel, String> {
    let client = create_client();
    let url = api_url(&format!("api/admin/verification-requests/{}/notes", id));
    let req = client
        .post(&url)
        .json(&serde_json::json!({ "notes": notes }));
    api_request(req).await
}

pub async fn request_verification_info(
    id: Uuid,
    message: Option<String>,
) -> Result<VerificationRequestModel, String> {
    let client = create_client();
    let url = api_url(&format!(
        "api/admin/verification-requests/{}/request-info",
        id
    ));
    let req = client
        .post(&url)
        .json(&serde_json::json!({ "message": message }));
    api_request(req).await
}
