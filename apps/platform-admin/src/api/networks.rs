use super::client::{api_url, create_client, with_credentials};
use super::models::PlatformAppModel;
use reqwest::StatusCode;

pub async fn get_networks() -> Result<Vec<PlatformAppModel>, String> {
    let client = create_client();
    let url = api_url("/api/admin/platform/apps");

    let req = client.get(&url);
    let req = with_credentials(req);

    if let Ok(res) = req.send().await {
        if res.status() == StatusCode::OK {
            if let Ok(dirs) = res.json::<Vec<PlatformAppModel>>().await { 
                return Ok(dirs); 
            }
        }
    }
    Err("Network Error: Backend unreachable".into())
}


// The old multi-step tenant provisioning flow (create_tenant_record, create_app_instance_record, and create_network)
// has been deprecated in favor of the atomic transaction-based provision_tenant API inside api/provision.rs.


pub async fn get_tenant_setting(tenant_id: &str, key: &str) -> Result<super::models::TenantSettingResponse, String> {
    let client = create_client();
    let url = api_url(&format!("/api/tenants/{}/settings?key={}", tenant_id, key));
    let req = client.get(&url);
    let req = with_credentials(req);

    let res = req.send().await.map_err(|e| e.to_string())?;
    if res.status() == StatusCode::OK {
        let setting = res.json::<super::models::TenantSettingResponse>().await.map_err(|e| e.to_string())?;
        Ok(setting)
    } else {
        Err("Failed to fetch tenant setting".into())
    }
}

pub async fn upsert_tenant_setting(tenant_id: &str, req_data: super::models::UpsertSettingRequest) -> Result<super::models::TenantSettingResponse, String> {
    let client = create_client();
    let url = api_url(&format!("/api/tenants/{}/settings", tenant_id));
    let req = client.post(&url).json(&req_data);
    let req = with_credentials(req);

    let res = req.send().await.map_err(|e| e.to_string())?;
    if res.status() == StatusCode::OK || res.status() == StatusCode::CREATED {
        let setting = res.json::<super::models::TenantSettingResponse>().await.map_err(|e| e.to_string())?;
        Ok(setting)
    } else {
        Err("Failed to upsert tenant setting".into())
    }
}

/// Grant syndication access to a network instance.
/// Calls `POST /api/admin/network/syndication/{instance_slug}`.
pub async fn grant_syndication(instance_slug: &str) -> Result<serde_json::Value, String> {
    use super::client::api_request;
    let client = create_client();
    let url = api_url(&format!("/api/admin/network/syndication/{}", instance_slug));
    let req = client.post(&url);
    let req = with_credentials(req);
    api_request(req).await
}

/// Revoke syndication access from a network instance.
/// Calls `DELETE /api/admin/network/syndication/{instance_slug}`.
pub async fn revoke_syndication(instance_slug: &str) -> Result<serde_json::Value, String> {
    use super::client::api_request;
    let client = create_client();
    let url = api_url(&format!("/api/admin/network/syndication/{}", instance_slug));
    let req = client.delete(&url);
    let req = with_credentials(req);
    api_request(req).await
}

