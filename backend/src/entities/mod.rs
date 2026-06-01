#![allow(dead_code, unused_imports)]
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

// GENERIC-03: Payments ledger + credentials (provider-agnostic)
pub mod atlas_ledger_entry;
pub mod atlas_ledger_split;
pub mod atlas_payment_credential;

// GENERIC-01: Spatial / PostGIS (geo service areas)
pub mod geo_service_area;

// GENERIC-05: External integrations gateway
pub mod atlas_external_integration;
pub mod atlas_integration_event;

// GENERIC-06: Verification queue (human + automated trust)
pub mod atlas_verification_request;

// GENERIC-07: Real-time WebSocket rooms + messages
pub mod atlas_ws_room;
pub mod atlas_ws_message;

// GENERIC-04: B2C recurring subscriptions
pub mod atlas_subscription;

// GENERIC-08: Async AI / LLM task queue
pub mod atlas_ai_task;
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

// Unified Account + Contact (replaces legacy customer/contact)
pub mod atlas_account;
pub mod atlas_contact;

// GENERIC-31: Canonical lead / prospect entity
pub mod atlas_lead;

// GENERIC-27: Atlas Scorecards — Universal Structured Evaluation Engine
// 10 entity files covering all 11 tables (composite-PK tables share one file each).
pub mod atlas_scorecard_template;
pub mod atlas_scorecard_dimension;
pub mod atlas_scorecard_dimension_option;
pub mod atlas_scorecard;
pub mod atlas_rating_session;
pub mod atlas_scorecard_entry;
pub mod atlas_scorecard_dimension_aggregate;
pub mod atlas_scorecard_poll_aggregate;
pub mod atlas_scorecard_time_series;

// GENERIC-28: atlas_note — Universal Polymorphic Note (promotes `notes` table)
pub mod atlas_note;

// GENERIC-29: atlas_activity — Universal Polymorphic Activity Log (promotes `activity` table)
pub mod atlas_activity;

// GENERIC-27: The Combinator — target profiles + per-dimension criteria
pub mod atlas_scorecard_target;
pub mod atlas_scorecard_target_criteria;
