use axum::{
    extract::{Extension, Path, Json},
    http::StatusCode,
    response::{IntoResponse, Json as JsonResponse},
    routing::{get, post, put, delete},
    Router,
};
use sea_orm::{
    DatabaseConnection, EntityTrait, QueryFilter, Set, ColumnTrait,
    ActiveModelTrait, ModelTrait,IntoActiveModel, ActiveValue, Value
};
use uuid::Uuid;
use chrono::Utc;
use crate::entities::{lead, listing, account, note,user, activity};
use crate::models::lead::{LeadModel, CreateLeadInput, UpdateLeadInput};
use crate::models::address::AddressJson;
use crate::models::file::FileAssociation;
use crate::models::note::{NoteModel, CreateNoteInput};
use crate::models::activity::{ActivityModel, CreateActivityInput};

pub fn routes() -> Router {
    Router::new()
        .route("/api/leads", post(create_lead))
        .route("/api/leads", get(get_leads))
        .route("/api/leads/{id}", get(get_lead))
        .route("/api/leads/{id}", put(update_lead))
        .route("/api/leads/{id}", delete(delete_lead))
        .route("/api/leads/{lead_id}/files/{file_id}", post(add_file_to_lead))
        .route("/api/leads/{id}/files", get(get_lead_files))
        .route("/api/leads/{id}/notes", get(get_lead_notes))
        .route("/api/leads/{id}/activities", get(get_lead_activities))
}

pub async fn create_lead(
    Extension(db): Extension<DatabaseConnection>,
    Json(input): Json<CreateLeadInput>,
) -> Result<impl IntoResponse, StatusCode> {
    // Check if the listing exists if provided
    if let Some(listing_id) = input.listing_id {
        listing::Entity::find_by_id(listing_id)
            .one(&db)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
            .ok_or(StatusCode::NOT_FOUND)?;
    }

    // Check if the account exists if provided
    if let Some(account_id) = input.account_id {
        account::Entity::find_by_id(account_id)
            .one(&db)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
            .ok_or(StatusCode::NOT_FOUND)?;
    }

    let mut new_lead = lead::ActiveModel {
        id: Set(Uuid::new_v4()),
        name: Set(input.name),
        listing_id: Set(input.listing_id),
        account_id: Set(input.account_id),
        first_name: Set(input.first_name),
        last_name: Set(input.last_name),
        email: Set(input.email),
        phone: Set(input.phone),
        whatsapp: Set(input.whatsapp),
        telegram: Set(input.telegram),
        twitter: Set(input.twitter),
        instagram: Set(input.instagram),
        facebook: Set(input.facebook),
        message: Set(input.message),
        source: Set(input.source),
        is_converted: Set(false),
        converted_to_contact: Set(false),
        associated_deal_id: Set(None),
        converted_customer_id: Set(None),
        converted_contact_id: Set(None),
        created_at: Set(Utc::now()),
        updated_at: Set(Utc::now()),
        ..Default::default()
    };

    if let Some(billing_address) = input.billing_address {
        new_lead.billing_address = Set(Some(billing_address));
    }

    if let Some(shipping_address) = input.shipping_address {
        new_lead.shipping_address = Set(Some(shipping_address));
    }

    let lead = new_lead.insert(&db).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    Ok((StatusCode::CREATED, JsonResponse(LeadModel::from(lead))))
}

pub async fn get_leads(
    Extension(db): Extension<DatabaseConnection>,
) -> Result<impl IntoResponse, StatusCode> {
    let leads = lead::Entity::find()
        .all(&db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let lead_models: Vec<LeadModel> = leads.into_iter().map(LeadModel::from).collect();
    Ok(JsonResponse(lead_models))
}

pub async fn get_lead(
    Extension(db): Extension<DatabaseConnection>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, StatusCode> {
    let lead = lead::Entity::find_by_id(id)
        .one(&db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;

    Ok(JsonResponse(LeadModel::from(lead)))
}

pub async fn update_lead(
    Extension(db): Extension<DatabaseConnection>,
    Path(id): Path<Uuid>,
    Json(input): Json<UpdateLeadInput>,
) -> Result<impl IntoResponse, StatusCode> {
    let mut lead: lead::ActiveModel = lead::Entity::find_by_id(id)
        .one(&db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?.into();

    if let Some(name) = input.name {
        lead.name = Set(name);
    }

    if let Some(listing_id) = input.listing_id {
        lead.listing_id = Set(Some(listing_id));
    }

    if let Some(account_id) = input.account_id {
        lead.account_id = Set(Some(account_id));
    }

    if let Some(first_name) = input.first_name {
        lead.first_name = Set(Some(first_name));
    }

    if let Some(last_name) = input.last_name {
        lead.last_name = Set(Some(last_name));
    }

    if let Some(email) = input.email {
        lead.email = Set(Some(email));
    }

    if let Some(phone) = input.phone {
        lead.phone = Set(Some(phone));
    }

    if let Some(whatsapp) = input.whatsapp {
        lead.whatsapp = Set(Some(whatsapp));
    }

    if let Some(telegram) = input.telegram {
        lead.telegram = Set(Some(telegram));
    }

    if let Some(twitter) = input.twitter {
        lead.twitter = Set(Some(twitter));
    }

    if let Some(instagram) = input.instagram {
        lead.instagram = Set(Some(instagram));
    }

    if let Some(facebook) = input.facebook {
        lead.facebook = Set(Some(facebook));
    }

    if let Some(billing_address) = input.billing_address {
        lead.billing_address = Set(Some(billing_address));
    }

    if let Some(shipping_address) = input.shipping_address {
        lead.shipping_address = Set(Some(shipping_address));
    }

    if let Some(message) = input.message {
        lead.message = Set(Some(message));
    }

    if let Some(source) = input.source {
        lead.source = Set(Some(source));
    }

    if let Some(is_converted) = input.is_converted {
        lead.is_converted = Set(is_converted);
    }

    if let Some(converted_to_contact) = input.converted_to_contact {
        lead.converted_to_contact = Set(converted_to_contact);
    }

    if let Some(associated_deal_id) = input.associated_deal_id {
        lead.associated_deal_id = Set(Some(associated_deal_id));
    }

    if let Some(converted_customer_id) = input.converted_customer_id {
        lead.converted_customer_id = Set(Some(converted_customer_id));
    }

    if let Some(converted_contact_id) = input.converted_contact_id {
        lead.converted_contact_id = Set(Some(converted_contact_id));
    }

    let updated_lead = lead.update(&db).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    Ok(JsonResponse(LeadModel::from(updated_lead)))
}

pub async fn delete_lead(
    Extension(db): Extension<DatabaseConnection>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, StatusCode> {
    let lead = lead::Entity::find_by_id(id)
        .one(&db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;

    lead.delete(&db).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(StatusCode::NO_CONTENT)
}

pub async fn add_file_to_lead(
    Extension(db): Extension<DatabaseConnection>,
    Path((lead_id, file_id)): Path<(Uuid, Uuid)>,
) -> Result<impl IntoResponse, StatusCode> {
    let lead = lead::Entity::find_by_id(lead_id)
        .one(&db)
        .await
        .map_err(|e| {
            tracing::error!("Failed to fetch lead: {:?}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?
        .ok_or(StatusCode::NOT_FOUND)?;

    lead.add_file(&db, file_id)
        .await
        .map_err(|e| {
            tracing::error!("Failed to add file to lead: {:?}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    Ok(StatusCode::OK)
}

pub async fn get_lead_files(
    Extension(db): Extension<DatabaseConnection>,
    Path(lead_id): Path<Uuid>,
) -> Result<impl IntoResponse, StatusCode> {
    let lead = lead::Entity::find_by_id(lead_id)
        .one(&db)
        .await
        .map_err(|e| {
            tracing::error!("Failed to fetch lead: {:?}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?
        .ok_or(StatusCode::NOT_FOUND)?;

    let file_ids = lead.get_associated_files(&db)
        .await
        .map_err(|e| {
            tracing::error!("Failed to get associated files: {:?}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    Ok(Json(file_ids))
}

pub async fn get_lead_notes(
    Extension(db): Extension<DatabaseConnection>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, StatusCode> {
    let lead = lead::Entity::find_by_id(id)
        .one(&db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;

    let notes = note::Entity::find()
        .filter(note::Column::EntityType.eq("Lead"))
        .filter(note::Column::EntityId.eq(lead.id))
        .all(&db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let note_models: Vec<NoteModel> = notes.into_iter().map(NoteModel::from).collect();
    Ok(JsonResponse(note_models))
}

pub async fn get_lead_activities(
    Extension(db): Extension<DatabaseConnection>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, StatusCode> {
    let lead = lead::Entity::find_by_id(id)
        .one(&db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;

    let activities = activity::Entity::find()
        .filter(activity::Column::LeadId.eq(Some(lead.id)))
        .all(&db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let activity_models: Vec<ActivityModel> = activities.into_iter().map(ActivityModel::from).collect();
    Ok(JsonResponse(activity_models))
}

pub async fn create_lead_note(
    Extension(db): Extension<DatabaseConnection>,
    Extension(current_user): Extension<user::Model>,
    Path(id): Path<Uuid>,
    Json(input): Json<CreateNoteInput>,
) -> Result<impl IntoResponse, StatusCode> {
    let lead = lead::Entity::find_by_id(id)
        .one(&db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;

    let new_note = note::ActiveModel {
        id: Set(Uuid::new_v4()),
        content: Set(input.content),
        created_by: Set(current_user.id),
        entity_type: Set("Lead".to_string()),
        entity_id: Set(lead.id),
        created_at: Set(Utc::now()),
        updated_at: Set(Utc::now()),
    };

    let inserted_note = new_note.insert(&db).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    Ok((StatusCode::CREATED, JsonResponse(NoteModel::from(inserted_note))))
}

pub async fn create_lead_activity(
    Extension(db): Extension<DatabaseConnection>,
    Extension(current_user): Extension<user::Model>,
    Path(id): Path<Uuid>,
    Json(input): Json<CreateActivityInput>,
) -> Result<impl IntoResponse, StatusCode> {
    let lead = lead::Entity::find_by_id(id)
        .one(&db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;

    let new_activity = activity::ActiveModel {
        id: Set(Uuid::new_v4()),
        lead_id: Set(Some(lead.id)),
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

    let inserted_activity = new_activity.insert(&db).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    Ok((StatusCode::CREATED, JsonResponse(ActivityModel::from(inserted_activity))))
}
