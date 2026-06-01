use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::NaiveDate;
use rust_decimal::Decimal;

/// G-27: atlas_scorecard_time_series — monthly/quarterly trend buckets per dimension.
///
/// PRIMARY KEY: (scorecard_id, dimension_id, period_type, period_start)
/// Refreshed hourly by `refresh_scorecard_time_series` background job.
///
/// Trend direction logic:
///   - 'improving':         current mean > prior mean + 0.3
///   - 'declining':         current mean < prior mean - 0.3
///   - 'stable':            within ±0.3 of prior mean
///   - 'insufficient_data': < 2 entries in the period
///
/// The time series is what separates a living record from a static snapshot.
/// An owner sees bathroom cleanliness declining month-over-month before it
/// becomes a problem visible in the overall score.
#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "atlas_scorecard_time_series")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub scorecard_id: Uuid,
    #[sea_orm(primary_key, auto_increment = false)]
    pub dimension_id: Uuid,
    /// First day of the period: always the 1st of the month for 'monthly'.
    #[sea_orm(primary_key, auto_increment = false)]
    pub period_start: NaiveDate,
    /// 'monthly' | 'quarterly'
    #[sea_orm(primary_key, auto_increment = false)]
    pub period_type: String,
    #[sea_orm(column_type = "Decimal(Some((5, 2)))", nullable)]
    pub mean_score: Option<Decimal>,
    pub session_count: i32,
    pub contributor_count: i32,
    /// Mean score delta from the previous period. None for the first period.
    #[sea_orm(column_type = "Decimal(Some((5, 2)))", nullable)]
    pub delta_from_prior: Option<Decimal>,
    /// 'improving' | 'stable' | 'declining' | 'insufficient_data'
    pub trend_direction: Option<String>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
