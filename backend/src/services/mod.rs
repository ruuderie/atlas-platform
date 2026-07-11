pub mod billing;
pub mod dns;
pub mod telephony;
pub mod tenant;

pub mod account_service;
pub mod contact_service;
pub mod lead_billing; // Legacy - to be migrated (now thin facade over ledger)
pub mod lead_service; // GENERIC-31
pub mod ledger; // New unified ledger service (G-03 + unification)
pub mod scorecard_analytics_service; // GENERIC-27 Phase 3 — portfolio analytics
pub mod scorecard_service; // GENERIC-27
pub mod scorecard_triggers;
pub mod search_sync;
pub mod telemetry;
pub mod webhook; // GENERIC-27 app-instance trigger → session

// === Complete Platform Generics Service Layer (v2) ===
pub mod application_service;
pub mod asset_service; // GENERIC-10
pub mod case_service; // GENERIC-13
pub mod contract_service; // GENERIC-11
pub mod document_service; // GENERIC-14
pub mod opportunity_service; // GENERIC-15
pub mod portfolio_service; // GENERIC-09
pub mod regulatory_registration_service; // GENERIC-16
pub mod service_provider_service; // GENERIC-12
pub mod tax_service; // GENERIC-17 // GENERIC-18
// === Platform Generics Round 1 Gap Fills (June 2026) ===
pub mod ai_task_service; // GENERIC-08
pub mod external_integration_service; // GENERIC-05
pub mod flag_service; // Feature flag resolution (instance → tenant → global)
pub mod geo_service;
pub mod notification_service; // GENERIC-07 ext: multi-channel notification dispatch
pub mod program_service; // GENERIC-36: growth/incentive programs
pub mod realtime_service; // GENERIC-07
pub mod reservation_service; // GENERIC-23 (+ background worker)
pub mod subscription_service; // GENERIC-04
pub mod verification_service; // GENERIC-06 // GENERIC-01: Spatial context

// === Folio — Property Management App ===
pub mod pm; // Domain services (zero net-new tables)

pub mod audit;
pub mod auth_service;
pub mod billing_service;
pub mod data_sync;
pub mod unification_data_migration;
pub mod user_service;
// Platform module provisioning
pub mod crm_validator;
pub mod ingress_provisioner;
pub mod module_provisioning;
pub mod outbox_worker;
pub mod product_localization;
pub mod rbac; // G-32: platform-generic RBAC — role resolution, assignment, permission checks
pub mod syndication_event_bus; // G-05 Syndication Event Bus // Product Launch Engine: AI-powered variant localization via G-08
