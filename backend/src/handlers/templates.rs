use axum::{
    extract::{Extension, Json, Path, State},
    http::StatusCode,
};
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, DbErr, EntityTrait, 
    QueryFilter, Set, TransactionTrait,IntoActiveModel, DatabaseTransaction
};
use axum::response::{ Response, IntoResponse};
use serde_json::json;
use uuid::Uuid;
use chrono::Utc;
use futures::TryFutureExt;
use std::result::Result;
use std::str::FromStr;
use crate::entities::{
    listing,
    listing_attribute,
    profile,
    template,
    template::Entity as Template, // Add this line
    user,
    user_account,
    listing::Entity as Listing,
};
use crate::models::{
    template::{TemplateModel, CreateTemplate, UpdateTemplate},
    listing::{ListingModel, ListingCreate, ListingStatus}, 
    listing_attribute::{ListingAttributeModel, CreateListingAttribute, UpdateListingAttribute}
    
};

pub async fn get_templates(
    Path(directory_id): Path<Uuid>,
    State(db): State<DatabaseConnection>,
) -> Result<Json<Vec<TemplateModel>>, (StatusCode, Json<serde_json::Value>)> {
    let templates = template::Entity::find()
        .filter(template::Column::DirectoryId.eq(directory_id))
        .all(&db)
        .await
        .map_err(|err| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": "Failed to fetch templates", "details": err.to_string()})),
            )
        })?;

    let template_models: Vec<TemplateModel> = templates
        .into_iter()
        .map(Into::<TemplateModel>::into)
        .collect();

    Ok(Json(template_models))
}

pub async fn get_template_by_id(
    Path((directory_id, template_id)): Path<(Uuid, Uuid)>,
    State(db): State<DatabaseConnection>,
) -> Result<Json<TemplateModel>, (StatusCode, Json<serde_json::Value>)> {
    let template = template::Entity::find()
        .filter(template::Column::Id.eq(template_id))
        .filter(template::Column::DirectoryId.eq(directory_id))
        .one(&db)
        .await
        .map_err(|err| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": "Failed to fetch template", "details": err.to_string()})),
            )
        })?;

    if let Some(template) = template {
        Ok(Json(TemplateModel::from(template)))
    } else {
        Err((StatusCode::NOT_FOUND, Json(json!({"error": "Template not found"}))))
    }
}

pub async fn create_template(
    State(db): State<DatabaseConnection>,
    Path(directory_id): Path<Uuid>,
    Json(payload): Json<CreateTemplate>,
) -> Result<Json<TemplateModel>, (StatusCode, Json<serde_json::Value>)> {
    let new_template = template::ActiveModel {
        id: Set(Uuid::new_v4()),
        directory_id: Set(directory_id),
        category_id: Set(payload.category_id),
        name: Set(payload.name),
        description: Set(payload.description),
        template_type: Set(payload.template_type),
        is_active: Set(payload.is_active),
        created_at: Set(Utc::now()),
        updated_at: Set(Utc::now()),
    };

    let template = new_template
        .insert(&db)
        .await
        .map_err(|err| {
            let (status, error_message) = match err {
                DbErr::Query(..) => (StatusCode::BAD_REQUEST, "Invalid data provided for template creation"),
                DbErr::Exec(..) => (StatusCode::INTERNAL_SERVER_ERROR, "Failed to create template in the database"),
                _ => (StatusCode::INTERNAL_SERVER_ERROR, "An unexpected error occurred"),
            };
            (status, Json(json!({"error": error_message})))
        })?;

    Ok(Json(TemplateModel::from(template)))
}

pub async fn update_template(
    State(db): State<DatabaseConnection>,
    Path((directory_id, template_id)): Path<(Uuid, Uuid)>,
    Json(payload): Json<UpdateTemplate>,
) -> Result<Json<TemplateModel>, (StatusCode, Json<serde_json::Value>)> {
    let mut template: template::ActiveModel = template::Entity::find()
        .filter(template::Column::Id.eq(template_id))
        .filter(template::Column::DirectoryId.eq(directory_id))
        .one(&db)
        .await
        .map_err(|err| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": "Failed to fetch template for update", "details": err.to_string()})),
            )
        })?
        .ok_or_else(|| (StatusCode::NOT_FOUND, Json(json!({"error": "Template not found"}))))?
        .into_active_model();
    // Update fields based on the payload
    if let name = payload.name {
        template.name = Set(name);
    }
    if let description = payload.description {
        template.description = Set(description);
    }
    if let template_type = payload.template_type {
        template.template_type = Set(template_type);
    }
    if let is_active = payload.is_active {
        template.is_active = Set(is_active);
    }


    template.updated_at = Set(Utc::now());

    let updated_template = template.update(&db).await.map_err(|err| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": "Failed to update template", "details": err.to_string()})),
        )
    })?;

    let template_model = TemplateModel::from(updated_template);

    Ok(Json(template_model))
}

pub async fn delete_template(
    Path((directory_id, template_id)): Path<(Uuid, Uuid)>,
    State(db): State<DatabaseConnection>,
) -> Result<StatusCode, (StatusCode, Json<serde_json::Value>)> {
    //  add checks here to prevent deletion if listings are based on this template

    let result = template::Entity::delete_many()
        .filter(template::Column::Id.eq(template_id))
        .filter(template::Column::DirectoryId.eq(directory_id))
        .exec(&db)
        .await
        .map_err(|err| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": "Failed to delete template", "details": err.to_string()})),
            )
        })?;

    if result.rows_affected == 0 {
        Err((StatusCode::NOT_FOUND, Json(json!({"error": "Template not found"}))))
    } else {
        Ok(StatusCode::NO_CONTENT)
    }
}


pub async fn get_user_directory_id(
    txn: &sea_orm::DatabaseTransaction,
    current_user: &user::Model
) -> Result<Uuid, (StatusCode, Json<serde_json::Value>)> {
    let user_account = crate::entities::user_account::Entity::find()
        .filter(crate::entities::user_account::Column::UserId.eq(current_user.id))
        .one(txn)
        .await
        .map_err(internal_error)?
        .ok_or_else(|| (StatusCode::NOT_FOUND, Json(json!({"error": "User account not found"}))))?;

    let profile = crate::entities::profile::Entity::find()
        .filter(crate::entities::profile::Column::AccountId.eq(user_account.account_id))
        .one(txn)
        .await
        .map_err(internal_error)?
        .ok_or_else(|| (StatusCode::NOT_FOUND, Json(json!({"error": "Profile not found"}))))?;

    Ok(profile.directory_id)
}

fn internal_error(err: impl std::error::Error) -> (StatusCode, Json<serde_json::Value>) {
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(json!({"error": err.to_string()})),
    )
}

impl From<template::Model> for TemplateModel {
    fn from(model: template::Model) -> Self {
        TemplateModel {
            id: model.id,
            directory_id: model.directory_id,
            name: model.name,
            description: model.description,
            template_type: model.template_type,
            is_active: model.is_active,
            category_id: model.category_id,
            created_at: model.created_at,
            updated_at: model.updated_at,
        }
    }
}

impl ListingModel {
    pub async fn from_insert_result(
        result: sea_orm::InsertResult<listing::ActiveModel>,
        db: &DatabaseConnection,
    ) -> Result<Self, sea_orm::DbErr> {
        let id = result.last_insert_id;
        let model = Listing::find_by_id(id)
            .one(db)
            .await?
            .expect("Failed to find inserted listing");
        Ok(Self::from_entity(model))
    }

    pub fn from_entity(model: crate::entities::listing::Model) -> Self {
        ListingModel {
            id: model.id,
            profile_id: model.profile_id,
            directory_id: model.directory_id,
            category_id: model.category_id,
            title: model.title,
            description: model.description,
            listing_type: model.listing_type,
            price: model.price,
            price_type: model.price_type,
            country: model.country,
            state: model.state,
            city: model.city,
            neighborhood: model.neighborhood,
            latitude: model.latitude,
            longitude: model.longitude,
            additional_info: model.additional_info.unwrap_or_else(|| serde_json::Value::Object(serde_json::Map::new())),
            status: model.status,
            is_featured: model.is_featured,
            is_based_on_template: model.is_based_on_template,
            based_on_template_id: model.based_on_template_id,
            is_ad_placement: model.is_ad_placement,
            is_active: model.is_active,
            created_at: model.created_at,
            updated_at: model.updated_at,
        }
    }
}
