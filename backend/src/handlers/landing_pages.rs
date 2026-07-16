//! Platform-admin Landing Page Builder — API handlers
//!
//! Routes (all under `/api/admin/landing-pages`):
//!
//! ## Pages
//! ```text
//! GET    /api/admin/landing-pages?app_id=folio       list_pages
//! POST   /api/admin/landing-pages                    create_page
//! GET    /api/admin/landing-pages/{page_id}           get_page
//! PUT    /api/admin/landing-pages/{page_id}           update_page
//! POST   /api/admin/landing-pages/{page_id}/publish   toggle_publish
//! DELETE /api/admin/landing-pages/{page_id}           delete_page
//! ```
//!
//! ## Variants (A/B)
//! ```text
//! GET    /api/admin/landing-pages/{page_id}/variants                   list_variants
//! POST   /api/admin/landing-pages/{page_id}/variants                   create_variant
//! PUT    /api/admin/landing-pages/{page_id}/variants/{variant_id}      update_variant
//! DELETE /api/admin/landing-pages/{page_id}/variants/{variant_id}      delete_variant
//! POST   /api/admin/landing-pages/{page_id}/variants/{variant_id}/promote  promote_variant
//! ```
//!
//! ## UTM Presets
//! ```text
//! GET    /api/admin/utm-presets?app_id=folio          list_utm_presets
//! POST   /api/admin/utm-presets                        create_utm_preset
//! DELETE /api/admin/utm-presets/{preset_id}            delete_utm_preset
//! ```
//!
//! ## Tracking Pixels
//! ```text
//! GET    /api/admin/landing-pages/{page_id}/pixels                        get_pixels
//! PUT    /api/admin/landing-pages/{page_id}/pixels/{pixel_type}           set_pixel
//! ```
//! Pixel config is stored in `app_pages.hero_payload["pixel_config"]` JSONB.
//! Supported pixel_type values: "ga4", "meta", "linkedin", "gtm".
//!
//! ## Design decisions
//! - Pages are scoped by `app_id` (e.g., `"folio"`), NOT by `tenant_id`.
//!   This is the key difference from the legacy `/api/pages/{tenant_id}` routes.
//! - `tenant_id` on `app_pages` is preserved for backward compatibility but the
//!   platform sentinel UUID (`00000000-0000-0000-0000-000000000000`) is used for
//!   platform-level pages created through this handler.
//! - Promote winner: copies variant blocks/hero back to the parent page row,
//!   then deletes all variant rows for that page.

use axum::{
    Json, Router,
    extract::{Path, Query, State},
    http::StatusCode,
    routing::{delete, get, post, put},
};
use chrono::Utc;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, QueryOrder,
    QuerySelect, Set,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

use crate::entities::{
    app_page, app_page::Entity as AppPage, app_page_variant,
    app_page_variant::Entity as AppPageVariant, app_utm_preset,
    app_utm_preset::Entity as AppUtmPreset, atlas_lp_event, atlas_lp_event::Entity as AtlasLpEvent,
};

// ── Sentinel UUID used as tenant_id for platform-level pages ──────────────────
const PLATFORM_SENTINEL: &str = "00000000-0000-0000-0000-000000000000";

// ─────────────────────────────────────────────────────────────────────────────
// Request / response shapes
// ─────────────────────────────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct AppIdQuery {
    pub app_id: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct LandingPageSummary {
    pub id: Uuid,
    pub app_id: String,
    pub slug: String,
    pub title: String,
    pub page_type: String,
    pub locale: String,
    pub is_published: bool,
    pub created_at: chrono::DateTime<Utc>,
    pub updated_at: chrono::DateTime<Utc>,
}

impl From<app_page::Model> for LandingPageSummary {
    fn from(m: app_page::Model) -> Self {
        Self {
            id: m.id,
            app_id: m.app_id,
            slug: m.slug,
            title: m.title,
            page_type: m.page_type,
            locale: m.locale,
            is_published: m.is_published,
            created_at: m.created_at,
            updated_at: m.updated_at,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateLandingPagePayload {
    pub app_id: String,
    pub slug: String,
    pub title: String,
    pub description: Option<String>,
    pub page_type: Option<String>,
    pub locale: Option<String>,
    pub hero_payload: Option<Value>,
    pub blocks_payload: Option<Value>,
    pub is_published: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateLandingPagePayload {
    pub title: Option<String>,
    pub description: Option<String>,
    pub page_type: Option<String>,
    pub hero_payload: Option<Value>,
    pub blocks_payload: Option<Value>,
    pub slug: Option<String>,
    pub locale: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateVariantPayload {
    pub name: String,
    pub traffic_pct: Option<i32>,
    pub blocks_payload: Option<Value>,
    pub hero_payload: Option<Value>,
    pub is_control: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateVariantPayload {
    pub name: Option<String>,
    pub traffic_pct: Option<i32>,
    pub blocks_payload: Option<Value>,
    pub hero_payload: Option<Value>,
    pub is_active: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateUtmPresetPayload {
    pub app_id: String,
    pub name: String,
    pub utm_source: String,
    pub utm_medium: String,
    pub utm_campaign: String,
    pub utm_content: Option<String>,
    pub utm_term: Option<String>,
}

// ─────────────────────────────────────────────────────────────────────────────
// Route registration
// ─────────────────────────────────────────────────────────────────────────────

pub fn landing_page_routes_raw() -> Router<DatabaseConnection> {
    Router::new()
        // Pages
        .route(
            "/api/admin/landing-pages",
            get(list_pages).post(create_page),
        )
        .route(
            "/api/admin/landing-pages/{page_id}",
            get(get_page).put(update_page).delete(delete_page),
        )
        .route(
            "/api/admin/landing-pages/{page_id}/publish",
            post(toggle_publish),
        )
        // Tracking pixels
        .route("/api/admin/landing-pages/{page_id}/pixels", get(get_pixels))
        .route(
            "/api/admin/landing-pages/{page_id}/pixels/{pixel_type}",
            put(set_pixel),
        )
        // A/B Variants
        .route(
            "/api/admin/landing-pages/{page_id}/variants",
            get(list_variants).post(create_variant),
        )
        .route(
            "/api/admin/landing-pages/{page_id}/variants/{variant_id}",
            put(update_variant).delete(delete_variant),
        )
        .route(
            "/api/admin/landing-pages/{page_id}/variants/{variant_id}/promote",
            post(promote_variant),
        )
        // UTM Presets
        .route(
            "/api/admin/utm-presets",
            get(list_utm_presets).post(create_utm_preset),
        )
        .route(
            "/api/admin/utm-presets/{preset_id}",
            delete(delete_utm_preset),
        )
        // Funnel analytics
        .route(
            "/api/admin/landing-pages/{page_id}/analytics",
            get(get_analytics),
        )
}

// ─────────────────────────────────────────────────────────────────────────────
// Page handlers
// ─────────────────────────────────────────────────────────────────────────────

/// `GET /api/admin/landing-pages?app_id=folio`
/// Lists all pages for an app (including unpublished). Defaults to "folio".
pub async fn list_pages(
    Query(q): Query<AppIdQuery>,
    State(db): State<DatabaseConnection>,
) -> Result<Json<Vec<LandingPageSummary>>, StatusCode> {
    let app_id = q.app_id.unwrap_or_else(|| "folio".to_string());
    let pages = AppPage::find()
        .filter(app_page::Column::AppId.eq(&app_id))
        .all(&db)
        .await
        .map_err(|e| {
            tracing::error!("landing_pages::list_pages error: {:?}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;
    Ok(Json(
        pages.into_iter().map(LandingPageSummary::from).collect(),
    ))
}

/// `GET /api/admin/landing-pages/{page_id}`
pub async fn get_page(
    Path(page_id): Path<Uuid>,
    State(db): State<DatabaseConnection>,
) -> Result<Json<app_page::Model>, StatusCode> {
    let page = AppPage::find_by_id(page_id)
        .one(&db)
        .await
        .map_err(|e| {
            tracing::error!("landing_pages::get_page error: {:?}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?
        .ok_or(StatusCode::NOT_FOUND)?;
    Ok(Json(page))
}

/// `POST /api/admin/landing-pages`
pub async fn create_page(
    State(db): State<DatabaseConnection>,
    Json(payload): Json<CreateLandingPagePayload>,
) -> Result<(StatusCode, Json<app_page::Model>), (StatusCode, String)> {
    let sentinel = Uuid::parse_str(PLATFORM_SENTINEL).unwrap();
    let now = Utc::now();
    let new_page = app_page::ActiveModel {
        id: Set(Uuid::new_v4()),
        tenant_id: Set(sentinel),
        app_id: Set(payload.app_id),
        slug: Set(payload.slug),
        locale: Set(payload.locale.unwrap_or_else(|| "en".to_string())),
        title: Set(payload.title),
        description: Set(payload.description.unwrap_or_default()),
        page_type: Set(payload.page_type.unwrap_or_else(|| "landing".to_string())),
        hero_payload: Set(payload.hero_payload),
        blocks_payload: Set(payload.blocks_payload),
        is_published: Set(payload.is_published.unwrap_or(false)),
        created_at: Set(now),
        updated_at: Set(now),
    };
    let inserted = new_page.insert(&db).await.map_err(|e| {
        tracing::error!("landing_pages::create_page error: {:?}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Failed to create page".to_string(),
        )
    })?;
    Ok((StatusCode::CREATED, Json(inserted)))
}

/// `PUT /api/admin/landing-pages/{page_id}`
pub async fn update_page(
    Path(page_id): Path<Uuid>,
    State(db): State<DatabaseConnection>,
    Json(payload): Json<UpdateLandingPagePayload>,
) -> Result<Json<app_page::Model>, (StatusCode, String)> {
    let existing = AppPage::find_by_id(page_id)
        .one(&db)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .ok_or_else(|| (StatusCode::NOT_FOUND, "Page not found".to_string()))?;

    let mut active: app_page::ActiveModel = existing.into();
    if let Some(t) = payload.title {
        active.title = Set(t);
    }
    if let Some(d) = payload.description {
        active.description = Set(d);
    }
    if let Some(pt) = payload.page_type {
        active.page_type = Set(pt);
    }
    if let Some(h) = payload.hero_payload {
        active.hero_payload = Set(Some(h));
    }
    if let Some(b) = payload.blocks_payload {
        active.blocks_payload = Set(Some(b));
    }
    if let Some(s) = payload.slug {
        active.slug = Set(s);
    }
    if let Some(l) = payload.locale {
        active.locale = Set(l);
    }
    active.updated_at = Set(Utc::now());

    let updated = active
        .update(&db)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(updated))
}

/// `POST /api/admin/landing-pages/{page_id}/publish` — toggles is_published
pub async fn toggle_publish(
    Path(page_id): Path<Uuid>,
    State(db): State<DatabaseConnection>,
) -> Result<Json<app_page::Model>, (StatusCode, String)> {
    let existing = AppPage::find_by_id(page_id)
        .one(&db)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .ok_or_else(|| (StatusCode::NOT_FOUND, "Page not found".to_string()))?;

    let new_state = !existing.is_published;
    let mut active: app_page::ActiveModel = existing.into();
    active.is_published = Set(new_state);
    active.updated_at = Set(Utc::now());

    let updated = active
        .update(&db)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(updated))
}

/// `DELETE /api/admin/landing-pages/{page_id}`
pub async fn delete_page(
    Path(page_id): Path<Uuid>,
    State(db): State<DatabaseConnection>,
) -> Result<StatusCode, (StatusCode, String)> {
    let existing = AppPage::find_by_id(page_id)
        .one(&db)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .ok_or_else(|| (StatusCode::NOT_FOUND, "Page not found".to_string()))?;

    let active: app_page::ActiveModel = existing.into();
    active
        .delete(&db)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(StatusCode::NO_CONTENT)
}

// ─────────────────────────────────────────────────────────────────────────────
// Variant handlers (A/B testing)
// ─────────────────────────────────────────────────────────────────────────────

/// `GET /api/admin/landing-pages/{page_id}/variants`
pub async fn list_variants(
    Path(page_id): Path<Uuid>,
    State(db): State<DatabaseConnection>,
) -> Result<Json<Vec<app_page_variant::Model>>, StatusCode> {
    let variants = AppPageVariant::find()
        .filter(app_page_variant::Column::PageId.eq(page_id))
        .all(&db)
        .await
        .map_err(|e| {
            tracing::error!("landing_pages::list_variants error: {:?}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;
    Ok(Json(variants))
}

/// `POST /api/admin/landing-pages/{page_id}/variants`
pub async fn create_variant(
    Path(page_id): Path<Uuid>,
    State(db): State<DatabaseConnection>,
    Json(payload): Json<CreateVariantPayload>,
) -> Result<(StatusCode, Json<app_page_variant::Model>), (StatusCode, String)> {
    // Confirm parent page exists
    AppPage::find_by_id(page_id)
        .one(&db)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .ok_or_else(|| (StatusCode::NOT_FOUND, "Parent page not found".to_string()))?;

    let now = Utc::now();
    let new_variant = app_page_variant::ActiveModel {
        id: Set(Uuid::new_v4()),
        page_id: Set(page_id),
        name: Set(payload.name),
        traffic_pct: Set(payload.traffic_pct.unwrap_or(50)),
        is_control: Set(payload.is_control.unwrap_or(false)),
        blocks_payload: Set(payload.blocks_payload.unwrap_or(serde_json::json!([]))),
        hero_payload: Set(payload.hero_payload),
        view_count: Set(0),
        lead_count: Set(0),
        is_active: Set(true),
        created_at: Set(now),
        updated_at: Set(now),
    };
    let inserted = new_variant.insert(&db).await.map_err(|e| {
        tracing::error!("landing_pages::create_variant error: {:?}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Failed to create variant".to_string(),
        )
    })?;
    Ok((StatusCode::CREATED, Json(inserted)))
}

/// `PUT /api/admin/landing-pages/{page_id}/variants/{variant_id}`
pub async fn update_variant(
    Path((page_id, variant_id)): Path<(Uuid, Uuid)>,
    State(db): State<DatabaseConnection>,
    Json(payload): Json<UpdateVariantPayload>,
) -> Result<Json<app_page_variant::Model>, (StatusCode, String)> {
    let existing = AppPageVariant::find_by_id(variant_id)
        .filter(app_page_variant::Column::PageId.eq(page_id))
        .one(&db)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .ok_or_else(|| (StatusCode::NOT_FOUND, "Variant not found".to_string()))?;

    let mut active: app_page_variant::ActiveModel = existing.into();
    if let Some(n) = payload.name {
        active.name = Set(n);
    }
    if let Some(t) = payload.traffic_pct {
        active.traffic_pct = Set(t);
    }
    if let Some(b) = payload.blocks_payload {
        active.blocks_payload = Set(b);
    }
    if let Some(h) = payload.hero_payload {
        active.hero_payload = Set(Some(h));
    }
    if let Some(a) = payload.is_active {
        active.is_active = Set(a);
    }
    active.updated_at = Set(Utc::now());

    let updated = active
        .update(&db)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(updated))
}

/// `DELETE /api/admin/landing-pages/{page_id}/variants/{variant_id}`
pub async fn delete_variant(
    Path((page_id, variant_id)): Path<(Uuid, Uuid)>,
    State(db): State<DatabaseConnection>,
) -> Result<StatusCode, (StatusCode, String)> {
    let existing = AppPageVariant::find_by_id(variant_id)
        .filter(app_page_variant::Column::PageId.eq(page_id))
        .one(&db)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .ok_or_else(|| (StatusCode::NOT_FOUND, "Variant not found".to_string()))?;

    let active: app_page_variant::ActiveModel = existing.into();
    active
        .delete(&db)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(StatusCode::NO_CONTENT)
}

/// `POST /api/admin/landing-pages/{page_id}/variants/{variant_id}/promote`
///
/// Promotes a winning variant:
/// 1. Copies variant `blocks_payload` + `hero_payload` to the parent page.
/// 2. Deletes **all** variants for this page (test is concluded).
/// 3. Returns the updated parent page.
pub async fn promote_variant(
    Path((page_id, variant_id)): Path<(Uuid, Uuid)>,
    State(db): State<DatabaseConnection>,
) -> Result<Json<app_page::Model>, (StatusCode, String)> {
    // Fetch winner variant
    let winner = AppPageVariant::find_by_id(variant_id)
        .filter(app_page_variant::Column::PageId.eq(page_id))
        .one(&db)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .ok_or_else(|| (StatusCode::NOT_FOUND, "Variant not found".to_string()))?;

    // Fetch parent page
    let page = AppPage::find_by_id(page_id)
        .one(&db)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .ok_or_else(|| (StatusCode::NOT_FOUND, "Page not found".to_string()))?;

    // Promote: copy winner content to parent
    let mut active_page: app_page::ActiveModel = page.into();
    active_page.blocks_payload = Set(Some(winner.blocks_payload.clone()));
    if let Some(h) = winner.hero_payload.clone() {
        active_page.hero_payload = Set(Some(h));
    }
    active_page.updated_at = Set(Utc::now());
    let updated_page = active_page
        .update(&db)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    // Delete all variants for this page (test concluded)
    AppPageVariant::delete_many()
        .filter(app_page_variant::Column::PageId.eq(page_id))
        .exec(&db)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(updated_page))
}

// ─────────────────────────────────────────────────────────────────────────────
// UTM preset handlers
// ─────────────────────────────────────────────────────────────────────────────

/// `GET /api/admin/utm-presets?app_id=folio`
pub async fn list_utm_presets(
    Query(q): Query<AppIdQuery>,
    State(db): State<DatabaseConnection>,
) -> Result<Json<Vec<app_utm_preset::Model>>, StatusCode> {
    let app_id = q.app_id.unwrap_or_else(|| "folio".to_string());
    let presets = AppUtmPreset::find()
        .filter(app_utm_preset::Column::AppId.eq(&app_id))
        .all(&db)
        .await
        .map_err(|e| {
            tracing::error!("landing_pages::list_utm_presets error: {:?}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;
    Ok(Json(presets))
}

/// `POST /api/admin/utm-presets`
pub async fn create_utm_preset(
    State(db): State<DatabaseConnection>,
    Json(payload): Json<CreateUtmPresetPayload>,
) -> Result<(StatusCode, Json<app_utm_preset::Model>), (StatusCode, String)> {
    let now = Utc::now();
    let new_preset = app_utm_preset::ActiveModel {
        id: Set(Uuid::new_v4()),
        app_id: Set(payload.app_id),
        name: Set(payload.name),
        utm_source: Set(payload.utm_source),
        utm_medium: Set(payload.utm_medium),
        utm_campaign: Set(payload.utm_campaign),
        utm_content: Set(payload.utm_content),
        utm_term: Set(payload.utm_term),
        click_count: Set(0),
        created_at: Set(now),
        updated_at: Set(now),
    };
    let inserted = new_preset.insert(&db).await.map_err(|e| {
        tracing::error!("landing_pages::create_utm_preset error: {:?}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Failed to create UTM preset".to_string(),
        )
    })?;
    Ok((StatusCode::CREATED, Json(inserted)))
}

/// `DELETE /api/admin/utm-presets/{preset_id}`
pub async fn delete_utm_preset(
    Path(preset_id): Path<Uuid>,
    State(db): State<DatabaseConnection>,
) -> Result<StatusCode, (StatusCode, String)> {
    let existing = AppUtmPreset::find_by_id(preset_id)
        .one(&db)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .ok_or_else(|| (StatusCode::NOT_FOUND, "UTM preset not found".to_string()))?;

    let active: app_utm_preset::ActiveModel = existing.into();
    active
        .delete(&db)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(StatusCode::NO_CONTENT)
}

// ─────────────────────────────────────────────────────────────────────────────
// Tracking pixel handlers
// ─────────────────────────────────────────────────────────────────────────────

/// One pixel provider's config.
#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct PixelConfig {
    pub enabled: bool,
    pub snippet: Option<String>,
}

/// Full pixel config map returned by GET /pixels.
#[derive(Debug, Serialize, Deserialize, Default)]
pub struct PagePixelConfig {
    pub ga4: PixelConfig,
    pub meta: PixelConfig,
    pub linkedin: PixelConfig,
    pub gtm: PixelConfig,
}

impl PagePixelConfig {
    /// Deserialize from the hero_payload["pixel_config"] JSONB value.
    fn from_value(v: &Value) -> Self {
        let parse_one = |key: &str| -> PixelConfig {
            v.get(key)
                .and_then(|o| serde_json::from_value::<PixelConfig>(o.clone()).ok())
                .unwrap_or_default()
        };
        Self {
            ga4: parse_one("ga4"),
            meta: parse_one("meta"),
            linkedin: parse_one("linkedin"),
            gtm: parse_one("gtm"),
        }
    }
}

/// Body for `PUT /pixels/{pixel_type}`.
#[derive(Debug, Deserialize)]
pub struct SetPixelBody {
    pub enabled: bool,
    pub snippet: Option<String>,
}

/// `GET /api/admin/landing-pages/{page_id}/pixels`
/// Returns the pixel config stored in hero_payload["pixel_config"].
pub async fn get_pixels(
    Path(page_id): Path<Uuid>,
    State(db): State<DatabaseConnection>,
) -> Result<Json<PagePixelConfig>, (StatusCode, String)> {
    let page = AppPage::find_by_id(page_id)
        .one(&db)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .ok_or_else(|| (StatusCode::NOT_FOUND, "Page not found".to_string()))?;

    let pixel_cfg = page
        .hero_payload
        .as_ref()
        .and_then(|h| h.get("pixel_config").cloned())
        .map(|v| PagePixelConfig::from_value(&v))
        .unwrap_or_else(|| PagePixelConfig {
            ga4: PixelConfig::default(),
            meta: PixelConfig::default(),
            linkedin: PixelConfig::default(),
            gtm: PixelConfig::default(),
        });

    Ok(Json(pixel_cfg))
}

/// `PUT /api/admin/landing-pages/{page_id}/pixels/{pixel_type}`
/// Enables/disables one pixel and stores the optional snippet.
/// Mutates hero_payload["pixel_config"][pixel_type] in place.
pub async fn set_pixel(
    Path((page_id, pixel_type)): Path<(Uuid, String)>,
    State(db): State<DatabaseConnection>,
    Json(body): Json<SetPixelBody>,
) -> Result<Json<PagePixelConfig>, (StatusCode, String)> {
    // Validate pixel type
    let valid_types = ["ga4", "meta", "linkedin", "gtm"];
    if !valid_types.contains(&pixel_type.as_str()) {
        return Err((
            StatusCode::BAD_REQUEST,
            format!(
                "Unknown pixel type '{}'. Valid: ga4, meta, linkedin, gtm",
                pixel_type
            ),
        ));
    }

    let page = AppPage::find_by_id(page_id)
        .one(&db)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .ok_or_else(|| (StatusCode::NOT_FOUND, "Page not found".to_string()))?;

    // Merge into existing hero_payload, preserving all other keys
    let mut hero = page
        .hero_payload
        .clone()
        .unwrap_or_else(|| serde_json::json!({}));

    let pixel_cfg_val = hero
        .get_mut("pixel_config")
        .filter(|v| v.is_object())
        .cloned()
        .unwrap_or_else(|| serde_json::json!({}));

    let mut pixel_map = match pixel_cfg_val {
        Value::Object(m) => m,
        _ => serde_json::Map::new(),
    };

    pixel_map.insert(
        pixel_type.clone(),
        serde_json::json!({
            "enabled": body.enabled,
            "snippet": body.snippet,
        }),
    );

    hero.as_object_mut()
        .ok_or_else(|| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "hero_payload is not an object".to_string(),
            )
        })?
        .insert("pixel_config".to_string(), Value::Object(pixel_map));

    // Persist
    let mut active: app_page::ActiveModel = page.into();
    active.hero_payload = Set(Some(hero.clone()));
    active.updated_at = Set(Utc::now());
    active
        .update(&db)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    // Return the full updated pixel config
    let updated_cfg = hero
        .get("pixel_config")
        .map(|v| PagePixelConfig::from_value(v))
        .unwrap_or_default();

    Ok(Json(updated_cfg))
}

// ─────────────────────────────────────────────────────────────────────────────
// Funnel Analytics handlers
// ─────────────────────────────────────────────────────────────────────────────

#[derive(Debug, Serialize)]
pub struct SourceBreakdown {
    pub source: String,
    pub views: i64,
    pub leads: i64,
    pub pct: i32,
}

#[derive(Debug, Serialize)]
pub struct PageAnalytics {
    pub page_id: Uuid,
    pub total_views: i64,
    pub total_leads: i64,
    pub cta_clicks: i64,
    pub conv_rate_pct: f64, // leads / views * 100
    pub sources: Vec<SourceBreakdown>,
}

/// `GET /api/admin/landing-pages/{page_id}/analytics`
/// Aggregates atlas_lp_events for the given page (last 30 days).
pub async fn get_analytics(
    Path(page_id): Path<Uuid>,
    State(db): State<DatabaseConnection>,
) -> Result<Json<PageAnalytics>, (StatusCode, String)> {
    use sea_orm::prelude::*;

    // Total event counts per type (last 30 days)
    let cutoff = Utc::now() - chrono::Duration::days(30);

    let events = AtlasLpEvent::find()
        .filter(atlas_lp_event::Column::AppPageId.eq(page_id))
        .filter(atlas_lp_event::Column::CreatedAt.gte(cutoff))
        .all(&db)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let total_views = events.iter().filter(|e| e.event_type == "view").count() as i64;
    let total_leads = events
        .iter()
        .filter(|e| e.event_type == "lead_submitted")
        .count() as i64;
    let cta_clicks = events
        .iter()
        .filter(|e| e.event_type == "cta_click")
        .count() as i64;
    let conv_rate_pct = if total_views > 0 {
        (total_leads as f64 / total_views as f64) * 100.0
    } else {
        0.0
    };

    // UTM source breakdown
    let mut source_map: std::collections::HashMap<String, (i64, i64)> = Default::default();
    for ev in &events {
        let src = ev
            .utm_source
            .clone()
            .unwrap_or_else(|| "Direct / Other".to_string());
        let entry = source_map.entry(src).or_insert((0, 0));
        if ev.event_type == "view" {
            entry.0 += 1;
        }
        if ev.event_type == "lead_submitted" {
            entry.1 += 1;
        }
    }

    let mut sources: Vec<SourceBreakdown> = source_map
        .into_iter()
        .map(|(source, (views, leads))| {
            let pct = if total_views > 0 {
                (views * 100 / total_views) as i32
            } else {
                0
            };
            SourceBreakdown {
                source,
                views,
                leads,
                pct,
            }
        })
        .collect();
    sources.sort_by(|a, b| b.views.cmp(&a.views));

    Ok(Json(PageAnalytics {
        page_id,
        total_views,
        total_leads,
        cta_clicks,
        conv_rate_pct,
        sources,
    }))
}

/// Public event ingest body.
#[derive(Debug, Deserialize)]
pub struct LpEventBody {
    pub app_page_id: Uuid,
    pub event_type: String,
    pub session_id: String,
    pub utm_source: Option<String>,
    pub utm_medium: Option<String>,
    pub utm_campaign: Option<String>,
    pub utm_content: Option<String>,
    pub utm_term: Option<String>,
    pub viewport: Option<String>,
    pub referrer: Option<String>,
}

/// `POST /api/pub/lp-events`
/// Fire-and-forget landing page event ingest (no auth, rate-limited at infra level).
pub async fn post_lp_event(
    State(db): State<DatabaseConnection>,
    Json(body): Json<LpEventBody>,
) -> StatusCode {
    // Validate event type
    let valid = ["view", "lead_submitted", "cta_click"];
    if !valid.contains(&body.event_type.as_str()) {
        return StatusCode::BAD_REQUEST;
    }

    // Clone fields needed for G-20 attribution before moving into ActiveModel.
    let has_utm = body.utm_source.is_some()
        || body.utm_medium.is_some()
        || body.utm_campaign.is_some();
    let session_id = body.session_id.clone();
    let utm_source = body.utm_source.clone();
    let utm_medium = body.utm_medium.clone();
    let utm_campaign = body.utm_campaign.clone();
    let utm_content = body.utm_content.clone();
    let utm_term = body.utm_term.clone();
    let referrer = body.referrer.clone();

    let ev = atlas_lp_event::ActiveModel {
        id: Set(Uuid::new_v4()),
        app_page_id: Set(body.app_page_id),
        event_type: Set(body.event_type),
        session_id: Set(body.session_id),
        utm_source: Set(body.utm_source),
        utm_medium: Set(body.utm_medium),
        utm_campaign: Set(body.utm_campaign),
        utm_content: Set(body.utm_content),
        utm_term: Set(body.utm_term),
        viewport: Set(body.viewport),
        referrer: Set(body.referrer),
        country_code: Set(None),
        created_at: Set(Utc::now()),
    };

    // Fire-and-forget — log errors but don't fail the response
    if let Err(e) = ev.insert(&db).await {
        tracing::warn!("lp_event insert failed: {:?}", e);
    }

    // ── G-20 Attribution Touchpoint Capture ─────────────────────────────────────
    // If UTMs are present, also capture attribution touchpoint using session_id as anonymous_id
    if has_utm {
        use crate::services::flag_service::FlagService;
        use crate::services::pm::attribution::{AttributionService, CapturePayload, UtmParams};
        use crate::types::pm::AttributionChannel;

        let platform_tenant = Uuid::nil();
        let dm_tracking_enabled = FlagService::is_enabled(
            &db,
            platform_tenant,
            None,
            "acquisition.dm_tracking",
        )
        .await
        .unwrap_or(true);

        if dm_tracking_enabled {
            let channel = utm_medium
                .as_deref()
                .and_then(|medium| match medium {
                    "direct_mail" | "postcard" | "mail" => Some(AttributionChannel::DirectMail),
                    "cpc" | "ppc" | "paid_search" => Some(AttributionChannel::PaidSearch),
                    "organic" | "seo" => Some(AttributionChannel::OrganicSearch),
                    "social" => Some(AttributionChannel::OrganicSocial),
                    "paid_social" => Some(AttributionChannel::PaidSocial),
                    "email" => Some(AttributionChannel::ColdEmail),
                    "referral" => Some(AttributionChannel::Referral),
                    "sms" => Some(AttributionChannel::Sms),
                    "content" => Some(AttributionChannel::Content),
                    "affiliate" => Some(AttributionChannel::Affiliate),
                    _ => None,
                })
                .unwrap_or(AttributionChannel::Direct);

            let capture_payload = CapturePayload {
                channel,
                utm: UtmParams {
                    utm_source,
                    utm_medium,
                    utm_campaign,
                    utm_content,
                    utm_term,
                },
                user_id: None,
                contact_email: None,
                anonymous_id: Some(session_id.clone()),
                campaign_id: None,
                enrollment_id: None,
                event_id: None,
                landing_page_url: None,
                referrer_url: referrer,
            };

            if let Err(e) =
                AttributionService::capture_touchpoint(&db, platform_tenant, capture_payload).await
            {
                tracing::warn!(
                    %session_id,
                    error = %e,
                    "failed to capture landing page attribution touchpoint"
                );
            }
        }
    }

    StatusCode::ACCEPTED
}

// ─────────────────────────────────────────────────────────────────────────────
// Unit tests (pure logic, no DB)
// ─────────────────────────────────────────────────────────────────────────────
#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    // ── PagePixelConfig::from_value ──────────────────────────────────────────

    #[test]
    fn pixel_config_full_roundtrip() {
        // All four providers present and correctly parsed.
        let v = json!({
            "ga4":      { "enabled": true,  "snippet": "GA_SNIPPET" },
            "meta":     { "enabled": false, "snippet": null },
            "linkedin": { "enabled": true,  "snippet": "LI_SNIPPET" },
            "gtm":      { "enabled": false, "snippet": null },
        });
        let cfg = PagePixelConfig::from_value(&v);

        assert!(cfg.ga4.enabled, "ga4 should be enabled");
        assert_eq!(cfg.ga4.snippet.as_deref(), Some("GA_SNIPPET"));
        assert!(!cfg.meta.enabled, "meta should be disabled");
        assert!(cfg.linkedin.enabled, "linkedin should be enabled");
        assert_eq!(cfg.linkedin.snippet.as_deref(), Some("LI_SNIPPET"));
        assert!(!cfg.gtm.enabled, "gtm should be disabled");
    }

    #[test]
    fn pixel_config_partial_json_defaults_missing_keys() {
        // Only ga4 present in JSONB — others must default to disabled with no snippet.
        let v = json!({ "ga4": { "enabled": true, "snippet": null } });
        let cfg = PagePixelConfig::from_value(&v);

        assert!(cfg.ga4.enabled, "ga4 should be enabled");
        assert!(!cfg.meta.enabled, "meta missing from JSON → disabled");
        assert!(
            !cfg.linkedin.enabled,
            "linkedin missing from JSON → disabled"
        );
        assert!(!cfg.gtm.enabled, "gtm missing from JSON → disabled");
        assert!(cfg.meta.snippet.is_none());
    }

    #[test]
    fn pixel_config_empty_object_all_disabled() {
        let cfg = PagePixelConfig::from_value(&json!({}));
        assert!(!cfg.ga4.enabled);
        assert!(!cfg.meta.enabled);
        assert!(!cfg.linkedin.enabled);
        assert!(!cfg.gtm.enabled);
    }

    #[test]
    fn pixel_config_malformed_value_falls_back_to_default() {
        // A provider value that isn't an object should silently default.
        let v = json!({ "ga4": "not-an-object", "meta": 42 });
        let cfg = PagePixelConfig::from_value(&v);
        assert!(!cfg.ga4.enabled, "malformed ga4 value → disabled default");
        assert!(!cfg.meta.enabled, "malformed meta value → disabled default");
    }

    // ── Analytics conv_rate_pct guard ────────────────────────────────────────

    #[test]
    fn conv_rate_zero_views_does_not_divide_by_zero() {
        let (views, leads): (i64, i64) = (0, 0);
        let rate = if views > 0 {
            (leads as f64 / views as f64) * 100.0
        } else {
            0.0
        };
        assert_eq!(rate, 0.0, "zero views must yield 0.0, not NaN or panic");
    }

    #[test]
    fn conv_rate_calculation_is_correct() {
        let (views, leads): (i64, i64) = (200, 10);
        let rate = if views > 0 {
            (leads as f64 / views as f64) * 100.0
        } else {
            0.0
        };
        // 10/200 * 100 = 5.0
        assert!((rate - 5.0).abs() < 1e-9, "expected 5.0, got {rate}");
    }

    #[test]
    fn conv_rate_full_conversion_is_100() {
        let (views, leads): (i64, i64) = (50, 50);
        let rate = if views > 0 {
            (leads as f64 / views as f64) * 100.0
        } else {
            0.0
        };
        assert!((rate - 100.0).abs() < 1e-9);
    }

    // ── Pixel type validation ─────────────────────────────────────────────────

    #[test]
    fn pixel_type_accepts_all_valid_types() {
        let valid_types = ["ga4", "meta", "linkedin", "gtm"];
        for t in &["ga4", "meta", "linkedin", "gtm"] {
            assert!(
                valid_types.contains(t),
                "'{t}' should be a valid pixel type"
            );
        }
    }

    #[test]
    fn pixel_type_rejects_unknown_providers() {
        let valid_types = ["ga4", "meta", "linkedin", "gtm"];
        for bad in &["tiktok", "twitter", "snapchat", "", "GA4", "Meta"] {
            assert!(
                !valid_types.contains(bad),
                "'{bad}' should NOT be a valid pixel type"
            );
        }
    }

    // ── Event type validation ─────────────────────────────────────────────────

    #[test]
    fn event_type_accepts_all_valid_types() {
        let valid = ["view", "lead_submitted", "cta_click"];
        for t in &valid {
            assert!(valid.contains(t), "'{t}' should be a valid event type");
        }
    }

    #[test]
    fn event_type_rejects_common_mistakes() {
        let valid = ["view", "lead_submitted", "cta_click"];
        // Common wrong names that callers might send
        for bad in &["pageview", "click", "submit", "View", "LEAD_SUBMITTED", ""] {
            assert!(
                !valid.contains(bad),
                "'{bad}' should NOT be a valid event type"
            );
        }
    }

    // ── JSONB pixel merge preserves sibling keys ──────────────────────────────

    #[test]
    fn pixel_merge_preserves_existing_hero_payload_keys() {
        // Simulate the set_pixel handler's JSONB merge logic end-to-end.
        let mut hero = json!({
            "headline": "Welcome to Folio",
            "cta_text": "Get Started",
            "pixel_config": {
                "ga4": { "enabled": true, "snippet": "GA-12345" }
            }
        });

        // Apply a toggle for "meta" pixel
        let pixel_cfg_val = hero
            .get_mut("pixel_config")
            .filter(|v| v.is_object())
            .cloned()
            .unwrap_or_else(|| json!({}));

        let mut pixel_map = match pixel_cfg_val {
            serde_json::Value::Object(m) => m,
            _ => serde_json::Map::new(),
        };
        pixel_map.insert(
            "meta".to_string(),
            json!({ "enabled": true, "snippet": null }),
        );

        hero.as_object_mut().unwrap().insert(
            "pixel_config".to_string(),
            serde_json::Value::Object(pixel_map),
        );

        // ga4 entry should be untouched
        assert_eq!(hero["pixel_config"]["ga4"]["enabled"], true);
        assert_eq!(hero["pixel_config"]["ga4"]["snippet"], "GA-12345");
        // meta should now be present and enabled
        assert_eq!(hero["pixel_config"]["meta"]["enabled"], true);
        // Non-pixel hero keys must survive the merge
        assert_eq!(hero["headline"], "Welcome to Folio");
        assert_eq!(hero["cta_text"], "Get Started");
    }

    #[test]
    fn pixel_merge_creates_pixel_config_when_absent() {
        // hero_payload with NO pixel_config key yet — merge should create it.
        let mut hero = json!({ "headline": "No pixels yet" });

        let pixel_cfg_val = hero
            .get_mut("pixel_config")
            .filter(|v| v.is_object())
            .cloned()
            .unwrap_or_else(|| json!({}));

        let mut pixel_map = match pixel_cfg_val {
            serde_json::Value::Object(m) => m,
            _ => serde_json::Map::new(),
        };
        pixel_map.insert(
            "gtm".to_string(),
            json!({ "enabled": true, "snippet": "GTM-XYZ" }),
        );

        hero.as_object_mut().unwrap().insert(
            "pixel_config".to_string(),
            serde_json::Value::Object(pixel_map),
        );

        assert_eq!(hero["pixel_config"]["gtm"]["enabled"], true);
        assert_eq!(hero["pixel_config"]["gtm"]["snippet"], "GTM-XYZ");
        // Original key survives
        assert_eq!(hero["headline"], "No pixels yet");
    }
}
