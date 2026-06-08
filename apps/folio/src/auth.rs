use leptos::prelude::*;
use serde::{Deserialize, Serialize};

/// Validate the current session against the Atlas backend.
/// Returns the session payload on success, or an error string.
#[server(CheckSession, "/api")]
pub async fn check_session() -> Result<SessionInfo, ServerFnError> {
    use axum::http::HeaderMap;
    use leptos_axum::extract;

    let headers = extract::<HeaderMap>().await.unwrap_or_default();
    let token = extract_bearer_token(&headers)
        .ok_or_else(|| ServerFnError::new("No session token"))?;

    let info = crate::atlas_client::authenticated_get::<SessionInfo>(
        "/api/auth/session",
        &token,
        None,
    )
    .await
    .map_err(|e| ServerFnError::new(format!("Session check failed: {e}")))?;

    Ok(info)
}

/// Request a magic-link login email.
#[server(RequestMagicLink, "/api")]
pub async fn request_magic_link(email: String) -> Result<(), ServerFnError> {
    let payload = serde_json::json!({ "email": email });
    crate::atlas_client::authenticated_post::<_, serde_json::Value>(
        "/api/auth/magic-link/request",
        "",          // no session token needed for this endpoint
        None,
        &payload,
    )
    .await
    .map(|_| ())
    .map_err(|e| ServerFnError::new(e))
}

/// Verify a magic-link token and create a session.
/// The backend sets the `atlas_session` cookie in its response;
/// leptos_axum forwards Set-Cookie headers automatically.
#[server(VerifyMagicLink, "/api")]
pub async fn verify_magic_link(token: String) -> Result<SessionInfo, ServerFnError> {
    let payload = serde_json::json!({ "token": token });
    let info = crate::atlas_client::authenticated_post::<_, SessionInfo>(
        "/api/auth/magic-link/verify",
        "",
        None,
        &payload,
    )
    .await
    .map_err(|e| ServerFnError::new(e))?;
    Ok(info)
}

/// Revoke the current session (logout).
#[server(RevokeSession, "/api")]
pub async fn revoke_session() -> Result<(), ServerFnError> {
    use axum::http::HeaderMap;
    use leptos_axum::extract;

    let headers = extract::<HeaderMap>().await.unwrap_or_default();
    if let Some(token) = extract_bearer_token(&headers) {
        let _ = crate::atlas_client::authenticated_post::<_, serde_json::Value>(
            "/api/auth/logout",
            &token,
            None,
            &serde_json::Value::Null,
        )
        .await;
    }
    Ok(())
}

// ── Helpers ──────────────────────────────────────────────────────────────────

#[cfg(feature = "ssr")]
fn extract_bearer_token(headers: &axum::http::HeaderMap) -> Option<String> {
    headers
        .get(axum::http::header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .and_then(|s| s.strip_prefix("Bearer "))
        .map(|s| s.to_string())
        .or_else(|| {
            // Fallback: read from atlas_session cookie
            headers
                .get(axum::http::header::COOKIE)
                .and_then(|v| v.to_str().ok())
                .and_then(|cookies| {
                    cookies.split(';').find_map(|part| {
                        let part = part.trim();
                        part.strip_prefix("atlas_session=").map(|t| t.to_string())
                    })
                })
        })
}

// ── Shared session model (available on both SSR and WASM) ────────────────────

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SessionInfo {
    pub user_id: uuid::Uuid,
    pub tenant_id: Option<uuid::Uuid>,
    pub email: String,
    pub role: String,
}
