use super::client::{ApiErrorResponse, api_url, create_client, with_credentials};
use super::models::{SessionResponse, UserInfo, UserLogin};
use reqwest::StatusCode;
use std::cell::RefCell;

// ── Client-side session cache ─────────────────────────────────────────────────
//
// Platform-admin is pure WASM (no SSR), so only Layer 1 caching is needed.
// TTL: 15 seconds — matches Folio's client-side cache policy.
// See docs/leptos_architecture_decisions.md §5.6 for the full rationale.

thread_local! {
    static CACHED_SESSION: RefCell<Option<(f64, UserInfo)>> = const { RefCell::new(None) };
}

const SESSION_TTL_MS: f64 = 15_000.0;

fn cache_get() -> Option<UserInfo> {
    CACHED_SESSION.with(|c| {
        if let Some((ts, ref info)) = *c.borrow() {
            if js_sys::Date::now() - ts < SESSION_TTL_MS {
                return Some(info.clone());
            }
        }
        None
    })
}

fn cache_set(info: &UserInfo) {
    CACHED_SESSION.with(|c| *c.borrow_mut() = Some((js_sys::Date::now(), info.clone())));
}

pub fn cache_clear() {
    CACHED_SESSION.with(|c| *c.borrow_mut() = None);
}

/// Session-cached wrapper. Use this in components instead of validate_session().
/// Returns cached UserInfo if within TTL, otherwise fetches and re-caches.
/// Call cache_clear() on logout to invalidate immediately.
pub async fn get_session() -> Result<UserInfo, String> {
    if let Some(cached) = cache_get() {
        return Ok(cached);
    }
    let result = validate_session().await;
    if let Ok(ref info) = result {
        cache_set(info);
    } else {
        cache_clear();
    }
    result
}

pub async fn login(credentials: UserLogin) -> Result<SessionResponse, String> {
    let client = create_client();
    let url = api_url("/login");

    let req = client.post(&url).json(&credentials);
    let req = with_credentials(req);

    let res = req.send().await.map_err(|e| e.to_string())?;

    if res.status() == StatusCode::OK {
        // Session cookie is set by the backend as HttpOnly — no client-side storage needed.
        let session = res
            .json::<SessionResponse>()
            .await
            .map_err(|e| e.to_string())?;
        Ok(session)
    } else {
        let err: ApiErrorResponse = res.json().await.unwrap_or(ApiErrorResponse {
            message: Some("Failed to parse error response".into()),
            error: None,
        });
        Err(err
            .message
            .unwrap_or_else(|| err.error.unwrap_or_else(|| "Unknown login error".into())))
    }
}

pub async fn validate_session() -> Result<UserInfo, String> {
    let client = create_client();
    // Use the unified Atlas Auth Protocol endpoint
    let url = api_url("/api/auth/session/validate");

    let req = client.get(&url);
    let req = with_credentials(req);

    let res = req.send().await.map_err(|e| e.to_string())?;

    if res.status() == StatusCode::OK {
        let session = res
            .json::<SessionResponse>()
            .await
            .map_err(|e| e.to_string())?;
        if let Some(user) = session.user {
            Ok(user)
        } else {
            Err("User payload missing".into())
        }
    } else {
        let text = res
            .text()
            .await
            .unwrap_or_else(|_| "Session invalid".into());
        Err(text)
    }
}

pub async fn logout() -> Result<(), String> {
    let client = create_client();
    // Call the unified revoke endpoint — backend deactivates session and clears cookie.
    let url = api_url("/api/auth/session/revoke");

    let req = client.post(&url);
    let req = with_credentials(req);

    let _ = req.send().await; // Best-effort. Always clear local state regardless.
    super::client::clear_auth_token();
    Ok(())
}

pub async fn impersonate_user(user_id: &str) -> Result<SessionResponse, String> {
    let client = create_client();
    let url = api_url(&format!("/admin/users/{}/impersonate", user_id));

    let req = client.post(&url);
    let req = with_credentials(req);

    let res = req.send().await.map_err(|e| e.to_string())?;

    if res.status() == StatusCode::OK {
        let session = res
            .json::<SessionResponse>()
            .await
            .map_err(|e| e.to_string())?;
        Ok(session)
    } else {
        let err: ApiErrorResponse = res.json().await.unwrap_or(ApiErrorResponse {
            message: Some("Failed to parse error response".into()),
            error: None,
        });
        Err(err.message.unwrap_or_else(|| {
            err.error
                .unwrap_or_else(|| "Failed to impersonate user".into())
        }))
    }
}
