pub mod tenant;         // TenantContext — platform-generic tenant + UserRole resolution
pub mod folio_role;     // Folio role extractors: VendorOnly, LandlordOnly, TenantOnly, etc.
pub mod app_config;     // AppDeploymentConfig — platform-generic app mode/config extractor (G-33)
pub mod client_context; // ClientContext — PMC client account scope via X-Folio-Client-Account header
