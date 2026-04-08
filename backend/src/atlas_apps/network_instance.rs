use crate::traits::atlas_app::{AtlasApp, BackgroundJob};
use axum::Router;
use sea_orm::DatabaseConnection;
use sea_orm_migration::MigrationTrait;
use async_trait::async_trait;

pub struct NetworkInstanceApp;

#[async_trait]
impl AtlasApp for NetworkInstanceApp {
    fn app_id(&self) -> &'static str {
        "network_instance"
    }

    fn public_router(&self, db: DatabaseConnection) -> Router<DatabaseConnection> {
        // Expose all Network / Directory specific API routes that are accessible without login
        Router::new()
            .merge(crate::handlers::listings::public_routes(db.clone()))
            .merge(crate::handlers::app_pages::public_routes(db.clone()))
            .merge(crate::handlers::app_menus::public_routes(db.clone()))
            .merge(crate::handlers::leads::public_routes())
    }

    fn authenticated_router(&self, db: DatabaseConnection) -> Router<DatabaseConnection> {
        // Expose all Network / Directory specific API routes that require standard authentication
        Router::new()
            .merge(crate::handlers::listings::authenticated_routes())
            .merge(crate::handlers::profiles::routes(db.clone()))
            .merge(crate::handlers::ad_purchases::routes())
            .merge(crate::handlers::leads::authenticated_routes())
            .merge(crate::handlers::customers::routes())
            .merge(crate::handlers::contacts::routes())
            .merge(crate::handlers::deals::routes())
            .merge(crate::handlers::activities::routes())
            .merge(crate::handlers::cases::routes())
            .merge(crate::handlers::notes::routes())
    }

    fn migrations(&self) -> Vec<Box<dyn MigrationTrait>> {
        // Collect ONLY the domain schemas for the Network / Listing / CRM modules as best practice.
        // Note: Legacy Directory migrations have been extracted to the Core Base Migrator since Directory was converted to a Core Tenant Architecture.
        vec![
            Box::new(crate::migration::m20230915_create_profiles_table::Migration),
            Box::new(crate::migration::m20230916_create_categories_table::Migration),
            Box::new(crate::migration::m20230917_create_templates_table::Migration),
            Box::new(crate::migration::m20230918_create_listings_table::Migration),
            Box::new(crate::migration::m20230919_create_listing_attributes_table::Migration),
            Box::new(crate::migration::m20230920_create_ad_purchases_table::Migration),
            Box::new(crate::migration::m20240922_create_crm_tables::Migration),
            Box::new(crate::migration::m20240924_update_listings_nullable_category::Migration),
            Box::new(crate::migration::m20241001_add_icon_and_slug_to_categories::Migration),
            Box::new(crate::migration::m20241003_add_slug_to_listings::Migration),
            Box::new(crate::migration::m20260324_000001_collapse_eav_to_jsonb::Migration),
            Box::new(crate::migration::m20260326_add_service_area_to_profile::Migration),
            Box::new(crate::migration::m20260326_create_lead_charges_table::Migration),
        ]
    }

    fn background_jobs(&self) -> Vec<BackgroundJob> {
        // Currently NetworkInstance endpoints hook mostly from direct HTTP. Background jobs handle Telemetry which is Core.
        vec![]
    }
}
