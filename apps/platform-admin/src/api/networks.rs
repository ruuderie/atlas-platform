use super::client::{api_url, create_client, with_credentials, api_request};
use super::models::{PlatformAppModel, CreateNetwork, TenantCreatedModel, CreateAppInstance};
use super::provision::provision_tenant;
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

/// Creates a new tenant record via `POST /api/tenants`.
async fn create_tenant_record(name: &str, domain: &str) -> Result<TenantCreatedModel, String> {
    let client = create_client();
    let url = api_url("/api/tenants");
    let body = serde_json::json!({
        "name": name,
        "description": format!("Tenant for {}", domain)
    });
    let req = client.post(&url).json(&body);
    api_request::<TenantCreatedModel>(req).await
}

/// Creates an `app_instance` record via `POST /api/app-instances`.
async fn create_app_instance_record(data: CreateAppInstance) -> Result<(), String> {
    let client = create_client();
    let url = api_url("/api/app-instances");
    let req = client.post(&url).json(&data);
    let req = with_credentials(req);
    let res = req.send().await.map_err(|e| e.to_string())?;
    if res.status().is_success() {
        Ok(())
    } else {
        Err(format!("Failed to create app_instance: HTTP {}", res.status()))
    }
}

/// Full tenant provisioning flow:
///   1. POST /api/tenants            → get tenant_id
///   2. POST /api/app-instances      → register app type for this tenant
///   3. POST /api/admin/platform/provision/{tenant_id}  → seed default pages + menus
pub async fn create_network(data: CreateNetwork) -> Result<PlatformAppModel, String> {
    // Step 1 — create tenant record
    let tenant = create_tenant_record(&data.name, &data.domain)
        .await
        .map_err(|e| format!("Step 1 (create tenant) failed: {e}"))?;

    // Step 2 — register app_instance
    let app_instance = CreateAppInstance {
        tenant_id: tenant.id,
        app_type: data.network_type_id.clone(),
        database_url: None,
        data_seed_name: None,
        settings: Some(serde_json::json!({ "domain": data.domain })),
    };
    create_app_instance_record(app_instance)
        .await
        .map_err(|e| format!("Step 2 (create app_instance) failed: {e}"))?;

    // Step 3 — provision (seeds pages, menus, etc.)
    provision_tenant(tenant.id)
        .await
        .map_err(|e| format!("Step 3 (provision) failed: {e}"))?;

    // Return a synthetic PlatformAppModel from the data we have
    Ok(PlatformAppModel {
        tenant_id: tenant.id.to_string(),
        instance_id: tenant.id.to_string(), // actual instance_id fetched from app_instances on next reload
        name: tenant.name,
        app_type: data.network_type_id,
        domain: data.domain,
        site_status: "active".to_string(),
        description: data.description,
    })
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
