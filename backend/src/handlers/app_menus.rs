#![allow(dead_code)]
use crate::entities::app_menu::{self, Entity as AppMenu};
use axum::{
    Json, Router,
    extract::{Path, State},
    http::StatusCode,
    routing::{delete, get, post, put},
};
use chrono::Utc;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, QueryOrder, Set,
};
use serde::Deserialize;
use uuid::Uuid;

// ── Request / Response types ──────────────────────────────────────────────────

#[derive(Deserialize)]
pub struct CreateMenuPayload {
    pub menu_type: String,
    pub label: String,
    pub href: Option<String>,
    pub parent_id: Option<Uuid>,
    pub display_order: Option<i32>,
    pub is_visible: Option<bool>,
}

#[derive(Deserialize)]
pub struct UpdateMenuPayload {
    pub label: Option<String>,
    pub href: Option<String>,
    pub parent_id: Option<Uuid>,
    pub display_order: Option<i32>,
    pub is_visible: Option<bool>,
}

// ── Route constructors ────────────────────────────────────────────────────────

/// State-free public route definitions.
/// Use inside `AtlasApp::public_router()`. Never call `.with_state()` here.
pub fn public_routes_raw() -> Router<DatabaseConnection> {
    Router::new()
        .route("/api/public/menus/{tenant_id}", get(list_menus))
        .route(
            "/api/public/menus/{tenant_id}/tree/{menu_type}",
            get(get_menu_tree),
        )
}

/// State-free authenticated CRUD route definitions.
/// Use inside `AtlasApp::authenticated_router()`. Never call `.with_state()` here.
pub fn authenticated_routes_raw() -> Router<DatabaseConnection> {
    Router::new()
        // List all menus for a tenant (all types, all visibility)
        .route("/api/menus/{tenant_id}", get(list_all_menus))
        // Create a menu item
        .route("/api/menus/{tenant_id}", post(create_menu))
        // Update a menu item by id
        .route("/api/menus/{tenant_id}/{menu_id}", put(update_menu))
        // Delete a menu item by id
        .route("/api/menus/{tenant_id}/{menu_id}", delete(delete_menu))
}

/// Legacy state-finalized constructor. Used by api.rs during transition period.
/// Remove after CorePlatformApp is active and api.rs is cleaned up (Phase 3).
pub fn public_routes(db: DatabaseConnection) -> Router<DatabaseConnection> {
    public_routes_raw().with_state(db)
}

// ── Public handlers ───────────────────────────────────────────────────────────

pub async fn list_menus(
    Path(tenant_id): Path<Uuid>,
    State(db): State<DatabaseConnection>,
) -> Result<Json<Vec<app_menu::Model>>, StatusCode> {
    let menus = AppMenu::find()
        .filter(app_menu::Column::TenantId.eq(tenant_id))
        .filter(app_menu::Column::IsVisible.eq(true))
        .order_by_asc(app_menu::Column::DisplayOrder)
        .all(&db)
        .await
        .map_err(|err| {
            tracing::error!("Database error fetching menus: {:?}", err);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    Ok(Json(menus))
}

pub async fn get_menu_tree(
    Path((tenant_id, menu_type)): Path<(Uuid, String)>,
    State(db): State<DatabaseConnection>,
) -> Result<Json<Vec<app_menu::Model>>, StatusCode> {
    let menus = AppMenu::find()
        .filter(app_menu::Column::TenantId.eq(tenant_id))
        .filter(app_menu::Column::MenuType.eq(menu_type))
        .filter(app_menu::Column::IsVisible.eq(true))
        .order_by_asc(app_menu::Column::DisplayOrder)
        .all(&db)
        .await
        .map_err(|err| {
            tracing::error!("Database error fetching menu tree: {:?}", err);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    Ok(Json(menus))
}

// ── Authenticated CRUD handlers ───────────────────────────────────────────────

/// Lists ALL menus for a tenant (all types + hidden items). Platform-admin only.
pub async fn list_all_menus(
    Path(tenant_id): Path<Uuid>,
    State(db): State<DatabaseConnection>,
) -> Result<Json<Vec<app_menu::Model>>, StatusCode> {
    let menus = AppMenu::find()
        .filter(app_menu::Column::TenantId.eq(tenant_id))
        .order_by_asc(app_menu::Column::MenuType)
        .order_by_asc(app_menu::Column::DisplayOrder)
        .all(&db)
        .await
        .map_err(|e| {
            tracing::error!("list_all_menus error: {:?}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    Ok(Json(menus))
}

/// Creates a new menu item for a tenant.
pub async fn create_menu(
    Path(tenant_id): Path<Uuid>,
    State(db): State<DatabaseConnection>,
    Json(payload): Json<CreateMenuPayload>,
) -> Result<(StatusCode, Json<app_menu::Model>), (StatusCode, String)> {
    let now = Utc::now();
    let new_menu = app_menu::ActiveModel {
        id: Set(Uuid::new_v4()),
        tenant_id: Set(tenant_id),
        menu_type: Set(payload.menu_type),
        label: Set(payload.label),
        href: Set(payload.href),
        parent_id: Set(payload.parent_id),
        display_order: Set(payload.display_order.unwrap_or(0)),
        is_visible: Set(payload.is_visible.unwrap_or(true)),
        created_at: Set(now),
        updated_at: Set(now),
    };

    let inserted = new_menu.insert(&db).await.map_err(|e| {
        tracing::error!("create_menu error: {:?}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Failed to create menu item".to_string(),
        )
    })?;

    Ok((StatusCode::CREATED, Json(inserted)))
}

/// Updates a menu item by id.
pub async fn update_menu(
    Path((tenant_id, menu_id)): Path<(Uuid, Uuid)>,
    State(db): State<DatabaseConnection>,
    Json(payload): Json<UpdateMenuPayload>,
) -> Result<Json<app_menu::Model>, (StatusCode, String)> {
    let existing = AppMenu::find_by_id(menu_id)
        .filter(app_menu::Column::TenantId.eq(tenant_id))
        .one(&db)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .ok_or_else(|| (StatusCode::NOT_FOUND, "Menu item not found".to_string()))?;

    let mut active: app_menu::ActiveModel = existing.into();
    if let Some(l) = payload.label {
        active.label = Set(l);
    }
    if let Some(h) = payload.href {
        active.href = Set(Some(h));
    }
    if let Some(p) = payload.parent_id {
        active.parent_id = Set(Some(p));
    }
    if let Some(o) = payload.display_order {
        active.display_order = Set(o);
    }
    if let Some(v) = payload.is_visible {
        active.is_visible = Set(v);
    }
    active.updated_at = Set(Utc::now());

    let updated = active
        .update(&db)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(updated))
}

/// Deletes a menu item by id (and cascades to children via DB FK).
pub async fn delete_menu(
    Path((tenant_id, menu_id)): Path<(Uuid, Uuid)>,
    State(db): State<DatabaseConnection>,
) -> Result<StatusCode, (StatusCode, String)> {
    let existing = AppMenu::find_by_id(menu_id)
        .filter(app_menu::Column::TenantId.eq(tenant_id))
        .one(&db)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .ok_or_else(|| (StatusCode::NOT_FOUND, "Menu item not found".to_string()))?;

    let active: app_menu::ActiveModel = existing.into();
    active
        .delete(&db)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(StatusCode::NO_CONTENT)
}
