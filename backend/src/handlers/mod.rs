pub use validator::Validate;
//CRM
pub mod cases;
pub mod activities;
pub mod deals;
pub mod leads;
pub mod customers;
pub mod contacts;
pub mod files;
pub mod notes;
pub mod crm_status_options;

//Admin
pub mod ad_purchases;
pub mod accounts;
pub mod categories;
pub mod tenant;
pub mod app_instance;
pub mod app_pages;
pub mod landing_pages; // Platform-admin Landing Page Builder (app-scoped, no tenant)
pub mod app_menus;

pub mod templates;
pub mod listings;
pub mod profiles;
pub mod user_accounts;
pub mod users;
pub mod forms;
pub mod passkeys;
pub mod admin;
pub mod sessions;
pub mod request_logs;
pub mod health;
// Admin module registry
pub mod admin_modules;
pub mod auth_frontend;
pub mod my_accounts;
pub mod ab_testing;
pub mod feeds;
pub mod feed_items;
pub mod communications;
pub mod setup;
pub mod search;
pub mod telemetry;
pub mod audit_logs;
pub mod anchor;
pub mod app_seeds;
pub mod onboarding;
pub mod version;
pub mod admin_provision;

// G-27 Scorecard
pub mod scorecard_entries;
pub mod scorecard_display_rules;
pub mod scorecard_analytics;    // Phase 3 — portfolio analytics, leaderboard, anomalies

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
pub mod pub_products;   // Product pages, variant pages, waitlist, pre-order, sitemap, view-count
pub mod pub_resolve;    // Domain resolver: folio.app, miami.folio.app → product/variant context

// PLATFORM-GENERIC SYNDICATION ADMIN
pub mod syndication_admin;  // Offer catalog CRUD + active link management + auto-provision
