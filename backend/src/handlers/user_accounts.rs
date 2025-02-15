// src/handlers/user_accounts.rs

use axum::{
    extract::{Extension, Path, Json},
    http::StatusCode,
    response::{IntoResponse, Json as JsonResponse},
    routing::{get, post, put, delete},
    Router,
};
use sea_orm::{
    DatabaseConnection, EntityTrait, QueryFilter, Set, ColumnTrait,
    ActiveModelTrait, ModelTrait,
};
use crate::entities::{
    user_account, user, account,
};
use crate::models::user_account::*;
use uuid::Uuid;
use chrono::Utc;

pub fn routes() -> Router<DatabaseConnection> {
    Router::new()
        .route("/user-accounts", post(create_user_account))
        .route("/user-accounts", get(get_user_accounts))
        .route("/user-accounts/:id", get(get_user_account))
        .route("/user-accounts/:id", put(update_user_account))
        .route("/user-accounts/:id", delete(delete_user_account))
}

pub async fn create_user_account(
    Extension(db): Extension<DatabaseConnection>,
    Json(payload): Json<UserAccountCreate>,
) -> Result<impl IntoResponse, StatusCode> {
    tracing::info!("Creating new user account: {:?}", payload);

    // Check if user exists
    let user = user::Entity::find_by_id(payload.user_id)
        .one(&db)
        .await
        .map_err(|e| {
            tracing::error!("Error fetching user: {:?}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?
        .ok_or_else(|| {
            tracing::warn!("User not found: {}", payload.user_id);
            StatusCode::NOT_FOUND
        })?;

    // Check if account exists
    let account = account::Entity::find_by_id(payload.account_id)
        .one(&db)
        .await
        .map_err(|e| {
            tracing::error!("Error fetching account: {:?}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?
        .ok_or_else(|| {
            tracing::warn!("Account not found: {}", payload.account_id);
            StatusCode::NOT_FOUND
        })?;

    let new_user_account = user_account::ActiveModel {
        id: Set(Uuid::new_v4()),
        user_id: Set(user.id),
        account_id: Set(account.id),
        role: Set(payload.role),
        is_active: Set(true),
        created_at: Set(Utc::now()),
        updated_at: Set(Utc::now()),
    };

    let inserted_user_account = new_user_account.insert(&db).await.map_err(|e| {
        tracing::error!("Error creating user account: {:?}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    Ok((StatusCode::CREATED, JsonResponse(inserted_user_account)))
}

pub async fn get_user_accounts(
    Extension(db): Extension<DatabaseConnection>,
) -> Result<impl IntoResponse, StatusCode> {
    tracing::info!("Fetching all user accounts");

    let user_accounts = user_account::Entity::find()
        .all(&db)
        .await
        .map_err(|e| {
            tracing::error!("Error fetching user accounts: {:?}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    Ok((StatusCode::OK, JsonResponse(user_accounts)))
}

pub async fn get_user_account(
    Extension(db): Extension<DatabaseConnection>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, StatusCode> {
    tracing::info!("Fetching user account: {}", id);

    let user_account = user_account::Entity::find_by_id(id)
        .one(&db)
        .await
        .map_err(|e| {
            tracing::error!("Error fetching user account: {:?}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?
        .ok_or_else(|| {
            tracing::warn!("User account not found: {}", id);
            StatusCode::NOT_FOUND
        })?;

    Ok((StatusCode::OK, JsonResponse(user_account)))
}

pub async fn update_user_account(
    Extension(db): Extension<DatabaseConnection>,
    Path(id): Path<Uuid>,
    Json(payload): Json<UserAccountUpdate>,
) -> Result<impl IntoResponse, StatusCode> {
    tracing::info!("Updating user account: {}", id);

    let user_account = user_account::Entity::find_by_id(id)
        .one(&db)
        .await
        .map_err(|e| {
            tracing::error!("Error fetching user account: {:?}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?
        .ok_or_else(|| {
            tracing::warn!("User account not found: {}", id);
            StatusCode::NOT_FOUND
        })?;

    let mut user_account: user_account::ActiveModel = user_account.into();
    user_account.role = Set(payload.role);
    user_account.is_active = Set(payload.is_active);
    user_account.updated_at = Set(Utc::now());

    let updated_user_account = user_account.update(&db).await.map_err(|e| {
        tracing::error!("Error updating user account: {:?}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    Ok((StatusCode::OK, JsonResponse(updated_user_account)))
}

pub async fn delete_user_account(
    Extension(db): Extension<DatabaseConnection>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, StatusCode> {
    tracing::info!("Deleting user account: {}", id);

    let result = user_account::Entity::delete_by_id(id)
        .exec(&db)
        .await
        .map_err(|e| {
            tracing::error!("Error deleting user account: {:?}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    if result.rows_affected == 0 {
        tracing::warn!("User account not found: {}", id);
        return Err(StatusCode::NOT_FOUND);
    }

    Ok(StatusCode::NO_CONTENT)
}