use axum::{
    extract::{Extension, Path, Json},
    http::StatusCode,
    response::{IntoResponse, Json as JsonResponse},
    routing::{get, post, put, delete},
    Router,
};
use sea_orm::{
    DatabaseConnection, EntityTrait, QueryFilter, Set, ColumnTrait,
    ActiveModelTrait, ModelTrait,
};
use uuid::Uuid;
use chrono::Utc;

use crate::entities::{note, user};
use crate::models::note::{NoteModel, CreateNoteInput, UpdateNoteInput};
use crate::models::file::FileAssociation;

pub fn routes() -> Router {
    Router::new()
        .route("/notes", post(create_note))
        .route("/notes", get(get_notes))
        .route("/notes/{id}", get(get_note))
        .route("/notes/{id}", put(update_note))
        .route("/notes/{id}", delete(delete_note))
        .route("/notes/{id}/files", get(get_note_files))
}

async fn create_note(
    Extension(db): Extension<DatabaseConnection>,
    Extension(current_user): Extension<user::Model>,
    Json(input): Json<CreateNoteInput>,
) -> Result<impl IntoResponse, StatusCode> {
    let new_note = note::ActiveModel {
        id: Set(Uuid::new_v4()),
        content: Set(input.content),
        created_by: Set(current_user.id),
        entity_type: Set(input.entity_type),
        entity_id: Set(input.entity_id),
        created_at: Set(Utc::now()),
        updated_at: Set(Utc::now()),
    };

    let note = new_note.insert(&db).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    Ok((StatusCode::CREATED, JsonResponse(NoteModel::from(note))))
}

async fn get_notes(
    Extension(db): Extension<DatabaseConnection>,
) -> Result<impl IntoResponse, StatusCode> {
    let notes = note::Entity::find()
        .all(&db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let note_models: Vec<NoteModel> = notes.into_iter().map(NoteModel::from).collect();
    Ok(JsonResponse(note_models))
}

async fn get_note(
    Extension(db): Extension<DatabaseConnection>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, StatusCode> {
    let note = note::Entity::find_by_id(id)
        .one(&db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;

    Ok(JsonResponse(NoteModel::from(note)))
}

async fn update_note(
    Extension(db): Extension<DatabaseConnection>,
    Path(id): Path<Uuid>,
    Json(input): Json<UpdateNoteInput>,
) -> Result<impl IntoResponse, StatusCode> {
    let mut note: note::ActiveModel = note::Entity::find_by_id(id)
        .one(&db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?
        .into();

    note.content = Set(input.content);
    note.updated_at = Set(Utc::now());


    let updated_note = note.update(&db).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(JsonResponse(NoteModel::from(updated_note)))
}

async fn delete_note(
    Extension(db): Extension<DatabaseConnection>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, StatusCode> {
    let note = note::Entity::find_by_id(id)
        .one(&db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;

    note.delete(&db).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(StatusCode::NO_CONTENT)
}

async fn get_note_files(
    Extension(db): Extension<DatabaseConnection>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, StatusCode> {
    let note = note::Entity::find_by_id(id)
        .one(&db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;

    let files = note.get_associated_files(&db).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(JsonResponse(files))
}
