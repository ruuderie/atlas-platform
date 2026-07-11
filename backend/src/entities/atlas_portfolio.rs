#![allow(dead_code, unused_imports)]
use chrono::{DateTime, Utc};
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

/// GENERIC-09: AtlasPortfolio
/// Groups assets (properties, vehicles, equipment, etc.) for reporting, billing, and access control.
/// This replaces the need for app-specific `pm_portfolios` / similar tables across verticals.
#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "atlas_portfolios")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub owner_user_id: Uuid,
    /// Discriminator for different portfolio kinds (e.g. "real_estate", "vehicle_fleet")
    pub portfolio_type: String,
    pub name: String,
    pub description: Option<String>,
    /// PMC: which client account owns this portfolio. NULL = PM's own. (m20260817)
    pub managed_account_id: Option<Uuid>,
    /// Flexible, app-specific configuration (e.g. reporting settings, visibility rules)
    pub metadata: Option<Value>,
    pub created_at: DateTime<Utc>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
