//! G-37 — Ambassador ↔ campaign attach (M:N).

use chrono::{DateTime, Utc};
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "atlas_ambassador_campaigns")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub ambassador_id: Uuid,
    #[sea_orm(primary_key, auto_increment = false)]
    pub campaign_id: Uuid,
    pub created_at: DateTime<Utc>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
