#![allow(dead_code, unused_imports)]
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

/// G-27: atlas_scorecards — a template applied to one specific entity instance.
///
/// Polymorphic via `subject_entity_type` + `subject_entity_id`. Any entity in the
/// platform can have a scorecard. The UNIQUE constraint on
/// (template_id, subject_entity_type, subject_entity_id) ensures one scorecard
/// per entity per template.
///
/// `composite_score` and `dimension_vector_v2` are recomputed by the background job
/// `recompute_scorecard_aggregates` (5-min interval). Never write these directly.
///
/// `dimension_vector` (legacy JSONB) is preserved for backward compatibility.
/// New code should use `dimension_vector_v2` (float4[]) + `has_data_mask` (bool[]).
/// The v2 masked cosine similarity in The Combinator ignores dimensions where
/// `has_data_mask[i] = false` on either entity being compared.
#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "atlas_scorecards")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub template_id: Uuid,
    /// Type discriminator for the rated entity.
    /// 'atlas_asset' | 'listing' | 'atlas_catalog_entry' | 'atlas_account' |
    /// 'atlas_service_provider' | 'profile' | 'atlas_opportunity' |
    /// 'atlas_lead' | 'atlas_contact' | 'atlas_portfolio'
    pub subject_entity_type: String,
    pub subject_entity_id: Uuid,
    /// Computed weighted composite (None until min_entries_to_publish met).
    /// Recomputed by background job — do NOT write directly.
    #[sea_orm(column_type = "Decimal(Some((5, 2)))", nullable)]
    pub composite_score: Option<Decimal>,
    /// 'insufficient'(<5) | 'low'(<10) | 'medium'(<50) | 'high'(<200) | 'very_high'
    pub confidence_level: String,
    pub total_contributors: i32,
    pub total_sessions: i32,
    pub total_entries: i32,
    /// Legacy: ordered Vec<f64> of weighted normalized scores per dimension (by sort_order).
    /// Zero-filled for dimensions with no entries — use dimension_vector_v2 for new code.
    #[sea_orm(column_type = "JsonBinary", nullable)]
    pub dimension_vector: Option<Value>,
    /// Typed float4 parallel array for masked cosine similarity (Gap 3 fix).
    ///
    /// Values: normalized (score - scale_min) / (scale_max - scale_min) * weight.
    /// For dimensions with no data: 0.5 * weight (midpoint placeholder, NOT sentinel zero).
    /// Only Rating/Absolute/Boolean dimensions are included; Poll dims are excluded.
    /// Updated alongside dimension_vector by recompute_scorecard_aggregates.
    ///
    /// SeaORM note: float4[] stored as JSONB in entity layer; converted to/from Vec<f32>
    /// in service code via serde_json::from_value / serde_json::to_value.
    #[sea_orm(column_type = "JsonBinary", nullable)]
    pub dimension_vector_v2: Option<Value>,
    /// Parallel bool array to dimension_vector_v2.
    ///
    /// true = this dimension has at least one verified entry (real observed data).
    /// false = no data yet; vector value is the midpoint placeholder.
    ///
    /// The Combinator requires >= 30% overlap (both masks true) to compute similarity.
    /// Stored as JSONB (Vec<bool>) in SeaORM; typed in service layer.
    #[sea_orm(column_type = "JsonBinary", nullable)]
    pub has_data_mask: Option<Value>,
    #[sea_orm(column_type = "TimestampWithTimeZone", nullable)]
    pub last_computed_at: Option<DateTime<Utc>>,
    #[sea_orm(column_type = "TimestampWithTimeZone")]
    pub created_at: DateTime<Utc>,
    #[sea_orm(column_type = "TimestampWithTimeZone")]
    pub updated_at: DateTime<Utc>,
    /// Soft-delete sentinel. NULL = active; non-null = archived.
    /// Set by DELETE /api/scorecards/:id — never hard-deletes.
    /// All views and queries should filter `WHERE deleted_at IS NULL`.
    #[sea_orm(column_type = "TimestampWithTimeZone", nullable)]
    pub deleted_at: Option<DateTime<Utc>>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}

// ── Typed JSONB helpers ───────────────────────────────────────────────────────

impl Model {
    /// Deserialize `dimension_vector` (legacy) into a `Vec<f64>`.
    ///
    /// Returns `None` if the column is NULL. Returns `Err` if the stored
    /// value is not a JSON array of numbers — this indicates data corruption
    /// and should be treated as a hard error in the recompute pipeline.
    pub fn dimension_vector_typed(&self) -> Result<Option<Vec<f64>>, serde_json::Error> {
        match &self.dimension_vector {
            Some(v) => serde_json::from_value(v.clone()).map(Some),
            None => Ok(None),
        }
    }

    /// Deserialize `dimension_vector_v2` (masked cosine similarity vector) into `Vec<f32>`.
    ///
    /// # Safety Contract
    /// The length of the returned vec MUST equal the number of active non-Poll
    /// dimensions for `self.template_id`. The Combinator assumes this invariant
    /// and panics (via `zip`) if lengths diverge.
    ///
    /// Returns `None` if the column is NULL (scorecard not yet computed).
    pub fn dimension_vector_v2_typed(&self) -> Result<Option<Vec<f32>>, serde_json::Error> {
        match &self.dimension_vector_v2 {
            Some(v) => serde_json::from_value(v.clone()).map(Some),
            None => Ok(None),
        }
    }

    /// Deserialize `has_data_mask` into `Vec<bool>`.
    ///
    /// `true` at index `i` means dimension `i` has at least one verified entry.
    /// `false` means the vector value is the midpoint placeholder — this dimension
    /// is excluded from The Combinator's cosine similarity when either entity's mask is false.
    ///
    /// Returns `None` if the column is NULL (scorecard not yet computed).
    pub fn has_data_mask_typed(&self) -> Result<Option<Vec<bool>>, serde_json::Error> {
        match &self.has_data_mask {
            Some(v) => serde_json::from_value(v.clone()).map(Some),
            None => Ok(None),
        }
    }

    /// Parse `confidence_level` into the typed `ConfidenceLevel` enum.
    ///
    /// Returns `Err` if the stored value is not a known variant — this should
    /// never happen in production but is checked defensively.
    pub fn confidence_level_typed(
        &self,
    ) -> Result<crate::types::scorecard::ConfidenceLevel, String> {
        crate::types::scorecard::ConfidenceLevel::try_from(self.confidence_level.clone())
    }

    /// Parse `subject_entity_type` into the typed `ScorecardEntityType` enum.
    pub fn subject_entity_type_typed(
        &self,
    ) -> Result<crate::types::scorecard::ScorecardEntityType, String> {
        crate::types::scorecard::ScorecardEntityType::try_from(self.subject_entity_type.clone())
    }

    /// Returns `true` if this scorecard has been computed at least once
    /// and has enough entries to produce a non-trivial composite score.
    pub fn is_published(&self) -> bool {
        self.composite_score.is_some() && self.last_computed_at.is_some()
    }
}
