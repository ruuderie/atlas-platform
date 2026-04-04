use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
    routing::get,
    Router,
};
use sea_orm::{DatabaseConnection, EntityTrait, QueryFilter, ColumnTrait, QueryOrder};
use crate::entities::app_menu::{self, Entity as AppMenu};
use uuid::Uuid;

pub fn public_routes(db: DatabaseConnection) -> Router<DatabaseConnection> {
    Router::new()
        .route("/api/public/menus/{tenant_id}", get(list_menus))
        .route("/api/public/menus/{tenant_id}/tree/{menu_type}", get(get_menu_tree))
        .with_state(db)
}

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
