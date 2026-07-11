use leptos::prelude::*;

pub use shared_ui::auth::atlas_auth::{
    check_session, request_magic_link, revoke_session, verify_magic_link,
};

#[server(IsSystemInitialized, "/api")]
pub async fn is_system_initialized() -> Result<bool, ServerFnError> {
    // With centralization, initialization is handled externally.
    Ok(true)
}

#[server(ExchangeSetupToken, "/api")]
pub async fn exchange_setup_token(token: String) -> Result<String, ServerFnError> {
    // Leaving this in anchor for now, as it might be specific to anchor setup
    #[cfg(feature = "ssr")]
    {
        let payload = serde_json::json!({
            "token": token,
        });

        let url = format!(
            "{}/api/auth/magic-link/verify",
            crate::atlas_client::get_atlas_api_url()
        );
        let client = reqwest::Client::new();
        let res = client.post(&url).json(&payload).send().await;

        match res {
            Ok(r) if r.status().is_success() => {
                let data: serde_json::Value = r.json().await.unwrap_or_default();
                if let Some(session_token) = data.get("token").and_then(|v| v.as_str()) {
                    use axum::http::HeaderMap;
                    use leptos_axum::extract;
                    use leptos_axum::ResponseOptions;
                    let headers = extract::<HeaderMap>().await.unwrap_or_default();
                    let host = headers
                        .get("host")
                        .and_then(|v| v.to_str().ok())
                        .unwrap_or("localhost");
                    let is_https = !host.starts_with("localhost") && !host.starts_with("127.");
                    let secure = if is_https { "; Secure" } else { "" };
                    let header_val = format!(
                        "session={}; HttpOnly; Path=/; SameSite=Strict{}",
                        session_token, secure
                    );
                    let response = expect_context::<ResponseOptions>();
                    response.append_header(
                        axum::http::header::SET_COOKIE,
                        axum::http::HeaderValue::from_str(&header_val).unwrap(),
                    );
                    return Ok("SUCCESS".to_string());
                }
                Err(ServerFnError::ServerError("Invalid response format".into()))
            }
            _ => Err(ServerFnError::ServerError(
                "Failed to exchange setup token".into(),
            )),
        }
    }
    #[cfg(not(feature = "ssr"))]
    {
        Ok("SUCCESS".to_string())
    }
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq)]
pub struct UserRecord {
    pub id: i32,
    pub username: String,
    pub created_at: String,
}

#[server(GetUsers, "/api")]
pub async fn get_users() -> Result<Vec<UserRecord>, ServerFnError> {
    Ok(vec![]) // Deprecated local access
}

#[server(DeleteUser, "/api")]
pub async fn delete_user(_id: i32) -> Result<(), ServerFnError> {
    Ok(()) // Deprecated local access
}
