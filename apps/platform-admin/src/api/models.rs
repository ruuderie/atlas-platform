use serde::{Deserialize, Serialize};
use uuid::Uuid;

// ── GTM discriminant enums ────────────────────────────────────────────────────
//
// Mirror of backend::types::gtm — kept in sync by convention since there is
// no shared crate yet. Each enum uses #[serde(rename_all = "snake_case")] so
// JSON from the backend round-trips transparently.
//
// Rule: if the backend adds a variant here, add it here too. The compiler will
// then flag every non-exhaustive match site in the UI.

/// Controls what a visitor sees when landing on a product page.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LaunchMode {
    Active,
    Waitlist,
    PreOrder,
    PreLaunch,
    Draft,
}

impl LaunchMode {
    /// Human-friendly label shown in the platform-admin UI.
    pub fn label(&self) -> &'static str {
        match self {
            Self::Active    => "Active",
            Self::Waitlist  => "Waitlist",
            Self::PreOrder  => "Pre-Order",
            Self::PreLaunch => "Pre-Launch",
            Self::Draft     => "Draft",
        }
    }

    /// Badge CSS class for the variants table.
    pub fn badge_class(&self) -> &'static str {
        match self {
            Self::Active    => "inline-flex items-center px-2 py-0.5 rounded text-[9px] font-bold bg-emerald-500/10 text-emerald-400 border border-emerald-500/20",
            Self::Waitlist  => "inline-flex items-center px-2 py-0.5 rounded text-[9px] font-bold bg-amber-500/10 text-amber-400 border border-amber-500/20",
            Self::PreOrder  => "inline-flex items-center px-2 py-0.5 rounded text-[9px] font-bold bg-purple-500/10 text-purple-400 border border-purple-500/20",
            Self::PreLaunch => "inline-flex items-center px-2 py-0.5 rounded text-[9px] font-bold bg-sky-500/10 text-sky-400 border border-sky-500/20",
            Self::Draft     => "inline-flex items-center px-2 py-0.5 rounded text-[9px] font-bold bg-outline-variant/20 text-on-surface-variant border border-outline-variant/20",
        }
    }
}

impl std::fmt::Display for LaunchMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.label())
    }
}

/// How a variant's copy was authored / last modified.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LocalizationStatus {
    Base,
    AiLocalized,
    Manual,
    Pending,
}

impl LocalizationStatus {
    /// Short badge label shown in the variants table.
    pub fn badge_label(&self) -> Option<&'static str> {
        match self {
            Self::AiLocalized => Some("AI"),
            Self::Manual      => Some("Manual"),
            Self::Pending     => Some("Pending"),
            Self::Base        => None,
        }
    }

    /// Badge CSS class — returns `None` for `Base` (no badge rendered).
    pub fn badge_class(&self) -> Option<&'static str> {
        match self {
            Self::AiLocalized => Some("bg-violet-500/10 text-violet-400 border-violet-500/20"),
            Self::Manual      => Some("bg-sky-500/10 text-sky-400 border-sky-500/20"),
            Self::Pending     => Some("bg-amber-500/10 text-amber-400 border-amber-500/20"),
            Self::Base        => None,
        }
    }
}

/// The strategy used when generating a variant's copy.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CopyStrategy {
    Localized,
    BaseCopy,
    AiGenerated,
}

impl CopyStrategy {
    /// Human-friendly label shown in the variants table.
    pub fn label(&self) -> &'static str {
        match self {
            Self::Localized    => "Source Content",
            Self::BaseCopy     => "Base Copy",
            Self::AiGenerated  => "AI ✦ Auto-Trans",
        }
    }
}

impl std::fmt::Display for CopyStrategy {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.label())
    }
}

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

/// Lightweight account record for the platform admin CRM account search picker.
/// Returned by `GET /api/admin/accounts`. Fields match `account::Model` serialization.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AccountSummary {
    pub id:         String,
    pub name:       String,
    pub is_active:  bool,
    pub tenant_id:  String,
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
    pub phone: Option<String>,
    pub whatsapp: Option<String>,
    pub telegram: Option<String>,
    pub company: Option<String>,
    pub title: Option<String>,
    pub source: Option<String>,
    /// Backend sends `lead_status`; maps to pipeline stage (New, Contacted, Qualified, etc.)
    pub lead_status: Option<String>,
    pub is_converted: bool,
    pub avatar_url: Option<String>,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
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

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
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
    /// Typed — compiler enforces exhaustive match in UI components.
    pub launch_mode: LaunchMode,
    pub is_published: bool,
    pub cta_label: Option<String>,
    pub cta_action: Option<String>,
    pub pre_order_cap: Option<i32>,
    pub pre_order_sold: i32,
    pub lead_count: i32,
    pub view_count: i32,
    /// Typed — compiler enforces exhaustive match in UI components.
    pub copy_strategy: CopyStrategy,
    /// Typed — compiler enforces exhaustive match in UI components.
    pub localization_status: LocalizationStatus,
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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct VerificationRequestModel {
    pub id: uuid::Uuid,
    pub tenant_id: uuid::Uuid,
    pub entity_name: String,
    pub req_type: String,
    pub status: String,
    pub created_at: String,
    pub document_count: u32,
    pub rejection_reason: Option<String>,
}


// ==== TENANT / PLATFORM ADMIN REGISTRY ====

/// Returned by `GET /api/admin/tenant-stats` (one row per tenant).
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TenantStatModel {
    pub tenant_id: String,
    pub name: String,
    /// Human-readable unique slug (same as `tenant.name` in the DB).
    /// Shown in the Feature Flags tenant targeting dropdown.
    pub slug: String,
    pub profile_count: u64,
    pub listing_count: u64,
    pub ad_purchase_count: u64,
    // Extended fields (populated after Tier 3 backend work)
    pub plan: Option<String>,
    pub mrr_cents: Option<i64>,
    pub site_status: Option<String>,
    pub joined_at: Option<String>,
    /// UUID of the tenant's primary (anchor) app instance.
    pub anchor_instance_id: Option<String>,
}

/// Returned by `GET /api/admin/platform/apps` (one row per app instance).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlatformAppSummary {
    pub tenant_id:           String,
    pub instance_id:         String,
    pub name:                String,
    pub app_type:            String,
    pub domain:              String,
    /// "standard" | "internal_operator". Defaults to "standard" when no deployment config exists.
    pub mode:                String,
    /// From deployment config instance_status, falls back to tenant.site_status.
    pub site_status:         String,
    pub description:         String,
    /// Set when the operator links this deployment to a CRM Account. Enables "View Account" action.
    pub platform_account_id: Option<String>,
    /// For InternalOperator instances: "demo" | "test" | "staging" | "managed_service".
    pub purpose:             Option<String>,
}

// ==== BILLING ====

/// Matches `backend/src/entities/transaction::Model`
/// Returned by `GET /api/admin/billing/transactions`
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TransactionModel {
    pub id: String,
    pub tenant_id: String,
    pub provider: String,
    pub amount: i64,
    pub currency: String,
    pub provider_tx_id: Option<String>,
    pub status: String,
    pub created_at: Option<String>,
}

/// Matches `backend/src/entities/billing_plan::Model`
/// Returned by `GET /api/admin/billing/plans`
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct BillingPlanModel {
    pub id: String,
    pub name: String,
    pub price: i64,
    pub currency: String,
    pub interval: String,
    pub created_at: Option<String>,
}

/// Matches `backend/src/entities/atlas_subscription::Model`
/// Returned by `GET /api/admin/billing/subscriptions` (future) or derived from tenant-stats
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AtlasSubscriptionModel {
    pub id: String,
    pub tenant_id: String,
    pub billing_interval: String,
    pub price_cents: i64,
    pub currency: String,
    pub status: String,
    pub stripe_subscription_id: Option<String>,
    pub trial_ends_at: Option<String>,
    pub current_period_end: Option<String>,
    pub is_billing_exempt: bool,
}

// ==== SUPPORT CASES ====

/// Matches `backend/src/models/note::NoteModel`
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct NoteModel {
    pub id: String,
    pub content: String,
    pub created_at: Option<String>,
}

/// Matches `backend/src/models/activity::ActivityModel`
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ActivityModel {
    pub id: String,
    pub activity_type: Option<String>,
    pub description: Option<String>,
    pub created_at: Option<String>,
}

/// Matches `backend/src/models/case::CaseModel`
/// Returned by `GET /api/admin/cases` and `GET /api/admin/cases/{id}`
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CaseModel {
    pub id: String,
    pub customer_id: String,
    pub title: String,
    pub description: String,
    pub status: String,
    pub priority: String,
    pub assigned_to: Option<String>,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
    pub closed_at: Option<String>,
    pub notes: Vec<NoteModel>,
    pub activities: Vec<ActivityModel>,
}

// ==== FEATURE FLAGS ====

/// Matches `backend/src/admin/feature_flags::FlagOverrideModel`
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct FlagOverrideModel {
    pub id: String,
    pub flag_id: String,
    pub tenant_id: String,
    pub override_type: String,
    pub rollout_pct: i32,
    pub reason: String,
    pub jira: Option<String>,
    pub changed_by: String,
    pub created_at: Option<String>,
}

/// Matches `backend/src/admin/feature_flags::FlagAuditLogModel`
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct FlagAuditLogModel {
    pub id: String,
    pub flag_id: String,
    pub user_id: String,
    pub action: String,
    pub created_at: Option<String>,
}

/// Matches `backend/src/admin/feature_flags::FeatureFlagModel`
/// Returned by `GET /api/admin/flags`
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct FeatureFlagModel {
    pub id: String,
    pub key: String,
    pub description: String,
    pub is_enabled: bool,
    pub has_global: bool,
    pub global_rollout_pct: i32,
    pub is_plan_gated: bool,
    pub plan_gate_tier: Option<String>,
    pub jira: Option<String>,
    pub owner: String,
    pub created_at: Option<String>,
    pub overrides: Vec<FlagOverrideModel>,
    pub audit_logs: Vec<FlagAuditLogModel>,
}

/// Input for `POST /api/admin/flags`
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CreateFlagInput {
    pub key: String,
    pub description: String,
    pub has_global: Option<bool>,
    pub global_rollout_pct: Option<i32>,
    pub jira: Option<String>,
}

/// Input for `PUT /api/admin/flags/{key}`
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct UpdateFlagInput {
    pub is_enabled: Option<bool>,
    pub global_rollout_pct: Option<i32>,
    pub description: Option<String>,
}

/// Input for `POST /api/admin/flags/{key}/overrides`
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CreateFlagOverrideInput {
    pub tenant_id: String,
    pub override_type: String,
    pub rollout_pct: Option<i32>,
    pub reason: String,
    pub jira: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct AbVariantModel {
    pub id: Uuid,
    pub test_id: Uuid,
    pub name: String,
    pub is_control: bool,
    pub views: i32,
    pub conversions: i32,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct AdminAbTestWithVariantsModel {
    pub id: Uuid,
    pub listing_id: Uuid,
    pub status: String,
    pub traffic_split_strategy: String,
    pub created_at: String,
    pub updated_at: String,
    pub variants: Vec<AbVariantModel>,
}

// ==== PLATFORM SUPPORT INBOX ====

/// Summary of one platform_support thread (from atlas_ws_room).
/// Returned by `GET /api/admin/support/threads`.
#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct SupportThreadSummary {
    pub id:              String,
    pub tenant_id:       String,
    pub entity_id:       String,   // submitting user's ID
    pub is_active:       bool,
    pub created_at:      String,
    pub last_message:    Option<String>,
    pub last_at:         Option<String>,
    pub message_count:   u64,
    pub submitter_name:  Option<String>,
    pub submitter_email: Option<String>,
}

/// A single message in a support thread.
#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct SupportMessageRow {
    pub id:             String,
    pub sender_user_id: Option<String>,
    pub sender_name:    Option<String>,
    pub message_type:   String,   // "text" | "system" | "operator_reply"
    pub content:        String,
    pub created_at:     String,
    pub is_operator:    bool,
}

/// Full detail for one support thread, including message history.
/// Returned by `GET /api/admin/support/threads/{id}`.
#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct SupportThreadDetail {
    // Flattened fields from ThreadSummary
    pub id:              String,
    pub tenant_id:       String,
    pub entity_id:       String,
    pub is_active:       bool,
    pub created_at:      String,
    pub last_message:    Option<String>,
    pub last_at:         Option<String>,
    pub message_count:   u64,
    pub submitter_name:  Option<String>,
    pub submitter_email: Option<String>,
    // Thread messages
    pub messages:        Vec<SupportMessageRow>,
}
