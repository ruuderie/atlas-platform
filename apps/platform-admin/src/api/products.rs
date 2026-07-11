use crate::api::client::{api_get, api_request, api_url, create_client};
use crate::api::models::{
    BulkGenerateBody, DeployStatusResponse, PlatformProductModel, ProductTemplateModel,
    ProductVariantModel, UpdateProductBody, WaitlistAnalyticsResponse,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub async fn get_products() -> Result<Vec<PlatformProductModel>, String> {
    api_get("api/admin/platform/products").await
}

#[derive(serde::Serialize)]
struct CreateProductInput {
    name: String,
    slug: String,
}

pub async fn create_product(name: String, slug: String) -> Result<PlatformProductModel, String> {
    let client = create_client();
    let url = api_url("api/admin/platform/products");
    let req = client.post(&url).json(&CreateProductInput { name, slug });
    api_request(req).await
}

pub async fn get_product_detail(id: Uuid) -> Result<PlatformProductModel, String> {
    api_get(&format!("api/admin/platform/products/{}", id)).await
}

pub async fn update_product_detail(
    id: Uuid,
    body: UpdateProductBody,
) -> Result<PlatformProductModel, String> {
    let client = create_client();
    let url = api_url(&format!("api/admin/platform/products/{}", id));
    let req = client.patch(&url).json(&body);
    api_request(req).await
}

pub async fn publish_marketing(id: Uuid) -> Result<serde_json::Value, String> {
    let client = create_client();
    let url = api_url(&format!(
        "api/admin/platform/products/{}/publish-marketing",
        id
    ));
    let req = client.post(&url);
    api_request(req).await
}

pub async fn get_deploy_status(id: Uuid) -> Result<DeployStatusResponse, String> {
    api_get(&format!("api/admin/platform/products/{}/deploy-status", id)).await
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProductPlanBillingInterval {
    Month,
    Year,
    Forever,
    Custom,
}

impl ProductPlanBillingInterval {
    pub fn label(&self) -> &'static str {
        match self {
            Self::Month => "Monthly",
            Self::Year => "Annually",
            Self::Forever => "Forever",
            Self::Custom => "Custom",
        }
    }

    pub fn short_label(&self) -> &'static str {
        match self {
            Self::Month => "mo",
            Self::Year => "yr",
            Self::Forever => "forever",
            Self::Custom => "custom",
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ProductPlanModel {
    pub id: Uuid,
    pub product_id: Uuid,
    pub slug: String,
    pub name: String,
    pub tagline: String,
    pub price_cents: i32,
    pub currency: String,
    pub billing_interval: ProductPlanBillingInterval,
    pub features: Vec<String>,
    pub cta_label: String,
    pub cta_href: Option<String>,
    pub is_featured: bool,
    pub sort_order: i32,
    pub is_active: bool,
    pub billing_plan_id: Option<Uuid>,
}

#[derive(Clone, Debug, Serialize)]
pub struct ProductPlanInput {
    pub slug: String,
    pub name: String,
    pub tagline: Option<String>,
    pub price_cents: Option<i32>,
    pub currency: Option<String>,
    pub billing_interval: Option<ProductPlanBillingInterval>,
    pub features: Vec<String>,
    pub cta_label: Option<String>,
    pub cta_href: Option<String>,
    pub is_featured: Option<bool>,
    pub sort_order: Option<i32>,
    pub is_active: Option<bool>,
    pub billing_plan_id: Option<Uuid>,
}

pub async fn list_product_plans(id: Uuid) -> Result<Vec<ProductPlanModel>, String> {
    api_get(&format!("api/admin/platform/products/{}/plans", id)).await
}

pub async fn create_product_plan(
    product_id: Uuid,
    input: ProductPlanInput,
) -> Result<ProductPlanModel, String> {
    let client = create_client();
    let url = api_url(&format!("api/admin/platform/products/{}/plans", product_id));
    let req = client.post(&url).json(&input);
    api_request(req).await
}

pub async fn update_product_plan(
    product_id: Uuid,
    plan_id: Uuid,
    input: ProductPlanInput,
) -> Result<ProductPlanModel, String> {
    let client = create_client();
    let url = api_url(&format!(
        "api/admin/platform/products/{}/plans/{}",
        product_id, plan_id
    ));
    let req = client.patch(&url).json(&input);
    api_request(req).await
}

pub async fn delete_product_plan(
    product_id: Uuid,
    plan_id: Uuid,
) -> Result<serde_json::Value, String> {
    let client = create_client();
    let url = api_url(&format!(
        "api/admin/platform/products/{}/plans/{}",
        product_id, plan_id
    ));
    let req = client.delete(&url);
    api_request(req).await
}

pub async fn get_template(id: Uuid) -> Result<ProductTemplateModel, String> {
    api_get(&format!("api/admin/platform/products/{}/template", id)).await
}

#[derive(Clone, Debug, Default, Serialize)]
pub struct UpsertProductTemplateBody {
    pub hero_payload: Option<serde_json::Value>,
    pub blocks_payload: Option<serde_json::Value>,
    pub meta_title: Option<String>,
    pub meta_description: Option<String>,
    pub og_image_url: Option<String>,
    pub structured_data: Option<serde_json::Value>,
    pub cta_label: Option<String>,
    pub cta_action: Option<String>,
}

pub async fn upsert_template(
    id: Uuid,
    body: UpsertProductTemplateBody,
) -> Result<ProductTemplateModel, String> {
    let client = create_client();
    let url = api_url(&format!("api/admin/platform/products/{}/template", id));
    let req = client.post(&url).json(&body);
    api_request(req).await
}

pub async fn get_variants(id: Uuid) -> Result<Vec<ProductVariantModel>, String> {
    api_get(&format!("api/admin/platform/products/{}/variants", id)).await
}

pub async fn bulk_generate_variants(
    id: Uuid,
    body: BulkGenerateBody,
) -> Result<serde_json::Value, String> {
    let client = create_client();
    let url = api_url(&format!(
        "api/admin/platform/products/{}/variants/bulk-generate",
        id
    ));
    let req = client.post(&url).json(&body);
    api_request(req).await
}

pub async fn localize_variant(
    product_id: Uuid,
    variant_id: Uuid,
) -> Result<serde_json::Value, String> {
    let client = create_client();
    let url = api_url(&format!(
        "api/admin/platform/products/{}/variants/{}/localize",
        product_id, variant_id
    ));
    let req = client.post(&url);
    api_request(req).await
}

pub async fn bulk_localize(product_id: Uuid) -> Result<serde_json::Value, String> {
    let client = create_client();
    let url = api_url(&format!(
        "api/admin/platform/products/{}/variants/bulk-localize",
        product_id
    ));
    let req = client.post(&url);
    api_request(req).await
}

pub async fn get_waitlist(id: Uuid) -> Result<WaitlistAnalyticsResponse, String> {
    api_get(&format!("api/admin/platform/products/{}/waitlist", id)).await
}
