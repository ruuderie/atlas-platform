use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use http_body_util::BodyExt;
use tower::ServiceExt;
use serde_json::json;
use crate::{api, tests::test_utils};
use uuid::Uuid;

use crate::tests::api_tests::setup_test_app;
use crate::entities::{app_instance, app_domain};
use sea_orm::{ActiveModelTrait, Set};

#[tokio::test]
async fn test_anchor_pages_crud() {
    let (app, db) = setup_test_app().await;

    // 1. Setup tenant & user
    let tenant = test_utils::create_test_tenant(&db).await;

    // Create AppInstance for Anchor
    let app_inst = app_instance::ActiveModel {
        id: Set(Uuid::new_v4()),
        tenant_id: Set(tenant.id),
        app_type: Set("anchor".to_string()),
        created_at: Set(chrono::Utc::now()),
        updated_at: Set(chrono::Utc::now()),
        ..Default::default()
    }.insert(&db).await.unwrap();

    // Create AppDomain
    app_domain::ActiveModel {
        id: Set(Uuid::new_v4()),
        app_instance_id: Set(app_inst.id),
        domain_name: Set("test-anchor.local".to_string()),
        created_at: Set(chrono::Utc::now()),
        ..Default::default()
    }.insert(&db).await.unwrap();

    let mut username = "anchortest".to_string();
    let (status, login_response) = test_utils::register_test_user(&app, tenant.id, &mut username).await;
    assert_eq!(status, StatusCode::CREATED);
    
    let token = login_response["token"].as_str().unwrap().to_string();


    let create_payload = json!({
        "title": "About Us",
        "slug": "about-us",
        "page_type": "Landing",
        "is_published": false,
        "hero_payload": { "heading": "Welcome" },
        "blocks_payload": { "blocks": [] }
    });

    let create_req = Request::builder()
        .method("POST")
        .uri("/api/anchor/pages")
        .header("Host", "test-anchor.local")
        .header("Authorization", format!("Bearer {}", token))
        .header("Content-Type", "application/json")
        .body(Body::from(create_payload.to_string()))
        .unwrap();

    let create_resp = app.clone().oneshot(create_req).await.unwrap();
    assert_eq!(create_resp.status(), StatusCode::CREATED);
    
    let body_bytes = create_resp.into_body().collect().await.unwrap().to_bytes();
    let created_page: serde_json::Value = serde_json::from_slice(&body_bytes).unwrap();
    let page_id = created_page["id"].as_str().unwrap();

    // 3. List Pages
    let list_req = Request::builder()
        .method("GET")
        .uri("/api/anchor/pages")
        .header("Host", "test-anchor.local")
        .header("Authorization", format!("Bearer {}", token))
        .body(Body::empty())
        .unwrap();

    let list_resp = app.clone().oneshot(list_req).await.unwrap();
    assert_eq!(list_resp.status(), StatusCode::OK);
    
    let body_bytes = list_resp.into_body().collect().await.unwrap().to_bytes();
    let pages: Vec<serde_json::Value> = serde_json::from_slice(&body_bytes).unwrap();
    assert_eq!(pages.len(), 1);
    assert_eq!(pages[0]["slug"], "about-us");

    // 4. Update Page
    let update_payload = json!({
        "is_published": true,
        "title": "About Us Updated"
    });

    let update_req = Request::builder()
        .method("PUT")
        .uri(&format!("/api/anchor/pages/{}", page_id))
        .header("Host", "test-anchor.local")
        .header("Authorization", format!("Bearer {}", token))
        .header("Content-Type", "application/json")
        .body(Body::from(update_payload.to_string()))
        .unwrap();

    let update_resp = app.clone().oneshot(update_req).await.unwrap();
    assert_eq!(update_resp.status(), StatusCode::OK);

    // 5. Delete Page
    let delete_req = Request::builder()
        .method("DELETE")
        .uri(&format!("/api/anchor/pages/{}", page_id))
        .header("Host", "test-anchor.local")
        .header("Authorization", format!("Bearer {}", token))
        .body(Body::empty())
        .unwrap();

    let delete_resp = app.clone().oneshot(delete_req).await.unwrap();
    assert_eq!(delete_resp.status(), StatusCode::NO_CONTENT);
}
