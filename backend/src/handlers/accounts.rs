use axum::{
    extract::{Extension, Path, Json},
    http::StatusCode,
    response::{IntoResponse, Json as JsonResponse},
    routing::{get, post, put, delete},
    Router,
};
use sea_orm::{
    DatabaseConnection, EntityTrait, QueryFilter, Set, ColumnTrait,
    InsertResult, ActiveModelTrait, ModelTrait,
};
use crate::entities::{
    account, user_account, user, lead_charge,
};
use crate::models::user_account::*;
use uuid::Uuid;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use crate::entities::user_account::UserRole;

#[derive(Deserialize, Clone)]
pub struct CreateAccountDto {
    name: String,
}

#[derive(Deserialize, Clone)]
pub struct AddUserToAccountDto {
    user_id: Uuid,
    role: UserRole,
}

#[derive(Serialize, Clone)]
pub struct AccountResponse {
    id: Uuid,
    name: String,
    created_at: chrono::DateTime<Utc>,
}

impl Default for AccountResponse {
    fn default() -> Self {
        AccountResponse {
            id: Uuid::nil(),
            name: String::new(),
            created_at: Utc::now(),
        }
    }
}

pub fn routes() -> Router<DatabaseConnection> {
    Router::new()
        .route("/api/accounts", post(create_account))
        .route("/api/accounts", get(get_accounts))
        .route("/api/accounts/{id}", get(get_account))
        .route("/api/accounts/{id}", put(update_account))
        .route("/api/accounts/{id}", delete(delete_account))
        .route("/api/accounts/{id}/users", post(add_user_to_account))
        .route("/api/accounts/{id}/users", get(get_account_users))
        .route("/api/accounts/{id}/ledger", get(get_account_ledger))
        .route("/api/accounts/{account_id}/users/{user_id}", delete(remove_user_from_account))
        .route("/api/accounts/{account_id}/users/{user_id}/role", put(update_user_role_in_account))
}

pub async fn create_account(
    Extension(db): Extension<DatabaseConnection>,
    Json(payload): Json<CreateAccountDto>,
) -> impl IntoResponse {
    tracing::info!("Creating new account: {}", payload.name);

    let new_account = account::ActiveModel {
        id: Set(Uuid::new_v4()),
        name: Set(payload.name.clone()),
        created_at: Set(Utc::now()),
        updated_at: Set(Utc::now()),
        ..Default::default()
    };

    match account::Entity::insert(new_account).exec(&db).await {
        Ok(res) => {
            let account_response = AccountResponse {
                id: res.last_insert_id,
                name: payload.name,
                created_at: Utc::now(),
            };
            (StatusCode::CREATED, JsonResponse(account_response))
        }
        Err(err) => {
            tracing::error!("Error creating account: {:?}", err);
            (StatusCode::INTERNAL_SERVER_ERROR, JsonResponse(AccountResponse::default()))
        }
    }
}

pub async fn get_account(
    Extension(db): Extension<DatabaseConnection>,
    Path(account_id): Path<Uuid>,
) -> impl IntoResponse {
    tracing::info!("Fetching account: {}", account_id);

    match account::Entity::find_by_id(account_id).one(&db).await {
        Ok(Some(account)) => {
            let account_response = AccountResponse {
                id: account.id,
                name: account.name,
                created_at: account.created_at,
            };
            (StatusCode::OK, JsonResponse(account_response))
        }
        Ok(None) => {
            tracing::warn!("Account not found: {}", account_id);
            (StatusCode::NOT_FOUND, JsonResponse(AccountResponse::default()))
        }
        Err(err) => {
            tracing::error!("Error fetching account: {:?}", err);
            (StatusCode::INTERNAL_SERVER_ERROR, JsonResponse(AccountResponse::default()))
        }
    }
}

pub async fn get_accounts(
    Extension(db): Extension<DatabaseConnection>,
) -> impl IntoResponse {
    tracing::info!("Fetching all accounts");

    match account::Entity::find().all(&db).await {
        Ok(accounts) => (StatusCode::OK, JsonResponse(accounts)),
        Err(err) => {
            tracing::error!("Error fetching accounts: {:?}", err);
            (StatusCode::INTERNAL_SERVER_ERROR, JsonResponse(Vec::new()))
        }
    }
}

pub async fn update_account(
    Extension(db): Extension<DatabaseConnection>,
    Path(account_id): Path<Uuid>,
    Json(payload): Json<CreateAccountDto>,
) -> impl IntoResponse {
    tracing::info!("Updating account: {}", account_id);

    let account = match account::Entity::find_by_id(account_id).one(&db).await {
        Ok(Some(account)) => account,
        Ok(None) => {
            tracing::warn!("Account not found: {}", account_id);
            return (StatusCode::NOT_FOUND, JsonResponse(()));
        }
        Err(err) => {
            tracing::error!("Error fetching account: {:?}", err);
            return (StatusCode::INTERNAL_SERVER_ERROR, JsonResponse(()));
        }
    };

    let mut account: account::ActiveModel = account.into();
    account.name = Set(payload.name);
    account.updated_at = Set(Utc::now());

    match account.update(&db).await {
        Ok(updated) => {
            let account_response = AccountResponse {
                id: updated.id,
                name: updated.name,
                created_at: updated.created_at,
            };
            
            (StatusCode::OK, JsonResponse(()))
        }
        Err(err) => {
            tracing::error!("Error updating account: {:?}", err);
            (StatusCode::INTERNAL_SERVER_ERROR, JsonResponse(()))
        }
    }
}

pub async fn delete_account(
    Extension(db): Extension<DatabaseConnection>,
    Path(account_id): Path<Uuid>,
) -> impl IntoResponse {
    tracing::info!("Deleting account: {}", account_id);

    match account::Entity::delete_by_id(account_id).exec(&db).await {
        Ok(_) => StatusCode::NO_CONTENT,
        Err(err) => {
            tracing::error!("Error deleting account: {:?}", err);
            StatusCode::INTERNAL_SERVER_ERROR
        }
    }
}

pub async fn add_user_to_account(
    Extension(db): Extension<DatabaseConnection>,
    Path(account_id): Path<Uuid>,
    Json(payload): Json<AddUserToAccountDto>,
) -> impl IntoResponse {
    tracing::info!("Adding user {} to account {}", payload.user_id, account_id);

    let new_user_account = user_account::ActiveModel {
        id: Set(Uuid::new_v4()),
        user_id: Set(payload.user_id),
        account_id: Set(account_id),
        role: Set(payload.role),
        is_active: Set(true),
        created_at: Set(Utc::now()),
        updated_at: Set(Utc::now()),
    };

    match user_account::Entity::insert(new_user_account).exec(&db).await {
        Ok(_) => StatusCode::CREATED,
        Err(err) => {
            tracing::error!("Error adding user to account: {:?}", err);
            StatusCode::INTERNAL_SERVER_ERROR
        }
    }
}

pub async fn get_account_users(
    Extension(db): Extension<DatabaseConnection>,
    Path(account_id): Path<Uuid>,
) -> impl IntoResponse {
    tracing::info!("Fetching users for account: {}", account_id);

    match user_account::Entity::find()
        .filter(user_account::Column::AccountId.eq(account_id))
        .all(&db)
        .await
    {
        Ok(user_accounts) => {
            let user_ids: Vec<Uuid> = user_accounts.iter().map(|ua| ua.user_id).collect();
            match user::Entity::find()
                .filter(user::Column::Id.is_in(user_ids))
                .all(&db)
                .await
            {
                Ok(users) => (StatusCode::OK, JsonResponse(users)),
                Err(err) => {
                    tracing::error!("Error fetching users: {:?}", err);
                    (StatusCode::INTERNAL_SERVER_ERROR, JsonResponse(Vec::new()))
                }
            }
        }
        Err(err) => {
            tracing::error!("Error fetching user accounts: {:?}", err);
            (StatusCode::INTERNAL_SERVER_ERROR, JsonResponse(Vec::new()))
        }
    }
}

pub async fn get_account_ledger(
    Extension(db): Extension<DatabaseConnection>,
    Path(account_id): Path<Uuid>,
) -> impl IntoResponse {
    tracing::info!("Fetching ledger for account: {}", account_id);

    match lead_charge::Entity::find()
        .filter(lead_charge::Column::AccountId.eq(account_id))
        .all(&db)
        .await
    {
        Ok(charges) => (StatusCode::OK, JsonResponse(charges)),
        Err(err) => {
            tracing::error!("Error fetching account ledger: {:?}", err);
            (StatusCode::INTERNAL_SERVER_ERROR, JsonResponse(Vec::<lead_charge::Model>::new()))
        }
    }
}

pub async fn remove_user_from_account(
    Extension(db): Extension<DatabaseConnection>,
    Extension(current_user): Extension<user::Model>,
    Path((account_id, user_id)): Path<(Uuid, Uuid)>,
) -> Result<impl IntoResponse, StatusCode> {
    tracing::info!("Removing user {} from account {}", user_id, account_id);

    // Fetch the account
    let account = account::Entity::find_by_id(account_id)
        .one(&db)
        .await
        .map_err(|err| {
            tracing::error!("Error fetching account: {:?}", err);
            StatusCode::INTERNAL_SERVER_ERROR
        })?
        .ok_or(StatusCode::NOT_FOUND)?;

    // Check if current user has permission to remove users from this account (e.g., is Owner)
    let current_user_account = user_account::Entity::find()
        .filter(user_account::Column::UserId.eq(current_user.id))
        .filter(user_account::Column::AccountId.eq(account.id))
        .one(&db)
        .await
        .map_err(|err| {
            tracing::error!("Error fetching user_account: {:?}", err);
            StatusCode::INTERNAL_SERVER_ERROR
        })?
        .ok_or(StatusCode::FORBIDDEN)?;

    if current_user_account.role != user_account::UserRole::Owner {
        tracing::warn!("User {} does not have permission to remove users from account {}", current_user.id, account_id);
        return Err(StatusCode::FORBIDDEN);
    }

    // Delete the user_account association
    let user_account_to_delete = user_account::Entity::find()
        .filter(user_account::Column::UserId.eq(user_id))
        .filter(user_account::Column::AccountId.eq(account.id))
        .one(&db)
        .await
        .map_err(|err| {
            tracing::error!("Error fetching user_account to delete: {:?}", err);
            StatusCode::INTERNAL_SERVER_ERROR
        })?
        .ok_or(StatusCode::NOT_FOUND)?;

    let user_account_active_model: user_account::ActiveModel = user_account_to_delete.into();
    user_account_active_model
        .delete(&db)
        .await
        .map_err(|err| {
            tracing::error!("Error removing user from account: {:?}", err);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    Ok(StatusCode::NO_CONTENT)
}

pub async fn update_user_role_in_account(
    Extension(db): Extension<DatabaseConnection>,
    Extension(current_user): Extension<user::Model>,
    Path((account_id, user_id)): Path<(Uuid, Uuid)>,
    Json(input): Json<UserAccountUpdate>,
) -> Result<impl IntoResponse, StatusCode> {
    tracing::info!("Updating role for user {} in account {}", user_id, account_id);

    // Fetch the account
    let account = account::Entity::find_by_id(account_id)
        .one(&db)
        .await
        .map_err(|err| {
            tracing::error!("Error fetching account: {:?}", err);
            StatusCode::INTERNAL_SERVER_ERROR
        })?
        .ok_or(StatusCode::NOT_FOUND)?;

    // Check if current user has permission to update roles in this account (e.g., is Owner)
    let current_user_account = user_account::Entity::find()
        .filter(user_account::Column::UserId.eq(current_user.id))
        .filter(user_account::Column::AccountId.eq(account.id))
        .one(&db)
        .await
        .map_err(|err| {
            tracing::error!("Error fetching user_account: {:?}", err);
            StatusCode::INTERNAL_SERVER_ERROR
        })?
        .ok_or(StatusCode::FORBIDDEN)?;

    if current_user_account.role != user_account::UserRole::Owner {
        tracing::warn!("User {} does not have permission to update roles in account {}", current_user.id, account_id);
        return Err(StatusCode::FORBIDDEN);
    }

    // Fetch the user_account association to update
    let user_account_to_update = user_account::Entity::find()
        .filter(user_account::Column::UserId.eq(user_id))
        .filter(user_account::Column::AccountId.eq(account.id))
        .one(&db)
        .await
        .map_err(|err| {
            tracing::error!("Error fetching user_account to update: {:?}", err);
            StatusCode::INTERNAL_SERVER_ERROR
        })?
        .ok_or(StatusCode::NOT_FOUND)?;

    let mut user_account_active_model: user_account::ActiveModel = user_account_to_update.into();

    // Update the role
    user_account_active_model.role = Set(input.role);
    user_account_active_model.updated_at = Set(Utc::now());

    let updated_user_account = user_account_active_model
        .update(&db)
        .await
        .map_err(|err| {
            tracing::error!("Error updating user role in account: {:?}", err);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    Ok(JsonResponse(updated_user_account))
}