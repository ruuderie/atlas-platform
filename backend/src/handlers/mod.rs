pub use validator::Validate;
//CRM
pub mod activities;
pub mod cases;
pub mod contacts;
pub mod crm_status_options;
pub mod customers;
pub mod deals;
pub mod files;
pub mod leads;
pub mod notes;

//Admin
pub mod accounts;
pub mod ad_purchases;
pub mod app_instance;
pub mod app_menus;
pub mod app_pages;
pub mod categories;
pub mod landing_pages; // Platform-admin Landing Page Builder (app-scoped, no tenant)
pub mod tenant;

pub mod admin;
pub mod forms;
pub mod health;
pub mod listings;
pub mod passkeys;
pub mod profiles;
pub mod request_logs;
pub mod sessions;
pub mod templates;
pub mod user_accounts;
pub mod users;
// Admin module registry
pub mod ab_testing;
pub mod admin_modules;
pub mod admin_provision;
pub mod anchor;
pub mod app_seeds;
pub mod audit_logs;
pub mod auth_frontend;
pub mod communications;
pub mod feed_items;
pub mod feeds;
pub mod my_accounts;
pub mod onboarding;
pub mod otp; // Inline OTP auth — wizard pre-step (send + verify)
pub mod search;
pub mod setup;
pub mod telemetry;
pub mod version;

// G-27 Scorecard
pub mod scorecard_admin;
pub mod scorecard_analytics; // Phase 3 — portfolio analytics, leaderboard, anomalies
pub mod scorecard_display_rules;
pub mod scorecard_entries; // Phase 1 — platform-admin REST (explicit tenant_id)

// Folio — Property Management App
pub mod folio;

// G01 — PostGIS spatial query routes (radius, nearest, containment)
pub mod geo;

// G06 — Verification Queue
pub mod verification;

// G07 — WebSocket Relay (realtime rooms, broadcast, message persistence)
pub mod ws;

// G-32 — Platform-generic RBAC management API (role assignment, inspection)
pub mod rbac;

// PRODUCT LAUNCH ENGINE — public zero-auth endpoints
pub mod pub_products; // Product pages, variant pages, waitlist, pre-order, sitemap, view-count
pub mod pub_resolve; // Domain resolver: folio.app, miami.folio.app → product/variant context

// PLATFORM-GENERIC SYNDICATION ADMIN
pub mod syndication_admin; // Offer catalog CRUD + active link management + auto-provision

// G-36 — Platform-admin Programs API
pub mod programs_admin;
