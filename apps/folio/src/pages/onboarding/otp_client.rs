// apps/folio/src/pages/onboarding/otp_client.rs
//
// Server functions for the inline OTP pre-auth flow.
//
// These are called from WizardShell's pre-auth phase before the wizard steps render.
// Both are unauthenticated — they are how the user establishes their session.

use leptos::prelude::*;
use serde::{Deserialize, Serialize};

// ── Response types ─────────────────────────────────────────────────────────────

/// Session info returned from otp/verify — mirrors the backend SessionResponse.
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct OtpSessionResponse {
    pub token: String,
    pub email: String,
}

// ── Server functions ───────────────────────────────────────────────────────────

/// Send a 6-digit OTP to the given email.
/// Creates a stub user if the email doesn't exist yet.
#[server(SendOtp, "/api")]
pub async fn send_otp(email: String) -> Result<(), server_fn::error::ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        use axum::http::HeaderMap;
        use leptos_axum::extract;

        let headers = extract::<HeaderMap>().await.unwrap_or_default();
        let fwd = crate::atlas_client::forward_client_ip(&headers);
        crate::atlas_client::post_with_headers::<_, serde_json::Value>(
            "/api/auth/otp/send",
            &serde_json::json!({ "email": email.trim().to_lowercase() }),
            fwd,
        )
        .await
        .map(|_| ())
        .map_err(|e| {
            server_fn::error::ServerFnError::new(crate::auth::humanize_auth_api_error(&e))
        })
    }
    #[cfg(not(feature = "ssr"))]
    {
        let _ = email;
        Ok(())
    }
}

/// Verify a 6-digit OTP for the given email.
/// Returns the session token + email on success.
/// The backend also sets an HttpOnly session cookie via Set-Cookie.
#[server(VerifyOtp, "/api")]
pub async fn verify_otp(
    email: String,
    code: String,
) -> Result<OtpSessionResponse, server_fn::error::ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        use axum::http::HeaderMap;
        use leptos_axum::{extract, ResponseOptions};

        let headers = extract::<HeaderMap>().await.unwrap_or_default();
        let fwd = crate::atlas_client::forward_client_ip(&headers);
        let resp_opts = use_context::<ResponseOptions>();

        // post_returning_session extracts the bearer token from the Set-Cookie header,
        // which is how the backend returns the session for OTP just like magic links.
        let (resp, token_opt) =
            crate::atlas_client::post_returning_session_with_headers::<_, serde_json::Value>(
                "/api/auth/otp/verify",
                &serde_json::json!({
                    "email": email.trim().to_lowercase(),
                    "code":  code.trim(),
                }),
                fwd,
            )
            .await
            .map_err(|e| {
                server_fn::error::ServerFnError::new(crate::auth::humanize_auth_api_error(&e))
            })?;

        let token = token_opt.unwrap_or_else(|| {
            // Fallback: try JSON body (shouldn't happen if backend is consistent)
            resp["token"].as_str().unwrap_or("").to_string()
        });

        if token.is_empty() {
            return Err(server_fn::error::ServerFnError::new(
                "No session cookie after OTP verify",
            ));
        }

        // Forward the session cookie (same attributes as magic-link verify).
        if let Some(resp_opts) = resp_opts {
            if let Ok(cookie_val) =
                axum::http::HeaderValue::from_str(&crate::auth::session_cookie_header(&token))
            {
                resp_opts.insert_header(axum::http::header::SET_COOKIE, cookie_val);
            }
        }

        let email_out = resp["email"].as_str().unwrap_or(&email).to_string();

        Ok(OtpSessionResponse {
            token,
            email: email_out,
        })
    }
    #[cfg(not(feature = "ssr"))]
    {
        let _ = (email, code);
        Err(server_fn::error::ServerFnError::new("Client fallback"))
    }
}
