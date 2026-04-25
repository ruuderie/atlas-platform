use axum::{
    extract::{Path, State, Extension},
    http::StatusCode,
    Json,
    routing::{get, post, put, delete},
    Router,
};
use sea_orm::{DatabaseConnection, EntityTrait, QueryFilter, ColumnTrait, ActiveModelTrait, Set, ModelTrait};
use crate::entities::app_page::{self, Entity as AppPage};
use crate::entities::user;
use crate::config::SiteConfig;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;
use chrono::Utc;

#[derive(Deserialize)]
pub struct CreatePagePayload {
    pub title: String,
    pub slug: String,
    pub page_type: String,
    pub is_published: bool,
    pub hero_payload: Option<Value>,
    pub blocks_payload: Option<Value>,
}

#[derive(Deserialize)]
pub struct UpdatePagePayload {
    pub title: Option<String>,
    pub slug: Option<String>,
    pub page_type: Option<String>,
    pub is_published: Option<bool>,
    pub hero_payload: Option<Value>,
    pub blocks_payload: Option<Value>,
}

pub fn authenticated_routes(db: DatabaseConnection) -> Router<DatabaseConnection> {
    Router::new()
        .route("/api/anchor/pages", get(list_pages).post(create_page))
        .route("/api/anchor/pages/{id}", get(get_page).put(update_page).delete(delete_page))
        .with_state(db)
}

pub async fn list_pages(
    State(db): State<DatabaseConnection>,
    Extension(_current_user): Extension<user::Model>,
    Extension(site_config): Extension<SiteConfig>,
) -> Result<Json<Vec<app_page::Model>>, StatusCode> {
    let pages = AppPage::find()
        .filter(app_page::Column::TenantId.eq(site_config.tenant_id))
        .all(&db)
        .await
        .map_err(|err| {
            tracing::error!("Database error fetching pages: {:?}", err);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    Ok(Json(pages))
}

pub async fn get_page(
    Path(id): Path<Uuid>,
    State(db): State<DatabaseConnection>,
    Extension(_current_user): Extension<user::Model>,
    Extension(site_config): Extension<SiteConfig>,
) -> Result<Json<app_page::Model>, StatusCode> {
    let page = AppPage::find()
        .filter(app_page::Column::Id.eq(id))
        .filter(app_page::Column::TenantId.eq(site_config.tenant_id))
        .one(&db)
        .await
        .map_err(|err| {
            tracing::error!("Database error fetching page by id: {:?}", err);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    if let Some(p) = page {
        Ok(Json(p))
    } else {
        Err(StatusCode::NOT_FOUND)
    }
}

pub async fn create_page(
    State(db): State<DatabaseConnection>,
    Extension(_current_user): Extension<user::Model>,
    Extension(site_config): Extension<SiteConfig>,
    Json(payload): Json<CreatePagePayload>,
) -> Result<(StatusCode, Json<app_page::Model>), (StatusCode, String)> {
    let clean_slug = payload.slug.trim_start_matches('/').to_string();

    // Check for existing slug
    let existing = AppPage::find()
        .filter(app_page::Column::TenantId.eq(site_config.tenant_id))
        .filter(app_page::Column::Slug.eq(&clean_slug))
        .one(&db)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    if existing.is_some() {
        return Err((StatusCode::CONFLICT, "A page with this slug already exists".into()));
    }

    let new_page = app_page::ActiveModel {
        id: Set(Uuid::new_v4()),
        tenant_id: Set(site_config.tenant_id),
        slug: Set(clean_slug),
        title: Set(payload.title),
        description: Set(String::new()),
        page_type: Set(payload.page_type),
        hero_payload: Set(payload.hero_payload),
        blocks_payload: Set(payload.blocks_payload),
        is_published: Set(payload.is_published),
        created_at: Set(Utc::now()),
        updated_at: Set(Utc::now()),
    };

    let inserted = new_page.insert(&db).await.map_err(|e| {
        tracing::error!("Error creating page: {:?}", e);
        (StatusCode::INTERNAL_SERVER_ERROR, "Failed to create page".into())
    })?;

    Ok((StatusCode::CREATED, Json(inserted)))
}

pub async fn update_page(
    Path(id): Path<Uuid>,
    State(db): State<DatabaseConnection>,
    Extension(_current_user): Extension<user::Model>,
    Extension(site_config): Extension<SiteConfig>,
    Json(payload): Json<UpdatePagePayload>,
) -> Result<Json<app_page::Model>, (StatusCode, String)> {
    let page = AppPage::find()
        .filter(app_page::Column::Id.eq(id))
        .filter(app_page::Column::TenantId.eq(site_config.tenant_id))
        .one(&db)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .ok_or((StatusCode::NOT_FOUND, "Page not found".into()))?;

    let mut active_page: app_page::ActiveModel = page.into();

    if let Some(slug) = payload.slug {
        let clean_slug = slug.trim_start_matches('/').to_string();
        
        // Check uniqueness if slug changed
        if active_page.slug.as_ref() != &clean_slug {
            let existing = AppPage::find()
                .filter(app_page::Column::TenantId.eq(site_config.tenant_id))
                .filter(app_page::Column::Slug.eq(&clean_slug))
                .one(&db)
                .await
                .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

            if existing.is_some() {
                return Err((StatusCode::CONFLICT, "A page with this slug already exists".into()));
            }
        }
        active_page.slug = Set(clean_slug);
    }

    if let Some(title) = payload.title {
        active_page.title = Set(title);
    }
    if let Some(page_type) = payload.page_type {
        active_page.page_type = Set(page_type);
    }
    if let Some(is_published) = payload.is_published {
        active_page.is_published = Set(is_published);
    }
    if let Some(hero_payload) = payload.hero_payload {
        active_page.hero_payload = Set(Some(hero_payload));
    }
    if let Some(blocks_payload) = payload.blocks_payload {
        active_page.blocks_payload = Set(Some(blocks_payload));
    }

    active_page.updated_at = Set(Utc::now());

    let updated = active_page.update(&db).await.map_err(|e| {
        tracing::error!("Error updating page: {:?}", e);
        (StatusCode::INTERNAL_SERVER_ERROR, "Failed to update page".into())
    })?;

    Ok(Json(updated))
}

pub async fn delete_page(
    Path(id): Path<Uuid>,
    State(db): State<DatabaseConnection>,
    Extension(_current_user): Extension<user::Model>,
    Extension(site_config): Extension<SiteConfig>,
) -> Result<StatusCode, (StatusCode, String)> {
    let page = AppPage::find()
        .filter(app_page::Column::Id.eq(id))
        .filter(app_page::Column::TenantId.eq(site_config.tenant_id))
        .one(&db)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .ok_or((StatusCode::NOT_FOUND, "Page not found".into()))?;

    page.delete(&db).await.map_err(|e| {
        tracing::error!("Error deleting page: {:?}", e);
        (StatusCode::INTERNAL_SERVER_ERROR, "Failed to delete page".into())
    })?;

    Ok(StatusCode::NO_CONTENT)
}
