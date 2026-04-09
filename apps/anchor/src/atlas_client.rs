use reqwest::Client;
use serde::{Serialize, de::DeserializeOwned};
use std::env;
use uuid::Uuid;
use once_cell::sync::Lazy;

#[cfg(feature = "ssr")]
static CLIENT: Lazy<Client> = Lazy::new(|| Client::new());

#[cfg(feature = "ssr")]
pub fn get_atlas_api_url() -> String {
    env::var("ATLAS_API_URL").unwrap_or_else(|_| "http://localhost:8000".to_string())
}

#[cfg(feature = "ssr")]
pub async fn fetch_atlas_data<T: DeserializeOwned>(
    endpoint_path: &str,
    tenant_id: Option<Uuid>,
    host: Option<String>,
) -> Result<T, String> {
    let _tenant_str = tenant_id.map(|t| t.to_string()).unwrap_or_else(|| "".to_string());
    
    let url = format!("{}{}", get_atlas_api_url(), endpoint_path);
    let mut req = CLIENT.get(&url);

    if let Some(h) = host {
        req = req.header("X-Forwarded-Host", h);
    }

    let res = req.send().await.map_err(|e| e.to_string())?;

    if !res.status().is_success() {
        return Err(format!("Atlas API returned error: {}", res.status()));
    }

    let data = res.json::<T>().await.map_err(|e| e.to_string())?;
    Ok(data)
}

#[cfg(feature = "ssr")]
pub async fn post_to_atlas<T: Serialize, R: DeserializeOwned>(
    endpoint_path: &str,
    tenant_id: Option<Uuid>,
    host: Option<String>,
    payload: &T,
) -> Result<R, String> {
    let _tenant_str = tenant_id.map(|t| t.to_string()).unwrap_or_else(|| "".to_string());
    
    let url = format!("{}{}", get_atlas_api_url(), endpoint_path);
    let mut req = CLIENT.post(&url);
    
    if let Some(h) = host {
        req = req.header("X-Forwarded-Host", h);
    }
    
    // Pass tenant_id as a common generic header if helpful or it's handled by payload/url.
    if let Some(t_id) = tenant_id {
        req = req.header("X-Tenant-ID", t_id.to_string());
    }

    let res = req.json(payload).send().await.map_err(|e| e.to_string())?;

    if !res.status().is_success() {
        return Err(format!("Atlas API error: {}", res.status()));
    }

    let data = res.json::<R>().await.map_err(|e| e.to_string())?;
    Ok(data)
}
