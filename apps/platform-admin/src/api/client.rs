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
    
    let path = path.trim_start_matches('/');
    format!("{}/{}", base_url, path)
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ApiErrorResponse {
    pub message: Option<String>,
    pub error: Option<String>,
}

pub fn is_demo_mode() -> bool {
    #[cfg(target_arch = "wasm32")]
    {
        if let Some(window) = web_sys::window() {
            if let Ok(Some(storage)) = window.local_storage() {
                return storage.get_item("demo_mode").unwrap_or(None).unwrap_or("false".into()) == "true";
            }
        }
    }
    false
}

pub fn set_demo_mode(enabled: bool) {
    #[cfg(target_arch = "wasm32")]
    {
        if let Some(window) = web_sys::window() {
            if let Ok(Some(storage)) = window.local_storage() {
                let val = if enabled { "true" } else { "false" };
                let _ = storage.set_item("demo_mode", val);
            }
        }
    }
}
