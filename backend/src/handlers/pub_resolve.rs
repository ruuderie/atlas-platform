//! Public — Domain Resolver
//!
//! Zero-auth endpoint that resolves any incoming domain (or domain + path) to
//! its corresponding product and optional market variant. Used by:
//!
//! - `folio.app` CDN edge worker / middleware: "what product am I serving?"
//! - `miami.folio.app`: "what variant?"
//! - `listings.oakwoodpm.com`: "what tenant context AND product variant?"
//!
//! ## Route
//!
//! ```ignore
//! GET /api/pub/resolve
//!     Query params:
//!       domain  — e.g. "folio.app" or "miami.folio.app" or "listings.oakwoodpm.com"
//!       path    — optional, e.g. "/miami" for path-based variant routing on folio.app
//!
//!     Response 200:
//!     {
//!       "resolution_type": "product" | "variant" | "tenant_app" | "not_found",
//!       "product_id": "uuid",
//!       "product_slug": "folio",
//!       "product_name": "Folio",
//!       "apex_domain": "folio.app",
//!       "variant_id": "uuid" | null,
//!       "variant_slug": "miami-fl" | null,
//!       "locale": "en-US",
//!       "launch_mode": "waitlist",
//!       // For white-label tenant landing pages:
//!       "tenant_id": "uuid" | null,
//!       "brand_name": "Oakwood PM" | null,
//!     }
//!
//!     Response 404: domain not registered
//! ```
//!
//! Cache-Control: public, s-maxage=3600 — CDN caches resolution per domain.
//! Cloudflare Workers can call this once on first request, cache at edge, avoid
//! round-tripping to origin on every visitor.
//!
//! ## Resolution priority
//!
//! 1. Exact match in `product_domain_aliases` (domain + path_prefix)
//! 2. Subdomain match: strip first label → check `platform_products.apex_domain`
//! 3. Exact match on `platform_products.apex_domain` (root domain)
//! 4. Check `atlas_app_deployment_config.custom_domain` (tenant-level white-label)
//! 5. 404

use axum::{
    extract::{Extension, Query},
    http::{HeaderMap, StatusCode, header},
    response::IntoResponse,
    Json,
};
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::entities::{
    atlas_app_deployment_config,
    platform_product,
    product_page::variant,
};

// ── Types ─────────────────────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct ResolveQuery {
    pub domain: String,
    pub path: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct ResolveResponse {
    pub resolution_type: String, // "product" | "variant" | "tenant_app" | "not_found"
    pub product_id:      Option<Uuid>,
    pub product_slug:    Option<String>,
    pub product_name:    Option<String>,
    pub apex_domain:     Option<String>,
    pub variant_id:      Option<Uuid>,
    pub variant_slug:    Option<String>,
    pub locale:          Option<String>,
    pub launch_mode:     Option<String>,
    // Tenant-level (white-label app deployments)
    pub tenant_id:       Option<Uuid>,
    pub brand_name:      Option<String>,
}

fn cdn_headers() -> HeaderMap {
    let mut h = HeaderMap::new();
    h.insert(
        header::CACHE_CONTROL,
        "public, s-maxage=3600, stale-while-revalidate=86400".parse().unwrap(),
    );
    h
}

// ── Handler ───────────────────────────────────────────────────────────────────

pub async fn resolve_domain(
    Extension(db): Extension<DatabaseConnection>,
    Query(q): Query<ResolveQuery>,
) -> impl IntoResponse {
    let domain = q.domain.to_lowercase().trim_matches('/').to_string();
    let path = q.path.as_deref().unwrap_or("").trim_matches('/').to_string();

    // ── Step 1: Check product_domain_aliases (exact domain + optional path) ──
    // TODO: wire product_domain_alias entity when entity is generated
    // For now proceed to steps 2-4.

    // ── Step 2: Subdomain resolution (miami.folio.app → folio apex, miami variant) ──
    if let Some(resp) = try_resolve_subdomain(&db, &domain).await {
        let mut response = (StatusCode::OK, Json(resp)).into_response();
        *response.headers_mut() = cdn_headers();
        return response;
    }

    // ── Step 3: Root apex domain (folio.app → product master) ──
    if let Some(resp) = try_resolve_apex(&db, &domain, &path).await {
        let mut response = (StatusCode::OK, Json(resp)).into_response();
        *response.headers_mut() = cdn_headers();
        return response;
    }

    // ── Step 4: Tenant white-label custom_domain (atlas_app_deployment_config) ──
    if let Some(resp) = try_resolve_tenant_domain(&db, &domain).await {
        let mut response = (StatusCode::OK, Json(resp)).into_response();
        *response.headers_mut() = cdn_headers();
        return response;
    }

    // ── 404 ──────────────────────────────────────────────────────────────────
    let resp = ResolveResponse {
        resolution_type: "not_found".to_string(),
        product_id:   None, product_slug:  None, product_name: None,
        apex_domain:  None, variant_id:    None, variant_slug: None,
        locale:       None, launch_mode:   None,
        tenant_id:    None, brand_name:    None,
    };
    (StatusCode::NOT_FOUND, Json(resp)).into_response()
}

// ── Resolution helpers ────────────────────────────────────────────────────────

/// "miami.folio.app" → apex_domain="folio.app", then find variant with subdomain_override="miami"
async fn try_resolve_subdomain(db: &DatabaseConnection, domain: &str) -> Option<ResolveResponse> {
    let parts: Vec<&str> = domain.splitn(2, '.').collect();
    if parts.len() < 2 {
        return None;
    }
    let subdomain = parts[0];
    let apex = parts[1];

    let product = platform_product::Entity::find()
        .filter(platform_product::Column::ApexDomain.eq(apex))
        .filter(platform_product::Column::ApexDomainVerified.eq(true))
        .one(db)
        .await
        .ok()??;

    // Find variant with matching subdomain_override
    let v = variant::Entity::find()
        .filter(variant::Column::ProductId.eq(product.id))
        .filter(variant::Column::SubdomainOverride.eq(subdomain))
        .filter(variant::Column::IsPublished.eq(true))
        .one(db)
        .await
        .ok()??;

    Some(ResolveResponse {
        resolution_type: "variant".to_string(),
        product_id:   Some(product.id),
        product_slug: Some(product.slug.clone()),
        product_name: Some(product.name.clone()),
        apex_domain:  Some(apex.to_string()),
        variant_id:   Some(v.id),
        variant_slug: Some(v.variant_slug.clone()),
        locale:       Some(v.locale.clone()),
        launch_mode:  Some(v.launch_mode.clone()),
        tenant_id:    None,
        brand_name:   None,
    })
}

/// "folio.app" → product master; "folio.app" + path="/miami" → check path-based variant
async fn try_resolve_apex(db: &DatabaseConnection, domain: &str, path: &str) -> Option<ResolveResponse> {
    let product = platform_product::Entity::find()
        .filter(platform_product::Column::ApexDomain.eq(domain))
        .filter(platform_product::Column::ApexDomainVerified.eq(true))
        .one(db)
        .await
        .ok()??;

    // If a path is provided, try to match it to a variant_slug
    if !path.is_empty() {
        if let Ok(Some(v)) = variant::Entity::find()
            .filter(variant::Column::ProductId.eq(product.id))
            .filter(variant::Column::VariantSlug.eq(path))
            .filter(variant::Column::IsPublished.eq(true))
            .one(db)
            .await
        {
            return Some(ResolveResponse {
                resolution_type: "variant".to_string(),
                product_id:   Some(product.id),
                product_slug: Some(product.slug.clone()),
                product_name: Some(product.name.clone()),
                apex_domain:  Some(domain.to_string()),
                variant_id:   Some(v.id),
                variant_slug: Some(v.variant_slug.clone()),
                locale:       Some(v.locale.clone()),
                launch_mode:  Some(v.launch_mode.clone()),
                tenant_id:    None,
                brand_name:   None,
            });
        }
    }

    // Root apex → product master
    Some(ResolveResponse {
        resolution_type: "product".to_string(),
        product_id:   Some(product.id),
        product_slug: Some(product.slug.clone()),
        product_name: Some(product.name.clone()),
        apex_domain:  Some(domain.to_string()),
        variant_id:   None,
        variant_slug: None,
        locale:       Some("en".to_string()),
        launch_mode:  Some(product.launch_mode.clone()),
        tenant_id:    None,
        brand_name:   None,
    })
}

/// "listings.oakwoodpm.com" → tenant white-label app (atlas_app_deployment_config.custom_domain)
async fn try_resolve_tenant_domain(db: &DatabaseConnection, domain: &str) -> Option<ResolveResponse> {
    let cfg = atlas_app_deployment_config::Entity::find()
        .filter(atlas_app_deployment_config::Column::CustomDomain.eq(domain))
        .one(db)
        .await
        .ok()??;

    Some(ResolveResponse {
        resolution_type: "tenant_app".to_string(),
        product_id:   None,
        product_slug: Some(cfg.app_slug.clone()),
        product_name: None,
        apex_domain:  Some(domain.to_string()),
        variant_id:   None,
        variant_slug: None,
        locale:       None,
        launch_mode:  None,
        tenant_id:    Some(cfg.tenant_id),
        brand_name:   None, // TODO: join tenant.name in follow-up
    })
}
