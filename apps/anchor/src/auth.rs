use leptos::prelude::*;

// Replaced with platform backend auth proxies
#[server(RequestMagicLink, "/api")]
pub async fn request_magic_link(email: String) -> Result<String, ServerFnError> {
    // Proxy to the Atlas backend magic link route.
    // Passes redirect_url derived from the current request's Host header so
    // the email link points back to this app's /admin page — not the Atlas
    // platform admin. The backend validates the host against app_domains.
    #[cfg(feature = "ssr")]
    {
        use axum::http::HeaderMap;
        use leptos_axum::extract;

        // Build redirect_url from the SSR request host (scheme inferred: https in prod, http for localhost)
        let headers = extract::<HeaderMap>().await.unwrap_or_default();
        let host = headers
            .get("host")
            .and_then(|v| v.to_str().ok())
            .unwrap_or("localhost");
        let scheme = if host.starts_with("localhost") || host.starts_with("127.") {
            "http"
        } else {
            "https"
        };
        let redirect_url = format!("{}://{}/admin", scheme, host);

        let payload = serde_json::json!({
            "email": email,
            "redirect_url": redirect_url,
        });

        let url = format!("{}/api/auth/magic-link/request", crate::atlas_client::get_atlas_api_url());
        let client = reqwest::Client::new();
        let res = client.post(&url).json(&payload).send().await;

        match res {
            Ok(r) if r.status().is_success() => Ok("SUCCESS".to_string()),
            Ok(r) => {
                tracing::warn!("Magic link request failed: HTTP {}", r.status());
                Err(ServerFnError::ServerError("Failed to request magic link".into()))
            }
            Err(e) => {
                tracing::error!("Magic link request error: {:?}", e);
                Err(ServerFnError::ServerError("Failed to request magic link".into()))
            }
        }
    }
    #[cfg(not(feature = "ssr"))]
    {
        Ok("SUCCESS".to_string())
    }
}

#[server(VerifyMagicLink, "/api")]
pub async fn verify_magic_link(token: String) -> Result<String, ServerFnError> {
    // Route: /magic-links/verify (registered in magic_links::public_routes(), no /api/auth prefix)
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

        // Extract session cookie and forward to Atlas for validation.
        let headers = extract::<HeaderMap>().await.unwrap_or_default();
        let cookie_header = headers.get("cookie").and_then(|v| v.to_str().ok()).unwrap_or("");

        let url = format!("{}/api/auth/session/validate", crate::atlas_client::get_atlas_api_url());
        let client = reqwest::Client::new();
        let res = client.get(&url).header("cookie", cookie_header).send().await;

        match res {
            Ok(r) if r.status().is_success() => Ok(true),
            Ok(_) => Ok(false),
            Err(_) => Ok(false), // Fail open to unauthenticated on network error
        }
    }
    #[cfg(not(feature = "ssr"))]
    {
        Ok(false)
    }
}

#[server(ExchangeSetupToken, "/api")]
pub async fn exchange_setup_token(token: String) -> Result<String, ServerFnError> {
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
            _ => Err(ServerFnError::ServerError("Failed to exchange setup token".into())),
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
