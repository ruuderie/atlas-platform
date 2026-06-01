#![allow(dead_code, unused_imports)]
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// G-27: atlas_scorecard_dimension_options — poll choices for poll_single/poll_multi dimensions.
///
/// Example: "Which mobile carrier do you use?" → options: Telkomsel, XL, Indosat, Tri
#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "atlas_scorecard_dimension_options")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub dimension_id: Uuid,
    pub tenant_id: Uuid,
    /// Human-readable label displayed to contributors. e.g. "Telkomsel"
    pub label: String,
    /// Stable API slug. e.g. "telkomsel". Used by API consumers for stable references.
    pub value_key: Option<String>,
    pub description: Option<String>,
    pub image_url: Option<String>,
    pub sort_order: i32,
    /// Write-in option — contributor typed it, not a pre-defined choice.
    pub is_write_in: bool,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
