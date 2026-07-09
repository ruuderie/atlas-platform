// apps/folio/src/pages/onboarding/invite_codes_client.rs
//
// Shared client-side server function for accepting invite codes at the end
// of every onboarding wizard.
//
// Called as the final step in:
//   - TenantApplicantWizard   (/onboard/tenant)
//   - CohostWizard            (/onboard/cohost)
//   - StrGuestWizard          (/onboard/str-guest)
//   - OwnerWizard             (/onboard/owner)
//   - AgentWizard             (/onboard/agent)
//   - BrokerWizard            (/onboard/broker)
//   - PmcWizard               (/onboard/pmc)
//   - VendorWizard            (/onboard/vendor)
//
// The accept endpoint:
//   1. Validates the code (active, not expired, not exhausted)
//   2. Atomically increments uses_count
//   3. Creates atlas_user_app_roles (G-32) for the accepting user
//   4. If property_manager + employer_user_id: creates G-11 contract
//   5. Returns { ok, role, redirect } — redirect used by all callers

use leptos::prelude::*;

/// Response shape from POST /api/folio/invite-codes/:id/accept
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AcceptCodeResponse {
    pub ok:       bool,
    pub role:     String,
    pub redirect: String,
}

/// Server function — accepts an invite code, provisions G-32 role, returns redirect.
///
/// `invite_code` is the SHORT CODE string (e.g. "OAK4B-K7X3"), not the UUID.
/// This matches the `code` field on `ResolvedInviteCode` which is what all
/// wizards have available. The backend performs the UUID lookup internally.
///
/// If `invite_code` is empty (direct sign-up without invite), the function
/// is a no-op and returns the fallback redirect unchanged.
#[server(AcceptInviteCode, "/api")]
pub async fn accept_invite_code(
    invite_code:       String,
    fallback_redirect: String,
) -> Result<AcceptCodeResponse, server_fn::error::ServerFnError> {
    if invite_code.is_empty() {
        return Ok(AcceptCodeResponse {
            ok:       true,
            role:     String::new(),
            redirect: fallback_redirect,
        });
    }

    use axum::http::HeaderMap;
    use leptos_axum::extract;
    let headers = extract::<HeaderMap>().await.unwrap_or_default();
    let token = crate::auth::extract_bearer_token(&headers)
        .ok_or_else(|| server_fn::error::ServerFnError::new("Not authenticated"))?;

    let result = crate::atlas_client::authenticated_post::<_, serde_json::Value>(
        &format!("/api/folio/invite-codes/by-code/{}/accept", invite_code),
        &token,
        None,
        &serde_json::json!({}),
    ).await.map_err(server_fn::error::ServerFnError::new)?;

    let redirect = result["redirect"]
        .as_str()
        .unwrap_or(&fallback_redirect)
        .to_string();

    Ok(AcceptCodeResponse {
        ok:       result["ok"].as_bool().unwrap_or(true),
        role:     result["role"].as_str().unwrap_or("").to_string(),
        redirect,
    })
}
