use super::client::{api_url, create_client, with_credentials, ApiErrorResponse};
use super::models::{SessionResponse, UserInfo, UserLogin};
use reqwest::StatusCode;
use std::error::Error;

pub async fn login(credentials: UserLogin) -> Result<SessionResponse, String> {
    let client = create_client();
    let url = api_url("/login");

    let req = client.post(&url).json(&credentials);
    let req = with_credentials(req);

    let res = req.send().await.map_err(|e| e.to_string())?;

    if res.status() == StatusCode::OK {
        let session = res.json::<SessionResponse>().await.map_err(|e| e.to_string())?;
        crate::api::client::set_auth_token(&session.token);
        Ok(session)
    } else {
        let err: ApiErrorResponse = res.json().await.unwrap_or(ApiErrorResponse {
            message: Some("Failed to parse error response".into()),
            error: None,
        });
        Err(err.message.unwrap_or_else(|| err.error.unwrap_or_else(|| "Unknown login error".into())))
    }
}

pub async fn validate_session() -> Result<UserInfo, String> {
    let client = create_client();
    let url = api_url("/validate-session");

    let req = client.get(&url);
    let req = with_credentials(req);

    let res = req.send().await.map_err(|e| e.to_string())?;

    if res.status() == StatusCode::OK {
        // validate-session typically returns the user directly or session info
        // Let's assume it returns `UserInfo` for now based on common patterns
        let user = res.json::<UserInfo>().await.map_err(|e| e.to_string())?;
        Ok(user)
    } else {
        Err("Session invalid".into())
    }
}

pub async fn logout() -> Result<(), String> {
    let client = create_client();
    let url = api_url("/logout");

    let req = client.post(&url);
    let req = with_credentials(req);

    let res = req.send().await.map_err(|e| e.to_string())?;

    if res.status().is_success() {
        crate::api::client::clear_auth_token();
        Ok(())
    } else {
        Err("Failed to logout".into())
    }
}

pub async fn impersonate_user(user_id: &str) -> Result<SessionResponse, String> {
    let client = create_client();
    let url = api_url(&format!("/admin/users/{}/impersonate", user_id));

    let req = client.post(&url);
    let req = with_credentials(req);

    let res = req.send().await.map_err(|e| e.to_string())?;

    if res.status() == StatusCode::OK {
        let session = res.json::<SessionResponse>().await.map_err(|e| e.to_string())?;
        crate::api::client::set_auth_token(&session.token);
        Ok(session)
    } else {
        let err: ApiErrorResponse = res.json().await.unwrap_or(ApiErrorResponse {
            message: Some("Failed to parse error response".into()),
            error: None,
        });
        Err(err.message.unwrap_or_else(|| err.error.unwrap_or_else(|| "Failed to impersonate user".into())))
    }
}
