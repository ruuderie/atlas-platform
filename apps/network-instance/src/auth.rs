use leptos::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct UserProfile {
    pub id: String,
    pub email: String,
    pub first_name: String,
    pub last_name: String,
    pub username: String,
    pub is_active: bool,
    pub is_admin: bool,
    pub app_permissions: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct AccountModel {
    pub id: String,
    pub network_id: String,
    pub name: String,
    pub is_active: bool,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct AccountItem {
    pub account: AccountModel,
    pub role: String,
}

#[derive(Clone, Debug)]
pub struct AuthContext {
    pub user: Resource<Result<Option<UserProfile>, ServerFnError>>,
    pub accounts: Resource<Result<Vec<AccountItem>, ServerFnError>>,
    pub is_logged_in: Signal<bool>,
}

/// Returns the backend API base URL from the ATLAS_API_URL environment variable.
/// Falls back to http://127.0.0.1:8000 in development only.
/// NOTE: ATLAS_API_URL must be set in production — the hardcoded fallback will not
/// resolve in a containerised environment.
pub fn api_base_url() -> String {
    #[cfg(feature = "ssr")]
    {
        std::env::var("ATLAS_API_URL")
            .unwrap_or_else(|_| "http://127.0.0.1:8000".to_string())
            .trim_end_matches('/')
            .to_string()
    }
    #[cfg(not(feature = "ssr"))]
    {
        // Client-side: read from window.__ENV__.API_BASE_URL injected at build time
        if let Some(window) = web_sys::window() {
            if let Ok(env_val) = js_sys::Reflect::get(&window, &wasm_bindgen::JsValue::from_str("__ENV__")) {
                if !env_val.is_undefined() {
                    if let Ok(api_val) = js_sys::Reflect::get(&env_val, &wasm_bindgen::JsValue::from_str("API_BASE_URL")) {
                        if let Some(s) = api_val.as_string() {
                            if s != "__API_BASE_URL__" && !s.is_empty() {
                                return s.trim_end_matches('/').to_string();
                            }
                        }
                    }
                }
            }
        }
        "http://127.0.0.1:8000".to_string()
    }
}

/// Session auth is now fully cookie-based.
/// The backend sets an HttpOnly session cookie on login — JS cannot read it.
/// This function is a no-op retained to ease migration. Remove call sites.
#[deprecated(note = "Auth is cookie-based. Backend sets HttpOnly cookie. Remove this call.")]
pub fn set_auth_token(_token: &str) {}

/// Session revocation is handled by calling POST /api/auth/session/revoke.
/// This function is a no-op retained to ease migration. Remove call sites.
#[deprecated(note = "Auth is cookie-based. Call POST /api/auth/session/revoke to clear session.")]
pub fn clear_auth_token() {}

#[server]
pub async fn fetch_current_user(
    _token: Option<String>, // Deprecated param — kept for API compat, ignored
) -> Result<Option<UserProfile>, ServerFnError> {
    use axum::http::request::Parts;

    // Read the session cookie from the incoming SSR request
    let session_cookie = if let Some(req_parts) = use_context::<Parts>() {
        req_parts
            .headers
            .get("cookie")
            .and_then(|v| v.to_str().ok())
            .and_then(|cookies| {
                cookies.split(';').find_map(|part| {
                    let part = part.trim();
                    part.strip_prefix("session=").map(|t| t.to_string())
                })
            })
    } else {
        None
    };

    let Some(token) = session_cookie else {
        return Ok(None);
    };

    let url = format!("{}/api/auth/session/validate", api_base_url());
    let client = reqwest::Client::new();
    let res = client
        .get(&url)
        .header("Cookie", format!("session={}", token))
        .send()
        .await?;

    if res.status().is_success() {
        // validate returns SessionResponse; extract the user field
        let body: serde_json::Value = res.json().await?;
        if let Some(user) = body.get("user") {
            let profile: UserProfile = serde_json::from_value(user.clone())?;
            return Ok(Some(profile));
        }
    }
    Ok(None)
}

#[server]
pub async fn fetch_my_accounts(
    _token: Option<String>, // Deprecated param — kept for API compat, ignored
) -> Result<Vec<AccountItem>, ServerFnError> {
    use axum::http::request::Parts;

    let session_cookie = if let Some(req_parts) = use_context::<Parts>() {
        req_parts
            .headers
            .get("cookie")
            .and_then(|v| v.to_str().ok())
            .and_then(|cookies| {
                cookies.split(';').find_map(|part| {
                    let part = part.trim();
                    part.strip_prefix("session=").map(|t| t.to_string())
                })
            })
    } else {
        None
    };

    let Some(token) = session_cookie else {
        return Ok(vec![]);
    };

    let url = format!("{}/api/me/accounts", api_base_url());
    let client = reqwest::Client::new();
    let res = client
        .get(&url)
        .header("Cookie", format!("session={}", token))
        .send()
        .await?;

    if res.status().is_success() {
        Ok(res.json::<Vec<AccountItem>>().await?)
    } else {
        Ok(vec![])
    }
}

#[component]
pub fn AuthProvider(children: Children) -> impl IntoView {
    // Token param is None — auth is fully cookie-based now.
    // The server functions read the session cookie from the request context.
    let user_resource = Resource::new(
        || (),
        move |_| async move { fetch_current_user(None).await },
    );

    let accounts_resource = Resource::new(
        || (),
        move |_| async move { fetch_my_accounts(None).await },
    );

    let is_logged_in = Signal::derive(move || {
        matches!(user_resource.get(), Some(Ok(Some(_))))
    });

    let auth_context = AuthContext {
        user: user_resource,
        accounts: accounts_resource,
        is_logged_in,
    };

    provide_context(auth_context);

    children()
}
