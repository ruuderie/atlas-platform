use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};
use serde_json::Value;

/// GENERIC-16: AtlasRegulatoryRegistration
/// Government permits, licenses, and regulatory registrations.
#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "atlas_regulatory_registrations")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub registration_type: String,
    pub asset_id: Option<Uuid>,
    pub service_provider_id: Option<Uuid>,
    pub jurisdiction_code: String,
    pub issuing_authority: Option<String>,
    pub registration_number: String,
    pub verification_request_id: Option<Uuid>,
    pub status: String,
    pub issued_date: Option<chrono::NaiveDate>,
    pub expires_at: Option<chrono::NaiveDate>,
    pub last_inspection_date: Option<chrono::NaiveDate>,
    pub next_inspection_due: Option<chrono::NaiveDate>,
    pub access_token: Option<String>,
    pub access_token_expires_at: Option<DateTime<Utc>>,
    pub jurisdiction_metadata: Option<Value>,
    pub created_at: DateTime<Utc>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
