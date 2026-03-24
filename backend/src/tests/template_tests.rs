use axum::{
    body::Body,
    http::{Request, StatusCode},
    Router,
};
use sea_orm::{Database, DatabaseConnection};
use sea_orm_migration::MigratorTrait;
use tower::ServiceExt;
use serde_json::json;
use std::env;
use uuid::Uuid;
use crate::{api, migration};
use http_body_util::BodyExt;
use super::test_utils;

async fn setup_test_app() -> (Router, DatabaseConnection) {
    let database_url = env::var("TEST_DATABASE_URL_LOCAL")
        .unwrap_or_else(|_| env::var("TEST_DATABASE_URL")
        .unwrap_or_else(|_| "postgres://postgres:postgres@127.0.0.1:5432/business_directory_test".to_string()));

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
async fn test_template_crud() {
    let (app, db) = setup_test_app().await;
    let (_admin_user, admin_token) = test_utils::create_and_login_admin_user(&app, &db).await;

    // Create a directory and category first to satisfy foreign keys
    let directory_type = test_utils::create_test_directory_type(&db).await;
    let directory = test_utils::create_test_directory(&db, directory_type.id).await;
    let category = test_utils::create_default_category(&db, directory_type.id).await;

    let payload = json!({
        "name": "Standard Template",
        "directory_id": directory.id,
        "category_id": category.id,
        "description": "A standard template for listings",
        "template_type": "Listing",
        "is_active": true,
        "attributes": "{}"
    });

    // Create Template
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
    if status != StatusCode::OK && status != StatusCode::CREATED {
        panic!("Failed to create template. Status: {}, Body: {}", status, String::from_utf8_lossy(&body_bytes));
    }
    let template: crate::models::template::TemplateModel = serde_json::from_slice(&body_bytes).unwrap();
    assert_eq!(template.name, "Standard Template");

    // Get Template
    let get_req = Request::builder()
        .header("Host", "localhost")
        .method("GET")
        .uri(format!("/api/admin/templates/{}", template.id))
        .header("Authorization", format!("Bearer {}", admin_token))
        .body(Body::empty())
        .unwrap();

    let get_res = app.clone().oneshot(get_req).await.unwrap();
    assert_eq!(get_res.status(), StatusCode::OK);

    // Update Template
    let update_payload = json!({
        "id": template.id,
        "name": "Updated Template",
        "directory_id": directory.id,
        "description": "Updated description",
        "template_type": "Premium",
        "is_active": false,
        "attributes": "{ \"color\": \"blue\" }"
    });

    let put_req = Request::builder()
        .header("Host", "localhost")
        .method("PUT")
        .uri(format!("/api/admin/templates/{}", template.id))
        .header("Content-Type", "application/json")
        .header("Authorization", format!("Bearer {}", admin_token))
        .body(Body::from(update_payload.to_string()))
        .unwrap();

    let put_res = app.clone().oneshot(put_req).await.unwrap();
    assert_eq!(put_res.status(), StatusCode::OK);

    // Delete Template
    let del_req = Request::builder()
        .header("Host", "localhost")
        .method("DELETE")
        .uri(format!("/api/admin/templates/{}", template.id))
        .header("Authorization", format!("Bearer {}", admin_token))
        .body(Body::empty())
        .unwrap();

    let del_res = app.clone().oneshot(del_req).await.unwrap();
    assert_eq!(del_res.status(), StatusCode::NO_CONTENT);
}
