use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
    response::IntoResponse,
    routing::{get, post, put, delete},
    Router,
};
use sea_orm::{DatabaseConnection, EntityTrait, QueryFilter, ColumnTrait, Set, ActiveModelTrait};
use crate::entities::directory::{self, Entity as Directory};
use crate::models::directory::{DirectoryModel, CreateDirectory, UpdateDirectory};
use chrono::Utc;
use uuid::Uuid;

pub fn public_routes(db: DatabaseConnection) -> Router<DatabaseConnection> {
    Router::new()
        .route("/directories", get(get_directories))
        .route("/directories/:id", get(get_directory_by_id))
        .route("/directories/type/:type_id", get(get_directories_by_type))
        .with_state(db)
}

pub fn authenticated_routes(db: DatabaseConnection) -> Router<DatabaseConnection> {
    Router::new()
        .route("/directories", post(create_directory))
        .route("/directories/:id", put(update_directory))
        .route("/directories/:id", delete(delete_directory))
        .with_state(db)
}

pub async fn get_directories(
    State(db): State<DatabaseConnection>,
) -> Result<(StatusCode, Json<Vec<DirectoryModel>>), StatusCode> {
    let directories = Directory::find()
        .all(&db)
        .await
        .map_err(|err| {
            tracing::error!("Database error: {:?}", err);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    let directory_models: Vec<DirectoryModel> = directories
        .into_iter()
        .map(DirectoryModel::from)
        .collect();

    Ok((StatusCode::OK, Json(directory_models)))
}

// The rest of your handlers (get_directory_by_id, get_directories_by_type, etc.) are already correct
pub async fn get_directory_by_id(
    Path(directory_id): Path<Uuid>,
    State(db): State<DatabaseConnection>,
) -> Result<(StatusCode, Json<DirectoryModel>), StatusCode> {
    let directory = directory::Entity::find_by_id(directory_id)
        .one(&db)
        .await
        .map_err(|err| {
            tracing::error!("Failed to fetch directory: {:?}", err);
            StatusCode::INTERNAL_SERVER_ERROR
        })?
        .ok_or(StatusCode::NOT_FOUND)?;

    Ok((StatusCode::OK, Json(DirectoryModel::from(directory))))
}

pub async fn get_directories_by_type(
    Path(directory_type_id): Path<Uuid>,
    State(db): State<DatabaseConnection>,
) -> Result<(StatusCode, Json<Vec<DirectoryModel>>), StatusCode> {
    let directories = directory::Entity::find()
        .filter(directory::Column::DirectoryTypeId.eq(directory_type_id))
        .all(&db)
        .await
        .map_err(|err| {
            tracing::error!("Database error: {:?}", err);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    let directory_models: Vec<DirectoryModel> = directories
        .into_iter()
        .map(DirectoryModel::from)
        .collect();

    Ok((StatusCode::OK, Json(directory_models)))
}

pub async fn create_directory(
    State(db): State<DatabaseConnection>,
    Json(input): Json<CreateDirectory>,
) -> Result<(StatusCode, Json<DirectoryModel>), StatusCode> {
    let new_directory = directory::ActiveModel {
        id: Set(Uuid::new_v4()),
        name: Set(input.name),
        description: Set(input.description),
        directory_type_id: Set(input.directory_type_id),
        domain: Set(input.domain),
        created_at: Set(Utc::now()),
        updated_at: Set(Utc::now()),
    };

    let directory = new_directory
        .insert(&db)
        .await
        .map_err(|err| {
            tracing::error!("Error creating directory: {:?}", err);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;
    println!("TEST LOG: from create_directory and directory: {:?}", directory);

    Ok((StatusCode::CREATED, Json(DirectoryModel::from(directory))))
}

pub async fn update_directory(
    Path(directory_id): Path<Uuid>,
    State(db): State<DatabaseConnection>,
    Json(input): Json<UpdateDirectory>,
) -> Result<(StatusCode, Json<DirectoryModel>), StatusCode> {
    let mut directory = directory::Entity::find_by_id(directory_id)
        .one(&db)
        .await
        .map_err(|err| {
            tracing::error!("Error fetching directory: {:?}", err);
            StatusCode::INTERNAL_SERVER_ERROR
        })?
        .ok_or(StatusCode::NOT_FOUND)?;

    let mut active_directory: directory::ActiveModel = directory.clone().into();
    if let Some(name) = input.name {
        active_directory.name = Set(name);
    }
    if let Some(description) = input.description {
        active_directory.description = Set(description);
    }
    if let Some(directory_type_id) = input.directory_type_id {
        active_directory.directory_type_id = Set(directory_type_id);
    }
    if let Some(domain) = input.domain {
        active_directory.domain = Set(domain);
    }
    active_directory.updated_at = Set(Utc::now());

    directory = active_directory
        .update(&db)
        .await
        .map_err(|err| {
            tracing::error!("Error updating directory: {:?}", err);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    Ok((StatusCode::OK, Json(DirectoryModel::from(directory))))
}

pub async fn delete_directory(
    Path(directory_id): Path<Uuid>,
    State(db): State<DatabaseConnection>,
) -> Result<StatusCode, StatusCode> {
    directory::Entity::delete_by_id(directory_id)
        .exec(&db)
        .await
        .map_err(|err| {
            tracing::error!("Error deleting directory: {:?}", err);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    Ok(StatusCode::NO_CONTENT)
}