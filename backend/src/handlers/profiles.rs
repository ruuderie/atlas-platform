use crate::entities::{
    account::{self, Entity as Account},
    ad_purchase::{self, Entity as AdPurchase},
    profile::{self, Entity as Profile},
    user::{self, Entity as User},
    user_account::{self, Entity as UserAccount}
};
use crate::models::profile::{ProfileSearch, CreateProfileInput, UpdateProfileInput};
use crate::models::account::{AccountModel, CreateAccountInput, UpdateAccountInput};
use axum::{
    extract::{Extension, Json, Path, Query,State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post, put, delete},
    Router,
};
use sea_orm::Database;
use sea_orm::{DatabaseConnection, EntityTrait, Set, Condition, ColumnTrait, QueryFilter, ActiveModelTrait, IntoActiveModel};
use uuid::Uuid;
use chrono::Utc;

pub fn routes(db_connection: DatabaseConnection) -> Router<DatabaseConnection> {
    
    Router::new()
        .route("/profiles", post(create_profile))
        .route("/profiles", get(get_profiles))
        .route("/profiles/:id", get(get_profile_by_id))
        .route("/profiles/:id", put(update_profile))
        .route("/profiles/:id", delete(delete_profile))
        .route("/profiles/search", get(search_profiles))
        .with_state(db_connection)
}

pub async fn create_profile(
    State(db): State<DatabaseConnection>,
    Extension(current_user): Extension<user::Model>,
    Json(input): Json<CreateProfileInput>,
) -> Result<impl IntoResponse, StatusCode> {
    tracing::info!("Creating new profile for user: {}", current_user.id);

    // create or find the account
    let account = match Account::find()
        .filter(account::Column::DirectoryId.eq(input.directory_id))
        .one(&db)
        .await
        .map_err(|err| {
            tracing::error!("Error fetching account: {:?}", err);
            StatusCode::INTERNAL_SERVER_ERROR
        })? {
            Some(account) => account,
            None => {
                let new_account = account::ActiveModel {
                    id: Set(Uuid::new_v4()),
                    directory_id: Set(input.directory_id),
                    name: Set(input.display_name.clone()),
                    is_active: Set(true),
                    created_at: Set(Utc::now()),
                    updated_at: Set(Utc::now()),
                };
                new_account.insert(&db).await.map_err(|err| {
                    tracing::error!("Error creating account: {:?}", err);
                    StatusCode::INTERNAL_SERVER_ERROR
                })?
            }
        };

    // Create the profile
    let mut new_profile = profile::ActiveModel {
        id: Set(Uuid::new_v4()),
        directory_id: Set(input.directory_id),
        account_id: Set(account.id),
        profile_type: Set(input.profile_type),
        display_name: Set(input.display_name),
        contact_info: Set(input.contact_info),
        business_name: Set(None),
        business_address: Set(None),
        business_phone: Set(None),
        business_website: Set(None),
        additional_info: Set(None),
        is_active: Set(true),
        created_at: Set(Utc::now()),
        updated_at: Set(Utc::now()),
    };

    if let Some(business_details) = input.business_details {
        new_profile.business_name = Set(Some(business_details.business_name));
        new_profile.business_address = Set(Some(business_details.business_address));
        new_profile.business_phone = Set(Some(business_details.business_phone));
        new_profile.business_website = Set(business_details.website);
    }

    let inserted_profile = new_profile.insert(&db).await.map_err(|err| {
        tracing::error!("Error inserting profile: {:?}", err);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    // Create the UserAccount association
    let new_user_account = user_account::ActiveModel {
        id: Set(Uuid::new_v4()),
        user_id: Set(current_user.id),
        account_id: Set(account.id),
        role: Set(user_account::UserRole::Owner),
        is_active: Set(true),
        created_at: Set(Utc::now()),
        updated_at: Set(Utc::now()),
    };

    new_user_account.insert(&db).await.map_err(|err| {
        tracing::error!("Error creating user account: {:?}", err);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    Ok((StatusCode::CREATED, Json(inserted_profile)))
}

pub async fn update_profile(
    Extension(db): Extension<DatabaseConnection>,
    Extension(current_user): Extension<user::Model>,
    Path(id): Path<Uuid>,
    Json(input): Json<UpdateProfileInput>,
) -> Result<Json<profile::Model>, StatusCode> {
    tracing::info!("Updating profile: {} for user: {}", id, current_user.id);

    // Fetch the profile
    let profile_to_update = Profile::find_by_id(id)
        .one(&db)
        .await
        .map_err(|err| {
            tracing::error!("Error fetching profile: {:?}", err);
            StatusCode::INTERNAL_SERVER_ERROR
        })?
        .ok_or(StatusCode::NOT_FOUND)?;

    // Check if the user has access to this profile via the account
    let user_account = UserAccount::find()
        .filter(user_account::Column::UserId.eq(current_user.id))
        .filter(user_account::Column::AccountId.eq(profile_to_update.account_id))
        .one(&db)
        .await
        .map_err(|err| {
            tracing::error!("Error fetching user account: {:?}", err);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    if user_account.is_none() {
        return Err(StatusCode::FORBIDDEN);
    }

    // Update the profile
    let mut active_model = profile_to_update.into_active_model();

    if let Some(display_name) = input.display_name {
        active_model.display_name = Set(display_name);
    }
    if let Some(contact_info) = input.contact_info {
        active_model.contact_info = Set(contact_info);
    }
    if let Some(business_details) = input.business_details {
        active_model.business_name = Set(Some(business_details.business_name));
        active_model.business_address = Set(Some(business_details.business_address));
        active_model.business_phone = Set(Some(business_details.business_phone));
        active_model.business_website = Set(business_details.website);
    }

    let updated_profile = active_model.update(&db).await.map_err(|err| {
        tracing::error!("Error updating profile: {:?}", err);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    Ok(Json(updated_profile))
}

pub async fn get_profiles(
    Extension(db): Extension<DatabaseConnection>,
    Extension(current_user): Extension<user::Model>,
) -> Result<Json<Vec<profile::Model>>, StatusCode> {
    tracing::info!("Fetching profiles for user: {}", current_user.id);

    // Fetch profiles associated with the user
    let user_accounts = UserAccount::find()
        .filter(user_account::Column::UserId.eq(current_user.id))
        .all(&db)
        .await
        .map_err(|err| {
            tracing::error!("Error fetching user profiles: {:?}", err);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    let profile_ids: Vec<Uuid> = user_accounts.into_iter().map(|up| up.account_id).collect();

    let profiles = Profile::find()
        .filter(profile::Column::AccountId.is_in(profile_ids))
        .all(&db)
        .await
        .map_err(|err| {
            tracing::error!("Error fetching profiles: {:?}", err);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    Ok(Json(profiles))
}

pub async fn search_profiles(
    Extension(db): Extension<DatabaseConnection>,
    Extension(current_user): Extension<user::Model>,
    Query(params): Query<ProfileSearch>,
) -> Result<Json<Vec<profile::Model>>, StatusCode> {
    tracing::info!("Searching profiles for user: {}", current_user.id);

    // Fetch user profiles
    let user_accounts = UserAccount::find()
        .filter(user_account::Column::UserId.eq(current_user.id))
        .all(&db)
        .await
        .map_err(|err| {
            tracing::error!("Error fetching user profiles: {:?}", err);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    let profile_ids: Vec<Uuid> = user_accounts.into_iter().map(|up| up.account_id).collect();

    // Search profiles associated with the user
    let profiles = Profile::find()
        .filter(profile::Column::AccountId.is_in(profile_ids))
        .filter(
            Condition::any()
                .add(profile::Column::DisplayName.contains(&params.q))
                .add(profile::Column::ContactInfo.contains(&params.q)),
        )
        .all(&db)
        .await
        .map_err(|err| {
            tracing::error!("Error searching profiles: {:?}", err);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    Ok(Json(profiles))
}

pub async fn delete_profile(
    Extension(db): Extension<DatabaseConnection>,
    Extension(current_user): Extension<user::Model>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, StatusCode> {
    tracing::info!("Deleting profile: {} for user: {}", id, current_user.id);

    // Check if the user has access to this profile
    let user_account = UserAccount::find()
        .filter(user_account::Column::UserId.eq(current_user.id))
        .filter(user_account::Column::AccountId.eq(id))
        .one(&db)
        .await
        .map_err(|err| {
            tracing::error!("Error fetching user profile: {:?}", err);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    if user_account.is_none() {
        return Err(StatusCode::FORBIDDEN);
    }

    // Delete the profile
    Profile::delete_by_id(id)
        .exec(&db)
        .await
        .map_err(|err| {
            tracing::error!("Error deleting profile: {:?}", err);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    Ok(StatusCode::NO_CONTENT)
}

pub async fn get_profile_by_id(
    Extension(db): Extension<DatabaseConnection>,
    Extension(current_user): Extension<user::Model>,
    Path(id): Path<Uuid>,
) -> Result<Json<profile::Model>, StatusCode> {
    tracing::info!("Fetching profile: {} for user: {}", id, current_user.id);

    // Check if the user has access to this profile
    let user_account = UserAccount::find()
        .filter(user_account::Column::UserId.eq(current_user.id))
        .filter(user_account::Column::AccountId.eq(id))
        .one(&db)
        .await
        .map_err(|err| {
            tracing::error!("Error fetching user profile: {:?}", err);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    if user_account.is_none() {
        return Err(StatusCode::FORBIDDEN);
    }

    // Fetch the profile by ID
    let profile = Profile::find_by_id(id)
        .one(&db)
        .await
        .map_err(|err| {
            tracing::error!("Error fetching profile: {:?}", err);
            StatusCode::INTERNAL_SERVER_ERROR
        })?
        .ok_or(StatusCode::NOT_FOUND)?;

    Ok(Json(profile))
}