//! Helpers for Folio → Atlas passkey reverse-proxy responses.
//!
//! Kept in the lib (not only `main.rs`) so the multi-`Set-Cookie` merge can be
//! unit-tested — a prior bug used `Response::builder().header()` which replaces
//! same-name keys and dropped `session=` when Atlas also cleared `passkey_session=`.

use axum::http::{header, HeaderMap, HeaderName, HeaderValue};

/// Hop-by-hop / framing headers that must not be forwarded from upstream.
fn should_skip_upstream_header(name: &HeaderName) -> bool {
    matches!(
        name.as_str(),
        "transfer-encoding"
            | "connection"
            | "keep-alive"
            | "content-length"
            | "set-cookie" // handled via get_all / append_set_cookies
    )
}

/// Merge upstream response headers into an outbound map, **appending** every
/// `Set-Cookie` value so dual cookies from passkey finish-login survive.
pub fn merge_proxied_response_headers(
    upstream: &HeaderMap,
    set_cookies: impl IntoIterator<Item = HeaderValue>,
) -> HeaderMap {
    let mut out = HeaderMap::new();
    for (k, v) in upstream.iter() {
        if should_skip_upstream_header(k) {
            continue;
        }
        out.append(k, v.clone());
    }
    for v in set_cookies {
        out.append(header::SET_COOKIE, v);
    }
    out
}

/// Collect `Set-Cookie` values from an upstream header map via `get_all`.
pub fn collect_set_cookies(upstream: &HeaderMap) -> Vec<HeaderValue> {
    upstream
        .get_all(header::SET_COOKIE)
        .iter()
        .cloned()
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::HeaderValue;

    fn cookie_vals(out: &HeaderMap) -> Vec<String> {
        out.get_all(header::SET_COOKIE)
            .iter()
            .map(|v| v.to_str().unwrap().to_string())
            .collect()
    }

    /// REGRESSION: finish-login returns `session=…` plus clearing `passkey_session=…`.
    /// Using insert/builder.header for Set-Cookie kept only the last value →
    /// browser never got `session=` → check_session "No session token".
    #[test]
    fn merge_preserves_both_set_cookie_values() {
        let mut upstream = HeaderMap::new();
        upstream.insert(header::CONTENT_TYPE, HeaderValue::from_static("application/json"));
        // Simulate what a naive proxy might put in a map (only one visible via get);
        // we pass both cookies explicitly the way get_all would.
        let cookies = [
            HeaderValue::from_static(
                "session=tok-abc; HttpOnly; SameSite=Strict; Path=/; Max-Age=86400",
            ),
            HeaderValue::from_static("passkey_session=; Path=/api/passkeys; HttpOnly; Max-Age=0"),
        ];

        let out = merge_proxied_response_headers(&upstream, cookies);

        let vals = cookie_vals(&out);
        assert_eq!(vals.len(), 2, "both Set-Cookie values must be forwarded; got {vals:?}");
        assert!(
            vals.iter().any(|v| v.starts_with("session=tok-abc")),
            "session cookie must be present; got {vals:?}"
        );
        assert!(
            vals.iter().any(|v| v.starts_with("passkey_session=")),
            "passkey_session clear cookie must be present; got {vals:?}"
        );
        assert_eq!(
            out.get(header::CONTENT_TYPE).and_then(|v| v.to_str().ok()),
            Some("application/json")
        );
    }

    #[test]
    fn collect_set_cookies_reads_all_appended_values() {
        let mut upstream = HeaderMap::new();
        upstream.append(header::SET_COOKIE, HeaderValue::from_static("session=a"));
        upstream.append(
            header::SET_COOKIE,
            HeaderValue::from_static("passkey_session=; Max-Age=0"),
        );

        let collected = collect_set_cookies(&upstream);
        assert_eq!(collected.len(), 2);

        let out = merge_proxied_response_headers(&HeaderMap::new(), collected);
        assert_eq!(cookie_vals(&out).len(), 2);
    }

    #[test]
    fn insert_would_drop_first_set_cookie_documenting_the_bug() {
        // Guard against "simplify" refactors that switch append → insert.
        let mut broken = HeaderMap::new();
        broken.insert(header::SET_COOKIE, HeaderValue::from_static("session=keep-me"));
        broken.insert(
            header::SET_COOKIE,
            HeaderValue::from_static("passkey_session=; Max-Age=0"),
        );
        assert_eq!(
            broken.get_all(header::SET_COOKIE).iter().count(),
            1,
            "HeaderMap::insert replaces same-name keys — this is the bug class"
        );
        assert!(
            broken
                .get(header::SET_COOKIE)
                .unwrap()
                .to_str()
                .unwrap()
                .starts_with("passkey_session="),
            "insert keeps only the last Set-Cookie (session cookie lost)"
        );
    }
}
