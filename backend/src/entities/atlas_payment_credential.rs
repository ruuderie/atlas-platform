use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};
use serde_json::Value;

/// GENERIC-03: AtlasPaymentCredential
///
/// Stores encrypted credentials for various payment rails in a provider-agnostic way.
///
/// The platform deliberately does not hardcode attachment to any specific payment provider.
/// The `credential_type` values are illustrative — new types can be supported by adding
/// to the enum (via migration) or by treating the field more flexibly in service code.
///
/// Bitcoin support is intentionally designed to allow future migration from
/// third-party services to self-hosted nodes/infrastructure.
#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "atlas_payment_credentials")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub credential_type: String,
    pub mor_type: String, // 'platform', 'client', 'hybrid'
    pub label: Option<String>,
    pub credentials_encrypted: Value,
    pub display_identifier: Option<String>,
    pub display_name: Option<String>,
    pub is_active: bool,
    pub is_default_for_type: bool,
    pub is_verified: bool,
    pub verified_at: Option<DateTime<Utc>>,
    pub payout_currency: String,
    pub payout_minimum_cents: i64,
    pub webhook_secret_enc: Option<String>,
    pub created_by_user_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
