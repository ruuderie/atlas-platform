use reqwest::{Client, RequestBuilder, StatusCode};
use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicBool, Ordering};

/// Set to true when any API call receives a 401 Unauthorized.
/// The app shell polls this and redirects to /login.
pub static SESSION_EXPIRED: AtomicBool = AtomicBool::new(false);

pub fn mark_session_expired() {
    clear_auth_token();
    SESSION_EXPIRED.store(true, Ordering::SeqCst);
    // Immediately redirect — safest path in a WASM context.
    #[cfg(target_arch = "wasm32")]
    if let Some(window) = web_sys::window() {
        let _ = window.location().set_href("/login");
    }
}

pub fn create_client() -> Client {
    Client::new()
}

/// Helper to attach credentials (cookies) for cross-origin requests
/// In leptos CSR, reqwest uses the web-sys fetch API.
pub fn with_credentials(builder: RequestBuilder) -> RequestBuilder {
    #[cfg(target_arch = "wasm32")]
    {
        let builder = builder.fetch_credentials_include();
        if let Some(token) = get_auth_token() {
            builder.header("Authorization", format!("Bearer {}", token))
        } else {
            builder
        }
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        builder
    }
}

/// Counts consecutive 401 responses across API calls.
///
/// Threshold is intentionally set to 3 (not 2) to avoid false-positive logouts
/// when a page mounts multiple concurrent fetches and all simultaneously receive
/// a 401 (e.g. during a brief session cookie propagation gap). Three consecutive
/// 401s — even accounting for concurrent requests — reliably indicates the
/// session is genuinely expired.
///
/// The counter is reset to 0 on every SPA navigation via `reset_consecutive_401s()`
/// (called from `AuthenticatedLayout`), and also on any non-401 response.
static CONSECUTIVE_401S: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);

/// Reset the consecutive-401 counter. Call this on every SPA route change so
/// that a stale 401 from a previous page does not falsely contribute to the
/// logout threshold on the next page.
pub fn reset_consecutive_401s() {
    CONSECUTIVE_401S.store(0, Ordering::SeqCst);
}

pub async fn api_request<T: serde::de::DeserializeOwned>(req: RequestBuilder) -> Result<T, String> {
    let req = with_credentials(req);
    let res = req.send().await.map_err(|e| e.to_string())?;

    if res.status() == StatusCode::UNAUTHORIZED {
        let count = CONSECUTIVE_401S.fetch_add(1, Ordering::SeqCst) + 1;
        if count >= 3 {
            // Three consecutive 401s: the session is definitively gone.
            // Threshold of 3 (not 2) prevents false-positive logouts from
            // concurrent page-load fetches racing through simultaneous 401s.
            mark_session_expired();
        }
        return Err("Unauthorized — session may have expired.".into());
    }

    // Reset the counter on any non-401 response.
    CONSECUTIVE_401S.store(0, Ordering::SeqCst);

    if res.status().is_success() {
        res.json::<T>().await.map_err(|e| e.to_string())
    } else {
        let text = res.text().await.unwrap_or_else(|_| "API Error".into());
        if let Ok(err) = serde_json::from_str::<ApiErrorResponse>(&text) {
            Err(err.message.unwrap_or_else(|| err.error.unwrap_or_else(|| text.clone())))
        } else {
            Err(text)
        }
    }
}


static AUTH_TOKEN: std::sync::Mutex<Option<String>> = std::sync::Mutex::new(None);

/// In-memory token storage fallback with sessionStorage persistence to survive page refreshes.
/// Returns the token if set.
pub fn get_auth_token() -> Option<String> {
    #[cfg(target_arch = "wasm32")]
    {
        if let Some(window) = web_sys::window() {
            if let Ok(Some(session_storage)) = window.session_storage() {
                if let Ok(Some(token)) = session_storage.get_item("auth_token") {
                    return Some(token);
                }
            }
        }
    }
    AUTH_TOKEN.lock().ok().and_then(|guard| guard.clone())
}

/// Saves the token for the duration of the browser tab session.
pub fn set_auth_token(token: &str) {
    #[cfg(target_arch = "wasm32")]
    {
        if let Some(window) = web_sys::window() {
            if let Ok(Some(session_storage)) = window.session_storage() {
                let _ = session_storage.set_item("auth_token", token);
            }
        }
    }
    if let Ok(mut guard) = AUTH_TOKEN.lock() {
        *guard = Some(token.to_string());
    }
}

/// Clears the token.
pub fn clear_auth_token() {
    #[cfg(target_arch = "wasm32")]
    {
        if let Some(window) = web_sys::window() {
            if let Ok(Some(session_storage)) = window.session_storage() {
                let _ = session_storage.remove_item("auth_token");
            }
        }
    }
    if let Ok(mut guard) = AUTH_TOKEN.lock() {
        *guard = None;
    }
}

pub fn api_url(path: &str) -> String {
    #[allow(unused_mut)]
    let mut base_url = "http://api.localhost".to_string();
    
    #[cfg(target_arch = "wasm32")]
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
    
    let base_url = base_url.trim_end_matches('/');
    let path = path.trim_start_matches('/');
    format!("{}/{}", base_url, path)
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ApiErrorResponse {
    pub message: Option<String>,
    pub error: Option<String>,
}

pub async fn api_get<T: serde::de::DeserializeOwned>(path: &str) -> Result<T, String> {
    let client = create_client();
    let url = api_url(path);
    let req = client.get(&url);
    api_request(req).await
}

pub async fn api_post<B: Serialize, T: serde::de::DeserializeOwned>(
    path: &str,
    body: &B,
) -> Result<T, String> {
    let client = create_client();
    let url = api_url(path);
    let req = client.post(&url).json(body);
    api_request(req).await
}

pub async fn api_put<B: Serialize, T: serde::de::DeserializeOwned>(
    path: &str,
    body: &B,
) -> Result<T, String> {
    let client = create_client();
    let url = api_url(path);
    let req = client.put(&url).json(body);
    api_request(req).await
}

pub async fn api_delete(path: &str) -> Result<(), String> {
    let client = create_client();
    let url = api_url(path);
    let req = client.delete(&url);
    let req = with_credentials(req);
    let res = req.send().await.map_err(|e| e.to_string())?;
    if res.status() == StatusCode::UNAUTHORIZED {
        mark_session_expired();
        return Err("Session expired.".into());
    }
    if res.status().is_success() {
        Ok(())
    } else {
        Err(format!("DELETE {} failed: {}", path, res.status()))
    }
}

/// PATCH with a JSON body, ignoring an empty/204 response body.
pub async fn api_patch_empty<B: Serialize>(path: &str, body: &B) -> Result<(), String> {
    let client = create_client();
    let url = api_url(path);
    let req = client.patch(&url).json(body);
    let req = with_credentials(req);
    let res = req.send().await.map_err(|e| e.to_string())?;
    if res.status() == StatusCode::UNAUTHORIZED {
        mark_session_expired();
        return Err("Session expired.".into());
    }
    if res.status().is_success() {
        Ok(())
    } else {
        Err(format!("PATCH {} failed: {}", path, res.status()))
    }
}

/// GET a path and unwrap a JSON envelope.
///
/// The API returns `{ "<key>": <value> }` — this helper extracts the value
/// under `key` and deserializes it directly into `T`.
///
/// Example: `api_get_key("api/folio/campaigns", "campaigns")` parses
/// `{"campaigns": [...]}` into `Vec<CampaignModel>`.
pub async fn api_get_key<T: serde::de::DeserializeOwned>(path: &str, key: &str) -> Result<T, String> {
    let raw: serde_json::Value = api_get(path).await?;
    raw.get(key)
        .cloned()
        .ok_or_else(|| format!("response missing key \"{key}\""))
        .and_then(|v| serde_json::from_value(v).map_err(|e| e.to_string()))
}

/// POST/PATCH a request and unwrap a JSON envelope.
///
/// Mirrors `api_get_key` for mutating requests.
pub async fn api_request_key<T: serde::de::DeserializeOwned>(
    req: reqwest::RequestBuilder,
    key: &str,
) -> Result<T, String> {
    let raw: serde_json::Value = api_request(req).await?;
    raw.get(key)
        .cloned()
        .ok_or_else(|| format!("response missing key \"{key}\""))
        .and_then(|v| serde_json::from_value(v).map_err(|e| e.to_string()))
}
