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
) -> Result<(StatusCode, Json<CategoryModel>), (StatusCode, Json<serde_json::Value>)> {
    println!("TEST LOG: from create_category and payload: {:?}", payload);
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
    println!("TEST LOG: from create_category and new_category: {:?}", new_category);
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

    Ok((StatusCode::CREATED, Json(CategoryModel::from(category))))
}

pub async fn update_category(
    State(db): State<DatabaseConnection>,
    Path(category_id): Path<Uuid>,
    Json(payload): Json<UpdateCategory>,
) -> Result<Json<CategoryModel>, (StatusCode, Json<serde_json::Value>)> {
    let category_result = category::Entity::find_by_id(category_id)
        .one(&db)
        .await
        .map_err(|err| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": "Failed to fetch category for update", "details": err.to_string()})),
            )
        })?;

    let category = match category_result {
        Some(c) => c,
        None => return Err((StatusCode::NOT_FOUND, Json(json!({"error": "Category not found"})))),
    };

    // Create an ActiveModel and apply the updates from the payload
    let mut category_active: category::ActiveModel = category.into();
    
    // Apply updates from payload
    if let Some(name) = payload.name {
        category_active.name = Set(name);
    }
    if let Some(description) = payload.description {
        category_active.description = Set(description);
    }
    if let Some(directory_type_id) = payload.directory_type_id {
        category_active.directory_type_id = Set(directory_type_id);
    }
    if let Some(parent_category_id) = payload.parent_category_id {
        category_active.parent_category_id = Set(Some(parent_category_id));
    }
    if let Some(is_custom) = payload.is_custom {
        category_active.is_custom = Set(is_custom);
    }
    if let Some(is_active) = payload.is_active {
        category_active.is_active = Set(is_active);
    }
    
    // Update the timestamp
    category_active.updated_at = Set(Utc::now());

    let updated_category = category_active.update(&db).await.map_err(|err| {
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