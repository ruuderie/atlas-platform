use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;

/// G-27: atlas_scorecard_poll_aggregates — vote counts for poll_single/poll_multi dimensions.
///
/// PRIMARY KEY: (scorecard_id, dimension_id, option_id)
/// Recomputed atomically by `ScorecardService::recompute_aggregates`.
/// One row per option per dimension per scorecard.
#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "atlas_scorecard_poll_aggregates")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub scorecard_id: Uuid,
    #[sea_orm(primary_key, auto_increment = false)]
    pub dimension_id: Uuid,
    #[sea_orm(primary_key, auto_increment = false)]
    pub option_id: Uuid,
    pub vote_count: i32,
    /// Percentage of all voters who chose this option.
    #[sea_orm(column_type = "Decimal(Some((5, 2)))", nullable)]
    pub vote_pct: Option<Decimal>,
    /// Rank by vote_count descending. 1 = most voted.
    pub rank: i32,
    /// Total unique voters for this dimension (denominator for vote_pct).
    pub total_voters: i32,
    #[sea_orm(column_type = "TimestampWithTimeZone", nullable)]
    pub last_computed_at: Option<DateTime<Utc>>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
