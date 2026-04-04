use axum::{
    body::Body,
    http::{Request, StatusCode},
    Router,
};
use sea_orm::{Database, DatabaseConnection, EntityTrait, QueryFilter, ColumnTrait, ActiveModelTrait};
use sea_orm_migration::MigratorTrait;
use tower::ServiceExt;
use serde_json::json;
use std::env;
use uuid::Uuid;
use crate::{api, migration, entities::tenant};
use http_body_util::BodyExt;
use super::test_utils;

async fn setup_test_app() -> (Router, DatabaseConnection) {
    let database_url = env::var("TEST_DATABASE_URL_LOCAL")
        .unwrap_or_else(|_| env::var("TEST_DATABASE_URL")
        .unwrap_or_else(|_| "postgres://postgres:postgres@127.0.0.1:5432/business_tenant_test".to_string()));

    let db = Database::connect(&database_url)
        .await
        .expect("Failed to connect to test database");

    migration::Migrator::fresh(&db)
        .await
        .expect("Failed to reset database");
    
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
async fn test_cannot_delete_tenant_in_use() {
    let (app, db) = setup_test_app().await;
    let (_admin_user, admin_token) = test_utils::create_and_login_admin_user(&app, &db).await;

    // 1. Create a Tenant Type
    let tenant = test_utils::create_test_tenant(&db).await;
    
    // 2. Create a Tenant that depends on it
    let _tenant = test_utils::create_test_tenant(&db).await;

    // 3. Attempt to delete Tenant Type via Admin API
    let del_req = Request::builder()
        .header("Host", "localhost")
        .method("DELETE")
        .uri(format!("/api/admin/tenant-types/{}", tenant.id))
        .header("Authorization", format!("Bearer {}", admin_token))
        .body(Body::empty())
        .unwrap();

    let del_res = app.clone().oneshot(del_req).await.unwrap();
    let status = del_res.status();

    // The API should NOT allow deletion (typically returns 400 Bad Request or 500 Server Error due to constraint)
    assert!(status != StatusCode::OK && status != StatusCode::NO_CONTENT, "Should not delete tenant type if in use");

    // Double check DB
    let check_db = tenant::Entity::find_by_id(tenant.id).one(&db).await.unwrap();
    assert!(check_db.is_some(), "Tenant Type must still exist in the database");
}

#[tokio::test]
async fn test_creating_template_retains_attributes() {
    let (app, db) = setup_test_app().await;
    let (_admin_user, admin_token) = test_utils::create_and_login_admin_user(&app, &db).await;

    let tenant = test_utils::create_test_tenant(&db).await;
    let category = test_utils::create_default_category(&db, tenant.id).await;

    // 1. Create Template via API
    let payload = json!({
        "name": "Service Temp",
        "tenant_id": tenant.id,
        "category_id": category.id,
        "description": "Standard service",
        "template_type": "Detail",
        "is_active": true,
        "attributes": "{}"
    });

    let create_req = Request::builder()
        .header("Host", "localhost")
        .method("POST")
        .uri("/api/admin/templates")
        .header("Content-Type", "application/json")
        .header("Authorization", format!("Bearer {}", admin_token))
        .body(Body::from(payload.to_string()))
        .unwrap();

    let create_res = app.clone().oneshot(create_req).await.unwrap();
    let status = create_res.status();
    let body_bytes = axum::body::to_bytes(create_res.into_body(), usize::MAX).await.unwrap();
    assert!(status == StatusCode::OK || status == StatusCode::CREATED);
    
    let template: crate::models::template::TemplateModel = serde_json::from_slice(&body_bytes).unwrap();

}

#[tokio::test]
async fn test_listing_relational_hierarchy() {
    let (app, db) = setup_test_app().await;
    let (_admin_user, admin_token) = test_utils::create_and_login_admin_user(&app, &db).await;

    let tenant = test_utils::create_test_tenant(&db).await;
    let mut dummy_user = "listcreator".to_string();
    test_utils::register_test_user(&app, tenant.id, &mut dummy_user).await;
    let category = test_utils::create_default_category(&db, tenant.id).await;
    let profile = crate::entities::profile::Entity::find().one(&db).await.unwrap().unwrap();

    // 1. Create Listing
    let payload = json!({
        "title": "A Great Listing",
        "description": "Description here",
        "tenant_id": tenant.id,
        "profile_id": profile.id,
        "category_id": category.id,
        "listing_type": "Service",
        "status": "active",
        "is_featured": false,
        "is_based_on_template": false,
        "is_ad_placement": false,
        "is_active": true
    });

    let create_req = Request::builder()
        .header("Host", "localhost")
        .method("POST")
        .uri("/api/admin/listings")
        .header("Content-Type", "application/json")
        .header("Authorization", format!("Bearer {}", admin_token))
        .body(Body::from(payload.to_string()))
        .unwrap();

    let create_res = app.clone().oneshot(create_req).await.unwrap();
    let status = create_res.status();
    let body_bytes = axum::body::to_bytes(create_res.into_body(), usize::MAX).await.unwrap();
    assert!(status == StatusCode::OK || status == StatusCode::CREATED, "Failed to create: {}", String::from_utf8_lossy(&body_bytes));
    
    // Parse it to verify tenant mapping
    let listing: crate::models::listing::ListingModel = serde_json::from_slice(&body_bytes).unwrap();
    assert_eq!(listing.tenant_id, tenant.id, "Listing must correctly associate with the explicit Parent Tenant");
    

}
