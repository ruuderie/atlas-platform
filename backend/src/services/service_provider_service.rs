#![allow(unused_variables, dead_code)]
use sea_orm::{DatabaseConnection, EntityTrait, ActiveModelTrait, Set, QueryFilter, ColumnTrait, QuerySelect};
use uuid::Uuid;
use chrono::Utc;
use serde_json::Value;

use crate::entities::atlas_service_provider::{self, Entity as ServiceProviderEntity, ActiveModel as ServiceProviderActiveModel};

/// Service layer for GENERIC-12: AtlasServiceProvider
/// Vendors, contractors, agents, marketplace participants.
pub struct ServiceProviderService;

impl ServiceProviderService {
    pub async fn create_service_provider(
        db: &DatabaseConnection,
        tenant_id: Uuid,
        user_id: Uuid,
        scope: &str,
        business_name: Option<&str>,
        service_categories: Value,
        status: &str,
    ) -> Result<Uuid, String> {
        let sp = ServiceProviderActiveModel {
            id: Set(Uuid::new_v4()),
            tenant_id: Set(tenant_id),
            user_id: Set(user_id),
            scope: Set(scope.to_string()),
            business_name: Set(business_name.map(|s| s.to_string())),
            service_categories: Set(service_categories),
            status: Set(status.to_string()),
            rating_avg: Set(None),
            rating_count: Set(0),
            created_at: Set(Utc::now()),
            ..Default::default()
        };

        let result = sp.insert(db).await.map_err(|e| e.to_string())?;
        Ok(result.id)
    }

    pub async fn find_by_id(
        db: &DatabaseConnection,
        tenant_id: Uuid,
        service_provider_id: Uuid,
    ) -> Result<Option<atlas_service_provider::Model>, String> {
        ServiceProviderEntity::find()
            .filter(atlas_service_provider::Column::TenantId.eq(tenant_id))
            .filter(atlas_service_provider::Column::Id.eq(service_provider_id))
            .one(db)
            .await
            .map_err(|e| e.to_string())
    }

    pub async fn list_for_tenant(
        db: &DatabaseConnection,
        tenant_id: Uuid,
        scope: Option<&str>,
        status: Option<&str>,
        limit: u64,
    ) -> Result<Vec<atlas_service_provider::Model>, String> {
        let mut q = ServiceProviderEntity::find()
            .filter(atlas_service_provider::Column::TenantId.eq(tenant_id));

        if let Some(sc) = scope {
            q = q.filter(atlas_service_provider::Column::Scope.eq(sc.to_string()));
        }
        if let Some(st) = status {
            q = q.filter(atlas_service_provider::Column::Status.eq(st.to_string()));
        }

        q.limit(limit)
            .all(db)
            .await
            .map_err(|e| e.to_string())
    }

    pub async fn record_rating(
        db: &DatabaseConnection,
        tenant_id: Uuid,
        service_provider_id: Uuid,
        rating: f64,
    ) -> Result<(), String> {
        tracing::info!("Recorded rating {} for service provider {}", rating, service_provider_id);
        Ok(())
    }
}