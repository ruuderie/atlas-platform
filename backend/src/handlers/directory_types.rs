use axum::{
    extract::{Path, State}, http::StatusCode, response::IntoResponse, Json
};
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, DbErr, EntityTrait, PaginatorTrait, QueryFilter, Set,
};
use serde_json::json;
use uuid::Uuid;
use chrono::{DateTime, Utc};
use crate::entities::{category, directory, directory_type};
use crate::models::directory_type::{DirectoryTypeModel, CreateDirectoryType, UpdateDirectoryType};

pub async fn get_directory_types(
    State(db): State<DatabaseConnection>,
) -> Result<impl IntoResponse, StatusCode> {
    let directory_types = directory_type::Entity::find()
        .all(&db)
        .await
        .map_err(|err| {
            eprintln!("Database error: {:?}", err);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    let directory_type_models: Vec<DirectoryTypeModel> = directory_types
        .into_iter()
        .map(DirectoryTypeModel::from)
        .collect();

    Ok(Json(directory_type_models))
}

pub async fn get_directory_type(
    Path(directory_type_id): Path<Uuid>,
    State(db): State<DatabaseConnection>,
) -> Result<Json<DirectoryTypeModel>, (StatusCode, Json<serde_json::Value>)> {
    let directory_type = directory_type::Entity::find_by_id(directory_type_id)
        .one(&db)
        .await
        .map_err(|err| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": "Failed to fetch directory type", "details": err.to_string()})),
            )
        })?;

    if let Some(directory_type) = directory_type {
        Ok(Json(DirectoryTypeModel::from(directory_type)))
    } else {
        Err((StatusCode::NOT_FOUND, Json(json!({"error": "Directory type not found"}))))
    }
}

pub async fn create_directory_type(
    State(db): State<DatabaseConnection>,
    Json(payload): Json<CreateDirectoryType>
) -> Result<DirectoryTypeModel, StatusCode> {
    let new_directory_type = directory_type::ActiveModel {
        id: Set(Uuid::new_v4()),
        name: Set(payload.name),
        description: Set(payload.description),
        created_at: Set(Utc::now()),
        updated_at: Set(Utc::now()),
    };

    println!("Creating directory type: {:?}", new_directory_type);

    let directory_type = new_directory_type
        .insert(&db)
        .await
        .map_err(|err| {
            eprintln!("Error creating directory type: {:?}", err);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    println!("Directory type created: {:?}", directory_type);

    Ok(DirectoryTypeModel::from(directory_type))
}

pub async fn update_directory_type(
    Path(directory_type_id): Path<Uuid>,
    State(db): State<DatabaseConnection>,
    Json(payload): Json<UpdateDirectoryType>,
) -> Result<Json<DirectoryTypeModel>, StatusCode> {
    let mut directory_type: directory_type::ActiveModel = directory_type::Entity::find_by_id(directory_type_id)
        .one(&db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?
        .into();

    directory_type.name = Set(payload.name);
    directory_type.description = Set(payload.description);
    directory_type.updated_at = Set(Utc::now());

    let updated_directory_type = directory_type.update(&db).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(DirectoryTypeModel::from(updated_directory_type)))
}

pub async fn delete_directory_type(
    Path(directory_type_id): Path<Uuid>,
    State(db): State<DatabaseConnection>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    // Check if there are any associated directories or categories
    let directory_count = directory::Entity::find()
        .filter(directory::Column::DirectoryTypeId.eq(directory_type_id))
        .count(&db)
        .await
        .map_err(|err| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": "Failed to check for associated directories", "details": err.to_string()})),
            )
        })?;

    let category_count = category::Entity::find()
        .filter(category::Column::DirectoryTypeId.eq(directory_type_id))
        .count(&db)
        .await
        .map_err(|err| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": "Failed to check for associated categories", "details": err.to_string()})),
            )
        })?;

    if directory_count > 0 || category_count > 0 {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(json!({"error": "Cannot delete directory type with associated directories or categories"})),
        ));
    }

    let result = directory_type::Entity::delete_by_id(directory_type_id)
        .exec(&db)
        .await
        .map_err(|err| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": "Failed to delete directory type", "details": err.to_string()})),
            )
        })?;

    if result.rows_affected == 0 {
        return Err((
            StatusCode::NOT_FOUND,
            Json(json!({"error": "Directory type not found"})),
        ));
    }

    Ok(StatusCode::NO_CONTENT)
}