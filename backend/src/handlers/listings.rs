use crate::entities::{
    ad_purchase::{self, Entity as AdPurchase},
    listing::{self, Entity as Listing},
    profile::{self, Entity as Profile},
    user::{self, Entity as User},
    user_account::{self, Entity as UserAccount},
    template::{self, Entity as Template},
    listing_attribute::{self, Entity as ListingAttribute},
};
use crate::models::listing::{ListingCreate, ListingUpdate, ListingStatus};
use sea_orm::{
    DatabaseConnection, EntityTrait, Set, QueryFilter, ColumnTrait, ActiveModelTrait, TransactionTrait,DatabaseTransaction, IntoActiveModel
};
use axum::{
    extract::{Path, Json, Extension, Query},
    response::IntoResponse,
    http::StatusCode,
    routing::{get, post, put, delete},
    Router,
};
use chrono::Utc;
use uuid::Uuid;
use serde::Deserialize;
use std::collections::HashMap;

#[derive(Deserialize, Debug)]
pub struct ListingSearch {
    q: String,
}

pub fn public_routes(db: DatabaseConnection) -> Router<DatabaseConnection> {
    Router::new()
        .route("/listings", get(get_listings))
        .route("/listings/{id}", get(get_listing_by_id))
        .route("/listings/search", get(search_listings))
        .with_state(db)
}

pub fn authenticated_routes() -> Router<DatabaseConnection> {
    Router::new()
        .route("/listings", post(create_listing))
        .route("/listings/{id}", get(get_listing_by_id))
        .route("/listings/{id}/with-attributes", get(get_listing_with_attributes))
        .route("/listings/{id}", put(update_listing))
        .route("/listings/{id}", delete(delete_listing))
        // Add other authenticated listing routes here
}

pub async fn get_listings(
    Extension(db): Extension<DatabaseConnection>,
    Query(params): Query<HashMap<String, String>>,
) -> Result<impl IntoResponse, StatusCode> {
    let directory_id = params.get("directory_id")
        .and_then(|id| Uuid::parse_str(id).ok())
        .ok_or(StatusCode::BAD_REQUEST)?;
    println!("TEST LOG: from get_listings and directory_id: {:?}", directory_id);
    tracing::info!("Fetching listings for Directory ID: {}", directory_id);

    let listings = Listing::find()
        .filter(listing::Column::DirectoryId.eq(directory_id))
        .all(&db)
        .await
        .map_err(|err| {
            println!("TEST LOG: from get_listings and err: {:?}", err);
            tracing::error!("Database error: {:?}", err);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    Ok(Json(listings))
}

pub async fn get_listing_by_id(
    Extension(db): Extension<DatabaseConnection>,
    Path(id): Path<Uuid>,
) -> Result<Json<listing::Model>, StatusCode> {
    tracing::info!("Fetching listing with ID: {}", id);
    let listing = Listing::find_by_id(id)
        .one(&db)
        .await
        .map_err(|err| {
            tracing::error!("Error fetching listing: {:?}", err);
            StatusCode::INTERNAL_SERVER_ERROR
        })?
        .ok_or(StatusCode::NOT_FOUND)?;
    tracing::info!("Listing found: {:?}", listing);
    Ok(Json(listing))
}
pub async fn create_listing(
    Extension(db): Extension<DatabaseConnection>,
    Extension(current_user): Extension<user::Model>,
    Json(input): Json<ListingCreate>,
    //tuple of (status, listing)
) -> Result<(StatusCode, Json<listing::Model>), StatusCode> {
    println!("TEST LOG: from create_listing and input: {:?}", input);
    // Start transaction
    let txn = db.begin().await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    println!("TEST LOG: from create_listing and txn: {:?}", txn);
    println!("TEST LOG: from create_listing and current_user: {:?}", current_user);
    println!("TEST LOG: from create_listing and input.profile_id: {:?}", input.profile_id);
    // Verify user has permission to create listing under this profile
    let profile = Profile::find_by_id(input.profile_id)
        .one(&txn)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;
    println!("TEST LOG: from create_listing and profile: {:?}", profile);
    let user_account_exists = UserAccount::find()
        .filter(user_account::Column::UserId.eq(current_user.id))
        .filter(user_account::Column::AccountId.eq(profile.account_id))
        .one(&txn)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .is_some();
    println!("TEST LOG: from create_listing and user_account_exists: {:?}", user_account_exists);
    if !user_account_exists {
        return Err(StatusCode::FORBIDDEN);
    }

    // Create the listing
    let new_listing = input.into_active_model();
    println!("TEST LOG: from create_listing and new_listing: {:?}", new_listing);
    let inserted_listing = new_listing.insert(&txn).await.map_err(|err| {
        println!("TEST LOG: from create_listing and err: {:?}", err);
        tracing::error!("Failed to insert listing: {:?}", err);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;
    println!("TEST LOG: from create_listing and inserted_listing: {:?}", inserted_listing);
    // Handle template attributes if applicable
    if let Some(template_id) = inserted_listing.based_on_template_id {
        copy_template_attributes(&txn, inserted_listing.id, template_id).await?;
    }
    
    txn.commit().await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    // send 201 created status code
    Ok((StatusCode::CREATED, Json(inserted_listing)))
}

// Separate function for template attribute copying
async fn copy_template_attributes(
    txn: &DatabaseTransaction,
    listing_id: Uuid,
    template_id: Uuid,
) -> Result<(), StatusCode> {
    let template_attributes = ListingAttribute::find()
        .filter(listing_attribute::Column::TemplateId.eq(template_id))
        .all(txn)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    for attr in template_attributes {
        listing_attribute::ActiveModel {
            id: Set(Uuid::new_v4()),
            listing_id: Set(Some(listing_id)),
            template_id: Set(None),
            attribute_type: Set(attr.attribute_type),
            attribute_key: Set(attr.attribute_key),
            value: Set(attr.value),
            created_at: Set(Utc::now()),
            updated_at: Set(Utc::now()),
        }
        .insert(txn)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    }
    Ok(())
}

pub async fn update_listing(
    Extension(db): Extension<DatabaseConnection>,
    Extension(current_user): Extension<user::Model>,
    Path(id): Path<Uuid>,
    Json(input): Json<ListingUpdate>,
) -> Result<Json<listing::Model>, StatusCode> {
    println!("TEST LOG: from update_listing and input: {:?}", input);
    let existing_listing = Listing::find_by_id(id)
        .one(&db)
        .await
        .map_err(|err| {
            println!("TEST LOG: from update_listing and err: {:?}", err);
            tracing::error!("Error fetching listing: {:?}", err);
            StatusCode::INTERNAL_SERVER_ERROR
            
        })?
        .ok_or(StatusCode::NOT_FOUND)?;

    let profile = Profile::find_by_id(existing_listing.profile_id)
        .one(&db)
        .await
        .map_err(|err| {
            println!("TEST LOG: from update_listing and err: {:?}", err);
            tracing::error!("Error fetching profile: {:?}", err);
            StatusCode::INTERNAL_SERVER_ERROR
        })?
        .ok_or(StatusCode::NOT_FOUND)?;

    let user_account_exists = UserAccount::find()
        .filter(user_account::Column::UserId.eq(current_user.id))
        .filter(user_account::Column::AccountId.eq(profile.account_id))
        .one(&db)
        .await
        .map_err(|err| {
            println!("TEST LOG: from update_listing and err: {:?}", err);
            tracing::error!("Error checking user_account association: {:?}", err);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    if user_account_exists.is_none() {
        println!("TEST LOG: from update_listing and user_account_exists: {:?}", user_account_exists);
        return Err(StatusCode::FORBIDDEN);
    }

    let mut listing_active_model: listing::ActiveModel = existing_listing.into();

    // Update fields if provided in input
    if let Some(title) = input.title.as_ref() { listing_active_model.title = Set(title.clone()); }
    if let Some(description) = input.description.as_ref() { listing_active_model.description = Set(description.clone()); }
    if let Some(category_id) = input.category_id { listing_active_model.category_id = Set(Some(category_id)); }
    if let Some(listing_type) = input.listing_type.as_ref() { listing_active_model.listing_type = Set(listing_type.clone()); }
    if let Some(price) = input.price { listing_active_model.price = Set(Some(price)); }
    if let Some(price_type) = input.price_type.as_ref() { listing_active_model.price_type = Set(Some(price_type.clone())); }
    if let Some(country) = input.country.as_ref() { listing_active_model.country = Set(Some(country.clone())); }
    if let Some(state) = input.state.as_ref() { listing_active_model.state = Set(Some(state.clone())); }
    if let Some(city) = input.city.as_ref() { listing_active_model.city = Set(Some(city.clone())); }
    if let Some(neighborhood) = input.neighborhood.as_ref() { listing_active_model.neighborhood = Set(Some(neighborhood.clone())); }
    if let Some(latitude) = input.latitude { listing_active_model.latitude = Set(Some(latitude)); }
    if let Some(longitude) = input.longitude { listing_active_model.longitude = Set(Some(longitude)); }
    if let Some(additional_info) = input.additional_info { listing_active_model.additional_info = Set(Some(additional_info)); }
    if let Some(is_featured) = input.is_featured { listing_active_model.is_featured = Set(is_featured); }
    if let Some(is_active) = input.is_active { listing_active_model.is_active = Set(is_active); }
    if let Some(is_ad_placement) = input.is_ad_placement { listing_active_model.is_ad_placement = Set(is_ad_placement); }
    if let Some(is_based_on_template) = input.is_based_on_template { listing_active_model.is_based_on_template = Set(is_based_on_template); }
    if let Some(based_on_template_id) = input.based_on_template_id { listing_active_model.based_on_template_id = Set(Some(based_on_template_id)); }
    if let Some(status) = input.status { listing_active_model.status = Set(status); }

    listing_active_model.updated_at = Set(Utc::now());

    let updated_listing = listing_active_model.update(&db).await.map_err(|err| {
        println!("TEST LOG: from update_listing and err: {:?}", err);
        tracing::error!("Error updating listing: {:?}", err);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;
    println!("TEST LOG: from update_listing and updated_listing: {:?}", updated_listing);
    Ok(Json(updated_listing))
}

pub async fn delete_listing(
    Extension(db): Extension<DatabaseConnection>,
    Extension(current_user): Extension<user::Model>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, StatusCode> {
    let listing = Listing::find_by_id(id)
        .one(&db)
        .await
        .map_err(|err| {
            tracing::error!("Error fetching listing: {:?}", err);
            StatusCode::INTERNAL_SERVER_ERROR
        })?
        .ok_or(StatusCode::NOT_FOUND)?;

    let profile = Profile::find_by_id(listing.profile_id)
        .one(&db)
        .await
        .map_err(|err| {
            tracing::error!("Error fetching profile: {:?}", err);
            StatusCode::INTERNAL_SERVER_ERROR
        })?
        .ok_or(StatusCode::NOT_FOUND)?;

    let user_account_exists = UserAccount::find()
        .filter(user_account::Column::UserId.eq(current_user.id))
        .filter(user_account::Column::AccountId.eq(profile.account_id))
        .one(&db)
        .await
        .map_err(|err| {
            tracing::error!("Error checking user_account association: {:?}", err);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    if user_account_exists.is_none() {
        return Err(StatusCode::FORBIDDEN);
    }

    Listing::delete_by_id(listing.id)
        .exec(&db)
        .await
        .map_err(|err| {
            tracing::error!("Error deleting listing: {:?}", err);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    Ok(StatusCode::NO_CONTENT)
}

pub async fn search_listings(
    Extension(db): Extension<DatabaseConnection>,
    Query(q): Query<ListingSearch>,
) -> Result<Json<Vec<listing::Model>>, StatusCode> {
    tracing::info!("Searching listings with query: {:?}", q);
    
    let listings = Listing::find()
        .filter(listing::Column::Title.like(format!("%{}%", q.q).as_str()))
        .all(&db)
        .await
        .map_err(|err| {
            tracing::error!("Error searching listings: {:?}", err);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    Ok(Json(listings))
}

pub async fn get_listing_with_attributes(
    Extension(db): Extension<DatabaseConnection>,
    Path(id): Path<Uuid>,
) -> Result<Json<crate::models::listing::ListingWithAttributes>, StatusCode> {
    tracing::info!("Fetching listing with attributes for ID: {}", id);
    
    // Get the listing
    let listing = Listing::find_by_id(id)
        .one(&db)
        .await
        .map_err(|err| {
            tracing::error!("Error fetching listing: {:?}", err);
            StatusCode::INTERNAL_SERVER_ERROR
        })?
        .ok_or(StatusCode::NOT_FOUND)?;
    
    // Get the attributes
    let attributes = ListingAttribute::find()
        .filter(listing_attribute::Column::ListingId.eq(Some(id)))
        .all(&db)
        .await
        .map_err(|err| {
            tracing::error!("Error fetching listing attributes: {:?}", err);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;
    
    // Combine them
    let listing_with_attributes = crate::models::listing::ListingWithAttributes::from((listing, attributes));
    
    Ok(Json(listing_with_attributes))
}