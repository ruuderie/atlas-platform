use http_body_util::BodyExt; // Brings collect() into scope
use axum::{
    body::Body,
    http::{Request, StatusCode},
    Router,
};
use chrono::Utc;
use fake::{
    faker::{
        address::en::StreetName,
        company::en::CompanyName,
        internet::en::{DomainSuffix, SafeEmail, Username},
        lorem::en::Sentence,
        name::en::{FirstName, LastName},
        phone_number::en::PhoneNumber,
    },
    Fake,
};
use sea_orm::{ActiveModelTrait, ConnectionTrait, DatabaseConnection, Set};
use serde_json::json;
use tower::ServiceExt;
use uuid::Uuid;

use dotenv::dotenv;
use crate::entities::{category, tenant, profile, user, user_account};
use std::sync::{Mutex, OnceLock};

/// Global mutex: serializes DROP SCHEMA + Migrator::up() across all test threads.
///
/// Must be std::sync::Mutex — each #[tokio::test] spawns its OWN independent
/// tokio runtime. tokio::sync::Mutex is only visible within a single runtime;
/// across runtimes it provides zero synchronization, so every thread races past
/// it and independently drops the schema mid-migration.
static DB_INIT_LOCK: Mutex<()> = Mutex::new(());

/// Tracks whether migration has been applied successfully this process.
/// std::sync::OnceLock — same cross-runtime reason as above.
pub static DB_INIT: OnceLock<()> = OnceLock::new();

/// Refreshes the test database schema exactly once per test process.
///
/// Uses double-checked locking with OS-level primitives:
///   1. Fast path: OnceLock already set → return immediately.
///   2. Slow path: spawn_blocking thread → acquire std::Mutex → re-check →
///      drop+migrate via the test runtime's Handle → set OnceLock.
///
/// Why spawn_blocking and not block_in_place?
///   block_in_place requires the multi-thread scheduler; #[tokio::test] defaults
///   to single-thread. spawn_blocking works on both and offloads the blocking
///   std::Mutex::lock() call to tokio's dedicated blocking thread pool.
///
/// Why std::sync::Mutex and OnceLock?
///   Each #[tokio::test] creates its own independent tokio runtime.
///   tokio::sync primitives are scoped to a single runtime — they provide zero
///   cross-runtime synchronization, so every test thread would race past them.
///   std primitives are OS-level and work across all threads/runtimes.
pub async fn initialize_database(db: &DatabaseConnection) {
    // Fast path: already done.
    if DB_INIT.get().is_some() {
        return;
    }

    let db = db.clone();
    // Capture the current runtime handle BEFORE entering spawn_blocking,
    // so we can drive async futures from inside the blocking thread.
    let handle = tokio::runtime::Handle::current();

    tokio::task::spawn_blocking(move || {
        // Acquire OS-level mutex — blocking call, safe on a blocking thread.
        // unwrap_or_else recovers a poisoned lock so retries work after a panic.
        let _guard = DB_INIT_LOCK
            .lock()
            .unwrap_or_else(|e| e.into_inner());

        // Re-check after acquiring lock — another thread may have just finished.
        if DB_INIT.get().is_some() {
            return;
        }

        // Drive the migration future to completion using the test's runtime.
        handle.block_on(async move {
            use sea_orm_migration::MigratorTrait;
            use sea_orm::{ConnectionTrait, Statement};

            let drop_res = db.execute(Statement::from_string(
                db.get_database_backend(),
                "DROP SCHEMA public CASCADE;".to_owned(),
            )).await;
            let create_res = db.execute(Statement::from_string(
                db.get_database_backend(),
                "CREATE SCHEMA public;".to_owned(),
            )).await;
            println!(
                "TEST LOG: Public schema drop result: {:?}, create result: {:?}",
                drop_res, create_res
            );

            if let Err(e) = crate::migration::Migrator::up(&db, None).await {
                panic!("Failed to run migrations: {e}");
            }
        });

        // Mark done — all subsequent callers fast-path out.
        let _ = DB_INIT.set(());
    })
    .await
    .expect("initialize_database: spawn_blocking panicked");
}



pub async fn create_test_tenant<C: ConnectionTrait>(db: &C) -> tenant::Model {
    let tenant_id = Uuid::new_v4();

    let company_domain = format!("{:?}.com", CompanyName().fake::<String>());
    let _company_domain = company_domain.replace(" ", "").replace("\"", "").to_lowercase();
    let new_tenant = tenant::ActiveModel {
        id: Set(tenant_id),
        name: Set(CompanyName().fake()),
        description: Set(Sentence(1..3).fake()),
        created_at: Set(Utc::now()),
        updated_at: Set(Utc::now()),
        ..Default::default()
    };

    new_tenant
        .insert(db)
        .await
        .expect("Failed to create test tenant")
}

pub async fn register_test_user(
    app: &Router,
    tenant_id: Uuid,
    username: &mut String,
) -> (StatusCode, serde_json::Value) {
    dotenv().ok();
    if username.is_empty() {
        *username = format!("user_{}", uuid::Uuid::new_v4()).replace("-", "").to_lowercase();
    } else {
        *username = username.replace(" ", "_").to_lowercase();
    }
    let first_name: String = username.split("_").next().unwrap_or_default().to_string();
    let last_name: String = username.split("_").nth(1).unwrap_or_default().to_string();

    let domain_suffix = DomainSuffix().fake::<String>();
    let password: String = std::env::var("TEST_PASSWORD").unwrap_or_default();
    let phone: String = PhoneNumber().fake();
    let company_name = CompanyName().fake::<String>();
    let company_domain = format!("{:?}.{:?}", company_name, domain_suffix).replace(" ", "").replace("\"", "").to_lowercase();
    let email = format!("{:?}@{:?}", username, company_domain).replace("\"", "").to_lowercase();

    let response: axum::http::Response<axum::body::Body> = app
        .clone()
        .oneshot(
            Request::builder().header("Host", "localhost")
                .method("POST")
                .uri("/register")
                .header("Content-Type", "application/json")
                .body(Body::from(
                    json!({
                        "tenant_id": tenant_id,
                        "username": username,
                        "first_name": first_name,
                        "last_name": last_name,
                        "email": email,
                        "password": password,
                        "phone": phone
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    let status = response.status();
    let mut token_from_cookie = None;
    for cookie in response.headers().get_all(axum::http::header::SET_COOKIE).iter() {
        if let Ok(c) = cookie.to_str() {
            if c.starts_with("session=") {
                token_from_cookie = Some(c.split(';').next().unwrap().strip_prefix("session=").unwrap().to_string());
            }
        }
    }
    let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let body = String::from_utf8_lossy(&body_bytes).to_string();
    println!("TEST LOG: Register response status: {}, body: {}", status, body);
    let mut json_body: serde_json::Value = serde_json::from_str(&body).unwrap_or_default();
    if let Some(token) = token_from_cookie {
        json_body["token"] = serde_json::json!(token);
    }

    // If registration successful, get token directly and update the profile
    if status.is_success() {
        let token = json_body["token"]
            .as_str()
            .expect("No token in register response");

        // First get the profiles to find the one we want to update
        let profiles_response: axum::http::Response<axum::body::Body> = app
            .clone()
            .oneshot(
                Request::builder().header("Host", "localhost")
                    .method("GET")
                    .uri("/api/profiles")
                    .header("Authorization", format!("Bearer {}", token))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        let status = profiles_response.status();
        let body_bytes = profiles_response.into_body().collect().await.unwrap().to_bytes();
        if !status.is_success() {
            panic!("GET /api/profiles failed with status: {}, body: {:?}", status, String::from_utf8_lossy(&body_bytes));
        }

        let profiles: Vec<serde_json::Value> = serde_json::from_slice(&body_bytes).unwrap();

        let profile = profiles.first().expect("No profile found");
        let profile_id = profile["id"].as_str().expect("No profile ID found");
        let company_name = CompanyName().fake::<String>();
        // Update the profile with business details
        let update_response: axum::http::Response<axum::body::Body> = app
            .clone()
            .oneshot(
                Request::builder().header("Host", "localhost")
                    .method("PUT")
                    .uri(format!("/api/profiles/{}", profile_id))
                    .header("Content-Type", "application/json")
                    .header("Authorization", format!("Bearer {}", token))
                    .body(Body::from(
                        json!({
                            "display_name": format!("{}'s Business", username),
                            "contact_info": format!("contact@{}s-business.com", username),
                            "business_details": {
                                "business_name": &company_name.as_str().to_string(),
                                "business_address": StreetName().fake::<String>(),
                                "business_phone": PhoneNumber().fake::<String>(),
                                "website": format!("https://{}.com", username)
                            }
                        })
                        .to_string(),
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();

        println!("TEST LOG: Profile update response: {:?}", update_response);
        assert_eq!(
            update_response.status(),
            StatusCode::OK,
            "Failed to update profile"
        );
    }

    (status, json_body)
}

pub async fn login_test_user(app: &Router, email: &str, password: &str) -> serde_json::Value {
    let response: axum::http::Response<axum::body::Body> = app
        .clone()
        .oneshot(
            Request::builder().header("Host", "localhost")
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
    println!(
        "TEST LOG: from login_test_user status and response: {:?}",
        response
    );
    let mut token_from_cookie = None;
    for cookie in response.headers().get_all(axum::http::header::SET_COOKIE).iter() {
        if let Ok(c) = cookie.to_str() {
            if c.starts_with("session=") {
                token_from_cookie = Some(c.split(';').next().unwrap().strip_prefix("session=").unwrap().to_string());
            }
        }
    }
    let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let body = String::from_utf8_lossy(&body_bytes).to_string();
    println!(
        "TEST LOG: from login_test_user and status: {:?} , body: {:?}",
        status, body
    );
    if status != StatusCode::OK {
        panic!("Login failed with status {}: {}", status, body);
    }

    let mut json_body: serde_json::Value = serde_json::from_str(&body).unwrap_or_else(|e| {
        panic!(
            "Failed to parse login response as JSON. Error: {}. Response body: {}",
            e, body
        )
    });
    if let Some(token) = token_from_cookie {
        json_body["token"] = serde_json::json!(token);
    }
    json_body
}

pub async fn create_and_login_admin_user(
    app: &Router,
    db: &DatabaseConnection,
) -> (user::Model, String) {
    let password = std::env::var("TEST_PASSWORD").unwrap_or_default();
    // Create admin user directly in the database
    let admin_user = user::ActiveModel {
        id: Set(Uuid::new_v4()),
        username: Set(format!("admin{}", Uuid::new_v4())),
        first_name: Set("Admin".to_string()),
        last_name: Set("User".to_string()),
        email: Set(format!("admin{}@example.com", Uuid::new_v4())),
        phone: Set("1234567890".to_string()),
        password_hash: Set(crate::auth::hash_password(&password).unwrap()),
        is_active: Set(true),
        last_login: Set(Some(Utc::now())),
        created_at: Set(Utc::now()),
        updated_at: Set(Utc::now()),
        ..Default::default()
    }
    .insert(db)
    .await
    .expect("Failed to create admin user");

    let tenant = create_test_tenant(db).await;
    let account = crate::entities::account::ActiveModel {
        id: Set(Uuid::new_v4()),
        tenant_id: Set(tenant.id),
        name: Set("Admin Test Account".to_string()),
        is_active: Set(true),
        created_at: Set(Utc::now()),
        updated_at: Set(Utc::now()),
        ..Default::default()
    }.insert(db).await.expect("Failed to create admin account");

    user_account::ActiveModel {
        id: Set(Uuid::new_v4()),
        user_id: Set(admin_user.id),
        account_id: Set(account.id),
        role: Set(user_account::UserRole::PlatformSuperAdmin),
        is_active: Set(true),
        created_at: Set(Utc::now()),
        updated_at: Set(Utc::now()),
    }.insert(db).await.expect("Failed to create admin user account");

    // Login the admin user
    let login_response: serde_json::Value = login_test_user(app, &admin_user.email, &password).await;

    let token = login_response["token"].as_str().unwrap().to_string();

    (admin_user, token)
}

// New function to create staff user accounts
pub async fn create_staff_user_account(
    db: &DatabaseConnection,
    _admin_user: &user::Model,
    profile: &profile::Model,
    role: user_account::UserRole,
) -> user_account::Model {
    let username: String = Username().fake();
    let first_name: String = FirstName().fake();
    let last_name: String = LastName().fake();
    let email: String = SafeEmail().fake();
    let phone: String = PhoneNumber().fake();

    // Create staff user
    let staff_user = user::ActiveModel {
        id: Set(Uuid::new_v4()),
        username: Set(username),
        first_name: Set(first_name),
        last_name: Set(last_name),
        email: Set(email),
        phone: Set(phone),
        password_hash: Set(crate::auth::hash_password("staffpass123").unwrap()),
        is_active: Set(true),
        last_login: Set(Some(Utc::now())),
        created_at: Set(Utc::now()),
        updated_at: Set(Utc::now()),
        ..Default::default()
    }
    .insert(db)
    .await
    .expect("Failed to create staff user");

    // Create user account association
    user_account::ActiveModel {
        id: Set(Uuid::new_v4()),
        user_id: Set(staff_user.id),
        account_id: Set(profile.account_id),
        role: Set(role),
        is_active: Set(true),
        created_at: Set(Utc::now()),
        updated_at: Set(Utc::now()),
        ..Default::default()
    }
    .insert(db)
    .await
    .expect("Failed to create staff user account")
}

pub async fn create_default_category(
    db: &DatabaseConnection,
    tenant_id: Uuid,
) -> category::Model {
    category::ActiveModel {
        id: Set(Uuid::new_v4()),
        name: Set("Default Category".to_string()),
        description: Set("Default category for listings".to_string()),
        tenant_id: Set(Some(tenant_id)),
        parent_category_id: Set(None),
        is_custom: Set(false),
        is_active: Set(true),
        created_at: Set(Utc::now()),
        updated_at: Set(Utc::now()),
        ..Default::default()
    }
    .insert(db)
    .await
    .expect("Failed to create default category")
}

fn turn_name_to_domain(companyName: String) -> String {
    let processed = companyName
        .to_lowercase()
        .replace(' ', "-")
        .chars()
        .filter(|c| c.is_alphanumeric() || *c == '-')
        .collect::<String>()
        .trim_matches('-')
        .to_string();
    format!("http://www.{:?}.", processed)
}
