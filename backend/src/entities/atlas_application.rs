use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};
use serde_json::Value;

/// GENERIC-18: AtlasApplication
/// Structured multi-step intake and onboarding workflows.
#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "atlas_applications")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub application_type: String,
    pub applicant_user_id: Uuid,
    pub target_asset_id: Option<Uuid>,
    pub target_opportunity_id: Option<Uuid>,
    pub target_program: Option<String>,
    pub status: String,
    pub primary_application_id: Option<Uuid>,
    pub monthly_income_cents: Option<i64>,
    pub income_currency: String,
    pub national_id_type: Option<String>,
    pub national_id_last4: Option<String>,
    pub screening_status: String,
    pub screening_provider: Option<String>,
    pub screening_passed: Option<bool>,
    pub disclosures_accepted: Option<Value>,
    pub application_metadata: Option<Value>,
    pub submitted_at: Option<DateTime<Utc>>,
    pub decided_at: Option<DateTime<Utc>>,
    pub decision_reason: Option<String>,
    pub resulting_contract_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
