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


use crate::entities::{directory_type, directory, user, profile, user_account, category};

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
) -> (StatusCode, serde_json::Value) {
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
    println!("TEST LOG: Registration response status: {:?}, body: {:?}", status, body);
    let json_body: serde_json::Value = serde_json::from_str(&body).unwrap_or_default();
    println!("TEST LOG: Registration response json_body: {:?}", json_body);
    let login_response = login_test_user(
        app,
        &format!("{}@example.com", username),
        "password123"
    ).await;
    println!("TEST LOG: Login response: {:?}", login_response);
    // After registration, create associated business profile
    if status.is_success() {
        // Login to get a valid token
        
        let token = &login_response["token"].as_str().expect("No token in login response");
        
        // Create a business profile for the new user
        let profile_response = app.clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/profiles")
                    .header("Content-Type", "application/json")
                    .header("Authorization", format!("Bearer {}", token.to_string()))
                    .body(Body::from(
                        json!({
                            "directory_id": directory_id,
                            "profile_type": "Business",
                            "display_name": format!("{}'s Business", username),
                            "contact_info": format!("contact@{}s-business.com", username),
                            "business_details": {
                                "business_name": format!("{} Enterprises", username),
                                "business_address": "123 Main St, Test City",
                                "business_phone": "555-1234",
                                "website": format!("https://{}-business.com", username)
                            }
                        }).to_string(),
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();
        
        println!("TEST LOG: Profile creation response: {:?}", profile_response);
    }

    (status, login_response)
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
    println!("TEST LOG: from login_test_user status and response: {:?}", response);
    let body_bytes = response.into_body()
        .collect()
        .await
        .unwrap()
        .to_bytes();
    let body = String::from_utf8_lossy(&body_bytes).to_string();
    println!("TEST LOG: from login_test_user and status: {:?} , body: {:?}", status, body);
    if status != StatusCode::OK {
        panic!("Login failed with status {}: {}", status, body);
    }
    
    serde_json::from_str(&body).unwrap_or_else(|e| {
        panic!("Failed to parse login response as JSON. Error: {}. Response body: {}", e, body)
    })
}

pub async fn create_and_login_admin_user(app: &Router, db: &DatabaseConnection) -> (user::Model, String) {
    // Create admin user directly in the database
    let admin_user = user::ActiveModel {
        id: Set(Uuid::new_v4()),
        username: Set(format!("admin{}", Uuid::new_v4())),
        first_name: Set("Admin".to_string()),
        last_name: Set("User".to_string()),
        email: Set(format!("admin{}@example.com", Uuid::new_v4())),
        phone: Set("1234567890".to_string()),
        password_hash: Set(crate::auth::hash_password("password123").unwrap()),
        is_admin: Set(true),
        is_active: Set(true),
        last_login: Set(Some(Utc::now())),
        created_at: Set(Utc::now()),
        updated_at: Set(Utc::now()),
    }.insert(db).await.expect("Failed to create admin user");

    // Login the admin user
    let login_response = login_test_user(
        app,
        &admin_user.email,
        "password123"
    ).await;

    let token = login_response["token"].as_str().unwrap().to_string();
    
    (admin_user, token)
}

// New function to create staff user accounts
pub async fn create_staff_user_account(
    db: &DatabaseConnection,
    admin_user: &user::Model,
    profile: &profile::Model,
    role: user_account::UserRole,
) -> user_account::Model {
    // Create staff user
    let staff_user = user::ActiveModel {
        id: Set(Uuid::new_v4()),
        username: Set(format!("staff-{}", Uuid::new_v4())),
        first_name: Set("Staff".to_string()),
        last_name: Set("User".to_string()),
        email: Set(format!("staff-{}@example.com", Uuid::new_v4())),
        phone: Set("555-9876".to_string()),
        password_hash: Set(crate::auth::hash_password("staffpass123").unwrap()),
        is_admin: Set(false),
        is_active: Set(true),
        last_login: Set(Some(Utc::now())),
        created_at: Set(Utc::now()),
        updated_at: Set(Utc::now()),
    }.insert(db).await.expect("Failed to create staff user");

    // Create user account association
    user_account::ActiveModel {
        id: Set(Uuid::new_v4()),
        user_id: Set(staff_user.id),
        account_id: Set(profile.account_id),
        role: Set(role),
        is_active: Set(true),
        created_at: Set(Utc::now()),
        updated_at: Set(Utc::now()),
    }.insert(db).await.expect("Failed to create staff user account")
}

pub async fn create_default_category(db: &DatabaseConnection, directory_type_id: Uuid) -> category::Model {
    category::ActiveModel {
        id: Set(Uuid::new_v4()),
        name: Set("Default Category".to_string()),
        description: Set("Default category for listings".to_string()),
        directory_type_id: Set(directory_type_id),
        parent_category_id: Set(None),
        is_custom: Set(false),
        is_active: Set(true),
        created_at: Set(Utc::now()),
        updated_at: Set(Utc::now()),
    }
    .insert(db)
    .await
    .expect("Failed to create default category")
}