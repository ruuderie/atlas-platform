/// Pure unit tests — no DB, no network.
/// Covers T3 (tenant cookie isolation) and regression-guards the session
/// cookie builder and token extractor used by all auth flows.
#[cfg(test)]
mod tests {
    use crate::handlers::sessions::{
        clear_session_cookie_header, extract_session_token, session_cookie_header,
    };
    use axum::http::{HeaderMap, HeaderValue};

    // ── Cookie builder ────────────────────────────────────────────────────────

    // T3 REGRESSION GUARD: session_cookie_header must NEVER include a Domain= attribute.
    // Without Domain=, the browser scopes the cookie to the exact issuing host,
    // giving us free tenant isolation across subdomains per RFC 6265 §5.3.
    // If Domain= is ever added (e.g. to share across *.atlas.com), cookies will
    // bleed between tenants — this test must fail loudly before that ships.
    #[test]
    fn session_cookie_has_no_domain_attribute() {
        let cookie = session_cookie_header("test-token-abc", 86_400);
        assert!(
            !cookie.contains("Domain="),
            "session cookie must NOT contain Domain= — omitting it is what enforces \
             subdomain tenant isolation per RFC 6265 §5.3"
        );
    }

    // T3: Required security attributes must always be present.
    // Any removal of HttpOnly, Secure, or SameSite=Strict is a security regression.
    #[test]
    fn session_cookie_has_required_security_attributes() {
        let cookie = session_cookie_header("tok", 3600);
        assert!(
            cookie.contains("session=tok"),
            "cookie must contain session=<token>; got: {cookie}"
        );
        assert!(
            cookie.contains("HttpOnly"),
            "must be HttpOnly; got: {cookie}"
        );
        assert!(cookie.contains("Secure"), "must be Secure; got: {cookie}");
        assert!(
            cookie.contains("SameSite=Strict"),
            "must be SameSite=Strict; got: {cookie}"
        );
        assert!(cookie.contains("Path=/"), "must be Path=/; got: {cookie}");
        assert!(
            cookie.contains("Max-Age=3600"),
            "Max-Age must match argument; got: {cookie}"
        );
    }

    // T3: Logout cookie must zero out the session and immediately expire it.
    #[test]
    fn clear_session_cookie_zeroes_max_age() {
        let cookie = clear_session_cookie_header();
        assert!(
            cookie.contains("session=;"),
            "clear cookie must blank the value; got: {cookie}"
        );
        assert!(
            cookie.contains("Max-Age=0"),
            "clear cookie must set Max-Age=0; got: {cookie}"
        );
    }

    // ── Token extraction ──────────────────────────────────────────────────────

    // T2 REGRESSION GUARD: extract_session_token must prefer the HttpOnly cookie
    // over the Authorization header. The T2 fix relies on this: ManagePasskeys now
    // sends credentials:include instead of Authorization: Bearer, meaning the browser
    // sends the session cookie — which must win here.
    #[test]
    fn extract_session_token_prefers_cookie_over_bearer() {
        let mut headers = HeaderMap::new();
        headers.insert(
            axum::http::header::COOKIE,
            HeaderValue::from_static("session=cookie-token"),
        );
        headers.insert(
            axum::http::header::AUTHORIZATION,
            HeaderValue::from_static("Bearer bearer-token"),
        );
        let token = extract_session_token(&headers).unwrap();
        assert_eq!(
            token, "cookie-token",
            "cookie must win over Authorization header (T2 fix relies on this)"
        );
    }

    // REGRESSION: Bearer fallback still works when no cookie is present.
    // This covers legacy API clients and the test harness that uses Bearer tokens.
    #[test]
    fn extract_session_token_falls_back_to_bearer() {
        let mut headers = HeaderMap::new();
        headers.insert(
            axum::http::header::AUTHORIZATION,
            HeaderValue::from_static("Bearer my-bearer"),
        );
        let token = extract_session_token(&headers).unwrap();
        assert_eq!(token, "my-bearer");
    }

    // REGRESSION: empty cookie value must be treated as absent and fall through to Bearer.
    // The existing implementation's `!token.is_empty()` guard handles this.
    #[test]
    fn extract_session_token_ignores_empty_cookie_value() {
        let mut headers = HeaderMap::new();
        headers.insert(
            axum::http::header::COOKIE,
            HeaderValue::from_static("session="),
        );
        headers.insert(
            axum::http::header::AUTHORIZATION,
            HeaderValue::from_static("Bearer fallback"),
        );
        let token = extract_session_token(&headers).unwrap();
        assert_eq!(
            token, "fallback",
            "empty session= cookie value must be treated as absent and fall through to Bearer"
        );
    }

    // REGRESSION: multiple cookies in one header — session= is extracted correctly.
    #[test]
    fn extract_session_token_handles_multiple_cookies() {
        let mut headers = HeaderMap::new();
        headers.insert(
            axum::http::header::COOKIE,
            HeaderValue::from_static("other=xyz; session=my-session; another=abc"),
        );
        let token = extract_session_token(&headers).unwrap();
        assert_eq!(token, "my-session");
    }

    // REGRESSION: no auth at all → None (not a panic).
    #[test]
    fn extract_session_token_returns_none_when_absent() {
        let headers = HeaderMap::new();
        assert!(extract_session_token(&headers).is_none());
    }
}
