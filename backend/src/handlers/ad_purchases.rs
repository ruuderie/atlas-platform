// src/handlers/ad_purchases.rs

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
    ad_purchase, profile, user_account, user,
};
use crate::models::ad_purchase::*;
use uuid::Uuid;
use chrono::Utc;

pub fn routes() -> Router<DatabaseConnection> {
    Router::new()
        .route("/api/ad-purchases", post(create_ad_purchase))
        .route("/api/ad-purchases", get(get_ad_purchases))
        .route("/api/ad-purchases/{id}", get(get_ad_purchase_by_id))
        .route("/api/ad-purchases/{id}", put(update_ad_purchase))
        .route("/api/ad-purchases/{id}", delete(delete_ad_purchase))
}

pub async fn create_ad_purchase(
    Extension(db): Extension<DatabaseConnection>,
    Extension(current_user): Extension<user::Model>,
    Extension(directory_ids): Extension<Vec<Uuid>>,
    Json(input): Json<AdPurchaseCreate>,
) -> Result<impl IntoResponse, StatusCode> {
    tracing::info!("Creating new ad purchase for profile: {}", input.profile_id);

    // Fetch the profile
    let profile = profile::Entity::find()
        .filter(profile::Column::Id.eq(input.profile_id))
        .filter(profile::Column::DirectoryId.is_in(directory_ids.clone()))
        .one(&db)
        .await
        .map_err(|err| {
            tracing::error!("Error fetching profile: {:?}", err);
            StatusCode::INTERNAL_SERVER_ERROR
        })?
        .ok_or_else(|| {
            tracing::warn!("Profile not found: {}", input.profile_id);
            StatusCode::NOT_FOUND
        })?;

    // Check if the user is associated with the profile
    let user_account_exists = user_account::Entity::find()
        .filter(user_account::Column::UserId.eq(current_user.id))
        .filter(user_account::Column::AccountId.eq(profile.account_id))
        .one(&db)
        .await
        .map_err(|err| {
            tracing::error!("Error checking user_account association: {:?}", err);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    if user_account_exists.is_none() {
        tracing::warn!("User {} not associated with profile {}", current_user.id, input.profile_id);
        return Err(StatusCode::FORBIDDEN);
    }
    
    // Create the ad purchase
    let new_ad_purchase = ad_purchase::ActiveModel {
        id: Set(Uuid::new_v4()),
        profile_id: Set(profile.id),
        listing_id: Set(input.listing_id),
        start_date: Set(input.start_date),
        end_date: Set(input.end_date),
        status: Set(AdStatus::Pending.to_string()),
        created_at: Set(Utc::now()),
        updated_at: Set(Utc::now()),
        price: Set(input.price),
    };

    let inserted_ad_purchase = new_ad_purchase.insert(&db).await.map_err(|err| {
        tracing::error!("Error creating ad purchase: {:?}", err);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    Ok((StatusCode::CREATED, JsonResponse(inserted_ad_purchase)))
}

pub async fn get_ad_purchases(
    Extension(db): Extension<DatabaseConnection>,
    Extension(current_user): Extension<user::Model>,
    Extension(directory_ids): Extension<Vec<Uuid>>,
) -> Result<impl IntoResponse, StatusCode> {
    tracing::info!("Fetching ad purchases for user: {}", current_user.id);

    // Fetch profiles associated with the user's directories
    let profiles = profile::Entity::find()
        .filter(profile::Column::DirectoryId.is_in(directory_ids))
        .all(&db)
        .await
        .map_err(|err| {
            tracing::error!("Error fetching profiles: {:?}", err);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    let profile_ids: Vec<Uuid> = profiles.into_iter().map(|p| p.id).collect();

    // Fetch ad purchases associated with these profiles
    let ad_purchases = ad_purchase::Entity::find()
        .filter(ad_purchase::Column::ProfileId.is_in(profile_ids))
        .all(&db)
        .await
        .map_err(|err| {
            tracing::error!("Error fetching ad purchases: {:?}", err);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    Ok((StatusCode::OK, JsonResponse(ad_purchases)))
}

pub async fn update_ad_purchase(
    Extension(db): Extension<DatabaseConnection>,
    Extension(current_user): Extension<user::Model>,
    Extension(directory_ids): Extension<Vec<Uuid>>,
    Path(id): Path<Uuid>,
    Json(input): Json<AdPurchaseUpdate>,
) -> Result<impl IntoResponse, StatusCode> {
    tracing::info!("Updating ad purchase: {}", id);

    // Fetch the ad purchase
    let ad_purchase = ad_purchase::Entity::find()
        .filter(ad_purchase::Column::Id.eq(id))
        .one(&db)
        .await
        .map_err(|err| {
            tracing::error!("Error fetching ad purchase: {:?}", err);
            StatusCode::INTERNAL_SERVER_ERROR
        })?
        .ok_or_else(|| {
            tracing::warn!("Ad purchase not found: {}", id);
            StatusCode::NOT_FOUND
        })?;

    // Fetch the profile associated with the ad purchase
    let profile = profile::Entity::find_by_id(ad_purchase.profile_id)
        .one(&db)
        .await
        .map_err(|err| {
            tracing::error!("Error fetching profile: {:?}", err);
            StatusCode::INTERNAL_SERVER_ERROR
        })?
        .ok_or_else(|| {
            tracing::warn!("Profile not found for ad purchase: {}", id);
            StatusCode::NOT_FOUND
        })?;

    // Check directory isolation
    if !directory_ids.contains(&profile.directory_id) {
        tracing::warn!("User {} not authorized to update ad purchase {}", current_user.id, id);
        return Err(StatusCode::FORBIDDEN);
    }

    // Update the ad purchase   
    let mut updated_ad_purchase: ad_purchase::ActiveModel = ad_purchase.into();
    updated_ad_purchase.listing_id = Set(input.listing_id);
    updated_ad_purchase.start_date = Set(input.start_date);
    updated_ad_purchase.end_date = Set(input.end_date);
    updated_ad_purchase.status = Set(AdStatus::Pending.to_string()); 
    updated_ad_purchase.price = Set(input.price);
    updated_ad_purchase.updated_at = Set(Utc::now());

    let updated_model = updated_ad_purchase.update(&db).await.map_err(|err| {
        tracing::error!("Error updating ad purchase: {:?}", err);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    Ok((StatusCode::OK, JsonResponse(updated_model)))
}

pub async fn delete_ad_purchase(
    Extension(db): Extension<DatabaseConnection>,
    Extension(current_user): Extension<user::Model>,
    Extension(directory_ids): Extension<Vec<Uuid>>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, StatusCode> {
    tracing::info!("Deleting ad purchase: {}", id);

    // Fetch the ad purchase
    let ad_purchase = ad_purchase::Entity::find()
        .filter(ad_purchase::Column::Id.eq(id))
        .one(&db)
        .await
        .map_err(|err| {
            tracing::error!("Error fetching ad purchase: {:?}", err);
            StatusCode::INTERNAL_SERVER_ERROR
        })?
        .ok_or_else(|| {
            tracing::warn!("Ad purchase not found: {}", id);
            StatusCode::NOT_FOUND
        })?;

    // Fetch the profile associated with the ad purchase
    let profile = profile::Entity::find_by_id(ad_purchase.profile_id)
        .one(&db)
        .await
        .map_err(|err| {
            tracing::error!("Error fetching profile: {:?}", err);
            StatusCode::INTERNAL_SERVER_ERROR
        })?
        .ok_or_else(|| {
            tracing::warn!("Profile not found for ad purchase: {}", id);
            StatusCode::NOT_FOUND
        })?;

    // Check directory isolation
    if !directory_ids.contains(&profile.directory_id) {
        tracing::warn!("User {} not authorized to delete ad purchase {}", current_user.id, id);
        return Err(StatusCode::FORBIDDEN);
    }

    // Delete the ad purchase
    ad_purchase::Entity::delete_by_id(ad_purchase.id)
        .exec(&db)
        .await
        .map_err(|err| {
            tracing::error!("Error deleting ad purchase: {:?}", err);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    Ok(StatusCode::NO_CONTENT)
}

pub async fn get_ad_purchase_by_id(
    Extension(db): Extension<DatabaseConnection>,
    Extension(current_user): Extension<user::Model>,
    Extension(directory_ids): Extension<Vec<Uuid>>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, StatusCode> {
    tracing::info!("Fetching ad purchase: {}", id);

    // Fetch the ad purchase
    let ad_purchase = ad_purchase::Entity::find()
        .filter(ad_purchase::Column::Id.eq(id))
        .one(&db)
        .await
        .map_err(|err| {
            tracing::error!("Error fetching ad purchase: {:?}", err);
            StatusCode::INTERNAL_SERVER_ERROR
        })?
        .ok_or_else(|| {
            tracing::warn!("Ad purchase not found: {}", id);
            StatusCode::NOT_FOUND
        })?;

    // Fetch the profile associated with the ad purchase
    let profile = profile::Entity::find_by_id(ad_purchase.profile_id)
        .one(&db)
        .await
        .map_err(|err| {
            tracing::error!("Error fetching profile: {:?}", err);
            StatusCode::INTERNAL_SERVER_ERROR
        })?
        .ok_or_else(|| {
            tracing::warn!("Profile not found for ad purchase: {}", id);
            StatusCode::NOT_FOUND
        })?;

    // Check directory isolation
    if !directory_ids.contains(&profile.directory_id) {
        tracing::warn!("User {} not authorized to view ad purchase {}", current_user.id, id);
        return Err(StatusCode::FORBIDDEN);
    }

    Ok((StatusCode::OK, JsonResponse(ad_purchase)))
}