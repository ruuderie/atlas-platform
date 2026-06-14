use super::client::{api_url, create_client, with_credentials, ApiErrorResponse};
use super::models::{SessionResponse, UserInfo, UserLogin};
use reqwest::StatusCode;

pub async fn login(credentials: UserLogin) -> Result<SessionResponse, String> {
    let client = create_client();
    let url = api_url("/login");

    let req = client.post(&url).json(&credentials);
    let req = with_credentials(req);

    let res = req.send().await.map_err(|e| e.to_string())?;

    if res.status() == StatusCode::OK {
        // Session cookie is set by the backend as HttpOnly — no client-side storage needed.
        let session = res.json::<SessionResponse>().await.map_err(|e| e.to_string())?;
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
    // Use the unified Atlas Auth Protocol endpoint
    let url = api_url("/api/auth/session/validate");

    let req = client.get(&url);
    let req = with_credentials(req);

    let res = req.send().await.map_err(|e| e.to_string())?;

    if res.status() == StatusCode::OK {
        let session = res.json::<SessionResponse>().await.map_err(|e| e.to_string())?;
        if let Some(user) = session.user {
            Ok(user)
        } else {
            Err("User payload missing".into())
        }
    } else {
        Err("Session invalid".into())
    }
}

pub async fn logout() -> Result<(), String> {
    let client = create_client();
    // Call the unified revoke endpoint — backend deactivates session and clears cookie.
    let url = api_url("/api/auth/session/revoke");

    let req = client.post(&url);
    let req = with_credentials(req);

    let _ = req.send().await; // Best-effort. Always clear local state regardless.
    Ok(())
}

pub async fn impersonate_user(user_id: &str) -> Result<SessionResponse, String> {
    let client = create_client();
    let url = api_url(&format!("/admin/users/{}/impersonate", user_id));

    let req = client.post(&url);
    let req = with_credentials(req);

    let res = req.send().await.map_err(|e| e.to_string())?;

    if res.status() == StatusCode::OK {
        let session = res.json::<SessionResponse>().await.map_err(|e| e.to_string())?;
        Ok(session)
    } else {
        let err: ApiErrorResponse = res.json().await.unwrap_or(ApiErrorResponse {
            message: Some("Failed to parse error response".into()),
            error: None,
        });
        Err(err.message.unwrap_or_else(|| err.error.unwrap_or_else(|| "Failed to impersonate user".into())))
    }
}
