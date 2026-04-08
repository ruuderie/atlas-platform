use reqwest::Client;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use super::client::{api_url, get_auth_token};

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct UserModel {
    pub id: Uuid,
    pub username: String,
    pub email: String,
    pub is_active: bool,
    pub is_admin: bool,
}

pub async fn get_users(network_id: Option<Uuid>) -> Result<Vec<UserModel>, String> {
    let client = Client::new();
    let url = if let Some(net_id) = network_id {
        format!("{}?network_id={}", api_url("api/admin/users"), net_id)
    } else {
        api_url("api/admin/users")
    };
    
    let token = get_auth_token().unwrap_or_default();
    
    let res = client.get(&url)
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await
        .map_err(|e| e.to_string())?;
        
    if res.status().is_success() {
        res.json::<Vec<UserModel>>().await.map_err(|e| e.to_string())
    } else {
        Err(res.text().await.unwrap_or_default())
    }
}

pub async fn toggle_admin(id: Uuid) -> Result<UserModel, String> {
    let client = Client::new();
    let url = api_url(&format!("api/admin/users/{}/toggle-admin", id));
    let token = get_auth_token().unwrap_or_default();
    
    let res = client.post(&url)
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await
        .map_err(|e| e.to_string())?;
        
    if res.status().is_success() {
        res.json::<UserModel>().await.map_err(|e| e.to_string())
    } else {
        Err(res.text().await.unwrap_or_default())
    }
}

pub async fn get_app_domains(instance_id: String) -> Result<Vec<String>, String> {
    let client = Client::new();
    let url = api_url(&format!("api/admin/platform/apps/{}/domains", instance_id));
    let token = get_auth_token().unwrap_or_default();
    
    let res = client.get(&url)
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await
        .map_err(|e| e.to_string())?;
        
    if res.status().is_success() {
        res.json::<Vec<String>>().await.map_err(|e| e.to_string())
    } else {
        Err(res.text().await.unwrap_or_default())
    }
}

pub async fn add_app_domain(instance_id: String, domain_name: String) -> Result<(), String> {
    let client = Client::new();
    let url = api_url(&format!("api/admin/platform/apps/{}/domains", instance_id));
    let token = get_auth_token().unwrap_or_default();
    
    let payload = serde_json::json!({
        "domain_name": domain_name
    });
    
    let res = client.post(&url)
        .header("Authorization", format!("Bearer {}", token))
        .json(&payload)
        .send()
        .await
        .map_err(|e| e.to_string())?;
        
    if res.status().is_success() {
        Ok(())
    } else {
        Err(res.text().await.unwrap_or_default())
    }
}

pub async fn remove_app_domain(instance_id: String, domain_name: String) -> Result<(), String> {
    let client = Client::new();
    let url = api_url(&format!("api/admin/platform/apps/{}/domains/{}", instance_id, domain_name));
    let token = get_auth_token().unwrap_or_default();
    
    let res = client.delete(&url)
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await
        .map_err(|e| e.to_string())?;
        
    if res.status().is_success() {
        Ok(())
    } else {
        Err(res.text().await.unwrap_or_default())
    }
}
