use sea_orm::{DatabaseConnection, Set, ActiveModelTrait, ConnectionTrait};
use uuid::Uuid;
use chrono::Utc;
use axum::{
    body::Body,
    http::{Request, StatusCode},
    Router,
};
use serde_json::json;
use tower::ServiceExt;
use hyper::body::Bytes;
use axum::body::HttpBody as _;  // Brings collect() into scope


use crate::entities::{directory_type, directory};

pub async fn create_test_directory_type<C: ConnectionTrait>(db: &C) -> directory_type::Model {
    let directory_type_id = Uuid::new_v4();
    let new_directory_type = directory_type::ActiveModel {
        id: Set(directory_type_id),
        name: Set("Test Directory Type".to_string()),
        description: Set("Test Directory Type Description".to_string()),
        created_at: Set(Utc::now()),
        updated_at: Set(Utc::now()),
    };

    new_directory_type.insert(db)
        .await
        .expect("Failed to create test directory type")
}

pub async fn create_test_directory<C: ConnectionTrait>(db: &C, directory_type_id: Uuid) -> directory::Model {
    let directory_id = Uuid::new_v4();
    println!("directory_id from create_test_directory: {:?}", directory_id);
    println!("directory_type_id from create_test_directory: {:?}", directory_type_id);
    let new_directory = directory::ActiveModel {
        id: Set(directory_id),
        directory_type_id: Set(directory_type_id),
        name: Set("Test Directory".to_string()),
        domain: Set("test.example.com".to_string()),
        description: Set("Test Directory Description".to_string()),
        created_at: Set(Utc::now()),
        updated_at: Set(Utc::now()),
    };

    new_directory.insert(db)
        .await
        .expect("Failed to create test directory")
    
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

    let body_bytes = response.into_body()
        .collect()
        .await
        .unwrap()
        .to_bytes();
    let body = String::from_utf8_lossy(&body_bytes).to_string();
    
    // Handle potential non-JSON responses
    serde_json::from_str(&body).unwrap_or_else(|_| {
        panic!("Failed to parse login response as JSON. Response body: {}", body)
    })
}
