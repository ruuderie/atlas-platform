#![allow(dead_code, unused_imports)]
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "atlas_record_relationships")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub tenant_id: Uuid,

    // ── Source entity ─────────────────────────────────────────────────────────
    /// The entity type originating the relationship.
    /// e.g. "atlas_campaigns", "atlas_events", "atlas_opportunities"
    pub source_entity_type: String,
    pub source_entity_id: Uuid,

    // ── Target entity ─────────────────────────────────────────────────────────
    /// The entity type on the other end of the relationship.
    pub target_entity_type: String,
    pub target_entity_id: Uuid,

    // ── Relationship label ────────────────────────────────────────────────────
    /// Named relationship type. e.g. "promotes", "attended_by", "generated_from".
    /// Together with the entity pair, this forms the unique constraint.
    pub relationship_type: String,
    /// Human-readable label for the reverse traversal direction.
    /// e.g. if forward is "promotes", inverse might be "promoted_by".
    pub inverse_label: Option<String>,

    // ── Metadata ──────────────────────────────────────────────────────────────
    /// Free-form context: { sort_order, weight, notes, ... }
    pub relationship_metadata: Option<serde_json::Value>,

    // ── Audit ─────────────────────────────────────────────────────────────────
    pub created_by_user_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
