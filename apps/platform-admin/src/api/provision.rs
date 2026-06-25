use super::client::{api_url, create_client, with_credentials};
use serde::{Deserialize, Serialize};
use reqwest::StatusCode;
use uuid::Uuid;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ProvisionTenantPayload {
    pub tenant_name: String,
    pub display_name: String,
    pub domain: String,
    pub admin_email: String,
    pub admin_first_name: String,
    pub admin_last_name: String,
    pub apps: Option<Vec<String>>,
    pub bypass_dns_verification: Option<bool>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct ProvisionTenantResponse {
    pub tenant_id: Uuid,
    pub account_id: Uuid,
    pub domain: String,
    pub setup_url: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ProvisionAdminPayload {
    pub email: String,
    pub first_name: String,
    pub last_name: String,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct ProvisionAdminResponse {
    pub message: String,
    pub setup_token: String,
    pub setup_url: String,
}

/// Calls `POST /api/admin/tenants/provision` on the backend to provision
/// a tenant, its app instance, app domain, admin user, CMS pages, and WebAuthn registry entry.
pub async fn provision_tenant(payload: ProvisionTenantPayload) -> Result<ProvisionTenantResponse, String> {
    let client = create_client();
    let url = api_url("/api/admin/tenants/provision");
    let req = client.post(&url);
    let req = with_credentials(req);

    let res = req.json(&payload).send().await.map_err(|e| e.to_string())?;

    if res.status() == StatusCode::CREATED {
        res.json::<ProvisionTenantResponse>()
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

/// Calls `POST /api/tenants/{tenant_id}/provision-admin` to provision a tenant administrator user
/// and generate a passkey setup link.
pub async fn provision_admin(tenant_id: Uuid, payload: ProvisionAdminPayload) -> Result<ProvisionAdminResponse, String> {
    let client = create_client();
    let url = api_url(&format!("/api/tenants/{}/provision-admin", tenant_id));
    let req = client.post(&url);
    let req = with_credentials(req);

    let res = req.json(&payload).send().await.map_err(|e| e.to_string())?;

    if res.status() == StatusCode::OK {
        res.json::<ProvisionAdminResponse>()
            .await
            .map_err(|e| format!("Failed to parse provision admin response: {e}"))
    } else {
        let status = res.status();
        let body = res.text().await.unwrap_or_default();
        Err(format!(
            "Provision admin failed (HTTP {}): {}",
            status.as_u16(),
            body
        ))
    }
}
