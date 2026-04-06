use sea_orm_migration::prelude::*;

mod m20230912_create_users_table;
mod m20230911_create_accounts_table;
mod m20230912_create_user_accounts_table;
mod m20230913_create_directory_types_table;
mod m20230914_create_directories_table;
mod m20230915_create_profiles_table;
mod m20230916_create_categories_table;
mod m20230917_create_templates_table;
mod m20230918_create_listings_table;
mod m20230919_create_listing_attributes_table;
mod m20230920_create_ad_purchases_table;
mod m20240001_update_timestamp_migration;
mod m20230911_create_sessions_table;
mod m20240922_create_crm_tables;
mod m20240922_create_request_log_table;
mod m20240923_create_feed_tables;
mod m20240924_update_listings_nullable_category;
mod m20240924_update_directory_multisite_fields;
mod m20240315_add_directory_domain_fields;
mod m20241001_add_icon_and_slug_to_categories;
mod m20241002_add_directory_id_to_crm_and_categories;
mod m20241003_add_slug_to_listings;
mod m20250101_create_ab_testing_tables;
mod m20260320_create_passkeys_table;
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

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20230911_create_accounts_table::Migration),
            Box::new(m20230911_create_sessions_table::Migration),
            Box::new(m20230912_create_users_table::Migration),
            Box::new(m20230912_create_user_accounts_table::Migration),
            Box::new(m20230913_create_directory_types_table::Migration),
            Box::new(m20230914_create_directories_table::Migration),
            Box::new(m20230915_create_profiles_table::Migration),
            Box::new(m20230916_create_categories_table::Migration),
            Box::new(m20230917_create_templates_table::Migration),
            Box::new(m20230918_create_listings_table::Migration),
            Box::new(m20230919_create_listing_attributes_table::Migration),
            Box::new(m20230920_create_ad_purchases_table::Migration),

            Box::new(m20240315_add_directory_domain_fields::Migration),
            Box::new(m20240922_create_request_log_table::Migration),
            Box::new(m20240922_create_crm_tables::Migration),
            Box::new(m20240923_create_feed_tables::Migration),
            Box::new(m20240924_update_directory_multisite_fields::Migration),
            Box::new(m20240924_update_listings_nullable_category::Migration),
            Box::new(m20241001_add_icon_and_slug_to_categories::Migration),
            Box::new(m20241002_add_directory_id_to_crm_and_categories::Migration),
            Box::new(m20241003_add_slug_to_listings::Migration),
            Box::new(m20240001_update_timestamp_migration::Migration),
            Box::new(m20250101_create_ab_testing_tables::Migration),
            Box::new(m20260320_create_passkeys_table::Migration),
            Box::new(m20260324_000001_collapse_eav_to_jsonb::Migration),
            Box::new(m20260326_add_service_area_to_profile::Migration),
            Box::new(m20260326_add_stripe_fields_to_account::Migration),
            Box::new(m20260326_create_lead_charges_table::Migration),
            Box::new(m20260404_000000_core_tenant_shift::Migration),
            Box::new(m20260404_000001_create_cms_tables::Migration),
            Box::new(m20260404_000002_anchor_seed::Migration),
            Box::new(m20260404_000005_create_magic_links::Migration),
            Box::new(m20260404_000006_create_tenant_settings::Migration),
            Box::new(m20260405_000001_rename_directory_to_network::Migration),
            Box::new(m20260406_000000_create_global_search::Migration),
            Box::new(m20260406_000001_create_billing_tables::Migration),
        ]
    }
}