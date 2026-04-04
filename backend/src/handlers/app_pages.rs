use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
    routing::get,
    Router,
};
use sea_orm::{DatabaseConnection, EntityTrait, QueryFilter, ColumnTrait};
use crate::entities::app_page::{self, Entity as AppPage};
use uuid::Uuid;

pub fn public_routes(db: DatabaseConnection) -> Router<DatabaseConnection> {
    Router::new()
        .route("/api/public/pages/{tenant_id}", get(list_pages))
        .route("/api/public/pages/{tenant_id}/{slug}", get(get_page_by_slug))
        .with_state(db)
}

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
    let page = AppPage::find()
        .filter(app_page::Column::TenantId.eq(tenant_id))
        .filter(app_page::Column::Slug.eq(slug))
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
