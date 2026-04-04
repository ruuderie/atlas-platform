use axum::{
    extract::{State, Path, Json},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Router,
};
use sea_orm::{DatabaseConnection, EntityTrait, QueryFilter, ColumnTrait, Set, ActiveModelTrait};
use serde::Deserialize;
use serde_json::json;
use uuid::Uuid;
use chrono::Utc;

use crate::entities::{listing_ab_test, listing_ab_variant, listing};

#[derive(Deserialize)]
pub struct CreateTestPayload {
    pub status: String,
    pub traffic_split_strategy: String,
}

#[derive(Deserialize)]
pub struct CreateVariantPayload {
    pub name: String,
    pub is_control: bool,
}

pub fn authenticated_routes() -> Router<DatabaseConnection> {
    Router::new()
        .route("/api/listings/{id}/ab-tests", post(create_test).get(get_tests_for_listing))
        .route("/api/ab-tests/{id}/variants", post(create_variant).get(get_variants_for_test))
}

pub fn public_routes() -> Router<DatabaseConnection> {
    Router::new()
        .route("/api/ab-variants/{id}/view", post(increment_variant_view))
        .route("/api/ab-variants/{id}/conversion", post(increment_variant_conversion))
        .route("/api/listings/by-slug/{slug}/active-test", get(get_active_test_for_listing_slug))
}

pub async fn create_test(
    State(db): State<DatabaseConnection>,
    Path(listing_id): Path<Uuid>,
    Json(payload): Json<CreateTestPayload>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let new_test = listing_ab_test::ActiveModel {
        id: Set(Uuid::new_v4()),
        listing_id: Set(listing_id),
        status: Set(payload.status),
        traffic_split_strategy: Set(payload.traffic_split_strategy),
        created_at: Set(Utc::now()),
        updated_at: Set(Utc::now()),
    };

    let inserted = new_test.insert(&db).await.map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok((StatusCode::CREATED, Json(inserted)))
}

pub async fn get_tests_for_listing(
    State(db): State<DatabaseConnection>,
    Path(listing_id): Path<Uuid>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let tests = listing_ab_test::Entity::find()
        .filter(listing_ab_test::Column::ListingId.eq(listing_id))
        .all(&db)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(tests))
}

pub async fn create_variant(
    State(db): State<DatabaseConnection>,
    Path(test_id): Path<Uuid>,
    Json(payload): Json<CreateVariantPayload>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let new_variant = listing_ab_variant::ActiveModel {
        id: Set(Uuid::new_v4()),
        test_id: Set(test_id),
        name: Set(payload.name),
        is_control: Set(payload.is_control),
        views: Set(0),
        conversions: Set(0),
        created_at: Set(Utc::now()),
        updated_at: Set(Utc::now()),
    };

    let inserted = new_variant.insert(&db).await.map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok((StatusCode::CREATED, Json(inserted)))
}

pub async fn get_variants_for_test(
    State(db): State<DatabaseConnection>,
    Path(test_id): Path<Uuid>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let variants = listing_ab_variant::Entity::find()
        .filter(listing_ab_variant::Column::TestId.eq(test_id))
        .all(&db)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(variants))
}

pub async fn increment_variant_view(
    State(db): State<DatabaseConnection>,
    Path(variant_id): Path<Uuid>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let variant = listing_ab_variant::Entity::find_by_id(variant_id)
        .one(&db)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    if let Some(mut v) = variant {
        v.views += 1;
        let mut active_model: listing_ab_variant::ActiveModel = v.into();
        active_model.updated_at = Set(Utc::now());
        active_model.update(&db).await.map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
        Ok((StatusCode::OK, Json(json!({"message": "View incremented"}))))
    } else {
        Err((StatusCode::NOT_FOUND, "Variant not found".into()))
    }
}

pub async fn increment_variant_conversion(
    State(db): State<DatabaseConnection>,
    Path(variant_id): Path<Uuid>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let variant = listing_ab_variant::Entity::find_by_id(variant_id)
        .one(&db)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    if let Some(mut v) = variant {
        v.conversions += 1;
        let mut active_model: listing_ab_variant::ActiveModel = v.into();
        active_model.updated_at = Set(Utc::now());
        active_model.update(&db).await.map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
        Ok((StatusCode::OK, Json(json!({"message": "Conversion incremented"}))))
    } else {
        Err((StatusCode::NOT_FOUND, "Variant not found".into()))
    }
}

pub async fn get_active_test_for_listing_slug(
    State(db): State<DatabaseConnection>,
    Path(slug): Path<String>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    // 1. Get the listing by slug
    let listing_record = listing::Entity::find()
        .filter(listing::Column::Slug.eq(slug))
        .one(&db)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
        
    let list_data = match listing_record {
        Some(l) => l,
        None => return Err((StatusCode::NOT_FOUND, "Listing not found".into())),
    };

    // 2. Find an active test
    let active_test = listing_ab_test::Entity::find()
        .filter(listing_ab_test::Column::ListingId.eq(list_data.id))
        .filter(listing_ab_test::Column::Status.eq("Active"))
        .one(&db)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
        
    if let Some(test) = active_test {
        let variants = listing_ab_variant::Entity::find()
            .filter(listing_ab_variant::Column::TestId.eq(test.id))
            .all(&db)
            .await
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
            
        Ok(Json(json!({
            "test": test,
            "variants": variants,
        })))
    } else {
        Err((StatusCode::NOT_FOUND, "No active test found".into()))
    }
}
