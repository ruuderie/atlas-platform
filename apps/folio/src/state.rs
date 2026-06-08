use axum::extract::FromRef;
use leptos_config::LeptosOptions;

/// Newtype wrappers to avoid conflicting `FromRef<AppState>` impls for `String`.
/// Axum derives blanket `FromRef` for the first `String` field; additional
/// `String` fields require distinct newtypes.
#[derive(Clone, Debug)]
pub struct AtlasApiUrl(pub String);

#[derive(Clone, Debug)]
pub struct PublicApiBaseUrl(pub String);

/// Shared state injected into every Axum handler and Leptos server fn context.
/// Folio does NOT hold a PgPool — all data access goes through the Atlas backend API
/// via reqwest in atlas_client.rs. This keeps Folio a pure SSR frontend.
#[derive(Clone, FromRef)]
pub struct AppState {
    pub leptos_options:      LeptosOptions,
    /// Internal cluster URL for SSR-side server fn → backend API calls.
    /// e.g. http://atlas-backend:8000 inside k8s.
    pub atlas_api_url:       AtlasApiUrl,
    /// Public HTTPS URL as seen from the browser, injected into window.__ENV__.
    /// e.g. https://api.dev.atlas.oply.co
    pub public_api_base_url: PublicApiBaseUrl,
}

impl AtlasApiUrl {
    pub fn as_str(&self) -> &str { &self.0 }
}

impl PublicApiBaseUrl {
    pub fn as_str(&self) -> &str { &self.0 }
}

/// Tenant context resolved from session or x-tenant-id header.
#[derive(Clone, Debug)]
pub struct TenantContext(pub Option<uuid::Uuid>);
