//! Integration tests for POST /api/auth/otp/send and /api/auth/otp/verify.
//!
//! Proves the backend captures email on OTP send and returns that email (plus a
//! session cookie) on verify — the contract Folio WizardShell relies on for
//! read-only `VerifiedEmailField` after warm onboarding OTP.

use crate::tests::api_tests::setup_test_app;
use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use sea_orm::{ColumnTrait, ConnectionTrait, EntityTrait, QueryFilter};
use serde_json::json;
use sha2::{Digest, Sha256};
use tower::ServiceExt;
use uuid::Uuid;

fn hash_otp(code: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(code.as_bytes());
    format!("{:x}", hasher.finalize())
}

#[tokio::test]
async fn otp_send_creates_user_and_stores_token() {
    let (app, db) = setup_test_app().await;
    let email = format!("otp-send-{}@example.com", Uuid::new_v4());

    let res = app
        .clone()
        .oneshot(
            Request::builder()
                .header("Host", "localhost")
                .method("POST")
                .uri("/api/auth/otp/send")
                .header("Content-Type", "application/json")
                .header("x-forwarded-for", "203.0.113.50")
                .body(Body::from(json!({ "email": email }).to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    let status = res.status();
    let body = axum::body::to_bytes(res.into_body(), usize::MAX)
        .await
        .unwrap();
    assert_eq!(
        status,
        StatusCode::OK,
        "otp/send body: {}",
        String::from_utf8_lossy(&body)
    );

    use crate::entities::user;
    let user = user::Entity::find()
        .filter(user::Column::Email.eq(&email))
        .one(&db)
        .await
        .unwrap()
        .expect("otp/send must find-or-create a user for the email");

    let token_count: u64 = db
        .query_one(sea_orm::Statement::from_sql_and_values(
            sea_orm::DatabaseBackend::Postgres,
            "SELECT COUNT(*)::bigint AS c FROM atlas_otp_tokens WHERE user_id = $1 AND is_used = false",
            [user.id.into()],
        ))
        .await
        .unwrap()
        .and_then(|r| r.try_get::<i64>("", "c").ok())
        .unwrap_or(0) as u64;

    assert!(
        token_count >= 1,
        "otp/send must persist at least one unused atlas_otp_tokens row"
    );
}

#[tokio::test]
async fn otp_verify_returns_session_cookie_and_email() {
    let (app, db) = setup_test_app().await;
    let email = format!("otp-verify-{}@example.com", Uuid::new_v4());
    let known_code = "123456";

    // 1. Send OTP (creates user + opaque hashed token)
    let send_res = app
        .clone()
        .oneshot(
            Request::builder()
                .header("Host", "localhost")
                .method("POST")
                .uri("/api/auth/otp/send")
                .header("Content-Type", "application/json")
                .header("x-forwarded-for", "203.0.113.51")
                .body(Body::from(json!({ "email": email }).to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(send_res.status(), StatusCode::OK);

    use crate::entities::user;
    let user = user::Entity::find()
        .filter(user::Column::Email.eq(&email))
        .one(&db)
        .await
        .unwrap()
        .expect("user after otp/send");

    // 2. Insert a known code hash so the test can verify without reading SMTP.
    let expires_at = chrono::Utc::now() + chrono::Duration::minutes(5);
    db.execute(sea_orm::Statement::from_sql_and_values(
        sea_orm::DatabaseBackend::Postgres,
        "INSERT INTO atlas_otp_tokens (id, user_id, code_hash, expires_at, is_used, created_at) \
         VALUES ($1, $2, $3, $4, false, NOW())",
        [
            Uuid::new_v4().into(),
            user.id.into(),
            hash_otp(known_code).into(),
            expires_at.into(),
        ],
    ))
    .await
    .expect("insert known otp token");

    // 3. Verify
    let ver_res = app
        .clone()
        .oneshot(
            Request::builder()
                .header("Host", "localhost")
                .method("POST")
                .uri("/api/auth/otp/verify")
                .header("Content-Type", "application/json")
                .body(Body::from(
                    json!({ "email": email, "code": known_code }).to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(ver_res.status(), StatusCode::OK);

    let set_cookie = ver_res
        .headers()
        .get("set-cookie")
        .expect("otp/verify must Set-Cookie session=…")
        .to_str()
        .unwrap()
        .to_string();
    assert!(
        set_cookie.contains("session="),
        "Set-Cookie must set session= cookie"
    );

    let body = axum::body::to_bytes(ver_res.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    let returned_email = json["user"]["email"]
        .as_str()
        .expect("verify response must include user.email");
    assert_eq!(
        returned_email, email,
        "captured email must round-trip on verify for Folio VerifiedEmailField"
    );

    // 4. Session validate sees the same email
    let token = set_cookie
        .split(';')
        .next()
        .and_then(|p| p.trim().strip_prefix("session="))
        .expect("parse session token from Set-Cookie")
        .to_string();

    let val_res = app
        .oneshot(
            Request::builder()
                .header("Host", "localhost")
                .method("GET")
                .uri("/api/auth/session/validate")
                .header("Cookie", format!("session={token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(val_res.status(), StatusCode::OK);
    let val_body = axum::body::to_bytes(val_res.into_body(), usize::MAX)
        .await
        .unwrap();
    let val_json: serde_json::Value = serde_json::from_slice(&val_body).unwrap();
    assert_eq!(
        val_json["user"]["email"].as_str(),
        Some(email.as_str()),
        "session/validate must expose the OTP-verified email"
    );
}
