use axum::{
    extract::{State, Path, Extension, Json},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post, delete},
    Router,
};
use sea_orm::{DatabaseConnection, EntityTrait, QueryFilter, ColumnTrait, Set, ActiveModelTrait, ModelTrait, RelationTrait, JoinType};
use serde::{Deserialize, Serialize};
use serde_json::json;
use uuid::Uuid;
use chrono::Utc;

use crate::entities::{user, account, profile, user_account};

#[derive(Deserialize)]
pub struct CreateAccountPayload {
    pub name: String,
    pub directory_id: Uuid,
}

#[derive(Deserialize)]
pub struct CreateProfilePayload {
    pub directory_id: Uuid,
    pub display_name: String,
    pub contact_info: String,
    pub business_name: Option<String>,
    pub business_address: Option<String>,
    pub business_phone: Option<String>,
    pub business_website: Option<String>,
}

#[derive(Deserialize)]
pub struct InviteUserPayload {
    pub email: String,
    pub role: user_account::UserRole,
}

pub fn authenticated_routes() -> Router<DatabaseConnection> {
    Router::new()
        .route("/api/me/accounts", get(list_accounts).post(create_account))
        .route("/api/me/accounts/{id}/users", get(list_account_users))
        .route("/api/me/accounts/{id}/profiles", post(create_profile))
        .route("/api/me/accounts/{id}/invitations", post(invite_user))
        .route("/api/me/accounts/{id}/users/{user_id}", delete(remove_user))
}

pub async fn list_accounts(
    State(db): State<DatabaseConnection>,
    Extension(current_user): Extension<user::Model>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let user_accounts = user_account::Entity::find()
        .filter(user_account::Column::UserId.eq(current_user.id))
        .find_also_related(account::Entity)
        .all(&db)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let accounts_data: Vec<_> = user_accounts.into_iter()
        .filter_map(|(ua, acc_opt)| {
            acc_opt.map(|acc| {
                json!({
                    "account": acc,
                    "role": ua.role,
                })
            })
        })
        .collect();

    Ok(Json(accounts_data))
}

pub async fn create_account(
    State(db): State<DatabaseConnection>,
    Extension(current_user): Extension<user::Model>,
    Json(payload): Json<CreateAccountPayload>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let new_account = account::ActiveModel {
        id: Set(Uuid::new_v4()),
        directory_id: Set(payload.directory_id),
        name: Set(payload.name),
        is_active: Set(true),
        created_at: Set(Utc::now()),
        updated_at: Set(Utc::now()),
    };

    let inserted_account = new_account.insert(&db).await.map_err(|e| {
        (StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
    })?;

    let new_ua = user_account::ActiveModel {
        id: Set(Uuid::new_v4()),
        user_id: Set(current_user.id),
        account_id: Set(inserted_account.id),
        role: Set(user_account::UserRole::Owner),
        is_active: Set(true),
        created_at: Set(Utc::now()),
        updated_at: Set(Utc::now()),
    };

    new_ua.insert(&db).await.map_err(|e| {
        (StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
    })?;

    Ok((StatusCode::CREATED, Json(inserted_account)))
}

pub async fn list_account_users(
    State(db): State<DatabaseConnection>,
    Extension(current_user): Extension<user::Model>,
    Path(account_id): Path<Uuid>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    // Verify user belongs to account
    check_user_account_access(&db, current_user.id, account_id).await?;

    let users = user_account::Entity::find()
        .filter(user_account::Column::AccountId.eq(account_id))
        .find_also_related(user::Entity)
        .all(&db)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let users_data: Vec<_> = users.into_iter()
        .filter_map(|(ua, u_opt)| {
            u_opt.map(|u| {
                json!({
                    "user_id": u.id,
                    "email": u.email,
                    "name": format!("{} {}", u.first_name, u.last_name),
                    "role": ua.role,
                    "user_account_id": ua.id,
                })
            })
        })
        .collect();

    Ok(Json(users_data))
}

pub async fn create_profile(
    State(db): State<DatabaseConnection>,
    Extension(current_user): Extension<user::Model>,
    Path(account_id): Path<Uuid>,
    Json(payload): Json<CreateProfilePayload>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    check_user_account_access(&db, current_user.id, account_id).await?;

    let new_profile = profile::ActiveModel {
        id: Set(Uuid::new_v4()),
        account_id: Set(account_id),
        directory_id: Set(payload.directory_id),
        profile_type: Set(profile::ProfileType::Business),
        display_name: Set(payload.display_name),
        contact_info: Set(payload.contact_info),
        business_name: Set(payload.business_name),
        business_address: Set(payload.business_address),
        business_phone: Set(payload.business_phone),
        business_website: Set(payload.business_website),
        additional_info: Set(None),
        properties: Set(None),
        is_active: Set(true),
        created_at: Set(Utc::now()),
        updated_at: Set(Utc::now()),
    };

    let inserted = new_profile.insert(&db).await.map_err(|e| {
        (StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
    })?;

    Ok((StatusCode::CREATED, Json(inserted)))
}

pub async fn invite_user(
    State(db): State<DatabaseConnection>,
    Extension(current_user): Extension<user::Model>,
    Path(account_id): Path<Uuid>,
    Json(payload): Json<InviteUserPayload>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    check_user_account_access(&db, current_user.id, account_id).await?;

    // Find user by email
    let target_user = user::Entity::find()
        .filter(user::Column::Email.eq(&payload.email))
        .one(&db)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    if let Some(tu) = target_user {
        // Check if already in account
        let existing = user_account::Entity::find()
            .filter(user_account::Column::UserId.eq(tu.id))
            .filter(user_account::Column::AccountId.eq(account_id))
            .one(&db)
            .await
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

        if existing.is_some() {
            return Err((StatusCode::CONFLICT, "User is already in this account".into()));
        }

        let new_ua = user_account::ActiveModel {
            id: Set(Uuid::new_v4()),
            user_id: Set(tu.id),
            account_id: Set(account_id),
            role: Set(payload.role),
            is_active: Set(true),
            created_at: Set(Utc::now()),
            updated_at: Set(Utc::now()),
        };

        let inserted = new_ua.insert(&db).await.map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
        Ok((StatusCode::CREATED, Json(inserted)))
    } else {
        Err((StatusCode::NOT_FOUND, "User not found. They must register first.".into()))
    }
}

pub async fn remove_user(
    State(db): State<DatabaseConnection>,
    Extension(current_user): Extension<user::Model>,
    Path((account_id, target_user_id)): Path<(Uuid, Uuid)>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    check_user_account_access(&db, current_user.id, account_id).await?;

    let ua = user_account::Entity::find()
        .filter(user_account::Column::AccountId.eq(account_id))
        .filter(user_account::Column::UserId.eq(target_user_id))
        .one(&db)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    if let Some(ua_model) = ua {
        ua_model.delete(&db).await.map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
        Ok((StatusCode::OK, Json(json!({"message": "User removed from account"}))))
    } else {
        Err((StatusCode::NOT_FOUND, "User is not part of this account".into()))
    }
}

async fn check_user_account_access(db: &DatabaseConnection, user_id: Uuid, account_id: Uuid) -> Result<(), (StatusCode, String)> {
    let ua = user_account::Entity::find()
        .filter(user_account::Column::UserId.eq(user_id))
        .filter(user_account::Column::AccountId.eq(account_id))
        .one(db)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    if ua.is_none() {
        return Err((StatusCode::FORBIDDEN, "You do not have access to this account".into()));
    }
    
    Ok(())
}
