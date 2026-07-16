//! First-party acquisition attribution helpers for Folio marketing pages.
//!
//! Feature flags (FlagService catalog; seeded by m20261103):
//! - `acquisition.dm_tracking` — backend gates G-20 capture (default on)
//! - `acquisition.open_signup` — when false, organic stays waitlist-only

use serde_json::{json, Value};

pub const FLAG_DM_TRACKING: &str = "acquisition.dm_tracking";
pub const FLAG_OPEN_SIGNUP: &str = "acquisition.open_signup";

const ANON_KEY: &str = "atlas_anon_id";

#[cfg(feature = "hydrate")]
fn query_map() -> std::collections::HashMap<String, String> {
    use std::collections::HashMap;
    let mut out = HashMap::new();
    if let Some(window) = web_sys::window() {
        if let Ok(search) = window.location().search() {
            let s = search.trim_start_matches('?');
            for pair in s.split('&') {
                if pair.is_empty() {
                    continue;
                }
                let mut it = pair.splitn(2, '=');
                let k = percent_decode(&it.next().unwrap_or("").replace('+', " "));
                let v = percent_decode(&it.next().unwrap_or("").replace('+', " "));
                if !k.is_empty() {
                    out.insert(k, v);
                }
            }
        }
    }
    out
}

#[cfg(feature = "hydrate")]
fn percent_decode(s: &str) -> String {
    let bytes = s.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'%' && i + 2 < bytes.len() {
            if let (Some(a), Some(b)) = (hex_nibble(bytes[i + 1]), hex_nibble(bytes[i + 2])) {
                out.push((a << 4) | b);
                i += 3;
                continue;
            }
        }
        out.push(bytes[i]);
        i += 1;
    }
    String::from_utf8_lossy(&out).into_owned()
}

#[cfg(feature = "hydrate")]
fn hex_nibble(c: u8) -> Option<u8> {
    match c {
        b'0'..=b'9' => Some(c - b'0'),
        b'a'..=b'f' => Some(c - b'a' + 10),
        b'A'..=b'F' => Some(c - b'A' + 10),
        _ => None,
    }
}

#[cfg(not(feature = "hydrate"))]
fn query_map() -> std::collections::HashMap<String, String> {
    std::collections::HashMap::new()
}

pub fn anonymous_id() -> String {
    #[cfg(feature = "hydrate")]
    {
        if let Some(window) = web_sys::window() {
            if let Ok(Some(storage)) = window.local_storage() {
                if let Ok(Some(existing)) = storage.get_item(ANON_KEY) {
                    if !existing.is_empty() {
                        return existing;
                    }
                }
                let id = uuid::Uuid::new_v4().to_string();
                let _ = storage.set_item(ANON_KEY, &id);
                return id;
            }
        }
    }
    String::new()
}

pub fn attribution_payload_fields() -> Value {
    let q = query_map();
    let get = |k: &str| q.get(k).cloned().filter(|s| !s.is_empty());
    let anon = anonymous_id();
    json!({
        "utm_source": get("utm_source"),
        "utm_medium": get("utm_medium"),
        "utm_campaign": get("utm_campaign"),
        "utm_content": get("utm_content"),
        "utm_term": get("utm_term"),
        "gclid": get("gclid"),
        "fbclid": get("fbclid"),
        "msclkid": get("msclkid"),
        "offer_code": get("offer_code").or_else(|| get("code")),
        "anonymous_id": if anon.is_empty() { Value::Null } else { Value::String(anon) },
    })
}

pub fn merge_into_waitlist_body(body: &mut Value) {
    if let Some(obj) = body.as_object_mut() {
        if let Some(attrs) = attribution_payload_fields().as_object() {
            for (k, v) in attrs {
                if !v.is_null() {
                    obj.insert(k.clone(), v.clone());
                }
            }
        }
    }
}
