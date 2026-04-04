use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use sea_orm::{DatabaseConnection, EntityTrait, Set, ActiveModelTrait, QueryFilter, ColumnTrait};
use uuid::Uuid;
use crate::entities::{file::{self, Entity as File}, file_association};
use crate::models::file::{FileModel, CreateFileInput, UpdateFileInput};

pub async fn create_file(
    State(db): State<DatabaseConnection>,
    Json(input): Json<CreateFileInput>,
) -> Result<impl IntoResponse, StatusCode> {
    let new_file = file::ActiveModel {
        id: Set(Uuid::new_v4().to_string()),
        name: Set(input.name),
        size: Set(input.size),
        mime_type: Set(input.mime_type),
        hash_sha256: Set(input.hash_sha256),
        storage_type: Set(input.storage_type.parse().map_err(|_| StatusCode::BAD_REQUEST)?),
        storage_path: Set(input.storage_path),
        views: Set(0),
        downloads: Set(0),
        bandwidth_used: Set(0),
        bandwidth_used_paid: Set(0),
        date_upload: Set(chrono::Utc::now().into()),
        date_last_view: Set(None),
        is_anonymous: Set(input.is_anonymous),
        user_id: Set(input.user_id.map(|id| id.to_string())),
    };

    let file = new_file.insert(&db).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok((StatusCode::CREATED, Json(FileModel::from(file))))
}

pub async fn update_file(
    State(db): State<DatabaseConnection>,
    Path(id): Path<Uuid>,
    Json(input): Json<UpdateFileInput>,
) -> Result<impl IntoResponse, StatusCode> {
    let mut file: file::ActiveModel = File::find_by_id(id.to_string())
        .one(&db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?
        .into();

    if let Some(name) = input.name {
        file.name = Set(name);
    }
    if let Some(views) = input.views {
        file.views = Set(views);
    }
    if let Some(downloads) = input.downloads {
        file.downloads = Set(downloads);
    }
    if let Some(bandwidth_used) = input.bandwidth_used {
        file.bandwidth_used = Set(bandwidth_used);
    }
    if let Some(bandwidth_used_paid) = input.bandwidth_used_paid {
        file.bandwidth_used_paid = Set(bandwidth_used_paid);
    }
    if let Some(date_last_view) = input.date_last_view {
        file.date_last_view = Set(Some(date_last_view.into()));
    }

    let updated_file = file.update(&db).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok((StatusCode::OK, Json(FileModel::from(updated_file))))
}

pub async fn get_file(
    State(db): State<DatabaseConnection>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, StatusCode> {
    let file = File::find_by_id(id.to_string())
        .one(&db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;

    Ok((StatusCode::OK, Json(FileModel::from(file))))
}

pub async fn get_file_info(
    State(db): State<DatabaseConnection>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, StatusCode> {
    let file = File::find_by_id(id.to_string())
        .one(&db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;

    Ok((StatusCode::OK, Json(FileModel::from(file))))
}

pub async fn get_file_thumbnail(
    State(db): State<DatabaseConnection>,
    Path(id): Path<Uuid>,
    Query(_params): Query<std::collections::HashMap<String, String>>,
) -> Result<impl IntoResponse, StatusCode> {
    // Implement thumbnail generation logic here
    // For now, we'll just return the file info
    let file = File::find_by_id(id.to_string())
        .one(&db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;

    Ok((StatusCode::OK, Json(FileModel::from(file))))
}

pub async fn delete_file(
    State(db): State<DatabaseConnection>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, StatusCode> {
    let result = File::delete_by_id(id.to_string())
        .exec(&db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    if result.rows_affected == 0 {
        return Err(StatusCode::NOT_FOUND);
    }

    Ok(StatusCode::NO_CONTENT)
}

pub async fn get_user_files(
    State(db): State<DatabaseConnection>,
    Path(user_id): Path<Uuid>,
) -> Result<impl IntoResponse, StatusCode> {
    let files = File::find()
        .filter(file::Column::UserId.eq(user_id.to_string()))
        .all(&db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let file_models: Vec<FileModel> = files.into_iter().map(FileModel::from).collect();
    Ok((StatusCode::OK, Json(file_models)))
}
/* 

Function to Get User Lists  after it has been implemented for user object

pub async fn get_user_lists(
    State(db): State<DatabaseConnection>,
    Path(user_id): Path<Uuid>,
) -> Result<impl IntoResponse, StatusCode> {
    // Implement logic to retrieve file lists for the user
    // This could involve fetching file associations or custom lists
    todo!("Implement get_user_lists")
}

*/

pub async fn associate_file(
    State(db): State<DatabaseConnection>,
    Path((file_id, entity_type, entity_id)): Path<(Uuid, String, Uuid)>,
) -> Result<impl IntoResponse, StatusCode> {
    let file_association = file_association::ActiveModel {
        id: Set(Uuid::new_v4()),
        file_id: Set(file_id.to_string()),
        associated_entity_type: Set(entity_type),
        associated_entity_id: Set(entity_id),
    };

    file_association.insert(&db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(StatusCode::CREATED)
}

pub async fn disassociate_file(
    State(db): State<DatabaseConnection>,
    Path((file_id, entity_type, entity_id)): Path<(Uuid, String, Uuid)>,
) -> Result<impl IntoResponse, StatusCode> {
    let result = file_association::Entity::delete_many()
        .filter(file_association::Column::FileId.eq(file_id.to_string()))
        .filter(file_association::Column::AssociatedEntityType.eq(entity_type))
        .filter(file_association::Column::AssociatedEntityId.eq(entity_id))
        .exec(&db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    if result.rows_affected == 0 {
        return Err(StatusCode::NOT_FOUND);
    }

    Ok(StatusCode::NO_CONTENT)
}

pub async fn get_associated_files(
    State(db): State<DatabaseConnection>,
    Path((entity_type, entity_id)): Path<(String, Uuid)>,
) -> Result<impl IntoResponse, StatusCode> {
    let associations = file_association::Entity::find()
        .filter(file_association::Column::AssociatedEntityType.eq(entity_type))
        .filter(file_association::Column::AssociatedEntityId.eq(entity_id))
        .all(&db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let file_ids: Vec<String> = associations.into_iter().map(|a| a.file_id).collect();
    let files = File::find()
        .filter(file::Column::Id.is_in(file_ids))
        .all(&db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let file_models: Vec<FileModel> = files.into_iter().map(FileModel::from).collect();
    Ok((StatusCode::OK, Json(file_models)))
}
