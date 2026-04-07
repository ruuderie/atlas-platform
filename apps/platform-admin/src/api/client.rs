use reqwest::{Client, RequestBuilder};
use serde::{Deserialize, Serialize};


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

pub async fn api_request<T: serde::de::DeserializeOwned>(req: RequestBuilder) -> Result<T, String> {
    let req = with_credentials(req);
    let res = req.send().await.map_err(|e| e.to_string())?;
    
    if res.status().is_success() {
        res.json::<T>().await.map_err(|e| e.to_string())
    } else {
        let err: ApiErrorResponse = res.json().await.unwrap_or(ApiErrorResponse {
            message: Some("Failed to parse API error response".into()),
            error: None,
        });
        Err(err.message.unwrap_or_else(|| err.error.unwrap_or_else(|| "API Error".into())))
    }
}

pub fn get_auth_token() -> Option<String> {
    #[cfg(target_arch = "wasm32")]
    {
        if let Some(window) = web_sys::window() {
            if let Ok(Some(storage)) = window.local_storage() {
                return storage.get_item("auth_token").unwrap_or(None);
            }
        }
    }
    None
}

pub fn set_auth_token(token: &str) {
    #[cfg(target_arch = "wasm32")]
    {
        if let Some(window) = web_sys::window() {
            if let Ok(Some(storage)) = window.local_storage() {
                let _ = storage.set_item("auth_token", token);
            }
        }
    }
}

pub fn clear_auth_token() {
    #[cfg(target_arch = "wasm32")]
    {
        if let Some(window) = web_sys::window() {
            if let Ok(Some(storage)) = window.local_storage() {
                let _ = storage.remove_item("auth_token");
            }
        }
    }
}

pub fn api_url(path: &str) -> String {
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
