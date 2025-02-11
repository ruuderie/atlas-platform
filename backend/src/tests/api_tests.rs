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
    
    // Run migrations
    migration::Migrator::up(&db, None)
        .await
        .expect("Failed to run migrations");

    (api::create_router(db.clone()), db)
}
pub async fn register_test_user(
    app: &Router,
    directory_id: Uuid,
    username: &str,
) -> (StatusCode, String) {
    let response = app.clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/register")
                .header("Content-Type", "application/json")
                .body(Body::from(
                    json!({
                        "directory_id": directory_id,
                        "username": username,
                        "first_name": "Test",
                        "last_name": "User",
                        "email": format!("{}@example.com", username),
                        "password": "password123",
                        "phone": "1234567890"
                    })
                    .to_string(),
                ))
                .unwrap(),
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

    (status, body)
}

pub async fn login_test_user(
    app: &Router,
    email: &str,
    password: &str,
) -> serde_json::Value {
    let response = app.clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/login")
                .header("Content-Type", "application/json")
                .body(Body::from(
                    json!({
                        "email": email,
                        "password": password
                    })
                    .to_string(),
                ))
                .unwrap(),
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
    
    if status != StatusCode::OK {
        panic!("Login failed with status {}: {}", status, body);
    }
    
    serde_json::from_str(&body).unwrap_or_else(|e| {
        panic!("Failed to parse login response as JSON. Error: {}. Response body: {}", e, body)
    })
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
