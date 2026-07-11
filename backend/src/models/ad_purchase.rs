#![allow(dead_code, unused)]
use chrono::{DateTime, Utc};
use sea_orm::DeriveActiveEnum;
use sea_orm::prelude::*;
use serde::{Deserialize, Serialize};
use strum_macros::Display;
use uuid::Uuid;

#[derive(Debug, Deserialize, Clone, PartialEq)]
pub struct AdPurchase {
    pub id: uuid::Uuid,
    pub profile_id: uuid::Uuid,
    pub listing_id: uuid::Uuid,
    pub start_date: DateTime<Utc>,
    pub end_date: DateTime<Utc>,
    pub status: AdStatus,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub price: f32,
}
#[derive(Debug, Deserialize, Clone, PartialEq)]
pub struct AdPurchaseCreate {
    pub profile_id: Uuid,
    pub listing_id: Uuid,
    pub start_date: DateTime<Utc>,
    pub end_date: DateTime<Utc>,
    pub status: AdStatus,
    pub price: f32,
}

#[derive(Debug, Deserialize, Clone, PartialEq)]
pub struct AdPurchaseUpdate {
    pub profile_id: Uuid,
    pub listing_id: Uuid,
    pub start_date: DateTime<Utc>,
    pub end_date: DateTime<Utc>,
    pub price: f32,
    pub status: AdStatus,
}

#[derive(
    Debug, Clone, PartialEq, Eq, DeriveActiveEnum, Serialize, Deserialize, Display, EnumIter,
)]
#[sea_orm(rs_type = "String", db_type = "Enum", enum_name = "ad_status")]
pub enum AdStatus {
    #[sea_orm(string_value = "pending")]
    Pending,
    #[sea_orm(string_value = "active")]
    Active,
    #[sea_orm(string_value = "expired")]
    Expired,
    #[sea_orm(string_value = "cancelled")]
    Cancelled,
}
