use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use sea_orm::{DatabaseConnection, EntityTrait, Set,  ActiveModelTrait};
use serde_json::json;
use uuid::Uuid;
use chrono::Utc;
use crate::entities::category; 
use crate::models::category::{CategoryModel, CreateCategory, UpdateCategory}; 


pub async fn get_categories(
    // Potentially add query parameters for filtering or pagination here
    State(db): State<DatabaseConnection>,
) -> Result<Json<Vec<CategoryModel>>, (StatusCode, Json<serde_json::Value>)> {
    let categories = category::Entity::find()
        .all(&db)
        .await
        .map_err(|err| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": "Failed to fetch categories", "details": err.to_string()})),
            )
        })?;

    let category_models: Vec<CategoryModel> = categories
        .into_iter()
        .map(CategoryModel::from) 
        .collect();

    Ok(Json(category_models))
}

pub async fn get_category(
    Path(category_id): Path<Uuid>,
    State(db): State<DatabaseConnection>,
) -> Result<Json<CategoryModel>, (StatusCode, Json<serde_json::Value>)> {
    let category = category::Entity::find_by_id(category_id)
        .one(&db)
        .await
        .map_err(|err| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": "Failed to fetch category", "details": err.to_string()})),
            )
        })?;

    if let Some(category) = category {
        Ok(Json(CategoryModel::from(category)))
    } else {
        Err((StatusCode::NOT_FOUND, Json(json!({"error": "Category not found"}))))
    }
}

pub async fn create_category(
    State(db): State<DatabaseConnection>,
    Json(payload): Json<CreateCategory>, 
) -> Result<Json<CategoryModel>, (StatusCode, Json<serde_json::Value>)> {

    let new_category = category::ActiveModel {
        id: Set(Uuid::new_v4()),
        directory_type_id: Set(payload.directory_type_id),
        parent_category_id: Set(payload.parent_category_id),
        name: Set(payload.name),
        description: Set(payload.description),
        is_custom: Set(payload.is_custom),
        is_active: Set(payload.is_active),
        created_at: Set(Utc::now()),
        updated_at: Set(Utc::now()),
    };

    let insert_result = category::Entity::insert(new_category)
        .exec(&db)
        .await
        .map_err(|err| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": "Failed to create category", "details": err.to_string()})),
            )
        })?;

    let category = category::Entity::find_by_id(insert_result.last_insert_id)
        .one(&db)
        .await
        .map_err(|err| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": "Failed to fetch created category", "details": err.to_string()})),
            )
        })?
        .ok_or_else(|| (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": "Created category not found"}))))?;

    Ok(Json(CategoryModel::from(category)))
}

pub async fn update_category(
    State(db): State<DatabaseConnection>,
    Path(category_id): Path<Uuid>,
    Json(payload): Json<UpdateCategory>,
) -> Result<Json<CategoryModel>, (StatusCode, Json<serde_json::Value>)> {


    let category: category::ActiveModel = category::Entity::find_by_id(category_id)
        .one(&db)
        .await
        .map_err(|err| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": "Failed to fetch category for update", "details": err.to_string()})),
            )
        })?
        .ok_or_else(|| (StatusCode::NOT_FOUND, Json(json!({"error": "Category not found"}))))?
        .into();


    let updated_category = category.update(&db).await.map_err(|err| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": "Failed to update category", "details": err.to_string()})),
        )
    })?;

    Ok(Json(CategoryModel::from(updated_category)))
}

pub async fn delete_category(
    Path(category_id): Path<Uuid>,
    State(db): State<DatabaseConnection>,
) -> Result<StatusCode, (StatusCode, Json<serde_json::Value>)> {

    let result = category::Entity::delete_by_id(category_id)
        .exec(&db)
        .await
        .map_err(|err| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": "Failed to delete category", "details": err.to_string()})),
            )
        })?;

    if result.rows_affected == 0 {
        Err((StatusCode::NOT_FOUND, Json(json!({"error": "Category not found"}))))
    } else {
        Ok(StatusCode::NO_CONTENT)
    }
}