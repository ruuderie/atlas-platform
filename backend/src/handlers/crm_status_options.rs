use axum::{
    extract::{Extension, Path, Query, Json},
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
use serde::Deserialize;

use crate::entities::{crm_status_option, user, user_account, profile};
use crate::models::crm_status_option::{
    CrmStatusOptionModel, CreateCrmStatusOptionInput, UpdateCrmStatusOptionInput
};

#[derive(Debug, Deserialize)]
pub struct StatusOptionQuery {
    pub object_type: Option<String>,
}

pub fn authenticated_routes() -> Router<DatabaseConnection> {
    Router::new()
        .route("/api/crm/status-options", get(get_status_options))
        .route("/api/crm/status-options", post(create_status_option))
        .route("/api/crm/status-options/{id}", put(update_status_option))
        .route("/api/crm/status-options/{id}", delete(delete_status_option))
}

async fn get_status_tenant_id(
    db: &DatabaseConnection,
    user_id: Uuid,
) -> Result<Uuid, StatusCode> {
    let user_accounts = user_account::Entity::find()
        .filter(user_account::Column::UserId.eq(user_id))
        .all(db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let account_ids: Vec<Uuid> = user_accounts.into_iter().map(|ua| ua.account_id).collect();
    
    let profile = profile::Entity::find()
        .filter(profile::Column::AccountId.is_in(account_ids))
        .one(db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::FORBIDDEN)?;
        
    Ok(profile.tenant_id)
}

pub async fn get_status_options(
    Extension(db): Extension<DatabaseConnection>,
    Extension(current_user): Extension<user::Model>,
    Query(query): Query<StatusOptionQuery>,
) -> Result<impl IntoResponse, StatusCode> {
    let tenant_id = get_status_tenant_id(&db, current_user.id).await?;

    let mut query_builder = crm_status_option::Entity::find()
        .filter(crm_status_option::Column::TenantId.eq(tenant_id));

    if let Some(obj_type) = query.object_type {
        query_builder = query_builder.filter(crm_status_option::Column::ObjectType.eq(obj_type));
    }

    let options = query_builder
        .all(&db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let mut option_models: Vec<CrmStatusOptionModel> = options.into_iter().map(CrmStatusOptionModel::from).collect();
    // Sort by sort_order ascending
    option_models.sort_by_key(|o| o.sort_order);

    Ok(JsonResponse(option_models))
}

pub async fn create_status_option(
    Extension(db): Extension<DatabaseConnection>,
    Extension(current_user): Extension<user::Model>,
    Json(input): Json<CreateCrmStatusOptionInput>,
) -> Result<impl IntoResponse, StatusCode> {
    let tenant_id = get_status_tenant_id(&db, current_user.id).await?;

    // Check duplicate key per tenant/object
    let duplicate = crm_status_option::Entity::find()
        .filter(crm_status_option::Column::TenantId.eq(tenant_id))
        .filter(crm_status_option::Column::ObjectType.eq(&input.object_type))
        .filter(crm_status_option::Column::StatusKey.eq(&input.status_key))
        .one(&db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    if duplicate.is_some() {
        return Err(StatusCode::CONFLICT);
    }

    let color = input.color.unwrap_or_else(|| "slate".to_string());
    let sort_order = input.sort_order.unwrap_or(10);

    let new_option = crm_status_option::ActiveModel {
        id: Set(Uuid::new_v4()),
        tenant_id: Set(tenant_id),
        object_type: Set(input.object_type),
        status_key: Set(input.status_key),
        label: Set(input.label),
        color: Set(color),
        sort_order: Set(sort_order),
        is_system: Set(false),
        created_at: Set(Utc::now().into()),
        updated_at: Set(Utc::now().into()),
    };

    let opt = new_option.insert(&db).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok((StatusCode::CREATED, JsonResponse(CrmStatusOptionModel::from(opt))))
}

pub async fn update_status_option(
    Extension(db): Extension<DatabaseConnection>,
    Extension(current_user): Extension<user::Model>,
    Path(id): Path<Uuid>,
    Json(input): Json<UpdateCrmStatusOptionInput>,
) -> Result<impl IntoResponse, StatusCode> {
    let tenant_id = get_status_tenant_id(&db, current_user.id).await?;

    let existing = crm_status_option::Entity::find_by_id(id)
        .one(&db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;

    if existing.tenant_id != tenant_id {
        return Err(StatusCode::FORBIDDEN);
    }

    let mut active: crm_status_option::ActiveModel = existing.into();

    if let Some(label) = input.label {
        active.label = Set(label);
    }
    if let Some(color) = input.color {
        active.color = Set(color);
    }
    if let Some(sort_order) = input.sort_order {
        active.sort_order = Set(sort_order);
    }
    active.updated_at = Set(Utc::now().into());

    let updated = active.update(&db).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(JsonResponse(CrmStatusOptionModel::from(updated)))
}

pub async fn delete_status_option(
    Extension(db): Extension<DatabaseConnection>,
    Extension(current_user): Extension<user::Model>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, StatusCode> {
    let tenant_id = get_status_tenant_id(&db, current_user.id).await?;

    let existing = crm_status_option::Entity::find_by_id(id)
        .one(&db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;

    if existing.tenant_id != tenant_id {
        return Err(StatusCode::FORBIDDEN);
    }

    if existing.is_system {
        // System options like 'converted' cannot be dropped
        return Err(StatusCode::BAD_REQUEST);
    }

    existing.delete(&db).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(StatusCode::NO_CONTENT)
}
