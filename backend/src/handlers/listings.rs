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
    DatabaseConnection, EntityTrait, Set, QueryFilter, ColumnTrait, ActiveModelTrait, TransactionTrait,
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

#[derive(Deserialize)]
pub struct ListingSearch {
    q: String,
}

pub fn public_routes(db: DatabaseConnection) -> Router<DatabaseConnection> {
    Router::new()
        .route("/listings", get(get_listings))
        .route("/listing/:id", get(get_listing_by_id))
        .route("/listings/search", get(search_listings))
        .with_state(db)
}

pub fn authenticated_routes() -> Router<DatabaseConnection> {
    Router::new()
        .route("/listings", post(create_listing))
        .route("/listings/:id", put(update_listing))
        .route("/listings/:id", delete(delete_listing))
        // Add other authenticated listing routes here
}

pub async fn get_listings(
    Extension(db): Extension<DatabaseConnection>,
    Query(params): Query<HashMap<String, String>>,
) -> Result<impl IntoResponse, StatusCode> {
    let directory_id = params.get("directory_id")
        .and_then(|id| Uuid::parse_str(id).ok())
        .ok_or(StatusCode::BAD_REQUEST)?;
    tracing::info!("Fetching listings for Directory ID: {}", directory_id);

    let listings = Listing::find()
        .filter(listing::Column::DirectoryId.eq(directory_id))
        .all(&db)
        .await
        .map_err(|err| {
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
) -> Result<Json<listing::Model>, StatusCode> {
    let txn = db.begin().await.map_err(|err| {
        tracing::error!("Error starting transaction: {:?}", err);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    let user_accounts = UserAccount::find()
        .filter(user_account::Column::UserId.eq(current_user.id))
        .filter(user_account::Column::AccountId.eq(input.profile_id))
        .all(&txn)
        .await
        .map_err(|err| {
            tracing::error!("Error fetching user accounts: {:?}", err);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    let profile = Profile::find_by_id(input.profile_id)
        .one(&txn)
        .await
        .map_err(|err| {
            tracing::error!("Error fetching profile: {:?}", err);
            StatusCode::INTERNAL_SERVER_ERROR
        })?
        .ok_or(StatusCode::NOT_FOUND)?;

    let user_account_exists = user_accounts
        .iter()
        .any(|ua| ua.account_id == profile.id);

    if !user_account_exists {
        return Err(StatusCode::FORBIDDEN);
    }

    let new_listing = listing::ActiveModel {
        id: Set(Uuid::new_v4()),
        profile_id: Set(profile.id),
        directory_id: Set(profile.directory_id),
        title: Set(input.title),
        description: Set(input.description),
        category_id: Set(input.category_id),
        listing_type: Set(input.listing_type),
        price: Set(input.price),
        price_type: Set(input.price_type),
        country: Set(input.country),
        state: Set(input.state),
        city: Set(input.city),
        neighborhood: Set(input.neighborhood),
        latitude: Set(input.latitude),
        longitude: Set(input.longitude),
        additional_info: Set(input.additional_info),
        status: Set(ListingStatus::Pending),
        is_featured: Set(input.is_featured),
        is_based_on_template: Set(input.is_based_on_template),
        based_on_template_id: Set(Some(input.template_id)),
        is_ad_placement: Set(input.is_ad_placement),
        is_active: Set(input.is_active),
        created_at: Set(Utc::now()),
        updated_at: Set(Utc::now()),
    };

    let inserted_listing = new_listing.insert(&txn).await.map_err(|err| {
        tracing::error!("Error creating listing: {:?}", err);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    if let template_id = input.template_id {
        let template_attributes = ListingAttribute::find()
            .filter(listing_attribute::Column::TemplateId.eq(template_id))
            .all(&txn)
            .await
            .map_err(|err| {
                tracing::error!("Error fetching template attributes: {:?}", err);
                StatusCode::INTERNAL_SERVER_ERROR
            })?;

        for attr in template_attributes {
            let new_attr = listing_attribute::ActiveModel {
                id: Set(Uuid::new_v4()),
                listing_id: Set(Some(inserted_listing.id)),
                template_id: Set(None),
                attribute_type: Set(attr.attribute_type),
                attribute_key: Set(attr.attribute_key),
                value: Set(attr.value),
                created_at: Set(Utc::now()),
                updated_at: Set(Utc::now()),
            };
            new_attr.insert(&txn).await.map_err(|err| {
                tracing::error!("Error creating listing attribute: {:?}", err);
                StatusCode::INTERNAL_SERVER_ERROR
            })?;
        }
    }

    txn.commit().await.map_err(|err| {
        tracing::error!("Error committing transaction: {:?}", err);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    Ok(Json(inserted_listing))
}

pub async fn update_listing(
    Extension(db): Extension<DatabaseConnection>,
    Extension(current_user): Extension<user::Model>,
    Path(id): Path<Uuid>,
    Json(input): Json<ListingUpdate>,
) -> Result<Json<listing::Model>, StatusCode> {
    let existing_listing = Listing::find_by_id(id)
        .one(&db)
        .await
        .map_err(|err| {
            tracing::error!("Error fetching listing: {:?}", err);
            StatusCode::INTERNAL_SERVER_ERROR
        })?
        .ok_or(StatusCode::NOT_FOUND)?;

    let profile = Profile::find_by_id(existing_listing.profile_id)
        .one(&db)
        .await
        .map_err(|err| {
            tracing::error!("Error fetching profile: {:?}", err);
            StatusCode::INTERNAL_SERVER_ERROR
        })?
        .ok_or(StatusCode::NOT_FOUND)?;

    let user_account_exists = UserAccount::find()
        .filter(user_account::Column::UserId.eq(current_user.id))
        .filter(user_account::Column::AccountId.eq(profile.id))
        .one(&db)
        .await
        .map_err(|err| {
            tracing::error!("Error checking user_account association: {:?}", err);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    if user_account_exists.is_none() {
        return Err(StatusCode::FORBIDDEN);
    }

    let mut listing_active_model: listing::ActiveModel = existing_listing.into();

    // Update fields if provided in input
    if let title = input.title { listing_active_model.title = Set(title); }
    if let description = input.description { listing_active_model.description = Set(description); }
    if let category_id = input.category_id { listing_active_model.category_id = Set(category_id); }
    if let listing_type = input.listing_type { listing_active_model.listing_type = Set(listing_type); }
    if let price = input.price { listing_active_model.price = Set(price); }
    if let price_type = input.price_type { listing_active_model.price_type = Set(price_type); }
    if let country = input.country { listing_active_model.country = Set(country); }
    if let state = input.state { listing_active_model.state = Set(state); }
    if let city = input.city { listing_active_model.city = Set(city); }
    if let neighborhood = input.neighborhood { listing_active_model.neighborhood = Set(neighborhood); }
    if let latitude = input.latitude { listing_active_model.latitude = Set(latitude); }
    if let longitude = input.longitude { listing_active_model.longitude = Set(longitude); }
    if let additional_info = input.additional_info { listing_active_model.additional_info = Set(additional_info.unwrap()); }
    if let is_featured = input.is_featured { listing_active_model.is_featured = Set(is_featured); }
    if let is_active = input.is_active { listing_active_model.is_active = Set(is_active); }
    if let is_ad_placement = input.is_ad_placement { listing_active_model.is_ad_placement = Set(is_ad_placement); }
    if let is_based_on_template = input.is_based_on_template { listing_active_model.is_based_on_template = Set(is_based_on_template); }
    if let based_on_template_id = input.based_on_template_id { listing_active_model.based_on_template_id = Set(based_on_template_id); }
    if let status = input.status { listing_active_model.status = Set(status); }

    listing_active_model.updated_at = Set(Utc::now());

    let updated_listing = listing_active_model.update(&db).await.map_err(|err| {
        tracing::error!("Error updating listing: {:?}", err);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

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
        .filter(user_account::Column::AccountId.eq(profile.id))
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
    Query(query): Query<ListingSearch>,
) -> Result<Json<Vec<listing::Model>>, StatusCode> {
    let listings = Listing::find()
        .filter(listing::Column::Title.like(format!("%{}%", query.q).as_str()))
        .all(&db)
        .await
        .map_err(|err| {
            tracing::error!("Error searching listings: {:?}", err);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    Ok(Json(listings))
}