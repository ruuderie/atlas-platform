pub mod api_tests;
pub mod test_utils;
pub mod crm_tests;
pub mod template_tests;
pub mod ad_purchase_tests;
pub mod admin_tests;
pub mod account_tests;
pub mod crm_extended_tests;
pub mod feed_tests;
pub mod relational_dependencies_tests;
pub mod tenant_settings_tests;
pub mod billing_tests;
pub mod magic_link_tests;
pub mod telemetry_tests;
pub mod audit_tests;
pub mod webhook_tests;
pub mod anchor_pages_tests;
pub mod search_tests;
pub mod webauthn_registry_tests;
// Admin Module Registry — Phase 4 tests
pub mod admin_module_tests;
// Phase 2 — Provisioning API tests
pub mod provision_tests;

// Complete generics service layer tests
pub mod services_tests;

// G-27 ScorecardService integration tests
pub mod g27_scorecard_tests;

// G-31 LeadService + AccountService integration tests
pub mod lead_account_tests;

// Pure unit tests (no DB, no I/O) — fast feedback on service logic
pub mod unit;

// Landing Page Builder — integration tests (DB + HTTP)
pub mod landing_page_builder_tests;

// Domain provisioning — critical path tests (sidecar logic + API integration)
pub mod domain_provisioning_tests;

// Instance lifecycle — reset, soft-delete, and reprovision-domain endpoint tests
pub mod instance_lifecycle_tests;
