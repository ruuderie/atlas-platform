use axum::{
    body::Body,
    http::{Request, StatusCode},
    Router,
};
use sea_orm::{Database, DatabaseConnection, ActiveModelTrait};
use sea_orm_migration::MigratorTrait;
use tower::ServiceExt;
use serde_json::json;
use std::env;
use uuid::Uuid;
use chrono::Utc;
use crate::{api, migration};
use http_body_util::BodyExt;
use super::test_utils;
use crate::models::ad_purchase::{AdStatus, AdPurchase};

async fn setup_test_app() -> (Router, DatabaseConnection) {
    let database_url = env::var("TEST_DATABASE_URL_LOCAL")
        .unwrap_or_else(|_| env::var("TEST_DATABASE_URL")
        .unwrap_or_else(|_| "postgres://postgres:postgres@localhost:5433/oplydbtest".to_string()));

    let db = Database::connect(&database_url)
        .await
        .expect("Failed to connect to test database");

    let _ = migration::Migrator::fresh(&db).await;
    
    migration::Migrator::up(&db, None)
        .await
        .expect("Failed to run migrations");

    let rate_limiter = crate::middleware::rate_limiter::RateLimiter::new();
    let app = api::create_router(db.clone())
        .layer(axum::Extension(db.clone()))
        .layer(axum::Extension(rate_limiter));
    (app, db)
}

#[tokio::test]
async fn test_ad_purchase_crud() {
    let (app, db) = setup_test_app().await;
    let (admin_user, admin_token) = test_utils::create_and_login_admin_user(&app, &db).await;

    // To create an AdPurchase, we might need a Profile and a Listing
    // creating a simple profile manually
    let tenant = test_utils::create_test_tenant(&db).await;
    let tenant_id = tenant.id;

    // Create an account
    let account = crate::entities::account::ActiveModel {
        id: sea_orm::Set(Uuid::new_v4()),
        tenant_id: sea_orm::Set(tenant.id),
        name: sea_orm::Set("Test Account".to_string()),
        is_active: sea_orm::Set(true),
        created_at: sea_orm::Set(Utc::now()),
        updated_at: sea_orm::Set(Utc::now()),
        ..Default::default()
    }.insert(&db).await.unwrap();

    // Create user_account mapping
    crate::entities::user_account::ActiveModel {
        id: sea_orm::Set(Uuid::new_v4()),
        user_id: sea_orm::Set(admin_user.id),
        account_id: sea_orm::Set(account.id),
        role: sea_orm::Set(crate::entities::user_account::UserRole::Owner),
        created_at: sea_orm::Set(Utc::now()),
        updated_at: sea_orm::Set(Utc::now()),
        ..Default::default()
    }.insert(&db).await.unwrap();

    // Create profile
    let profile = crate::entities::profile::ActiveModel {
        id: sea_orm::Set(Uuid::new_v4()),
        account_id: sea_orm::Set(account.id),
        tenant_id: sea_orm::Set(tenant.id),
        profile_type: sea_orm::Set(crate::entities::profile::ProfileType::Individual),
        display_name: sea_orm::Set("Admin".to_string()),
        contact_info: sea_orm::Set("admin@example.com".to_string()),
        is_active: sea_orm::Set(true),
        properties: sea_orm::Set(None),
        created_at: sea_orm::Set(Utc::now()),
        updated_at: sea_orm::Set(Utc::now()),
        ..Default::default()
    }.insert(&db).await.unwrap();

    let profile_id = profile.id;

    // Create a listing for the ad to attach to
    let listing = crate::entities::listing::ActiveModel {
        id: sea_orm::Set(Uuid::new_v4()),
        profile_id: sea_orm::Set(profile_id),
        tenant_id: sea_orm::Set(tenant.id),
        title: sea_orm::Set("Test Listing".to_string()),
        description: sea_orm::Set("A test listing for an ad.".to_string()),
        listing_type: sea_orm::Set("Business".to_string()),
        status: sea_orm::Set(crate::models::listing::ListingStatus::Active),
        is_featured: sea_orm::Set(false),
        is_based_on_template: sea_orm::Set(false),
        is_ad_placement: sea_orm::Set(false),
        is_active: sea_orm::Set(true),
        properties: sea_orm::Set(None),
        country: sea_orm::Set(Some("United States".to_string())),
        state: sea_orm::Set(Some("CA".to_string())),
        city: sea_orm::Set(Some("San Francisco".to_string())),
        created_at: sea_orm::Set(Utc::now()),
        updated_at: sea_orm::Set(Utc::now()),
        ..Default::default()
    }.insert(&db).await.unwrap();

    let listing_id = listing.id;

    let start_date = Utc::now();
    let end_date = Utc::now() + chrono::Duration::days(30);

    let payload = json!({
        "profile_id": profile_id,
        "listing_id": listing_id,
        "start_date": start_date,
        "end_date": end_date,
        "status": "Pending",
        "price": 100.5
    });

    let create_req = Request::builder()
        .header("Host", "localhost")
        .method("POST")
        .uri("/api/ad-purchases")
        .header("Content-Type", "application/json")
        .header("Authorization", format!("Bearer {}", admin_token))
        .body(Body::from(payload.to_string()))
        .unwrap();

    let create_res = app.clone().oneshot(create_req).await.unwrap();
    
    // Print the response if not CREATED to help with debugging FK errors
    let status = create_res.status();
    let body_bytes = axum::body::to_bytes(create_res.into_body(), usize::MAX).await.unwrap();
    if status != StatusCode::CREATED {
        panic!("Failed to create ad purchase. Status: {}, Body: {}", status, String::from_utf8_lossy(&body_bytes));
    }
    
    assert_eq!(status, StatusCode::CREATED, "Failed to create ad_purchase");
    let ad_purchase: AdPurchase = serde_json::from_slice(&body_bytes).unwrap();

    // GET AdPurchases List
    let get_all_req = Request::builder()
        .header("Host", "localhost")
        .method("GET")
        .uri("/api/admin/ad-purchases")
        .header("Authorization", format!("Bearer {}", admin_token))
        .body(Body::empty())
        .unwrap();

    let get_all_res = app.clone().oneshot(get_all_req).await.unwrap();
    assert_eq!(get_all_res.status(), StatusCode::OK);

    // Cancel AdPurchase
    let cancel_req = Request::builder()
        .header("Host", "localhost")
        .method("POST")
        .uri(format!("/api/admin/ad-purchases/{}/cancel", ad_purchase.id))
        .header("Authorization", format!("Bearer {}", admin_token))
        .body(Body::empty())
        .unwrap();

    let cancel_res = app.clone().oneshot(cancel_req).await.unwrap();
    assert_eq!(cancel_res.status(), StatusCode::OK);
}
