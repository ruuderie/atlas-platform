use axum::{
    extract::{Extension, Path, Json},
    http::StatusCode,
    response::{IntoResponse, Json as JsonResponse},
    routing::{get, post, put, delete},
    Router,
};
use sea_orm::{
    DatabaseConnection, EntityTrait, QueryFilter, Set, ColumnTrait,
    ActiveModelTrait,
};
use uuid::Uuid;
use chrono::Utc;
use crate::handlers::Validate;
use crate::models::{address::AddressJson, contact::Contact as ContactModel};
use crate::entities::{contact, note, activity, user};
use crate::models::contact::{ CreateContactInput, UpdateContactInput};
use crate::models::file::FileAssociation;
use crate::models::note::{NoteModel, CreateNoteInput};
use crate::models::activity::{ActivityModel, CreateActivityInput};
use crate::models::contact::Contact;

pub fn routes() -> Router<DatabaseConnection> {
    Router::new()
        .route("/api/contacts", post(create_contact))
        .route("/api/contacts", get(get_contacts))
        .route("/api/contacts/{id}", get(get_contact))
        .route("/api/contacts/{id}", put(update_contact))
        .route("/api/contacts/{id}", delete(delete_contact))
        .route("/api/contacts/{contact_id}/files/{file_id}", post(add_file_to_contact))
        .route("/api/contacts/{id}/files", get(get_contact_files))
        .route("/api/contacts/{id}/notes", post(create_contact_note))
        .route("/api/contacts/{id}/notes", get(get_contact_notes))
        .route("/api/contacts/{id}/activities", post(create_contact_activity))
        .route("/api/contacts/{id}/activities", get(get_contact_activities))
}

pub async fn create_contact(
    Extension(db): Extension<DatabaseConnection>,
    Json(payload): Json<CreateContactInput>,
) -> Result<impl IntoResponse, StatusCode> {
    let new_contact = contact::ActiveModel {
        id: Set(Uuid::new_v4()),
        customer_id: Set(payload.customer_id),
        name: Set(payload.name),
        first_name: Set(payload.first_name),
        last_name: Set(payload.last_name),
        email: Set(payload.email),
        phone: Set(payload.phone),
        whatsapp: Set(payload.whatsapp),
        telegram: Set(payload.telegram),
        twitter: Set(payload.twitter),
        instagram: Set(payload.instagram),
        facebook: Set(payload.facebook),
        created_at: Set(Utc::now()),
        updated_at: Set(Utc::now()),
        properties: Set(None),
        ..Default::default()
    };

    let mut contact = new_contact.insert(&db).await.map_err(|e| {
        tracing::error!("Failed to create contact: {:?}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    if let Some(billing_address) = payload.billing_address {
        contact.billing_address = Some(AddressJson(billing_address.into()));
    }

    if let Some(shipping_address) = payload.shipping_address {
        contact.shipping_address = Some(AddressJson(shipping_address.into()));
    }

    let contact_model: Contact = contact.into();
    Ok((StatusCode::CREATED, JsonResponse(contact_model)))
}

pub async fn get_contacts(
    Extension(db): Extension<DatabaseConnection>,
) -> Result<impl IntoResponse, StatusCode> {
    let contacts = contact::Entity::find()
        .all(&db)
        .await
        .map_err(|e| {
            tracing::error!("Failed to fetch contacts: {:?}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    let contact_models: Vec<ContactModel> = contacts.into_iter().map(|c| c.into()).collect();
    Ok((StatusCode::OK, JsonResponse(contact_models)))
}

pub async fn get_contact(
    Extension(db): Extension<DatabaseConnection>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, StatusCode> {
    let contact = contact::Entity::find_by_id(id)
        .one(&db)
        .await
        .map_err(|e| {
            tracing::error!("Failed to fetch contact: {:?}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?
        .ok_or(StatusCode::NOT_FOUND)?;

    let contact_model: ContactModel = contact.into();
    Ok((StatusCode::OK, JsonResponse(contact_model)))
}

pub async fn update_contact(
    Extension(db): Extension<DatabaseConnection>,
    Path(id): Path<Uuid>,
    Json(payload): Json<UpdateContactInput>,
) -> Result<impl IntoResponse, StatusCode> {
    let mut contact: contact::ActiveModel = contact::Entity::find_by_id(id)
        .one(&db)
        .await
        .map_err(|e| {
            tracing::error!("Failed to fetch contact: {:?}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?
        .ok_or(StatusCode::NOT_FOUND)?
        .into();

    if let Some(customer_id) = payload.customer_id {
        contact.customer_id = Set(Some(customer_id));
    }
    if let Some(name) = payload.name {
        contact.name = Set(name);
    }
    if let Some(first_name) = payload.first_name {
        contact.first_name = Set(Some(first_name));
    }
    if let Some(last_name) = payload.last_name {
        contact.last_name = Set(Some(last_name));
    }
    if let Some(email) = payload.email {
        contact.email = Set(Some(email));
    }
    if let Some(phone) = payload.phone {
        contact.phone = Set(Some(phone));
    }
    if let Some(whatsapp) = payload.whatsapp {
        contact.whatsapp = Set(Some(whatsapp));
    }
    if let Some(telegram) = payload.telegram {
        contact.telegram = Set(Some(telegram));
    }
    if let Some(twitter) = payload.twitter {
        contact.twitter = Set(Some(twitter));
    }
    if let Some(instagram) = payload.instagram {
        contact.instagram = Set(Some(instagram));
    }
    if let Some(facebook) = payload.facebook {
        contact.facebook = Set(Some(facebook));
    }
    contact.updated_at = Set(Utc::now());

    let mut updated_contact = contact.update(&db).await.map_err(|e| {
        tracing::error!("Failed to update contact: {:?}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    if let Some(billing_address) = payload.billing_address {
        // Validate the address
        billing_address.validate().map_err(|e| {
            tracing::error!("Invalid billing address: {:?}", e);
            StatusCode::BAD_REQUEST
        })?;
        updated_contact.billing_address = Some(AddressJson(billing_address.into()));
    }

    if let Some(shipping_address) = payload.shipping_address {
        // Validate the address
        shipping_address.validate().map_err(|e| {
            tracing::error!("Invalid shipping address: {:?}", e);
            StatusCode::BAD_REQUEST
        })?;
        updated_contact.shipping_address = Some(AddressJson(shipping_address.into()));
    }

    let contact_model: ContactModel = updated_contact.into();
    Ok((StatusCode::OK, JsonResponse(contact_model)))
}

pub async fn delete_contact(
    Extension(db): Extension<DatabaseConnection>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, StatusCode> {
    let result = contact::Entity::delete_by_id(id).exec(&db).await.map_err(|e| {
        tracing::error!("Failed to delete contact: {:?}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    if result.rows_affected == 0 {
        return Err(StatusCode::NOT_FOUND);
    }

    Ok(StatusCode::NO_CONTENT)
}

pub async fn add_file_to_contact(
    Extension(db): Extension<DatabaseConnection>,
    Path((contact_id, file_id)): Path<(Uuid, Uuid)>,
) -> Result<impl IntoResponse, StatusCode> {
    let contact = contact::Entity::find_by_id(contact_id)
        .one(&db)
        .await
        .map_err(|e| {
            tracing::error!("Failed to fetch contact: {:?}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?
        .ok_or(StatusCode::NOT_FOUND)?;

    contact.add_file(&db, file_id)
        .await
        .map_err(|e| {
            tracing::error!("Failed to add file to contact: {:?}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    Ok(StatusCode::OK)
}

pub async fn get_contact_files(
    Extension(db): Extension<DatabaseConnection>,
    Path(contact_id): Path<Uuid>,
) -> Result<impl IntoResponse, StatusCode> {
    let contact = contact::Entity::find_by_id(contact_id)
        .one(&db)
        .await
        .map_err(|e| {
            tracing::error!("Failed to fetch contact: {:?}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?
        .ok_or(StatusCode::NOT_FOUND)?;

    let file_ids = contact.get_associated_files(&db)
        .await
        .map_err(|e| {
            tracing::error!("Failed to get associated files: {:?}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    Ok(Json(file_ids))
}

pub async fn create_contact_note(
    Extension(db): Extension<DatabaseConnection>,
    Extension(current_user): Extension<user::Model>,
    Path(id): Path<Uuid>,
    Json(input): Json<CreateNoteInput>,
) -> Result<impl IntoResponse, StatusCode> {
    let contact = contact::Entity::find_by_id(id)
        .one(&db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;

    let new_note = note::ActiveModel {
        id: Set(Uuid::new_v4()),
        content: Set(input.content),
        created_by: Set(current_user.id),
        entity_type: Set("Contact".to_string()),
        entity_id: Set(contact.id),
        created_at: Set(Utc::now()),
        updated_at: Set(Utc::now()),
    };

    let inserted_note = new_note.insert(&db).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    Ok((StatusCode::CREATED, JsonResponse(NoteModel::from(inserted_note))))
}

pub async fn get_contact_notes(
    Extension(db): Extension<DatabaseConnection>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, StatusCode> {
    let contact = contact::Entity::find_by_id(id)
        .one(&db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;

    let notes = note::Entity::find()
        .filter(note::Column::EntityType.eq("Contact"))
        .filter(note::Column::EntityId.eq(contact.id))
        .all(&db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let note_models: Vec<NoteModel> = notes.into_iter().map(NoteModel::from).collect();
    Ok(JsonResponse(note_models))
}

pub async fn create_contact_activity(
    Extension(db): Extension<DatabaseConnection>,
    Extension(current_user): Extension<user::Model>,
    Path(id): Path<Uuid>,
    Json(input): Json<CreateActivityInput>,
) -> Result<impl IntoResponse, StatusCode> {
    let contact = contact::Entity::find_by_id(id)
        .one(&db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;

    let new_activity = activity::ActiveModel {
        id: Set(Uuid::new_v4()),
        contact_id: Set(Some(contact.id)),
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

pub async fn get_contact_activities(
    Extension(db): Extension<DatabaseConnection>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, StatusCode> {
    let contact = contact::Entity::find_by_id(id)
        .one(&db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;

    let activities = activity::Entity::find()
        .filter(activity::Column::ContactId.eq(Some(contact.id)))
        .all(&db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let activity_models: Vec<ActivityModel> = activities.into_iter().map(ActivityModel::from).collect();
    Ok(JsonResponse(activity_models))
}
