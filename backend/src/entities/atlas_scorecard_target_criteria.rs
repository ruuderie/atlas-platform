use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

/// G-27 atlas_scorecard_target_criteria — Per-dimension criteria for a target profile.
///
/// Each row defines the ideal / acceptable range for one dimension within a
/// `atlas_scorecard_targets` profile.
///
/// Three constraint types (not mutually exclusive):
///   - Range gate:     `min_score`..`max_score`  — hard filter for search_filter targets
///   - Ideal point:   `ideal_score`              — used for proximity scoring in find_similar
///   - Dealbreaker:   `is_dealbreaker = true`    — candidates outside range are excluded entirely
///   - Search weight: `search_weight`            — overrides the dimension's default weight for this search
///
/// Spec: docs/architecture/platform_generics_v2.md G-27 §7.2
#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "atlas_scorecard_target_criteria")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,

    /// The target profile this criterion belongs to.
    pub target_id: Uuid,

    /// The scorecard dimension this criterion applies to.
    pub dimension_id: Uuid,

    /// Minimum acceptable score for this dimension (inclusive).
    /// `None` means no lower bound.
    pub min_score: Option<rust_decimal::Decimal>,

    /// Maximum acceptable score for this dimension (inclusive).
    /// `None` means no upper bound.
    pub max_score: Option<rust_decimal::Decimal>,

    /// The ideal score for proximity-based ranking.
    /// Used in weighted distance calculation: distance += weight * |actual - ideal|.
    pub ideal_score: Option<rust_decimal::Decimal>,

    /// If `true`, candidates that fail the min/max range for this dimension
    /// are completely excluded from `find_similar` results.
    pub is_dealbreaker: bool,

    /// Per-dimension search weight, overriding the template dimension weight.
    /// `None` → use the dimension's default weight.
    pub search_weight: Option<rust_decimal::Decimal>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::atlas_scorecard_target::Entity",
        from = "Column::TargetId",
        to = "super::atlas_scorecard_target::Column::Id",
        on_update = "NoAction",
        on_delete = "Cascade"
    )]
    Target,
    #[sea_orm(
        belongs_to = "super::atlas_scorecard_dimension::Entity",
        from = "Column::DimensionId",
        to = "super::atlas_scorecard_dimension::Column::Id",
        on_update = "NoAction",
        on_delete = "Cascade"
    )]
    Dimension,
}

impl Related<super::atlas_scorecard_target::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Target.def()
    }
}

impl Related<super::atlas_scorecard_dimension::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Dimension.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}

impl Model {
    /// Returns `true` if the given `score` satisfies this criterion's range constraint.
    ///
    /// Used by the Combinator to implement hard dealbreaker filtering before
    /// running the vector similarity calculation.
    pub fn score_passes_range(&self, score: rust_decimal::Decimal) -> bool {
        if let Some(min) = self.min_score {
            if score < min {
                return false;
            }
        }
        if let Some(max) = self.max_score {
            if score > max {
                return false;
            }
        }
        true
    }

    /// Returns the signed delta between `actual` and `ideal_score`, or `None`
    /// if `ideal_score` is not set for this criterion.
    pub fn delta_from_ideal(&self, actual: rust_decimal::Decimal) -> Option<rust_decimal::Decimal> {
        self.ideal_score.map(|ideal| actual - ideal)
    }
}
