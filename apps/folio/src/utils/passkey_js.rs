// apps/folio/src/utils/passkey_js.rs
//
// WebAuthn / Passkey JS interop stub.
//
// The full WebAuthn flow is handled server-side via the backend's
// /api/passkeys/register/* endpoints. This module bridges the client-side
// navigator.credentials.create() call via wasm-bindgen.
//
// For now this is a compile-safe stub that returns an error, allowing
// the rest of the onboarding UI to be wired and tested. The actual
// WebAuthn implementation should call the browser API and return the
// serialized PublicKeyCredential JSON.

/// Trigger the browser's native passkey registration dialog.
/// Returns the serialized credential JSON on success.
pub async fn create_passkey() -> Result<String, String> {
    // TODO: implement via wasm-bindgen + web-sys navigator.credentials.create()
    // Stub: return error so the UI falls through to the skip path gracefully
    Err("WebAuthn JS bridge not yet implemented — use Skip for now.".to_string())
}
