use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

/// G-27 atlas_scorecard_targets — Target profiles for The Combinator similarity search.
///
/// A target profile is a named vector of per-dimension ideal values that the
/// Combinator uses to rank scorecards by closeness. Three profile types are supported:
///
///   - `search_filter`   — hard filters (min/max range per dimension)
///   - `job_specification` — ideal profile derived from a job description
///   - `ideal_profile`   — aggregated vector from `seed_entity_ids` (top performers)
///
/// Target profiles are tenant-scoped: each tenant defines its own ideal profiles
/// for its templates. Cross-tenant profiles are not permitted.
///
/// Relationship to find_similar:
///   Callers can either supply a raw `target_vector` to `find_similar`, or
///   store a reusable target profile here and look it up before calling.
///
/// Spec: docs/architecture/platform_generics_v2.md G-27 §7 (The Combinator)
#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "atlas_scorecard_targets")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,

    /// The scorecard template this target belongs to.
    pub template_id: Uuid,

    /// Tenant that owns this target profile.
    pub tenant_id: Uuid,

    /// Human-readable name: "Ideal FMCSA Carrier", "Senior Staff Engineer".
    pub name: String,

    /// Discriminator: `search_filter` | `job_specification` | `ideal_profile`
    pub target_type: String,

    /// Optional narrative description of the target profile.
    pub description: Option<String>,

    /// For `ideal_profile`: UUIDs of the top-performing subjects whose dimension
    /// vectors were averaged to derive `target_vector`.
    /// JSONB array of UUID strings.
    pub seed_entity_ids: Option<serde_json::Value>,

    /// Precomputed target vector: JSONB array of f64 values in dimension sort_order.
    /// Populated on save and invalidated when seed_entity_ids changes.
    pub target_vector: Option<serde_json::Value>,

    /// User who created this target profile (nullable for system-generated profiles).
    pub created_by_user_id: Option<Uuid>,

    pub created_at: DateTimeWithTimeZone,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::atlas_scorecard_template::Entity",
        from = "Column::TemplateId",
        to = "super::atlas_scorecard_template::Column::Id",
        on_update = "NoAction",
        on_delete = "Cascade"
    )]
    Template,
}

impl Related<super::atlas_scorecard_template::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Template.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}

impl Model {
    /// Parse the target_vector JSONB field into a `Vec<f64>` for use in
    /// `ScorecardService::find_similar`.
    ///
    /// Returns `None` if the field is absent or contains non-numeric values.
    pub fn parse_target_vector(&self) -> Option<Vec<f64>> {
        self.target_vector.as_ref()?.as_array().map(|arr| {
            arr.iter()
                .map(|v| v.as_f64().unwrap_or(0.0))
                .collect()
        })
    }

    /// Returns `true` if this target has a computable vector (i.e. it can be
    /// passed directly to `find_similar` without first aggregating seed entities).
    pub fn has_precomputed_vector(&self) -> bool {
        self.target_vector.as_ref()
            .and_then(|v| v.as_array())
            .map(|arr| !arr.is_empty())
            .unwrap_or(false)
    }
}
