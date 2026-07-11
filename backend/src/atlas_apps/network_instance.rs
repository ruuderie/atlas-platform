use crate::traits::atlas_app::{
    AppSeedPack, AtlasApp, BackgroundJob, OnboardingStep, StepCompletionCheck,
};
use async_trait::async_trait;
use axum::Router;
use sea_orm::DatabaseConnection;
use sea_orm_migration::MigrationTrait;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub enum NetworkPermission {
    ListingPoster,
    Subscriber,
}

pub struct NetworkInstanceApp;

#[async_trait]
impl AtlasApp for NetworkInstanceApp {
    fn app_id(&self) -> &'static str {
        "network_instance"
    }

    fn public_router(&self, db: DatabaseConnection) -> Router<DatabaseConnection> {
        // Expose all Network / Directory specific API routes that are accessible without login.
        //
        // NOTE: app_pages::public_routes is intentionally NOT registered here.
        // CMS pages are a cross-cutting concern (consumed by anchor, network_instance, etc.)
        // and are owned by the global api.rs router alongside app_menus. Registering it
        // here would create a duplicate route and cause Axum to panic at startup with
        // "Overlapping method route". See backend/src/api.rs for the canonical registration.
        Router::new()
            .merge(crate::handlers::listings::public_routes(db.clone()))
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
            .merge(crate::handlers::cases::routes())
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

    fn seed_packs(&self) -> Vec<AppSeedPack> {
        crate::atlas_apps::seeds::network_instance::all_packs()
    }

    fn onboarding_steps(&self) -> Vec<OnboardingStep> {
        vec![
            OnboardingStep {
                id: "identity".to_string(),
                title: "Network Identity".to_string(),
                description: "Set your network's name and tagline so members know what it's about."
                    .to_string(),
                is_required: true,
                position: 1,
                completion_check: StepCompletionCheck::TenantSettingExists {
                    key: "site_title".to_string(),
                },
            },
            OnboardingStep {
                id: "domain".to_string(),
                title: "Custom Domain".to_string(),
                description: "Connect your domain so your network has its live web address."
                    .to_string(),
                is_required: true,
                position: 2,
                completion_check: StepCompletionCheck::AppDomainExists,
            },
            OnboardingStep {
                id: "categories".to_string(),
                title: "Categories".to_string(),
                description: "Add at least one category to organize your listings.".to_string(),
                is_required: true,
                position: 3,
                completion_check: StepCompletionCheck::EntityCountGte {
                    table: "category",
                    min: 1,
                },
            },
            OnboardingStep {
                id: "first_template".to_string(),
                title: "Listing Template".to_string(),
                description:
                    "Choose a listing template from the library to define how listings appear."
                        .to_string(),
                is_required: true,
                position: 4,
                completion_check: StepCompletionCheck::EntityCountGte {
                    table: "template",
                    min: 1,
                },
            },
            OnboardingStep {
                id: "first_listing".to_string(),
                title: "First Listing".to_string(),
                description: "Add your first listing to populate the network for members."
                    .to_string(),
                is_required: false,
                position: 5,
                completion_check: StepCompletionCheck::EntityCountGte {
                    table: "listing",
                    min: 1,
                },
            },
        ]
    }

    fn default_modules(
        &self,
    ) -> Vec<(
        crate::models::admin_module::AdminModuleType,
        &'static str,
        i32,
        bool,
    )> {
        use crate::models::admin_module::AdminModuleType as M;
        vec![
            // Fixed platform modules — cannot be disabled
            (M::Dashboard, "Dashboard", 0, true),
            (M::Settings, "Settings", 50, true),
            (M::Security, "Security", 60, true),
            // Marketplace core
            (M::Listings, "Listings", 10, false),
            (M::Leads, "Leads", 20, false),
            (M::Contacts, "Contacts", 30, false),
            // Appearance
            (M::Navigation, "Navigation", 40, false),
        ]
    }

    async fn provision(
        &self,
        db: &DatabaseConnection,
        tenant_id: uuid::Uuid,
    ) -> Result<(), String> {
        use crate::services::module_provisioning::{resolve_app_instance_id, seed_default_modules};
        let app_instance_id = resolve_app_instance_id(db, tenant_id, self.app_id()).await?;
        seed_default_modules(db, app_instance_id, self.default_modules()).await
    }
}
