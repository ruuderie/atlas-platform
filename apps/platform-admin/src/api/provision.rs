use super::client::{api_url, create_client, with_credentials};
use serde::{Deserialize, Serialize};
use reqwest::StatusCode;
use uuid::Uuid;

/// Response from the backend provision endpoint.
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct ProvisionResponse {
    pub tenant_id: Uuid,
    pub success: bool,
    pub message: String,
}

/// Calls `POST /api/admin/platform/provision/{tenant_id}` on the backend.
///
/// Bootstraps a new tenant with default CMS scaffolding (home page, header menu).
/// Should be called immediately after creating a new app instance.
pub async fn provision_tenant(tenant_id: Uuid) -> Result<ProvisionResponse, String> {
    let client = create_client();
    let url = api_url(&format!("/api/admin/platform/provision/{}", tenant_id));
    let req = client.post(&url);
    let req = with_credentials(req);

    let res = req.send().await.map_err(|e| e.to_string())?;

    if res.status() == StatusCode::OK {
        res.json::<ProvisionResponse>()
            .await
            .map_err(|e| format!("Failed to parse provision response: {e}"))
    } else {
        let status = res.status();
        let body = res.text().await.unwrap_or_default();
        Err(format!(
            "Provision failed (HTTP {}): {}",
            status.as_u16(),
            body
        ))
    }
}
