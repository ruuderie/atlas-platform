//! # G22 RecordRelationshipService — Universal M:M Junction Table
//!
//! ## Scope
//!
//! Creates and queries labeled relationships between any two platform entities.
//! This is the Salesforce Junction Object pattern — enabling cross-entity
//! Related Lists without per-combination join tables.
//!
//! ## Common relationship types
//!
//! | source_entity_type | target_entity_type | relationship_type |
//! |--------------------|--------------------|--------------------|
//! | `atlas_campaigns` | `atlas_assets` | `promotes` |
//! | `atlas_events` | `atlas_service_providers` | `hosted_by` |
//! | `atlas_cases` | `atlas_contracts` | `referenced_in` |
//! | `atlas_campaigns` | `atlas_events` | `includes_event` |
//! | `atlas_opportunities` | `atlas_contacts` | `influenced_by` |
//!
//! ## Traversal
//!
//! - `find_targets()` — "what does this entity link TO?" (forward)
//! - `find_sources()` — "what links TO this entity?" (reverse / Related List)
//!
//! Both are indexed — O(log n) in both directions.

use anyhow::{Result, anyhow};
use chrono::Utc;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, QueryOrder, Set,
};
use uuid::Uuid;

use crate::entities::atlas_record_relationship;

// ── Input types ───────────────────────────────────────────────────────────────

#[derive(Debug, Clone, serde::Deserialize)]
pub struct CreateRelationshipPayload {
    pub source_entity_type: String,
    pub source_entity_id: Uuid,
    pub target_entity_type: String,
    pub target_entity_id: Uuid,
    /// Named label for the relationship. e.g. "promotes", "references", "attended_by".
    pub relationship_type: String,
    /// Human-readable label for the reverse direction. Optional.
    pub inverse_label: Option<String>,
    /// Free-form metadata. e.g. { "sort_order": 1, "weight": 0.8 }
    pub relationship_metadata: Option<serde_json::Value>,
    pub created_by_user_id: Option<Uuid>,
}

// ── Service ───────────────────────────────────────────────────────────────────

pub struct RecordRelationshipService;

impl RecordRelationshipService {
    /// Create a relationship between two entities.
    /// Returns `Err` if the relationship already exists (unique constraint).
    pub async fn create(
        db: &DatabaseConnection,
        tenant_id: Uuid,
        payload: CreateRelationshipPayload,
    ) -> Result<atlas_record_relationship::Model> {
        let active = atlas_record_relationship::ActiveModel {
            id: Set(Uuid::new_v4()),
            tenant_id: Set(tenant_id),
            source_entity_type: Set(payload.source_entity_type.clone()),
            source_entity_id: Set(payload.source_entity_id),
            target_entity_type: Set(payload.target_entity_type.clone()),
            target_entity_id: Set(payload.target_entity_id),
            relationship_type: Set(payload.relationship_type.clone()),
            inverse_label: Set(payload.inverse_label),
            relationship_metadata: Set(payload.relationship_metadata),
            created_by_user_id: Set(payload.created_by_user_id),
            created_at: Set(Utc::now()),
        };

        active.insert(db).await.map_err(|e| {
            // Surface duplicate constraint violations with a useful message.
            if e.to_string().contains("unique") || e.to_string().contains("duplicate") {
                anyhow!(
                    "Relationship '{}' between {}:{} and {}:{} already exists",
                    payload.relationship_type,
                    payload.source_entity_type,
                    payload.source_entity_id,
                    payload.target_entity_type,
                    payload.target_entity_id,
                )
            } else {
                anyhow!("create relationship failed: {e:#}")
            }
        })
    }

    /// Create if not exists — idempotent upsert variant.
    /// Returns the existing or newly created relationship.
    pub async fn upsert(
        db: &DatabaseConnection,
        tenant_id: Uuid,
        payload: CreateRelationshipPayload,
    ) -> Result<atlas_record_relationship::Model> {
        // Try to find existing first.
        if let Some(existing) = Self::find_one(
            db,
            tenant_id,
            &payload.source_entity_type,
            payload.source_entity_id,
            &payload.target_entity_type,
            payload.target_entity_id,
            &payload.relationship_type,
        )
        .await?
        {
            return Ok(existing);
        }
        Self::create(db, tenant_id, payload).await
    }

    /// Delete a specific relationship. Returns `Ok(true)` if deleted, `Ok(false)` if not found.
    pub async fn delete(
        db: &DatabaseConnection,
        tenant_id: Uuid,
        source_entity_type: &str,
        source_entity_id: Uuid,
        target_entity_type: &str,
        target_entity_id: Uuid,
        relationship_type: &str,
    ) -> Result<bool> {
        let result = atlas_record_relationship::Entity::delete_many()
            .filter(atlas_record_relationship::Column::TenantId.eq(tenant_id))
            .filter(atlas_record_relationship::Column::SourceEntityType.eq(source_entity_type))
            .filter(atlas_record_relationship::Column::SourceEntityId.eq(source_entity_id))
            .filter(atlas_record_relationship::Column::TargetEntityType.eq(target_entity_type))
            .filter(atlas_record_relationship::Column::TargetEntityId.eq(target_entity_id))
            .filter(atlas_record_relationship::Column::RelationshipType.eq(relationship_type))
            .exec(db)
            .await?;
        Ok(result.rows_affected > 0)
    }

    // ── Traversal ─────────────────────────────────────────────────────────────

    /// Forward traversal: find all records that `source` links TO via the given
    /// relationship type. Used for "show me all assets this campaign promotes".
    pub async fn find_targets(
        db: &DatabaseConnection,
        tenant_id: Uuid,
        source_entity_type: &str,
        source_entity_id: Uuid,
        relationship_type: &str,
    ) -> Result<Vec<atlas_record_relationship::Model>> {
        Ok(atlas_record_relationship::Entity::find()
            .filter(atlas_record_relationship::Column::TenantId.eq(tenant_id))
            .filter(atlas_record_relationship::Column::SourceEntityType.eq(source_entity_type))
            .filter(atlas_record_relationship::Column::SourceEntityId.eq(source_entity_id))
            .filter(atlas_record_relationship::Column::RelationshipType.eq(relationship_type))
            .order_by_asc(atlas_record_relationship::Column::CreatedAt)
            .all(db)
            .await?)
    }

    /// Reverse traversal (Related List): find all records that link TO `target`.
    /// Used for "show me all campaigns that promote this asset".
    pub async fn find_sources(
        db: &DatabaseConnection,
        tenant_id: Uuid,
        target_entity_type: &str,
        target_entity_id: Uuid,
        relationship_type: &str,
    ) -> Result<Vec<atlas_record_relationship::Model>> {
        Ok(atlas_record_relationship::Entity::find()
            .filter(atlas_record_relationship::Column::TenantId.eq(tenant_id))
            .filter(atlas_record_relationship::Column::TargetEntityType.eq(target_entity_type))
            .filter(atlas_record_relationship::Column::TargetEntityId.eq(target_entity_id))
            .filter(atlas_record_relationship::Column::RelationshipType.eq(relationship_type))
            .order_by_asc(atlas_record_relationship::Column::CreatedAt)
            .all(db)
            .await?)
    }

    /// All relationships of any type touching a given entity (both source and target).
    /// Useful for a "Related Records" panel showing every linked entity.
    pub async fn find_all_for_entity(
        db: &DatabaseConnection,
        tenant_id: Uuid,
        entity_type: &str,
        entity_id: Uuid,
    ) -> Result<Vec<atlas_record_relationship::Model>> {
        use sea_orm::Condition;
        Ok(atlas_record_relationship::Entity::find()
            .filter(atlas_record_relationship::Column::TenantId.eq(tenant_id))
            .filter(
                Condition::any()
                    .add(
                        Condition::all()
                            .add(
                                atlas_record_relationship::Column::SourceEntityType.eq(entity_type),
                            )
                            .add(atlas_record_relationship::Column::SourceEntityId.eq(entity_id)),
                    )
                    .add(
                        Condition::all()
                            .add(
                                atlas_record_relationship::Column::TargetEntityType.eq(entity_type),
                            )
                            .add(atlas_record_relationship::Column::TargetEntityId.eq(entity_id)),
                    ),
            )
            .order_by_asc(atlas_record_relationship::Column::CreatedAt)
            .all(db)
            .await?)
    }

    // ── Internal helpers ──────────────────────────────────────────────────────

    async fn find_one(
        db: &DatabaseConnection,
        tenant_id: Uuid,
        source_entity_type: &str,
        source_entity_id: Uuid,
        target_entity_type: &str,
        target_entity_id: Uuid,
        relationship_type: &str,
    ) -> Result<Option<atlas_record_relationship::Model>> {
        Ok(atlas_record_relationship::Entity::find()
            .filter(atlas_record_relationship::Column::TenantId.eq(tenant_id))
            .filter(atlas_record_relationship::Column::SourceEntityType.eq(source_entity_type))
            .filter(atlas_record_relationship::Column::SourceEntityId.eq(source_entity_id))
            .filter(atlas_record_relationship::Column::TargetEntityType.eq(target_entity_type))
            .filter(atlas_record_relationship::Column::TargetEntityId.eq(target_entity_id))
            .filter(atlas_record_relationship::Column::RelationshipType.eq(relationship_type))
            .one(db)
            .await?)
    }
}
