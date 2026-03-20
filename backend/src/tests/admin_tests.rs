use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use serde_json::json;
use tower::ServiceExt;
use uuid::Uuid;

use crate::tests::api_tests::setup_test_app;
use crate::tests::test_utils;
use crate::models::user::{UserAdminView, UserLogin};
use crate::models::listing::ListingStatus;
use crate::entities::listing;
use sea_orm::{EntityTrait, QueryFilter, ColumnTrait, ActiveModelTrait, Set};

#[tokio::test]
async fn test_admin_user_management() {
    let (app, db) = setup_test_app().await;
    
    // Create admin user
    let (admin_user, admin_token) = test_utils::create_and_login_admin_user(&app, &db).await;
    
    // Create a regular user
    let directory_type = test_utils::create_test_directory_type(&db).await;
    let directory = test_utils::create_test_directory(&db, directory_type.id).await;
    let mut username = format!("regularuser{}", Uuid::new_v4());
    let (status, login_res) = test_utils::register_test_user(&app, directory.id, &mut username).await;
    assert_eq!(status, StatusCode::CREATED);
    
    let regular_user_id = login_res["user"]["id"].as_str().unwrap().to_string();
    
    // 1. List users
    let response = app.clone()
        .oneshot(
            Request::builder()
                .header("Host", "localhost")
                .method("GET")
                .uri("/admin/users")
                .header("Authorization", format!("Bearer {}", admin_token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    
    // 2. Get specific user
    let response = app.clone()
        .oneshot(
            Request::builder()
                .header("Host", "localhost")
                .method("GET")
                .uri(format!("/admin/users/{}", regular_user_id).as_str())
                .header("Authorization", format!("Bearer {}", admin_token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    
    // 3. Toggle admin status
    let response = app.clone()
        .oneshot(
            Request::builder()
                .header("Host", "localhost")
                .method("POST")
                .uri(format!("/admin/users/{}/toggle-admin", regular_user_id).as_str())
                .header("Authorization", format!("Bearer {}", admin_token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_admin_listing_approvals() {
    let (app, db) = setup_test_app().await;
    let (admin_user, admin_token) = test_utils::create_and_login_admin_user(&app, &db).await;
    
    // 1. Fetch pending listings
    let response = app.clone()
        .oneshot(
            Request::builder()
                .header("Host", "localhost")
                .method("GET")
                .uri("/admin/listings/pending")
                .header("Authorization", format!("Bearer {}", admin_token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_admin_statistics() {
    let (app, db) = setup_test_app().await;
    let (admin_user, admin_token) = test_utils::create_and_login_admin_user(&app, &db).await;
    
    // 1. Directory Stats
    let response = app.clone()
        .oneshot(
            Request::builder()
                .header("Host", "localhost")
                .method("GET")
                .uri("/admin/directory-stats")
                .header("Authorization", format!("Bearer {}", admin_token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    
    // 2. Ad Purchases Stats
    let response = app.clone()
        .oneshot(
            Request::builder()
                .header("Host", "localhost")
                .method("GET")
                .uri("/admin/ad-purchases/stats")
                .header("Authorization", format!("Bearer {}", admin_token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
}
