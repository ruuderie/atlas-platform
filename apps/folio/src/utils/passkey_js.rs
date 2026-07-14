// apps/folio/src/utils/passkey_js.rs
//
// WebAuthn / Passkey JS interop — real implementation via inline JS.
//
// Strategy: use `#[wasm_bindgen(inline_js)]` to write the browser-side
// logic in JavaScript. This is the most robust approach for WebAuthn because:
//   - Web-sys typed bindings for ArrayBuffer/PublicKey types are incomplete
//   - JS handles base64url ↔ ArrayBuffer conversion natively
//   - .toJSON() and fallback serialization is cleaner in JS
//
// SSR stub: all browser code is gated on `#[cfg(feature = "hydrate")]`.
// The public API (`create_passkey`, `authenticate_passkey`) compiles in all
// feature configurations and returns Err on the SSR path.
//
// Flow:
//   Registration:
//     1. POST /api/passkeys/start-register  → CreationChallengeResponse JSON
//     2. webauthn_create_js(challenge_json) → browser dialog → credential JSON
//     3. POST /api/passkeys/finish-register (credential JSON) → 200 OK
//
//   Authentication:
//     1. POST /api/passkeys/start-login     → RequestChallengeResponse JSON
//     2. webauthn_get_js(challenge_json)    → browser dialog → assertion JSON
//     3. POST /api/passkeys/finish-login    (assertion JSON)  → session cookie

// ── Browser-only JS glue ──────────────────────────────────────────────────────

#[cfg(feature = "hydrate")]
#[wasm_bindgen::prelude::wasm_bindgen(inline_js = r#"

// ── Helpers ──────────────────────────────────────────────────────────────────

function b64urlDecode(str) {
    // Normalise base64url → base64, then decode to Uint8Array
    const b64 = str.replace(/-/g, '+').replace(/_/g, '/');
    const padded = b64.padEnd(b64.length + (4 - b64.length % 4) % 4, '=');
    const binary = atob(padded);
    const bytes = new Uint8Array(binary.length);
    for (let i = 0; i < binary.length; i++) bytes[i] = binary.charCodeAt(i);
    return bytes.buffer;
}

function b64urlEncode(buf) {
    const bytes = new Uint8Array(buf);
    let binary = '';
    for (let i = 0; i < bytes.byteLength; i++) binary += String.fromCharCode(bytes[i]);
    return btoa(binary).replace(/\+/g, '-').replace(/\//g, '_').replace(/=/g, '');
}

function serializeCredential(cred) {
    // Use .toJSON() where available (Chrome 132+, Safari 18+)
    if (typeof cred.toJSON === 'function') {
        return JSON.stringify(cred.toJSON());
    }
    // Manual fallback for older browsers
    const r = cred.response;
    const obj = {
        id: cred.id,
        rawId: b64urlEncode(cred.rawId),
        type: cred.type,
        response: {
            clientDataJSON: b64urlEncode(r.clientDataJSON),
        }
    };
    if (r.attestationObject) {
        obj.response.attestationObject = b64urlEncode(r.attestationObject);
    }
    if (r.authenticatorData) {
        obj.response.authenticatorData = b64urlEncode(r.authenticatorData);
    }
    if (r.signature) {
        obj.response.signature = b64urlEncode(r.signature);
    }
    if (r.userHandle) {
        obj.response.userHandle = b64urlEncode(r.userHandle);
    }
    return JSON.stringify(obj);
}

// ── Registration ─────────────────────────────────────────────────────────────

export async function webauthn_create_js(challenge_json) {
    const opts = JSON.parse(challenge_json);
    const pk = opts.publicKey;

    // Decode ArrayBuffer fields
    pk.challenge = b64urlDecode(pk.challenge);
    if (pk.user && pk.user.id) pk.user.id = b64urlDecode(pk.user.id);
    if (pk.excludeCredentials) {
        pk.excludeCredentials = pk.excludeCredentials.map(c => ({
            ...c, id: b64urlDecode(c.id)
        }));
    }

    const cred = await navigator.credentials.create({ publicKey: pk });
    if (!cred) throw new Error('No credential returned from browser');
    return serializeCredential(cred);
}

// ── Authentication ────────────────────────────────────────────────────────────

export async function webauthn_get_js(challenge_json) {
    const opts = JSON.parse(challenge_json);
    const pk = opts.publicKey;

    pk.challenge = b64urlDecode(pk.challenge);
    if (pk.allowCredentials) {
        pk.allowCredentials = pk.allowCredentials.map(c => ({
            ...c, id: b64urlDecode(c.id)
        }));
    }

    const cred = await navigator.credentials.get({ publicKey: pk });
    if (!cred) throw new Error('No credential returned from browser');
    return serializeCredential(cred);
}

"#)]
extern "C" {
    /// Call the browser's `navigator.credentials.create()` API.
    /// `challenge_json` is the raw JSON string from `POST /api/passkeys/start-register`.
    /// Returns a Promise<String> resolving to the serialized credential JSON.
    #[wasm_bindgen::prelude::wasm_bindgen(catch)]
    async fn webauthn_create_js(
        challenge_json: &str,
    ) -> Result<wasm_bindgen::JsValue, wasm_bindgen::JsValue>;

    /// Call the browser's `navigator.credentials.get()` API.
    /// `challenge_json` is the raw JSON string from `POST /api/passkeys/start-login`.
    /// Returns a Promise<String> resolving to the serialized assertion JSON.
    #[wasm_bindgen::prelude::wasm_bindgen(catch)]
    async fn webauthn_get_js(
        challenge_json: &str,
    ) -> Result<wasm_bindgen::JsValue, wasm_bindgen::JsValue>;
}

// ── Public API ────────────────────────────────────────────────────────────────

/// Trigger the browser's native passkey registration dialog.
///
/// 1. Fetches a creation challenge from `POST /api/passkeys/start-register`.
/// 2. Calls `navigator.credentials.create()` via the inline JS bridge.
/// 3. POSTs the credential to `POST /api/passkeys/finish-register`.
///
/// Returns the server's finish-register response body on success.
pub async fn create_passkey() -> Result<String, String> {
    #[cfg(feature = "hydrate")]
    {
        // Step 1: Get challenge
        let challenge_json = post_json("/api/passkeys/start-register", "{}").await?;

        // Step 2: Browser dialog
        let credential_json = webauthn_create_js(&challenge_json)
            .await
            .map_err(|e| map_js_error(e))?
            .as_string()
            .ok_or_else(|| "Unexpected non-string result from WebAuthn bridge".to_string())?;

        // Step 3: Finish registration
        let response = post_json("/api/passkeys/finish-register", &credential_json).await?;
        Ok(response)
    }

    #[cfg(not(feature = "hydrate"))]
    {
        Err("WebAuthn is a browser-only API.".to_string())
    }
}

/// Trigger the browser's native passkey authentication prompt.
///
/// 1. Fetches an assertion challenge from `POST /api/passkeys/start-login`.
/// 2. Calls `navigator.credentials.get()` via the inline JS bridge.
/// 3. POSTs the assertion to `POST /api/passkeys/finish-login`.
///
/// Returns the server's finish-login response body on success.
pub async fn authenticate_passkey() -> Result<String, String> {
    #[cfg(feature = "hydrate")]
    {
        let challenge_json = post_json("/api/passkeys/start-login", "{}").await?;

        let assertion_json = webauthn_get_js(&challenge_json)
            .await
            .map_err(|e| map_js_error(e))?
            .as_string()
            .ok_or_else(|| "Unexpected non-string result from WebAuthn bridge".to_string())?;

        let response = post_json("/api/passkeys/finish-login", &assertion_json).await?;
        Ok(response)
    }

    #[cfg(not(feature = "hydrate"))]
    {
        Err("WebAuthn is a browser-only API.".to_string())
    }
}

// ── Helpers (hydrate-only) ────────────────────────────────────────────────────

/// POST JSON to a same-origin URL via gloo-net fetch, forwarding session cookies.
#[cfg(feature = "hydrate")]
async fn post_json(url: &str, body: &str) -> Result<String, String> {
    use gloo_net::http::Request;

    let response = Request::post(url)
        .header("Content-Type", "application/json")
        .body(body)
        .map_err(|e| format!("Request build error: {e}"))?
        .send()
        .await
        .map_err(|e| format!("Network error: {e}"))?;

    if !response.ok() {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        let detail = body.trim();
        if detail.is_empty() {
            return Err(format!("Server returned HTTP {status}"));
        }
        // Keep the message short for UI; full body is still useful for debugging.
        let clipped = if detail.len() > 180 {
            format!("{}…", &detail[..180])
        } else {
            detail.to_string()
        };
        return Err(format!("Server returned HTTP {status}: {clipped}"));
    }

    response
        .text()
        .await
        .map_err(|e| format!("Failed to read response: {e}"))
}

/// Map a JS error value to a user-friendly string.
/// Handles common WebAuthn DOMException names.
#[cfg(feature = "hydrate")]
fn map_js_error(e: wasm_bindgen::JsValue) -> String {
    let raw = e
        .as_string()
        .or_else(|| {
            // Try to extract .message from the error object
            use wasm_bindgen::JsValue;
            js_sys::Reflect::get(&e, &JsValue::from_str("message"))
                .ok()
                .and_then(|v| v.as_string())
        })
        .unwrap_or_else(|| format!("{:?}", e));

    if raw.contains("NotAllowedError") {
        "The biometric prompt was cancelled or timed out. Please try again.".to_string()
    } else if raw.contains("SecurityError") {
        "Security error: ensure you are on a secure (HTTPS) connection.".to_string()
    } else if raw.contains("InvalidStateError") {
        "A passkey already exists for this account on this device.".to_string()
    } else if raw.contains("NotSupportedError") {
        "Your device does not support passkeys. Please use a passkey-capable device.".to_string()
    } else {
        format!("Passkey error: {raw}")
    }
}
