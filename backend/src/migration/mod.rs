use sea_orm_migration::prelude::*;

// ═══════════════════════════════════════════════════════════════════════════════
// CORE PLATFORM MIGRATION MODULES
// These migrations handle platform-level schema: users, accounts, sessions,
// directories, CRM, tenants, billing, telemetry, and other shared infrastructure.
//
// RULE: Only platform schema belongs here.
// App-specific content/seed migrations belong in each AtlasApp::migrations() impl.
// ═══════════════════════════════════════════════════════════════════════════════

// --- Legacy schema (pre-2024) ---
pub mod m20230911_create_accounts_table;
pub mod m20230911_create_sessions_table;
pub mod m20230912_000000_create_users_table;
pub mod m20230912_000001_create_user_accounts_table;
pub mod m20230913_create_directory_types_table;
pub mod m20230914_create_directories_table;
pub mod m20230915_create_profiles_table;
pub mod m20230916_create_categories_table;
pub mod m20230917_create_templates_table;
pub mod m20230918_create_listings_table;
pub mod m20230919_create_listing_attributes_table;
pub mod m20230920_create_ad_purchases_table;
pub mod m20240001_update_timestamp_migration;
pub mod m20240315_add_directory_domain_fields;
pub mod m20240922_create_crm_tables;
pub mod m20240922_create_request_log_table;
pub mod m20240923_create_feed_tables;
pub mod m20240924_update_directory_multisite_fields;
pub mod m20240924_update_listings_nullable_category;
pub mod m20241001_add_icon_and_slug_to_categories;
pub mod m20241002_add_directory_id_to_crm_and_categories;
pub mod m20241003_add_slug_to_listings;
pub mod m20250101_create_ab_testing_tables;

// --- Core platform schema (2026) ---
pub mod m20260320_create_passkeys_table;
pub mod m20260324_000001_collapse_eav_to_jsonb;
pub mod m20260326_add_service_area_to_profile;
pub mod m20260326_add_stripe_fields_to_account;
pub mod m20260326_create_lead_charges_table;
pub mod m20260404_000000_core_tenant_shift;
pub mod m20260404_000001_create_cms_tables;
pub mod m20260404_000002_anchor_seed;
pub mod m20260404_000005_create_magic_links;
pub mod m20260404_000006_create_tenant_settings;
pub mod m20260405_000001_rename_directory_to_network;
pub mod m20260406_000000_create_global_search;
pub mod m20260406_000001_create_billing_tables;
pub mod m20260406_000002_create_telemetry_tables;
pub mod m20260406_000003_create_developer_console_tables;
pub mod m20260408_000000_create_audit_logs;
pub mod m20260408_000000_fix_tenant_app_alignments;
pub mod m20260408_000001_fix_uat_app_domains;

// ═══════════════════════════════════════════════════════════════════════════════
// ANCHOR APP MIGRATION MODULES (pub mod declarations only — not in base vec)
// These are declared here so Rust can resolve them via crate::migration::*,
// but they are registered exclusively in AnchorApp::migrations(), not below.
// ═══════════════════════════════════════════════════════════════════════════════
pub mod m20260408_000002_create_anchor_legacy_tables;
pub mod m20260408_000003_seed_anchor_background_jobs;
pub mod m20260408_000004_fix_anchor_tables_and_seed;
pub mod m20260408_000006_create_app_content;
pub mod m20260412_000001_form_engine;
pub mod m20260412_000002_add_tenant_slug;
pub mod m20260412_000003_seed_oplystusa;
pub mod m20260413_000001_seed_oplystusa_domains;
pub mod m20260415_000001_seed_oplystusa_home_page;
pub mod m20260415_000002_upgrade_buildwithruud_home_page;
pub mod m20260415_000003_seed_oplystusa_pages;
pub mod m20260416_000001_rename_resume_tables;
pub mod m20260416_000002_seed_buildwithruud_block_pages;
pub mod m20260417_000001_seed_design_system_config;
pub mod m20260417_000002_fix_buildwithruud_pages;
pub mod m20260417_000003_seed_formbuilder_pages;
pub mod m20260425_000001_update_buildwithruud_home;
pub mod m20260425_000002_create_footer_items_table;
pub mod m20260425_000003_fix_buildwithruud_padding;
pub mod m20260425_000004_stitch_ruuderie_payload;
pub mod m20260425_000005_fix_ruud_tenant_lookup;
pub mod m20260425_000006_force_ruud_payload;
pub mod m20260426_000001_hardened_ruud_payload;
pub mod m20260427_000001_restore_consulting_page;
pub mod m20260427_000002_real_estate_ventures_redesign;
pub mod m20260427_000003_widget_instance_config;
pub mod m20260427_000005_seed_p_vs_np_blog_post;
pub mod m20260427_000006_real_estate_newsletter_form;
pub mod m20260427_000007_seed_kami_resume_profile;
pub mod m20260427_000008_fix_p_vs_np_math_delimiters;
pub mod m20260427_000009_blog_download_leads;
pub mod m20260427_000010_enable_kami_mode_buildwithruud;
pub mod m20260427_000011_kami_projects_layout;

// ONBOARDING SYSTEM
pub mod m20260429_000001_create_onboarding_progress;
pub mod m20260430_000001_drop_anchor_legacy_tables;

// DATA INTEGRITY FIXES
// Canonicalizes tenant_setting from app_instances.settings — fixes UAT content gap (2026-04-30)
pub mod m20260501_000001_canonicalize_tenant_settings;
pub mod m20260502_000001_seed_app_content_resume;

pub mod m20260504_000001_create_user_app_permission;
pub mod m20260504_000002_remove_is_admin_from_user;
pub mod m20260504_000003_seed_platform_sentinel_account;
pub mod m20260507_000001_add_redirect_url_to_magic_link;
pub mod m20260507_000002_add_is_setup_to_magic_link;
// 2026-05-14: These two migrations were applied to the dev DB (in de583e29) but the
// backend Docker image was never rebuilt afterward — all subsequent commits only
// touched apps/ or .woodpecker.yml. This comment triggers a backend rebuild so the
// atlas-dev pod boots with the correct migration manifest and stops CrashLoopBackOff.
pub mod m20260513_000001_add_dev_domain_buildwithruud;
pub mod m20260513_000002_unique_active_magic_link_per_user;
// 2026-05-15: Add SHA-256 hash columns for bearer/refresh tokens (Security #7).
pub mod m20260515_000001_hash_session_tokens;
// 2026-05-17: Bug 4 — ensure mailing_list table has the correct schema
// (preferences JSONB, tenant_id UUID). The legacy drop migration was never
// registered, so the table may exist with an old schema missing these columns.
pub mod m20260517_000001_ensure_mailing_list_schema;
// 2026-05-18: Dynamic Admin Module Registry — creates app_instance_module table
// and seeds buildwithruud's existing module set for backward compatibility.
pub mod m20260518_000001_create_app_instance_modules;
// 2026-05-22: Create webauthn_challenges table to persist session/challenge state in PostgreSQL.
pub mod m20260522_000001_create_webauthn_challenges;
// 2026-05-23: Create outbox_job table to support Transactional Outbox Pattern
pub mod m20260523_000001_create_outbox_jobs;
pub mod m20260523_000002_extend_lead_table;
pub mod m20260523_000003_create_crm_status_options;
pub mod m20260523_000004_ensure_crm_modules;
pub mod m20260523_000005_create_crm_activity_and_deep_fields;
pub mod m20260523_000006_create_headless_email_tables;
pub mod m20260524_000001_extend_crm_avatar_attachments;
pub mod m20260525_000001_extend_notes_and_activities;

// --- Platform Generics v2 (Round 2/3 domain objects) ---
// These are registered via CorePlatformApp::migrations() for proper encapsulation.
// See docs/architecture/platform_generics_v2.md
pub mod m20260601_g09_portfolios;
pub mod m20260601_g10_assets;
pub mod m20260601_g11_contracts;
pub mod m20260601_g12_service_providers;
pub mod m20260601_g13_cases;
pub mod m20260601_g14_documents;
pub mod m20260601_g15_opportunities;
pub mod m20260601_g16_regulatory_registrations;
pub mod m20260601_g17_tax;
pub mod m20260601_g18_applications;

// --- Platform Generics Round 1 Gap Fills (G-19, G-23, G-25, G-26) ---
// Identified via horizontal gap analysis (June 2026). Promoted to generics
// before Direct Booking Engine and CoverFlow wrote conflicting app-specific tables.
// See docs/architecture/platform_generics_gap_analysis.md
//
// ORDERING NOTE: These use the m20260701_ prefix, which sorts AFTER all m20260601_
// generics alphabetically, ensuring G-01–G-18 are applied before G-19+.
//
// G-19: Multi-channel campaign management (atlas_campaigns, atlas_sequence_steps,
//        atlas_campaign_enrollments, atlas_campaign_events)
pub mod m20260701_g19_campaigns;
// G-23: Time-bounded reservation with inventory hold (atlas_reservations)
//        !! REQUIRES: release_expired_holds background worker registration !!
pub mod m20260701_g23_reservations;
// G-25: Commission plan & split governance (atlas_commission_plans,
//        atlas_commission_plan_splits + backfills atlas_ledger_splits.commission_plan_id)
pub mod m20260701_g25_commission_plans;
// G-26: Product catalog, pricebook & availability grid (atlas_catalog_entries,
//        atlas_catalog_rate_rules, atlas_catalog_availability)
pub mod m20260701_g26_catalog;

// --- Original Infrastructure Generics (G-01 to G-08) ---
// Started with highest priority: G-02 atlas_vault
pub mod m20260601_g01_geo_postgis;
pub mod m20260601_g02_vault_extension;
pub mod m20260601_g03_payments;
pub mod m20260601_g05_external_integrations;
pub mod m20260601_g06_verification_queue;
pub mod m20260601_g04_subscriptions;
pub mod m20260601_g07_realtime;
pub mod m20260601_g08_ai_tasks;

// Unification: New canonical Account + Contact model (replaces legacy CRM)
pub mod m20260601_unify_accounts_contacts;

// GENERIC-31: Canonical lead / prospect entity
pub mod m20260601_g31_atlas_lead;

// Gap-fill: atlas_accounts + atlas_contacts firmographic promotion
// Sorts after G-31 (m20260702_ > m20260601_) so set_updated_at_column() exists.
pub mod m20260702_gap_fill_accounts_contacts;

// GENERIC-27: Atlas Scorecards — Universal Structured Evaluation Engine (11 tables)
// Sorts as m20260701_ alongside G-19/G-25/G-26. set_updated_at_column() from G-31 exists.
pub mod m20260701_g27_scorecards;

// GENERIC-28: atlas_note — Promotes `notes` table to platform generic.
// Adds note_type, visibility, is_pinned, parent_note_id, note_metadata.
pub mod m20260703_g28_atlas_note;

// GENERIC-29: atlas_activity — Promotes `activity` table to platform generic.
// Adds polymorphic subject, direction, outcome, duration, scheduled_at, activity_metadata.
pub mod m20260704_g29_atlas_activity;

// Drop legacy `lead` table + rename compat view.
// Requires G-31 (atlas_lead + atlas_lead_compat_view) to exist.
pub mod m20260705_drop_legacy_lead;

// G-27 background job seeding: registers recompute_scorecard_aggregates +
// refresh_scorecard_time_series for all tenants with active scorecard templates.
pub mod m20260706_seed_g27_background_jobs;

// G-27 gap fill: Add is_inverted to atlas_scorecard_dimensions.
// Enables low-score-is-better dimensions (timeline_slippage, competition_risk, etc.)
pub mod m20260707_g27_is_inverted;

// G-27 gap fill: Context-Aware Display Rules engine.
// Adds atlas_scorecard_display_rules — the "second axis" of G-27 configurability.
// Tier-gated: Professional+ tenants only (enforced in ScorecardService).
pub mod m20260708_g27_display_rules;

// G-27 Data Science Upgrade — Phase 1: Cold-start strategy + Bayesian prior weight
// Adds: cold_start_strategy, cold_start_saturation_threshold on templates;
//       bayesian_prior_weight on dimensions.
pub mod m20260709_g27_data_science_v1;

// G-27 Data Science Upgrade — Phase 2: Masked vectors + anomaly detection + calibration
// Adds: dimension_vector_v2 + has_data_mask on scorecards;
//       percentile_rank fields on aggregates;
//       z_score + is_anomaly + anomaly_direction on time_series;
//       atlas_scorecard_contributor_calibration table.
pub mod m20260710_g27_data_science_v2;
// G-27 Patch: Add deleted_at soft-delete column to atlas_scorecards.
// Sorts before v3 (m20260711_0_ < m20260711_g_) — column must exist when the
// portfolio analytics MV and v_scorecard_recent_anomalies views reference sc.deleted_at.
pub mod m20260711_0_g27_add_deleted_at_scorecards;
// G-27 Patch: Add deleted_at to atlas_scorecard_templates.
// Separate from m20260711_0_ because that migration already ran on UAT/dev.
// m20260711_1_ sorts after m20260711_0_ and before m20260711_g_ (v3 views + seed job).
pub mod m20260711_1_g27_add_deleted_at_templates;
// G-27 Data Science Upgrade Phase 3: mv_scorecard_portfolio_analytics materialized view
// + v_scorecard_recent_anomalies live view. Powers portfolio API + BYOC peer pool.
pub mod m20260711_g27_data_science_v3;
// G-27 Phase 3: Seed refresh_scorecard_portfolio background job (every 4 hours).
pub mod m20260712_g27_seed_portfolio_job;
// G-27 Phase 4: Seed calibrate_scorecard_contributors weekly background job.
pub mod m20260713_g27_seed_calibration_job;

// G-27 gap fill: Add display_config JSONB to atlas_scorecard_templates.
// This column was added to the entity and types in the type-safety refactor
// but the migration was never written. Required by all scorecard template inserts.
pub mod m20260714_g27_display_config;

// Phase 0 — PM G-27 prerequisites: template_scope + is_tenant_extension.
// Required before PropertyManagementApp::provision() seeds PM scorecard templates.
pub mod m20260801_pm_g27_template_scope;
// GENERIC-23: atlas_reservations — Time-bounded reservation with inventory hold.
// Unifies STR bookings, hotel rooms, equipment rentals, truck parking, appointments.
pub mod m20260802_g23_atlas_reservations;
pub mod m20260803_g26_atlas_catalog; // GENERIC-26: Product catalog, pricebook & availability grid
pub mod m20260804_g19_atlas_campaigns; // GENERIC-19: Multi-channel campaign management
pub mod m20260805_g20_atlas_attribution; // GENERIC-20: Multi-channel attribution touchpoints
pub mod m20260806_g21_atlas_events;      // GENERIC-21: Event management, ticketing & check-in
pub mod m20260807_g22_atlas_record_relationships; // GENERIC-22: Universal M:M junction table
pub mod m20260808_g24_atlas_quotes;               // GENERIC-24: Pre-purchase pricing proposals
pub mod m20260809_g26_catalog_forward;             // G26 forward: enum, GENERATED column, triggers, indexes
pub mod m20260810_add_folio_role_to_user_account;  // Folio multi-role: folio_role column on user_account
pub mod m20260811_g32_atlas_rbac;                  // G-32: atlas_role_profiles, atlas_user_app_roles, atlas_role_profile_permissions
pub mod m20260812_g32_folio_role_seed;             // G-32: platform-default Folio role profiles seed
pub mod m20260813_g32_migrate_folio_roles;         // G-32: backfill atlas_user_app_roles from folio_role column
pub mod m20260814_g32_drop_folio_role_column;      // G-32: drop user_account.folio_role (superseded by G-32)
pub mod m20260815_g33_app_deployment_config;       // G-33: atlas_app_deployment_config — platform-generic app mode config
// G-33/Folio: property_manager role profile + permissions seed (base vec — platform generic schema change)
pub mod m20260816_g33_folio_pmc_seed;
// Folio PMC: managed_account_id FK on contract/asset/portfolio/lead (base vec — platform generic schema change)
pub mod m20260817_folio_managed_account_id;
pub mod m20260818_folio_client_role_scope;         // Folio PMC: client_account_id scope FK on atlas_user_app_roles
pub mod m20260819_g34_vendor_marketplace;          // G-34: vendor marketplace opt-in columns on atlas_service_providers
pub mod m20260900_g10_asset_lifecycle;             // G-10: universal asset lifecycle extension (scheduled_service_date, expiry_date, condition, lifecycle_metadata)
pub mod m20260901_platform_products;              // Platform Admin: platform_products registry (Folio, Anchor, Network, Meridian) + deploy hooks
pub mod m20260902_app_instance_public_config;    // Platform Admin: public_slug + custom_domain + instance_status on atlas_app_deployment_config
pub mod m20260903_platform_products_launch_engine; // Product Launch Engine: launch_mode, pre-order, waitlist_count on platform_products
pub mod m20260904_product_page_variants;          // Product Launch Engine: product_page_templates + product_page_variants (programmatic SEO)
pub mod m20260905_product_domain_localization;    // Product Launch Engine: apex_domain, AI localization fields, product_domain_aliases
pub mod m20260906_subscription_grace_period;       // Billing Grace Period: adds is_billing_exempt, billing_exemption_reason, grace_period_ends_at
pub mod m20260907_feature_flags;                   // Feature Flags: feature_flags, flag_overrides, flag_audit_log tables
pub mod m20260908_platform_invitations;            // Platform Invitations: platform_invite table

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        // ═══════════════════════════════════════════════════════════════════
        // BASE VEC — Core platform schema migrations ONLY.
        //
        // DO NOT add app-specific content or seed migrations here.
        // Those belong exclusively in each AtlasApp::migrations() impl.
        //
        // This vec + app migrations are merged then sorted alphabetically
        // by migration name (filename prefix), which is why consistent
        // naming is critical for deterministic ordering.
        // ═══════════════════════════════════════════════════════════════════
        let mut base: Vec<Box<dyn MigrationTrait>> = vec![
            Box::new(m20230911_create_accounts_table::Migration),
            Box::new(m20230911_create_sessions_table::Migration),
            Box::new(m20230912_000000_create_users_table::Migration),
            Box::new(m20230912_000001_create_user_accounts_table::Migration),
            Box::new(m20230913_create_directory_types_table::Migration),
            Box::new(m20230914_create_directories_table::Migration),
            Box::new(m20240315_add_directory_domain_fields::Migration),
            Box::new(m20240924_update_directory_multisite_fields::Migration),
            Box::new(m20241002_add_directory_id_to_crm_and_categories::Migration),
            Box::new(m20240922_create_request_log_table::Migration),
            Box::new(m20240923_create_feed_tables::Migration),
            Box::new(m20240001_update_timestamp_migration::Migration),
            Box::new(m20250101_create_ab_testing_tables::Migration),
            Box::new(m20260320_create_passkeys_table::Migration),
            Box::new(m20260326_add_stripe_fields_to_account::Migration),
            Box::new(m20260404_000000_core_tenant_shift::Migration),
            Box::new(m20260404_000001_create_cms_tables::Migration),
            Box::new(m20260404_000002_anchor_seed::Migration),
            Box::new(m20260404_000005_create_magic_links::Migration),
            Box::new(m20260404_000006_create_tenant_settings::Migration),
            Box::new(m20260405_000001_rename_directory_to_network::Migration),
            Box::new(m20260406_000000_create_global_search::Migration),
            Box::new(m20260406_000001_create_billing_tables::Migration),
            Box::new(m20260406_000002_create_telemetry_tables::Migration),
            Box::new(m20260406_000003_create_developer_console_tables::Migration),
            Box::new(m20260408_000000_create_audit_logs::Migration),
            Box::new(m20260408_000000_fix_tenant_app_alignments::Migration),
            Box::new(m20260408_000001_fix_uat_app_domains::Migration),
            Box::new(m20260408_000006_create_app_content::Migration),
            Box::new(m20260412_000001_form_engine::Migration),
            Box::new(m20260412_000002_add_tenant_slug::Migration),
            Box::new(m20260412_000003_seed_oplystusa::Migration),
            Box::new(m20260413_000001_seed_oplystusa_domains::Migration),
            Box::new(m20260415_000001_seed_oplystusa_home_page::Migration),
            Box::new(m20260415_000002_upgrade_buildwithruud_home_page::Migration),
            Box::new(m20260415_000003_seed_oplystusa_pages::Migration),
            Box::new(m20260416_000001_rename_resume_tables::Migration),
            Box::new(m20260416_000002_seed_buildwithruud_block_pages::Migration),
            Box::new(m20260417_000001_seed_design_system_config::Migration),
            Box::new(m20260417_000002_fix_buildwithruud_pages::Migration),
            // ONBOARDING SYSTEM — must follow tenant shift and app_instances table
            Box::new(m20260429_000001_create_onboarding_progress::Migration),
            Box::new(m20260504_000001_create_user_app_permission::Migration),
            // Seeds the nil-UUID platform sentinel tenant + account required by toggle_admin
            // when granting PlatformSuperAdmin to a user with no existing user_account.
            Box::new(m20260504_000003_seed_platform_sentinel_account::Migration),
            // Adds redirect_url to magic_link_token for app-aware email link routing.
            Box::new(m20260507_000001_add_redirect_url_to_magic_link::Migration),
            // Adds is_setup_token to distinguish between first login and regular magic links.
            Box::new(m20260507_000002_add_is_setup_to_magic_link::Migration),
            // Registers dev.buildwithruud.com in app_domains so the tenant-resolution
            // middleware resolves TenantContext from the Host header on fresh sessions.
            Box::new(m20260513_000001_add_dev_domain_buildwithruud::Migration),
            // Unique active magic link per user
            Box::new(m20260513_000002_unique_active_magic_link_per_user::Migration),
            // 2026-05-15 Security #7: hash bearer_token and refresh_token before storage.
            // Non-breaking: adds hash columns alongside plaintext, backfills existing rows.
            // Plaintext columns dropped in a follow-up migration after full rollout.
            Box::new(m20260515_000001_hash_session_tokens::Migration),
            // 2026-05-17: Ensure mailing_list table exists with preferences JSONB + tenant_id.
            // Fixes ERR_NO_DATA on the Mailing List admin tab.
            Box::new(m20260517_000001_ensure_mailing_list_schema::Migration),
            // 2026-05-18: Dynamic Admin Module Registry — platform-wide module config table.
            Box::new(m20260518_000001_create_app_instance_modules::Migration),
            // 2026-05-22: Create webauthn_challenges table to persist session/challenge state in PostgreSQL.
            Box::new(m20260522_000001_create_webauthn_challenges::Migration),
            // 2026-05-23: Create outbox_job table to support Transactional Outbox Pattern
            Box::new(m20260523_000001_create_outbox_jobs::Migration),
            Box::new(m20260523_000002_extend_lead_table::Migration),
            Box::new(m20260523_000003_create_crm_status_options::Migration),
            Box::new(m20260523_000004_ensure_crm_modules::Migration),
            Box::new(m20260523_000005_create_crm_activity_and_deep_fields::Migration),
            Box::new(m20260523_000006_create_headless_email_tables::Migration),
            Box::new(m20260524_000001_extend_crm_avatar_attachments::Migration),
            Box::new(m20260525_000001_extend_notes_and_activities::Migration),
            // GENERIC-31: Canonical lead / prospect entity (G-31)
            // Includes set_updated_at_column() trigger function — must run before gap-fill.
            Box::new(m20260601_g31_atlas_lead::Migration),
            // Gap-fill: atlas_accounts + atlas_contacts firmographic promotion
            // Sorts after G-31 alphabetically (m20260702_ > m20260601_).
            Box::new(m20260702_gap_fill_accounts_contacts::Migration),
            // GENERIC-27: Atlas Scorecards — Universal Structured Evaluation Engine (11 tables)
            // m20260701_ sorts before m20260702_ so G-27 runs before gap-fill (name sort).
            Box::new(m20260701_g27_scorecards::Migration),
            // GENERIC-28: atlas_note — promotes notes table
            Box::new(m20260703_g28_atlas_note::Migration),
            // GENERIC-29: atlas_activity — promotes activity table
            Box::new(m20260704_g29_atlas_activity::Migration),
            // Drop legacy lead table + compat view rename
            Box::new(m20260705_drop_legacy_lead::Migration),
            // G-27 background job seeding: registers scorecard aggregate + time-series jobs
            // Sorts after G-27 schema (m20260706_ > m20260701_) — schema must exist first.
            Box::new(m20260706_seed_g27_background_jobs::Migration),
            // G-27 gap fill: is_inverted column on atlas_scorecard_dimensions
            Box::new(m20260707_g27_is_inverted::Migration),
            // G-27 gap fill: atlas_scorecard_display_rules (Context-Aware Display Rules engine)
            Box::new(m20260708_g27_display_rules::Migration),
            // G-27 Data Science Upgrade Phase 1: cold_start_strategy + bayesian_prior_weight
            Box::new(m20260709_g27_data_science_v1::Migration),
            // G-27 Data Science Upgrade Phase 2: masked vectors + anomaly detection + calibration
            Box::new(m20260710_g27_data_science_v2::Migration),
            // G-27 Patch: Add deleted_at to atlas_scorecards before v3 creates views that
            // reference sc.deleted_at. m20260711_0_ sorts before m20260711_g_ in the vec.
            Box::new(m20260711_0_g27_add_deleted_at_scorecards::Migration),
            // G-27 Patch: Add deleted_at to atlas_scorecard_templates.
            // Required by m20260712_seed_portfolio_job which queries templates.deleted_at.
            // Separate migration because m20260711_0_ already ran on UAT/dev live databases.
            Box::new(m20260711_1_g27_add_deleted_at_templates::Migration),
            // G-27 Data Science Upgrade Phase 3: portfolio analytics materialized view
            // + recent anomalies live view. Sorts after v2 (m20260711_ > m20260710_).
            Box::new(m20260711_g27_data_science_v3::Migration),
            // G-27 Phase 3: Register refresh_scorecard_portfolio background job.
            // Sorts after the MV creation (m20260712_ > m20260711_) — view must exist first.
            Box::new(m20260712_g27_seed_portfolio_job::Migration),
            // G-27 Phase 4: Register calibrate_scorecard_contributors weekly job.
            // Sorts after portfolio job seed (m20260713_ > m20260712_).
            Box::new(m20260713_g27_seed_calibration_job::Migration),
            // G-27 gap fill: display_config JSONB on atlas_scorecard_templates.
            // Added in type-safety refactor; migration was missing from previous runs.
            Box::new(m20260714_g27_display_config::Migration),
            // Phase 0 — PM: template_scope (platform/tenant) + is_tenant_extension (bool).
            // Must run before PropertyManagementApp::provision() seeds PM scorecard templates.
            Box::new(m20260801_pm_g27_template_scope::Migration),
            // GENERIC-23: atlas_reservations — Time-bounded reservation with inventory hold.
            // Sorts after all G-27 work (m20260802_ > m20260801_) — schema must be stable first.
            Box::new(m20260802_g23_atlas_reservations::Migration),
            // GENERIC-26: atlas_catalog — Product catalog, pricebook & availability grid.
            // Three tables: atlas_catalog_entries, atlas_catalog_rate_rules, atlas_catalog_availability.
            // Sorts after G23 (m20260803_ > m20260802_) — no FK dependency on G23, but sequenced
            // for stability in the migration vector.
            Box::new(m20260803_g26_atlas_catalog::Migration),
            // GENERIC-19: atlas_campaigns — Multi-channel campaign management.
            // 4 tables (atlas_campaigns, atlas_sequence_steps, atlas_campaign_enrollments,
            // atlas_campaign_events) + 3 enums + 6 indexes.
            Box::new(m20260804_g19_atlas_campaigns::Migration),
            // GENERIC-20: atlas_attribution — Multi-channel attribution touchpoints.
            // Depends on G19 (FKs to atlas_campaigns + atlas_campaign_enrollments).
            Box::new(m20260805_g20_atlas_attribution::Migration),
            // GENERIC-21: atlas_events — Event management, ticketing & check-in.
            // 3 tables (atlas_events, atlas_event_ticket_types, atlas_event_registrations)
            // + 6 indexes. Integrates with G19 (campaign FK), G03 (ledger FK), G20 (attribution FK).
            Box::new(m20260806_g21_atlas_events::Migration),
            // GENERIC-22: atlas_record_relationships — Universal polymorphic M:M junction table.
            // Salesforce Junction Object pattern. 1 table + 3 indexes (unique, source, target).
            Box::new(m20260807_g22_atlas_record_relationships::Migration),
            // GENERIC-24: atlas_quotes — Pre-purchase pricing proposals + line items.
            // Closes the commerce chain: G26 catalog → G24 quotes → G23 reservations.
            // Both G23 migrations have nullable quote_id FKs pointing to this table.
            Box::new(m20260808_g24_atlas_quotes::Migration),
            // G26 forward: backfills atlas_catalog_entry_type enum, available_count GENERATED column,
            // update_updated_at_column() triggers, and performance indexes on live DBs that had the
            // original G26 (m20260701_g26_catalog, renamed to m20260803_g26_atlas_catalog) applied
            // before the enum/column additions were written. All steps are fully idempotent.
            Box::new(m20260809_g26_catalog_forward::Migration),
            // Folio multi-role: adds folio_role VARCHAR(20) DEFAULT 'landlord' to user_account.
            // Transitional — superseded by G-32. Column is dropped by m20260814 after backfill.
            Box::new(m20260810_add_folio_role_to_user_account::Migration),
            // G-32: Platform-generic RBAC — App→Profile→Role→User hierarchy.
            // Adds atlas_role_profiles, atlas_user_app_roles, atlas_role_profile_permissions.
            Box::new(m20260811_g32_atlas_rbac::Migration),
            // G-32: Seeds platform-default Folio role profiles (landlord/tenant/vendor).
            Box::new(m20260812_g32_folio_role_seed::Migration),
            // G-32: Backfills atlas_user_app_roles from existing user_account.folio_role values.
            Box::new(m20260813_g32_migrate_folio_roles::Migration),
            // G-32: Drops user_account.folio_role column now that G-32 owns role storage.
            Box::new(m20260814_g32_drop_folio_role_column::Migration),
            // G-33: atlas_app_deployment_config — platform-generic app mode + config table.
            // Any app (Folio, future CRM, HR, etc.) uses this to declare its deployment mode.
            Box::new(m20260815_g33_app_deployment_config::Migration),
            // G-33/Folio: Seeds the property_manager role profile + permissions into RBAC tables.
            // Requires G-33 atlas_app_deployment_config and G-32 RBAC tables to already exist.
            // Moved from FolioApp::migrations() to the base vec — Rule 7 forbids app-scoped migrations.
            Box::new(m20260816_g33_folio_pmc_seed::Migration),
            // Folio PMC: nullable managed_account_id FK on contract/asset/portfolio/lead tables.
            // Non-breaking: NULL = single-landlord mode (default). UUID = PMC client book assignment.
            // Moved from FolioApp::migrations() to the base vec — Rule 7 forbids app-scoped migrations.
            Box::new(m20260817_folio_managed_account_id::Migration),
            // Folio PMC: client_account_id scope column on atlas_user_app_roles.
            // Enables scoped Landlord role assignment for PMC client invite flow.
            Box::new(m20260818_folio_client_role_scope::Migration),
            // G-34: vendor marketplace opt-in columns (is_marketplace_visible, bio, location).
            // Reuses atlas_service_providers + G-22 endorsements + G-01 PostGIS.
            Box::new(m20260819_g34_vendor_marketplace::Migration),
            // G-10: universal asset lifecycle extension.
            // Adds scheduled_service_date, expiry_date, condition, lifecycle_metadata + 3 indexes.
            // Powers cross-vertical alert queries. No per-vertical migrations needed after this.
            Box::new(m20260900_g10_asset_lifecycle::Migration),
            // Platform Admin: platform_products — product registry with deploy hooks (zero-tenant marketing management)
            Box::new(m20260901_platform_products::Migration),
            // Platform Admin: public_slug + custom_domain + instance_status on atlas_app_deployment_config
            Box::new(m20260902_app_instance_public_config::Migration),
            // Product Launch Engine: launch_mode, pre-order, founding cap, waitlist_count
            Box::new(m20260903_platform_products_launch_engine::Migration),
            // Product Launch Engine: product_page_templates + product_page_variants
            // Enables programmatic hyperlocalized SEO pages (100+ market variants per product)
            Box::new(m20260904_product_page_variants::Migration),
            // Product Launch Engine: apex_domain per product, AI localization columns on variants, product_domain_aliases
            Box::new(m20260905_product_domain_localization::Migration),
            // Billing Grace Period and Exemption Override fields
            Box::new(m20260906_subscription_grace_period::Migration),
            // Feature Flags: flag registry, per-tenant NI overrides, audit trail
            Box::new(m20260907_feature_flags::Migration),
            // Platform Invitations: platform_invite table
            Box::new(m20260908_platform_invitations::Migration),
        ];

        for app in crate::atlas_apps::get_active_apps() {
            base.extend(app.migrations());
        }

        base.sort_by(|a, b| a.name().cmp(b.name()));

        base
    }
}