use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UserInfo {
    pub id: String,
    pub email: String,
    pub first_name: String,
    pub last_name: String,
    pub is_admin: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionResponse {
    pub token: String,
    pub refresh_token: String,
    pub user: Option<UserInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserLogin {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DirectoryTypeModel {
    pub id: String,
    pub name: String,
    pub description: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CategoryModel {
    pub id: String,
    pub directory_type_id: String,
    pub parent_category_id: Option<String>,
    pub name: String,
    pub description: String,
    pub icon: Option<String>,
    pub slug: Option<String>,
    pub is_custom: bool,
    pub is_active: bool,
    pub created_at: String,
    pub updated_at: String,
    pub directory_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateModel {
    pub id: String,
    pub directory_id: String,
    pub category_id: String,
    pub name: String,
    pub description: String,
    pub template_type: String,
    pub is_active: bool,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DirectoryModel {
    pub id: String,
    pub name: String,
    pub directory_type_id: String,
    pub domain: String,
    pub description: String,
    pub created_at: String,
    pub updated_at: String,
    pub enabled_modules: u32,
    pub theme: Option<String>,
    pub site_status: String,
    pub subdomain: Option<String>,
    pub custom_domain: Option<String>,
    // omit custom_settings and SEO fields for now to simplify MVP
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateDirectory {
    pub name: String,
    pub directory_type_id: String,
    pub domain: String,
    pub description: String,
    pub deployment_strategy: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountModel {
    pub id: String,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateAccount {
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LeadModel {
    pub id: String,
    pub name: String,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub email: Option<String>,
    pub status: Option<String>, // Status is often customized in lead models or mapped to is_converted natively
    pub is_converted: bool,
}

// ==== LISTINGS ====

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum ListingStatus {
    #[serde(rename = "pending")]
    Pending,
    #[serde(rename = "approved")]
    Approved,
    #[serde(rename = "rejected")]
    Rejected,
    #[serde(rename = "active")]
    Active,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ListingModel {
    pub id: String,
    pub profile_id: String,
    pub directory_id: String,
    pub category_id: Option<String>,
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
    pub additional_info: serde_json::Value,
    pub status: ListingStatus,
    pub is_featured: bool,
    pub is_based_on_template: bool,
    pub based_on_template_id: Option<String>,
    pub is_ad_placement: bool,
    pub is_active: bool,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ListingCreate {
    pub title: String,
    pub description: String,
    pub directory_id: String,
    pub profile_id: String,
    pub category_id: Option<String>,
    pub listing_type: Option<String>,
    pub price: Option<f64>,
    pub price_type: Option<String>,
    pub country: Option<String>,
    pub state: Option<String>,
    pub city: Option<String>,
    pub neighborhood: Option<String>,
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,
    pub additional_info: Option<serde_json::Value>,
    pub is_featured: Option<bool>,
    pub is_based_on_template: Option<bool>,
    pub based_on_template_id: Option<String>,
    pub is_ad_placement: Option<bool>,
    pub is_active: Option<bool>,
    pub slug: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ListingUpdate {
    pub title: Option<String>,
    pub description: Option<String>,
    pub directory_id: Option<String>,
    pub profile_id: Option<String>,
    pub category_id: Option<String>,
    pub listing_type: Option<String>,
    pub price: Option<f64>,
    pub price_type: Option<String>,
    pub country: Option<String>,
    pub state: Option<String>,
    pub city: Option<String>,
    pub neighborhood: Option<String>,
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,
    pub additional_info: Option<serde_json::Value>,
    pub status: Option<ListingStatus>,
    pub is_featured: Option<bool>,
    pub is_based_on_template: Option<bool>,
    pub based_on_template_id: Option<String>,
    pub is_ad_placement: Option<bool>,
    pub is_active: Option<bool>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ListingAttributeModel {
    pub id: String,
    pub listing_id: Option<String>,
    pub template_id: Option<String>,
    pub attribute_type: String,
    pub attribute_key: String,
    pub value: serde_json::Value,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ListingWithAttributes {
    pub listing: ListingModel,
    pub attributes: Vec<ListingAttributeModel>,
}

// ==== FILES ====

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FileModel {
    pub id: String,
    pub name: String,
    pub size: i64,
    pub mime_type: String,
    pub hash_sha256: String,
    pub storage_type: String,
    pub storage_path: String,
    pub views: i32,
    pub downloads: i32,
    pub bandwidth_used: i64,
    pub bandwidth_used_paid: i64,
    pub date_upload: String,
    pub date_last_view: Option<String>,
    pub is_anonymous: bool,
    pub user_id: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CreateFileInput {
    pub name: String,
    pub size: i64,
    pub mime_type: String,
    pub hash_sha256: String,
    pub storage_type: String,
    pub storage_path: String,
    pub is_anonymous: bool,
    pub user_id: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UpdateFileInput {
    pub name: Option<String>,
    pub views: Option<i32>,
    pub downloads: Option<i32>,
    pub bandwidth_used: Option<i64>,
    pub bandwidth_used_paid: Option<i64>,
    pub date_last_view: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateLead {
    pub name: String,
    pub email: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DealModel {
    pub id: String,
    pub customer_id: String,
    pub name: String,
    pub amount: f32,
    pub status: String,
    pub stage: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateDeal {
    pub customer_id: String,
    pub name: String,
    pub amount: f32,
    pub status: String,
    pub stage: String,
}
