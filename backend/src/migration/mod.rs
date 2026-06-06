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
// G-27 Data Science Upgrade Phase 3: mv_scorecard_portfolio_analytics materialized view
// + v_scorecard_recent_anomalies live view. Powers portfolio API + BYOC peer pool.
pub mod m20260711_g27_data_science_v3;
// G-27 Phase 3: Seed refresh_scorecard_portfolio background job (every 4 hours).
pub mod m20260712_g27_seed_portfolio_job;
// G-27 Phase 4: Seed calibrate_scorecard_contributors weekly background job.
pub mod m20260713_g27_seed_calibration_job;

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
            // G-27 Data Science Upgrade Phase 3: portfolio analytics materialized view
            // + recent anomalies live view. Sorts after v2 (m20260711_ > m20260710_).
            Box::new(m20260711_g27_data_science_v3::Migration),
            // G-27 Phase 3: Register refresh_scorecard_portfolio background job.
            // Sorts after the MV creation (m20260712_ > m20260711_) — view must exist first.
            Box::new(m20260712_g27_seed_portfolio_job::Migration),
            // G-27 Phase 4: Register calibrate_scorecard_contributors weekly job.
            // Sorts after portfolio job seed (m20260713_ > m20260712_).
            Box::new(m20260713_g27_seed_calibration_job::Migration),
        ];

        for app in crate::atlas_apps::get_active_apps() {
            base.extend(app.migrations());
        }

        base.sort_by(|a, b| a.name().cmp(b.name()));

        base
    }
}