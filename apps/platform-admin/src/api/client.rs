use reqwest::{Client, RequestBuilder};
use serde::{Deserialize, Serialize};

// The backend runs on port 8000 by default
pub const API_BASE_URL: &str = "http://localhost:8000";

pub fn create_client() -> Client {
    Client::new()
}

/// Helper to attach credentials (cookies) for cross-origin requests
/// In leptos CSR, reqwest uses the web-sys fetch API.
pub fn with_credentials(builder: RequestBuilder) -> RequestBuilder {
    #[cfg(target_arch = "wasm32")]
    {
        builder.fetch_credentials_include()
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        builder
    }
}

pub fn api_url(path: &str) -> String {
    let path = path.trim_start_matches('/');
    format!("{}/{}", API_BASE_URL, path)
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
