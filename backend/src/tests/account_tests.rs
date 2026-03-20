use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use serde_json::json;
use tower::ServiceExt;
use uuid::Uuid;

use crate::tests::api_tests::setup_test_app;
use crate::tests::test_utils;

#[tokio::test]
async fn test_account_creation_and_listing() {
    let (app, db) = setup_test_app().await;
    
    // Register User A
    let directory_type = test_utils::create_test_directory_type(&db).await;
    let directory = test_utils::create_test_directory(&db, directory_type.id).await;
    let mut username_a = format!("accountuser{}", Uuid::new_v4());
    let (status, login_res_a) = test_utils::register_test_user(&app, directory.id, &mut username_a).await;
    assert_eq!(status, StatusCode::CREATED);
    
    let user_a_token = login_res_a["token"].as_str().unwrap().to_string();
    
    // 1. Fetch my accounts
    let response = app.clone()
        .oneshot(
            Request::builder()
                .header("Host", "localhost")
                .method("GET")
                .uri("/api/me/accounts")
                .header("Authorization", format!("Bearer {}", user_a_token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    
    // 2. Create a new account
    let response = app.clone()
        .oneshot(
            Request::builder()
                .header("Host", "localhost")
                .method("POST")
                .uri("/api/me/accounts")
                .header("Authorization", format!("Bearer {}", user_a_token))
                .header("Content-Type", "application/json")
                .body(Body::from(json!({
                    "name": format!("Org {}", Uuid::new_v4()),
                    "directory_id": directory.id,
                }).to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::CREATED);
}

#[tokio::test]
async fn test_user_account_invitations() {
    let (app, db) = setup_test_app().await;
    let (admin_user, admin_token) = test_utils::create_and_login_admin_user(&app, &db).await;
    
    // 1. Admin gets all accounts
    let response = app.clone()
        .oneshot(
            Request::builder()
                .header("Host", "localhost")
                .method("GET")
                .uri("/api/accounts")
                .header("Authorization", format!("Bearer {}", admin_token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
}
