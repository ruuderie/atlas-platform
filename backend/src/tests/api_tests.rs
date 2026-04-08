use axum::{
    body::Body,
    http::{Request, StatusCode},
    Router,
};
use sea_orm::{Database, DatabaseConnection, EntityTrait};
use sea_orm_migration::MigratorTrait;
use tower::ServiceExt;
use serde_json::json;
use uuid::Uuid;
use crate::{api, migration};
use http_body_util::BodyExt;
use super::test_utils;
use crate::models::tenant::TenantModel;
use crate::entities::user_account::UserRole;
use fake::{Fake, faker::{
    company::en::{CompanyName, CatchPhrase},
    internet::en::SafeEmail,
    address::en::{StreetName,CityName, StateAbbr, ZipCode},
    phone_number::en::PhoneNumber,
    name::en::{FirstName, LastName},
}};
use urlencoding;
use dotenv::dotenv;
use crate::handlers::passkeys::{WebauthnStateRaw, WebauthnState};
use webauthn_rs::prelude::*;
use std::sync::Arc;
use moka::future::Cache;
use std::time::Duration;


pub async fn setup_test_app() -> (Router, DatabaseConnection) {
    let opt_local_machine = sea_orm::ConnectOptions::new("postgres://postgres:postgres@localhost:5432/oplydbtest")
        .connect_timeout(std::time::Duration::from_secs(2))
        .to_owned();

    let opt_docker_compose = sea_orm::ConnectOptions::new("postgres://postgres:postgres@localhost:5433/oplydbtest")
        .connect_timeout(std::time::Duration::from_secs(2))
        .to_owned();

    let opt_woodpecker = sea_orm::ConnectOptions::new("postgresql://postgres:postgres@database:5432/oplydbtest")
        .connect_timeout(std::time::Duration::from_secs(2))
        .to_owned();

    let db = match Database::connect(opt_local_machine).await {
        Ok(db) => db,
        Err(_) => match Database::connect(opt_docker_compose).await {
            Ok(db) => db,
            Err(_) => Database::connect(opt_woodpecker)
                .await
                .expect("Failed to connect to test database globally (localhost:5432, :5433, and CI database:5432 all failed). Make sure PostgreSQL is running."),
        },
    };

    // Reset database schema Exactly Once
    test_utils::initialize_database(&db).await;

    let rp_origin = url::Url::parse("http://localhost:5001").unwrap();
    let webauthn = Arc::new(
        WebauthnBuilder::new("localhost", &rp_origin)
            .expect("Invalid WebAuthn config")
            .rp_name("Atlas Platform Test")
            .build()
            .expect("Failed to build Webauthn")
    );
    
    let webauthn_state: WebauthnState = Arc::new(WebauthnStateRaw {
        webauthn,
        reg_state: Cache::builder().time_to_live(Duration::from_secs(300)).build(),
        auth_state: Cache::builder().time_to_live(Duration::from_secs(300)).build(),
    });

    let app = api::create_router(db.clone())
        .layer(axum::Extension(webauthn_state));

    (app, db)
}

#[tokio::test]
async fn test_logout_with_invalid_token() {
    let (app, _) = setup_test_app().await;
    
    // Changed URI to "/api/logout" to match route nesting
    let response = app.clone()
        .oneshot(
            Request::builder().header("Host", "localhost")
                .method("POST")
                .uri("/logout")
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
    dotenv().ok();
    let (app, db) = setup_test_app().await;
    let tenant = test_utils::create_test_tenant(&db).await;
    
    // Register and login
    let mut username = format!("testuser{}", Uuid::new_v4());
    let (_, login_response) = test_utils::register_test_user(&app, tenant.id, &mut username).await;
    let _password = std::env::var("TEST_PASSWORD").unwrap_or_default();
    let token = login_response["token"].as_str().unwrap();
    
    // Test valid session - update the URI to include the /api prefix
    let validation_response = app.clone()
        .oneshot(
            Request::builder().header("Host", "localhost")
                .method("GET")
                .uri("/api/validate-session")  // Changed from "/validate-session" to "/api/validate-session"
                .header("Authorization", format!("Bearer {}", token))
                .body(Body::empty())
                .unwrap()
        )
        .await
        .unwrap();
    assert_eq!(validation_response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_tenant_operations() {
    let (app, db) = setup_test_app().await;
    let (_admin_user, admin_token) = test_utils::create_and_login_admin_user(&app, &db).await;
    
    let test_name = CompanyName().fake::<String>();
    let test_desc = CatchPhrase().fake::<String>();
    
    // Test creating tenant
    let response = app.clone()
        .oneshot(
            Request::builder().header("Host", "localhost")
                .method("POST")
                .uri("/api/tenants")
                .header("Content-Type", "application/json")
                .header("Authorization", format!("Bearer {}", admin_token))
                .body(Body::from(json!({
                    "name": test_name,
                    "description": test_desc,
                }).to_string()))
                .unwrap()
        )
        .await
        .unwrap();
    let status = response.status();
    let body_bytes = response.into_body().collect().await.unwrap().to_bytes();
    let body = String::from_utf8_lossy(&body_bytes).to_string();
    assert_eq!(status, StatusCode::CREATED, "Failed to create tenant: {}", body);
    
    let tenant: TenantModel = serde_json::from_slice(body.as_bytes()).unwrap();
    let tenant_id = tenant.id;

    // Test fetching tenant
    let response = app.clone()
        .oneshot(
            Request::builder().header("Host", "localhost")
                .method("GET")
                .uri(format!("/tenants/{}", tenant_id))
                .body(Body::empty())
                .unwrap()
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    
    // Test updating tenant
    let response = app.clone()
        .oneshot(
            Request::builder().header("Host", "localhost")
                .method("PUT")
                .uri(format!("/api/tenants/{}", tenant_id))
                .header("Content-Type", "application/json")
                .header("Authorization", format!("Bearer {}", admin_token))
                .body(Body::from(json!({
                    "name": format!("Updated Tenant {}", tenant_id),
                    "description": "Updated Description",
                }).to_string()))
                .unwrap()
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    // Test fetching all tenants
    let response = app.clone()
        .oneshot(
            Request::builder().header("Host", "localhost")
                .method("GET")
                .uri("/tenants")
                .body(Body::empty())
                .unwrap()
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    
    // Test deleting tenant
    let response = app.clone()
        .oneshot(
            Request::builder().header("Host", "localhost")
                .method("DELETE")
                .uri(format!("/api/tenants/{}", tenant_id))
                .header("Authorization", format!("Bearer {}", admin_token))
                .body(Body::empty())
                .unwrap()
        )
        .await
        .unwrap();  
    assert_eq!(response.status(), StatusCode::NO_CONTENT);
    
}

#[tokio::test]
async fn test_profile_management() {
    println!("TEST LOG: from test_profile_management");
    let (app, db) = setup_test_app().await;

    let tenant = test_utils::create_test_tenant(&db).await;
    println!("TEST LOG: from test_profile_management and tenant: {:?}", tenant);
    println!("TEST LOG: from test_profile_management and tenant: {:?}", tenant);
    let first_name = FirstName().fake::<String>();
    let last_name = LastName().fake::<String>();
    // Test profile creation
    let mut username = format!("{}_{}", first_name, last_name).to_lowercase();
    let (status, registration_body) = test_utils::register_test_user(&app, tenant.id, &mut username).await;
    assert_eq!(status, StatusCode::CREATED);
    
    let token = registration_body["token"].as_str().unwrap();
    println!("TEST LOG: from test_profile_management and token: {:?}", token);
    // query for profile using get
    let profile_response = app.clone()
        .oneshot(
            Request::builder().header("Host", "localhost")
                .method("GET")
                .uri("/api/profiles")
                .header("Authorization", format!("Bearer {}", token))
                .body(Body::empty())
                .unwrap()
        )
        .await
        .unwrap();
    println!("TEST LOG: from test_profile_management and profile_response: {:?}", profile_response);
    assert_eq!(profile_response.status(), StatusCode::OK);
    //how many profiles are in the response
    let body_bytes = axum::body::to_bytes(profile_response.into_body(), usize::MAX).await.unwrap();
    let profiles: Vec<crate::entities::profile::Model> = serde_json::from_slice(&body_bytes).unwrap();
    println!("TEST LOG: from test_profile_management and profiles: {:?}", profiles.len());
    //print the first profile in the response
    println!("TEST LOG: from test_profile_management and profiles[0]: {:?}", profiles[0]);
    assert!(!profiles.is_empty(), "No profile returned");

    // we use the first profile from the array we already parsed
    let profile = &profiles[0];
    println!("TEST LOG: from test_profile_management and profile: {:?}", profile);

    let display_name = CompanyName().fake::<String>();
    // Test profile update
    let update_response = app.clone()
        .oneshot(
            Request::builder().header("Host", "localhost")
                .method("PUT")
                .uri(format!("/api/profiles/{}", profile.id))
                .header("Authorization", format!("Bearer {}", token))
                .header("Content-Type", "application/json")
                .body(Body::from(json!({
                    "display_name": display_name,
                    "contact_info": SafeEmail().fake::<String>(),
                    "business_description": CatchPhrase().fake::<String>(),
                    "address": {
                        "street": StreetName().fake::<String>(),
                        "city": CityName().fake::<String>(),
                        "state": StateAbbr().fake::<String>(),
                        "zip": ZipCode().fake::<String>()
                    }
                }).to_string()))
                .unwrap()
        )
        .await
        .unwrap();
    assert_eq!(update_response.status(), StatusCode::OK);
    
    // Test profile search
    let search_response = app.clone()
        .oneshot(
            Request::builder().header("Host", "localhost")
                .method("GET")
                .uri(format!("/api/profiles/search?q={}", urlencoding::encode(&display_name)).as_str())
                .header("Authorization", format!("Bearer {}", token))
                .body(Body::empty())
                .unwrap()
        )
        .await
        .unwrap();
    assert_eq!(search_response.status(), StatusCode::OK);
    let body_bytes = axum::body::to_bytes(search_response.into_body(), usize::MAX).await.unwrap();
    let profiles: Vec<crate::entities::profile::Model> = serde_json::from_slice(&body_bytes).unwrap();
    assert!(!profiles.is_empty(), "No profiles returned");
    assert_eq!(profiles[0].display_name, display_name);
}

#[tokio::test]
async fn test_category_management() {
    let (app, db) = setup_test_app().await;
    let (_admin_user, admin_token) = test_utils::create_and_login_admin_user(&app, &db).await;
    
    // Create a tenant type first (needed for category)
    let tenant = test_utils::create_test_tenant(&db).await;
    
    // Test category creation
    let response = app.clone()
        .oneshot(
            Request::builder().header("Host", "localhost")
                .method("POST")
                .uri("/api/admin/categories")
                .header("Content-Type", "application/json")
                .header("Authorization", format!("Bearer {}", admin_token))
                .body(Body::from(json!({
                    "name": "Test Category",
                    "description": "Test Category Description",
                    "tenant_id": tenant.id,
                    "parent_category_id": null,
                    "is_custom": false,
                    "is_active": true
                }).to_string()))
                .unwrap()
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::CREATED);
    
    // Get the created category ID from the response
    let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let category: crate::entities::category::Model = serde_json::from_slice(&body_bytes).unwrap();
    
    // Test fetching the category
    let get_response = app.clone()
        .oneshot(
            Request::builder().header("Host", "localhost")
                .method("GET")
                .uri(format!("/api/admin/categories/{}", category.id))
                .header("Authorization", format!("Bearer {}", admin_token))
                .body(Body::empty())
                .unwrap()
        )
        .await
        .unwrap();
    assert_eq!(get_response.status(), StatusCode::OK);
    
    // Test updating the category
    let update_response = app.clone()
        .oneshot(
            Request::builder().header("Host", "localhost")
                .method("PUT")
                .uri(format!("/api/admin/categories/{}", category.id))
                .header("Content-Type", "application/json")
                .header("Authorization", format!("Bearer {}", admin_token))
                .body(Body::from(json!({
                    "name": "Updated Category",
                    "description": "Updated Description",
                    "tenant_id": tenant.id,
                    "parent_category_id": null,
                    "is_custom": true,
                    "is_active": true
                }).to_string()))
                .unwrap()
        )
        .await
        .unwrap();
    assert_eq!(update_response.status(), StatusCode::OK);

    //query for the category
    let get_response = app.clone()
        .oneshot(
            Request::builder().header("Host", "localhost")
                .method("GET")
                .uri(format!("/api/admin/categories/{}", category.id))
                .header("Authorization", format!("Bearer {}", admin_token))
                .body(Body::empty())
                .unwrap()
        )
        .await
        .unwrap();
    assert_eq!(get_response.status(), StatusCode::OK);
    let body_bytes = axum::body::to_bytes(get_response.into_body(), usize::MAX).await.unwrap();
    let category: crate::entities::category::Model = serde_json::from_slice(&body_bytes).unwrap();
    assert_eq!(category.name, "Updated Category");
    
    // Test deleting the category
    let delete_response = app.clone()
        .oneshot(
            Request::builder().header("Host", "localhost")
                .method("DELETE")
                .uri(format!("/api/admin/categories/{}", category.id))
                .header("Authorization", format!("Bearer {}", admin_token))
                .body(Body::empty())
                .unwrap()
        )
        .await
        .unwrap();
    assert_eq!(delete_response.status(), StatusCode::NO_CONTENT);
}

#[tokio::test]
async fn test_profile_operations() {
    let (app, db) = setup_test_app().await;
    let tenant = test_utils::create_test_tenant(&db).await;
    
    // Register a user and get their profile
    let mut username = format!("testuser{}", Uuid::new_v4());
    let (status, login_response) = test_utils::register_test_user(&app, tenant.id, &mut username).await;
    assert_eq!(status, StatusCode::CREATED);
    
    let token = login_response["token"].as_str().unwrap();
    println!("TEST LOG: from test_profile_operations and token: {:?}", token);
    // Test getting profiles
    let profiles_response = app.clone()
        .oneshot(
            Request::builder().header("Host", "localhost")
                .method("GET")
                .uri("/api/profiles")
                .header("Authorization", format!("Bearer {}", token))
                .body(Body::empty())
                .unwrap()
        )
        .await
        .unwrap();
    
    assert_eq!(profiles_response.status(), StatusCode::OK);
    
    let profiles: Vec<serde_json::Value> = serde_json::from_slice(
        &axum::body::to_bytes(profiles_response.into_body(), usize::MAX).await.unwrap()
    ).unwrap();
    //how many profiles are in the response
    println!("TEST LOG: from test_profile_operations and profiles: {:?}", profiles.len());
    //print the first profile in the response
    println!("TEST LOG: from test_profile_operations and profiles[0]: {:?}", profiles[0]);
    
    assert_eq!(profiles.len(), 1, "User should have exactly one profile");
    assert_eq!(
        profiles[0]["display_name"].as_str().unwrap(),
        format!("{}'s Business", username),
        "Profile display name should match"
    );
}

#[tokio::test]
async fn test_concurrent_logout_requests() {
    // Set up the test environment
    let (app, db) = setup_test_app().await;

    // Create a test user and get their token
    let mut username = format!("concurrentuser{}", Uuid::new_v4());
    let tenant = test_utils::create_test_tenant(&db).await;
    
    // Register and login the user
    let (status, login_response) = test_utils::register_test_user(&app, tenant.id, &mut username).await;
    assert_eq!(status, StatusCode::CREATED);
    
    // Get the login token
    let token = login_response["token"].as_str().unwrap().to_string();
    println!("TEST LOG: from test_concurrent_logout_requests and token: {:?}", token);
    
    // Create multiple concurrent logout requests
    let mut handles = Vec::new();
    for _ in 0..5 {
        let app_clone = app.clone();
        let token_clone = token.clone();
        let handle = tokio::spawn(async move {
            let response = app_clone
                .oneshot(
                    Request::builder().header("Host", "localhost")
                        .method("POST")
                        .uri("/logout")
                        .header("Authorization", format!("Bearer {}", token_clone))
                        .body(Body::empty())
                        .unwrap(),
                )
                .await
                .unwrap();
            
            println!("Logout response: {:?}", response);
            response
        });
        handles.push(handle);
    }
    
    // Wait for all requests to complete
    let mut responses = Vec::new();
    for handle in handles {
        responses.push(handle.await.unwrap());
    }
    
    // The first request should succeed, others should fail with 401 Unauthorized
    // because the session is deactivated after the first logout
    let mut success_count = 0;
    let mut unauthorized_count = 0;
    
    for response in responses {
        println!("response from test_concurrent_logout_requests: {:?}", response);
        match response.status() {
            StatusCode::OK => success_count += 1,
            StatusCode::UNAUTHORIZED => unauthorized_count += 1,
            status => panic!("Unexpected status code: {}", status),
        }
    }
    
    // At least one request should succeed
    assert!(success_count > 0, "No logout requests succeeded");
    // The rest should fail with 401 Unauthorized
    assert_eq!(success_count + unauthorized_count, 5, "Not all requests returned expected status codes");
}

#[tokio::test]
async fn test_listing_crud_operations() {
    let (app, db) = setup_test_app().await;
    
    // Create test tenant type and tenant
    let tenant = test_utils::create_test_tenant(&db).await;
    
    // Create a default category for listings
    let default_category = test_utils::create_default_category(&db, tenant.id).await;
    
    // Register a user and get their profile
    let mut username = format!("testuser{}", Uuid::new_v4());
    let (status, login_response) = test_utils::register_test_user(&app, tenant.id, &mut username).await;
    assert_eq!(status, StatusCode::CREATED);
    
    let token = login_response["token"].as_str().unwrap();
    
    // Get user's profile
    let profiles_response = app.clone()
        .oneshot(
            Request::builder().header("Host", "localhost")
                .method("GET")
                .uri("/api/profiles")
                .header("Authorization", format!("Bearer {}", token))
                .body(Body::empty())
                .unwrap()
        )
        .await
        .unwrap();
    
    // Get the first profile from the array
    let profiles: Vec<crate::entities::profile::Model> = serde_json::from_slice(
        &axum::body::to_bytes(profiles_response.into_body(), usize::MAX).await.unwrap()
    ).unwrap();
    let profile = profiles.first().expect("No profile found");
    
    // Test creating a listing
    let listing_title = fake::faker::company::en::CompanyName().fake::<String>();
    let listing_description = fake::faker::lorem::en::Sentence(5..10).fake::<String>();
    
    let create_listing_response = app.clone()
        .oneshot(
            Request::builder().header("Host", "localhost")
                .method("POST")
                .uri("/api/listings")
                .header("Authorization", format!("Bearer {}", token))
                .header("Content-Type", "application/json")
                .body(Body::from(json!({
                    "title": listing_title.clone(),
                    "description": listing_description.clone(),
                    "tenant_id": tenant.id,
                    "profile_id": profile.id,
                    "category_id": default_category.id,
                    "status": "active"
                }).to_string()))
                .unwrap()
        )
        .await
        .unwrap();
    
    assert_eq!(create_listing_response.status(), StatusCode::CREATED);
    
    // Parse the created listing
    let body_bytes = axum::body::to_bytes(create_listing_response.into_body(), usize::MAX).await.unwrap();
    let created_listing: crate::entities::listing::Model = serde_json::from_slice(&body_bytes).unwrap();
    
    // Test getting a specific listing
    let get_listing_response = app.clone()
        .oneshot(
            Request::builder().header("Host", "localhost")
                .method("GET")
                .uri(format!("/listings/{}", created_listing.id))
                //.header("Authorization", format!("Bearer {}", token))
                .body(Body::empty())
                .unwrap()
        )
        .await
        .unwrap();
    
    assert_eq!(get_listing_response.status(), StatusCode::OK);
    
    // Test updating the listing
    let updated_title = fake::faker::company::en::CompanyName().fake::<String>();
    let update_listing_response = app.clone()
        .oneshot(
            Request::builder().header("Host", "localhost")
                .method("PUT")
                .uri(format!("/api/listings/{}", created_listing.id))
                .header("Authorization", format!("Bearer {}", token))
                .header("Content-Type", "application/json")
                .body(Body::from(json!({
                    "title": updated_title.clone(),
                    "description": listing_description,
                    "tenant_id": tenant.id,
                    "profile_id": profile.id,
                    "category_id": default_category.id,
                    "status": "active"
                }).to_string()))
                .unwrap()
        )
        .await
        .unwrap();
    
    assert_eq!(update_listing_response.status(), StatusCode::OK);
    
    // Verify the listing was updated
    let body_bytes = axum::body::to_bytes(update_listing_response.into_body(), usize::MAX).await.unwrap();
    let updated_listing: crate::entities::listing::Model = serde_json::from_slice(&body_bytes).unwrap();
    assert_eq!(updated_listing.title, updated_title);
    
    // Test getting all listings for a tenant
    let tenant_listings_response = app.clone()
        .oneshot(
            Request::builder().header("Host", "localhost")
                .method("GET")
                .uri(format!("/listings?tenant_id={}", tenant.id))
                .body(Body::empty())
                .unwrap()
        )
        .await
        .unwrap();
    
    assert_eq!(tenant_listings_response.status(), StatusCode::OK);
    
    // Verify response contains listings
    let body_bytes = axum::body::to_bytes(tenant_listings_response.into_body(), usize::MAX).await.unwrap();
    let tenant_listings: Vec<crate::entities::listing::Model> = serde_json::from_slice(&body_bytes).unwrap();
    assert!(!tenant_listings.is_empty(), "No listings returned for tenant");
    
    // Test deleting the listing
    let delete_listing_response = app.clone()
        .oneshot(
            Request::builder().header("Host", "localhost")
                .method("DELETE")
                .uri(format!("/api/listings/{}", created_listing.id))
                .header("Authorization", format!("Bearer {}", token))
                .body(Body::empty())
                .unwrap()
        )
        .await
        .unwrap();
    
    assert_eq!(delete_listing_response.status(), StatusCode::NO_CONTENT);
    
    // Verify the listing was deleted
    let get_deleted_listing_response = app.clone()
        .oneshot(
            Request::builder().header("Host", "localhost")
                .method("GET")
                .uri(format!("/api/listings/{}", created_listing.id))
                .header("Authorization", format!("Bearer {}", token))
                .body(Body::empty())
                .unwrap()
        )
        .await
        .unwrap();
    
    assert_eq!(get_deleted_listing_response.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_listing_operations() {
    let (app, db) = setup_test_app().await;
    let _tenant = test_utils::create_test_tenant(&db).await;
    let tenant = test_utils::create_test_tenant(&db).await;

    let mut username = format!("testuser{}", Uuid::new_v4());
    println!("TEST LOG: from test_listing_operations and username: {:?}", username);
    let (status, registration_body) = test_utils::register_test_user(&app, tenant.id, &mut username).await;
    assert_eq!(status, StatusCode::CREATED);
    println!("TEST LOG: from test_listing_operations and registration_body: {:?}", registration_body);
    
    // Get the created profile from the registration
    let token = registration_body["token"].as_str().unwrap();
    let profile_response = app.clone()
        .oneshot(
            Request::builder().header("Host", "localhost")
                .method("GET")
                .uri("/api/profiles")
                .header("Authorization", format!("Bearer {}", token))
                .body(Body::empty())
                .unwrap()
        )
        .await
        .unwrap();
    
    // Get the first profile from the array
    let profiles: Vec<crate::entities::profile::Model> = serde_json::from_slice(
        &axum::body::to_bytes(profile_response.into_body(), usize::MAX).await.unwrap()
    ).unwrap();
    let profile = profiles.first().expect("No profile found");

    // In your test setup:
    let default_category = test_utils::create_default_category(&db, tenant.id).await;

    // Test creating a listing with full business context
    let create_listing_response = app.clone()
        .oneshot(
            Request::builder().header("Host", "localhost")
                .method("POST")
                .uri("/api/listings")
                .header("Authorization", format!("Bearer {}", token))
                .header("Content-Type", "application/json")
                .body(Body::from(json!({
                    "title": CompanyName().fake::<String>(),
                    "description": CatchPhrase().fake::<String>(),
                    "tenant_id": tenant.id,
                    "profile_id": profile.id,
                    "category_id": default_category.id,
                    "business_details": {
                        "opening_hours": "9AM-5PM",
                        "address": {
                            "street": StreetName().fake::<String>(),
                            "city": CityName().fake::<String>(),
                            "state": StateAbbr().fake::<String>(),
                            "zip": ZipCode().fake::<String>()
                        },
                        "contact": {
                            "phone": PhoneNumber().fake::<String>(),
                            "email": SafeEmail().fake::<String>()
                        },
                        "services": ["Consulting", "Development", "Design"]
                    }
                }).to_string()))
                .unwrap()
        )
        .await
        .unwrap();
    assert_eq!(create_listing_response.status(), StatusCode::CREATED);
    
    // Parse the created listing
    let body_bytes = axum::body::to_bytes(create_listing_response.into_body(), usize::MAX).await.unwrap();
    let created_listing: crate::entities::listing::Model = serde_json::from_slice(&body_bytes).unwrap();
    println!("TEST LOG: Created listing: {:?}", created_listing);
    

    
    // Continue with staff user access testing
    let staff_user = test_utils::create_staff_user_account(
        &db,
        &test_utils::create_and_login_admin_user(&app, &db).await.0,
        &profile,  // Now passing the actual Model
        UserRole::Member
    ).await;
    
    // Get the full user record from the user_account
    let user = crate::entities::user::Entity::find_by_id(staff_user.user_id)
        .one(&db)
        .await
        .unwrap()
        .unwrap();

    let staff_login = test_utils::login_test_user(
        &app,
        &user.email,  // Use email from user entity
        "staffpass123"
    ).await;
    let staff_token = staff_login["token"].as_str().unwrap();
    
    // Verify staff can access the listing
    let listings_response = app.clone()
        .oneshot(
            Request::builder().header("Host", "localhost")
                .method("GET")
                .uri(format!("/listings?tenant_id={}", tenant.id).as_str())
                .header("Authorization", format!("Bearer {}", staff_token))
                .body(Body::empty())
                .unwrap()
        )
        .await
        .unwrap();
    assert_eq!(listings_response.status(), StatusCode::OK);
    
    // Verify response contains listings
    let body_bytes = axum::body::to_bytes(listings_response.into_body(), usize::MAX).await.unwrap();
    let listings: Vec<crate::entities::listing::Model> = serde_json::from_slice(&body_bytes).unwrap();
    assert!(!listings.is_empty(), "No listings returned");



}

#[tokio::test]
async fn test_inline_passkey_registration() {
    let (app, db) = setup_test_app().await;

    // Create a valid tenant directly via db handle
    let _tenant = test_utils::create_test_tenant(&db).await;
    let tenant = test_utils::create_test_tenant(&db).await;

    // Call test registration which intrinsically wraps the new auto-authenticating users::register endpoint
    let mut username = String::new();
    let (status, json_body) = test_utils::register_test_user(&app, tenant.id, &mut username).await;

    // Verify successful registration
    assert_eq!(status, StatusCode::CREATED, "Registration failed: {:?}", json_body);

    let token = json_body["token"]
        .as_str()
        .expect("Generated token should be returned cleanly directly from the register endpoints JSON struct")
        .to_string();

    // Now securely call the passkey initialization challenge endpoint mimicking the immediate frontend inline passkey view
    let webauthn_req = app.clone()
        .oneshot(
            Request::builder().header("Host", "localhost")
                .method("POST")
                .uri("/api/passkeys/start-register")
                .header("Authorization", format!("Bearer {}", token))
                .header("Content-Type", "application/json")
                .body(Body::empty())
                .unwrap()
        ).await.unwrap();

    // In a pristine WebAuthn instance, this challenge request should always yield 200 OK 
    assert_eq!(webauthn_req.status(), StatusCode::OK, "Passkey challenge rejected with status {}", webauthn_req.status());
}
