use axum::{
    extract::{Extension, Path, Json, State},
    http::StatusCode,
    response::{IntoResponse, Json as JsonResponse},
    routing::{get, post, put, delete},
    Router,
};
use sea_orm::{
    DatabaseConnection, EntityTrait, Set,
    ActiveModelTrait, ModelTrait,
};
use uuid::Uuid;
use chrono::Utc;

use crate::entities::{deal, customer, contact, user, note, activity};
use crate::models::deal::{DealModel, CreateDealInput, UpdateDealInput};
use crate::models::file::FileAssociation;
use crate::models::note::{NoteModel, CreateNoteInput};
use crate::models::activity::{ActivityModel, CreateActivityInput};
use crate::models::contact::{Contact as ContactModel};

pub fn routes() -> Router<DatabaseConnection> {
    Router::new()
        .route("/api/deals", post(create_deal))
        .route("/api/deals", get(get_deals))
        .route("/api/deals/{id}", get(get_deal))
        .route("/api/deals/{id}", put(update_deal))
        .route("/api/deals/{id}", delete(delete_deal))
        .route("/api/deals/{deal_id}/files/{file_id}", post(add_file_to_deal))
        .route("/api/deals/{id}/files", get(get_deal_files))
        .route("/api/deals/{id}/contacts", get(get_deal_contacts))
        .route("/api/deals/{id}/contacts/{contact_id}", post(add_contact_to_deal))
        .route("/api/deals/{id}/contacts/{contact_id}", delete(remove_contact_from_deal))
        .route("/api/deals/{id}/notes", post(create_deal_note))
        .route("/api/deals/{id}/notes", get(get_deal_notes))
        .route("/api/deals/{id}/activities", post(create_deal_activity))
        .route("/api/deals/{id}/activities", get(get_deal_activities))
}

pub async fn create_deal(
    Extension(db): Extension<DatabaseConnection>,
    Json(input): Json<CreateDealInput>,
) -> Result<impl IntoResponse, StatusCode> {
    // Check if the customer exists
    let customer = customer::Entity::find_by_id(input.customer_id)
        .one(&db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;

    let new_deal = deal::ActiveModel {
        id: Set(Uuid::new_v4()),
        customer_id: Set(customer.id),
        name: Set(input.name),
        amount: Set(input.amount),
        status: Set(input.status),
        stage: Set(input.stage),
        close_date: Set(input.close_date),
        is_active: Set(true),
        created_at: Set(Utc::now()),
        updated_at: Set(Utc::now()),
        tenant_id: Set(None),
        properties: Set(None),
    };

    let deal = new_deal.insert(&db).await.map_err(|e| {
        eprintln!("Failed to insert deal DB ERROR: {:?}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;
    
    Ok((StatusCode::CREATED, JsonResponse(DealModel::from(deal))))
}

pub async fn get_deals(
    Extension(db): Extension<DatabaseConnection>,
) -> Result<impl IntoResponse, StatusCode> {
    let deals = deal::Entity::find()
        .all(&db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let deal_models: Vec<DealModel> = deals.into_iter().map(DealModel::from).collect();
    Ok(JsonResponse(deal_models))
}

pub async fn get_deal(
    Extension(db): Extension<DatabaseConnection>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, StatusCode> {
    let deal = deal::Entity::find_by_id(id)
        .one(&db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;

    Ok(JsonResponse(DealModel::from(deal)))
}

pub async fn update_deal(
    Extension(db): Extension<DatabaseConnection>,
    Path(id): Path<Uuid>,
    Json(input): Json<UpdateDealInput>,
) -> Result<impl IntoResponse, StatusCode> {
    let mut deal: deal::ActiveModel = deal::Entity::find_by_id(id)
        .one(&db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?
        .into();

    if let Some(name) = input.name {
        deal.name = Set(name);
    }
    if let Some(amount) = input.amount {
        deal.amount = Set(amount);
    }
    if let Some(status) = input.status {
        deal.status = Set(status);
    }
    if let Some(stage) = input.stage {
        deal.stage = Set(stage);
    }
    if let Some(close_date) = input.close_date {
        deal.close_date = Set(Some(close_date));
    }
    if let Some(is_active) = input.is_active {
        deal.is_active = Set(is_active);
    }
    deal.updated_at = Set(Utc::now());

    let updated_deal = deal.update(&db).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(JsonResponse(DealModel::from(updated_deal)))
}

pub async fn delete_deal(
    Extension(db): Extension<DatabaseConnection>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, StatusCode> {
    let result = deal::Entity::delete_by_id(id)
        .exec(&db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    if result.rows_affected == 0 {
        return Err(StatusCode::NOT_FOUND);
    }

    Ok(StatusCode::NO_CONTENT)
}

pub async fn add_file_to_deal(
    State(db): State<DatabaseConnection>,
    Path((deal_id, file_id)): Path<(Uuid, Uuid)>,
) -> Result<impl IntoResponse, StatusCode> {
    let deal = deal::Entity::find_by_id(deal_id)
        .one(&db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;

    deal.add_file(&db, file_id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(StatusCode::OK)
}

pub async fn get_deal_files(
    State(db): State<DatabaseConnection>,
    Path(deal_id): Path<Uuid>,
) -> Result<impl IntoResponse, StatusCode> {
    let deal = deal::Entity::find_by_id(deal_id)
        .one(&db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;

    let file_ids = deal.get_associated_files(&db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(file_ids))
}

pub async fn get_deal_contacts(
    Extension(db): Extension<DatabaseConnection>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, StatusCode> {
    let deal = deal::Entity::find_by_id(id)
        .one(&db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;

    let contacts = deal.get_contacts(&db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(JsonResponse(contacts.into_iter().map(ContactModel::from).collect::<Vec<_>>()))
}

pub async fn add_contact_to_deal(
    Extension(db): Extension<DatabaseConnection>,
    Path((deal_id, contact_id)): Path<(Uuid, Uuid)>,
) -> Result<impl IntoResponse, StatusCode> {
    let deal = deal::Entity::find_by_id(deal_id)
        .one(&db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;

    let contact = contact::Entity::find_by_id(contact_id)
        .one(&db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;

    deal.add_contact(&db, &contact)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(StatusCode::OK)
}

pub async fn remove_contact_from_deal(
    Extension(db): Extension<DatabaseConnection>,
    Path((deal_id, contact_id)): Path<(Uuid, Uuid)>,
) -> Result<impl IntoResponse, StatusCode> {
    let deal = deal::Entity::find_by_id(deal_id)
        .one(&db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;

    let contact = contact::Entity::find_by_id(contact_id)
        .one(&db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;

    deal.remove_contact(&db, &contact)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(StatusCode::NO_CONTENT)
}

pub async fn create_deal_note(
    Extension(db): Extension<DatabaseConnection>,
    Extension(current_user): Extension<user::Model>,
    Path(id): Path<Uuid>,
    Json(input): Json<CreateNoteInput>,
) -> Result<impl IntoResponse, StatusCode> {
    let deal = deal::Entity::find_by_id(id)
        .one(&db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;

    let new_note = note::ActiveModel {
        id: Set(Uuid::new_v4()),
        content: Set(input.content),
        created_by: Set(current_user.id),
        entity_type: Set("Deal".to_string()),
        entity_id: Set(deal.id),
        created_at: Set(Utc::now()),
        updated_at: Set(Utc::now()),
    };

    let note = new_note.insert(&db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok((StatusCode::CREATED, JsonResponse(NoteModel::from(note))))
}

pub async fn get_deal_notes(
    Extension(db): Extension<DatabaseConnection>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, StatusCode> {
    let deal = deal::Entity::find_by_id(id)
        .one(&db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;

    let notes = deal
        .find_related(note::Entity)
        .all(&db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let note_models: Vec<NoteModel> = notes.into_iter().map(NoteModel::from).collect();
    Ok(JsonResponse(note_models))
}

pub async fn create_deal_activity(
    Extension(db): Extension<DatabaseConnection>,
    Extension(current_user): Extension<user::Model>,
    Path(id): Path<Uuid>,
    Json(input): Json<CreateActivityInput>,
) -> Result<impl IntoResponse, StatusCode> {
    let deal = deal::Entity::find_by_id(id)
        .one(&db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;

    let new_activity = activity::ActiveModel {
        id: Set(Uuid::new_v4()),
        deal_id: Set(Some(deal.id)),
        activity_type: Set(input.activity_type),
        title: Set(input.title),
        description: Set(input.description),
        status: Set(input.status),
        due_date: Set(input.due_date),
        completed_at: Set(None),
        created_by: Set(current_user.id),
        assigned_to: Set(input.assigned_to),
        created_at: Set(Utc::now()),
        updated_at: Set(Utc::now()),
        ..Default::default()
    };

    let activity = new_activity.insert(&db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok((StatusCode::CREATED, JsonResponse(ActivityModel::from(activity))))
}

pub async fn get_deal_activities(
    Extension(db): Extension<DatabaseConnection>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, StatusCode> {
    let deal = deal::Entity::find_by_id(id)
        .one(&db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;

    let activities = deal
        .find_related(activity::Entity)
        .all(&db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let activity_models: Vec<ActivityModel> = activities.into_iter().map(ActivityModel::from).collect();
    Ok(JsonResponse(activity_models))
}
