use chrono::Utc;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, QuerySelect, Set,
};
use uuid::Uuid;

use crate::entities::atlas_verification_request::{
    self, ActiveModel as VerificationRequestActiveModel, Entity as VerificationRequestEntity,
};
use crate::types::verification::{
    VerificationRequestType, VerificationStatus, VerificationSubjectType,
};

/// Service layer for GENERIC-06: AtlasVerificationRequest
/// Human-in-the-loop verification queue (KYC, document review, manual approvals, etc.).
pub struct VerificationService;

impl VerificationService {
    pub async fn create_verification_request(
        db: &DatabaseConnection,
        tenant_id: Uuid,
        request_type: VerificationRequestType,
        subject_type: VerificationSubjectType,
        subject_id: Uuid,
        requested_by_user_id: Uuid,
        status: VerificationStatus,
        notes: Option<&str>,
    ) -> Result<Uuid, String> {
        let auto_check_result = notes.map(|n| serde_json::json!({ "submitter_notes": n }));

        let vr = VerificationRequestActiveModel {
            id: Set(Uuid::new_v4()),
            tenant_id: Set(tenant_id),
            subject_type: Set(subject_type.to_string()),
            subject_id: Set(subject_id),
            requested_by_user_id: Set(requested_by_user_id),
            request_type: Set(Some(request_type.to_string())),
            status: Set(status.to_string()),
            auto_check_result: Set(auto_check_result),
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

    pub async fn find_by_id_global(
        db: &DatabaseConnection,
        verification_id: Uuid,
    ) -> Result<Option<atlas_verification_request::Model>, String> {
        VerificationRequestEntity::find_by_id(verification_id)
            .one(db)
            .await
            .map_err(|e| e.to_string())
    }

    pub async fn list_for_tenant(
        db: &DatabaseConnection,
        tenant_id: Uuid,
        status: Option<VerificationStatus>,
        limit: u64,
    ) -> Result<Vec<atlas_verification_request::Model>, String> {
        let mut q = VerificationRequestEntity::find()
            .filter(atlas_verification_request::Column::TenantId.eq(tenant_id));

        if let Some(s) = status {
            q = q.filter(atlas_verification_request::Column::Status.eq(s.to_string()));
        }

        q.limit(limit).all(db).await.map_err(|e| e.to_string())
    }

    /// Move a request into active review and record the reviewer.
    pub async fn assign_and_start(
        db: &DatabaseConnection,
        tenant_id: Uuid,
        verification_id: Uuid,
        reviewed_by_user_id: Uuid,
    ) -> Result<(), String> {
        let request = Self::find_by_id(db, tenant_id, verification_id)
            .await?
            .ok_or_else(|| "Verification request not found".to_string())?;

        let mut active: VerificationRequestActiveModel = request.into();
        active.status = Set(VerificationStatus::Review.to_string());
        active.reviewed_by_user_id = Set(Some(reviewed_by_user_id));
        active.update(db).await.map_err(|e| e.to_string())?;
        Ok(())
    }

    /// Complete a verification as approved or rejected (or other terminal status).
    pub async fn complete_verification(
        db: &DatabaseConnection,
        tenant_id: Uuid,
        verification_id: Uuid,
        status: VerificationStatus,
        rejection_reason: Option<&str>,
        reviewed_by_user_id: Option<Uuid>,
    ) -> Result<atlas_verification_request::Model, String> {
        let request = Self::find_by_id(db, tenant_id, verification_id)
            .await?
            .ok_or_else(|| "Verification request not found".to_string())?;

        let mut active: VerificationRequestActiveModel = request.into();
        active.status = Set(status.to_string());
        if matches!(status, VerificationStatus::Rejected) {
            active.rejection_reason = Set(rejection_reason.map(|s| s.to_string()));
        }
        if let Some(uid) = reviewed_by_user_id {
            active.reviewed_by_user_id = Set(Some(uid));
        }
        active.reviewed_at = Set(Some(Utc::now()));
        active.update(db).await.map_err(|e| e.to_string())
    }

    /// Complete without tenant scoping (platform-admin path).
    pub async fn complete_verification_global(
        db: &DatabaseConnection,
        verification_id: Uuid,
        status: VerificationStatus,
        rejection_reason: Option<&str>,
        reviewed_by_user_id: Uuid,
    ) -> Result<atlas_verification_request::Model, String> {
        let request = Self::find_by_id_global(db, verification_id)
            .await?
            .ok_or_else(|| "Verification request not found".to_string())?;

        let mut active: VerificationRequestActiveModel = request.into();
        active.status = Set(status.to_string());
        if matches!(status, VerificationStatus::Rejected) {
            active.rejection_reason = Set(rejection_reason.map(|s| s.to_string()));
        }
        active.reviewed_by_user_id = Set(Some(reviewed_by_user_id));
        active.reviewed_at = Set(Some(Utc::now()));
        active.update(db).await.map_err(|e| e.to_string())
    }

    /// Append a reviewer note.
    pub async fn add_reviewer_note(
        db: &DatabaseConnection,
        verification_id: Uuid,
        note: &str,
        reviewed_by_user_id: Uuid,
    ) -> Result<atlas_verification_request::Model, String> {
        let note = note.trim();
        if note.is_empty() {
            return Err("Note must not be empty".to_string());
        }

        let request = Self::find_by_id_global(db, verification_id)
            .await?
            .ok_or_else(|| "Verification request not found".to_string())?;

        let stamp = Utc::now().format("%Y-%m-%d %H:%M UTC");
        let entry = format!("[{stamp}] {note}");
        let combined = match request.reviewer_notes.as_ref() {
            Some(existing) if !existing.is_empty() => format!("{existing}\n{entry}"),
            _ => entry,
        };

        let mut active: VerificationRequestActiveModel = request.into();
        active.reviewer_notes = Set(Some(combined));
        active.reviewed_by_user_id = Set(Some(reviewed_by_user_id));
        active.update(db).await.map_err(|e| e.to_string())
    }

    /// Mark the request as needing more info from the applicant.
    pub async fn request_more_info(
        db: &DatabaseConnection,
        verification_id: Uuid,
        message: Option<&str>,
        reviewed_by_user_id: Uuid,
    ) -> Result<atlas_verification_request::Model, String> {
        let request = Self::find_by_id_global(db, verification_id)
            .await?
            .ok_or_else(|| "Verification request not found".to_string())?;

        let mut active: VerificationRequestActiveModel = request.clone().into();
        active.status = Set(VerificationStatus::NeedsInfo.to_string());
        active.reviewed_by_user_id = Set(Some(reviewed_by_user_id));

        if let Some(msg) = message.map(str::trim).filter(|s| !s.is_empty()) {
            let stamp = Utc::now().format("%Y-%m-%d %H:%M UTC");
            let entry = format!("[{stamp}] Request more info: {msg}");
            let combined = match request.reviewer_notes.as_ref() {
                Some(existing) if !existing.is_empty() => format!("{existing}\n{entry}"),
                _ => entry,
            };
            active.reviewer_notes = Set(Some(combined));
        }

        active.update(db).await.map_err(|e| e.to_string())
    }
}
