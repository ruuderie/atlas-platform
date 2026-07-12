use leptos::prelude::*;
use serde::{Deserialize, Serialize};
pub use server_fn::error::ServerFnError;

// ── FolioRole — shared between SSR and WASM ───────────────────────────────────
//
// IMPORTANT: This enum must stay in sync with the backend `FolioRole` in
// `backend/src/types/pm.rs`. When the backend adds a new variant the frontend
// must add the corresponding arm here — otherwise the login redirect silently
// falls back to Landlord for any new-variant user and their API calls return 403.

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum FolioRole {
    #[default]
    Landlord,
    Tenant,
    /// STR booking guest — short-term rental visitor.
    /// Linked to a booking (atlas_bookings), not a lease.
    /// Onboarding: select/confirm dates → profile → house rules.
    /// Home path: `/g`. Distinct from Tenant (LTR applicant/renter).
    StrGuest,
    Vendor,
    /// Property Management Company operator — manages multiple client landlord
    /// accounts. Only valid when the tenant has `pmc_enabled: true` in their
    /// `atlas_app_deployment_config`.
    PropertyManager,
    /// Beneficial property owner who has delegated day-to-day management to a
    /// PMC. Read-only visibility into their own portfolio.
    Owner,
    /// STR co-host — manages bookings, messaging, and operations for specific
    /// STR properties they've been delegated access to. Asset-scoped.
    Cohost,
    // NOTE: StrHost is NOT a separate role. STR capability is a trait on
    // atlas_assets (str_eligible = true). A Landlord who has STR-eligible assets
    // gets the STR nav sections shown dynamically in their Landlord portal.
    /// Real estate agent — manages client files, listings, and deals.
    /// Requires `folio_mode = "brokerage"` on the instance. Home path: `/a`.
    Agent,
    /// Licensed real estate broker — manages agents and the office.
    /// Requires `folio_mode = "brokerage"` on the instance. Home path: `/b`.
    Broker,
    /// Free-tier property owner — self-registered, no landlord invite required.
    /// Can: log property valuations, browse vendor marketplace, submit G-27 reviews.
    /// Cannot: manage leases, billing, tenants — those require upgrade to Landlord.
    /// Home path: `/po`. Deserializes from backend "property_owner_lite" role slug.
    PropertyOwnerLite,
}

impl FolioRole {
    /// Frontend namespace path for this role.
    pub fn home_path(&self) -> &'static str {
        match self {
            Self::Landlord => "/l",
            Self::Tenant => "/t",
            Self::StrGuest => "/g",
            Self::Vendor => "/v",
            Self::PropertyManager => "/pmc",
            Self::Owner => "/o",
            Self::Cohost => "/ch",
            Self::Agent => "/a",
            Self::Broker => "/b",
            Self::PropertyOwnerLite => "/po",
        }
    }
    pub fn label(&self) -> &'static str {
        match self {
            Self::Landlord => "Property Manager",
            Self::Tenant => "Tenant Portal",
            Self::StrGuest => "Guest Portal",
            Self::Vendor => "Vendor Portal",
            Self::PropertyManager => "PMC Dashboard",
            Self::Owner => "Owner Portal",
            Self::Cohost => "Cohost Portal",
            Self::Agent => "Agent Portal",
            Self::Broker => "Broker Portal",
            Self::PropertyOwnerLite => "Property Owner Portal",
        }
    }

    /// True for the free-tier property owner (no lease/billing capabilities).
    pub fn is_lite(&self) -> bool {
        matches!(self, Self::PropertyOwnerLite)
    }
}

// ── SessionInfo — returned by check_session() server fn ──────────────────────

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SessionInfo {
    pub user_id: uuid::Uuid,
    pub tenant_id: Option<uuid::Uuid>,
    pub email: String,
    pub display_name: Option<String>,
    pub folio_role: FolioRole,
    /// True if the user has at least one registered passkey.
    #[serde(default)]
    pub has_passkey: bool,
    /// True when all required onboarding steps are complete for their instance.
    #[serde(default)]
    pub onboarding_complete: bool,
    /// Number of wizard steps with a `completed_at` timestamp (for banner progress).
    #[serde(default)]
    pub wizard_steps_completed: usize,
    /// Total wizard steps for this instance (floor 7).
    #[serde(default = "default_wizard_total")]
    pub wizard_steps_total: usize,
    /// True if the user previously dismissed the setup banner (persisted server-side).
    #[serde(default)]
    pub wizard_dismissed: bool,
    /// True if the user (Landlord role) has at least one STR-eligible asset in their portfolio.
    /// When true, the STR nav sections (calendar, reservations, channels) are shown
    /// in the Landlord dashboard. This is an asset trait, NOT a role distinction.
    #[serde(default)]
    pub has_str_assets: bool,
    /// Lease type for the user's active lease (Tenant role only): "ltr" | "str".
    /// Determines which tenant portal view is shown (full portal vs guest view).
    /// None if role != Tenant or no active lease found.
    #[serde(default)]
    pub active_lease_type: Option<String>,
}

fn default_wizard_total() -> usize {
    7
}

/// sessionStorage key written after magic-link / OTP so onboarding can skip
/// re-asking for email in the same browser tab.
pub const FOLIO_VERIFIED_EMAIL_KEY: &str = "folio_verified_email";

/// Persist verified email for same-tab handoff into onboarding wizards.
pub fn stash_verified_email(email: &str) {
    let email = email.trim();
    if email.is_empty() {
        return;
    }
    #[cfg(target_arch = "wasm32")]
    {
        if let Some(window) = web_sys::window() {
            if let Ok(Some(storage)) = window.session_storage() {
                let _ = storage.set_item(FOLIO_VERIFIED_EMAIL_KEY, email);
            }
        }
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = email;
    }
}

/// Read stashed verified email (same-tab assist after magic-link verify).
pub fn read_stashed_verified_email() -> Option<String> {
    #[cfg(target_arch = "wasm32")]
    {
        let window = web_sys::window()?;
        let storage = window.session_storage().ok()??;
        let email = storage.get_item(FOLIO_VERIFIED_EMAIL_KEY).ok()??;
        let email = email.trim().to_string();
        if email.is_empty() {
            None
        } else {
            Some(email)
        }
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        None
    }
}

/// Resolve verified email for WizardShell pre-auth skip.
/// Precedence: Folio session (`/api/folio/me`) → auth peek → same-tab stash.
pub fn resolve_verified_email_probe(
    session_email: Option<String>,
    peek_email: Option<String>,
    stash_email: Option<String>,
) -> Option<String> {
    [session_email, peek_email, stash_email]
        .into_iter()
        .flatten()
        .map(|e| e.trim().to_string())
        .find(|e| !e.is_empty())
}

/// Map Atlas API errors into a user-facing message for auth server fns.
pub fn humanize_auth_api_error(err: &str) -> String {
    if err.contains("429") {
        "Too many sign-in attempts. Wait a few minutes and try again.".to_string()
    } else {
        err.to_string()
    }
}

// ── Shared SSR token extractor ────────────────────────────────────────────────
//
// Single authoritative function for extracting the session token from an
// incoming request inside a Leptos server function.
//
// Accepts tokens in priority order:
//   1. `Authorization: Bearer <TOKEN>` header
//   2. `session=<TOKEN>` cookie  (set by auth_frontend.rs / verify_handler)
//   3. `atlas_session=<TOKEN>` cookie  (legacy name — kept for compatibility)
//
// The `session=` cookie is the canonical name used by all backend handlers
// (sessions.rs, auth_frontend.rs). The `atlas_session=` fallback exists because
// several server functions were written before the naming was standardised.
// Both are accepted here so neither old nor new sessions fail.
//
// Usage inside a #[server] function:
//   let headers = extract::<HeaderMap>().await.unwrap_or_default();
//   let token = crate::auth::extract_bearer_token(&headers)
//       .ok_or_else(|| ServerFnError::new("No session token"))?;
#[cfg(feature = "ssr")]
pub fn extract_bearer_token(headers: &axum::http::HeaderMap) -> Option<String> {
    // 1. Authorization: Bearer
    if let Some(token) = headers
        .get(axum::http::header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .and_then(|s| s.strip_prefix("Bearer ").map(str::to_string))
    {
        return Some(token);
    }

    // 2. Cookie header — try both known names
    let cookie_str = headers
        .get(axum::http::header::COOKIE)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");

    for part in cookie_str.split(';') {
        let part = part.trim();
        if let Some(t) = part.strip_prefix("session=") {
            return Some(t.to_string());
        }
        if let Some(t) = part.strip_prefix("atlas_session=") {
            return Some(t.to_string());
        }
    }

    None
}

// ── Server functions ──────────────────────────────────────────────────────────

#[cfg(feature = "ssr")]
static SERVER_SESSION_CACHE: std::sync::OnceLock<moka::future::Cache<String, SessionInfo>> =
    std::sync::OnceLock::new();

#[cfg(feature = "ssr")]
fn get_server_session_cache() -> moka::future::Cache<String, SessionInfo> {
    SERVER_SESSION_CACHE
        .get_or_init(|| {
            moka::future::Cache::builder()
                .max_capacity(2000)
                .time_to_live(std::time::Duration::from_secs(30)) // 30 seconds TTL
                .build()
        })
        .clone()
}

#[cfg(not(feature = "ssr"))]
mod client_cache {
    use super::SessionInfo;
    use std::cell::RefCell;

    thread_local! {
        static CACHED_SESSION: RefCell<Option<(f64, SessionInfo)>> = const { RefCell::new(None) };
    }

    const TTL_MS: f64 = 15_000.0; // 15 seconds TTL on client

    pub fn get() -> Option<SessionInfo> {
        CACHED_SESSION.with(|cache| {
            if let Some((timestamp, ref info)) = *cache.borrow() {
                let now = js_sys::Date::now();
                if now - timestamp < TTL_MS {
                    return Some(info.clone());
                }
            }
            None
        })
    }

    pub fn set(info: SessionInfo) {
        CACHED_SESSION.with(|cache| {
            *cache.borrow_mut() = Some((js_sys::Date::now(), info));
        });
    }

    pub fn clear() {
        CACHED_SESSION.with(|cache| {
            *cache.borrow_mut() = None;
        });
    }
}

/// Client/Server caching wrapper around check_session.
/// Use this instead of check_session() directly in components.
pub async fn get_session() -> Result<SessionInfo, ServerFnError> {
    #[cfg(not(feature = "ssr"))]
    {
        if let Some(info) = client_cache::get() {
            return Ok(info);
        }
    }

    let res = check_session().await;

    #[cfg(not(feature = "ssr"))]
    {
        match &res {
            Ok(info) => client_cache::set(info.clone()),
            Err(_) => client_cache::clear(),
        }
    }

    res
}

/// Minimal auth identity for flows that run *before* Folio RBAC exists.
///
/// Fresh magic-link users often have a valid `session` cookie but
/// `GET /api/folio/me` returns 403 (no tenant / folio role yet). Landlord
/// onboarding must still treat them as authenticated and skip the OTP gate.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AuthPeek {
    pub email: String,
}

/// Peek a valid platform session without requiring Folio RBAC.
/// Uses `GET /api/auth/session/validate` (works with cookie alone).
#[server(PeekAuthSession, "/api")]
pub async fn peek_auth_session() -> Result<AuthPeek, ServerFnError> {
    use axum::http::HeaderMap;
    use leptos_axum::extract;

    let headers = extract::<HeaderMap>().await.unwrap_or_default();
    let token =
        extract_bearer_token(&headers).ok_or_else(|| ServerFnError::new("No session token"))?;

    #[cfg(feature = "ssr")]
    {
        #[derive(Deserialize)]
        struct ValidateUser {
            email: String,
        }
        #[derive(Deserialize)]
        struct ValidateResp {
            user: Option<ValidateUser>,
        }

        let resp = crate::atlas_client::authenticated_get_with_headers::<ValidateResp>(
            "/api/auth/session/validate",
            &token,
            None,
            crate::atlas_client::folio_proxy_headers(&headers),
        )
        .await
        .map_err(|e| ServerFnError::new(format!("Session peek failed: {e}")))?;

        let email = resp
            .user
            .map(|u| u.email)
            .filter(|e| !e.is_empty())
            .ok_or_else(|| ServerFnError::new("Session has no email"))?;

        Ok(AuthPeek { email })
    }
    #[cfg(not(feature = "ssr"))]
    {
        let _ = token;
        Err(ServerFnError::new("Client fallback"))
    }
}

/// Validate the current session and return the user's Folio identity.
/// Calls `GET /api/folio/me` on the Atlas backend.
#[server(CheckSession, "/api")]
pub async fn check_session() -> Result<SessionInfo, ServerFnError> {
    use axum::http::HeaderMap;
    use leptos_axum::extract;

    let headers = extract::<HeaderMap>().await.unwrap_or_default();
    let token =
        extract_bearer_token(&headers).ok_or_else(|| ServerFnError::new("No session token"))?;

    #[cfg(feature = "ssr")]
    {
        let cache = get_server_session_cache();
        if let Some(info) = cache.get(&token).await {
            return Ok(info);
        }

        let info =
            crate::atlas_client::authenticated_get::<SessionInfo>("/api/folio/me", &token, None)
                .await
                .map_err(|e| ServerFnError::new(format!("Session check failed: {e}")))?;

        cache.insert(token, info.clone()).await;
        Ok(info)
    }
    #[cfg(not(feature = "ssr"))]
    {
        let _ = token;
        Err(ServerFnError::new("Client fallback"))
    }
}

/// Local Compose/Caddy serves plain HTTP on `*.localhost` (no TLS on :443).
/// `folio.localhost` does **not** start with `"localhost"`, so a naive prefix
/// check wrongly picks `https` and magic-link emails refuse to connect.
#[cfg(feature = "ssr")]
fn request_scheme_for_host(host: &str) -> &'static str {
    let host = host.split(':').next().unwrap_or(host);
    let h = host.to_ascii_lowercase();
    if h == "localhost" || h.starts_with("127.") || h.ends_with(".localhost") {
        "http"
    } else {
        "https"
    }
}

/// Request a magic-link login email.
#[server(RequestMagicLink, "/api")]
pub async fn request_magic_link(email: String) -> Result<(), ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        use axum::http::HeaderMap;
        use leptos_axum::extract;

        let headers = extract::<HeaderMap>().await.unwrap_or_default();
        let host = headers
            .get("host")
            .and_then(|v| v.to_str().ok())
            .unwrap_or("localhost");
        let scheme = request_scheme_for_host(host);
        let redirect_url = format!("{}://{}/verify", scheme, host);

        let payload = serde_json::json!({
            "email": email,
            "redirect_url": redirect_url,
        });
        // Forward the browser IP so auth rate limits aren't shared across all Folio users.
        let fwd = crate::atlas_client::forward_client_ip(&headers);
        crate::atlas_client::post_with_headers::<_, serde_json::Value>(
            "/api/auth/magic-link/request",
            &payload,
            fwd,
        )
        .await
        .map(|_| ())
        .map_err(|e| ServerFnError::new(humanize_auth_api_error(&e)))
    }
    #[cfg(not(feature = "ssr"))]
    {
        let _ = email;
        Ok(())
    }
}

/// Verify a magic-link token and return the Folio session.
///
/// # Why this is not a simple round-trip
///
/// The backend's `SessionResponse.token` field carries
/// `#[serde(skip_serializing)]` — it is intentionally absent from the JSON
/// body for security. The session token travels exclusively as a
/// `Set-Cookie: session=TOKEN` response header.
///
/// This server function therefore:
/// 1. Calls the backend verify endpoint via `post_returning_session` which
///    reads the session token out of the `Set-Cookie` response header.
/// 2. Forwards the session cookie to the browser via `ResponseOptions` so
///    all subsequent browser requests are authenticated.
/// 3. Uses the captured token directly to call `/api/folio/me` and return
///    the Folio identity — no need to re-extract from incoming headers.
#[server(VerifyMagicLink, "/api")]
pub async fn verify_magic_link(token: String) -> Result<SessionInfo, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        use leptos_axum::ResponseOptions;
        let resp_opts = use_context::<ResponseOptions>();

        let payload = serde_json::json!({ "token": token });

        let (body, session_token_opt) = crate::atlas_client::post_returning_session::<
            _,
            serde_json::Value,
        >("/api/auth/magic-link/verify", &payload)
        .await
        .map_err(|e| ServerFnError::new(format!("Token verification failed: {e}")))?;

        let session_token = session_token_opt
            .ok_or_else(|| ServerFnError::new("No session cookie after verify"))?;

        // Forward the session cookie to the browser so it persists for all
        // subsequent requests. We mirror the same cookie attributes the backend
        // uses in session_cookie_header().
        if let Some(resp) = resp_opts {
            if let Ok(cookie_val) = axum::http::HeaderValue::from_str(&format!(
                "session={}; HttpOnly; Secure; SameSite=Strict; Path=/; Max-Age=86400",
                session_token
            )) {
                resp.insert_header(axum::http::header::SET_COOKIE, cookie_val);
            }
        }

        // Prefer full Folio identity when RBAC exists. If /api/folio/me fails
        // (common before onboarding), fall back to verify-response user email
        // so the client can still enter wizards without OTP again.
        match crate::atlas_client::authenticated_get::<SessionInfo>(
            "/api/folio/me",
            &session_token,
            None,
        )
        .await
        {
            Ok(info) => Ok(info),
            Err(_) => {
                let user = body.get("user").ok_or_else(|| {
                    ServerFnError::new("Session created but Folio identity unavailable")
                })?;
                let email = user
                    .get("email")
                    .and_then(|v| v.as_str())
                    .filter(|e| !e.is_empty())
                    .ok_or_else(|| ServerFnError::new("Verify response has no email"))?
                    .to_string();
                let user_id = user
                    .get("id")
                    .and_then(|v| v.as_str())
                    .and_then(|s| uuid::Uuid::parse_str(s).ok())
                    .unwrap_or_else(uuid::Uuid::nil);
                let first = user
                    .get("first_name")
                    .and_then(|v| v.as_str())
                    .unwrap_or("");
                let last = user
                    .get("last_name")
                    .and_then(|v| v.as_str())
                    .unwrap_or("");
                let display_name = {
                    let n = format!("{first} {last}").trim().to_string();
                    if n.is_empty() {
                        None
                    } else {
                        Some(n)
                    }
                };
                Ok(SessionInfo {
                    user_id,
                    tenant_id: None,
                    email,
                    display_name,
                    folio_role: FolioRole::Landlord,
                    has_passkey: false,
                    onboarding_complete: false,
                    wizard_steps_completed: 0,
                    wizard_steps_total: default_wizard_total(),
                    wizard_dismissed: false,
                    has_str_assets: false,
                    active_lease_type: None,
                })
            }
        }
    }
    #[cfg(not(feature = "ssr"))]
    {
        let _ = token;
        Err(ServerFnError::new("Client fallback"))
    }
}

/// Revoke the current session (logout).
#[server(RevokeSession, "/api")]
pub async fn revoke_session() -> Result<(), ServerFnError> {
    use axum::http::HeaderMap;
    use leptos_axum::extract;
    let headers = extract::<HeaderMap>().await.unwrap_or_default();
    if let Some(token) = extract_bearer_token(&headers) {
        let _ = crate::atlas_client::authenticated_post::<_, serde_json::Value>(
            "/api/auth/logout",
            &token,
            None,
            &serde_json::Value::Null,
        )
        .await;
    }
    Ok(())
}

// ── Unit tests ────────────────────────────────────────────────────────────────
//
// Note: extract_bearer_token is SSR-only (cfg(feature = "ssr")).
// Those tests are compiled only when running cargo test with --features ssr.

#[cfg(test)]
mod probe_tests {
    use super::{humanize_auth_api_error, resolve_verified_email_probe};

    #[test]
    fn probe_prefers_session_over_peek_and_stash() {
        let got = resolve_verified_email_probe(
            Some("session@example.com".into()),
            Some("peek@example.com".into()),
            Some("stash@example.com".into()),
        );
        assert_eq!(got.as_deref(), Some("session@example.com"));
    }

    #[test]
    fn probe_falls_back_to_peek_when_no_session() {
        let got = resolve_verified_email_probe(
            None,
            Some("peek@example.com".into()),
            Some("stash@example.com".into()),
        );
        assert_eq!(got.as_deref(), Some("peek@example.com"));
    }

    #[test]
    fn probe_falls_back_to_stash_when_session_and_peek_missing() {
        let got =
            resolve_verified_email_probe(None, None, Some("stash@example.com".into()));
        assert_eq!(got.as_deref(), Some("stash@example.com"));
    }

    #[test]
    fn probe_skips_blank_sources() {
        let got = resolve_verified_email_probe(
            Some("  ".into()),
            Some("".into()),
            Some(" stash@example.com ".into()),
        );
        assert_eq!(got.as_deref(), Some("stash@example.com"));
    }

    #[test]
    fn probe_returns_none_when_all_missing() {
        assert!(resolve_verified_email_probe(None, None, None).is_none());
    }

    #[test]
    fn humanize_maps_429_to_friendly_message() {
        let msg = humanize_auth_api_error("API 429 Too Many Requests: ");
        assert!(msg.contains("Too many sign-in attempts"));
        assert!(!msg.contains("429"));
    }

    #[test]
    fn humanize_passes_through_other_errors() {
        let raw = "API 500: boom";
        assert_eq!(humanize_auth_api_error(raw), raw);
    }
}

#[cfg(all(test, feature = "ssr"))]
mod tests {
    use super::extract_bearer_token;
    use axum::http::{header, HeaderMap, HeaderValue};

    fn headers_with(cookie: &str) -> HeaderMap {
        let mut h = HeaderMap::new();
        h.insert(header::COOKIE, HeaderValue::from_str(cookie).unwrap());
        h
    }

    fn headers_with_bearer(token: &str) -> HeaderMap {
        let mut h = HeaderMap::new();
        h.insert(
            header::AUTHORIZATION,
            HeaderValue::from_str(&format!("Bearer {}", token)).unwrap(),
        );
        h
    }

    // ── session= cookie (canonical backend cookie name) ───────────────────────

    /// REGRESSION: backend sets `Set-Cookie: session=TOKEN`. This must be
    /// recognised by extract_bearer_token.  The original implementation only
    /// looked for `atlas_session=` and so returned None even after a successful
    /// magic-link verify, causing "No session cookie after verify".
    #[test]
    fn session_cookie_is_accepted() {
        let headers = headers_with("session=my-token-abc");
        assert_eq!(
            extract_bearer_token(&headers).as_deref(),
            Some("my-token-abc"),
            "REGRESSION: 'session=' cookie must be accepted — it is what the backend sets"
        );
    }

    /// session= with multiple cookies — correct value extracted.
    #[test]
    fn session_cookie_among_multiple_cookies() {
        let headers = headers_with("other=xyz; session=correct-token; another=abc");
        assert_eq!(
            extract_bearer_token(&headers).as_deref(),
            Some("correct-token")
        );
    }

    // ── atlas_session= cookie (legacy alias) ──────────────────────────────────

    /// Legacy alias must still work so existing browser sessions aren't invalidated
    /// if cookies were set before the cookie-name normalisation.
    #[test]
    fn atlas_session_cookie_legacy_alias_is_accepted() {
        let headers = headers_with("atlas_session=legacy-token");
        assert_eq!(
            extract_bearer_token(&headers).as_deref(),
            Some("legacy-token"),
            "'atlas_session=' must still be accepted as a legacy alias"
        );
    }

    // ── Authorization: Bearer ─────────────────────────────────────────────────

    /// Bearer token (used by server-to-server SSR calls) must be accepted.
    #[test]
    fn bearer_header_is_accepted_when_no_cookie() {
        let headers = headers_with_bearer("srv-token-xyz");
        assert_eq!(
            extract_bearer_token(&headers).as_deref(),
            Some("srv-token-xyz")
        );
    }

    // ── Missing auth ──────────────────────────────────────────────────────────

    /// No auth at all → None (not a panic).
    #[test]
    fn returns_none_when_no_auth_present() {
        assert!(extract_bearer_token(&HeaderMap::new()).is_none());
    }

    /// Unrelated cookie, no bearer → None.
    #[test]
    fn returns_none_when_only_unrelated_cookie() {
        let headers = headers_with("csrf=abcdef; other=value");
        assert!(extract_bearer_token(&headers).is_none());
    }
}

#[cfg(all(test, feature = "ssr"))]
mod forward_ip_tests {
    use axum::http::{header, HeaderMap, HeaderValue};

    #[test]
    fn forwards_x_forwarded_for_first_hop() {
        let mut h = HeaderMap::new();
        h.insert(
            header::HeaderName::from_static("x-forwarded-for"),
            HeaderValue::from_static("203.0.113.9, 10.0.0.1"),
        );
        let out = crate::atlas_client::forward_client_ip(&h);
        assert_eq!(
            out.get("x-forwarded-for").and_then(|v| v.to_str().ok()),
            Some("203.0.113.9")
        );
    }

    #[test]
    fn falls_back_to_x_real_ip() {
        let mut h = HeaderMap::new();
        h.insert(
            header::HeaderName::from_static("x-real-ip"),
            HeaderValue::from_static("198.51.100.7"),
        );
        let out = crate::atlas_client::forward_client_ip(&h);
        assert_eq!(
            out.get("x-forwarded-for").and_then(|v| v.to_str().ok()),
            Some("198.51.100.7")
        );
    }

    #[test]
    fn empty_when_no_client_ip_headers() {
        let out = crate::atlas_client::forward_client_ip(&HeaderMap::new());
        assert!(out.get("x-forwarded-for").is_none());
    }
}
