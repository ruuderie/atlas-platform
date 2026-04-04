use leptos::*;

// Replaced with platform backend auth proxies
#[server(RequestMagicLink, "/api")]
pub async fn request_magic_link(email: String) -> Result<String, ServerFnError> {
    // We will proxy to the centralized magic link route
    #[cfg(feature = "ssr")]
    {
        let payload = serde_json::json!({
            "email": email,
        });
        
        let url = format!("{}/api/auth/magic-link/request", crate::atlas_client::get_atlas_api_url());
        let client = reqwest::Client::new();
        let res = client.post(&url).json(&payload).send().await;

        match res {
            Ok(r) if r.status().is_success() => Ok("SUCCESS".to_string()),
            _ => Err(ServerFnError::ServerError("Failed to request magic link".into())),
        }
    }
    #[cfg(not(feature = "ssr"))]
    {
        Ok("SUCCESS".to_string())
    }
}

#[server(VerifyMagicLink, "/api")]
pub async fn verify_magic_link(token: String) -> Result<String, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        let payload = serde_json::json!({
            "token": token,
        });
        
        let url = format!("{}/api/auth/magic-link/verify", crate::atlas_client::get_atlas_api_url());
        let client = reqwest::Client::new();
        let res = client.post(&url).json(&payload).send().await;

        match res {
            Ok(r) if r.status().is_success() => {
                let data: serde_json::Value = r.json().await.unwrap_or_default();
                if let Some(session_token) = data.get("token").and_then(|v| v.as_str()) {
                    use leptos_axum::ResponseOptions;
                    let response = leptos::expect_context::<ResponseOptions>();
                    let header_val = format!(
                        "session={}; HttpOnly; Path=/; SameSite=Strict",
                        session_token
                    );
                    response.append_header(
                        axum::http::header::SET_COOKIE,
                        axum::http::HeaderValue::from_str(&header_val).unwrap(),
                    );
                    return Ok("SUCCESS".to_string());
                }
                Err(ServerFnError::ServerError("Invalid response format".into()))
            },
            _ => Err(ServerFnError::ServerError("Failed to verify magic link".into())),
        }
    }
    #[cfg(not(feature = "ssr"))]
    {
        Ok("SUCCESS".to_string())
    }
}

#[server(IsSystemInitialized, "/api")]
pub async fn is_system_initialized() -> Result<bool, ServerFnError> {
    // With centralization, initialization is handled externally.
    Ok(true)
}

#[server(CheckSession, "/api")]
pub async fn check_session() -> Result<bool, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        use axum::http::HeaderMap;
        use axum_extra::extract::cookie::CookieJar;
        use leptos_axum::extract;

        let headers = extract::<HeaderMap>().await.unwrap_or_default();
        let cookies = CookieJar::from_headers(&headers);
        let session_cookie = cookies.get("session");
        
        // Return true if cookie exists. We eventually validate its signature with the backend.
        Ok(session_cookie.is_some())
    }
    #[cfg(not(feature = "ssr"))]
    {
        Ok(false)
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
