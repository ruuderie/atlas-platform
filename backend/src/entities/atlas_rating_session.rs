use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};

/// G-27: atlas_rating_sessions — one per discrete occurrence.
///
/// The difference between a static profile and a living record. Without sessions,
/// a contractor can only be rated once. With sessions, every job is a data point.
///
/// Sessions optionally link to existing platform records (via context_entity_type +
/// context_entity_id) to avoid data duplication:
///   - contractor job → atlas_case (G-13)
///   - hotel stay    → atlas_reservation (G-23)
///   - event shift   → atlas_events (G-21)
///   - product use   → atlas_catalog_entry (G-26)
///   - call/demo     → atlas_activity (G-29)
#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "atlas_rating_sessions")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub scorecard_id: Uuid,
    pub tenant_id: Uuid,
    pub rater_user_id: Uuid,
    #[sea_orm(column_type = "TimestampWithTimeZone")]
    pub occurred_at: DateTime<Utc>,
    /// 'job' | 'stay' | 'visit' | 'event_shift' | 'purchase' | 'flight' |
    /// 'meeting' | 'pipeline_review' | 'call' | 'email_thread' | 'demo' |
    /// 'monthly_review' | 'quarterly_review'
    pub session_type: String,
    /// e.g. 'atlas_case', 'atlas_reservation', 'atlas_activity'
    pub context_entity_type: Option<String>,
    pub context_entity_id: Option<Uuid>,
    pub session_label: Option<String>,
    /// 'draft' | 'submitted' | 'verified' | 'disputed'
    pub status: String,
    /// G-06 verification gate ID when verification is required
    pub verification_request_id: Option<Uuid>,
    #[sea_orm(column_type = "TimestampWithTimeZone")]
    pub created_at: DateTime<Utc>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
