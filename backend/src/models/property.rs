use serde::{Deserialize, Serialize};
use sea_orm::{EnumIter, DeriveActiveEnum};
use strum_macros::{EnumString, Display};
use sea_orm::sea_query::StringLen;

#[derive(Debug, Clone, PartialEq, EnumIter, DeriveActiveEnum, Serialize, Deserialize, EnumString, Display)]
#[sea_orm(rs_type = "String", db_type = "String(StringLen::N(32))")]
pub enum PropertyType {
    #[sea_orm(string_value = "ServiceDetail")]
    ServiceDetail,
    #[sea_orm(string_value = "ProductDetail")]
    ProductDetail,
    #[sea_orm(string_value = "EventDetail")]
    EventDetail,
    #[sea_orm(string_value = "Location")]
    Location,
    #[sea_orm(string_value = "BusinessHours")]
    BusinessHours,
    #[sea_orm(string_value = "Custom")]
    Custom,
    #[sea_orm(string_value = "Fees")]
    Fees,
    #[sea_orm(string_value = "Payment")]
    Payment,
    #[sea_orm(string_value = "Media")]
    Media,
    #[sea_orm(string_value = "Amenity")]
    Amenity,
    #[sea_orm(string_value = "Tag")]
    Tag,
}

#[derive(Debug, Clone, PartialEq, EnumIter, DeriveActiveEnum, Serialize, Deserialize, EnumString, Display)]
#[sea_orm(rs_type = "String", db_type = "String(StringLen::N(50))")]
pub enum PropertyKey {
    // Service-related keys
    #[sea_orm(string_value = "Specialization")]
    Specialization,
    #[sea_orm(string_value = "Experience")]
    Experience,
    #[sea_orm(string_value = "Certification")]
    Certification,

    // Product-related keys
    #[sea_orm(string_value = "Brand")]
    Brand,
    #[sea_orm(string_value = "Condition")]
    Condition,
    #[sea_orm(string_value = "Warranty")]
    Warranty,

    // Event-related keys
    #[sea_orm(string_value = "EventDate")]
    EventDate,
    #[sea_orm(string_value = "Venue")]
    Venue,
    #[sea_orm(string_value = "Capacity")]
    Capacity,

    // Location-related keys
    #[sea_orm(string_value = "Address")]
    Address,
    #[sea_orm(string_value = "City")]
    City,
    #[sea_orm(string_value = "State")]
    State,
    #[sea_orm(string_value = "Country")]
    Country,
    #[sea_orm(string_value = "PostalCode")]
    PostalCode,

    // Availability-related keys
    #[sea_orm(string_value = "DaysAvailable")]
    DaysAvailable,
    #[sea_orm(string_value = "HoursAvailable")]
    HoursAvailable,

    // Custom key for flexibility
    #[sea_orm(string_value = "CustomKey")]
    CustomKey,
}
