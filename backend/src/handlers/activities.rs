use axum::{
    extract::{Extension, Path, Json, Query},
    http::StatusCode,
    response::{IntoResponse, Json as JsonResponse},
    routing::{get, post, put, delete},
    Router,
};
use sea_orm::{
    DatabaseConnection, EntityTrait, QueryFilter, Set, ColumnTrait,
    ActiveModelTrait, ModelTrait, PaginatorTrait,
};
use uuid::Uuid;
use chrono::Utc;
use std::collections::HashMap;
use crate::entities::{activity, user};
use crate::models::activity::{ActivityModel, CreateActivityInput, UpdateActivityInput};
use crate::models::file::FileAssociation;

pub fn routes() -> Router {
    Router::new()
        .route("/activities", post(create_activity))
        .route("/activities", get(get_activities))
        .route("/activities/{id}", get(get_activity))
        .route("/activities/{id}", put(update_activity))
        .route("/activities/{id}", delete(delete_activity))
        .route("/activities/{id}/files", get(get_activity_files))
}

pub async fn create_activity(
    Extension(db): Extension<DatabaseConnection>,
    Extension(current_user): Extension<user::Model>,
    Json(input): Json<CreateActivityInput>,
) -> Result<impl IntoResponse, StatusCode> {
    let new_activity = activity::ActiveModel {
        id: Set(Uuid::new_v4()),
        account_id: Set(Some(input.account_id)),
        deal_id: Set(input.deal_id),
        customer_id: Set(input.customer_id),
        lead_id: Set(input.lead_id),
        contact_id: Set(input.contact_id),
        case_id: Set(input.case_id),
        activity_type: Set(input.activity_type),
        title: Set(input.title),
        description: Set(input.description),
        status: Set(input.status),
        due_date: Set(input.due_date),
        completed_at: Set(None),
        associated_entities: Set(serde_json::to_value(input.associated_entities).unwrap()),
        created_by: Set(current_user.id),
        assigned_to: Set(input.assigned_to),
        created_at: Set(Utc::now()),
        updated_at: Set(Utc::now()),
    };

    let activity = new_activity.insert(&db).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    // Associate files with the activity
    for file_id in input.files {
        activity.add_file(&db, file_id.id).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    }
    
    Ok((StatusCode::CREATED, JsonResponse(ActivityModel::from(activity))))
}

pub async fn get_activities(
    Extension(db): Extension<DatabaseConnection>,
    Query(params): Query<HashMap<String, String>>,
) -> Result<impl IntoResponse, StatusCode> {
    let mut query = activity::Entity::find();

    // Add filters based on query parameters
    if let Some(account_id) = params.get("account_id") {
        query = query.filter(activity::Column::AccountId.eq(Uuid::parse_str(account_id).unwrap()));
    }
    // Add more filters for other fields as needed

    let page: u64 = params.get("page").unwrap_or(&"1".to_string()).parse().unwrap_or(1);
    let items_per_page: u64 = params.get("items_per_page").unwrap_or(&"10".to_string()).parse().unwrap_or(10);

    let paginator = query.paginate(&db, items_per_page);
    let activities = paginator.fetch_page(page).await.unwrap();

    Ok(Json(activities))
}

pub async fn get_activity(
    Extension(db): Extension<DatabaseConnection>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, StatusCode> {
    let activity = activity::Entity::find_by_id(id).one(&db).await.map_err(|_| StatusCode::NOT_FOUND)?;

    Ok(Json(activity))
}

pub async fn update_activity(
    Extension(db): Extension<DatabaseConnection>,
    Path(id): Path<Uuid>,
    Json(input): Json<UpdateActivityInput>,
) -> Result<impl IntoResponse, StatusCode> {
    let activity = activity::Entity::find_by_id(id)
        .one(&db)
        .await
        .map_err(|_| StatusCode::NOT_FOUND)?
        .ok_or(StatusCode::NOT_FOUND)?;

    // Create an ActiveModel from the existing model
    let mut activity_active: activity::ActiveModel = activity.clone().into();

    // Update only the fields that are present in the input
    if let Some(deal_id) = input.deal_id {
        activity_active.deal_id = Set(Some(deal_id));
    }
    if let Some(customer_id) = input.customer_id {
        activity_active.customer_id = Set(Some(customer_id));
    }
    if let Some(lead_id) = input.lead_id {
        activity_active.lead_id = Set(Some(lead_id));
    }
    if let Some(contact_id) = input.contact_id {
        activity_active.contact_id = Set(Some(contact_id));
    }
    if let Some(case_id) = input.case_id {
        activity_active.case_id = Set(Some(case_id));
    }
    if let Some(activity_type) = input.activity_type {
        activity_active.activity_type = Set(activity_type);
    }
    if let Some(title) = input.title {
        activity_active.title = Set(title);
    }
    if let Some(description) = input.description {
        activity_active.description = Set(Some(description));
    }
    if let Some(status) = input.status {
        activity_active.status = Set(status);
    }
    if let Some(due_date) = input.due_date {
        activity_active.due_date = Set(Some(due_date));
    }
    if let Some(completed_at) = input.completed_at {
        activity_active.completed_at = Set(Some(completed_at));
    }
    if let Some(associated_entities) = input.associated_entities {
        activity_active.associated_entities = Set(serde_json::to_value(associated_entities).unwrap());
    }
    if let Some(assigned_to) = input.assigned_to {
        activity_active.assigned_to = Set(Some(assigned_to));
    }
    
    activity_active.updated_at = Set(Utc::now());

    let updated_activity = activity_active
        .update(&db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(JsonResponse(ActivityModel::from(updated_activity)))
}

pub async fn delete_activity(
    Extension(db): Extension<DatabaseConnection>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, StatusCode> {
    let activity = activity::Entity::find_by_id(id).one(&db).await.map_err(|_| StatusCode::NOT_FOUND)?;
    if let Some(activity) = activity {
        activity.delete(&db).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    } else {
        return Err(StatusCode::NOT_FOUND);
    }
    Ok(())
}

pub async fn get_activity_files(
    Extension(db): Extension<DatabaseConnection>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, StatusCode> {
    let activity = activity::Entity::find_by_id(id)
        .one(&db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;

    let files = activity.get_associated_files(&db).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(JsonResponse(files))
}

pub async fn get_activity_notes(
    Extension(db): Extension<DatabaseConnection>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, StatusCode> {
    let activity = activity::Entity::find_by_id(id).one(&db).await.map_err(|_| StatusCode::NOT_FOUND)?;

    Ok(Json(activity))
}
