use axum::{
    body::{Body, HttpBody},
    http::{Request, StatusCode},
    Router,
    extract::FromRequest,
};
use sea_orm::{Database, DatabaseConnection, EntityTrait, Set, ActiveModelTrait, ConnectionTrait, TransactionTrait};
use sea_orm_migration::MigratorTrait;
use tower::ServiceExt;
use serde_json::json;
use std::env;
use uuid::Uuid;
use crate::{api, migration};
use hyper::body::Bytes;
use chrono::Utc;
use super::test_utils;
use tokio::sync::Mutex;
use once_cell::sync::Lazy;
use tokio::sync::OnceCell;

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
    migration::Migrator::up(&db, None)
        .await
        .expect("Failed to run migrations");

    (api::create_router(db.clone()), db)
}

#[tokio::test]
async fn test_user_registration_and_login_flow() {
    let (app, db) = setup_test_app().await;
    let unique_id = Uuid::new_v4();
    // Create test directory USING THE TRANSACTION
    let directory_type = test_utils::create_test_directory_type(&db).await;
    let directory = test_utils::create_test_directory(&db, directory_type.id).await;

    // Test registration - will use transaction through app state
    let (status, body) = test_utils::register_test_user(&app, directory.id, format!("testuser{}", unique_id).as_str()).await;
    assert_eq!(status, StatusCode::CREATED, "Registration failed: {}", body);

    // Test login with same credentials
    let response = app.clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/login")
                .header("Content-Type", "application/json")
                .body(Body::from(
                    json!({
                        "email": format!("testuser{}@example.com", unique_id).as_str(),
                        "password": "password123"
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body_bytes = response.into_body()
        .collect()
        .await
        .unwrap()
        .to_bytes();
    let body = String::from_utf8_lossy(&body_bytes).to_string();
    // Verify response contains valid token
    let login_response: serde_json::Value = serde_json::from_str(&body.as_str()).unwrap();
    println!("login response from test_user_registration_and_login_flow: {}", login_response);
    assert!(login_response.get("token").is_some());
    assert!(login_response.get("refresh_token").is_some());
    assert!(login_response.get("user").is_some());
    let user = login_response.get("user").unwrap();
    assert!(user.get("id").is_some());
    assert!(user.get("email").is_some());
    assert!(user.get("first_name").is_some());
    assert!(user.get("last_name").is_some());
    assert!(user.get("is_admin").unwrap().as_bool().unwrap() == false);
}

#[tokio::test]
async fn test_logout_flow() {
    let (app, db) = setup_test_app().await;
    let unique_id = Uuid::new_v4();
    
    // Create test directory using direct DB connection
    let directory_type = test_utils::create_test_directory_type(&db).await;
    let directory = test_utils::create_test_directory(&db, directory_type.id).await;
    
    // Update URI to "/logout"
    let response = app.clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/logout")
                .header("Authorization", format!("Bearer {}", test_utils::login_test_user(&app, format!("logoutuser{}@example.com", unique_id).as_str(), "password123").await["token"].as_str().unwrap()))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_invalid_session_after_logout() {
    let (app, db) = setup_test_app().await;
    let txn = db.begin().await.expect("Failed to start transaction");
    
    // Create test data
    let directory_type = test_utils::create_test_directory_type(&txn).await;
    let directory = test_utils::create_test_directory(&txn, directory_type.id).await;
    let unique_id = Uuid::new_v4();
    let (status, _) = test_utils::register_test_user(&app, directory.id, format!("testuser{}", unique_id).as_str()).await;
    assert_eq!(status, StatusCode::CREATED);

    // Login and get session
    let login_response = test_utils::login_test_user(&app, format!("testuser{}@example.com", unique_id).as_str(), "password123").await;
    let token = login_response["token"].as_str().unwrap();

    // Logout - REMOVE Content-Type header
    let response = app.clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/logout")
                .header("Authorization", format!("Bearer {}", token))
                .body(Body::empty())  // Removed Content-Type header
                .unwrap(),
        )
        .await
        .unwrap();
    println!("response from test_invalid_session_after_logout: {:?}", response);
    assert_eq!(response.status(), StatusCode::OK);

    // Attempt to access protected resource
    let response = app.clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri(&format!("/profile/{}", login_response["user"]["id"].as_str().unwrap()))
                .header("Authorization", format!("Bearer {}", token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    println!("response from test_invalid_session_after_logout: {:?}", response);
    assert_eq!(response.status(), StatusCode::from_u16(401).unwrap());

    // Rollback transaction at end of test
    txn.rollback().await.expect("Failed to rollback transaction");
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
    
    // Update URI to "/logout"
    let response = app.clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/logout")
                .header("Authorization", "Bearer invalid_token")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_session_validation_after_logout() {
    let (app, db) = setup_test_app().await;
   // println!("app from test_session_validation_after_logout: {:?}", app);
    let directory_type = test_utils::create_test_directory_type(&db).await;
    println!("directory_type from test_session_validation_after_logout: {:?}", directory_type);
    let directory = test_utils::create_test_directory(&db, directory_type.id).await;
    println!("directory from test_session_validation_after_logout: {:?}", directory);
    let unique_id = Uuid::new_v4();
    let (status, _) = test_utils::register_test_user(&app, directory.id, format!("validationuser{}", unique_id).as_str()).await;
    assert_eq!(status, StatusCode::CREATED);

    let login_response = test_utils::login_test_user(&app, format!("validationuser{}@example.com", unique_id).as_str(), "password123").await;
    let token = login_response["token"].as_str().unwrap();

    // Initial validation should work
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
    assert_eq!(response.status(), StatusCode::OK);

    // Logout
    let response = app.clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/logout")
                .header("Authorization", format!("Bearer {}", token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    println!("response from test_session_validation_after_logout: {:?}", response);
    assert_eq!(response.status(), StatusCode::OK);

    // Post-logout validation should fail
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
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}