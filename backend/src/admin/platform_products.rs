//! Admin — Platform Products handler
//!
//! CRUD + marketing-site deploy hooks for Atlas Platform products.
//! All routes require platform super-admin auth (gated in admin routes.rs).
//!
//! # Routes
//!
//! ```ignore
//! GET  /api/admin/platform/products
//!      List all platform products.
//!      -> 200 [PlatformProduct]
//!
//! GET  /api/admin/platform/products/:id
//!      Get a single product.
//!      -> 200 PlatformProduct
//!
//! PATCH /api/admin/platform/products/:id
//!       Update name, tagline, status, deploy_hook_url, marketing_page_cms_id.
//!       -> 200 PlatformProduct
//!
//! POST /api/admin/platform/products/:id/publish-marketing
//!      Trigger Cloudflare Pages deploy hook (or other CI hook) via HTTP POST.
//!      -> 202 { deploy_id, status: "deploying" }
//!
//! GET  /api/admin/platform/products/:id/deploy-status
//!      Returns last known deploy status (polled from Cloudflare API or stored).
//!      -> 200 { status: "success" | "failure" | "deploying" | "unknown", deployed_at }
//! ```

use axum::{
    Router,
    extract::{Extension, Json, Path},
    http::StatusCode,
    response::IntoResponse,
    routing::{delete, get, patch, post},
};
use chrono::Utc;
use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, DatabaseConnection, EntityTrait,
    IntoActiveModel, QueryFilter, QueryOrder,
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use uuid::Uuid;

use crate::entities::{platform_product, platform_product_plan, product_tracking_pixel};
use crate::types::gtm::{InjectAt, PixelType};

// ── Route registration ────────────────────────────────────────────────────────

/// State-free router — merge before outer .with_state(db) in admin_routes().
pub fn routes_raw() -> Router<DatabaseConnection> {
    Router::new()
        .route("/api/admin/platform/products", get(list_products))
        .route(
            "/api/admin/platform/products/{id}",
            get(get_product).patch(update_product),
        )
        .route(
            "/api/admin/platform/products/{id}/publish-marketing",
            post(publish_marketing),
        )
        .route(
            "/api/admin/platform/products/{id}/deploy-status",
            get(get_deploy_status),
        )
        .route(
            "/api/admin/platform/products/{id}/plans",
            get(list_product_plans).post(create_product_plan),
        )
        .route(
            "/api/admin/platform/products/{id}/plans/{plan_id}",
            patch(update_product_plan).delete(delete_product_plan),
        )
        .route(
            "/api/admin/platform/products/{id}/pixels",
            get(list_product_pixels).post(create_product_pixel),
        )
        .route(
            "/api/admin/platform/products/{id}/pixels/{pixel_id}",
            delete(delete_product_pixel),
        )
}

/// Convenience wrapper with state applied (for standalone use / tests).
pub fn routes(db: DatabaseConnection) -> Router {
    routes_raw().with_state(db)
}

// ── Input types ───────────────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct UpdateProductBody {
    pub name: Option<String>,
    pub tagline: Option<String>,
    /// "active" | "beta" | "deprecated"
    pub status: Option<String>,
    pub deploy_hook_url: Option<String>,
    pub marketing_page_cms_id: Option<Uuid>,
}

#[derive(Debug, Serialize)]
pub struct DeployStatusResponse {
    pub product_id: Uuid,
    pub status: String, // "success" | "failure" | "deploying" | "unknown" | "no_hook"
    pub deployed_at: Option<String>,
    pub message: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct ProductPlanResponse {
    pub id: Uuid,
    pub product_id: Uuid,
    pub slug: String,
    pub name: String,
    pub tagline: String,
    pub price_cents: i32,
    pub currency: String,
    pub billing_interval: platform_product_plan::ProductPlanBillingInterval,
    pub features: Vec<String>,
    pub cta_label: String,
    pub cta_href: Option<String>,
    pub is_featured: bool,
    pub sort_order: i32,
    pub is_active: bool,
    pub billing_plan_id: Option<Uuid>,
}

#[derive(Debug, Deserialize)]
pub struct CreateProductPlanBody {
    pub slug: String,
    pub name: String,
    pub tagline: Option<String>,
    pub price_cents: Option<i32>,
    pub currency: Option<String>,
    pub billing_interval: Option<platform_product_plan::ProductPlanBillingInterval>,
    #[serde(default)]
    pub features: Vec<String>,
    pub cta_label: Option<String>,
    pub cta_href: Option<String>,
    pub is_featured: Option<bool>,
    pub sort_order: Option<i32>,
    pub is_active: Option<bool>,
    pub billing_plan_id: Option<Uuid>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateProductPlanBody {
    pub slug: Option<String>,
    pub name: Option<String>,
    pub tagline: Option<String>,
    pub price_cents: Option<i32>,
    pub currency: Option<String>,
    pub billing_interval: Option<platform_product_plan::ProductPlanBillingInterval>,
    pub features: Option<Vec<String>>,
    pub cta_label: Option<String>,
    pub cta_href: Option<String>,
    pub is_featured: Option<bool>,
    pub sort_order: Option<i32>,
    pub is_active: Option<bool>,
    pub billing_plan_id: Option<Uuid>,
}

// ── Handlers ──────────────────────────────────────────────────────────────────

pub async fn list_products(Extension(db): Extension<DatabaseConnection>) -> impl IntoResponse {
    match platform_product::Entity::find().all(&db).await {
        Ok(products) => (StatusCode::OK, Json(products)).into_response(),
        Err(e) => {
            tracing::error!("list_products: {e:#}");
            (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response()
        }
    }
}

pub async fn get_product(
    Extension(db): Extension<DatabaseConnection>,
    Path(product_id): Path<Uuid>,
) -> impl IntoResponse {
    match platform_product::Entity::find_by_id(product_id)
        .one(&db)
        .await
    {
        Ok(Some(p)) => (StatusCode::OK, Json(p)).into_response(),
        Ok(None) => (StatusCode::NOT_FOUND, "product not found").into_response(),
        Err(e) => {
            tracing::error!("get_product: {e:#}");
            (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response()
        }
    }
}

pub async fn update_product(
    Extension(db): Extension<DatabaseConnection>,
    Path(product_id): Path<Uuid>,
    Json(body): Json<UpdateProductBody>,
) -> impl IntoResponse {
    let existing = match platform_product::Entity::find_by_id(product_id)
        .one(&db)
        .await
    {
        Ok(Some(p)) => p,
        Ok(None) => return (StatusCode::NOT_FOUND, "product not found").into_response(),
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    };

    let mut active: platform_product::ActiveModel = existing.into();
    if let Some(name) = body.name {
        active.name = Set(name);
    }
    if let Some(tagline) = body.tagline {
        active.tagline = Set(Some(tagline));
    }
    if let Some(status) = body.status {
        active.status = Set(status);
    }
    if let Some(url) = body.deploy_hook_url {
        active.deploy_hook_url = Set(Some(url));
    }
    if let Some(cms_id) = body.marketing_page_cms_id {
        active.marketing_page_cms_id = Set(Some(cms_id));
    }

    match active.update(&db).await {
        Ok(updated) => (StatusCode::OK, Json(updated)).into_response(),
        Err(e) => {
            tracing::error!("update_product: {e:#}");
            (StatusCode::UNPROCESSABLE_ENTITY, e.to_string()).into_response()
        }
    }
}

fn features_from_value(value: &serde_json::Value) -> Vec<String> {
    value
        .as_array()
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str().map(str::to_string))
                .collect()
        })
        .unwrap_or_default()
}

fn product_plan_response(plan: platform_product_plan::Model) -> ProductPlanResponse {
    ProductPlanResponse {
        id: plan.id,
        product_id: plan.product_id,
        slug: plan.slug,
        name: plan.name,
        tagline: plan.tagline,
        price_cents: plan.price_cents,
        currency: plan.currency,
        billing_interval: plan.billing_interval,
        features: features_from_value(&plan.features),
        cta_label: plan.cta_label,
        cta_href: plan.cta_href,
        is_featured: plan.is_featured,
        sort_order: plan.sort_order,
        is_active: plan.is_active,
        billing_plan_id: plan.billing_plan_id,
    }
}

async fn ensure_product_exists(
    db: &DatabaseConnection,
    product_id: Uuid,
) -> Result<(), axum::response::Response> {
    match platform_product::Entity::find_by_id(product_id)
        .one(db)
        .await
    {
        Ok(Some(_)) => Ok(()),
        Ok(None) => Err((StatusCode::NOT_FOUND, "product not found").into_response()),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response()),
    }
}

pub async fn list_product_plans(
    Extension(db): Extension<DatabaseConnection>,
    Path(product_id): Path<Uuid>,
) -> impl IntoResponse {
    if let Err(resp) = ensure_product_exists(&db, product_id).await {
        return resp;
    }

    match platform_product_plan::Entity::find()
        .filter(platform_product_plan::Column::ProductId.eq(product_id))
        .order_by_asc(platform_product_plan::Column::SortOrder)
        .order_by_asc(platform_product_plan::Column::CreatedAt)
        .all(&db)
        .await
    {
        Ok(plans) => (
            StatusCode::OK,
            Json(
                plans
                    .into_iter()
                    .map(product_plan_response)
                    .collect::<Vec<_>>(),
            ),
        )
            .into_response(),
        Err(e) => {
            tracing::error!("list_product_plans: {e:#}");
            (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response()
        }
    }
}

pub async fn create_product_plan(
    Extension(db): Extension<DatabaseConnection>,
    Path(product_id): Path<Uuid>,
    Json(body): Json<CreateProductPlanBody>,
) -> impl IntoResponse {
    if let Err(resp) = ensure_product_exists(&db, product_id).await {
        return resp;
    }
    if body.slug.trim().is_empty() || body.name.trim().is_empty() {
        return (
            StatusCode::UNPROCESSABLE_ENTITY,
            "slug and name are required",
        )
            .into_response();
    }

    let now = Utc::now().into();
    let plan = platform_product_plan::ActiveModel {
        id: Set(Uuid::new_v4()),
        product_id: Set(product_id),
        slug: Set(body.slug),
        name: Set(body.name),
        tagline: Set(body.tagline.unwrap_or_default()),
        price_cents: Set(body.price_cents.unwrap_or(0)),
        currency: Set(body.currency.unwrap_or_else(|| "USD".to_string())),
        billing_interval: Set(body
            .billing_interval
            .unwrap_or(platform_product_plan::ProductPlanBillingInterval::Month)),
        features: Set(json!(body.features)),
        cta_label: Set(body.cta_label.unwrap_or_else(|| "Get started".to_string())),
        cta_href: Set(body.cta_href),
        is_featured: Set(body.is_featured.unwrap_or(false)),
        sort_order: Set(body.sort_order.unwrap_or(0)),
        is_active: Set(body.is_active.unwrap_or(true)),
        billing_plan_id: Set(body.billing_plan_id),
        created_at: Set(now),
        updated_at: Set(now),
    };

    match plan.insert(&db).await {
        Ok(created) => (StatusCode::CREATED, Json(product_plan_response(created))).into_response(),
        Err(e) => {
            tracing::error!("create_product_plan: {e:#}");
            (StatusCode::UNPROCESSABLE_ENTITY, e.to_string()).into_response()
        }
    }
}

pub async fn update_product_plan(
    Extension(db): Extension<DatabaseConnection>,
    Path((product_id, plan_id)): Path<(Uuid, Uuid)>,
    Json(body): Json<UpdateProductPlanBody>,
) -> impl IntoResponse {
    let plan = match platform_product_plan::Entity::find_by_id(plan_id)
        .filter(platform_product_plan::Column::ProductId.eq(product_id))
        .one(&db)
        .await
    {
        Ok(Some(plan)) => plan,
        Ok(None) => return (StatusCode::NOT_FOUND, "plan not found").into_response(),
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    };

    let mut active = plan.into_active_model();
    if let Some(slug) = body.slug {
        active.slug = Set(slug);
    }
    if let Some(name) = body.name {
        active.name = Set(name);
    }
    if let Some(tagline) = body.tagline {
        active.tagline = Set(tagline);
    }
    if let Some(price_cents) = body.price_cents {
        active.price_cents = Set(price_cents);
    }
    if let Some(currency) = body.currency {
        active.currency = Set(currency);
    }
    if let Some(interval) = body.billing_interval {
        active.billing_interval = Set(interval);
    }
    if let Some(features) = body.features {
        active.features = Set(json!(features));
    }
    if let Some(cta_label) = body.cta_label {
        active.cta_label = Set(cta_label);
    }
    if let Some(cta_href) = body.cta_href {
        active.cta_href = Set(Some(cta_href));
    }
    if let Some(is_featured) = body.is_featured {
        active.is_featured = Set(is_featured);
    }
    if let Some(sort_order) = body.sort_order {
        active.sort_order = Set(sort_order);
    }
    if let Some(is_active) = body.is_active {
        active.is_active = Set(is_active);
    }
    if let Some(billing_plan_id) = body.billing_plan_id {
        active.billing_plan_id = Set(Some(billing_plan_id));
    }
    active.updated_at = Set(Utc::now().into());

    match active.update(&db).await {
        Ok(updated) => (StatusCode::OK, Json(product_plan_response(updated))).into_response(),
        Err(e) => {
            tracing::error!("update_product_plan: {e:#}");
            (StatusCode::UNPROCESSABLE_ENTITY, e.to_string()).into_response()
        }
    }
}

pub async fn delete_product_plan(
    Extension(db): Extension<DatabaseConnection>,
    Path((product_id, plan_id)): Path<(Uuid, Uuid)>,
) -> impl IntoResponse {
    match platform_product_plan::Entity::delete_many()
        .filter(platform_product_plan::Column::Id.eq(plan_id))
        .filter(platform_product_plan::Column::ProductId.eq(product_id))
        .exec(&db)
        .await
    {
        Ok(result) if result.rows_affected > 0 => (
            StatusCode::OK,
            Json(json!({ "id": plan_id, "status": "deleted" })),
        )
            .into_response(),
        Ok(_) => (StatusCode::NOT_FOUND, "plan not found").into_response(),
        Err(e) => {
            tracing::error!("delete_product_plan: {e:#}");
            (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response()
        }
    }
}

pub async fn publish_marketing(
    Extension(db): Extension<DatabaseConnection>,
    Path(product_id): Path<Uuid>,
) -> impl IntoResponse {
    let product = match platform_product::Entity::find_by_id(product_id)
        .one(&db)
        .await
    {
        Ok(Some(p)) => p,
        Ok(None) => return (StatusCode::NOT_FOUND, "product not found").into_response(),
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    };

    let hook_url = match &product.deploy_hook_url {
        Some(url) if !url.is_empty() => url.clone(),
        _ => {
            return (
                StatusCode::UNPROCESSABLE_ENTITY,
                "no deploy_hook_url configured for this product",
            )
                .into_response();
        }
    };

    // Fire the deploy hook (Cloudflare Pages: POST with empty body)
    let client = reqwest::Client::new();
    match client.post(&hook_url).send().await {
        Ok(resp) if resp.status().is_success() => {
            let deploy_id = Uuid::new_v4(); // Cloudflare returns a build ID — store if needed
            tracing::info!(
                product_id = %product_id,
                product_slug = %product.slug,
                "marketing deploy triggered"
            );
            (
                StatusCode::ACCEPTED,
                Json(serde_json::json!({
                    "deploy_id": deploy_id,
                    "status": "deploying",
                    "product_slug": product.slug,
                    "triggered_at": Utc::now().to_rfc3339(),
                })),
            )
                .into_response()
        }
        Ok(resp) => {
            let status = resp.status().as_u16();
            tracing::error!(%product_id, "deploy hook returned {status}");
            (
                StatusCode::BAD_GATEWAY,
                format!("deploy hook returned HTTP {status}"),
            )
                .into_response()
        }
        Err(e) => {
            tracing::error!(%product_id, "deploy hook failed: {e:#}");
            (StatusCode::BAD_GATEWAY, e.to_string()).into_response()
        }
    }
}

pub async fn get_deploy_status(
    Extension(db): Extension<DatabaseConnection>,
    Path(product_id): Path<Uuid>,
) -> impl IntoResponse {
    let product = match platform_product::Entity::find_by_id(product_id)
        .one(&db)
        .await
    {
        Ok(Some(p)) => p,
        Ok(None) => return (StatusCode::NOT_FOUND, "product not found").into_response(),
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    };

    // Cloudflare Pages deploy status could be stored in a deploy_log table in future.
    // For now: if deploy_hook_url is set, status is "unknown" (no polling yet);
    // if not set, status is "no_hook".
    let status = if product.deploy_hook_url.is_some() {
        "unknown"
    } else {
        "no_hook"
    };

    (
        StatusCode::OK,
        Json(DeployStatusResponse {
            product_id,
            status: status.to_string(),
            deployed_at: None,
            message: Some("Real-time deploy status polling coming in Phase 2.".to_string()),
        }),
    )
        .into_response()
}

// ── Tracking pixels ───────────────────────────────────────────────────────────

#[derive(Debug, Serialize)]
pub struct ProductPixelResponse {
    pub id: Uuid,
    pub product_id: Uuid,
    pub name: String,
    pub pixel_type: String,
    pub snippet: String,
    pub inject_at: String,
    pub is_active: bool,
}

impl From<product_tracking_pixel::Model> for ProductPixelResponse {
    fn from(m: product_tracking_pixel::Model) -> Self {
        Self {
            id: m.id,
            product_id: m.product_id,
            name: m.name,
            pixel_type: m.pixel_type,
            snippet: m.snippet,
            inject_at: m.inject_at,
            is_active: m.is_active,
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct CreateProductPixelBody {
    pub name: String,
    pub pixel_type: String,
    pub snippet: String,
    pub inject_at: Option<String>,
}

pub async fn list_product_pixels(
    Extension(db): Extension<DatabaseConnection>,
    Path(product_id): Path<Uuid>,
) -> impl IntoResponse {
    if platform_product::Entity::find_by_id(product_id)
        .one(&db)
        .await
        .ok()
        .flatten()
        .is_none()
    {
        return (StatusCode::NOT_FOUND, "product not found").into_response();
    }

    match product_tracking_pixel::Entity::find()
        .filter(product_tracking_pixel::Column::ProductId.eq(product_id))
        .order_by_asc(product_tracking_pixel::Column::CreatedAt)
        .all(&db)
        .await
    {
        Ok(rows) => {
            let out: Vec<ProductPixelResponse> = rows.into_iter().map(Into::into).collect();
            (StatusCode::OK, Json(out)).into_response()
        }
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}

pub async fn create_product_pixel(
    Extension(db): Extension<DatabaseConnection>,
    Path(product_id): Path<Uuid>,
    Json(body): Json<CreateProductPixelBody>,
) -> impl IntoResponse {
    if platform_product::Entity::find_by_id(product_id)
        .one(&db)
        .await
        .ok()
        .flatten()
        .is_none()
    {
        return (StatusCode::NOT_FOUND, "product not found").into_response();
    }

    let pixel_type = match PixelType::try_from(body.pixel_type.as_str()) {
        Ok(t) => t.to_string(),
        Err(e) => return (StatusCode::UNPROCESSABLE_ENTITY, e).into_response(),
    };
    let inject_at = match InjectAt::try_from(body.inject_at.as_deref().unwrap_or("head")) {
        Ok(v) => v.to_string(),
        Err(e) => return (StatusCode::UNPROCESSABLE_ENTITY, e).into_response(),
    };
    let name = body.name.trim().to_string();
    let snippet = body.snippet.trim().to_string();
    if name.is_empty() || snippet.is_empty() {
        return (
            StatusCode::UNPROCESSABLE_ENTITY,
            "name and snippet are required",
        )
            .into_response();
    }

    let now = Utc::now();
    let model = product_tracking_pixel::ActiveModel {
        id: Set(Uuid::new_v4()),
        product_id: Set(product_id),
        name: Set(name),
        pixel_type: Set(pixel_type),
        snippet: Set(snippet),
        inject_at: Set(inject_at),
        is_active: Set(true),
        created_at: Set(now.into()),
        updated_at: Set(now.into()),
    };

    match model.insert(&db).await {
        Ok(row) => (StatusCode::CREATED, Json(ProductPixelResponse::from(row))).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}

pub async fn delete_product_pixel(
    Extension(db): Extension<DatabaseConnection>,
    Path((product_id, pixel_id)): Path<(Uuid, Uuid)>,
) -> impl IntoResponse {
    match product_tracking_pixel::Entity::find_by_id(pixel_id)
        .filter(product_tracking_pixel::Column::ProductId.eq(product_id))
        .one(&db)
        .await
    {
        Ok(Some(row)) => {
            let active: product_tracking_pixel::ActiveModel = row.into();
            match active.delete(&db).await {
                Ok(_) => StatusCode::NO_CONTENT.into_response(),
                Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
            }
        }
        Ok(None) => (StatusCode::NOT_FOUND, "pixel not found").into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}
