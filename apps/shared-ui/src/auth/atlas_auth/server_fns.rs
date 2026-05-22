use leptos::prelude::*;

#[cfg(feature = "ssr")]
pub fn get_atlas_api_url() -> String {
    std::env::var("ATLAS_API_URL").unwrap_or_else(|_| "http://localhost:3000".to_string())
}

#[cfg(not(feature = "ssr"))]
pub fn get_atlas_api_url() -> String {
    let mut base_url = "http://api.localhost".to_string();
    if let Some(window) = web_sys::window() {
        if let Ok(env_val) = js_sys::Reflect::get(&window, &wasm_bindgen::JsValue::from_str("__ENV__")) {
            if !env_val.is_undefined() {
                if let Ok(api_val) = js_sys::Reflect::get(&env_val, &wasm_bindgen::JsValue::from_str("API_BASE_URL")) {
                    if let Some(s) = api_val.as_string() {
                        if s != "__API_BASE_URL__" && !s.is_empty() {
                            base_url = s;
                        }
                    }
                }
            }
        }
    }
    base_url.trim_end_matches('/').to_string()
}

#[cfg(any(feature = "ssr", feature = "hydrate"))]
#[server(RequestMagicLink, "/api")]
pub async fn request_magic_link(email: String) -> Result<String, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        use axum::http::HeaderMap;
        use leptos_axum::extract;

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
        let secure_flag = if scheme == "https" { "; Secure" } else { "" };

        let payload = serde_json::json!({
            "email": email,
            "redirect_url": redirect_url,
        });

        let url = format!("{}/api/auth/magic-link/request", get_atlas_api_url());
        let client = reqwest::Client::new();
        let res = client.post(&url).json(&payload).send().await;

        match res {
            Ok(r) if r.status().is_success() => Ok("SUCCESS".to_string()),
            Ok(r) => {
                leptos::logging::warn!("Magic link request failed: HTTP {}", r.status());
                Err(ServerFnError::ServerError("Failed to request magic link".into()))
            }
            Err(e) => {
                leptos::logging::error!("Magic link request error: {:?}", e);
                Err(ServerFnError::ServerError("Failed to request magic link".into()))
            }
        }
    }
    #[cfg(not(feature = "ssr"))]
    {
        Ok("SUCCESS".to_string())
    }
}

#[cfg(not(any(feature = "ssr", feature = "hydrate")))]
pub async fn request_magic_link(email: String) -> Result<String, ServerFnError> {
    let url = format!("{}/api/auth/magic-link/request", get_atlas_api_url());
    let payload = serde_json::json!({ "email": email });
    #[allow(unused_mut)]
    let mut req = reqwest::Client::new().post(&url).json(&payload);
    #[cfg(target_arch = "wasm32")]
    {
        req = req.fetch_credentials_include();
    }
    let res = req.send().await;
    match res {
        Ok(r) if r.status().is_success() => Ok("SUCCESS".to_string()),
        _ => Err(ServerFnError::ServerError("Failed to request magic link".into()))
    }
}

#[cfg(any(feature = "ssr", feature = "hydrate"))]
#[server(VerifyMagicLink, "/api")]
pub async fn verify_magic_link(token: String) -> Result<String, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        use axum::http::HeaderMap;
        use leptos_axum::extract;
        let headers = extract::<HeaderMap>().await.unwrap_or_default();
        let host = headers
            .get("host")
            .and_then(|v| v.to_str().ok())
            .unwrap_or("localhost");
        let is_https = !host.starts_with("localhost") && !host.starts_with("127.");
        let _secure_flag = if is_https { "; Secure" } else { "" };

        let payload = serde_json::json!({ "token": token });
        let url = format!("{}/api/auth/magic-link/verify", get_atlas_api_url());
        let client = reqwest::Client::new();
        let res = client.post(&url).json(&payload).send().await;

        match res {
            Ok(r) if r.status().is_success() => {
                // Proxy the Set-Cookie from the backend to the browser.
                let cookie = r.headers()
                    .get("set-cookie")
                    .and_then(|v| v.to_str().ok())
                    .map(|s| s.to_string());
                if let Some(cookie_val) = cookie {
                    use leptos_axum::ResponseOptions;
                    let response = expect_context::<ResponseOptions>();
                    response.append_header(
                        axum::http::header::SET_COOKIE,
                        axum::http::HeaderValue::from_str(&cookie_val).unwrap(),
                    );
                    return Ok("SUCCESS".to_string());
                }
                Err(ServerFnError::ServerError("error_code:server_error".into()))
            },
            Ok(r) => {
                // Surface the structured error_code so the frontend TokenFailure enum
                // can branch on Expired vs AlreadyUsed vs NotFound.
                let body = r.text().await.unwrap_or_default();
                let code = if body.contains("token_already_used") {
                    "error_code:token_already_used"
                } else if body.contains("token_expired") {
                    "error_code:token_expired"
                } else {
                    "error_code:token_not_found"
                };
                Err(ServerFnError::ServerError(code.into()))
            },
            Err(e) => {
                leptos::logging::error!("Magic link verify error: {:?}", e);
                Err(ServerFnError::ServerError("error_code:server_error".into()))
            },
        }
    }
    #[cfg(not(feature = "ssr"))]
    {
        Ok("SUCCESS".to_string())
    }
}

#[cfg(not(any(feature = "ssr", feature = "hydrate")))]
pub async fn verify_magic_link(token: String) -> Result<String, ServerFnError> {
    let url = format!("{}/api/auth/magic-link/verify", get_atlas_api_url());
    let payload = serde_json::json!({ "token": token });
    #[allow(unused_mut)]
    let mut req = reqwest::Client::new().post(&url).json(&payload);
    #[cfg(target_arch = "wasm32")]
    {
        req = req.fetch_credentials_include();
    }
    let res = req.send().await;
    match res {
        Ok(r) if r.status().is_success() => Ok("SUCCESS".to_string()),
        _ => Err(ServerFnError::ServerError("Failed to verify magic link".into()))
    }
}

#[server(CheckHasPasskey, "/api")]
pub async fn check_has_passkey() -> Result<bool, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        use axum::http::HeaderMap;
        use leptos_axum::extract;

        let headers = extract::<HeaderMap>().await.unwrap_or_default();
        let cookie_header = headers.get("cookie").and_then(|v| v.to_str().ok()).unwrap_or("");

        let url = format!("{}/api/passkeys/has-passkey", get_atlas_api_url());
        let client = reqwest::Client::new();
        match client.get(&url).header("cookie", cookie_header).send().await {
            Ok(r) if r.status().is_success() => {
                let val: serde_json::Value = r.json().await.unwrap_or_default();
                Ok(val["has_passkey"].as_bool().unwrap_or(true))
            }
            // If the call fails (unauthenticated or network error), assume has passkey
            // so we don't nag users we can't verify.
            _ => Ok(true),
        }
    }
    #[cfg(not(feature = "ssr"))]
    {
        Ok(true)
    }
}

#[cfg(any(feature = "ssr", feature = "hydrate"))]
#[server(CheckSession, "/api")]
pub async fn check_session() -> Result<bool, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        use axum::http::HeaderMap;
        use leptos_axum::extract;

        let headers = extract::<HeaderMap>().await.unwrap_or_default();
        let cookie_header = headers.get("cookie").and_then(|v| v.to_str().ok()).unwrap_or("");

        let url = format!("{}/api/auth/session/validate", get_atlas_api_url());
        let client = reqwest::Client::new();
        let res = client.get(&url).header("cookie", cookie_header).send().await;

        match res {
            Ok(r) if r.status().is_success() => Ok(true),
            Ok(_) => Ok(false),
            Err(_) => Ok(false),
        }
    }
    #[cfg(not(feature = "ssr"))]
    {
        Ok(false)
    }
}

#[cfg(not(any(feature = "ssr", feature = "hydrate")))]
pub async fn check_session() -> Result<bool, ServerFnError> {
    let url = format!("{}/api/auth/session/validate", get_atlas_api_url());
    #[allow(unused_mut)]
    let mut req = reqwest::Client::new().get(&url);
    #[cfg(target_arch = "wasm32")]
    {
        req = req.fetch_credentials_include();
    }
    let res = req.send().await;
    match res {
        Ok(r) if r.status().is_success() => Ok(true),
        _ => Ok(false)
    }
}

#[cfg(any(feature = "ssr", feature = "hydrate"))]
#[server(RevokeSession, "/api")]
pub async fn revoke_session() -> Result<(), ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        use leptos_axum::ResponseOptions;
        let response = expect_context::<ResponseOptions>();
        let header_val = "session=; HttpOnly; Path=/; SameSite=Strict; Max-Age=0";
        response.append_header(
            axum::http::header::SET_COOKIE,
            axum::http::HeaderValue::from_str(header_val).unwrap(),
        );
        Ok(())
    }
    #[cfg(not(feature = "ssr"))]
    {
        Ok(())
    }
}

#[cfg(not(any(feature = "ssr", feature = "hydrate")))]
pub async fn revoke_session() -> Result<(), ServerFnError> {
    let url = format!("{}/api/auth/session/revoke", get_atlas_api_url());
    #[allow(unused_mut)]
    let mut req = reqwest::Client::new().post(&url);
    #[cfg(target_arch = "wasm32")]
    {
        req = req.fetch_credentials_include();
    }
    let _ = req.send().await;
    Ok(())
}

#[cfg(any(feature = "ssr", feature = "hydrate"))]
#[server(StartPasskeyRegistration, "/api")]
pub async fn start_passkey_registration() -> Result<serde_json::Value, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        use axum::http::HeaderMap;
        use leptos_axum::extract;

        let headers = extract::<HeaderMap>().await.unwrap_or_default();
        let cookie_header = headers.get("cookie").and_then(|v| v.to_str().ok()).unwrap_or("");
        
        let origin = if let Some(origin_val) = headers.get("origin").and_then(|v| v.to_str().ok()) {
            origin_val.to_string()
        } else {
            let host = headers
                .get("host")
                .and_then(|v| v.to_str().ok())
                .unwrap_or("localhost");
            let scheme = if host.starts_with("localhost") || host.starts_with("127.") {
                "http"
            } else {
                "https"
            };
            format!("{}://{}", scheme, host)
        };

        let url = format!("{}/api/passkeys/start-register", get_atlas_api_url());
        let client = reqwest::Client::new();
        let res = client.post(&url)
            .header("cookie", cookie_header)
            .header("origin", &origin)
            .send()
            .await;

        match res {
            Ok(r) if r.status().is_success() => {
                match r.json::<serde_json::Value>().await {
                    Ok(val) => Ok(val),
                    Err(e) => Err(ServerFnError::ServerError(format!("Invalid JSON from backend: {:?}", e))),
                }
            }
            Ok(r) => {
                let status = r.status();
                let err_text = r.text().await.unwrap_or_default();
                leptos::logging::warn!("Start passkey registration failed: HTTP {} - {}", status, err_text);
                Err(ServerFnError::ServerError(err_text))
            }
            Err(e) => {
                leptos::logging::error!("Start passkey registration error: {:?}", e);
                Err(ServerFnError::ServerError("Failed to start passkey registration".into()))
            }
        }
    }
    #[cfg(not(feature = "ssr"))]
    {
        Err(ServerFnError::ServerError("Client-side stub".into()))
    }
}

#[cfg(not(any(feature = "ssr", feature = "hydrate")))]
pub async fn start_passkey_registration() -> Result<serde_json::Value, ServerFnError> {
    Err(ServerFnError::ServerError("Server functions not available in CSR-only build".into()))
}

#[cfg(any(feature = "ssr", feature = "hydrate"))]
#[server(FinishPasskeyRegistration, "/api")]
pub async fn finish_passkey_registration(credential: serde_json::Value) -> Result<serde_json::Value, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        use axum::http::HeaderMap;
        use leptos_axum::extract;

        let headers = extract::<HeaderMap>().await.unwrap_or_default();
        let cookie_header = headers.get("cookie").and_then(|v| v.to_str().ok()).unwrap_or("");
        
        let origin = if let Some(origin_val) = headers.get("origin").and_then(|v| v.to_str().ok()) {
            origin_val.to_string()
        } else {
            let host = headers
                .get("host")
                .and_then(|v| v.to_str().ok())
                .unwrap_or("localhost");
            let scheme = if host.starts_with("localhost") || host.starts_with("127.") {
                "http"
            } else {
                "https"
            };
            format!("{}://{}", scheme, host)
        };

        let url = format!("{}/api/passkeys/finish-register", get_atlas_api_url());
        let client = reqwest::Client::new();
        let res = client.post(&url)
            .header("cookie", cookie_header)
            .header("origin", &origin)
            .json(&credential)
            .send()
            .await;

        match res {
            Ok(r) if r.status().is_success() => {
                match r.json::<serde_json::Value>().await {
                    Ok(val) => Ok(val),
                    Err(e) => Err(ServerFnError::ServerError(format!("Invalid JSON from backend: {:?}", e))),
                }
            }
            Ok(r) => {
                let status = r.status();
                let err_text = r.text().await.unwrap_or_default();
                leptos::logging::warn!("Finish passkey registration failed: HTTP {} - {}", status, err_text);
                Err(ServerFnError::ServerError(err_text))
            }
            Err(e) => {
                leptos::logging::error!("Finish passkey registration error: {:?}", e);
                Err(ServerFnError::ServerError("Failed to finish passkey registration".into()))
            }
        }
    }
    #[cfg(not(feature = "ssr"))]
    {
        Err(ServerFnError::ServerError("Client-side stub".into()))
    }
}

#[cfg(not(any(feature = "ssr", feature = "hydrate")))]
pub async fn finish_passkey_registration(_credential: serde_json::Value) -> Result<serde_json::Value, ServerFnError> {
    Err(ServerFnError::ServerError("Server functions not available in CSR-only build".into()))
}

