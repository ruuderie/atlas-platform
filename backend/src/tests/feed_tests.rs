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
async fn test_feed_creation_and_listing() {
    let (app, db) = setup_test_app().await;

    let tenant = test_utils::create_test_tenant(&db).await;
    let mut username_a = format!("feeduser{}", Uuid::new_v4());
    let (status, login_res_a) = test_utils::register_test_user(&app, tenant.id, &mut username_a).await;
    assert_eq!(status, StatusCode::CREATED);
    
    let user_a_token = login_res_a["token"].as_str().unwrap().to_string();

    // 1. Create a Feed
    let feed_res = app.clone()
        .oneshot(
            Request::builder()
                .header("Host", "localhost")
                .method("POST")
                .uri("/api/feeds")
                .header("Authorization", format!("Bearer {}", user_a_token))
                .header("Content-Type", "application/json")
                .body(Body::from(json!({
                    "tenant_id": tenant.id,
                    "title": "General Business Feed",
                    "description": "Feed for general updates",
                }).to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    let feed_status = feed_res.status();
    let feed_body = axum::body::to_bytes(feed_res.into_body(), usize::MAX).await.unwrap();
    if feed_status != StatusCode::CREATED {
        println!("POST Error: {:?}", String::from_utf8_lossy(&feed_body));
    }
    assert_eq!(feed_status, StatusCode::CREATED);
    
    let feed_json: serde_json::Value = serde_json::from_slice(&feed_body).unwrap();
    let feed_id = feed_json["id"].as_str().unwrap();

    // 2. Fetch the Feed
    let get_res = app.clone()
        .oneshot(
            Request::builder()
                .header("Host", "localhost")
                .method("GET")
                .uri(format!("/feeds/{}", feed_id))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    let get_status = get_res.status();
    let get_body = axum::body::to_bytes(get_res.into_body(), usize::MAX).await.unwrap();
    println!("GET status: {}, body: {:?}", get_status, String::from_utf8_lossy(&get_body));
    assert_eq!(get_status, StatusCode::OK);
}
