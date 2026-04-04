use chrono::{Utc, DateTime, Duration};
use uuid::Uuid;
use serde::{Serialize, Deserialize};
use sea_orm::prelude::*;
use serde_json::Value;
use sea_orm::{IntoActiveModel, Set, ActiveModelTrait};
use crate::entities::listing;
use std::str::FromStr;

#[derive(Debug, Deserialize)]
pub struct ListingSearch {
    pub q: String,
    pub page: Option<u64>,
    pub limit: Option<u64>,
    pub category: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PaginatedListings<T> {
    pub items: Vec<T>,
    pub total: u64,
    pub page: u64,
    pub limit: u64,
    pub total_pages: u64,
}
#[derive(Debug, Deserialize, Serialize)]
pub struct ListingModel {
    pub id: Uuid,
    pub profile_id: Uuid,
    pub tenant_id: Uuid,
    pub category_id: Option<Uuid>,
    pub title: String,
    pub description: String,
    pub listing_type: String,
    pub price: Option<f64>,
    pub price_type: Option<String>,
    pub country: Option<String>,
    pub state: Option<String>,
    pub city: Option<String>,
    pub neighborhood: Option<String>,
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,
    pub additional_info: Value,
    pub properties: Option<Value>,
    pub status: ListingStatus,
    pub is_featured: bool,
    pub is_based_on_template: bool,
    pub based_on_template_id: Option<Uuid>,
    pub is_ad_placement: bool,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq, EnumIter, DeriveActiveEnum, Serialize, Deserialize)]
#[sea_orm(rs_type = "String", db_type = "String(StringLen::N(32))")]
pub enum ListingStatus {
    #[sea_orm(string_value = "pending")]
    Pending,
    #[sea_orm(string_value = "approved")]
    Approved,
    #[sea_orm(string_value = "rejected")]
    Rejected,
    #[sea_orm(string_value = "active")]
    Active,
}

impl FromStr for ListingStatus {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "pending" => Ok(ListingStatus::Pending),
            "approved" => Ok(ListingStatus::Approved),
            "rejected" => Ok(ListingStatus::Rejected),
            _ => Err(()),
        }
    }
}
#[derive(Debug, Deserialize, Serialize)]
pub struct ListingCreate {
    // Required fields
    pub title: String,
    pub description: String,
    pub tenant_id: Uuid,
    pub profile_id: Uuid,

    // Optional fields
    pub category_id: Option<Uuid>,
    pub listing_type: Option<String>,
    pub price: Option<f64>,
    pub price_type: Option<String>,
    pub country: Option<String>,
    pub state: Option<String>,
    pub city: Option<String>,
    pub neighborhood: Option<String>,
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,
    pub additional_info: Option<Value>,
    pub properties: Option<Value>,
    pub is_featured: Option<bool>,
    pub is_based_on_template: Option<bool>,
    pub based_on_template_id: Option<Uuid>,
    pub is_ad_placement: Option<bool>,
    pub is_active: Option<bool>,
    pub slug: Option<String>,
}

impl IntoActiveModel<listing::ActiveModel> for ListingCreate {
    fn into_active_model(self) -> listing::ActiveModel {
        listing::ActiveModel {
            id: Set(Uuid::new_v4()),
            profile_id: Set(self.profile_id),
            tenant_id: Set(self.tenant_id),
            category_id: Set(self.category_id),
            title: Set(self.title),
            description: Set(self.description),
            listing_type: Set(self.listing_type.unwrap_or("standard".to_string())),
            price: Set(self.price),
            price_type: Set(self.price_type),
            country: Set(Some(self.country.unwrap_or("Unknown".to_string()))),
            state: Set(Some(self.state.unwrap_or("Unknown".to_string()))),
            city: Set(Some(self.city.unwrap_or("Unknown".to_string()))),
            neighborhood: Set(self.neighborhood),
            latitude: Set(self.latitude),
            longitude: Set(self.longitude),
            additional_info: Set(Some(self.additional_info.unwrap_or(Value::Object(Default::default())))),
            properties: Set(self.properties),
            status: Set(ListingStatus::Pending),
            is_featured: Set(self.is_featured.unwrap_or(false)),
            is_based_on_template: Set(self.is_based_on_template.unwrap_or(false)),
            based_on_template_id: Set(self.based_on_template_id),
            is_ad_placement: Set(self.is_ad_placement.unwrap_or(false)),
            is_active: Set(self.is_active.unwrap_or(true)),
            created_at: Set(Utc::now()),
            updated_at: Set(Utc::now()),
            slug: Set(self.slug),
        }
    }
}
#[derive(Debug, Deserialize)]
pub struct ListingUpdate {
    pub title: Option<String>,
    pub description: Option<String>,
    pub tenant_id: Option<Uuid>,
    pub profile_id: Option<Uuid>,
    pub category_id: Option<Uuid>,
    pub listing_type: Option<String>,
    pub price: Option<f64>,
    pub price_type: Option<String>,
    pub country: Option<String>,
    pub state: Option<String>,
    pub city: Option<String>,
    pub neighborhood: Option<String>,
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,
    pub additional_info: Option<Value>,
    pub properties: Option<Value>,
    #[serde(deserialize_with = "deserialize_listing_status_option")]
    pub status: Option<ListingStatus>,
    pub is_featured: Option<bool>,
    pub is_based_on_template: Option<bool>,
    pub based_on_template_id: Option<Uuid>,
    pub is_ad_placement: Option<bool>,
    pub is_active: Option<bool>,
}

fn deserialize_listing_status<'de, D>(deserializer: D) -> Result<ListingStatus, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let s: String = String::deserialize(deserializer)?;
    match s.as_str() {
        "pending" => Ok(ListingStatus::Pending),
        "approved" => Ok(ListingStatus::Approved),
        "rejected" => Ok(ListingStatus::Rejected),
        _ => Err(serde::de::Error::custom("Invalid listing status")),
    }
}

// Add a deserializer for Option<ListingStatus>
fn deserialize_listing_status_option<'de, D>(deserializer: D) -> Result<Option<ListingStatus>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let opt = Option::<String>::deserialize(deserializer)?;
    match opt {
        Some(s) => match s.as_str() {
            "pending" => Ok(Some(ListingStatus::Pending)),
            "approved" => Ok(Some(ListingStatus::Approved)),
            "rejected" => Ok(Some(ListingStatus::Rejected)),
            "active" => Ok(Some(ListingStatus::Approved)), // Map "active" to "approved"
            _ => Err(serde::de::Error::custom("Invalid listing status")),
        },
        None => Ok(None),
    }
}

