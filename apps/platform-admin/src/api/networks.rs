use super::client::{api_url, create_client, with_credentials, ApiErrorResponse};
use super::models::{PlatformAppModel, CreateNetwork};
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

pub async fn create_network(data: CreateNetwork) -> Result<PlatformAppModel, String> {
    // Note: Creating a unified app requires hitting /api/app-instances and /api/tenants usually.
    // For now we map it to fallback since a pure POST to /api/admin/networks doesn't exist anymore anyway.
    Err("Unified App creation not fully implemented backend-side.".into())
}

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
