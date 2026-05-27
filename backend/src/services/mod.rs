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
pub mod subscription_service;              // GENERIC-04
pub mod external_integration_service;      // GENERIC-05
pub mod verification_service;              // GENERIC-06
pub mod realtime_service;                  // GENERIC-07
pub mod ai_task_service;                   // GENERIC-08

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
pub mod crm_validator;