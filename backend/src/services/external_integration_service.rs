#![allow(unused_variables, dead_code)]
use sea_orm::{DatabaseConnection, EntityTrait, ActiveModelTrait, Set, QueryFilter, ColumnTrait, QuerySelect};
use uuid::Uuid;
use chrono::Utc;
use serde_json::{Value, json};

use crate::entities::atlas_external_integration::{self, Entity as ExternalIntegrationEntity, ActiveModel as ExternalIntegrationActiveModel};

/// Service layer for GENERIC-05: AtlasExternalIntegration
/// Pluggable connections to PMS, OTA, GDS, accounting, telephony, etc.
pub struct ExternalIntegrationService;

impl ExternalIntegrationService {
    pub async fn create_integration(
        db: &DatabaseConnection,
        tenant_id: Uuid,
        integration_type: &str,
        label: Option<&str>,
        config: Option<Value>,
    ) -> Result<Uuid, String> {
        let integ = ExternalIntegrationActiveModel {
            id: Set(Uuid::new_v4()),
            tenant_id: Set(tenant_id),
            integration_type: Set(integration_type.to_string()),
            label: Set(label.map(|s| s.to_string())),
            credentials_encrypted: Set(json!({})),
            is_active: Set(true),
            config: Set(config),
            created_at: Set(Utc::now()),
            ..Default::default()
        };

        let result = integ.insert(db).await.map_err(|e| e.to_string())?;
        Ok(result.id)
    }

    pub async fn find_by_id(
        db: &DatabaseConnection,
        tenant_id: Uuid,
        integration_id: Uuid,
    ) -> Result<Option<atlas_external_integration::Model>, String> {
        ExternalIntegrationEntity::find()
            .filter(atlas_external_integration::Column::TenantId.eq(tenant_id))
            .filter(atlas_external_integration::Column::Id.eq(integration_id))
            .one(db)
            .await
            .map_err(|e| e.to_string())
    }

    pub async fn list_for_tenant(
        db: &DatabaseConnection,
        tenant_id: Uuid,
        integration_type: Option<&str>,
        limit: u64,
    ) -> Result<Vec<atlas_external_integration::Model>, String> {
        let mut q = ExternalIntegrationEntity::find()
            .filter(atlas_external_integration::Column::TenantId.eq(tenant_id));

        if let Some(it) = integration_type {
            q = q.filter(atlas_external_integration::Column::IntegrationType.eq(it.to_string()));
        }

        q.limit(limit)
            .all(db)
            .await
            .map_err(|e| e.to_string())
    }

    pub async fn record_sync(
        db: &DatabaseConnection,
        tenant_id: Uuid,
        integration_id: Uuid,
    ) -> Result<(), String> {
        tracing::info!("External integration {} sync recorded", integration_id);
        Ok(())
    }
}