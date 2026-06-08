use once_cell::sync::Lazy;
use reqwest::Client;
use serde::{Serialize, de::DeserializeOwned};

static CLIENT: Lazy<Client> = Lazy::new(Client::new);

pub fn get_atlas_api_url() -> String {
    std::env::var("ATLAS_API_URL").unwrap_or_else(|_| "http://localhost:8000".to_string())
}

/// Unauthenticated GET — for public endpoints (health, etc.)
pub async fn fetch<T: DeserializeOwned>(path: &str) -> Result<T, String> {
    let url = format!("{}{}", get_atlas_api_url(), path);
    let res = CLIENT.get(&url).send().await.map_err(|e| e.to_string())?;
    if !res.status().is_success() {
        return Err(format!("API {}", res.status()));
    }
    res.json::<T>().await.map_err(|e| e.to_string())
}

/// Authenticated GET — forwards session cookie and optional tenant-id header.
pub async fn authenticated_get<T: DeserializeOwned>(
    path: &str,
    session_token: &str,
    tenant_id: Option<uuid::Uuid>,
) -> Result<T, String> {
    let url = format!("{}{}", get_atlas_api_url(), path);
    let mut req = CLIENT
        .get(&url)
        .header("Authorization", format!("Bearer {}", session_token));
    if let Some(tid) = tenant_id {
        req = req.header("x-tenant-id", tid.to_string());
    }
    let res = req.send().await.map_err(|e| e.to_string())?;
    if !res.status().is_success() {
        return Err(format!("API {}", res.status()));
    }
    res.json::<T>().await.map_err(|e| e.to_string())
}

/// Authenticated POST — forwards session + tenant, serializes body as JSON.
pub async fn authenticated_post<B: Serialize, T: DeserializeOwned>(
    path: &str,
    session_token: &str,
    tenant_id: Option<uuid::Uuid>,
    body: &B,
) -> Result<T, String> {
    let url = format!("{}{}", get_atlas_api_url(), path);
    let mut req = CLIENT
        .post(&url)
        .header("Authorization", format!("Bearer {}", session_token))
        .json(body);
    if let Some(tid) = tenant_id {
        req = req.header("x-tenant-id", tid.to_string());
    }
    let res = req.send().await.map_err(|e| e.to_string())?;
    if !res.status().is_success() {
        return Err(format!("API {}", res.status()));
    }
    res.json::<T>().await.map_err(|e| e.to_string())
}

/// Authenticated DELETE — for resource removal.
pub async fn authenticated_delete(
    path: &str,
    session_token: &str,
    tenant_id: Option<uuid::Uuid>,
) -> Result<(), String> {
    let url = format!("{}{}", get_atlas_api_url(), path);
    let mut req = CLIENT
        .delete(&url)
        .header("Authorization", format!("Bearer {}", session_token));
    if let Some(tid) = tenant_id {
        req = req.header("x-tenant-id", tid.to_string());
    }
    let res = req.send().await.map_err(|e| e.to_string())?;
    if !res.status().is_success() {
        return Err(format!("API {}", res.status()));
    }
    Ok(())
}
