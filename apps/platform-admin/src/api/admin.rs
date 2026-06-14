use serde::{Deserialize, Serialize};
use uuid::Uuid;
use super::client::{api_url, create_client, with_credentials};

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct UserModel {
    pub id: Uuid,
    pub username: String,
    pub email: String,
    pub is_active: bool,
    pub is_admin: bool,
}

pub async fn get_users(network_id: Option<Uuid>) -> Result<Vec<UserModel>, String> {
    let client = create_client();
    let url = if let Some(net_id) = network_id {
        format!("{}?network_id={}", api_url("api/admin/users"), net_id)
    } else {
        api_url("api/admin/users")
    };
    
    let req = client.get(&url);
    let req = with_credentials(req);
    let res = req.send().await.map_err(|e| e.to_string())?;
        
    if res.status().is_success() {
        res.json::<Vec<UserModel>>().await.map_err(|e| e.to_string())
    } else {
        Err(res.text().await.unwrap_or_default())
    }
}

pub async fn toggle_admin(id: Uuid) -> Result<UserModel, String> {
    let client = create_client();
    let url = api_url(&format!("api/admin/users/{}/toggle-admin", id));
    
    let req = client.post(&url);
    let req = with_credentials(req);
    let res = req.send().await.map_err(|e| e.to_string())?;
        
    if res.status().is_success() {
        res.json::<UserModel>().await.map_err(|e| e.to_string())
    } else {
        Err(res.text().await.unwrap_or_default())
    }
}

pub async fn get_app_domains(instance_id: String) -> Result<Vec<String>, String> {
    let client = create_client();
    let url = api_url(&format!("api/admin/platform/apps/{}/domains", instance_id));
    
    let req = client.get(&url);
    let req = with_credentials(req);
    let res = req.send().await.map_err(|e| e.to_string())?;
        
    if res.status().is_success() {
        res.json::<Vec<String>>().await.map_err(|e| e.to_string())
    } else {
        Err(res.text().await.unwrap_or_default())
    }
}

pub async fn add_app_domain(instance_id: String, domain_name: String) -> Result<(), String> {
    let client = create_client();
    let url = api_url(&format!("api/admin/platform/apps/{}/domains", instance_id));
    
    let payload = serde_json::json!({
        "domain_name": domain_name
    });
    
    let req = client.post(&url).json(&payload);
    let req = with_credentials(req);
    let res = req.send().await.map_err(|e| e.to_string())?;
        
    if res.status().is_success() {
        Ok(())
    } else {
        Err(res.text().await.unwrap_or_default())
    }
}

pub async fn remove_app_domain(instance_id: String, domain_name: String) -> Result<(), String> {
    let client = create_client();
    let url = api_url(&format!("api/admin/platform/apps/{}/domains/{}", instance_id, domain_name));
    
    let req = client.delete(&url);
    let req = with_credentials(req);
    let res = req.send().await.map_err(|e| e.to_string())?;
        
    if res.status().is_success() {
        Ok(())
    } else {
        Err(res.text().await.unwrap_or_default())
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct PublicConfigResponse {
    pub instance_id: Uuid,
    pub tenant_id: Uuid,
    pub app_slug: String,
    pub public_slug: Option<String>,
    pub custom_domain: Option<String>,
    pub instance_status: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct AdminModuleConfig {
    pub module_type: String,
    pub display_name: String,
    pub icon: Option<String>,
    pub sort_order: i32,
    pub is_fixed: bool,
    pub category: String,
}

pub async fn get_public_config(id: Uuid) -> Result<PublicConfigResponse, String> {
    crate::api::client::api_get(&format!("api/admin/app-instances/{}/public-config", id)).await
}

pub async fn update_public_config(id: Uuid, public_slug: Option<String>, custom_domain: Option<String>) -> Result<PublicConfigResponse, String> {
    let client = crate::api::client::create_client();
    let url = crate::api::client::api_url(&format!("api/admin/app-instances/{}/public-config", id));
    let payload = serde_json::json!({
        "public_slug": public_slug,
        "custom_domain": custom_domain
    });
    let req = client.put(&url).json(&payload);
    crate::api::client::api_request(req).await
}

pub async fn suspend_instance(id: Uuid, reason: String) -> Result<(), String> {
    let client = crate::api::client::create_client();
    let url = crate::api::client::api_url(&format!("api/admin/app-instances/{}/suspend", id));
    let payload = serde_json::json!({ "reason": reason });
    let req = client.post(&url).json(&payload);
    let req = crate::api::client::with_credentials(req);
    let res = req.send().await.map_err(|e| e.to_string())?;
    if res.status().is_success() { Ok(()) } else { Err(res.text().await.unwrap_or_default()) }
}

pub async fn resume_instance(id: Uuid) -> Result<(), String> {
    let client = crate::api::client::create_client();
    let url = crate::api::client::api_url(&format!("api/admin/app-instances/{}/resume", id));
    let req = client.post(&url);
    let req = crate::api::client::with_credentials(req);
    let res = req.send().await.map_err(|e| e.to_string())?;
    if res.status().is_success() { Ok(()) } else { Err(res.text().await.unwrap_or_default()) }
}

pub async fn upsert_module(tenant_id: Uuid, module_type: &str, is_enabled: bool, display_name: Option<String>, sort_order: Option<i32>) -> Result<(), String> {
    let client = crate::api::client::create_client();
    let url = crate::api::client::api_url(&format!("api/platform/tenants/{}/modules", tenant_id));
    let payload = serde_json::json!({
        "module_type": module_type,
        "is_enabled": is_enabled,
        "display_name": display_name,
        "sort_order": sort_order
    });
    let req = client.post(&url).json(&payload);
    let req = crate::api::client::with_credentials(req);
    let res = req.send().await.map_err(|e| e.to_string())?;
    if res.status().is_success() { Ok(()) } else { Err(res.text().await.unwrap_or_default()) }
}

pub async fn get_admin_modules() -> Result<Vec<AdminModuleConfig>, String> {
    crate::api::client::api_get("api/admin/modules").await
}

pub async fn impersonate_user(user_id: Uuid) -> Result<(), String> {
    let client = crate::api::client::create_client();
    let url = crate::api::client::api_url(&format!("api/admin/users/{}/impersonate", user_id));
    let req = client.post(&url);
    let req = crate::api::client::with_credentials(req);
    let res = req.send().await.map_err(|e| e.to_string())?;
    if res.status().is_success() { Ok(()) } else { Err(res.text().await.unwrap_or_default()) }
}
