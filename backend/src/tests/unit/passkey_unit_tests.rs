//! Pure unit tests for passkey finish-login wire contract.
//!
//! Folio proxies `/api/passkeys/*` and plants the browser session from the
//! JSON `token` when multi-value `Set-Cookie` is lost in transit. These guards
//! keep that contract from regressing to magic-link semantics (token omitted).

#[cfg(test)]
mod tests {
    use crate::handlers::passkeys::{passkey_finish_login_cookies, passkey_finish_login_json};
    use crate::models::session::{SessionResponse, UserInfo};
    use uuid::Uuid;

    fn sample_session(token: &str) -> SessionResponse {
        SessionResponse {
            user: Some(UserInfo {
                id: Uuid::nil(),
                email: "r.erie@example.com".into(),
                first_name: "Ruud".into(),
                last_name: "Erie".into(),
                is_admin: false,
                app_permissions: vec![],
            }),
            token: token.into(),
            refresh_token: "refresh".into(),
        }
    }

    /// REGRESSION: SessionResponse.token is skip_serializing — naive
    /// `to_value(&session)` omits it. finish-login must put `token` back so
    /// Folio `establish_session_from_token` can run after WebAuthn.
    #[test]
    fn finish_login_json_includes_session_token() {
        let session = sample_session("passkey-session-token-xyz");
        let raw = serde_json::to_value(&session).expect("serialize");
        assert!(
            raw.get("token").is_none(),
            "SessionResponse must still skip_serializing token by default"
        );

        let body = passkey_finish_login_json(&session).expect("finish json");
        assert_eq!(
            body.get("token").and_then(|t| t.as_str()),
            Some("passkey-session-token-xyz"),
            "finish-login body must expose token for Folio cookie planting"
        );
        assert_eq!(
            body.pointer("/user/email").and_then(|e| e.as_str()),
            Some("r.erie@example.com")
        );
        // refresh_token stays omitted from the wire body
        assert!(body.get("refresh_token").is_none());
    }

    #[test]
    fn finish_login_cookies_set_session_and_clear_passkey_session() {
        let prev = std::env::var("ENVIRONMENT").ok();
        unsafe { std::env::set_var("ENVIRONMENT", "development") };

        let (session_cookie, clear_pk) = passkey_finish_login_cookies("tok-123");
        assert!(
            session_cookie.starts_with("session=tok-123"),
            "first cookie must be session=; got {session_cookie}"
        );
        assert!(session_cookie.contains("HttpOnly"));
        assert!(session_cookie.contains("Path=/"));
        assert!(
            clear_pk.starts_with("passkey_session="),
            "second cookie must clear passkey_session; got {clear_pk}"
        );
        assert!(
            clear_pk.contains("Max-Age=0"),
            "passkey_session clear must expire immediately; got {clear_pk}"
        );

        match prev {
            Some(v) => unsafe { std::env::set_var("ENVIRONMENT", v) },
            None => unsafe { std::env::remove_var("ENVIRONMENT") },
        }
    }
}
