#![allow(dead_code, unused_imports)]
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};

/// GENERIC-04: AtlasSubscription
///
/// B2C recurring subscriptions (creator tiers, plans, etc.).
/// Distinct from the platform's own B2B tenant subscriptions.
#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "atlas_subscriptions")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub subscriber_user_id: Uuid,
    pub subscribed_to_type: String,
    pub subscribed_to_id: Uuid,
    pub billing_plan_id: Option<Uuid>,
    pub price_cents: i64,
    pub currency: String,
    pub billing_interval: String,
    pub stripe_subscription_id: Option<String>,
    pub stripe_customer_id: Option<String>,
    pub status: SubscriptionStatus,
    pub trial_ends_at: Option<DateTime<Utc>>,
    pub current_period_start: Option<DateTime<Utc>>,
    pub current_period_end: Option<DateTime<Utc>>,
    pub canceled_at: Option<DateTime<Utc>>,
    pub is_billing_exempt: bool,
    pub billing_exemption_reason: Option<String>,
    pub grace_period_ends_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq, Eq, EnumIter, DeriveActiveEnum, Serialize, Deserialize, strum_macros::Display)]
#[sea_orm(rs_type = "String", db_type = "String(StringLen::N(50))")]
pub enum SubscriptionStatus {
    #[sea_orm(string_value = "active")]
    Active,
    #[sea_orm(string_value = "trial")]
    Trial,
    #[sea_orm(string_value = "past_due")]
    PastDue,
    #[sea_orm(string_value = "suspended")]
    Suspended,
    #[sea_orm(string_value = "canceled")]
    Canceled,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
