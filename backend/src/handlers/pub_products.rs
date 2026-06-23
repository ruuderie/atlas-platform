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
//! GET  /api/pub/products/:slug                    Master landing (template rendered)
//! GET  /api/pub/products/:slug/:variant           Variant (template + overrides merged)
//! GET  /api/pub/products/:slug/sitemap.xml        Sitemap for all published variants
//! POST /api/pub/products/:slug/waitlist           Lead capture (product-level)
//! POST /api/pub/products/:slug/:variant/waitlist  Lead capture (variant/market-scoped)
//! POST /api/pub/products/:slug/pre-order          Stripe Checkout Session
//! POST /api/pub/products/:slug/:variant/view      Increment view_count (no PII)
//! ```

use axum::{
    extract::{Extension, Json, Path, Query},
    http::{HeaderMap, StatusCode, header},
    response::IntoResponse,
    routing::{get, post},
    Router,
};
use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, DatabaseConnection, EntityTrait,
    QueryFilter, QueryOrder,
};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use uuid::Uuid;
use chrono::Utc;

use crate::entities::{
    platform_product,
    product_page::{template, variant},
    atlas_lead,
};

// ── Route registration ────────────────────────────────────────────────────────

pub fn public_routes_raw() -> Router<DatabaseConnection> {
    Router::new()
        .route("/api/pub/products",                                 get(list_products))
        .route("/api/pub/products/{slug}",                          get(get_product_master))
        .route("/api/pub/products/{slug}/sitemap.xml",              get(get_product_sitemap))
        .route("/api/pub/products/{slug}/waitlist",                 post(join_waitlist_product))
        .route("/api/pub/products/{slug}/pre-order",                post(create_pre_order))
        .route("/api/pub/products/{slug}/{variant}",                get(get_product_variant))
        .route("/api/pub/products/{slug}/{variant}/waitlist",       post(join_waitlist_variant))
        .route("/api/pub/products/{slug}/{variant}/view",           post(record_view))
}

pub fn public_routes(db: DatabaseConnection) -> Router {
    public_routes_raw().with_state(db)
}


// ── Response types ────────────────────────────────────────────────────────────

/// Merged page response — template fields overridden by variant
#[derive(Debug, Serialize)]
pub struct ProductPageResponse {
    pub product_id:       Uuid,
    pub product_slug:     String,
    pub product_name:     String,
    pub variant_slug:     Option<String>,
    pub locale:           String,
    pub city:             Option<String>,
    pub country_code:     Option<String>,

    // Merged content
    pub hero:             Value,
    pub blocks:           Value,

    // Merged SEO (variant overrides template)
    pub meta_title:       String,
    pub meta_description: String,
    pub og_image_url:     Option<String>,
    pub canonical_url:    String,
    pub structured_data:  Value,

    // hreflang list (all published variants for this product)
    pub hreflang:         Vec<HreflangEntry>,

    // CTA
    pub cta_label:        String,
    pub cta_action:       String,
    pub launch_mode:      String,
    pub pre_order_enabled: bool,
    pub pre_order_price_cents: Option<i32>,
    pub pre_order_currency:    String,
    pub pre_order_available:   Option<i32>, // cap - sold (null = unlimited)

    pub waitlist_count:   i32,
    pub lead_count:       i32,
}

#[derive(Debug, Serialize)]
pub struct HreflangEntry {
    pub locale:   String,
    pub url:      String,
    pub is_default: bool,
}

// ── Input types ───────────────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct WaitlistBody {
    pub email:   String,
    pub name:    Option<String>,
    pub phone:   Option<String>,
    pub company: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct PreOrderBody {
    pub email:       String,
    pub name:        Option<String>,
    pub success_url: String,
    pub cancel_url:  String,
}

// ── Helpers ───────────────────────────────────────────────────────────────────

fn base_url() -> String {
    std::env::var("PUBLIC_BASE_URL")
        .unwrap_or_else(|_| "https://atlas.app".to_string())
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

fn cdn_headers() -> HeaderMap {
    let mut headers = HeaderMap::new();
    headers.insert(
        header::CACHE_CONTROL,
        "public, s-maxage=3600, stale-while-revalidate=86400".parse().unwrap(),
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
        .ok_or_else(|| (StatusCode::NOT_FOUND, "product template not configured").into_response())?;

    Ok((product, tmpl))
}

async fn load_hreflang(db: &DatabaseConnection, product_id: Uuid, product_slug: &str) -> Vec<HreflangEntry> {
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
            locale:     v.locale.clone(),
            url:        format!("{base}/products/{product_slug}/{}", v.variant_slug),
            is_default: false,
        })
        .collect();

    // x-default → master page
    entries.push(HreflangEntry {
        locale:     "x-default".to_string(),
        url:        format!("{base}/products/{product_slug}"),
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

async fn list_products(
    Extension(db): Extension<DatabaseConnection>,
) -> impl IntoResponse {
    let products = platform_product::Entity::find()
        .filter(
            platform_product::Column::Status.is_in(["active", "beta"])
        )
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
) -> impl IntoResponse {
    let (product, tmpl) = match load_product_and_template(&db, &slug).await {
        Ok(r) => r,
        Err(e) => return e,
    };

    let hreflang = load_hreflang(&db, product.id, &slug).await;
    let base = base_url();

    let meta_title = tmpl.meta_title.clone()
        .unwrap_or_else(|| format!("{} — {}", product.name, product.tagline.as_deref().unwrap_or("")));
    let meta_description = tmpl.meta_description.clone().unwrap_or_default();
    let canonical = format!("{base}/products/{slug}");

    let page = ProductPageResponse {
        product_id:    product.id,
        product_slug:  product.slug.clone(),
        product_name:  product.name.clone(),
        variant_slug:  None,
        locale:        "en".to_string(),
        city:          None,
        country_code:  None,
        hero:          tmpl.hero_payload.clone(),
        blocks:        tmpl.blocks_payload.clone(),
        meta_title,
        meta_description,
        og_image_url:  tmpl.og_image_url.clone(),
        canonical_url: canonical,
        structured_data: tmpl.structured_data.clone(),
        hreflang,
        cta_label:     tmpl.cta_label.clone(),
        cta_action:    tmpl.cta_action.clone(),
        launch_mode:   product.launch_mode.clone(),
        pre_order_enabled:     product.pre_order_enabled,
        pre_order_price_cents: product.pre_order_price_cents,
        pre_order_currency:    product.pre_order_currency.clone(),
        pre_order_available:   product.pre_order_cap.map(|cap| cap - product.pre_order_sold),
        waitlist_count: product.waitlist_count,
        lead_count:    0,
    };

    let mut resp = (StatusCode::OK, Json(page)).into_response();
    *resp.headers_mut() = cdn_headers();
    resp
}

async fn get_product_variant(
    Extension(db): Extension<DatabaseConnection>,
    Path((slug, variant_slug)): Path<(String, String)>,
) -> impl IntoResponse {
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
        Ok(None) => return (StatusCode::NOT_FOUND, "variant not found or not published").into_response(),
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    };

    let hreflang = load_hreflang(&db, product.id, &slug).await;
    let base = base_url();

    // Merge: variant overrides template
    let hero = merge_json(&tmpl.hero_payload, &v.hero_overrides);
    // Block overrides are applied at render time by the frontend (keyed by block_id)
    let blocks = tmpl.blocks_payload.clone();

    let meta_title = v.meta_title.clone()
        .or_else(|| tmpl.meta_title.clone())
        .unwrap_or_else(|| {
            format!(
                "{} — {} {}",
                product.name,
                v.city.as_deref().unwrap_or(""),
                v.region.as_deref().unwrap_or(""),
            ).trim().to_string()
        });

    let meta_description = v.meta_description.clone()
        .or_else(|| tmpl.meta_description.clone())
        .unwrap_or_default();

    let canonical = v.canonical_url.clone()
        .unwrap_or_else(|| format!("{base}/products/{slug}/{variant_slug}"));

    let structured_data = v.structured_data.clone()
        .unwrap_or_else(|| build_local_business_jsonld(&product, &v));

    // Pre-order availability — variant cap takes priority over product cap
    let pre_order_available = v.pre_order_cap
        .map(|cap| cap - v.pre_order_sold)
        .or_else(|| product.pre_order_cap.map(|cap| cap - product.pre_order_sold));

    let page = ProductPageResponse {
        product_id:    product.id,
        product_slug:  product.slug.clone(),
        product_name:  product.name.clone(),
        variant_slug:  Some(v.variant_slug.clone()),
        locale:        v.locale.clone(),
        city:          v.city.clone(),
        country_code:  v.country_code.clone(),
        hero,
        blocks,
        meta_title,
        meta_description,
        og_image_url:  v.og_image_url.clone().or(tmpl.og_image_url.clone()),
        canonical_url: canonical,
        structured_data,
        hreflang,
        cta_label:     v.cta_label.clone().unwrap_or(tmpl.cta_label.clone()),
        cta_action:    v.cta_action.clone().unwrap_or(tmpl.cta_action.clone()),
        launch_mode:   v.launch_mode.clone(),
        pre_order_enabled:     product.pre_order_enabled,
        pre_order_price_cents: product.pre_order_price_cents,
        pre_order_currency:    product.pre_order_currency.clone(),
        pre_order_available,
        waitlist_count: product.waitlist_count,
        lead_count:    v.lead_count,
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
    headers.insert(header::CONTENT_TYPE, "application/xml; charset=utf-8".parse().unwrap());
    headers.insert(header::CACHE_CONTROL, "public, s-maxage=3600".parse().unwrap());

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
        .filter(atlas_lead::Column::Source.eq(format!("waitlist:{}", product_slug)))
        .one(db)
        .await
        .ok()
        .flatten();

    if existing_lead.is_none() {
        // Compute display name per entity rule: first+last ?? company ?? email
        let display_name = body.name.clone()
            .or_else(|| body.company.clone())
            .unwrap_or_else(|| body.email.clone());

        let new_lead = atlas_lead::ActiveModel {
            id:          Set(Uuid::new_v4()),
            tenant_id:   Set(product.sentinel_tenant_id.unwrap_or_else(Uuid::nil)),
            name:        Set(display_name),
            email:       Set(Some(body.email.clone())),
            phone:       Set(body.phone.clone()),
            company:     Set(body.company.clone()),
            source:      Set(Some(format!("waitlist:{}", product_slug))),
            lead_status: Set("new".to_string()),
            email_verified: Set(false),
            phone_verified: Set(false),
            created_at:  Set(Utc::now()),
            updated_at:  Set(Utc::now()),
            ..Default::default()
        };

        if let Err(e) = new_lead.insert(db).await {
            // Non-fatal — waitlist count still increments; log and continue
            tracing::warn!(email = %body.email, error = %e, "atlas_lead insert failed");
        } else {
            // Atomically increment platform_product.waitlist_count
            let mut prod_active: platform_product::ActiveModel = product.into();
            prod_active.waitlist_count = Set(prod_active.waitlist_count.unwrap() + 1);
            let _ = prod_active.update(db).await;
        }
    } else {
        tracing::debug!(email = %body.email, "duplicate waitlist signup — skipping lead insert");
    }

    (
        StatusCode::CREATED,
        Json(json!({
            "message": "You're on the list! We'll be in touch.",
            "product": product_slug,
            "market": variant_slug,
        })),
    )
        .into_response()
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
        return (StatusCode::UNPROCESSABLE_ENTITY, "pre-order is not enabled for this product").into_response();
    }

    let stripe_price_id = match &product.stripe_price_id {
        Some(id) if !id.is_empty() => id.clone(),
        _ => return (StatusCode::UNPROCESSABLE_ENTITY, "stripe_price_id not configured").into_response(),
    };

    // Check cap
    if let Some(cap) = product.pre_order_cap {
        if product.pre_order_sold >= cap {
            return (
                StatusCode::CONFLICT,
                "Pre-order capacity is sold out. Join the waitlist instead.",
            ).into_response();
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
        ).into_response();
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
            ).into_response()
        }
        Err(e) => {
            tracing::error!(error = %e, "Stripe CheckoutSession::create failed");
            (
                StatusCode::BAD_GATEWAY,
                Json(json!({
                    "error": "Failed to create Stripe checkout session.",
                    "details": e.to_string(),
                })),
            ).into_response()
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
