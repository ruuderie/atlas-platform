#![allow(dead_code)]
use axum::{
    extract::{Extension, Path, Json, Query},
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
use std::collections::HashMap;

use crate::entities::{activity, user};
// ============================================================
// LEGACY CRM HANDLER - CUTOVER IN PROGRESS
// Activities being consolidated into Case + Realtime + Audit services.
// ============================================================

use crate::entities::activity::{ActivityType, ActivityStatus};

// Cutover in progress — activities moving to Case + Realtime + Audit
use crate::models::activity::{ActivityModel, CreateActivityInput, UpdateActivityInput};
use crate::models::file::FileAssociation;
use crate::handlers::notes::get_user_tenant_id;

pub fn routes() -> Router<DatabaseConnection> {
    Router::new()
        .route("/api/crm/activities", post(create_activity))
        .route("/api/crm/activities", get(get_activities))
        .route("/api/crm/activities/{id}", get(get_activity))
        .route("/api/crm/activities/{id}", put(update_activity))
        .route("/api/crm/activities/{id}", delete(delete_activity))
        .route("/api/crm/activities/{id}/files", get(get_activity_files))
}

pub async fn create_activity(
    Extension(db): Extension<DatabaseConnection>,
    Extension(current_user): Extension<user::Model>,
    Json(input): Json<CreateActivityInput>,
) -> Result<impl IntoResponse, StatusCode> {
    let tenant_id = get_user_tenant_id(&db, current_user.id).await?;

    let mut completed_at = None;
    let status = match input.activity_type {
        ActivityType::Log => {
            completed_at = Some(input.completed_at.unwrap_or_else(Utc::now));
            ActivityStatus::Completed
        }
        ActivityType::Task | ActivityType::Event => {
            if input.due_date.is_none() {
                return Err(StatusCode::BAD_REQUEST);
            }
            if input.status == ActivityStatus::Completed {
                completed_at = Some(input.completed_at.unwrap_or_else(Utc::now));
            }
            input.status
        }
    };

    let new_activity = activity::ActiveModel {
        id: Set(Uuid::new_v4()),
        tenant_id: Set(Some(tenant_id)),
        account_id: Set(input.account_id),
        deal_id: Set(input.deal_id),
        customer_id: Set(input.customer_id),
        lead_id: Set(input.lead_id),
        contact_id: Set(input.contact_id),
        case_id: Set(input.case_id),
        activity_type: Set(input.activity_type),
        title: Set(input.title),
        description: Set(input.description),
        status: Set(status),
        due_date: Set(input.due_date),
        completed_at: Set(completed_at),
        associated_entities: Set(serde_json::to_value(input.associated_entities).unwrap()),
        created_by: Set(current_user.id),
        assigned_to: Set(input.assigned_to),
        created_at: Set(Utc::now()),
        updated_at: Set(Utc::now()),
    };

    let activity = new_activity.insert(&db).await.map_err(|e| {
        tracing::error!("Failed to insert activity: {:?}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;
    
    // Associate files with the activity, auto-inserting the File record if needed
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
            new_file_db.insert(&db).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        }
        activity.add_file(&db, file.id).await.map_err(|e| {
            tracing::error!("Failed to add file to activity: {:?}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;
    }
    
    let model = ActivityModel::from_with_files(activity, &db).await;
    Ok((StatusCode::CREATED, JsonResponse(model)))
}

pub async fn get_activities(
    Extension(db): Extension<DatabaseConnection>,
    Extension(current_user): Extension<user::Model>,
    Query(params): Query<HashMap<String, String>>,
) -> Result<impl IntoResponse, StatusCode> {
    let tenant_id = get_user_tenant_id(&db, current_user.id).await?;
    let mut query = activity::Entity::find()
        .filter(activity::Column::TenantId.eq(tenant_id));

    // Polymorphic entity filters
    if let (Some(entity_type), Some(entity_id_str)) = (params.get("entity_type"), params.get("entity_id")) {
        if let Ok(entity_id) = Uuid::parse_str(entity_id_str) {
            match entity_type.as_str() {
                "Contact" => query = query.filter(activity::Column::ContactId.eq(entity_id)),
                "Lead" => query = query.filter(activity::Column::LeadId.eq(entity_id)),
                "Deal" => query = query.filter(activity::Column::DealId.eq(entity_id)),
                "Customer" => query = query.filter(activity::Column::CustomerId.eq(entity_id)),
                "Case" => query = query.filter(activity::Column::CaseId.eq(entity_id)),
                "Account" => query = query.filter(activity::Column::AccountId.eq(entity_id)),
                _ => {}
            }
        }
    } else {
        // Fallback to legacy individual query parameters if entity_type / entity_id are not provided
        if let Some(account_id) = params.get("account_id") {
            if let Ok(id) = Uuid::parse_str(account_id) {
                query = query.filter(activity::Column::AccountId.eq(id));
            }
        }
        if let Some(deal_id) = params.get("deal_id") {
            if let Ok(id) = Uuid::parse_str(deal_id) {
                query = query.filter(activity::Column::DealId.eq(id));
            }
        }
        if let Some(customer_id) = params.get("customer_id") {
            if let Ok(id) = Uuid::parse_str(customer_id) {
                query = query.filter(activity::Column::CustomerId.eq(id));
            }
        }
        if let Some(lead_id) = params.get("lead_id") {
            if let Ok(id) = Uuid::parse_str(lead_id) {
                query = query.filter(activity::Column::LeadId.eq(id));
            }
        }
        if let Some(contact_id) = params.get("contact_id") {
            if let Ok(id) = Uuid::parse_str(contact_id) {
                query = query.filter(activity::Column::ContactId.eq(id));
            }
        }
        if let Some(case_id) = params.get("case_id") {
            if let Ok(id) = Uuid::parse_str(case_id) {
                query = query.filter(activity::Column::CaseId.eq(id));
            }
        }
    }

    let activities = query.all(&db).await.map_err(|e| {
        tracing::error!("Failed to fetch activities: {:?}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    let mut activity_models = Vec::new();
    for act in activities {
        activity_models.push(ActivityModel::from_with_files(act, &db).await);
    }

    Ok(JsonResponse(activity_models))
}

pub async fn get_activity(
    Extension(db): Extension<DatabaseConnection>,
    Extension(current_user): Extension<user::Model>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, StatusCode> {
    let tenant_id = get_user_tenant_id(&db, current_user.id).await?;
    let activity = activity::Entity::find_by_id(id)
        .one(&db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;

    if activity.tenant_id != Some(tenant_id) {
        return Err(StatusCode::NOT_FOUND);
    }

    Ok(JsonResponse(ActivityModel::from_with_files(activity, &db).await))
}

pub async fn update_activity(
    Extension(db): Extension<DatabaseConnection>,
    Extension(current_user): Extension<user::Model>,
    Path(id): Path<Uuid>,
    Json(input): Json<UpdateActivityInput>,
) -> Result<impl IntoResponse, StatusCode> {
    let tenant_id = get_user_tenant_id(&db, current_user.id).await?;

    let activity = activity::Entity::find_by_id(id)
        .one(&db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;

    if activity.tenant_id != Some(tenant_id) {
        return Err(StatusCode::NOT_FOUND);
    }

    let final_type = input.activity_type.clone().unwrap_or_else(|| activity.activity_type.clone());
    let final_due_date = input.due_date.or(activity.due_date);

    if (final_type == ActivityType::Task || final_type == ActivityType::Event) && final_due_date.is_none() {
        return Err(StatusCode::BAD_REQUEST);
    }

    // Create an ActiveModel from the existing model
    let mut activity_active: activity::ActiveModel = activity.clone().into();

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

    // Handle status transitions and completed_at
    let mut completed_at_set = activity.completed_at;
    if let Some(status) = input.status {
        if status == ActivityStatus::Completed {
            if activity.status != ActivityStatus::Completed {
                completed_at_set = Some(input.completed_at.unwrap_or_else(Utc::now));
            } else if let Some(new_completed_at) = input.completed_at {
                completed_at_set = Some(new_completed_at);
            }
        } else {
            completed_at_set = None;
        }
        activity_active.status = Set(status);
    } else if let Some(new_completed_at) = input.completed_at {
        completed_at_set = Some(new_completed_at);
    }
    activity_active.completed_at = Set(completed_at_set);

    if let Some(due_date) = input.due_date {
        activity_active.due_date = Set(Some(due_date));
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
        .map_err(|e| {
            tracing::error!("Failed to update activity: {:?}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    // Handle files updating if provided (similar to note updating)
    if let Some(file_ids) = input.files {
        // Disassociate previous files
        let current_files = activity.get_associated_files(&db).await.unwrap_or_default();
        for f in current_files {
            activity.remove_file(&db, f.id).await.map_err(|e| {
                tracing::error!("Failed to remove file from activity: {:?}", e);
                StatusCode::INTERNAL_SERVER_ERROR
            })?;
        }
        // Associate new files
        for fid in file_ids {
            activity.add_file(&db, fid).await.map_err(|e| {
                tracing::error!("Failed to add file to activity: {:?}", e);
                StatusCode::INTERNAL_SERVER_ERROR
            })?;
        }
    }

    let model = ActivityModel::from_with_files(updated_activity, &db).await;
    Ok(JsonResponse(model))
}

pub async fn delete_activity(
    Extension(db): Extension<DatabaseConnection>,
    Extension(current_user): Extension<user::Model>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, StatusCode> {
    let tenant_id = get_user_tenant_id(&db, current_user.id).await?;
    let activity = activity::Entity::find_by_id(id)
        .one(&db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;

    if activity.tenant_id != Some(tenant_id) {
        return Err(StatusCode::NOT_FOUND);
    }

    activity.delete(&db).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(StatusCode::NO_CONTENT)
}

pub async fn get_activity_files(
    Extension(db): Extension<DatabaseConnection>,
    Extension(current_user): Extension<user::Model>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, StatusCode> {
    let tenant_id = get_user_tenant_id(&db, current_user.id).await?;
    let activity = activity::Entity::find_by_id(id)
        .one(&db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;

    if activity.tenant_id != Some(tenant_id) {
        return Err(StatusCode::NOT_FOUND);
    }

    let files = activity.get_associated_files(&db).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(JsonResponse(files))
}

pub async fn get_activity_notes(
    Extension(db): Extension<DatabaseConnection>,
    Extension(current_user): Extension<user::Model>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, StatusCode> {
    let tenant_id = get_user_tenant_id(&db, current_user.id).await?;
    let activity = activity::Entity::find_by_id(id)
        .one(&db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;

    if activity.tenant_id != Some(tenant_id) {
        return Err(StatusCode::NOT_FOUND);
    }

    Ok(JsonResponse(ActivityModel::from_with_files(activity, &db).await))
}
