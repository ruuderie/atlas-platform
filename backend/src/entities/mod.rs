//CRM
pub mod lead;
pub mod deal;
pub mod deal_contact;
pub mod customer;
pub mod contact;
pub mod activity;
pub mod case;
pub mod file;
pub mod file_association;
pub mod note;
pub mod lead_charge;
pub mod crm_status_option;


//DIRECTORIES
pub mod user;
pub mod profile;
pub mod user_account;
pub mod template;
pub mod category;
pub mod account;
pub mod ad_purchase;
pub mod tenant;
pub mod listing;
pub mod session;
pub mod request_log;
pub mod listing_ab_test;
pub mod listing_ab_variant;

//NEW ENTITIES
pub mod feed;
pub mod feed_item;
pub mod attachment;
pub mod passkey;
pub mod magic_link_token;
pub mod tenant_setting;
pub mod audit_log;
// MULTI-TENANT ARCHITECTURE
pub mod app_instance;
pub mod app_instance_module;
pub mod app_domain;
pub mod app_page;
pub mod app_menu;
pub mod global_search_index;

// BILLING & MONETIZATION
pub mod billing_plan;
pub mod tenant_subscription;
pub mod transaction;

// TELEMETRY & ANALYTICS
pub mod telemetry_events;
pub mod platform_metrics_daily;

// DEVELOPER CONSOLE
pub mod api_token;
pub mod webhook_endpoint;
pub mod webhook_delivery;

// ANCHOR APP LEGACY
pub mod page_view;
pub mod bitcoin_block;
pub mod tenant_background_job;

// ONBOARDING
pub mod onboarding_progress;
pub mod user_app_permission;

// WEBAUTHN SESSION PERSISTENCE
pub mod webauthn_challenge;

// DISTRIBUTED TRANSACTIONAL OUTBOX
pub mod outbox_job;

// PLATFORM GENERICS v2 (GENERIC-09+)
pub mod atlas_portfolio;

// GENERIC-02: Vault extensions (share tokens + multipart)
pub mod attachment_share_token;
pub mod attachment_multipart_upload;
pub mod atlas_asset;
pub mod atlas_contract;
pub mod atlas_case;
pub mod atlas_document;
pub mod atlas_service_provider;
pub mod atlas_opportunity;
pub mod atlas_regulatory_registration;
pub mod atlas_tax_event;
pub mod atlas_tax_filing;
pub mod atlas_application;
