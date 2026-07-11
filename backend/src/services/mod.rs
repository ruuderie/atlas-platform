pub mod tenant;
pub mod telephony;
pub mod billing;
pub mod dns;

pub mod search_sync;
pub mod telemetry;
pub mod webhook;
pub mod lead_billing; // Legacy - to be migrated (now thin facade over ledger)
pub mod ledger;           // New unified ledger service (G-03 + unification)
pub mod account_service;
pub mod contact_service;
pub mod lead_service;                      // GENERIC-31
pub mod scorecard_service;                 // GENERIC-27
pub mod scorecard_analytics_service;      // GENERIC-27 Phase 3 — portfolio analytics
pub mod scorecard_triggers;               // GENERIC-27 app-instance trigger → session

// === Complete Platform Generics Service Layer (v2) ===
pub mod portfolio_service;                 // GENERIC-09
pub mod asset_service;                     // GENERIC-10
pub mod contract_service;                  // GENERIC-11
pub mod service_provider_service;          // GENERIC-12
pub mod case_service;                      // GENERIC-13
pub mod document_service;                  // GENERIC-14
pub mod opportunity_service;               // GENERIC-15
pub mod regulatory_registration_service;   // GENERIC-16
pub mod tax_service;                       // GENERIC-17
pub mod application_service;               // GENERIC-18
// === Platform Generics Round 1 Gap Fills (June 2026) ===
pub mod reservation_service;               // GENERIC-23 (+ background worker)
pub mod subscription_service;              // GENERIC-04
pub mod external_integration_service;      // GENERIC-05
pub mod verification_service;              // GENERIC-06
pub mod realtime_service;                  // GENERIC-07
pub mod notification_service;              // GENERIC-07 ext: multi-channel notification dispatch
pub mod ai_task_service;                   // GENERIC-08
pub mod geo_service;                       // GENERIC-01: Spatial context

// === Folio — Property Management App ===
pub mod pm;                                // Domain services (zero net-new tables)

pub mod unification_data_migration;
pub mod audit;
pub mod user_service;
pub mod auth_service;
pub mod billing_service;
pub mod data_sync;
// Platform module provisioning
pub mod module_provisioning;
pub mod ingress_provisioner;
pub mod outbox_worker;
pub mod syndication_event_bus;     // G-05 Syndication Event Bus
pub mod crm_validator;
pub mod rbac;              // G-32: platform-generic RBAC — role resolution, assignment, permission checks
pub mod product_localization; // Product Launch Engine: AI-powered variant localization via G-08