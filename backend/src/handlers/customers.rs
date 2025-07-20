use axum::{
    extract::{Extension, Path, Json, Query},
    http::StatusCode,
    response::{IntoResponse, Json as JsonResponse},
    routing::{get, post, put, delete},
    Router,
};
use sea_orm::{
    DatabaseConnection, EntityTrait, QueryFilter, Set, ColumnTrait,
    ActiveModelTrait, ModelTrait, PaginatorTrait,ActiveValue, Value
};
use crate::handlers::Validate;
use uuid::Uuid;
use chrono::Utc;
use serde_json::json;
use crate::entities::{customer, user, contact, note, activity};
use crate::entities::customer::CustomerType;
use crate::models::customer::{CreateCustomerInput, UpdateCustomerInput};
use crate::models::contact::{ CreateContactInput};
use crate::models::note::{CreateNoteInput};
use crate::models::activity::{ActivityModel, CreateActivityInput};
use crate::models::file::FileAssociation;
use crate::models::address::AddressJson;
use crate::models::customer::Customer as CustomerModel;
use crate::models::contact::Contact as ContactModel;
use crate::models::note::NoteModel;

pub fn routes() -> Router {
    Router::new()
        .route("/customers", post(create_customer))
        .route("/customers", get(get_customers))
        .route("/customers/{id}", get(get_customer))
        .route("/customers/{id}", put(update_customer))
        .route("/customers/{id}", delete(delete_customer))
        .route("/customers/{customer_id}/files/{file_id}", post(add_file_to_customer))
        .route("/customers/{id}/files", get(get_customer_files))
        .route("/customers/{id}/contacts", post(create_customer_contact))
        .route("/customers/{id}/contacts", get(get_customer_contacts))
        .route("/customers/{id}/notes", post(create_customer_note))
        .route("/customers/{id}/notes", get(get_customer_notes))
        .route("/customers/{id}/activities", post(create_customer_activity))
        .route("/customers/{id}/activities", get(get_customer_activities))
}

pub async fn create_customer(
    Extension(db): Extension<DatabaseConnection>,
    Extension(current_user): Extension<user::Model>,
    Json(payload): Json<CreateCustomerInput>,
) -> Result<impl IntoResponse, StatusCode> {
    // Parse the customer type string into the enum
    let customer_type = match payload.customer_type.as_str() {
        "Household" => CustomerType::Household,
        "BusinessEntity" => CustomerType::BusinessEntity,
        "Person" => CustomerType::Person,
        _ => return Err(StatusCode::BAD_REQUEST),
    };

    let new_customer = customer::ActiveModel {
        id: Set(Uuid::new_v4()),
        name: Set(payload.name),
        primary_contact_id: Set(None), // Fixed: Initialize as None since it's not in payload
        customer_type: Set(customer_type),
        attributes: Set(payload.attributes),
        cpf: Set(payload.cpf),
        cnpj: Set(payload.cnpj),
        tin: Set(payload.tin),
        email: Set(payload.email),
        phone: Set(payload.phone),
        whatsapp: Set(payload.whatsapp),
        telegram: Set(payload.telegram),
        twitter: Set(payload.twitter),
        instagram: Set(payload.instagram),
        facebook: Set(payload.facebook),
        website: Set(payload.website),
        annual_revenue: Set(payload.annual_revenue),
        employee_count: Set(payload.employee_count),
        is_active: Set(true),
        created_at: Set(Utc::now()),
        updated_at: Set(Utc::now()),
        billing_address: Set(None),
        shipping_address: Set(None),
    };

    let mut customer = new_customer.insert(&db).await.map_err(|e| {
        tracing::error!("Failed to create customer: {:?}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    if let Some(billing_address) = payload.billing_address {
        billing_address.validate().map_err(|e| {
            tracing::error!("Invalid billing address: {:?}", e);
            StatusCode::BAD_REQUEST
        })?;
        let mut active_model: customer::ActiveModel = customer.clone().into();
        active_model.billing_address = Set(Some(billing_address));
        customer = active_model.update(&db).await.map_err(|e| {
            tracing::error!("Failed to update customer: {:?}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;
    }

    if let Some(shipping_address) = payload.shipping_address {
        shipping_address.validate().map_err(|e| {
            tracing::error!("Invalid shipping address: {:?}", e);
            StatusCode::BAD_REQUEST
        })?;
        let mut active_model: customer::ActiveModel = customer.clone().into();
        active_model.shipping_address = Set(Some(shipping_address));
        customer = active_model.update(&db).await.map_err(|e| {
            tracing::error!("Failed to update customer: {:?}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;
    }

    let customer_model: CustomerModel = customer.into();
    Ok((StatusCode::CREATED, JsonResponse(customer_model)))
}

pub async fn get_customers(
    Extension(db): Extension<DatabaseConnection>,
    Extension(current_user): Extension<user::Model>,
    Query(params): Query<std::collections::HashMap<String, String>>,
) -> Result<impl IntoResponse, StatusCode> {
    let page: u64 = params.get("page").and_then(|v| v.parse().ok()).unwrap_or(1);
    let items_per_page: u64 = params.get("items_per_page").and_then(|v| v.parse().ok()).unwrap_or(10);

    let paginator = customer::Entity::find()
        .paginate(&db, items_per_page);
    let total_pages = paginator.num_pages().await.map_err(|e| {
        tracing::error!("Failed to get total pages: {:?}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    let customers = paginator
        .fetch_page(page - 1)
        .await
        .map_err(|e| {
            tracing::error!("Failed to fetch customers: {:?}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    let customer_models: Vec<CustomerModel> = customers.into_iter().map(Into::into).collect();

    Ok((StatusCode::OK, JsonResponse(json!({
        "customers": customer_models,
        "total_pages": total_pages,
        "current_page": page,
    }))))
}

pub async fn get_customer(
    Extension(db): Extension<DatabaseConnection>,
    Extension(current_user): Extension<user::Model>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, StatusCode> {
    let customer = customer::Entity::find_by_id(id)
        .one(&db)
        .await
        .map_err(|e| {
            tracing::error!("Failed to fetch customer: {:?}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?
        .ok_or(StatusCode::NOT_FOUND)?;

    let customer_model: CustomerModel = customer.into();
    Ok((StatusCode::OK, JsonResponse(customer_model)))
}

pub async fn update_customer(
    Extension(db): Extension<DatabaseConnection>,
    Extension(current_user): Extension<user::Model>,
    Path(id): Path<Uuid>,
    Json(payload): Json<UpdateCustomerInput>,
) -> Result<impl IntoResponse, StatusCode> {
    let mut customer: customer::ActiveModel = customer::Entity::find_by_id(id)
        .one(&db)
        .await
        .map_err(|e| {
            tracing::error!("Failed to fetch customer: {:?}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?
        .ok_or(StatusCode::NOT_FOUND)?
        .into();

    if let Some(name) = payload.name {
        customer.name = Set(name);
    }
    if let Some(primary_contact_id) = payload.primary_contact_id {
        customer.primary_contact_id = Set(Some(primary_contact_id));
    }
    if let Some(customer_type) = payload.customer_type {
        let customer_type = match customer_type.as_str() {
            "Household" => CustomerType::Household,
            "BusinessEntity" => CustomerType::BusinessEntity,
            "Person" => CustomerType::Person,
            _ => return Err(StatusCode::BAD_REQUEST),
        };
        customer.customer_type = Set(customer_type);
    }
    if let Some(attributes) = payload.attributes {
        customer.attributes = Set(attributes);
    }
    if let Some(cpf) = payload.cpf {
        customer.cpf = Set(Some(cpf));
    }
    if let Some(cnpj) = payload.cnpj {
        customer.cnpj = Set(Some(cnpj));
    }
    if let Some(tin) = payload.tin {
        customer.tin = Set(Some(tin));
    }
    if let Some(email) = payload.email {
        customer.email = Set(Some(email));
    }
    if let Some(phone) = payload.phone {
        customer.phone = Set(Some(phone));
    }
    if let Some(whatsapp) = payload.whatsapp {
        customer.whatsapp = Set(Some(whatsapp));
    }
    if let Some(telegram) = payload.telegram {
        customer.telegram = Set(Some(telegram));
    }
    if let Some(twitter) = payload.twitter {
        customer.twitter = Set(Some(twitter));
    }
    if let Some(instagram) = payload.instagram {
        customer.instagram = Set(Some(instagram));
    }
    if let Some(facebook) = payload.facebook {
        customer.facebook = Set(Some(facebook));
    }
    if let Some(website) = payload.website {
        customer.website = Set(Some(website));
    }
    if let Some(annual_revenue) = payload.annual_revenue {
        customer.annual_revenue = Set(Some(annual_revenue));
    }
    if let Some(employee_count) = payload.employee_count {
        customer.employee_count = Set(Some(employee_count));
    }
    if let Some(is_active) = payload.is_active {
        customer.is_active = Set(is_active);
    }
    customer.updated_at = Set(Utc::now());

    let mut updated_customer = customer.update(&db).await.map_err(|e| {
        tracing::error!("Failed to update customer: {:?}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    if let Some(billing_address) = payload.billing_address {
        // Validate the address
        billing_address.validate().map_err(|e| {
            tracing::error!("Invalid billing address: {:?}", e);
            StatusCode::BAD_REQUEST
        })?;
        updated_customer.billing_address = Some(billing_address);  // Removed .0
    }

    if let Some(shipping_address) = payload.shipping_address {
        // Validate the address
        shipping_address.validate().map_err(|e| {
            tracing::error!("Invalid shipping address: {:?}", e);
            StatusCode::BAD_REQUEST
        })?;
        updated_customer.shipping_address = Some(shipping_address);  // Removed .0
    }

    let customer_model: CustomerModel = updated_customer.into();
    Ok((StatusCode::OK, JsonResponse(customer_model)))
}

pub async fn delete_customer(
    Extension(db): Extension<DatabaseConnection>,
    Extension(current_user): Extension<user::Model>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, StatusCode> {
    let result = customer::Entity::delete_by_id(id)
        .exec(&db)
        .await
        .map_err(|e| {
            tracing::error!("Failed to delete customer: {:?}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    if result.rows_affected == 0 {
        return Err(StatusCode::NOT_FOUND);
    }

    Ok(StatusCode::NO_CONTENT)
}

pub async fn add_file_to_customer(
    Extension(db): Extension<DatabaseConnection>,
    Extension(current_user): Extension<user::Model>,
    Path((customer_id, file_id)): Path<(Uuid, Uuid)>,
) -> Result<impl IntoResponse, StatusCode> {
    let customer = customer::Entity::find_by_id(customer_id)
        .one(&db)
        .await
        .map_err(|e| {
            tracing::error!("Failed to fetch customer: {:?}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?
        .ok_or(StatusCode::NOT_FOUND)?;

    customer.add_file(&db, file_id)
        .await
        .map_err(|e| {
            tracing::error!("Failed to add file to customer: {:?}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    Ok(StatusCode::OK)
}

pub async fn get_customer_files(
    Extension(db): Extension<DatabaseConnection>,
    Extension(current_user): Extension<user::Model>,
    Path(customer_id): Path<Uuid>,
) -> Result<impl IntoResponse, StatusCode> {
    let customer = customer::Entity::find_by_id(customer_id)
        .one(&db)
        .await
        .map_err(|e| {
            tracing::error!("Failed to fetch customer: {:?}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?
        .ok_or(StatusCode::NOT_FOUND)?;

    let file_ids = customer.get_associated_files(&db)
        .await
        .map_err(|e| {
            tracing::error!("Failed to get associated files: {:?}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    Ok(Json(file_ids))
}

pub async fn create_customer_contact(
    Extension(db): Extension<DatabaseConnection>,
    Path(customer_id): Path<Uuid>,
    Json(input): Json<CreateContactInput>,
) -> Result<impl IntoResponse, StatusCode> {
    let customer = customer::Entity::find_by_id(customer_id)
        .one(&db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;

    let new_contact = contact::ActiveModel {
        id: Set(Uuid::new_v4()),
        customer_id: Set(Some(customer.id)),
        name: Set(input.name),
        first_name: Set(input.first_name),
        last_name: Set(input.last_name),
        email: Set(input.email),
        phone: Set(input.phone),
        whatsapp: Set(input.whatsapp),
        telegram: Set(input.telegram),
        twitter: Set(input.twitter),
        instagram: Set(input.instagram),
        facebook: Set(input.facebook),
        created_at: Set(Utc::now()),
        updated_at: Set(Utc::now()),
        ..Default::default()
    };

    let contact = new_contact.insert(&db).await.map_err(|e| {
        tracing::error!("Failed to create contact: {:?}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    let contact_model: ContactModel = contact.into();
    Ok((StatusCode::CREATED, JsonResponse(contact_model)))
}

pub async fn get_customer_contacts(
    Extension(db): Extension<DatabaseConnection>,
    Path(customer_id): Path<Uuid>,
) -> Result<impl IntoResponse, StatusCode> {
    let contacts = contact::Entity::find()
        .filter(contact::Column::CustomerId.eq(customer_id))
        .all(&db)
        .await
        .map_err(|e| {
            tracing::error!("Failed to fetch customer contacts: {:?}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    let contact_models: Vec<ContactModel> = contacts.into_iter().map(Into::into).collect();
    Ok((StatusCode::OK, JsonResponse(contact_models)))
}

pub async fn create_customer_note(
    Extension(db): Extension<DatabaseConnection>,
    Extension(current_user): Extension<user::Model>,
    Path(customer_id): Path<Uuid>,
    Json(input): Json<CreateNoteInput>,
) -> Result<impl IntoResponse, StatusCode> {
    let customer = customer::Entity::find_by_id(customer_id)
        .one(&db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;

    let new_note = note::ActiveModel {
        id: Set(Uuid::new_v4()),
        content: Set(input.content),
        created_by: Set(current_user.id),
        entity_type: Set("Customer".to_string()),
        entity_id: Set(customer.id),
        created_at: Set(Utc::now()),
        updated_at: Set(Utc::now()),
    };

    let inserted_note = new_note.insert(&db).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    Ok((StatusCode::CREATED, JsonResponse(NoteModel::from(inserted_note))))
}

pub async fn get_customer_notes(
    Extension(db): Extension<DatabaseConnection>,
    Path(customer_id): Path<Uuid>,
) -> Result<impl IntoResponse, StatusCode> {
    let notes = note::Entity::find()
        .filter(note::Column::EntityType.eq("Customer"))
        .filter(note::Column::EntityId.eq(customer_id))
        .all(&db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let note_models: Vec<NoteModel> = notes.into_iter().map(NoteModel::from).collect();
    Ok(JsonResponse(note_models))
}

pub async fn create_customer_activity(
    Extension(db): Extension<DatabaseConnection>,
    Extension(current_user): Extension<user::Model>,
    Path(customer_id): Path<Uuid>,
    Json(input): Json<CreateActivityInput>,
) -> Result<impl IntoResponse, StatusCode> {
    let customer = customer::Entity::find_by_id(customer_id)
        .one(&db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;

    let new_activity = activity::ActiveModel {
        id: Set(Uuid::new_v4()),
        customer_id: Set(Some(customer.id)),
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

pub async fn get_customer_activities(
    Extension(db): Extension<DatabaseConnection>,
    Path(customer_id): Path<Uuid>,
) -> Result<impl IntoResponse, StatusCode> {
    let activities = activity::Entity::find()
        .filter(activity::Column::CustomerId.eq(Some(customer_id)))
        .all(&db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let activity_models: Vec<ActivityModel> = activities.into_iter().map(ActivityModel::from).collect();
    Ok(JsonResponse(activity_models))
}

