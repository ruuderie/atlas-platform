use chrono::{Utc, DateTime, Duration};
use uuid::Uuid;
use sea_orm::DeriveActiveEnum;
use serde::{Serialize, Deserialize};
use sea_orm::prelude::*;
use strum_macros::Display;

#[derive(Debug, Deserialize, Clone, PartialEq)]
pub struct AdPurchase {
    id: uuid::Uuid,
    profile_id: uuid::Uuid,
    listing_id: uuid::Uuid,
    content: String,
    start_date: DateTime<Utc>,
    end_date: DateTime<Utc>,
    status: AdStatus,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
    price: f32,

}
#[derive(Debug, Deserialize, Clone, PartialEq)]
pub struct AdPurchaseCreate {
    pub profile_id: Uuid,
    pub listing_id: Uuid,
    pub content: String,
    pub start_date: DateTime<Utc>,
    pub end_date: DateTime<Utc>,
    pub status: AdStatus,
    pub price: f32,
}

#[derive(Debug, Deserialize, Clone, PartialEq)]
pub struct AdPurchaseUpdate {
    pub profile_id: Uuid,
    pub listing_id: Uuid,
    pub content: String,
    pub start_date: DateTime<Utc>,
    pub end_date: DateTime<Utc>,
    pub price: f32,
    pub status: AdStatus,
}

#[derive(Debug, Clone, PartialEq, Eq, DeriveActiveEnum, Serialize, Deserialize,Display, EnumIter)]
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
