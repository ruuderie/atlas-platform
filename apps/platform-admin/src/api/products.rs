use crate::api::client::{api_get, api_url, create_client, api_request};
use crate::api::models::{
    PlatformProductModel, ProductVariantModel, ProductTemplateModel,
    DeployStatusResponse, WaitlistAnalyticsResponse, UpdateProductBody,
    BulkGenerateBody
};
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

pub async fn update_product_detail(id: Uuid, body: UpdateProductBody) -> Result<PlatformProductModel, String> {
    let client = create_client();
    let url = api_url(&format!("api/admin/platform/products/{}", id));
    let req = client.patch(&url).json(&body);
    api_request(req).await
}

pub async fn publish_marketing(id: Uuid) -> Result<serde_json::Value, String> {
    let client = create_client();
    let url = api_url(&format!("api/admin/platform/products/{}/publish-marketing", id));
    let req = client.post(&url);
    api_request(req).await
}

pub async fn get_deploy_status(id: Uuid) -> Result<DeployStatusResponse, String> {
    api_get(&format!("api/admin/platform/products/{}/deploy-status", id)).await
}

pub async fn get_template(id: Uuid) -> Result<ProductTemplateModel, String> {
    api_get(&format!("api/admin/platform/products/{}/template", id)).await
}

pub async fn get_variants(id: Uuid) -> Result<Vec<ProductVariantModel>, String> {
    api_get(&format!("api/admin/platform/products/{}/variants", id)).await
}

pub async fn bulk_generate_variants(id: Uuid, body: BulkGenerateBody) -> Result<serde_json::Value, String> {
    let client = create_client();
    let url = api_url(&format!("api/admin/platform/products/{}/variants/bulk-generate", id));
    let req = client.post(&url).json(&body);
    api_request(req).await
}

pub async fn localize_variant(product_id: Uuid, variant_id: Uuid) -> Result<serde_json::Value, String> {
    let client = create_client();
    let url = api_url(&format!("api/admin/platform/products/{}/variants/{}/localize", product_id, variant_id));
    let req = client.post(&url);
    api_request(req).await
}

pub async fn bulk_localize(product_id: Uuid) -> Result<serde_json::Value, String> {
    let client = create_client();
    let url = api_url(&format!("api/admin/platform/products/{}/variants/bulk-localize", product_id));
    let req = client.post(&url);
    api_request(req).await
}

pub async fn get_waitlist(id: Uuid) -> Result<WaitlistAnalyticsResponse, String> {
    api_get(&format!("api/admin/platform/products/{}/waitlist", id)).await
}
