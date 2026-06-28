use leptos::prelude::*;
pub use server_fn::error::ServerFnError;
use serde::{Deserialize, Serialize};

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
    Vendor,
    /// Property Management Company operator — manages multiple client landlord
    /// accounts. Only valid when the tenant has `pmc_enabled: true` in their
    /// `atlas_app_deployment_config`.
    PropertyManager,
    /// Beneficial property owner who has delegated day-to-day management to a
    /// PMC. Read-only visibility into their own portfolio.
    Owner,
    /// Real estate agent — manages client files, listings, and deals.
    /// Requires `folio_mode = "brokerage"` on the instance. Home path: `/a`.
    Agent,
    /// Licensed real estate broker — manages agents and the office.
    /// Requires `folio_mode = "brokerage"` on the instance. Home path: `/b`.
    Broker,
}

impl FolioRole {
    /// Frontend namespace path for this role.
    pub fn home_path(&self) -> &'static str {
        match self {
            Self::Landlord        => "/l",
            Self::Tenant          => "/t",
            Self::Vendor          => "/v",
            Self::PropertyManager => "/pmc",
            Self::Owner           => "/o",
            Self::Agent           => "/a",
            Self::Broker          => "/b",
        }
    }
    pub fn label(&self) -> &'static str {
        match self {
            Self::Landlord        => "Property Manager",
            Self::Tenant          => "Tenant Portal",
            Self::Vendor          => "Vendor Portal",
            Self::PropertyManager => "PMC Dashboard",
            Self::Owner           => "Owner Portal",
            Self::Agent           => "Agent Portal",
            Self::Broker          => "Broker Portal",
        }
    }
}

// ── SessionInfo — returned by check_session() server fn ──────────────────────

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SessionInfo {
    pub user_id:             uuid::Uuid,
    pub tenant_id:           Option<uuid::Uuid>,
    pub email:               String,
    pub display_name:        Option<String>,
    pub folio_role:          FolioRole,
    /// True if the user has at least one registered passkey.
    #[serde(default)]
    pub has_passkey:         bool,
    /// True when all required onboarding steps are complete for their instance.
    #[serde(default)]
    pub onboarding_complete: bool,
    /// Number of wizard steps with a `completed_at` timestamp (for banner progress).
    #[serde(default)]
    pub wizard_steps_completed: usize,
    /// Total wizard steps for this instance (floor 7).
    #[serde(default = "default_wizard_total")]
    pub wizard_steps_total:  usize,
    /// True if the user previously dismissed the setup banner (persisted server-side).
    #[serde(default)]
    pub wizard_dismissed:    bool,
}

fn default_wizard_total() -> usize { 7 }


// ── Server functions ──────────────────────────────────────────────────────────

/// Validate the current session and return the user's Folio identity.
/// Calls `GET /api/folio/me` on the Atlas backend.
#[server(CheckSession, "/api")]
pub async fn check_session() -> Result<SessionInfo, ServerFnError> {
    use axum::http::HeaderMap;
    use leptos_axum::extract;

    let headers = extract::<HeaderMap>().await.unwrap_or_default();
    let token = extract_bearer_token(&headers)
        .ok_or_else(|| ServerFnError::new("No session token"))?;

    let info = crate::atlas_client::authenticated_get::<SessionInfo>(
        "/api/folio/me",
        &token,
        None,
    )
    .await
    .map_err(|e| ServerFnError::new(format!("Session check failed: {e}")))?;

    Ok(info)
}

/// Request a magic-link login email.
#[server(RequestMagicLink, "/api")]
pub async fn request_magic_link(email: String) -> Result<(), ServerFnError> {
    let payload = serde_json::json!({ "email": email });
    crate::atlas_client::authenticated_post::<_, serde_json::Value>(
        "/api/auth/magic-link/request",
        "",
        None,
        &payload,
    )
    .await
    .map(|_| ())
    .map_err(ServerFnError::new)
}

/// Verify a magic-link token, return session info (backend sets cookie).
#[server(VerifyMagicLink, "/api")]
pub async fn verify_magic_link(token: String) -> Result<SessionInfo, ServerFnError> {
    let payload = serde_json::json!({ "token": token });
    // Verify with the generic auth endpoint first to get the session cookie set,
    // then call /api/folio/me to get the folio-specific role.
    crate::atlas_client::authenticated_post::<_, serde_json::Value>(
        "/api/auth/magic-link/verify",
        "",
        None,
        &payload,
    )
    .await
    .map_err(|e| ServerFnError::new(format!("Token verification failed: {e}")))?;

    // Now fetch folio identity (session cookie is set by the verify call above).
    use axum::http::HeaderMap;
    use leptos_axum::extract;
    let headers = extract::<HeaderMap>().await.unwrap_or_default();
    let session_token = extract_bearer_token(&headers)
        .ok_or_else(|| ServerFnError::new("No session cookie after verify"))?;

    crate::atlas_client::authenticated_get::<SessionInfo>(
        "/api/folio/me",
        &session_token,
        None,
    )
    .await
    .map_err(|e| ServerFnError::new(e))
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

// ── Helpers ───────────────────────────────────────────────────────────────────

#[cfg(feature = "ssr")]
fn extract_bearer_token(headers: &axum::http::HeaderMap) -> Option<String> {
    headers
        .get(axum::http::header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .and_then(|s| s.strip_prefix("Bearer "))
        .map(|s| s.to_string())
        .or_else(|| {
            headers
                .get(axum::http::header::COOKIE)
                .and_then(|v| v.to_str().ok())
                .and_then(|cookies| {
                    cookies.split(';').find_map(|part| {
                        part.trim()
                            .strip_prefix("atlas_session=")
                            .map(|t| t.to_string())
                    })
                })
        })
}
