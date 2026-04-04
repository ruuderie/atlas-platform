use reqwest::{Client, Error};
use serde::de::DeserializeOwned;
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
) -> Result<T, String> {
    // If tenant_id is not provided, we can maybe fallback, but for now we expect it.
    let tenant_str = tenant_id.map(|t| t.to_string()).unwrap_or_else(|| "".to_string());
    
    // Safety check if endpoint_path already handled the tenant_id placement (e.g. /api/public/pages/:tenant_id/home)
    let url = format!("{}{}", get_atlas_api_url(), endpoint_path);

    let res = CLIENT.get(&url).send().await.map_err(|e| e.to_string())?;

    if !res.status().is_success() {
        return Err(format!("Atlas API returned error: {}", res.status()));
    }

    let data = res.json::<T>().await.map_err(|e| e.to_string())?;
    Ok(data)
}
