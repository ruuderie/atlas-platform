//! Admin — Product Page Variants handler
//!
//! Full CRUD for product_page_variants plus the bulk-generate power feature
//! and AI localization trigger. All routes require platform super-admin auth.
//!
//! # Routes
//!
//! ```ignore
//! GET  /api/admin/platform/products/{id}/template
//! POST /api/admin/platform/products/{id}/template
//!
//! GET  /api/admin/platform/products/{id}/variants
//! POST /api/admin/platform/products/{id}/variants          (single)
//! POST /api/admin/platform/products/{id}/variants/bulk-generate
//!
//! PATCH  /api/admin/platform/products/{id}/variants/{vid}
//! POST   /api/admin/platform/products/{id}/variants/{vid}/publish
//! POST   /api/admin/platform/products/{id}/variants/{vid}/localize
//! DELETE /api/admin/platform/products/{id}/variants/{vid}
//!
//! POST   /api/admin/platform/products/{id}/variants/bulk-publish
//! POST   /api/admin/platform/products/{id}/variants/bulk-localize
//!
//! GET    /api/admin/platform/products/{id}/waitlist
//! GET    /api/admin/platform/products/{id}/waitlist/export  (CSV)
//! ```

use axum::{
    body::Body,
    extract::{Extension, Json, Path, Query},
    http::{HeaderMap, StatusCode, header},
    response::IntoResponse,
    routing::{delete, get, patch, post},
    Router,
};
use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter,
    QueryOrder,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use uuid::Uuid;

use crate::{
    entities::{
        platform_product,
        product_page::{template, variant},
    },
    services::product_localization::ProductLocalizationService,
};

// ── Route registration ────────────────────────────────────────────────────────
//
// Returns a state-free router (Router<DatabaseConnection>) so it can be merged
// before the outer .with_state(db) call in admin::routes::admin_routes().
// NEVER call .with_state() here — state is applied exactly once at the boundary.

pub fn routes_raw() -> Router<DatabaseConnection> {
    Router::new()
        // Template
        .route(
            "/api/admin/platform/products/{id}/template",
            get(get_template).post(upsert_template),
        )
        // Variant list + create
        .route(
            "/api/admin/platform/products/{id}/variants",
            get(list_variants).post(create_variant),
        )
        // Bulk operations
        .route(
            "/api/admin/platform/products/{id}/variants/bulk-generate",
            post(bulk_generate_variants),
        )
        .route(
            "/api/admin/platform/products/{id}/variants/bulk-publish",
            post(bulk_publish_variants),
        )
        .route(
            "/api/admin/platform/products/{id}/variants/bulk-localize",
            post(bulk_localize_variants),
        )
        // Single variant operations
        .route(
            "/api/admin/platform/products/{id}/variants/{vid}",
            patch(update_variant).delete(delete_variant),
        )
        .route(
            "/api/admin/platform/products/{id}/variants/{vid}/publish",
            post(publish_variant),
        )
        .route(
            "/api/admin/platform/products/{id}/variants/{vid}/localize",
            post(localize_variant),
        )
        // Waitlist analytics
        .route(
            "/api/admin/platform/products/{id}/waitlist",
            get(get_waitlist_analytics),
        )
        .route(
            "/api/admin/platform/products/{id}/waitlist/export",
            get(export_waitlist_csv),
        )
}

// Keep a convenience wrapper for standalone use (e.g. tests)
pub fn routes(db: DatabaseConnection) -> Router {
    routes_raw().with_state(db)
}

// ── Input types ───────────────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct UpsertTemplateBody {
    pub hero_payload:      Option<Value>,
    pub blocks_payload:    Option<Value>,
    pub meta_title:        Option<String>,
    pub meta_description:  Option<String>,
    pub og_image_url:      Option<String>,
    pub structured_data:   Option<Value>,
    pub cta_label:         Option<String>,
    pub cta_action:        Option<String>,
}

/// Single variant creation input
#[derive(Debug, Deserialize)]
pub struct CreateVariantBody {
    pub variant_slug:     String,
    pub locale:           String,
    pub country_code:     Option<String>,
    pub region:           Option<String>,
    pub city:             Option<String>,
    pub geo_lat:          Option<f64>,
    pub geo_lng:          Option<f64>,
    pub launch_mode:      Option<String>,
    pub copy_strategy:    Option<String>,   // "manual" | "city_inject" | "ai_localize"
    pub subdomain_override: Option<String>,
    pub pre_order_cap:    Option<i32>,
    // SEO overrides (optional — if copy_strategy=city_inject, auto-generated)
    pub meta_title:       Option<String>,
    pub meta_description: Option<String>,
}

/// Bulk generate input — list of markets
#[derive(Debug, Deserialize, Clone)]
pub struct MarketSpec {
    pub slug:         String,
    pub locale:       String,
    pub city:         Option<String>,
    pub region:       Option<String>,
    pub country_code: Option<String>,
    pub geo_lat:      Option<f64>,
    pub geo_lng:      Option<f64>,
    pub subdomain_override: Option<String>,
    pub pre_order_cap: Option<i32>,
}

#[derive(Debug, Deserialize)]
pub struct BulkGenerateBody {
    pub markets:       Vec<MarketSpec>,
    pub launch_mode:   Option<String>,
    /// "manual" | "city_inject" | "ai_localize"
    pub copy_strategy: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateVariantBody {
    pub hero_overrides:   Option<Value>,
    pub block_overrides:  Option<Value>,
    pub meta_title:       Option<String>,
    pub meta_description: Option<String>,
    pub og_image_url:     Option<String>,
    pub canonical_url:    Option<String>,
    pub structured_data:  Option<Value>,
    pub launch_mode:      Option<String>,
    pub is_published:     Option<bool>,
    pub cta_label:        Option<String>,
    pub cta_action:       Option<String>,
    pub subdomain_override: Option<String>,
    pub pre_order_cap:    Option<i32>,
}

#[derive(Debug, Deserialize)]
pub struct PublishVariantBody {
    pub launch_mode: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct BulkPublishFilter {
    pub country_code: Option<String>,
    pub locale:       Option<String>,
    pub launch_mode:  Option<String>,
}

// ── Template handlers ─────────────────────────────────────────────────────────

async fn get_template(
    Extension(db): Extension<DatabaseConnection>,
    Path(product_id): Path<Uuid>,
) -> impl IntoResponse {
    match template::Entity::find()
        .filter(template::Column::ProductId.eq(product_id))
        .one(&db)
        .await
    {
        Ok(Some(t)) => (StatusCode::OK, Json(t)).into_response(),
        Ok(None) => (StatusCode::NOT_FOUND, "no template for this product").into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}

async fn upsert_template(
    Extension(db): Extension<DatabaseConnection>,
    Path(product_id): Path<Uuid>,
    Json(body): Json<UpsertTemplateBody>,
) -> impl IntoResponse {
    // Upsert — if template exists update it, else create
    let existing = template::Entity::find()
        .filter(template::Column::ProductId.eq(product_id))
        .one(&db)
        .await;

    match existing {
        Ok(Some(t)) => {
            let mut active: template::ActiveModel = t.into();
            if let Some(h) = body.hero_payload       { active.hero_payload = Set(h); }
            if let Some(b) = body.blocks_payload     { active.blocks_payload = Set(b); }
            if let Some(v) = body.meta_title         { active.meta_title = Set(Some(v)); }
            if let Some(v) = body.meta_description   { active.meta_description = Set(Some(v)); }
            if let Some(v) = body.og_image_url       { active.og_image_url = Set(Some(v)); }
            if let Some(v) = body.structured_data    { active.structured_data = Set(v); }
            if let Some(v) = body.cta_label          { active.cta_label = Set(v); }
            if let Some(v) = body.cta_action         { active.cta_action = Set(v); }
            match active.update(&db).await {
                Ok(t) => (StatusCode::OK, Json(t)).into_response(),
                Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
            }
        }
        Ok(None) => {
            let new = template::ActiveModel {
                id:             Set(Uuid::new_v4()),
                product_id:     Set(product_id),
                hero_payload:   Set(body.hero_payload.unwrap_or_else(|| json!({}))),
                blocks_payload: Set(body.blocks_payload.unwrap_or_else(|| json!([]))),
                meta_title:     Set(body.meta_title),
                meta_description: Set(body.meta_description),
                og_image_url:   Set(body.og_image_url),
                structured_data: Set(body.structured_data.unwrap_or_else(|| json!({}))),
                cta_label:      Set(body.cta_label.unwrap_or_else(|| "Join the Waitlist".into())),
                cta_action:     Set(body.cta_action.unwrap_or_else(|| "waitlist".into())),
                ..Default::default()
            };
            match new.insert(&db).await {
                Ok(t) => (StatusCode::CREATED, Json(t)).into_response(),
                Err(e) => (StatusCode::UNPROCESSABLE_ENTITY, e.to_string()).into_response(),
            }
        }
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}

// ── Variant handlers ──────────────────────────────────────────────────────────

async fn list_variants(
    Extension(db): Extension<DatabaseConnection>,
    Path(product_id): Path<Uuid>,
) -> impl IntoResponse {
    match variant::Entity::find()
        .filter(variant::Column::ProductId.eq(product_id))
        .order_by_asc(variant::Column::CreatedAt)
        .all(&db)
        .await
    {
        Ok(vs) => (StatusCode::OK, Json(vs)).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}

async fn create_variant(
    Extension(db): Extension<DatabaseConnection>,
    Path(product_id): Path<Uuid>,
    Json(body): Json<CreateVariantBody>,
) -> impl IntoResponse {
    let tmpl = match template::Entity::find()
        .filter(template::Column::ProductId.eq(product_id))
        .one(&db)
        .await
    {
        Ok(Some(t)) => t,
        Ok(None) => return (StatusCode::UNPROCESSABLE_ENTITY, "create a template first").into_response(),
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    };

    let copy_strategy = body.copy_strategy.as_deref().unwrap_or("manual").to_string();
    let (meta_title, meta_description) = if copy_strategy == "city_inject" {
        city_inject_seo(&body)
    } else {
        (body.meta_title, body.meta_description)
    };

    let new_variant = variant::ActiveModel {
        id:               Set(Uuid::new_v4()),
        product_id:       Set(product_id),
        template_id:      Set(tmpl.id),
        variant_slug:     Set(body.variant_slug),
        locale:           Set(body.locale),
        country_code:     Set(body.country_code),
        region:           Set(body.region),
        city:             Set(body.city),
        geo_lat:          Set(body.geo_lat),
        geo_lng:          Set(body.geo_lng),
        hero_overrides:   Set(json!({})),
        block_overrides:  Set(json!({})),
        meta_title:       Set(meta_title),
        meta_description: Set(meta_description),
        launch_mode:      Set(body.launch_mode.unwrap_or_else(|| "draft".into())),
        is_published:     Set(false),
        copy_strategy:    Set(copy_strategy.clone()),
        localization_status: Set("not_started".to_string()),
        subdomain_override: Set(body.subdomain_override),
        pre_order_cap:    Set(body.pre_order_cap),
        ..Default::default()
    };

    match new_variant.insert(&db).await {
        Ok(v) => {
            // Auto-enqueue AI localization if strategy is ai_localize
            if copy_strategy == "ai_localize" {
                let db2 = db.clone();
                let vid = v.id;
                tokio::spawn(async move {
                    if let Err(e) = ProductLocalizationService::enqueue_variant_localization(&db2, vid).await {
                        tracing::warn!(variant_id = %vid, error = %e, "auto-localize enqueue failed");
                    }
                });
            }
            (StatusCode::CREATED, Json(v)).into_response()
        }
        Err(e) if e.to_string().contains("unique") => {
            (StatusCode::CONFLICT, "variant_slug already exists for this product").into_response()
        }
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}

async fn bulk_generate_variants(
    Extension(db): Extension<DatabaseConnection>,
    Path(product_id): Path<Uuid>,
    Json(body): Json<BulkGenerateBody>,
) -> impl IntoResponse {
    let tmpl = match template::Entity::find()
        .filter(template::Column::ProductId.eq(product_id))
        .one(&db)
        .await
    {
        Ok(Some(t)) => t,
        Ok(None) => return (StatusCode::UNPROCESSABLE_ENTITY, "create a template first").into_response(),
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    };

    let launch_mode = body.launch_mode.as_deref().unwrap_or("draft").to_string();
    let copy_strategy = body.copy_strategy.as_deref().unwrap_or("city_inject").to_string();

    let mut created = 0usize;
    let mut skipped = 0usize;
    let mut localize_queue: Vec<Uuid> = Vec::new();

    for market in &body.markets {
        let body_equiv = CreateVariantBody {
            variant_slug: market.slug.clone(),
            locale:       market.locale.clone(),
            country_code: market.country_code.clone(),
            region:       market.region.clone(),
            city:         market.city.clone(),
            geo_lat:      market.geo_lat,
            geo_lng:      market.geo_lng,
            launch_mode:  Some(launch_mode.clone()),
            copy_strategy: Some(copy_strategy.clone()),
            subdomain_override: market.subdomain_override.clone(),
            pre_order_cap: market.pre_order_cap,
            meta_title:   None,
            meta_description: None,
        };

        let (meta_title, meta_description) = if copy_strategy == "city_inject" {
            city_inject_seo(&body_equiv)
        } else {
            (None, None)
        };

        let new_variant = variant::ActiveModel {
            id:               Set(Uuid::new_v4()),
            product_id:       Set(product_id),
            template_id:      Set(tmpl.id),
            variant_slug:     Set(market.slug.clone()),
            locale:           Set(market.locale.clone()),
            country_code:     Set(market.country_code.clone()),
            region:           Set(market.region.clone()),
            city:             Set(market.city.clone()),
            geo_lat:          Set(market.geo_lat),
            geo_lng:          Set(market.geo_lng),
            hero_overrides:   Set(json!({})),
            block_overrides:  Set(json!({})),
            meta_title:       Set(meta_title),
            meta_description: Set(meta_description),
            launch_mode:      Set(launch_mode.clone()),
            is_published:     Set(false),
            copy_strategy:    Set(copy_strategy.clone()),
            localization_status: Set("not_started".to_string()),
            subdomain_override: Set(market.subdomain_override.clone()),
            pre_order_cap:    Set(market.pre_order_cap),
            ..Default::default()
        };

        match new_variant.insert(&db).await {
            Ok(v) => {
                created += 1;
                if copy_strategy == "ai_localize" {
                    localize_queue.push(v.id);
                }
            }
            Err(e) if e.to_string().contains("unique") => {
                skipped += 1;
            }
            Err(e) => {
                tracing::warn!(slug = %market.slug, error = %e, "bulk-generate: insert failed");
                skipped += 1;
            }
        }
    }

    // Enqueue AI localization in background
    if !localize_queue.is_empty() {
        let db2 = db.clone();
        let queue = localize_queue.clone();
        tokio::spawn(async move {
            for vid in queue {
                if let Err(e) = ProductLocalizationService::enqueue_variant_localization(&db2, vid).await {
                    tracing::warn!(variant_id = %vid, error = %e, "bulk localize enqueue failed");
                }
            }
        });
    }

    (
        StatusCode::CREATED,
        Json(json!({
            "created": created,
            "skipped": skipped,
            "localization_queued": localize_queue.len(),
            "copy_strategy": copy_strategy,
        })),
    )
        .into_response()
}

async fn update_variant(
    Extension(db): Extension<DatabaseConnection>,
    Path((_product_id, vid)): Path<(Uuid, Uuid)>,
    Json(body): Json<UpdateVariantBody>,
) -> impl IntoResponse {
    let v = match variant::Entity::find_by_id(vid).one(&db).await {
        Ok(Some(v)) => v,
        Ok(None) => return (StatusCode::NOT_FOUND, "variant not found").into_response(),
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    };

    let mut active: variant::ActiveModel = v.into();
    if let Some(v) = body.hero_overrides   { active.hero_overrides = Set(v); }
    if let Some(v) = body.block_overrides  { active.block_overrides = Set(v); }
    if let Some(v) = body.meta_title       { active.meta_title = Set(Some(v)); }
    if let Some(v) = body.meta_description { active.meta_description = Set(Some(v)); }
    if let Some(v) = body.og_image_url     { active.og_image_url = Set(Some(v)); }
    if let Some(v) = body.canonical_url    { active.canonical_url = Set(Some(v)); }
    if let Some(v) = body.structured_data  { active.structured_data = Set(Some(v)); }
    if let Some(v) = body.launch_mode      { active.launch_mode = Set(v); }
    if let Some(v) = body.is_published     { active.is_published = Set(v); }
    if let Some(v) = body.cta_label        { active.cta_label = Set(Some(v)); }
    if let Some(v) = body.cta_action       { active.cta_action = Set(Some(v)); }
    if let Some(v) = body.subdomain_override { active.subdomain_override = Set(Some(v)); }
    if let Some(v) = body.pre_order_cap    { active.pre_order_cap = Set(Some(v)); }

    match active.update(&db).await {
        Ok(v) => (StatusCode::OK, Json(v)).into_response(),
        Err(e) => (StatusCode::UNPROCESSABLE_ENTITY, e.to_string()).into_response(),
    }
}

async fn publish_variant(
    Extension(db): Extension<DatabaseConnection>,
    Path((_product_id, vid)): Path<(Uuid, Uuid)>,
    Json(body): Json<PublishVariantBody>,
) -> impl IntoResponse {
    let v = match variant::Entity::find_by_id(vid).one(&db).await {
        Ok(Some(v)) => v,
        Ok(None) => return (StatusCode::NOT_FOUND, "variant not found").into_response(),
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    };
    let mut active: variant::ActiveModel = v.into();
    active.is_published = Set(true);
    if let Some(mode) = body.launch_mode {
        active.launch_mode = Set(mode);
    }
    match active.update(&db).await {
        Ok(v) => (StatusCode::OK, Json(v)).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}

async fn localize_variant(
    Extension(db): Extension<DatabaseConnection>,
    Path((_product_id, vid)): Path<(Uuid, Uuid)>,
) -> impl IntoResponse {
    match ProductLocalizationService::enqueue_variant_localization(&db, vid).await {
        Ok(task_id) => (
            StatusCode::ACCEPTED,
            Json(json!({ "task_id": task_id, "status": "queued" })),
        )
            .into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e).into_response(),
    }
}

async fn delete_variant(
    Extension(db): Extension<DatabaseConnection>,
    Path((_product_id, vid)): Path<(Uuid, Uuid)>,
) -> impl IntoResponse {
    match variant::Entity::delete_by_id(vid).exec(&db).await {
        Ok(_) => StatusCode::NO_CONTENT.into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}

async fn bulk_publish_variants(
    Extension(db): Extension<DatabaseConnection>,
    Path(product_id): Path<Uuid>,
    Json(filter): Json<BulkPublishFilter>,
) -> impl IntoResponse {
    let mut q = variant::Entity::find()
        .filter(variant::Column::ProductId.eq(product_id))
        .filter(variant::Column::IsPublished.eq(false));

    if let Some(cc) = filter.country_code { q = q.filter(variant::Column::CountryCode.eq(cc)); }
    if let Some(loc) = filter.locale      { q = q.filter(variant::Column::Locale.eq(loc)); }

    let variants = match q.all(&db).await {
        Ok(vs) => vs,
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    };

    let mut published = 0usize;
    for v in variants {
        let mut active: variant::ActiveModel = v.into();
        active.is_published = Set(true);
        if let Some(ref mode) = filter.launch_mode {
            active.launch_mode = Set(mode.clone());
        }
        if active.update(&db).await.is_ok() { published += 1; }
    }

    (StatusCode::OK, Json(json!({ "published": published }))).into_response()
}

async fn bulk_localize_variants(
    Extension(db): Extension<DatabaseConnection>,
    Path(product_id): Path<Uuid>,
) -> impl IntoResponse {
    match ProductLocalizationService::enqueue_all_pending_for_product(&db, product_id).await {
        Ok(task_ids) => (
            StatusCode::ACCEPTED,
            Json(json!({ "queued": task_ids.len(), "task_ids": task_ids })),
        )
            .into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e).into_response(),
    }
}

// Waitlist analytics (TODO: full G-31 join in follow-up — counts from variant.lead_count for now)
async fn get_waitlist_analytics(
    Extension(db): Extension<DatabaseConnection>,
    Path(product_id): Path<Uuid>,
) -> impl IntoResponse {
    let product = match platform_product::Entity::find_by_id(product_id).one(&db).await {
        Ok(Some(p)) => p,
        Ok(None) => return (StatusCode::NOT_FOUND, "product not found").into_response(),
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    };

    let variants = variant::Entity::find()
        .filter(variant::Column::ProductId.eq(product_id))
        .order_by_desc(variant::Column::LeadCount)
        .all(&db)
        .await
        .unwrap_or_default();

    let total_leads: i32 = variants.iter().map(|v| v.lead_count).sum();
    let by_market: Vec<Value> = variants
        .iter()
        .map(|v| json!({
            "variant_id":   v.id,
            "variant_slug": v.variant_slug,
            "city":         v.city,
            "country_code": v.country_code,
            "locale":       v.locale,
            "launch_mode":  v.launch_mode,
            "is_published": v.is_published,
            "lead_count":   v.lead_count,
            "view_count":   v.view_count,
        }))
        .collect();

    (
        StatusCode::OK,
        Json(json!({
            "product_id":     product_id,
            "product_name":   product.name,
            "total_leads":    total_leads,
            "waitlist_count": product.waitlist_count,
            "variant_count":  variants.len(),
            "by_market":      by_market,
        })),
    )
        .into_response()
}

async fn export_waitlist_csv(
    Extension(db): Extension<DatabaseConnection>,
    Path(product_id): Path<Uuid>,
) -> impl IntoResponse {
    // TODO: full G-31 atlas_lead join for real email/name data
    // For now: return variant-level summary as CSV placeholder
    let variants = variant::Entity::find()
        .filter(variant::Column::ProductId.eq(product_id))
        .order_by_desc(variant::Column::LeadCount)
        .all(&db)
        .await
        .unwrap_or_default();

    let mut csv = "variant_slug,city,country_code,locale,launch_mode,lead_count,view_count\n".to_string();
    for v in &variants {
        csv.push_str(&format!(
            "{},{},{},{},{},{},{}\n",
            v.variant_slug,
            v.city.as_deref().unwrap_or(""),
            v.country_code.as_deref().unwrap_or(""),
            v.locale,
            v.launch_mode,
            v.lead_count,
            v.view_count,
        ));
    }

    let mut headers = HeaderMap::new();
    headers.insert(header::CONTENT_TYPE, "text/csv".parse().unwrap());
    headers.insert(
        header::CONTENT_DISPOSITION,
        format!("attachment; filename=\"waitlist-{product_id}.csv\"").parse().unwrap(),
    );

    (StatusCode::OK, headers, csv).into_response()
}

// ── City inject SEO helper ────────────────────────────────────────────────────

fn city_inject_seo(body: &CreateVariantBody) -> (Option<String>, Option<String>) {
    let city = body.city.as_deref().unwrap_or("");
    let region = body.region.as_deref().unwrap_or("");
    let location = if region.is_empty() { city.to_string() } else { format!("{city}, {region}") };

    let title = if !city.is_empty() {
        Some(format!("Folio — Property Management in {location}"))
    } else {
        None
    };

    let desc = if !city.is_empty() {
        Some(format!(
            "Folio helps landlords in {location} manage cross-border rentals, STRs, \
             and long-term leases from one platform. Join the waitlist."
        ))
    } else {
        None
    };

    (title, desc)
}
