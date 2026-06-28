//! Folio PMC handlers — mod.rs
//!
//! Sub-router for all `/api/folio/pm/*` endpoints.
//! All routes in this module require:
//!   1. A valid authenticated session (applied by the outer auth middleware)
//!   2. `FolioRole::PropertyManager` role (enforced by `require_property_manager`)
//!   3. Folio app-specific config has `"pmc_enabled": true` (enforced by
//!      `PropertyManagerOnly` extractor — checked inside require_property_manager)

pub mod clients;
pub mod client_detail;
pub mod analytics;
pub mod app_config;
pub mod invite;   // POST /api/folio/pm/clients/:account_id/invite
pub mod onboard;  // POST /api/folio/pm/onboard  (public, token-gated)
