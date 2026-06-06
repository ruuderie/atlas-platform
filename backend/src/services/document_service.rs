#![allow(dead_code, unused)]

use sea_orm::{DatabaseConnection, EntityTrait, ActiveModelTrait, Set, QueryFilter, ColumnTrait, QuerySelect};
use uuid::Uuid;
use chrono::Utc;

use crate::entities::atlas_document::{self, Entity as DocumentEntity, ActiveModel as DocumentActiveModel};

/// Service layer for GENERIC-14: AtlasDocument
/// Generic document registry with e-signature, versioning, and app_namespace scoping.
/// Used by contracts, applications, cases, regulatory items, etc.
pub struct DocumentService;

impl DocumentService {
    /// Create / register a new document (metadata only; actual bytes via vault attachment).
    /// Uses the polymorphic related_entity_* fields + attachment_id (required).
    pub async fn create_document(
        db: &DatabaseConnection,
        tenant_id: Uuid,
        app_namespace: &str,
        document_category: &str,
        attachment_id: Uuid,
        related_entity_type: Option<&str>,
        related_entity_id: Option<Uuid>,
    ) -> Result<Uuid, String> {
        let doc = DocumentActiveModel {
            id: Set(Uuid::new_v4()),
            tenant_id: Set(tenant_id),
            attachment_id: Set(attachment_id),
            app_namespace: Set(app_namespace.to_string()),
            document_category: Set(document_category.to_string()),
            related_entity_type: Set(related_entity_type.map(|s| s.to_string())),
            related_entity_id: Set(related_entity_id),
            is_counterparty_visible: Set(false),
            requires_signature: Set(false),
            is_signed: Set(false),
            version_number: Set(1),
            created_at: Set(Utc::now()),
            ..Default::default()
        };

        let result = doc.insert(db).await.map_err(|e| e.to_string())?;
        Ok(result.id)
    }

    pub async fn find_by_id(
        db: &DatabaseConnection,
        tenant_id: Uuid,
        document_id: Uuid,
    ) -> Result<Option<atlas_document::Model>, String> {
        DocumentEntity::find()
            .filter(atlas_document::Column::TenantId.eq(tenant_id))
            .filter(atlas_document::Column::Id.eq(document_id))
            .one(db)
            .await
            .map_err(|e| e.to_string())
    }

    pub async fn list_for_tenant(
        db: &DatabaseConnection,
        tenant_id: Uuid,
        limit: u64,
    ) -> Result<Vec<atlas_document::Model>, String> {
        DocumentEntity::find()
            .filter(atlas_document::Column::TenantId.eq(tenant_id))
            .limit(limit)
            .all(db)
            .await
            .map_err(|e| e.to_string())
    }

    /// Request e-signature on a document (would enqueue via verification or realtime).
    pub async fn request_esignature(
        _db: &DatabaseConnection,
        _tenant_id: Uuid,
        document_id: Uuid,
        signer_contact_id: Uuid,
    ) -> Result<(), String> {
        tracing::info!(
            "e-signature requested for document {} by contact {}",
            document_id, signer_contact_id
        );
        Ok(())
    }
}