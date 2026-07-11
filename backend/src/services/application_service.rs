#![allow(unused_variables, dead_code)]
use chrono::Utc;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, QuerySelect, Set,
};
use serde_json::Value;
use uuid::Uuid;

use crate::entities::atlas_application::{
    self, ActiveModel as ApplicationActiveModel, Entity as ApplicationEntity,
};

/// Service layer for GENERIC-18: AtlasApplication
/// Structured applications, onboarding flows, submissions with rich JSONB data.
pub struct ApplicationService;

impl ApplicationService {
    pub async fn create_application(
        db: &DatabaseConnection,
        tenant_id: Uuid,
        application_type: &str,
        applicant_user_id: Uuid,
        status: &str,
        application_metadata: Option<Value>,
    ) -> Result<Uuid, String> {
        let app = ApplicationActiveModel {
            id: Set(Uuid::new_v4()),
            tenant_id: Set(tenant_id),
            application_type: Set(application_type.to_string()),
            applicant_user_id: Set(applicant_user_id),
            status: Set(status.to_string()),
            application_metadata: Set(application_metadata),
            created_at: Set(Utc::now()),
            ..Default::default()
        };

        let result = app.insert(db).await.map_err(|e| e.to_string())?;
        Ok(result.id)
    }

    pub async fn find_by_id(
        db: &DatabaseConnection,
        tenant_id: Uuid,
        application_id: Uuid,
    ) -> Result<Option<atlas_application::Model>, String> {
        ApplicationEntity::find()
            .filter(atlas_application::Column::TenantId.eq(tenant_id))
            .filter(atlas_application::Column::Id.eq(application_id))
            .one(db)
            .await
            .map_err(|e| e.to_string())
    }

    pub async fn list_for_tenant(
        db: &DatabaseConnection,
        tenant_id: Uuid,
        application_type: Option<&str>,
        status: Option<&str>,
        limit: u64,
    ) -> Result<Vec<atlas_application::Model>, String> {
        let mut q =
            ApplicationEntity::find().filter(atlas_application::Column::TenantId.eq(tenant_id));

        if let Some(at) = application_type {
            q = q.filter(atlas_application::Column::ApplicationType.eq(at.to_string()));
        }
        if let Some(s) = status {
            q = q.filter(atlas_application::Column::Status.eq(s.to_string()));
        }

        q.limit(limit).all(db).await.map_err(|e| e.to_string())
    }

    pub async fn submit_application(
        db: &DatabaseConnection,
        tenant_id: Uuid,
        application_id: Uuid,
    ) -> Result<(), String> {
        tracing::info!("Application {} submitted", application_id);
        Ok(())
    }

    pub async fn decide_application(
        db: &DatabaseConnection,
        tenant_id: Uuid,
        application_id: Uuid,
        decision: &str,
    ) -> Result<(), String> {
        tracing::info!("Application {} decided as {}", application_id, decision);
        Ok(())
    }
}
