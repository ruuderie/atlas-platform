#![allow(dead_code, unused_imports)]
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

/// G-27: atlas_scorecard_entries — sparse scores per dimension per session.
///
/// One row per (session_id, dimension_id, contributor_user_id).
/// Sparse by design: contributors submit only the dimensions they have direct
/// experience with. The UNIQUE constraint enforces one entry per person per
/// dimension per session at the database level.
///
/// Exactly one of `score` or `option_id` must be non-null:
///   - rating / absolute / boolean dimensions → use `score`
///   - poll_single / poll_multi dimensions    → use `option_id`
///
/// This invariant is enforced by ScorecardService::submit_entry.
///
/// `context` JSONB carries credibility signals used in weighted aggregation:
///   - community_rating: {"visit_start":"2024-03","duration_days":90,"purpose":"work"}
///   - peer_review:      {"relationship":"peer","worked_together_months":18}
///   - test_result:      {"test_name":"CRT","date":"2024-01","administered_by":"HR"}
///   - manager_review:   {"call_recording_url":"...","reviewed_at":"2024-01-15"}
#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "atlas_scorecard_entries")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub session_id: Uuid,
    pub scorecard_id: Uuid,
    pub dimension_id: Uuid,
    pub tenant_id: Uuid,
    pub contributor_user_id: Uuid,
    /// For rating / absolute / boolean dimensions. None for poll dimensions.
    #[sea_orm(column_type = "Decimal(Some((8, 2)))", nullable)]
    pub score: Option<Decimal>,
    /// For poll_single / poll_multi dimensions. None for numeric dimensions.
    pub option_id: Option<Uuid>,
    /// 'community_rating' | 'peer_review' | 'self_assessment' |
    /// 'manager_review' | 'test_result' | 'behavioral_signal' | 'official_data'
    pub source_type: String,
    /// Source-type specific credibility context — drives weighted aggregation.
    #[sea_orm(column_type = "JsonBinary", nullable)]
    pub context: Option<Value>,
    pub note: Option<String>,
    pub is_verified: bool,
    pub verification_request_id: Option<Uuid>,
    #[sea_orm(column_type = "TimestampWithTimeZone")]
    pub created_at: DateTime<Utc>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
