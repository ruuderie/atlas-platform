#![allow(dead_code)]
use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
    routing::{get, post, put, delete},
    Router,
};
use sea_orm::{
    DatabaseConnection, EntityTrait, QueryFilter, ColumnTrait,
    ActiveModelTrait, Set,
};
use serde::{Deserialize, Serialize};
use crate::entities::app_page::{self, Entity as AppPage};
use uuid::Uuid;
use chrono::Utc;

// ── Request / Response types ──────────────────────────────────────────────────

#[derive(Deserialize)]
pub struct CreatePagePayload {
    pub slug: String,
    pub title: String,
    pub description: String,
    pub page_type: Option<String>,
    pub hero_payload: Option<serde_json::Value>,
    pub blocks_payload: Option<serde_json::Value>,
    pub is_published: Option<bool>,
}

#[derive(Deserialize)]
pub struct UpdatePagePayload {
    pub title: Option<String>,
    pub description: Option<String>,
    pub page_type: Option<String>,
    pub hero_payload: Option<serde_json::Value>,
    pub blocks_payload: Option<serde_json::Value>,
    pub is_published: Option<bool>,
}

#[derive(Serialize)]
pub struct PageSummary {
    pub id: Uuid,
    pub slug: String,
    pub title: String,
    pub page_type: String,
    pub is_published: bool,
    pub created_at: chrono::DateTime<Utc>,
    pub updated_at: chrono::DateTime<Utc>,
}

impl From<app_page::Model> for PageSummary {
    fn from(m: app_page::Model) -> Self {
        Self {
            id: m.id,
            slug: m.slug,
            title: m.title,
            page_type: m.page_type,
            is_published: m.is_published,
            created_at: m.created_at,
            updated_at: m.updated_at,
        }
    }
}

// ── Route constructors ────────────────────────────────────────────────────────

/// State-free public route definitions.
/// Use inside `AtlasApp::public_router()`. Never call `.with_state()` here.
pub fn public_routes_raw() -> Router<DatabaseConnection> {
    Router::new()
        .route("/api/public/pages/{tenant_id}", get(list_pages))
        .route("/api/public/pages/{tenant_id}/{*slug}", get(get_page_by_slug))
}

/// State-free authenticated CRUD route definitions.
/// Use inside `AtlasApp::authenticated_router()`. Never call `.with_state()` here.
pub fn authenticated_routes_raw() -> Router<DatabaseConnection> {
    Router::new()
        // List all pages (including unpublished) for platform-admin
        .route("/api/pages/{tenant_id}", get(list_all_pages))
        // Get a single page by slug (including unpublished)
        .route("/api/pages/{tenant_id}/{*slug}", get(get_page_admin))
        // Create a page
        .route("/api/pages/{tenant_id}", post(create_page))
        // Update a page by slug
        .route("/api/pages/{tenant_id}/{*slug}", put(update_page))
        // Delete a page by slug
        .route("/api/pages/{tenant_id}/{*slug}", delete(delete_page))
}

/// Legacy state-finalized constructor. Used by api.rs during transition period.
/// Remove after CorePlatformApp is active and api.rs is cleaned up (Phase 3).
pub fn public_routes(db: DatabaseConnection) -> Router<DatabaseConnection> {
    public_routes_raw().with_state(db)
}

// ── Public handlers ───────────────────────────────────────────────────────────

pub async fn list_pages(
    Path(tenant_id): Path<Uuid>,
    State(db): State<DatabaseConnection>,
) -> Result<Json<Vec<app_page::Model>>, StatusCode> {
    let pages = AppPage::find()
        .filter(app_page::Column::TenantId.eq(tenant_id))
        .filter(app_page::Column::IsPublished.eq(true))
        .all(&db)
        .await
        .map_err(|err| {
            tracing::error!("Database error fetching pages: {:?}", err);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    Ok(Json(pages))
}

pub async fn get_page_by_slug(
    Path((tenant_id, slug)): Path<(Uuid, String)>,
    State(db): State<DatabaseConnection>,
) -> Result<Json<app_page::Model>, StatusCode> {
    let clean_slug = slug.trim_start_matches('/');
    
    tracing::info!("DEBUG get_page_by_slug: tenant_id={}, slug='{}', clean_slug='{}'", tenant_id, slug, clean_slug);
    let page = AppPage::find()
        .filter(app_page::Column::TenantId.eq(tenant_id))
        .filter(app_page::Column::Slug.eq(clean_slug))
        .filter(app_page::Column::IsPublished.eq(true))
        .one(&db)
        .await
        .map_err(|err| {
            tracing::error!("Database error fetching page by slug: {:?}", err);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    if let Some(p) = page {
        Ok(Json(p))
    } else {
        Err(StatusCode::NOT_FOUND)
    }
}

// ── Authenticated CRUD handlers ───────────────────────────────────────────────

/// Lists ALL pages for a tenant (including unpublished). Platform-admin only.
pub async fn list_all_pages(
    Path(tenant_id): Path<Uuid>,
    State(db): State<DatabaseConnection>,
) -> Result<Json<Vec<PageSummary>>, StatusCode> {
    let pages = AppPage::find()
        .filter(app_page::Column::TenantId.eq(tenant_id))
        .all(&db)
        .await
        .map_err(|e| {
            tracing::error!("list_all_pages error: {:?}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    Ok(Json(pages.into_iter().map(PageSummary::from).collect()))
}

/// Gets a single page by slug (including unpublished). Platform-admin only.
pub async fn get_page_admin(
    Path((tenant_id, slug)): Path<(Uuid, String)>,
    State(db): State<DatabaseConnection>,
) -> Result<Json<app_page::Model>, StatusCode> {
    let clean_slug = slug.trim_start_matches('/');
    let page = AppPage::find()
        .filter(app_page::Column::TenantId.eq(tenant_id))
        .filter(app_page::Column::Slug.eq(clean_slug))
        .one(&db)
        .await
        .map_err(|e| {
            tracing::error!("get_page_admin error: {:?}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?
        .ok_or(StatusCode::NOT_FOUND)?;

    Ok(Json(page))
}

/// Creates a new page for a tenant.
pub async fn create_page(
    Path(tenant_id): Path<Uuid>,
    State(db): State<DatabaseConnection>,
    Json(payload): Json<CreatePagePayload>,
) -> Result<(StatusCode, Json<app_page::Model>), (StatusCode, String)> {
    let now = Utc::now();
    let new_page = app_page::ActiveModel {
        id: Set(Uuid::new_v4()),
        tenant_id: Set(tenant_id),
        slug: Set(payload.slug),
        title: Set(payload.title),
        description: Set(payload.description),
        page_type: Set(payload.page_type.unwrap_or_else(|| "standard".to_string())),
        hero_payload: Set(payload.hero_payload),
        blocks_payload: Set(payload.blocks_payload),
        is_published: Set(payload.is_published.unwrap_or(false)),
        created_at: Set(now),
        updated_at: Set(now),
    };

    let inserted = new_page.insert(&db).await.map_err(|e| {
        tracing::error!("create_page error: {:?}", e);
        (StatusCode::INTERNAL_SERVER_ERROR, "Failed to create page".to_string())
    })?;

    Ok((StatusCode::CREATED, Json(inserted)))
}

/// Updates a page by slug.
pub async fn update_page(
    Path((tenant_id, slug)): Path<(Uuid, String)>,
    State(db): State<DatabaseConnection>,
    Json(payload): Json<UpdatePagePayload>,
) -> Result<Json<app_page::Model>, (StatusCode, String)> {
    let clean_slug = slug.trim_start_matches('/');
    let existing = AppPage::find()
        .filter(app_page::Column::TenantId.eq(tenant_id))
        .filter(app_page::Column::Slug.eq(clean_slug))
        .one(&db)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .ok_or_else(|| (StatusCode::NOT_FOUND, "Page not found".to_string()))?;

    let mut active: app_page::ActiveModel = existing.into();
    if let Some(t) = payload.title         { active.title = Set(t); }
    if let Some(d) = payload.description   { active.description = Set(d); }
    if let Some(pt) = payload.page_type    { active.page_type = Set(pt); }
    if let Some(h) = payload.hero_payload  { active.hero_payload = Set(Some(h)); }
    if let Some(b) = payload.blocks_payload { active.blocks_payload = Set(Some(b)); }
    if let Some(p) = payload.is_published  { active.is_published = Set(p); }
    active.updated_at = Set(Utc::now());

    let updated = active.update(&db).await.map_err(|e| {
        (StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
    })?;

    Ok(Json(updated))
}

/// Deletes a page by slug.
pub async fn delete_page(
    Path((tenant_id, slug)): Path<(Uuid, String)>,
    State(db): State<DatabaseConnection>,
) -> Result<StatusCode, (StatusCode, String)> {
    let clean_slug = slug.trim_start_matches('/');
    let existing = AppPage::find()
        .filter(app_page::Column::TenantId.eq(tenant_id))
        .filter(app_page::Column::Slug.eq(clean_slug))
        .one(&db)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .ok_or_else(|| (StatusCode::NOT_FOUND, "Page not found".to_string()))?;

    let active: app_page::ActiveModel = existing.into();
    active.delete(&db).await.map_err(|e| {
        (StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
    })?;

    Ok(StatusCode::NO_CONTENT)
}
