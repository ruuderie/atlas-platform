use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};
use serde_json::Value;

/// GENERIC-05: AtlasExternalIntegration
///
/// Registry of third-party integrations (PMS, OTA, AMS, GDS, Telephony, etc.).
///
/// Credentials are stored encrypted (application-layer encryption recommended).
/// The platform does not hardcode any specific providers — integration_type
/// is treated as an open string with common examples documented in the schema.
#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "atlas_external_integrations")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub integration_type: String,
    pub label: Option<String>,
    pub credentials_encrypted: Value,
    pub webhook_secret: Option<String>,
    pub webhook_url: Option<String>,
    pub is_active: bool,
    pub last_synced_at: Option<DateTime<Utc>>,
    pub last_error: Option<String>,
    pub config: Option<Value>,
    pub created_at: DateTime<Utc>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
