use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// The backend-side `TenantModel` returned by `POST /api/tenants`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TenantCreatedModel {
    pub id: Uuid,
    pub name: String,
    pub description: String,
}

/// Payload for `POST /api/app-instances`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateAppInstance {
    pub tenant_id: Uuid,
    pub app_type: String,
    pub database_url: Option<String>,
    pub data_seed_name: Option<String>,
    pub settings: Option<serde_json::Value>,
}

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
    pub token: Option<String>,
    pub refresh_token: Option<String>,
    pub user: Option<UserInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserLogin {
    pub email: String,
    pub password: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct CaseLogModel {
    pub id: Uuid,
    pub case_id: Uuid,
    pub body: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SearchResult {
    pub id: Uuid,
    pub entity_type: String,
    pub entity_id: Uuid,
    pub tenant_id: Option<Uuid>,
    pub metadata: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkTypeModel {
    pub id: String,
    pub name: String,
    pub description: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CategoryModel {
    pub id: String,
    pub network_type_id: String,
    pub parent_category_id: Option<String>,
    pub name: String,
    pub description: String,
    pub icon: Option<String>,
    pub slug: Option<String>,
    pub is_custom: bool,
    pub is_active: bool,
    pub created_at: String,
    pub updated_at: String,
    pub network_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateModel {
    pub id: String,
    pub network_id: String,
    pub category_id: String,
    pub name: String,
    pub description: String,
    pub template_type: String,
    pub is_active: bool,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlatformAppModel {
    pub tenant_id: String,
    pub instance_id: String,
    pub name: String,
    pub app_type: String,
    pub domain: String,
    pub site_status: String,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateNetwork {
    pub name: String,
    pub network_type_id: String,
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
    pub network_id: String,
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
    pub network_id: String,
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
    pub network_id: Option<String>,
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpsertSettingRequest {
    pub key: String,
    pub value: String,
    pub is_encrypted: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TenantSettingResponse {
    pub id: String,
    pub tenant_id: String,
    pub key: String,
    pub value: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ContactModel {
    pub id: String,
    pub customer_id: Option<String>,
    pub name: String,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub whatsapp: Option<String>,
    pub telegram: Option<String>,
    pub twitter: Option<String>,
    pub instagram: Option<String>,
    pub facebook: Option<String>,
    pub properties: Option<serde_json::Value>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateContact {
    pub name: String,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub whatsapp: Option<String>,
    pub telegram: Option<String>,
    pub twitter: Option<String>,
    pub instagram: Option<String>,
    pub facebook: Option<String>,
    pub properties: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrmNote {
    pub id: String,
    pub content: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrmActivity {
    pub id: String,
    pub activity_type: String,
    pub description: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrmStatusOption {
    pub status_key: String,
    pub label: String,
    pub color: String,
    pub sort_order: i32,
    pub is_system: bool,
}

// ==== PLATFORM PRODUCTS ====

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PlatformProductModel {
    pub id: uuid::Uuid,
    pub name: String,
    pub slug: String,
    pub tagline: Option<String>,
    pub status: String,
    pub marketing_page_cms_id: Option<uuid::Uuid>,
    pub deploy_hook_url: Option<String>,
    pub launch_mode: String,
    pub pre_order_enabled: bool,
    pub pre_order_price_cents: Option<i32>,
    pub pre_order_currency: String,
    pub stripe_price_id: Option<String>,
    pub pre_order_cap: Option<i32>,
    pub pre_order_sold: i32,
    pub waitlist_count: i32,
    pub sentinel_tenant_id: Option<uuid::Uuid>,
    pub apex_domain: Option<String>,
    pub apex_domain_verified: bool,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateProductBody {
    pub name: Option<String>,
    pub tagline: Option<String>,
    pub status: Option<String>,
    pub deploy_hook_url: Option<String>,
    pub marketing_page_cms_id: Option<uuid::Uuid>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DeployStatusResponse {
    pub product_id: uuid::Uuid,
    pub status: String,
    pub deployed_at: Option<String>,
    pub message: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ProductTemplateModel {
    pub id: uuid::Uuid,
    pub product_id: uuid::Uuid,
    pub hero_payload: serde_json::Value,
    pub blocks_payload: serde_json::Value,
    pub meta_title: Option<String>,
    pub meta_description: Option<String>,
    pub og_image_url: Option<String>,
    pub structured_data: serde_json::Value,
    pub cta_label: String,
    pub cta_action: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ProductVariantModel {
    pub id: uuid::Uuid,
    pub product_id: uuid::Uuid,
    pub template_id: uuid::Uuid,
    pub variant_slug: String,
    pub locale: String,
    pub country_code: Option<String>,
    pub region: Option<String>,
    pub city: Option<String>,
    pub geo_lat: Option<f64>,
    pub geo_lng: Option<f64>,
    pub hero_overrides: serde_json::Value,
    pub block_overrides: serde_json::Value,
    pub meta_title: Option<String>,
    pub meta_description: Option<String>,
    pub og_image_url: Option<String>,
    pub canonical_url: Option<String>,
    pub structured_data: Option<serde_json::Value>,
    pub launch_mode: String,
    pub is_published: bool,
    pub cta_label: Option<String>,
    pub cta_action: Option<String>,
    pub pre_order_cap: Option<i32>,
    pub pre_order_sold: i32,
    pub lead_count: i32,
    pub view_count: i32,
    pub copy_strategy: String,
    pub localization_status: String,
    pub localization_task_id: Option<uuid::Uuid>,
    pub subdomain_override: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketSpec {
    pub slug: String,
    pub locale: String,
    pub city: Option<String>,
    pub region: Option<String>,
    pub country_code: Option<String>,
    pub geo_lat: Option<f64>,
    pub geo_lng: Option<f64>,
    pub subdomain_override: Option<String>,
    pub pre_order_cap: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BulkGenerateBody {
    pub markets: Vec<MarketSpec>,
    pub launch_mode: Option<String>,
    pub copy_strategy: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct WaitlistMarketSummary {
    pub variant_id: uuid::Uuid,
    pub variant_slug: String,
    pub city: Option<String>,
    pub country_code: Option<String>,
    pub locale: String,
    pub launch_mode: String,
    pub is_published: bool,
    pub lead_count: i32,
    pub view_count: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct WaitlistAnalyticsResponse {
    pub product_id: uuid::Uuid,
    pub product_name: String,
    pub total_leads: i32,
    pub waitlist_count: i32,
    pub variant_count: usize,
    pub by_market: Vec<WaitlistMarketSummary>,
}

