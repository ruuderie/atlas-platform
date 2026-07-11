#![allow(unused_variables, dead_code)]
use chrono::Utc;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, QuerySelect, Set,
};
use uuid::Uuid;

use crate::entities::atlas_verification_request::{
    self, ActiveModel as VerificationRequestActiveModel, Entity as VerificationRequestEntity,
};

/// Service layer for GENERIC-06: AtlasVerificationRequest
/// Human-in-the-loop verification queue (KYC, document review, manual approvals, etc.).
pub struct VerificationService;

impl VerificationService {
    pub async fn create_verification_request(
        db: &DatabaseConnection,
        tenant_id: Uuid,
        subject_type: &str,
        subject_id: Uuid,
        requested_by_user_id: Uuid,
        status: &str,
    ) -> Result<Uuid, String> {
        let vr = VerificationRequestActiveModel {
            id: Set(Uuid::new_v4()),
            tenant_id: Set(tenant_id),
            subject_type: Set(subject_type.to_string()),
            subject_id: Set(subject_id),
            requested_by_user_id: Set(requested_by_user_id),
            status: Set(status.to_string()),
            created_at: Set(Utc::now()),
            ..Default::default()
        };

        let result = vr.insert(db).await.map_err(|e| e.to_string())?;
        Ok(result.id)
    }

    pub async fn find_by_id(
        db: &DatabaseConnection,
        tenant_id: Uuid,
        verification_id: Uuid,
    ) -> Result<Option<atlas_verification_request::Model>, String> {
        VerificationRequestEntity::find()
            .filter(atlas_verification_request::Column::TenantId.eq(tenant_id))
            .filter(atlas_verification_request::Column::Id.eq(verification_id))
            .one(db)
            .await
            .map_err(|e| e.to_string())
    }

    pub async fn list_for_tenant(
        db: &DatabaseConnection,
        tenant_id: Uuid,
        status: Option<&str>,
        limit: u64,
    ) -> Result<Vec<atlas_verification_request::Model>, String> {
        let mut q = VerificationRequestEntity::find()
            .filter(atlas_verification_request::Column::TenantId.eq(tenant_id));

        if let Some(s) = status {
            q = q.filter(atlas_verification_request::Column::Status.eq(s.to_string()));
        }

        q.limit(limit).all(db).await.map_err(|e| e.to_string())
    }

    pub async fn assign_and_start(
        db: &DatabaseConnection,
        tenant_id: Uuid,
        verification_id: Uuid,
        reviewed_by_user_id: Uuid,
    ) -> Result<(), String> {
        tracing::info!(
            "Verification request {} assigned/reviewed by {}",
            verification_id,
            reviewed_by_user_id
        );
        Ok(())
    }

    pub async fn complete_verification(
        db: &DatabaseConnection,
        tenant_id: Uuid,
        verification_id: Uuid,
        status: &str,
        rejection_reason: Option<&str>,
    ) -> Result<(), String> {
        tracing::info!(
            "Verification {} completed as {} (reason: {:?})",
            verification_id,
            status,
            rejection_reason
        );
        Ok(())
    }
}
