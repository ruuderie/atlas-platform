use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};
use serde_json::Value;

/// Unified Account entity (replaces legacy customer + parts of contact).
///
/// This is the top-level party concept. It can represent either an individual (B2C)
/// or an organization (B2B).
///
/// Designed to work cleanly with the rest of the Platform Generics:
/// - atlas_opportunities
/// - atlas_cases
/// - atlas_contracts
/// - atlas_assets
/// - atlas_applications
/// - atlas_subscriptions
/// - atlas_service_providers
#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "atlas_accounts")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub tenant_id: Uuid,
    /// 'individual' | 'organization'
    pub account_type: String,
    pub name: String,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub primary_contact_id: Option<Uuid>, // FK to atlas_contacts (when implemented)
    pub status: String,
    pub attributes: Option<Value>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
