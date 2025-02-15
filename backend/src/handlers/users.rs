use crate::entities::{
    ad_purchase::{self, Entity as AdPurchase},
    profile::{self, Entity as Profile},
    user::{self, Entity as User},
    user_account::{self, Entity as UserAccount},
    session::{self, Entity as Session},
    account::{self, Entity as Account},
    directory::Entity,
};
use axum::{
    body::Body, extract::{Extension, Json, Path, State}, headers::{HeaderMap, UserAgent}, http::{header::USER_AGENT, StatusCode}, response::IntoResponse, routing::{get, post}, Router
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use crate::auth::{hash_password, verify_password, generate_jwt};
use crate::models::session::{UserInfo, SessionResponse};
use crate::models::user::UserRegistration;
use crate::handlers::sessions::{refresh_token, validate_session, create_user_session};
use crate::handlers::profiles::get_profile_by_id;
use sea_orm::{DatabaseConnection, EntityTrait, Set, ColumnTrait, QueryFilter, ActiveModelTrait};
use uuid::Uuid;
use chrono::{Utc};
use crate::handlers::request_logs::log_request;

#[derive(Deserialize, Debug)]
pub struct LoginCredentials {
    email: String,
    password: String,
}

#[derive(Serialize)]
pub struct LoginResponse {
    pub token: String,
}
/*
pub fn auth_routes() -> Router<DatabaseConnection> {
    Router::new()
        .route("/login", post(login_user))
        .route("/register", post(register_user))
        .route("/validate-session", get(validate_session))
        .route("/logout", post(logout_user))
}*/

pub fn authenticated_routes(db_connection: DatabaseConnection) -> Router<DatabaseConnection> {
    Router::new()
        .route("/profile", get(get_user_profile))
        .route("/refresh-token", post(refresh_token))
        .route("/validate-session", get(validate_session))
        .with_state(db_connection)

}

pub async fn get_user_profile(
    State(db): State<DatabaseConnection>,
    Extension(current_user): Extension<user::Model>,
    Path(id): Path<Uuid>,
) -> Result<Json<profile::Model>, StatusCode> {
    get_profile_by_id(Extension(db), Extension(current_user), Path(id)).await
}

pub async fn register_user(
    State(db): State<DatabaseConnection>,
    Json(user_data): Json<UserRegistration>,
) -> Result<(StatusCode, Json<user::Model>), (StatusCode, String)> {
    tracing::info!("Received registration request for email: {}", user_data.email);

    let directory_id = user_data.directory_id;

    // Verify that the directory exists
    let directory = Entity::find_by_id(directory_id)
        .one(&db)
        .await
        .map_err(|err| {
            let error_msg = format!("Database error when checking directory: {:?}", err);
            tracing::error!("{}", error_msg);
            (StatusCode::INTERNAL_SERVER_ERROR, error_msg)
        })?
        .ok_or_else(|| {
            let error_msg = format!("Directory not found: {}", directory_id);
            tracing::error!("{}", error_msg);
            (StatusCode::NOT_FOUND, error_msg)
        })?;

    // Step 1: Check if a user already exists with the same email in the directory
    let existing_user = User::find()
        .filter(user::Column::Email.eq(&user_data.email))
        .one(&db)
        .await
        .map_err(|err| {
            let error_msg = format!("Database error when checking for existing user: {:?}", err);
            tracing::error!("{}", error_msg);
            (StatusCode::INTERNAL_SERVER_ERROR, error_msg)
        })?;

    if existing_user.is_some() {
        let error_msg = format!("User with email {} already exists in the system", user_data.email);
        tracing::warn!("{}", error_msg);
        return Err((StatusCode::CONFLICT, error_msg));
    }

    // Step 2: Hash password and create a new user
    let hashed_password = hash_password(&user_data.password)
        .map_err(|err| {
            let error_msg = format!("Error hashing password: {:?}", err);
            tracing::error!("{}", error_msg);
            (StatusCode::INTERNAL_SERVER_ERROR, error_msg)
        })?;

    // Clone username and email before moving them
    let username = user_data.username.clone();
    let email = user_data.email.clone();

    // Step 3: Create the user
    let new_user = user::ActiveModel {
        id: Set(Uuid::new_v4()),
        username: Set(username.clone()),
        first_name: Set(user_data.first_name),
        last_name: Set(user_data.last_name),
        phone: Set(user_data.phone),
        email: Set(email.clone()),
        password_hash: Set(hashed_password),
        is_admin: Set(false),
        is_active: Set(true),
        last_login: Set(Some(Utc::now())),
        created_at: Set(Utc::now()),
        updated_at: Set(Utc::now()),
    };

    let inserted_user = new_user.insert(&db).await.map_err(|err| {
        let error_msg = format!("Database error when inserting user: {:?}", err);
        tracing::error!("{}", error_msg);
        (StatusCode::INTERNAL_SERVER_ERROR, error_msg)
    })?;

    // Step 4: Find or create the Account for the directory
    let account = match account::Entity::find()
        .filter(account::Column::DirectoryId.eq(directory_id))
        .one(&db)
        .await
        .map_err(|err| {
            let error_msg = format!("Database error when finding account: {:?}", err);
            tracing::error!("{}", error_msg);
            (StatusCode::INTERNAL_SERVER_ERROR, error_msg)
        })? {
            Some(account) => account,
            None => {
                // Create a new Account if not found
                let new_account = account::ActiveModel {
                    id: Set(Uuid::new_v4()),
                    directory_id: Set(directory_id),
                    name: Set(username.clone()),
                    is_active: Set(true),
                    created_at: Set(Utc::now()),
                    updated_at: Set(Utc::now()),
                };
                new_account.insert(&db).await.map_err(|err| {
                    let error_msg = format!("Database error when creating account: {:?}", err);
                    tracing::error!("{}", error_msg);
                    (StatusCode::INTERNAL_SERVER_ERROR, error_msg)
                })?
            }
        };

    // Step 5: Find or create the Profile for the directory
    let profile = profile::Entity::find()
        .filter(profile::Column::DirectoryId.eq(directory_id))
        .one(&db)
        .await
        .map_err(|err| {
            let error_msg = format!("Database error when finding profile: {:?}", err);
            tracing::error!("{}", error_msg);
            (StatusCode::INTERNAL_SERVER_ERROR, error_msg)
        })?;

    let profile_id = if let Some(profile) = profile {
        profile.id
    } else {
        // Create a new Profile if not found
        let new_profile = profile::ActiveModel {
            id: Set(Uuid::new_v4()),
            account_id: Set(account.id),
            additional_info: Set(None),
            is_active: Set(true),
            directory_id: Set(directory_id),
            profile_type: Set(profile::ProfileType::Business), // Assuming it's a business profile
            display_name: Set(username),
            contact_info: Set(email),
            business_name: Set(None),
            business_address: Set(None),
            business_phone: Set(None),
            business_website: Set(None),
            created_at: Set(Utc::now()),
            updated_at: Set(Utc::now()),
        };

        let inserted_profile = new_profile.insert(&db).await.map_err(|err| {
            let error_msg = format!("Database error when inserting profile: {:?}", err);
            tracing::error!("{}", error_msg);
            (StatusCode::INTERNAL_SERVER_ERROR, error_msg)
        })?;

        inserted_profile.id
    };

    // Step 6: Create the UserAccount to link user and profile
    let new_user_account = user_account::ActiveModel {
        id: Set(Uuid::new_v4()),
        user_id: Set(inserted_user.id),
        account_id: Set(account.id),
        role: Set(user_account::UserRole::Owner),
        created_at: Set(Utc::now()),
        is_active: Set(true),
        updated_at: Set(Utc::now()),
    };

    new_user_account.insert(&db).await.map_err(|err| {
        let error_msg = format!("Database error when creating user account: {:?}", err);
        tracing::error!("{}", error_msg);
        (StatusCode::INTERNAL_SERVER_ERROR, error_msg)
    })?;

    Ok((StatusCode::CREATED, Json(inserted_user)))
}

pub async fn login_user(
    State(db): State<DatabaseConnection>,
    Json(credentials): Json<LoginCredentials>,
) -> Result<impl IntoResponse, StatusCode> {
    tracing::info!("Attempting to log in user: {}", credentials.email);
    println!("TEST LOG: from login_user and credentials: {:?}", credentials);
    let user = match User::find()
        .filter(user::Column::Email.eq(&credentials.email))
        .one(&db)
        .await
    {
        Ok(Some(user)) => {
            println!("TEST LOG: from login_user and user found: {:?}", user);
            user
        },
        Ok(None) => {
            println!("TEST LOG: from login_user and user not found");
            tracing::warn!("User not found for email: {}", credentials.email);
            return Err(StatusCode::UNAUTHORIZED);
        },
        Err(e) => {
            println!("TEST LOG: from login_user and error: {:?}", e);
            tracing::error!("Database error when finding user: {:?}", e);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    match verify_password(&credentials.password, &user.password_hash) {
        Ok(true) =>{
            tracing::info!("Password verified successfully");
            println!("TEST LOG: from login_user and password verified successfully");
        },
        Ok(false) => {
            println!("TEST LOG: from login_user and invalid password for user: {}", user.id);
            tracing::warn!("Invalid password for user: {}", user.id);
            return Err(StatusCode::UNAUTHORIZED);
        },
        Err(e) => {
            println!("TEST LOG: from login_user and error verifying password: {:?}", e);
            tracing::error!("Error verifying password: {:?}", e);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    }

    // Replace direct handler call with shared logic
    let session_response = create_user_session(&db, &credentials.email, &credentials.password).await?;

    println!("TEST LOG: from login_user and session created successfully for user: {}", user.id);
    tracing::info!("Session created from user handler successfully for user: {}", user.id);
    
    Ok(Json(session_response))
}


pub async fn logout_user(
    State(db): State<DatabaseConnection>,
    Extension(session): Extension<crate::entities::session::Model>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    println!("TEST LOG: from logout_user and session: {:?}", session);
    let mut active_session: session::ActiveModel = session.into();
    active_session.is_active = Set(false);
    active_session.last_modified_date = Set(Utc::now());

    active_session.update(&db).await.map_err(|e| {
        println!("TEST LOG: from logout_user and error updating session: {:?}", e);
        tracing::error!("Failed to update session: {:?}", e);
        (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error".to_string())
    })?;

    Ok((StatusCode::OK, Json(json!({"message": "Successfully logged out"}))))
}
