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
    extract::{Extension, Json, Path},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, patch, post},
    Router,
};
use chrono::Utc;
use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::entities::platform_product;

// ── Route registration ────────────────────────────────────────────────────────

/// State-free router — merge before outer .with_state(db) in admin_routes().
pub fn routes_raw() -> Router<DatabaseConnection> {
    Router::new()
        .route("/api/admin/platform/products", get(list_products))
        .route(
            "/api/admin/platform/products/:id",
            get(get_product).patch(update_product),
        )
        .route(
            "/api/admin/platform/products/:id/publish-marketing",
            post(publish_marketing),
        )
        .route(
            "/api/admin/platform/products/:id/deploy-status",
            get(get_deploy_status),
        )
}

/// Convenience wrapper with state applied (for standalone use / tests).
pub fn routes(db: DatabaseConnection) -> Router {
    routes_raw().with_state(db)
}

// ── Input types ───────────────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct UpdateProductBody {
    pub name:                   Option<String>,
    pub tagline:                Option<String>,
    /// "active" | "beta" | "deprecated"
    pub status:                 Option<String>,
    pub deploy_hook_url:        Option<String>,
    pub marketing_page_cms_id:  Option<Uuid>,
}

#[derive(Debug, Serialize)]
pub struct DeployStatusResponse {
    pub product_id:  Uuid,
    pub status:      String,   // "success" | "failure" | "deploying" | "unknown" | "no_hook"
    pub deployed_at: Option<String>,
    pub message:     Option<String>,
}

// ── Handlers ──────────────────────────────────────────────────────────────────

pub async fn list_products(
    Extension(db): Extension<DatabaseConnection>,
) -> impl IntoResponse {
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
    match platform_product::Entity::find_by_id(product_id).one(&db).await {
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
    let existing = match platform_product::Entity::find_by_id(product_id).one(&db).await {
        Ok(Some(p)) => p,
        Ok(None) => return (StatusCode::NOT_FOUND, "product not found").into_response(),
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    };

    let mut active: platform_product::ActiveModel = existing.into();
    if let Some(name) = body.name             { active.name = Set(name); }
    if let Some(tagline) = body.tagline       { active.tagline = Set(Some(tagline)); }
    if let Some(status) = body.status         { active.status = Set(status); }
    if let Some(url) = body.deploy_hook_url   { active.deploy_hook_url = Set(Some(url)); }
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

pub async fn publish_marketing(
    Extension(db): Extension<DatabaseConnection>,
    Path(product_id): Path<Uuid>,
) -> impl IntoResponse {
    let product = match platform_product::Entity::find_by_id(product_id).one(&db).await {
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
    let product = match platform_product::Entity::find_by_id(product_id).one(&db).await {
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
