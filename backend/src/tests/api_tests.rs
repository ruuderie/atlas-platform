use axum::{
    body::{Body, HttpBody},
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
use hyper::body::Bytes;
use super::test_utils;
use crate::models::directory_type::DirectoryTypeModel;
use crate::models::directory::DirectoryModel;
async fn setup_test_app() -> (Router, DatabaseConnection) {
    let database_url = env::var("TEST_DATABASE_URL")
        .unwrap_or_else(|_| "postgres://postgres:postgres@localhost:5432/business_directory_test".to_string());

    let db = Database::connect(&database_url)
        .await
        .expect("Failed to connect to test database");

    // Reset database state before each test
    migration::Migrator::fresh(&db)
        .await
        .expect("Failed to reset database");
    
    // Run migrations
    migration::Migrator::up(&db, None)
        .await
        .expect("Failed to run migrations");

    (api::create_router(db.clone()), db)
}
#[tokio::test]
async fn test_concurrent_logout_requests() {
    let (app, db) = setup_test_app().await;
    let directory_type = test_utils::create_test_directory_type(&db).await;
    let directory = test_utils::create_test_directory(&db, directory_type.id).await;
    let unique_id = Uuid::new_v4();
    let (status, _) = test_utils::register_test_user(&app, directory.id, format!("concurrentuser{}", unique_id).as_str()).await;
    assert_eq!(status, StatusCode::CREATED);

    let login_response = test_utils::login_test_user(&app, format!("concurrentuser{}@example.com", unique_id).as_str(), "password123").await;
    let token = login_response["token"].as_str().unwrap();
    println!("TEST LOG: from test_concurrent_logout_requests and token: {:?}", token);

    // Send multiple logout requests concurrently
    let requests = (0..3).map(|_| {
        let app = app.clone();
        let token = token.to_string();
        tokio::spawn(async move {
            app.oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/logout")
                    .header("Authorization", format!("Bearer {}", token))
                    .body(Body::empty())
                    .unwrap()
            ).await.unwrap()
        })
    });

    let responses = futures::future::join_all(requests).await;
    for response in responses {
        let res = response.unwrap();
        println!("response from test_concurrent_logout_requests: {:?}", res);
        assert!(
            res.status() == StatusCode::OK || res.status() == StatusCode::UNAUTHORIZED,
            "Unexpected status code: {}",
            res.status()
        );
    }

    // Verify session is invalid
    let response = app.clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/validate-session")
                .header("Authorization", format!("Bearer {}", token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    println!("response from test_concurrent_logout_requests: {:?}", response);
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_logout_with_invalid_token() {
    let (app, _) = setup_test_app().await;
    
    // Changed URI to "/api/logout" to match route nesting
    let response = app.clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/logout")  // Added /api prefix
                .header("Authorization", "Bearer invalid_token")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    println!("TEST LOG: from test_logout_with_invalid_token and status: {:?} , body: {:?}", response.status(), response);
    println!("TEST LOG: expected status: {:?}", StatusCode::UNAUTHORIZED);
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}
#[tokio::test]
async fn test_session_validation_and_expiry() {
    let (app, db) = setup_test_app().await;
    let directory_type = test_utils::create_test_directory_type(&db).await;
    let directory = test_utils::create_test_directory(&db, directory_type.id).await;
    
    // Register and login
    let username = format!("testuser{}", Uuid::new_v4());
    let (_, _) = test_utils::register_test_user(&app, directory.id, &username).await;
    let login_response = test_utils::login_test_user(&app, format!("{}@example.com", username).as_str(), "password123").await;
    let token = login_response["token"].as_str().unwrap();
    
    // Test valid session
    let validation_response = app.clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/validate-session")
                .header("Authorization", format!("Bearer {}", token))
                .body(Body::empty())
                .unwrap()
        )
        .await
        .unwrap();
    assert_eq!(validation_response.status(), StatusCode::OK);
}
#[tokio::test]
async fn test_directory_operations() {
    let (app, db) = setup_test_app().await;
    
    // Create and login as admin user
    let admin_username = format!("admin{}", Uuid::new_v4());
    let (admin_user, admin_token) = test_utils::create_and_login_admin_user(&app, &db).await;
    
    let directory_type_id = Uuid::new_v4();
    
    // Test creating directory type
    let response = app.clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/admin/directory-types")
                .header("Content-Type", "application/json")
                .header("Authorization", format!("Bearer {}", admin_token))
                .body(Body::from(json!({
                    "name": format!("Test Directory Type {}", directory_type_id.to_string()),
                    "description": format!("Test Description {}", directory_type_id.to_string()),
                }).to_string()))
                .unwrap()
        )
        .await
        .unwrap();
    println!("TEST LOG: from create directory type test_directory_operations and response: {:?}", response);
    let status = response.status();
    let body_bytes = response.into_body()
        .collect()
        .await
        .unwrap()
        .to_bytes();
    println!("TEST LOG: from create directory type test_directory_operations and body_bytes: {:?}", body_bytes);
    let body = String::from_utf8_lossy(&body_bytes).to_string();
    let directory_type: DirectoryTypeModel = serde_json::from_slice(body.as_bytes()).unwrap();
    let directory_type_id = directory_type.id;
    println!("TEST LOG: from create directory type test_directory_operations and directory_type: {:?}", directory_type);
    assert_eq!(status, StatusCode::CREATED);

    // Test fetching directory types
    let response = app.clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri(format!("/api/admin/directory-types/{}", directory_type_id.clone()).as_str())
                .header("Authorization", format!("Bearer {}", admin_token))
                .body(Body::empty())
                .unwrap()
        )
        .await
        .unwrap();
    println!("TEST LOG: from get directory type test_directory_operations and response: {:?}", response);
    assert_eq!(response.status(), StatusCode::OK);
    // Test updating directory type
    let response = app.clone()
        .oneshot(
            Request::builder()
                .method("PUT")
                .uri(format!("/api/admin/directory-types/{}", directory_type_id.clone().to_string()).as_str())
                .header("Content-Type", "application/json")
                .header("Authorization", format!("Bearer {}", admin_token))
                .body(Body::from(json!({
                    "name": format!("Updated Directory Type {}", directory_type_id.to_string()),
                    "description": format!("Updated Description {}", directory_type_id.to_string()),
                }).to_string()))
                .unwrap()
        )
        .await
        .unwrap();
    println!("TEST LOG: from PUT directory type test_directory_operations and response: {:?}", response);
    assert_eq!(response.status(), StatusCode::OK);
    
    // Test creating directory
    let response = app.clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/admin/directories/")
                .header("Content-Type", "application/json")
                .header("Authorization", format!("Bearer {}", admin_token))
                .body(Body::from(json!({
                    "name": "Test Directory",
                    "description": "Test Directory Description",
                    "directory_type_id": directory_type_id.clone(),
                    "domain": "test.com"
                }).to_string()))
                .unwrap()
        )
        .await
        .unwrap();
    
    let status = response.status();
    let body_bytes = response.into_body()
        .collect()
        .await
        .unwrap()
        .to_bytes();
    let body = String::from_utf8_lossy(&body_bytes).to_string();
    let directory: DirectoryModel = serde_json::from_slice(body.as_bytes()).unwrap();
    println!("TEST LOG: from POST CREATE directory test_directory_operations and status: {:?}", status);
    assert_eq!(status, StatusCode::CREATED);
    println!("TEST LOG: from ABOUT TO UPDATE directory test_directory_operations and directory: {:?}", directory.id);
    // update directory
    let response = app.clone()
    .oneshot(
        Request::builder()
            .method("PUT")
            .uri(format!("/api/admin/directories/{}", directory.id))
            .header("Content-Type", "application/json")
            .header("Authorization", format!("Bearer {}", admin_token))
            .body(Body::from(json!({
                "name": "Updated Directory",
                "description": "Updated Description",
                "directory_type_id": directory_type_id, // Include this
                "domain": "updated.com"                 // Include this
            }).to_string()))
            .unwrap()
    )
    .await
    .unwrap();
assert_eq!(response.status(), StatusCode::OK); // Updates return 200, not 201

    // delete directory
    let response = app.clone()
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri(format!("/api/admin/directories/{}", directory.id).as_str())
                .header("Authorization", format!("Bearer {}", admin_token))
                .body(Body::empty())
                .unwrap()
        )
        .await
        .unwrap();  
    println!("TEST LOG: from DELETE directory test_directory_operations and response: {:?}", response);
    assert_eq!(response.status(), StatusCode::NO_CONTENT);
    
    
    // Test deleting directory type
    let response = app.clone()
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri(format!("/api/admin/directory-types/{}", directory_type_id).as_str())
                .header("Authorization", format!("Bearer {}", admin_token))
                .body(Body::empty())
                .unwrap()
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::NO_CONTENT);
    
}
#[tokio::test]
async fn test_listing_operations() {
    let (app, db) = setup_test_app().await;
    let directory_type = test_utils::create_test_directory_type(&db).await;
    let directory = test_utils::create_test_directory(&db, directory_type.id).await;
    
    // Register and login a user
    let username = format!("testuser{}", Uuid::new_v4());
    let (_, _) = test_utils::register_test_user(&app, directory.id, &username).await;
    let login_response = test_utils::login_test_user(&app, format!("{}@example.com", username).as_str(), "password123").await;
    let token = login_response["token"].as_str().unwrap();
    
    // Test creating a listing
    let create_listing_response = app.clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/listings")
                .header("Authorization", format!("Bearer {}", token))
                .header("Content-Type", "application/json")
                .body(Body::from(json!({
                    "name": "Test Listing",
                    "description": "Test Description",
                    "directory_id": directory.id
                }).to_string()))
                .unwrap()
        )
        .await
        .unwrap();
    assert_eq!(create_listing_response.status(), StatusCode::CREATED);
}