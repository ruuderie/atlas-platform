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
