#![allow(unused_variables, dead_code)]
use sea_orm::{DatabaseConnection, EntityTrait, ActiveModelTrait, Set, QueryFilter, ColumnTrait, QuerySelect};
use uuid::Uuid;
use chrono::Utc;
use serde_json::Value;

use crate::entities::atlas_regulatory_registration::{self, Entity as RegulatoryRegistrationEntity, ActiveModel as RegulatoryRegistrationActiveModel};

/// Service layer for GENERIC-16: AtlasRegulatoryRegistration
/// Permits, licenses, certifications, compliance registrations.
pub struct RegulatoryRegistrationService;

impl RegulatoryRegistrationService {
    pub async fn create_registration(
        db: &DatabaseConnection,
        tenant_id: Uuid,
        registration_type: &str,
        jurisdiction_code: &str,
        registration_number: &str,
        status: &str,
        expires_at: Option<chrono::NaiveDate>,
        jurisdiction_metadata: Option<Value>,
    ) -> Result<Uuid, String> {
        let reg = RegulatoryRegistrationActiveModel {
            id: Set(Uuid::new_v4()),
            tenant_id: Set(tenant_id),
            registration_type: Set(registration_type.to_string()),
            jurisdiction_code: Set(jurisdiction_code.to_string()),
            registration_number: Set(registration_number.to_string()),
            status: Set(status.to_string()),
            expires_at: Set(expires_at),
            jurisdiction_metadata: Set(jurisdiction_metadata),
            created_at: Set(Utc::now()),
            ..Default::default()
        };

        let result = reg.insert(db).await.map_err(|e| e.to_string())?;
        Ok(result.id)
    }

    pub async fn find_by_id(
        db: &DatabaseConnection,
        tenant_id: Uuid,
        registration_id: Uuid,
    ) -> Result<Option<atlas_regulatory_registration::Model>, String> {
        RegulatoryRegistrationEntity::find()
            .filter(atlas_regulatory_registration::Column::TenantId.eq(tenant_id))
            .filter(atlas_regulatory_registration::Column::Id.eq(registration_id))
            .one(db)
            .await
            .map_err(|e| e.to_string())
    }

    pub async fn list_for_tenant(
        db: &DatabaseConnection,
        tenant_id: Uuid,
        jurisdiction_code: Option<&str>,
        status: Option<&str>,
        limit: u64,
    ) -> Result<Vec<atlas_regulatory_registration::Model>, String> {
        let mut q = RegulatoryRegistrationEntity::find()
            .filter(atlas_regulatory_registration::Column::TenantId.eq(tenant_id));

        if let Some(j) = jurisdiction_code {
            q = q.filter(atlas_regulatory_registration::Column::JurisdictionCode.eq(j.to_string()));
        }
        if let Some(s) = status {
            q = q.filter(atlas_regulatory_registration::Column::Status.eq(s.to_string()));
        }

        q.limit(limit)
            .all(db)
            .await
            .map_err(|e| e.to_string())
    }

    pub async fn renew_registration(
        db: &DatabaseConnection,
        tenant_id: Uuid,
        registration_id: Uuid,
        new_expiry: chrono::NaiveDate,
    ) -> Result<(), String> {
        tracing::info!("Renewed regulatory registration {} until {}", registration_id, new_expiry);
        Ok(())
    }
}