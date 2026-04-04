use uuid::Uuid;
use chrono::Utc;
use axum::{
    extract::{Extension, Path, Json, State},
    http::StatusCode,
    response::{IntoResponse, Json as JsonResponse},
    routing::{get, post, put, delete},
    Router,
};
use sea_orm::{
    
    ActiveModelTrait, 
    Set, 
    DatabaseConnection,
     
    EntityTrait, 
    QueryFilter,
    
    ColumnTrait,
};
use serde::Deserialize;
use crate::entities::{case, activity, note, customer, user};
use crate::models::case::{CaseModel, CreateCaseInput, UpdateCaseInput};
use crate::models::activity::ActivityModel;
use crate::models::note::NoteModel;
use crate::models::file::FileAssociation;
use crate::entities::activity::{ActivityType, ActivityStatus, AssociatedEntity, AssociatedEntityType};

pub fn routes() -> Router<DatabaseConnection> {
    Router::new()
        .route("/api/cases", post(create_case))
        .route("/api/cases", get(get_cases))
        .route("/api/cases/{id}", get(get_case))
        .route("/api/cases/{id}", put(update_case))
        .route("/api/cases/{id}", delete(delete_case))
        .route("/api/cases/{id}/activities", get(get_case_activities))
        .route("/api/cases/{id}/activities", post(create_case_activity))
        .route("/api/cases/{id}/notes", get(get_case_notes))
        .route("/api/cases/{id}/notes", post(create_case_note))
        .route("/api/cases/{id}/files", get(get_case_files))
        .route("/api/cases/{id}/files/{file_id}", post(add_file_to_case))
}

pub async fn create_case(
    State(db): State<DatabaseConnection>,
    Extension(_current_user): Extension<user::Model>,
    Json(input): Json<CreateCaseInput>,
) -> Result<impl IntoResponse, StatusCode> {
    // Check if the customer exists
    let customer = customer::Entity::find_by_id(input.customer_id)
        .one(&db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;

    let new_case = case::ActiveModel {
        id: Set(Uuid::new_v4()),
        customer_id: Set(customer.id),
        title: Set(input.title),
        description: Set(input.description),
        status: Set("Open".to_string()),
        priority: Set(input.priority),
        assigned_to: Set(input.assigned_to),
        created_at: Set(Utc::now()),
        updated_at: Set(Utc::now()),
        closed_at: Set(None),
        properties: Set(None),
    };

    let inserted_case = new_case.insert(&db).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    Ok((StatusCode::CREATED, JsonResponse(CaseModel::from(inserted_case))))
}

pub async fn get_cases(
    State(db): State<DatabaseConnection>,
    Extension(_current_user): Extension<user::Model>,
) -> Result<impl IntoResponse, StatusCode> {
    let cases = case::Entity::find()
        .all(&db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let case_models: Vec<CaseModel> = cases.into_iter().map(CaseModel::from).collect();
    Ok(JsonResponse(case_models))
}

pub async fn get_case(
    State(db): State<DatabaseConnection>,
    Extension(_current_user): Extension<user::Model>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, StatusCode> {
    let case = case::Entity::find_by_id(id)
        .one(&db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;

    Ok(JsonResponse(CaseModel::from(case)))
}

pub async fn update_case(
    State(db): State<DatabaseConnection>,
    Extension(_current_user): Extension<user::Model>,
    Path(id): Path<Uuid>,
    Json(input): Json<UpdateCaseInput>,
) -> Result<impl IntoResponse, StatusCode> {
    let mut case: case::ActiveModel = case::Entity::find_by_id(id)
        .one(&db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?
        .into();

    if let Some(title) = input.title { case.title = Set(title); }
    if let Some(description) = input.description { case.description = Set(description); }
    if let Some(status) = input.status { case.status = Set(status); }
    if let Some(priority) = input.priority { case.priority = Set(priority); }
    if let Some(assigned_to) = input.assigned_to { case.assigned_to = Set(Some(assigned_to)); }
    case.updated_at = Set(Utc::now());

    let updated_case = case.update(&db).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(JsonResponse(CaseModel::from(updated_case)))
}

pub async fn delete_case(
    State(db): State<DatabaseConnection>,
    Extension(_current_user): Extension<user::Model>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, StatusCode> {
    let result = case::Entity::delete_by_id(id)
        .exec(&db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    if result.rows_affected == 0 {
        return Err(StatusCode::NOT_FOUND);
    }

    Ok(StatusCode::NO_CONTENT)
}

pub async fn get_case_activities(
    State(db): State<DatabaseConnection>,
    Extension(_current_user): Extension<user::Model>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, StatusCode> {
    let activities = activity::Entity::find()
        .filter(activity::Column::CaseId.eq(id))
        .all(&db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let activity_models: Vec<ActivityModel> = activities.into_iter().map(ActivityModel::from).collect();
    Ok(JsonResponse(activity_models))
}

pub async fn create_case_activity(
    State(db): State<DatabaseConnection>,
    Extension(current_user): Extension<user::Model>,
    Path(id): Path<Uuid>,
    Json(input): Json<CreateCaseActivityInput>,
) -> Result<impl IntoResponse, StatusCode> {
    let case = case::Entity::find_by_id(id)
        .one(&db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;

    let new_activity = activity::ActiveModel {
        id: Set(Uuid::new_v4()),
        account_id: Set(None),
        case_id: Set(Some(case.id)),
        customer_id: Set(Some(case.customer_id)),
        deal_id: Set(None),
        lead_id: Set(None),
        contact_id: Set(None),
        activity_type: Set(ActivityType::Task),
        title: Set(input.title),
        description: Set(input.description),
        status: Set(ActivityStatus::Pending),
        due_date: Set(None),
        completed_at: Set(None),
        associated_entities: Set(serde_json::to_value(vec![
            AssociatedEntity {
                entity_type: AssociatedEntityType::Case,
                entity_id: case.id,
            }
        ]).unwrap()),
        created_by: Set(current_user.id),
        assigned_to: Set(case.assigned_to),
        created_at: Set(Utc::now()),
        updated_at: Set(Utc::now()),
    };

    let inserted_activity = new_activity
        .insert(&db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    Ok((StatusCode::CREATED, JsonResponse(ActivityModel::from(inserted_activity))))
}

pub async fn add_file_to_case(
    State(db): State<DatabaseConnection>,
    Extension(_current_user): Extension<user::Model>,
    Path((case_id, file_id)): Path<(Uuid, Uuid)>,
) -> Result<impl IntoResponse, StatusCode> {
    let case = case::Entity::find_by_id(case_id)
        .one(&db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;

    case.add_file(&db, file_id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(StatusCode::OK)
}

pub async fn get_case_files(
    State(db): State<DatabaseConnection>,
    Extension(_current_user): Extension<user::Model>,
    Path(case_id): Path<Uuid>,
) -> Result<impl IntoResponse, StatusCode> {
    let case = case::Entity::find_by_id(case_id)
        .one(&db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;

    let file_ids = case.get_associated_files(&db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(file_ids))
}

pub async fn get_case_notes(
    State(db): State<DatabaseConnection>,
    Extension(_current_user): Extension<user::Model>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, StatusCode> {
    let case = case::Entity::find_by_id(id)
        .one(&db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;

    let notes = note::Entity::find()
        .filter(note::Column::EntityType.eq("Case"))
        .filter(note::Column::EntityId.eq(case.id))
        .all(&db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let note_models: Vec<NoteModel> = notes.into_iter().map(NoteModel::from).collect();
    Ok(JsonResponse(note_models))
}

pub async fn create_case_note(
    State(db): State<DatabaseConnection>,
    Extension(current_user): Extension<user::Model>,
    Path(id): Path<Uuid>,
    Json(input): Json<CreateCaseNoteInput>,
) -> Result<impl IntoResponse, StatusCode> {
    let case = case::Entity::find_by_id(id)
        .one(&db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;

    let new_note = note::ActiveModel {
        id: Set(Uuid::new_v4()),
        content: Set(input.content),
        created_by: Set(current_user.id),
        entity_type: Set("Case".to_string()),
        entity_id: Set(case.id),
        created_at: Set(Utc::now()),
        updated_at: Set(Utc::now()),
    };

    let inserted_note = new_note.insert(&db).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    Ok((StatusCode::CREATED, JsonResponse(NoteModel::from(inserted_note))))
}

#[derive(Debug, Deserialize)]
struct CreateCaseActivityInput {
    title: String,
    description: Option<String>,
}

#[derive(Debug, Deserialize)]
struct CreateCaseNoteInput {
    content: String,
}
