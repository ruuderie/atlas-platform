use sea_orm_migration::prelude::*;

pub mod m20230912_000000_create_users_table;
pub mod m20230911_create_accounts_table;
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
pub mod m20230911_create_sessions_table;
pub mod m20240922_create_crm_tables;
pub mod m20240922_create_request_log_table;
pub mod m20240923_create_feed_tables;
pub mod m20240924_update_listings_nullable_category;
pub mod m20240924_update_directory_multisite_fields;
pub mod m20240315_add_directory_domain_fields;
pub mod m20241001_add_icon_and_slug_to_categories;
pub mod m20241002_add_directory_id_to_crm_and_categories;
pub mod m20241003_add_slug_to_listings;
pub mod m20250101_create_ab_testing_tables;
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
pub mod m20260408_000002_create_anchor_legacy_tables;
pub mod m20260408_000003_seed_anchor_background_jobs;
pub mod m20260408_000004_fix_anchor_tables_and_seed;
pub mod m20260408_000005_add_missing_anchor_tables;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
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
            Box::new(m20260408_000002_create_anchor_legacy_tables::Migration),
            Box::new(m20260408_000003_seed_anchor_background_jobs::Migration),
            Box::new(m20260408_000004_fix_anchor_tables_and_seed::Migration),
            Box::new(m20260408_000005_add_missing_anchor_tables::Migration),
        ];

        for app in crate::atlas_apps::get_active_apps() {
            base.extend(app.migrations());
        }

        base.sort_by(|a, b| a.name().cmp(b.name()));

        base
    }
}