use once_cell::sync::Lazy;
use reqwest::Client;
use serde::{de::DeserializeOwned, Serialize};
use std::time::Duration;

/// Process-wide Atlas HTTP client (SSR).
///
/// Tuned for Docker Desktop's flaky first-connect behavior: short idle timeout
/// so we don't reuse dead keep-alives, and an explicit connect timeout.
/// Does not change ATLAS_API_URL / Service DNS — only how we dial it.
static CLIENT: Lazy<Client> = Lazy::new(|| {
    Client::builder()
        .timeout(Duration::from_secs(30))
        .connect_timeout(Duration::from_secs(5))
        .pool_idle_timeout(Duration::from_secs(5))
        .tcp_nodelay(true)
        .build()
        .unwrap_or_else(|_| Client::new())
});

/// Shared reqwest client for Folio → Atlas calls (verify, server fns, warmup).
pub fn http_client() -> &'static Client {
    &CLIENT
}

pub fn get_atlas_api_url() -> String {
    std::env::var("ATLAS_API_URL").unwrap_or_else(|_| "http://localhost:8000".to_string())
}

/// Forward the browser client IP when Folio SSR proxies to Atlas.
///
/// Without this, every magic-link/OTP request from Folio shares one pod IP (or
/// `unknown`), so `check_auth_rate_limit` (5 req / IP / 10 min) trips for all
/// users after a few attempts and surfaces as a opaque 500 in the browser.
#[cfg(feature = "ssr")]
pub fn forward_client_ip(incoming: &axum::http::HeaderMap) -> reqwest::header::HeaderMap {
    let mut out = reqwest::header::HeaderMap::new();
    let client_ip = incoming
        .get("x-forwarded-for")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.split(',').next().unwrap_or(s).trim())
        .filter(|s| !s.is_empty())
        .or_else(|| {
            incoming
                .get("x-real-ip")
                .and_then(|v| v.to_str().ok())
                .map(str::trim)
                .filter(|s| !s.is_empty())
        });
    if let Some(ip) = client_ip {
        if let Ok(val) = reqwest::header::HeaderValue::from_str(ip) {
            out.insert("x-forwarded-for", val);
        }
    }
    out
}

/// Headers Folio SSR should forward when proxying authenticated calls to Atlas.
///
/// Includes client IP plus `X-Forwarded-Host` so Atlas can resolve the Folio
/// app instance / tenant from `app_domains` (needed to provision landlord
/// workspaces for fresh magic-link users who have no `user_account` yet).
#[cfg(feature = "ssr")]
pub fn folio_proxy_headers(incoming: &axum::http::HeaderMap) -> reqwest::header::HeaderMap {
    let mut out = forward_client_ip(incoming);

    let host = incoming
        .get("x-forwarded-host")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.split(',').next().unwrap_or(s).trim().to_string())
        .filter(|s| !s.is_empty())
        .or_else(|| {
            incoming
                .get(axum::http::header::HOST)
                .and_then(|v| v.to_str().ok())
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
        })
        .or_else(|| {
            incoming
                .get(axum::http::header::ORIGIN)
                .and_then(|v| v.to_str().ok())
                .and_then(|o| url::Url::parse(o).ok())
                .and_then(|u| u.host_str().map(|h| h.to_string()))
        });

    if let Some(h) = host {
        // Strip port if present (folio1.atlas.oply.co:443 → folio1.atlas.oply.co)
        let host_only = h.split(':').next().unwrap_or(&h);
        if let Ok(val) = reqwest::header::HeaderValue::from_str(host_only) {
            out.insert("x-forwarded-host", val);
        }
    }
    out
}

/// Unauthenticated GET — for public endpoints (health, etc.)
pub async fn fetch<T: DeserializeOwned>(path: &str) -> Result<T, String> {
    let url = format!("{}{}", get_atlas_api_url(), path);
    let res = CLIENT.get(&url).send().await.map_err(|e| e.to_string())?;
    if !res.status().is_success() {
        return Err(format!("API {}", res.status()));
    }
    res.json::<T>().await.map_err(|e| e.to_string())
}

/// Unauthenticated POST — for public token-gated endpoints (PMC onboard, etc.)
pub async fn post<B: Serialize, T: DeserializeOwned>(path: &str, body: &B) -> Result<T, String> {
    post_with_headers(path, body, reqwest::header::HeaderMap::new()).await
}

/// Like [`post`], but forwards selected request headers (e.g. client IP).
pub async fn post_with_headers<B: Serialize, T: DeserializeOwned>(
    path: &str,
    body: &B,
    headers: reqwest::header::HeaderMap,
) -> Result<T, String> {
    let url = format!("{}{}", get_atlas_api_url(), path);
    let res = CLIENT
        .post(&url)
        .headers(headers)
        .json(body)
        .send()
        .await
        .map_err(|e| e.to_string())?;
    if !res.status().is_success() {
        let status = res.status();
        let msg = res.text().await.unwrap_or_default();
        return Err(format!("API {status}: {msg}"));
    }
    res.json::<T>().await.map_err(|e| e.to_string())
}

/// Unauthenticated POST that also extracts the bearer token from the `Set-Cookie`
/// response header. Used by the magic-link verify flow: the backend marks the token
/// used, creates a session, and returns it as `Set-Cookie: session=TOKEN; …`.
/// The `SessionResponse.token` field has `#[serde(skip_serializing)]` so it is NOT
/// in the JSON body — we MUST read it from the response headers.
///
/// Returns `(parsed_body, Some(bearer_token))` on success.
/// `bearer_token` is `None` if the backend didn't set a session cookie
/// (shouldn't happen on a 200, but we surface the error clearly).
pub async fn post_returning_session<B: Serialize, T: DeserializeOwned>(
    path: &str,
    body: &B,
) -> Result<(T, Option<String>), String> {
    post_returning_session_with_headers(path, body, reqwest::header::HeaderMap::new()).await
}

pub async fn post_returning_session_with_headers<B: Serialize, T: DeserializeOwned>(
    path: &str,
    body: &B,
    headers: reqwest::header::HeaderMap,
) -> Result<(T, Option<String>), String> {
    let url = format!("{}{}", get_atlas_api_url(), path);
    let res = CLIENT
        .post(&url)
        .headers(headers)
        .json(body)
        .send()
        .await
        .map_err(|e| e.to_string())?;
    if !res.status().is_success() {
        let status = res.status();
        let msg = res.text().await.unwrap_or_default();
        return Err(format!("API {status}: {msg}"));
    }
    // Extract the session token from `Set-Cookie: session=TOKEN; …`
    // The backend's session_cookie_header() always uses the name "session".
    let session_token = res
        .headers()
        .get_all(reqwest::header::SET_COOKIE)
        .iter()
        .find_map(|v| {
            let s = v.to_str().ok()?;
            // Handle both `session=TOKEN` and `session=TOKEN; HttpOnly; …`
            s.split(';')
                .next()
                .and_then(|kv| kv.trim().strip_prefix("session="))
                .map(|t| t.to_string())
        });
    let body_parsed = res.json::<T>().await.map_err(|e| e.to_string())?;
    Ok((body_parsed, session_token))
}

/// Authenticated GET — forwards session cookie and optional tenant-id header.
pub async fn authenticated_get<T: DeserializeOwned>(
    path: &str,
    session_token: &str,
    tenant_id: Option<uuid::Uuid>,
) -> Result<T, String> {
    authenticated_get_with_headers(path, session_token, tenant_id, reqwest::header::HeaderMap::new())
        .await
}

/// Like [`authenticated_get`], with extra proxy headers (e.g. `X-Forwarded-Host`).
pub async fn authenticated_get_with_headers<T: DeserializeOwned>(
    path: &str,
    session_token: &str,
    tenant_id: Option<uuid::Uuid>,
    extra: reqwest::header::HeaderMap,
) -> Result<T, String> {
    let url = format!("{}{}", get_atlas_api_url(), path);
    let mut req = CLIENT
        .get(&url)
        .header("Authorization", format!("Bearer {}", session_token))
        .headers(extra);
    if let Some(tid) = tenant_id {
        req = req.header("x-tenant-id", tid.to_string());
    }
    let res = req.send().await.map_err(|e| e.to_string())?;
    if !res.status().is_success() {
        let status = res.status();
        let msg = res.text().await.unwrap_or_default();
        return Err(format!("API {status}: {msg}"));
    }
    res.json::<T>().await.map_err(|e| e.to_string())
}

/// Authenticated POST — forwards session + tenant, serializes body as JSON.
pub async fn authenticated_post<B: Serialize, T: DeserializeOwned>(
    path: &str,
    session_token: &str,
    tenant_id: Option<uuid::Uuid>,
    body: &B,
) -> Result<T, String> {
    authenticated_post_with_headers(
        path,
        session_token,
        tenant_id,
        body,
        reqwest::header::HeaderMap::new(),
    )
    .await
}

/// Like [`authenticated_post`], with extra proxy headers (e.g. `X-Forwarded-Host`).
pub async fn authenticated_post_with_headers<B: Serialize, T: DeserializeOwned>(
    path: &str,
    session_token: &str,
    tenant_id: Option<uuid::Uuid>,
    body: &B,
    extra: reqwest::header::HeaderMap,
) -> Result<T, String> {
    let url = format!("{}{}", get_atlas_api_url(), path);
    let mut req = CLIENT
        .post(&url)
        .header("Authorization", format!("Bearer {}", session_token))
        .headers(extra)
        .json(body);
    if let Some(tid) = tenant_id {
        req = req.header("x-tenant-id", tid.to_string());
    }
    let res = req.send().await.map_err(|e| e.to_string())?;
    if !res.status().is_success() {
        let status = res.status();
        let msg = res.text().await.unwrap_or_default();
        return Err(format!("API {status}: {msg}"));
    }
    res.json::<T>().await.map_err(|e| e.to_string())
}

/// Authenticated DELETE — for resource removal.
pub async fn authenticated_delete(
    path: &str,
    session_token: &str,
    tenant_id: Option<uuid::Uuid>,
) -> Result<(), String> {
    let url = format!("{}{}", get_atlas_api_url(), path);
    let mut req = CLIENT
        .delete(&url)
        .header("Authorization", format!("Bearer {}", session_token));
    if let Some(tid) = tenant_id {
        req = req.header("x-tenant-id", tid.to_string());
    }
    let res = req.send().await.map_err(|e| e.to_string())?;
    if !res.status().is_success() {
        return Err(format!("API {}", res.status()));
    }
    Ok(())
}

/// Authenticated PATCH — forwards session + tenant, serializes body as JSON.
pub async fn authenticated_patch<B: Serialize, T: DeserializeOwned>(
    path: &str,
    session_token: &str,
    body: B,
) -> Result<T, String> {
    let url = format!("{}{}", get_atlas_api_url(), path);
    let req = CLIENT
        .patch(&url)
        .header("Authorization", format!("Bearer {}", session_token))
        .json(&body);
    let res = req.send().await.map_err(|e| e.to_string())?;
    if !res.status().is_success() {
        return Err(format!("API {}", res.status()));
    }
    res.json::<T>().await.map_err(|e| e.to_string())
}

/// Authenticated PUT — forwards session + optional tenant, serializes body as JSON.
pub async fn authenticated_put<B: Serialize, T: serde::de::DeserializeOwned>(
    path: &str,
    session_token: &str,
    tenant_id: Option<uuid::Uuid>,
    body: &B,
) -> Result<T, String> {
    let url = format!("{}{}", get_atlas_api_url(), path);
    let mut req = CLIENT
        .put(&url)
        .header("Authorization", format!("Bearer {}", session_token))
        .json(body);
    if let Some(tid) = tenant_id {
        req = req.header("x-tenant-id", tid.to_string());
    }
    let res = req.send().await.map_err(|e| e.to_string())?;
    if !res.status().is_success() {
        return Err(format!("API {}", res.status()));
    }
    // Handle empty 204 bodies
    if res.status() == reqwest::StatusCode::NO_CONTENT {
        return serde_json::from_value::<T>(serde_json::Value::Null).map_err(|_| String::new());
    }
    res.json::<T>().await.map_err(|e| e.to_string())
}
