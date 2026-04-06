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

pub fn get_auth_token() -> Option<String> {
    #[cfg(feature = "ssr")]
    {
        use axum::http::request::Parts;
        if let Some(req_parts) = use_context::<Parts>() {
            if let Some(cookie_header) = req_parts.headers.get("cookie") {
                if let Ok(cookie_str) = cookie_header.to_str() {
                    for part in cookie_str.split(';') {
                        let part_str = part.trim();
                        if part_str.starts_with("auth_token=") {
                            return Some(part_str["auth_token=".len()..].to_string());
                        }
                    }
                }
            }
        }
        None
    }
    #[cfg(not(feature = "ssr"))]
    {
        if let Some(window) = web_sys::window() {
            if let Some(document) = window.document() {
                use wasm_bindgen::JsCast;
                if let Ok(html_doc) = document.dyn_into::<web_sys::HtmlDocument>() {
                    let cookies = html_doc.cookie().unwrap_or_default();
                    for part in cookies.split(';') {
                        let part_str = part.trim();
                        if part_str.starts_with("auth_token=") {
                            return Some(part_str["auth_token=".len()..].to_string());
                        }
                    }
                }
            }
        }
        None
    }
}

pub fn set_auth_token(token: &str) {
    #[cfg(not(feature = "ssr"))]
    {
        if let Some(window) = web_sys::window() {
            if let Some(document) = window.document() {
                use wasm_bindgen::JsCast;
                if let Ok(html_doc) = document.dyn_into::<web_sys::HtmlDocument>() {
                    // Set cookie to expire far into future for now
                    let expires = "Fri, 31 Dec 9999 23:59:59 GMT"; 
                    let cookie_str = format!("auth_token={}; expires={}; path=/; SameSite=Lax", token, expires);
                    let _ = html_doc.set_cookie(&cookie_str);
                }
            }
        }
    }
}

pub fn clear_auth_token() {
    #[cfg(not(feature = "ssr"))]
    {
        if let Some(window) = web_sys::window() {
            if let Some(document) = window.document() {
                use wasm_bindgen::JsCast;
                if let Ok(html_doc) = document.dyn_into::<web_sys::HtmlDocument>() {
                    let _ = html_doc.set_cookie("auth_token=; expires=Thu, 01 Jan 1970 00:00:00 UTC; path=/;");
                }
            }
        }
    }
}

#[server]
pub async fn fetch_current_user(token: Option<String>) -> Result<Option<UserProfile>, ServerFnError> {
    let active_token = if let Some(t) = token {
        t
    } else {
        if let Some(t) = get_auth_token() {
            t
        } else {
            return Ok(None);
        }
    };

    let url = "http://127.0.0.1:8000/api/me";
    let client = reqwest::Client::new();
    let res = client.get(url)
        .header("Authorization", format!("Bearer {}", active_token))
        .send()
        .await?;
    
    if res.status().is_success() {
        Ok(Some(res.json::<UserProfile>().await?))
    } else {
        Ok(None)
    }
}

#[server]
pub async fn fetch_my_accounts(token: Option<String>) -> Result<Vec<AccountItem>, ServerFnError> {
    let active_token = if let Some(t) = token {
        t
    } else {
        if let Some(t) = get_auth_token() {
            t
        } else {
            return Ok(vec![]);
        }
    };

    let url = "http://127.0.0.1:8000/api/me/accounts";
    let client = reqwest::Client::new();
    let res = client.get(url)
        .header("Authorization", format!("Bearer {}", active_token))
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
    let token = get_auth_token();
    let token2 = token.clone();
    
    let user_resource = Resource::new(
        || (),
        move |_| {
            let t = token.clone();
            async move { fetch_current_user(t).await }
        }
    );

    let accounts_resource = Resource::new(
        || (),
        move |_| {
            let t = token2.clone();
            async move { fetch_my_accounts(t).await }
        }
    );

    let is_logged_in = Signal::derive(move || {
        match user_resource.get() {
            Some(Ok(Some(_))) => true,
            _ => false,
        }
    });

    let auth_context = AuthContext {
        user: user_resource,
        accounts: accounts_resource,
        is_logged_in,
    };

    provide_context(auth_context);

    children()
}
