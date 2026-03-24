use super::client::{api_url, create_client, with_credentials, ApiErrorResponse};
use super::models::SessionResponse;
use reqwest::StatusCode;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct SetupStatusResponse {
    pub needs_setup: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SetupInitializeRequest {
    pub email: String,
    pub password: String,
    pub first_name: String,
    pub last_name: String,
}

pub async fn get_setup_status() -> Result<SetupStatusResponse, String> {
    let client = create_client();
    let url = api_url("/setup/status");

    let req = client.get(&url);
    // Don't strictly need credentials for public check, but it's safe to include
    let req = with_credentials(req);

    let res = req.send().await.map_err(|e| e.to_string())?;

    if res.status() == StatusCode::OK {
        let status = res.json::<SetupStatusResponse>().await.map_err(|e| e.to_string())?;
        Ok(status)
    } else {
        Err("Failed to fetch setup status".into())
    }
}

pub async fn initialize_system(req: SetupInitializeRequest) -> Result<crate::api::models::SessionResponse, String> {
    let client = create_client();
    let res = client
        .post(api_url("/setup/initialize"))
        .json(&req)
        .send()
        .await
        .map_err(|e| e.to_string())?;

    if res.status().is_success() {
        let text = res.text().await.map_err(|e| e.to_string())?;
        serde_json::from_str(&text).map_err(|e| e.to_string())
    } else {
        let err: ApiErrorResponse = res.json().await.unwrap_or(ApiErrorResponse {
            message: Some("Failed to parse error response".into()),
            error: None,
        });
        Err(err.message.or(err.error).unwrap_or("Unknown error".into()))
    }
}

pub async fn purge_admin() -> Result<(), String> {
    let client = create_client();
    let res = client
        .post(api_url("/setup/purge_admin"))
        .send()
        .await
        .map_err(|e| e.to_string())?;

    if res.status().is_success() {
        Ok(())
    } else {
        let err: ApiErrorResponse = res.json().await.unwrap_or(ApiErrorResponse {
            message: Some("Failed to parse error response".into()),
            error: None,
        });
        Err(err.message.or(err.error).unwrap_or("Unknown error".into()))
    }
}
