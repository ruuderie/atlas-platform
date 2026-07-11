//! Public — Product Launch Engine endpoints (zero-auth, CDN-cacheable)
//!
//! Serves product landing pages, market variants, lead capture (waitlist),
//! pre-order Stripe checkout, view counting, and sitemap generation.
//!
//! # SEO delivery strategy
//! These handlers return JSON (for the marketing site to SSR/ISR).
//! The marketing site (separate Cloudflare Pages deploy) fetches on first visit
//! and caches at CDN edge. HTML rendering lives in `handlers::pub_html`.
//!
//! # Routes
//!
//! ```ignore
//! GET  /api/pub/products                          List active/beta products
//! GET  /api/pub/products/:slug                    Master landing (CMS app_page, then template)
//! GET  /api/pub/products/:slug/:variant           Variant (template + overrides merged)
//! GET  /api/pub/products/:slug/sitemap.xml        Sitemap for all published variants
//! POST /api/pub/products/:slug/waitlist           Lead capture (product-level)
//! POST /api/pub/products/:slug/:variant/waitlist  Lead capture (variant/market-scoped)
//! POST /api/pub/products/:slug/pre-order          Stripe Checkout Session
//! POST /api/pub/products/:slug/:variant/view      Increment view_count (no PII)
//!
//! # Folio CMS public path registry
//!
//! - `folio/master` → `/`
//! - `folio-broker/master` → `/brokers`
//! - `folio-pm/master` → `/property-managers`
//! - `folio-vendor/master` → `/vendors`
//! ```

use axum::{
    Router,
    extract::{Extension, Json, Path, Query},
    http::{HeaderMap, StatusCode, header},
    response::IntoResponse,
    routing::{get, post},
};
use chrono::Utc;
use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter,
    QueryOrder,
};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use uuid::Uuid;

use crate::entities::{
    app_page, app_page_variant, atlas_campaign, atlas_lead, outbox_job, platform_product,
    platform_product_plan, product_page::{template, variant},
};
use crate::services::pm::campaign::{CampaignService, EnrollContactPayload, RecordEventPayload};
use crate::types::gtm::PlanTier;
use crate::types::pm::{CampaignChannel, CampaignEventType};

// ── Route registration ────────────────────────────────────────────────────────

pub fn public_routes_raw() -> Router<DatabaseConnection> {
    Router::new()
        .route("/api/pub/products", get(list_products))
        .route("/api/pub/beta-applications", post(join_beta_application))
        .route("/api/pub/products/{slug}", get(get_product_master))
        .route(
            "/api/pub/products/{slug}/sitemap.xml",
            get(get_product_sitemap),
        )
        .route(
            "/api/pub/products/{slug}/waitlist",
            post(join_waitlist_product),
        )
        .route("/api/pub/products/{slug}/pre-order", post(create_pre_order))
        .route(
            "/api/pub/products/{slug}/{variant}",
            get(get_product_variant),
        )
        .route(
            "/api/pub/products/{slug}/{variant}/waitlist",
            post(join_waitlist_variant),
        )
        .route("/api/pub/products/{slug}/{variant}/view", post(record_view))
}

pub fn public_routes(db: DatabaseConnection) -> Router {
    public_routes_raw().with_state(db)
}

// ── Response types ────────────────────────────────────────────────────────────

/// Merged page response — template fields overridden by variant
#[derive(Debug, Serialize)]
pub struct ProductPageResponse {
    pub product_id: Uuid,
    pub product_slug: String,
    pub product_name: String,
    pub variant_slug: Option<String>,
    pub locale: String,
    pub city: Option<String>,
    pub country_code: Option<String>,

    // Merged content
    pub hero: Value,
    pub blocks: Value,

    // Merged SEO (variant overrides template)
    pub meta_title: String,
    pub meta_description: String,
    pub og_image_url: Option<String>,
    pub canonical_url: String,
    pub structured_data: Value,

    // hreflang list (all published variants for this product)
    pub hreflang: Vec<HreflangEntry>,

    // CTA
    pub cta_label: String,
    pub cta_action: String,
    pub launch_mode: String,
    pub pre_order_enabled: bool,
    pub pre_order_price_cents: Option<i32>,
    pub pre_order_currency: String,
    pub pre_order_available: Option<i32>, // cap - sold (null = unlimited)

    pub waitlist_count: i32,
    pub lead_count: i32,
    pub page_id: Option<Uuid>,
    pub variant_id: Option<Uuid>,
    pub serve_source: ServeSource,
    pub plans: Vec<PublicProductPlan>,
}

#[derive(Debug, Serialize)]
pub struct PublicProductPlan {
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
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ServeSource {
    Cms,
    ProductTemplate,
}

#[derive(Debug, Serialize)]
pub struct HreflangEntry {
    pub locale: String,
    pub url: String,
    pub is_default: bool,
}

// ── Input types ───────────────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct WaitlistBody {
    // Contact
    pub email: String,
    pub name: Option<String>,
    pub phone: Option<String>,
    pub company: Option<String>,

    // Role segmentation (analytics + invite-mode hint — NOT a provisioning directive)
    /// Self-reported role: "Landlord" | "Property Manager" | "STR Host" | "Tenant" | "Vendor" | "Investor"
    pub role: Option<String>,
    /// Self-reported portfolio size label: "1\u20135" | "6\u201320" | "21\u2013100" | "100+" | "n/a"
    pub portfolio_size_label: Option<String>,

    // UTM attribution (populated by JS utm_passthrough snippet)
    pub utm_source: Option<String>,
    pub utm_medium: Option<String>,
    pub utm_campaign: Option<String>,
    pub utm_content: Option<String>,
    pub utm_term: Option<String>,

    // Click IDs
    pub gclid: Option<String>,   // Google Ads
    pub fbclid: Option<String>,  // Meta
    pub msclkid: Option<String>, // Microsoft Ads

    // Landing context
    pub referrer: Option<String>,
    pub landing_url: Option<String>,

    // Plan selection (for Trial mode — step 3 of modal)
    /// `None` means the visitor did not select a plan during signup.
    pub plan: Option<PlanTier>, // Starter | Professional | Portfolio
    pub unit_count: Option<i32>, // self-reported portfolio size as integer

    // Extended capture fields used by Folio beta/founding campaign surfaces.
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub trade: Option<String>,
    pub biz_name: Option<String>,
    pub service_area: Option<String>,
    pub current_tool: Option<String>,
    pub pain_point: Option<String>,
    pub is_active: Option<String>,
    pub feedback_call: Option<String>,
    pub why_beta: Option<String>,
    /// Optional source override hint for campaign-specific capture flows.
    pub source: Option<String>,
    /// Friends & Family / referral attribution — who shared the link.
    /// Also mirrored into utm_content when that field is empty.
    pub referred_by: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct PreOrderBody {
    pub email: String,
    pub name: Option<String>,
    pub success_url: String,
    pub cancel_url: String,
}

#[derive(Debug, Deserialize, Default)]
pub struct AbQuery {
    pub ab: Option<String>,
}

// ── Helpers ───────────────────────────────────────────────────────────────────

fn base_url() -> String {
    std::env::var("PUBLIC_BASE_URL").unwrap_or_else(|_| "https://atlas.app".to_string())
}

fn merge_json(base: &Value, overrides: &Value) -> Value {
    match (base, overrides) {
        (Value::Object(b), Value::Object(o)) => {
            let mut merged = b.clone();
            for (k, v) in o {
                merged.insert(k.clone(), v.clone());
            }
            Value::Object(merged)
        }
        _ => overrides.clone(),
    }
}

fn ab_cookie(headers: &HeaderMap) -> Option<String> {
    headers
        .get(header::COOKIE)
        .and_then(|v| v.to_str().ok())
        .and_then(|cookies| {
            cookies.split(';').find_map(|cookie| {
                let (name, value) = cookie.trim().split_once('=')?;
                (name == "atlas_lp_ab").then(|| value.to_string())
            })
        })
}

fn find_requested_variant(
    variants: &[app_page_variant::Model],
    requested: Option<&str>,
) -> Option<app_page_variant::Model> {
    let requested = requested?;
    variants
        .iter()
        .find(|v| v.id.to_string() == requested || v.name.eq_ignore_ascii_case(requested))
        .cloned()
}

fn choose_weighted_variant(
    variants: &[app_page_variant::Model],
) -> Option<app_page_variant::Model> {
    if variants.is_empty() {
        return None;
    }

    let total_weight: i32 = variants.iter().map(|v| v.traffic_pct.max(0)).sum();
    if total_weight <= 0 {
        return variants.first().cloned();
    }

    let mut bucket = (rand::random::<u32>() % total_weight as u32) as i32;
    for variant in variants {
        bucket -= variant.traffic_pct.max(0);
        if bucket < 0 {
            return Some(variant.clone());
        }
    }

    variants.last().cloned()
}

async fn select_app_page_variant(
    db: &DatabaseConnection,
    page_id: Uuid,
    requested: Option<String>,
) -> Result<Option<(app_page_variant::Model, bool)>, axum::response::Response> {
    let variants = app_page_variant::Entity::find()
        .filter(app_page_variant::Column::PageId.eq(page_id))
        .filter(app_page_variant::Column::IsActive.eq(true))
        .order_by_asc(app_page_variant::Column::CreatedAt)
        .all(db)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response())?;

    if variants.is_empty() {
        return Ok(None);
    }

    if let Some(v) = find_requested_variant(&variants, requested.as_deref()) {
        return Ok(Some((v, false)));
    }

    Ok(choose_weighted_variant(&variants).map(|v| (v, true)))
}

fn apply_ab_cookie(resp: &mut axum::response::Response, variant_id: Uuid) {
    resp.headers_mut().insert(
        header::SET_COOKIE,
        format!(
            "atlas_lp_ab={variant_id}; HttpOnly; Path=/; SameSite=Lax; Max-Age={}",
            60 * 60 * 24 * 30
        )
        .parse()
        .unwrap(),
    );
}

fn public_plan_from_model(plan: platform_product_plan::Model) -> PublicProductPlan {
    let features = plan
        .features
        .as_array()
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str().map(str::to_string))
                .collect()
        })
        .unwrap_or_default();

    PublicProductPlan {
        slug: plan.slug,
        name: plan.name,
        tagline: plan.tagline,
        price_cents: plan.price_cents,
        currency: plan.currency,
        billing_interval: plan.billing_interval,
        features,
        cta_label: plan.cta_label,
        cta_href: plan.cta_href,
        is_featured: plan.is_featured,
        sort_order: plan.sort_order,
    }
}

async fn load_public_plans(db: &DatabaseConnection, product_id: Uuid) -> Vec<PublicProductPlan> {
    platform_product_plan::Entity::find()
        .filter(platform_product_plan::Column::ProductId.eq(product_id))
        .filter(platform_product_plan::Column::IsActive.eq(true))
        .order_by_asc(platform_product_plan::Column::SortOrder)
        .order_by_asc(platform_product_plan::Column::CreatedAt)
        .all(db)
        .await
        .map(|plans| plans.into_iter().map(public_plan_from_model).collect())
        .unwrap_or_default()
}

async fn load_public_plans_by_slug(
    db: &DatabaseConnection,
    product_slug: &str,
) -> Vec<PublicProductPlan> {
    match platform_product::Entity::find()
        .filter(platform_product::Column::Slug.eq(product_slug))
        .one(db)
        .await
    {
        Ok(Some(product)) => load_public_plans(db, product.id).await,
        _ => vec![],
    }
}

fn cdn_headers() -> HeaderMap {
    let mut headers = HeaderMap::new();
    headers.insert(
        header::CACHE_CONTROL,
        "public, s-maxage=3600, stale-while-revalidate=86400"
            .parse()
            .unwrap(),
    );
    headers
}

async fn load_product_and_template(
    db: &DatabaseConnection,
    slug: &str,
) -> Result<(platform_product::Model, template::Model), axum::response::Response> {
    let product = platform_product::Entity::find()
        .filter(platform_product::Column::Slug.eq(slug))
        .one(db)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response())?
        .ok_or_else(|| (StatusCode::NOT_FOUND, "product not found").into_response())?;

    let tmpl = template::Entity::find()
        .filter(template::Column::ProductId.eq(product.id))
        .one(db)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response())?
        .ok_or_else(|| {
            (StatusCode::NOT_FOUND, "product template not configured").into_response()
        })?;

    Ok((product, tmpl))
}

async fn load_hreflang(
    db: &DatabaseConnection,
    product_id: Uuid,
    product_slug: &str,
) -> Vec<HreflangEntry> {
    let variants = variant::Entity::find()
        .filter(variant::Column::ProductId.eq(product_id))
        .filter(variant::Column::IsPublished.eq(true))
        .all(db)
        .await
        .unwrap_or_default();

    let base = base_url();
    let mut entries: Vec<HreflangEntry> = variants
        .iter()
        .map(|v| HreflangEntry {
            locale: v.locale.clone(),
            url: format!("{base}/products/{product_slug}/{}", v.variant_slug),
            is_default: false,
        })
        .collect();

    // x-default → master page
    entries.push(HreflangEntry {
        locale: "x-default".to_string(),
        url: format!("{base}/products/{product_slug}"),
        is_default: true,
    });

    entries
}

fn build_local_business_jsonld(product: &platform_product::Model, v: &variant::Model) -> Value {
    json!({
        "@context": "https://schema.org",
        "@type": "SoftwareApplication",
        "name": format!("{} — {}", product.name, v.city.as_deref().unwrap_or("")),
        "applicationCategory": "BusinessApplication",
        "operatingSystem": "Web",
        "offers": {
            "@type": "Offer",
            "availability": "https://schema.org/InStock"
        },
        "areaServed": {
            "@type": "Place",
            "name": v.city.as_deref().unwrap_or(v.country_code.as_deref().unwrap_or(""))
        }
    })
}

// ── Handlers ──────────────────────────────────────────────────────────────────

async fn list_products(Extension(db): Extension<DatabaseConnection>) -> impl IntoResponse {
    let products = platform_product::Entity::find()
        .filter(platform_product::Column::Status.is_in(["active", "beta"]))
        .all(&db)
        .await;

    match products {
        Ok(p) => {
            let mut resp = (StatusCode::OK, Json(p)).into_response();
            *resp.headers_mut() = cdn_headers();
            resp
        }
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}

async fn get_product_master(
    Extension(db): Extension<DatabaseConnection>,
    Path(slug): Path<String>,
    Query(q): Query<AbQuery>,
    headers: HeaderMap,
) -> impl IntoResponse {
    // ── Builder-first: check app_pages for a published master page ────────────
    // Convention: the master page has slug = "master" OR slug = the app_id itself.
    // We check both to be flexible.
    let builder_page = app_page::Entity::find()
        .filter(app_page::Column::AppId.eq(&slug))
        .filter(app_page::Column::IsPublished.eq(true))
        .filter(
            sea_orm::Condition::any()
                .add(app_page::Column::Slug.eq("master"))
                .add(app_page::Column::Slug.eq(&slug)),
        )
        .one(&db)
        .await
        .ok()
        .flatten();

    if let Some(pg) = builder_page {
        let selected_variant =
            match select_app_page_variant(&db, pg.id, q.ab.or_else(|| ab_cookie(&headers))).await {
                Ok(v) => v,
                Err(e) => return e,
            };
        let (hero, blocks, variant_id, should_set_cookie) =
            if let Some((v, should_set_cookie)) = selected_variant {
                let hero = v
                    .hero_payload
                    .as_ref()
                    .map(|h| merge_json(pg.hero_payload.as_ref().unwrap_or(&Value::Null), h))
                    .unwrap_or_else(|| pg.hero_payload.clone().unwrap_or_default());
                let blocks = merge_json(
                    pg.blocks_payload.as_ref().unwrap_or(&Value::Null),
                    &v.blocks_payload,
                );
                (hero, blocks, Some(v.id), should_set_cookie)
            } else {
                (
                    pg.hero_payload.clone().unwrap_or_default(),
                    pg.blocks_payload.clone().unwrap_or_default(),
                    None,
                    false,
                )
            };
        let hreflang = load_hreflang(&db, pg.id, &slug).await;
        let plans = load_public_plans_by_slug(&db, &slug).await;
        let product_waitlist_count = platform_product::Entity::find()
            .filter(platform_product::Column::Slug.eq(&slug))
            .one(&db)
            .await
            .ok()
            .flatten()
            .map(|product| product.waitlist_count)
            .unwrap_or_default();
        let base = base_url();
        let canonical = format!("{base}/products/{slug}");
        let page = ProductPageResponse {
            product_id: pg.id,
            product_slug: slug.clone(),
            product_name: pg.title.clone(),
            variant_slug: None,
            locale: "en".to_string(),
            city: None,
            country_code: None,
            hero,
            blocks,
            meta_title: pg.title.clone(),
            meta_description: pg.description.clone(),
            og_image_url: None,
            canonical_url: canonical,
            structured_data: serde_json::Value::Null,
            hreflang,
            cta_label: "Get Started".to_string(),
            cta_action: format!("/signup"),
            launch_mode: "active".to_string(),
            pre_order_enabled: false,
            pre_order_price_cents: None,
            pre_order_currency: "usd".to_string(),
            pre_order_available: None,
            waitlist_count: product_waitlist_count,
            lead_count: 0,
            page_id: Some(pg.id),
            variant_id,
            serve_source: ServeSource::Cms,
            plans,
        };
        let mut resp = (StatusCode::OK, Json(page)).into_response();
        *resp.headers_mut() = cdn_headers();
        if should_set_cookie {
            if let Some(id) = variant_id {
                apply_ab_cookie(&mut resp, id);
            }
        }
        return resp;
    }

    // ── Fallback: product_page_templates + product_page_variants ──────────────
    let (product, tmpl) = match load_product_and_template(&db, &slug).await {
        Ok(r) => r,
        Err(e) => return e,
    };

    let hreflang = load_hreflang(&db, product.id, &slug).await;
    let plans = load_public_plans(&db, product.id).await;
    let base = base_url();

    let meta_title = tmpl.meta_title.clone().unwrap_or_else(|| {
        format!(
            "{} — {}",
            product.name,
            product.tagline.as_deref().unwrap_or("")
        )
    });
    let meta_description = tmpl.meta_description.clone().unwrap_or_default();
    let canonical = format!("{base}/products/{slug}");

    let page = ProductPageResponse {
        product_id: product.id,
        product_slug: product.slug.clone(),
        product_name: product.name.clone(),
        variant_slug: None,
        locale: "en".to_string(),
        city: None,
        country_code: None,
        hero: tmpl.hero_payload.clone(),
        blocks: tmpl.blocks_payload.clone(),
        meta_title,
        meta_description,
        og_image_url: tmpl.og_image_url.clone(),
        canonical_url: canonical,
        structured_data: tmpl.structured_data.clone(),
        hreflang,
        cta_label: tmpl.cta_label.clone(),
        cta_action: tmpl.cta_action.clone(),
        launch_mode: product.launch_mode.clone(),
        pre_order_enabled: product.pre_order_enabled,
        pre_order_price_cents: product.pre_order_price_cents,
        pre_order_currency: product.pre_order_currency.clone(),
        pre_order_available: product
            .pre_order_cap
            .map(|cap| cap - product.pre_order_sold),
        waitlist_count: product.waitlist_count,
        lead_count: 0,
        page_id: None,
        variant_id: None,
        serve_source: ServeSource::ProductTemplate,
        plans,
    };

    let mut resp = (StatusCode::OK, Json(page)).into_response();
    *resp.headers_mut() = cdn_headers();
    resp
}

async fn get_product_variant(
    Extension(db): Extension<DatabaseConnection>,
    Path((slug, variant_slug)): Path<(String, String)>,
    Query(q): Query<AbQuery>,
    headers: HeaderMap,
) -> impl IntoResponse {
    // ── Builder-first: check app_pages for a published variant page ───────────
    let builder_variant = app_page::Entity::find()
        .filter(app_page::Column::AppId.eq(&slug))
        .filter(app_page::Column::Slug.eq(&variant_slug))
        .filter(app_page::Column::IsPublished.eq(true))
        .one(&db)
        .await
        .ok()
        .flatten();

    if let Some(pg) = builder_variant {
        let selected_variant =
            match select_app_page_variant(&db, pg.id, q.ab.or_else(|| ab_cookie(&headers))).await {
                Ok(v) => v,
                Err(e) => return e,
            };
        let (hero, blocks, ab_variant_id, should_set_cookie) =
            if let Some((v, should_set_cookie)) = selected_variant {
                let hero = v
                    .hero_payload
                    .as_ref()
                    .map(|h| merge_json(pg.hero_payload.as_ref().unwrap_or(&Value::Null), h))
                    .unwrap_or_else(|| pg.hero_payload.clone().unwrap_or_default());
                let blocks = merge_json(
                    pg.blocks_payload.as_ref().unwrap_or(&Value::Null),
                    &v.blocks_payload,
                );
                (hero, blocks, Some(v.id), should_set_cookie)
            } else {
                (
                    pg.hero_payload.clone().unwrap_or_default(),
                    pg.blocks_payload.clone().unwrap_or_default(),
                    None,
                    false,
                )
            };
        // For variants we look up the parent product for hreflang + product context
        let hreflang = if let Ok(Some(product)) = platform_product::Entity::find()
            .filter(platform_product::Column::Slug.eq(&slug))
            .one(&db)
            .await
        {
            load_hreflang(&db, product.id, &slug).await
        } else {
            vec![]
        };

        let base = base_url();
        let plans = load_public_plans_by_slug(&db, &slug).await;
        let product_waitlist_count = platform_product::Entity::find()
            .filter(platform_product::Column::Slug.eq(&slug))
            .one(&db)
            .await
            .ok()
            .flatten()
            .map(|product| product.waitlist_count)
            .unwrap_or_default();
        let canonical = format!("{base}/products/{slug}/{variant_slug}");
        let page = ProductPageResponse {
            product_id: pg.id,
            product_slug: slug.clone(),
            product_name: pg.title.clone(),
            variant_slug: Some(variant_slug.clone()),
            locale: "en".to_string(),
            city: None,
            country_code: None,
            hero,
            blocks,
            meta_title: pg.title.clone(),
            meta_description: pg.description.clone(),
            og_image_url: None,
            canonical_url: canonical,
            structured_data: serde_json::Value::Null,
            hreflang,
            cta_label: "Get Started".to_string(),
            cta_action: "/signup".to_string(),
            launch_mode: "active".to_string(),
            pre_order_enabled: false,
            pre_order_price_cents: None,
            pre_order_currency: "usd".to_string(),
            pre_order_available: None,
            waitlist_count: product_waitlist_count,
            lead_count: 0,
            page_id: Some(pg.id),
            variant_id: ab_variant_id,
            serve_source: ServeSource::Cms,
            plans,
        };
        let mut resp = (StatusCode::OK, Json(page)).into_response();
        *resp.headers_mut() = cdn_headers();
        if should_set_cookie {
            if let Some(id) = ab_variant_id {
                apply_ab_cookie(&mut resp, id);
            }
        }
        return resp;
    }

    // ── Fallback: product_page_templates + product_page_variants ──────────────
    let (product, tmpl) = match load_product_and_template(&db, &slug).await {
        Ok(r) => r,
        Err(e) => return e,
    };

    let v = match variant::Entity::find()
        .filter(variant::Column::ProductId.eq(product.id))
        .filter(variant::Column::VariantSlug.eq(&variant_slug))
        .filter(variant::Column::IsPublished.eq(true))
        .one(&db)
        .await
    {
        Ok(Some(v)) => v,
        Ok(None) => {
            return (StatusCode::NOT_FOUND, "variant not found or not published").into_response();
        }
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    };

    let hreflang = load_hreflang(&db, product.id, &slug).await;
    let plans = load_public_plans(&db, product.id).await;
    let base = base_url();

    // Merge: variant overrides template
    let hero = merge_json(&tmpl.hero_payload, &v.hero_overrides);
    // Block overrides are applied at render time by the frontend (keyed by block_id)
    let blocks = tmpl.blocks_payload.clone();

    let meta_title = v
        .meta_title
        .clone()
        .or_else(|| tmpl.meta_title.clone())
        .unwrap_or_else(|| {
            format!(
                "{} — {} {}",
                product.name,
                v.city.as_deref().unwrap_or(""),
                v.region.as_deref().unwrap_or(""),
            )
            .trim()
            .to_string()
        });

    let meta_description = v
        .meta_description
        .clone()
        .or_else(|| tmpl.meta_description.clone())
        .unwrap_or_default();

    let canonical = v
        .canonical_url
        .clone()
        .unwrap_or_else(|| format!("{base}/products/{slug}/{variant_slug}"));

    let structured_data = v
        .structured_data
        .clone()
        .unwrap_or_else(|| build_local_business_jsonld(&product, &v));

    // Pre-order availability — variant cap takes priority over product cap
    let pre_order_available = v
        .pre_order_cap
        .map(|cap| cap - v.pre_order_sold)
        .or_else(|| {
            product
                .pre_order_cap
                .map(|cap| cap - product.pre_order_sold)
        });

    let page = ProductPageResponse {
        product_id: product.id,
        product_slug: product.slug.clone(),
        product_name: product.name.clone(),
        variant_slug: Some(v.variant_slug.clone()),
        locale: v.locale.clone(),
        city: v.city.clone(),
        country_code: v.country_code.clone(),
        hero,
        blocks,
        meta_title,
        meta_description,
        og_image_url: v.og_image_url.clone().or(tmpl.og_image_url.clone()),
        canonical_url: canonical,
        structured_data,
        hreflang,
        cta_label: v.cta_label.clone().unwrap_or(tmpl.cta_label.clone()),
        cta_action: v.cta_action.clone().unwrap_or(tmpl.cta_action.clone()),
        launch_mode: v.launch_mode.clone(),
        pre_order_enabled: product.pre_order_enabled,
        pre_order_price_cents: product.pre_order_price_cents,
        pre_order_currency: product.pre_order_currency.clone(),
        pre_order_available,
        waitlist_count: product.waitlist_count,
        lead_count: v.lead_count,
        page_id: None,
        variant_id: None,
        serve_source: ServeSource::ProductTemplate,
        plans,
    };

    let mut resp = (StatusCode::OK, Json(page)).into_response();
    *resp.headers_mut() = cdn_headers();
    resp
}

async fn get_product_sitemap(
    Extension(db): Extension<DatabaseConnection>,
    Path(slug): Path<String>,
) -> impl IntoResponse {
    let product = match platform_product::Entity::find()
        .filter(platform_product::Column::Slug.eq(&slug))
        .one(&db)
        .await
    {
        Ok(Some(p)) => p,
        Ok(None) => return (StatusCode::NOT_FOUND, "").into_response(),
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    };

    let variants = variant::Entity::find()
        .filter(variant::Column::ProductId.eq(product.id))
        .filter(variant::Column::IsPublished.eq(true))
        .all(&db)
        .await
        .unwrap_or_default();

    let base = base_url();
    let mut urls = format!(
        "  <url><loc>{base}/products/{slug}</loc><changefreq>weekly</changefreq><priority>1.0</priority></url>\n"
    );

    for v in &variants {
        urls.push_str(&format!(
            "  <url><loc>{base}/products/{slug}/{}</loc><changefreq>weekly</changefreq><priority>0.8</priority></url>\n",
            v.variant_slug
        ));
    }

    let xml = format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<urlset xmlns="http://www.sitemaps.org/schemas/sitemap/0.9"
        xmlns:xhtml="http://www.w3.org/1999/xhtml">
{urls}</urlset>"#
    );

    let mut headers = HeaderMap::new();
    headers.insert(
        header::CONTENT_TYPE,
        "application/xml; charset=utf-8".parse().unwrap(),
    );
    headers.insert(
        header::CACHE_CONTROL,
        "public, s-maxage=3600".parse().unwrap(),
    );

    (StatusCode::OK, headers, xml).into_response()
}

async fn join_waitlist_product(
    Extension(db): Extension<DatabaseConnection>,
    Path(slug): Path<String>,
    Json(body): Json<WaitlistBody>,
) -> impl IntoResponse {
    join_waitlist_inner(&db, &slug, None, body).await
}

async fn join_waitlist_variant(
    Extension(db): Extension<DatabaseConnection>,
    Path((slug, variant_slug)): Path<(String, String)>,
    Json(body): Json<WaitlistBody>,
) -> impl IntoResponse {
    join_waitlist_inner(&db, &slug, Some(&variant_slug.clone()), body).await
}

async fn join_beta_application(
    Extension(db): Extension<DatabaseConnection>,
    Json(body): Json<WaitlistBody>,
) -> impl IntoResponse {
    join_waitlist_inner(
        &db,
        crate::types::gtm::FolioMarketingSlug::FolioBeta.as_str(),
        None,
        body,
    )
    .await
}

async fn join_waitlist_inner(
    db: &DatabaseConnection,
    product_slug: &str,
    variant_slug: Option<&str>,
    body: WaitlistBody,
) -> axum::response::Response {
    if body.email.is_empty() || !body.email.contains('@') {
        return (StatusCode::UNPROCESSABLE_ENTITY, "valid email required").into_response();
    }

    let product = match platform_product::Entity::find()
        .filter(platform_product::Column::Slug.eq(product_slug))
        .one(db)
        .await
    {
        Ok(Some(p)) => p,
        Ok(None) => return (StatusCode::NOT_FOUND, "product not found").into_response(),
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    };

    // Build source_metadata
    let variant_info = if let Some(vs) = variant_slug {
        variant::Entity::find()
            .filter(variant::Column::ProductId.eq(product.id))
            .filter(variant::Column::VariantSlug.eq(vs))
            .one(db)
            .await
            .ok()
            .flatten()
    } else {
        None
    };

    let source_meta = json!({
        "product_slug": product_slug,
        "variant_slug": variant_slug,
        "city": variant_info.as_ref().and_then(|v| v.city.as_ref()),
        "country_code": variant_info.as_ref().and_then(|v| v.country_code.as_ref()),
        "locale": variant_info.as_ref().map(|v| &v.locale),
    });

    // TODO: Upsert into atlas_lead (G-31) via LeadService
    // Dedup: 1 lead per (email + product_id); append source_meta to JSONB array on dup
    // For now: log and return success (LeadService wiring is a follow-up task)
    tracing::info!(
        email = %body.email,
        product_slug = product_slug,
        ?variant_slug,
        source_meta = %source_meta,
        "waitlist lead captured"
    );

    let lead_source = body
        .source
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
        .unwrap_or_else(|| format!("waitlist:{}", product_slug));

    let referred_by = body
        .referred_by
        .as_deref()
        .map(str::trim)
        .filter(|v| !v.is_empty())
        .map(str::to_string)
        .or_else(|| {
            body.utm_content
                .as_deref()
                .map(str::trim)
                .filter(|v| !v.is_empty())
                .map(str::to_string)
        });
    let utm_content = body.utm_content.clone().or_else(|| referred_by.clone());

    // Increment variant lead_count if applicable
    if let Some(v) = variant_info {
        let mut active: variant::ActiveModel = v.into();
        active.lead_count = Set(active.lead_count.unwrap() + 1);
        let _ = active.update(db).await;
    }

    // Upsert into atlas_lead (G-31)
    // Dedup: skip insert if a lead with the same email already exists for this product
    let existing_lead = atlas_lead::Entity::find()
        .filter(atlas_lead::Column::Email.eq(&body.email))
        .filter(atlas_lead::Column::Source.eq(&lead_source))
        .one(db)
        .await
        .ok()
        .flatten();

    let mut inserted_lead_id: Option<Uuid> = existing_lead.as_ref().map(|l| l.id);

    if existing_lead.is_none() {
        // Compute display name per entity rule: first+last ?? company ?? email
        let composed_name = [body.first_name.as_deref(), body.last_name.as_deref()]
            .into_iter()
            .flatten()
            .map(str::trim)
            .filter(|part| !part.is_empty())
            .collect::<Vec<_>>()
            .join(" ");
        let display_name = body
            .name
            .as_deref()
            .map(str::trim)
            .filter(|name| !name.is_empty())
            .map(str::to_string)
            .or_else(|| (!composed_name.is_empty()).then_some(composed_name))
            .or_else(|| body.company.clone())
            .or_else(|| body.biz_name.clone())
            .unwrap_or_else(|| body.email.clone());
        // Capture sentinel_tenant_id before product is moved into the ActiveModel
        let sentinel_tenant_id = product.sentinel_tenant_id.unwrap_or_else(Uuid::nil);

        let new_lead = atlas_lead::ActiveModel {
            id: Set(Uuid::new_v4()),
            tenant_id: Set(sentinel_tenant_id),
            name: Set(display_name.clone()),
            email: Set(Some(body.email.clone())),
            phone: Set(body.phone.clone()),
            company: Set(body.company.clone()),
            source: Set(Some(lead_source.clone())),
            lead_status: Set("new".to_string()),
            email_verified: Set(false),
            phone_verified: Set(false),
            // Attribution + signup context stored in lead_metadata JSONB
            lead_metadata: Set(Some(json!({
                "role":          body.role,
                "portfolio_size": body.portfolio_size_label,
                "variant_slug":  variant_slug,
                "utm_source":    body.utm_source,
                "utm_medium":    body.utm_medium,
                "utm_campaign":  body.utm_campaign,
                "utm_content":   utm_content,
                "utm_term":      body.utm_term,
                "gclid":         body.gclid,
                "fbclid":        body.fbclid,
                "msclkid":       body.msclkid,
                "referrer":      body.referrer,
                "landing_url":   body.landing_url,
                "plan":          body.plan,
                "unit_count":    body.unit_count,
                "first_name":    body.first_name,
                "last_name":     body.last_name,
                "trade":         body.trade,
                "biz_name":      body.biz_name,
                "service_area":  body.service_area,
                "current_tool":  body.current_tool,
                "pain_point":    body.pain_point,
                "is_active":     body.is_active,
                "feedback_call": body.feedback_call,
                "why_beta":      body.why_beta,
                "source_hint":   body.source,
                "referred_by":   referred_by.clone(),
                "captured_at":   Utc::now().to_rfc3339(),
            }))),
            created_at: Set(Utc::now()),
            updated_at: Set(Utc::now()),
            ..Default::default()
        };

        match new_lead.insert(db).await {
            Ok(lead) => {
                inserted_lead_id = Some(lead.id);
                // Atomically increment platform_product.waitlist_count
                let mut prod_active: platform_product::ActiveModel = product.into();
                prod_active.waitlist_count = Set(prod_active.waitlist_count.unwrap() + 1);
                let _ = prod_active.update(db).await;

                // Enqueue confirmation email via outbox (fire-and-forget — non-fatal)
                let confirmation_job = outbox_job::ActiveModel {
                    id: Set(Uuid::new_v4()),
                    tenant_id: Set(sentinel_tenant_id),
                    job_type: Set(
                        crate::types::outbox::OutboxJobType::SendWaitlistConfirmation.to_string(),
                    ),
                    payload: Set(json!({
                        "to_email":     body.email,
                        "name":         display_name,
                        "product_slug": product_slug,
                        "variant_slug": variant_slug,
                    })),
                    status: Set(crate::types::outbox::OutboxJobStatus::Pending.to_string()),
                    run_at: Set(Utc::now()),
                    attempts: Set(0),
                    created_at: Set(Utc::now()),
                    ..Default::default()
                };
                if let Err(e) = confirmation_job.insert(db).await {
                    tracing::warn!(email = %body.email, error = %e, "outbox job insert failed — email will not be sent");
                }
            }
            Err(e) => {
                tracing::warn!(email = %body.email, error = %e, "atlas_lead insert failed");
            }
        }
    } else {
        tracing::debug!(email = %body.email, "duplicate waitlist signup — skipping lead insert");
    }

    // Auto-enroll into a matching active campaign when utm_campaign is set.
    if let Some(utm_campaign) = body
        .utm_campaign
        .as_deref()
        .map(str::trim)
        .filter(|v| !v.is_empty())
    {
        if let Err(e) = maybe_enroll_waitlist_in_campaign(
            db,
            utm_campaign,
            &body.email,
            body.name.as_deref().or(body.biz_name.as_deref()),
            inserted_lead_id,
            referred_by.as_deref(),
            &lead_source,
        )
        .await
        {
            tracing::warn!(
                email = %body.email,
                utm_campaign,
                error = %e,
                "waitlist campaign auto-enroll failed"
            );
        }
    }

    // Return 201 with position (waitlist_count reflects all leads for this product)
    let position = platform_product::Entity::find()
        .filter(platform_product::Column::Slug.eq(product_slug))
        .one(db)
        .await
        .ok()
        .flatten()
        .map(|p| p.waitlist_count)
        .unwrap_or(0);

    (
        StatusCode::CREATED,
        Json(json!({
            "message": "You're on the list! We'll be in touch.",
            "product": product_slug,
            "market":  variant_slug,
            "position": position,
            "status":  "waiting",
        })),
    )
        .into_response()
}

/// When a waitlist signup carries `utm_campaign`, enroll the contact into the
/// matching active `atlas_campaigns` row (Friends & Family, etc.).
async fn maybe_enroll_waitlist_in_campaign(
    db: &DatabaseConnection,
    utm_campaign: &str,
    email: &str,
    name: Option<&str>,
    lead_id: Option<Uuid>,
    referred_by: Option<&str>,
    lead_source: &str,
) -> Result<(), String> {
    let campaign = atlas_campaign::Entity::find()
        .filter(atlas_campaign::Column::UtmCampaign.eq(utm_campaign))
        .filter(atlas_campaign::Column::Status.eq("active"))
        .one(db)
        .await
        .map_err(|e| e.to_string())?;

    let Some(campaign) = campaign else {
        return Ok(());
    };

    if CampaignService::find_enrollment_by_email(db, campaign.id, email)
        .await
        .map_err(|e| e.to_string())?
        .is_some()
    {
        return Ok(());
    }

    let enrollment = CampaignService::enroll(
        db,
        campaign.tenant_id,
        EnrollContactPayload {
            campaign_id: campaign.id,
            contact_user_id: None,
            contact_email: Some(email.to_string()),
            contact_name: name.map(str::to_string).filter(|s| !s.is_empty()),
            contact_metadata: Some(json!({
                "lead_id": lead_id,
                "source": lead_source,
                "referred_by": referred_by,
                "utm_campaign": utm_campaign,
            })),
            external_enrollment_id: None,
            next_step_at: None,
        },
    )
    .await
    .map_err(|e| e.to_string())?;

    let _ = CampaignService::record_event(
        db,
        campaign.tenant_id,
        RecordEventPayload {
            enrollment_id: enrollment.id,
            event_type: CampaignEventType::FormFill,
            channel: CampaignChannel::Referral,
            sequence_step_id: None,
            link_clicked: None,
            ip_address: None,
            user_agent: None,
            metadata: Some(json!({
                "referred_by": referred_by,
                "utm_campaign": utm_campaign,
            })),
            conversion_entity_type: None,
            conversion_entity_id: None,
        },
    )
    .await;

    Ok(())
}

async fn create_pre_order(
    Extension(db): Extension<DatabaseConnection>,
    Path(slug): Path<String>,
    Json(body): Json<PreOrderBody>,
) -> impl IntoResponse {
    let product = match platform_product::Entity::find()
        .filter(platform_product::Column::Slug.eq(&slug))
        .one(&db)
        .await
    {
        Ok(Some(p)) => p,
        Ok(None) => return (StatusCode::NOT_FOUND, "product not found").into_response(),
        Err(_) => return StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    };

    if !product.pre_order_enabled {
        return (
            StatusCode::UNPROCESSABLE_ENTITY,
            "pre-order is not enabled for this product",
        )
            .into_response();
    }

    let stripe_price_id = match &product.stripe_price_id {
        Some(id) if !id.is_empty() => id.clone(),
        _ => {
            return (
                StatusCode::UNPROCESSABLE_ENTITY,
                "stripe_price_id not configured",
            )
                .into_response();
        }
    };

    // Check cap
    if let Some(cap) = product.pre_order_cap {
        if product.pre_order_sold >= cap {
            return (
                StatusCode::CONFLICT,
                "Pre-order capacity is sold out. Join the waitlist instead.",
            )
                .into_response();
        }
    }

    // Create Stripe Checkout Session
    let stripe_secret = std::env::var("STRIPE_SECRET_KEY")
        .or_else(|_| std::env::var("STRIPE_PLATFORM_SECRET_KEY"))
        .unwrap_or_default();

    if stripe_secret.is_empty() {
        tracing::error!("STRIPE_SECRET_KEY not set — cannot create checkout session");
        return (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(json!({
                "error": "Payment processing is not configured on this server.",
                "stripe_price_id": stripe_price_id,
            })),
        )
            .into_response();
    }

    let client = stripe::Client::new(stripe_secret);

    let mut create_session = stripe::CreateCheckoutSession::new();
    create_session.mode = Some(stripe::CheckoutSessionMode::Payment);
    create_session.customer_email = Some(&body.email);
    create_session.success_url = Some(&body.success_url);
    create_session.cancel_url = Some(&body.cancel_url);

    let line_item = stripe::CreateCheckoutSessionLineItems {
        price: Some(stripe_price_id.clone()),
        quantity: Some(1),
        ..Default::default()
    };
    create_session.line_items = Some(vec![line_item]);

    match stripe::CheckoutSession::create(&client, create_session).await {
        Ok(session) => {
            // Increment pre_order_sold count
            let mut prod_active: platform_product::ActiveModel = product.into();
            prod_active.pre_order_sold = Set(prod_active.pre_order_sold.unwrap() + 1);
            let _ = prod_active.update(&db).await;

            tracing::info!(
                email = %body.email,
                product_slug = slug,
                session_id = ?session.id,
                "Stripe checkout session created"
            );

            (
                StatusCode::OK,
                Json(json!({
                    "checkout_url": session.url,
                    "session_id": session.id,
                    "product_slug": slug,
                })),
            )
                .into_response()
        }
        Err(e) => {
            tracing::error!(error = %e, "Stripe CheckoutSession::create failed");
            (
                StatusCode::BAD_GATEWAY,
                Json(json!({
                    "error": "Failed to create Stripe checkout session.",
                    "details": e.to_string(),
                })),
            )
                .into_response()
        }
    }
}

async fn record_view(
    Extension(db): Extension<DatabaseConnection>,
    Path((slug, variant_slug)): Path<(String, String)>,
) -> impl IntoResponse {
    // Fire-and-forget view count increment — ignore errors silently
    if let Ok(Some(product)) = platform_product::Entity::find()
        .filter(platform_product::Column::Slug.eq(&slug))
        .one(&db)
        .await
    {
        if let Ok(Some(v)) = variant::Entity::find()
            .filter(variant::Column::ProductId.eq(product.id))
            .filter(variant::Column::VariantSlug.eq(&variant_slug))
            .one(&db)
            .await
        {
            let mut active: variant::ActiveModel = v.into();
            active.view_count = Set(active.view_count.unwrap() + 1);
            let _ = active.update(&db).await;
        }
    }

    StatusCode::NO_CONTENT.into_response()
}
