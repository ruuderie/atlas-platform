#![allow(dead_code, unused_imports)]
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};
use serde_json::Value;

/// GENERIC-06: AtlasVerificationRequest
///
/// Tracks verification workflows that may start automated and escalate to human review.
/// Used for licenses, permits, identity documents, GPS fraud checks, etc.
#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "atlas_verification_requests")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub subject_type: String,
    pub subject_id: Uuid,
    pub requested_by_user_id: Uuid,
    pub attachment_id: Option<Uuid>,
    pub auto_check_result: Option<Value>,
    pub auto_check_passed: Option<bool>,
    pub status: String,
    pub reviewed_by_user_id: Option<Uuid>,
    pub reviewed_at: Option<DateTime<Utc>>,
    pub rejection_reason: Option<String>,
    pub verified_value: Option<String>,
    pub expires_at: Option<chrono::NaiveDate>,
    pub created_at: DateTime<Utc>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
