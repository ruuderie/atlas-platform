use axum::extract::FromRef;
use leptos_config::LeptosOptions;

/// Shared state injected into every Axum handler and Leptos server fn context.
/// Folio does NOT hold a PgPool — all data access goes through the Atlas backend API
/// via reqwest in atlas_client.rs. This keeps Folio a pure SSR frontend.
#[derive(Clone, FromRef)]
pub struct AppState {
    pub leptos_options: LeptosOptions,
    /// Internal cluster URL for SSR-side server fn → backend API calls.
    /// e.g. http://atlas-backend:8000 inside k8s.
    pub atlas_api_url: String,
    /// Public HTTPS URL as seen from the browser, injected into window.__ENV__.
    /// e.g. https://api.dev.atlas.oply.co
    pub public_api_base_url: String,
}

/// Tenant context resolved from session or x-tenant-id header.
#[derive(Clone, Debug)]
pub struct TenantContext(pub Option<uuid::Uuid>);
