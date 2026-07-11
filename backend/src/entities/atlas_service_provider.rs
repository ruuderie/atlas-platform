#![allow(dead_code, unused_imports)]
use chrono::{DateTime, Utc};
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

/// GENERIC-12: AtlasServiceProvider
/// Registry for vendors, contractors, adjusters, agents, etc.
///
/// Supports three scopes:
/// - tenant: hired exclusively by one operator
/// - platform: available across the Atlas network
/// - marketplace: self-listed
#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "atlas_service_providers")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub user_id: Uuid,
    pub scope: String,
    pub business_name: Option<String>,
    pub service_categories: Value,
    pub status: String,
    pub rating_avg: Option<f64>,
    pub rating_count: i32,
    pub preferred_payment_rail: Option<String>,
    pub btc_wallet_address: Option<String>,
    pub stripe_connect_id: Option<String>,
    pub is_insured: bool,
    pub is_bonded: bool,
    pub profile_metadata: Option<Value>,
    pub notes: Option<String>,
    pub created_at: DateTime<Utc>,
    // ── G-34: Vendor Marketplace fields (m20260819) ───────────────────────────
    /// When true, this vendor is discoverable cross-tenant in the marketplace.
    /// Default false — opt-in only.
    pub is_marketplace_visible: bool,
    /// Short public-facing bio shown on marketplace cards (max 500 chars).
    pub marketplace_bio: Option<String>,
    /// Trade type slugs advertised in the marketplace (TEXT[] in Postgres).
    /// Sea-ORM maps TEXT[] as Vec<String>.
    pub marketplace_trade_types: Vec<String>,
    /// WKT POINT string for PostGIS proximity queries.
    /// e.g. "SRID=4326;POINT(-80.1918 25.7617)"
    pub marketplace_location: Option<String>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
