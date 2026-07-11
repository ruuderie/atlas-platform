use axum::{
    Router,
    extract::{Extension, Json, Path, Query},
    http::StatusCode,
    response::{IntoResponse, Json as JsonResponse},
    routing::{delete, get, post, put},
};
use chrono::Utc;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, Condition, DatabaseConnection, EntityTrait, ModelTrait,
    QueryFilter, Set,
};
use std::collections::HashMap;
use uuid::Uuid;

use crate::entities::{note, user};
use crate::models::file::FileAssociation;
use crate::models::note::{CreateNoteInput, NoteModel, UpdateNoteInput};

pub fn routes() -> Router<DatabaseConnection> {
    Router::new()
        .route("/api/crm/notes", post(create_note))
        .route("/api/crm/notes", get(get_notes))
        .route("/api/crm/notes/{id}", get(get_note))
        .route("/api/crm/notes/{id}", put(update_note))
        .route("/api/crm/notes/{id}", delete(delete_note))
        .route("/api/crm/notes/{id}/files", get(get_note_files))
}

pub async fn get_user_tenant_id(
    db: &DatabaseConnection,
    user_id: Uuid,
) -> Result<Uuid, StatusCode> {
    let user_accounts = crate::entities::user_account::Entity::find()
        .filter(crate::entities::user_account::Column::UserId.eq(user_id))
        .all(db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let account_ids: Vec<Uuid> = user_accounts.into_iter().map(|ua| ua.account_id).collect();

    let profile = crate::entities::profile::Entity::find()
        .filter(crate::entities::profile::Column::AccountId.is_in(account_ids))
        .one(db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::FORBIDDEN)?;

    Ok(profile.tenant_id)
}

async fn create_note(
    Extension(db): Extension<DatabaseConnection>,
    Extension(current_user): Extension<user::Model>,
    Json(input): Json<CreateNoteInput>,
) -> Result<impl IntoResponse, StatusCode> {
    let tenant_id = get_user_tenant_id(&db, current_user.id).await?;

    let new_note = note::ActiveModel {
        id: Set(Uuid::new_v4()),
        content: Set(input.content),
        created_by: Set(current_user.id),
        entity_type: Set(input.entity_type),
        entity_id: Set(input.entity_id),
        tenant_id: Set(Some(tenant_id)),
        is_private: Set(input.is_private),
        created_at: Set(Utc::now()),
        updated_at: Set(Utc::now()),
    };

    let note_inserted = new_note
        .insert(&db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Associate files with the note, auto-inserting the File record if needed
    for file in input.files {
        let existing = crate::entities::file::Entity::find_by_id(file.id.to_string())
            .one(&db)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        if existing.is_none() {
            let new_file_db = crate::entities::file::ActiveModel {
                id: Set(file.id.to_string()),
                name: Set(file.name.clone()),
                size: Set(0),
                mime_type: Set("application/octet-stream".to_string()),
                hash_sha256: Set("".to_string()),
                storage_type: Set(crate::entities::file::StorageType::S3),
                storage_path: Set(file.storage_path.clone()),
                views: Set(0),
                downloads: Set(0),
                bandwidth_used: Set(0),
                bandwidth_used_paid: Set(0),
                date_upload: Set(Utc::now().into()),
                date_last_view: Set(None),
                is_anonymous: Set(false),
                user_id: Set(Some(current_user.id.to_string())),
            };
            new_file_db
                .insert(&db)
                .await
                .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        }
        note_inserted
            .add_file(&db, file.id)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    }

    let mut model = NoteModel::from(note_inserted.clone());
    model.files = note_inserted
        .get_associated_files(&db)
        .await
        .unwrap_or_default();

    Ok((StatusCode::CREATED, JsonResponse(model)))
}

async fn get_notes(
    Extension(db): Extension<DatabaseConnection>,
    Extension(current_user): Extension<user::Model>,
    Query(params): Query<HashMap<String, String>>,
) -> Result<impl IntoResponse, StatusCode> {
    let tenant_id = get_user_tenant_id(&db, current_user.id).await?;

    // Setup base query scoped to tenant and privacy rules:
    // Only show private notes if they belong to current_user
    let privacy_cond = Condition::any()
        .add(note::Column::IsPrivate.eq(false))
        .add(note::Column::CreatedBy.eq(current_user.id));

    let mut query = note::Entity::find()
        .filter(note::Column::TenantId.eq(tenant_id))
        .filter(privacy_cond);

    // Apply polymorphic entity filters
    if let Some(entity_type) = params.get("entity_type") {
        query = query.filter(note::Column::EntityType.eq(entity_type));
    }
    if let Some(entity_id_str) = params.get("entity_id") {
        if let Ok(entity_id) = Uuid::parse_str(entity_id_str) {
            query = query.filter(note::Column::EntityId.eq(entity_id));
        }
    }

    let notes = query
        .all(&db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let mut note_models = Vec::new();
    for note_item in notes {
        let mut model = NoteModel::from(note_item.clone());
        model.files = note_item
            .get_associated_files(&db)
            .await
            .unwrap_or_default();
        note_models.push(model);
    }

    Ok(JsonResponse(note_models))
}

async fn get_note(
    Extension(db): Extension<DatabaseConnection>,
    Extension(current_user): Extension<user::Model>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, StatusCode> {
    let tenant_id = get_user_tenant_id(&db, current_user.id).await?;

    let note_item = note::Entity::find_by_id(id)
        .one(&db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;

    if note_item.tenant_id != Some(tenant_id) {
        return Err(StatusCode::NOT_FOUND);
    }

    if note_item.is_private && note_item.created_by != current_user.id {
        return Err(StatusCode::FORBIDDEN);
    }

    let mut model = NoteModel::from(note_item.clone());
    model.files = note_item
        .get_associated_files(&db)
        .await
        .unwrap_or_default();

    Ok(JsonResponse(model))
}

async fn update_note(
    Extension(db): Extension<DatabaseConnection>,
    Extension(current_user): Extension<user::Model>,
    Path(id): Path<Uuid>,
    Json(input): Json<UpdateNoteInput>,
) -> Result<impl IntoResponse, StatusCode> {
    let tenant_id = get_user_tenant_id(&db, current_user.id).await?;

    let note_item = note::Entity::find_by_id(id)
        .one(&db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;

    if note_item.tenant_id != Some(tenant_id) {
        return Err(StatusCode::NOT_FOUND);
    }

    if note_item.created_by != current_user.id {
        return Err(StatusCode::FORBIDDEN);
    }

    let mut note_active: note::ActiveModel = note_item.clone().into();
    note_active.content = Set(input.content);
    if let Some(is_private) = input.is_private {
        note_active.is_private = Set(is_private);
    }
    note_active.updated_at = Set(Utc::now());

    let updated_note = note_active
        .update(&db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Handle files updating if provided
    if let Some(file_ids) = input.files {
        // Disassociate previous files
        let current_files = note_item
            .get_associated_files(&db)
            .await
            .unwrap_or_default();
        for f in current_files {
            note_item
                .remove_file(&db, f.id)
                .await
                .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        }
        // Associate new files
        for fid in file_ids {
            note_item
                .add_file(&db, fid)
                .await
                .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        }
    }

    let mut model = NoteModel::from(updated_note.clone());
    model.files = updated_note
        .get_associated_files(&db)
        .await
        .unwrap_or_default();

    Ok(JsonResponse(model))
}

async fn delete_note(
    Extension(db): Extension<DatabaseConnection>,
    Extension(current_user): Extension<user::Model>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, StatusCode> {
    let tenant_id = get_user_tenant_id(&db, current_user.id).await?;

    let note_item = note::Entity::find_by_id(id)
        .one(&db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;

    if note_item.tenant_id != Some(tenant_id) {
        return Err(StatusCode::NOT_FOUND);
    }

    if note_item.created_by != current_user.id {
        return Err(StatusCode::FORBIDDEN);
    }

    note_item
        .delete(&db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(StatusCode::NO_CONTENT)
}

async fn get_note_files(
    Extension(db): Extension<DatabaseConnection>,
    Extension(current_user): Extension<user::Model>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, StatusCode> {
    let tenant_id = get_user_tenant_id(&db, current_user.id).await?;

    let note_item = note::Entity::find_by_id(id)
        .one(&db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;

    if note_item.tenant_id != Some(tenant_id) {
        return Err(StatusCode::NOT_FOUND);
    }

    if note_item.is_private && note_item.created_by != current_user.id {
        return Err(StatusCode::FORBIDDEN);
    }

    let files = note_item
        .get_associated_files(&db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(JsonResponse(files))
}
