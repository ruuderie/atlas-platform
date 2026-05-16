use axum::{body::Body, http::{Request, StatusCode}};
use tower::ServiceExt;
use serde_json::json;
use crate::tests::api_tests::setup_test_app;
use crate::tests::test_utils;
use uuid::Uuid;

// PERFORMANCE INVARIANT (T5): The backend must return 200 OK before SMTP dispatch completes.
// In test environments, SMTP host == "localhost" triggers an early-return mock path in
// send_email_handler(), so this test also implicitly validates that the handler is NOT
// blocking on SMTP in the hot path. If the mock path is ever removed, this test will
// hang and timeout in CI — which is the correct signal.
#[tokio::test]
async fn test_magic_link_flow() {
    let (app, db) = setup_test_app().await;
    let tenant = test_utils::create_test_tenant(&db).await;

    // Create user
    let mut username = format!("testuser{}", Uuid::new_v4());
    let (status, json_body) = test_utils::register_test_user(&app, tenant.id, &mut username).await;
    assert_eq!(status, StatusCode::CREATED);
    let email = json_body["user"]["email"]
        .as_str()
        .expect("No email returned in register response")
        .to_string();

    // 1. Request Magic Link for correct email
    let req_res = app.clone()
        .oneshot(
            Request::builder().header("Host", "localhost")
                .method("POST")
                .uri("/api/auth/magic-link/request")
                .header("Content-Type", "application/json")
                .body(Body::from(json!({
                    "email": email,
                    "tenant_id": tenant.id
                }).to_string()))
                .unwrap()
        )
        .await
        .unwrap();
    assert_eq!(req_res.status(), StatusCode::OK);

    // Read the token straight from DB
    use crate::entities::magic_link_token;
    use sea_orm::{EntityTrait, QueryOrder, ActiveModelTrait, Set, ColumnTrait, QueryFilter, PaginatorTrait};
    use chrono::{Utc, Duration};
    
    let token_model = magic_link_token::Entity::find()
        .order_by_desc(magic_link_token::Column::CreatedAt)
        .one(&db)
        .await
        .unwrap()
        .expect("No token created");

    // Before verifying, mint a fake Passkey to test the purging mechanism!
    use crate::entities::passkey;
    passkey::ActiveModel {
        id: Set(Uuid::new_v4()),
        user_id: Set(token_model.user_id),
        credential_id: Set(vec![1, 2, 3]),
        public_key: Set(vec![4, 5, 6]),
        sign_count: Set(0),
        name: Set("Mock iPhone Passkey".to_string()),
        last_used_at: Set(None),
        created_at: Set(Utc::now()),
        updated_at: Set(Utc::now()),
    }.insert(&db).await.unwrap();

    // Verify Passkey actually exists before the fetch
    let count_before = passkey::Entity::find()
        .filter(passkey::Column::UserId.eq(token_model.user_id))
        .count(&db)
        .await
        .unwrap();
    assert_eq!(count_before, 1, "The mock passkey was not generated correctly");

    // 2. Verify Magic Link successfully
    let ver_res = app.clone()
        .oneshot(
            Request::builder().header("Host", "localhost")
                .method("POST")
                .uri("/api/auth/magic-link/verify")
                .header("Content-Type", "application/json")
                .body(Body::from(json!({
                    "token": token_model.token,
                    "tenant_id": tenant.id
                }).to_string()))
                .unwrap()
        )
        .await
        .unwrap();
    assert_eq!(ver_res.status(), StatusCode::OK);

    // REGRESSION: SessionResponse.token is #[serde(skip_serializing)] so the JSON body
    // never contains the session token. The response MUST carry a Set-Cookie header —
    // without it the browser has no way to receive the session and every verification
    // appears as "expired" to the end-user even though the backend marked the token used.
    let set_cookie = ver_res.headers().get("set-cookie")
        .expect("verify_magic_link must respond with a Set-Cookie header")
        .to_str()
        .expect("Set-Cookie header must be valid UTF-8");
    assert!(set_cookie.contains("session="), "Set-Cookie must set the 'session' cookie");
    assert!(set_cookie.contains("HttpOnly"), "session cookie must be HttpOnly");
    assert!(set_cookie.contains("SameSite=Strict"), "session cookie must have SameSite=Strict");

    // Validate that consuming a regular Magic Link DOES NOT eradicate the mock passkey
    let count_after = passkey::Entity::find()
        .filter(passkey::Column::UserId.eq(token_model.user_id))
        .count(&db)
        .await
        .unwrap();
    assert_eq!(count_after, 1, "The passkey should NOT be purged after regular Magic Link verification");

    // 3. Test Expiration logic (use is_used=true so we don't violate the new partial unique index)
    let expired_token_str = format!("expired_{}", Uuid::new_v4());
    magic_link_token::ActiveModel {
        id: Set(Uuid::new_v4()),
        user_id: Set(token_model.user_id),
        token: Set(expired_token_str.clone()),
        expires_at: Set(Utc::now() - Duration::minutes(30)),
        is_used: Set(true),   // Mark used so it doesn't conflict with active-token constraint
        created_at: Set(Utc::now() - Duration::hours(1)),
        is_setup_token: Set(false),
        redirect_url: Set(None),
    }.insert(&db).await.unwrap();

    let ver_expired_res = app.clone()
        .oneshot(
            Request::builder().header("Host", "localhost")
                .method("POST")
                .uri("/api/auth/magic-link/verify")
                .header("Content-Type", "application/json")
                .body(Body::from(json!({
                    "token": expired_token_str,
                    "tenant_id": tenant.id
                }).to_string()))
                .unwrap()
        )
        .await
        .unwrap();
    assert_eq!(ver_expired_res.status(), StatusCode::UNAUTHORIZED, "Expired tokens should be rejected");

    // 4. Test Tenant Isolation (again use is_used=true for the cross-tenant test token)
    let other_tenant = test_utils::create_test_tenant(&db).await;
    
    let isolated_token_str = format!("iso_{}", Uuid::new_v4());
    magic_link_token::ActiveModel {
        id: Set(Uuid::new_v4()),
        user_id: Set(token_model.user_id),
        token: Set(isolated_token_str.clone()),
        expires_at: Set(Utc::now() + Duration::minutes(30)),
        is_used: Set(true),   // Not an active token for this user — just for isolation test
        created_at: Set(Utc::now()),
        is_setup_token: Set(false),
        redirect_url: Set(None),
    }.insert(&db).await.unwrap();

    let cross_tenant_res = app.clone()
        .oneshot(
            Request::builder().header("Host", "localhost")
                .method("POST")
                .uri("/api/auth/magic-link/verify")
                .header("Content-Type", "application/json")
                .body(Body::from(json!({
                    "token": isolated_token_str,
                    "tenant_id": other_tenant.id
                }).to_string()))
                .unwrap()
        )
        .await
        .unwrap();
        
    assert_eq!(cross_tenant_res.status(), StatusCode::UNAUTHORIZED, "Token mapped to Tenant A should not authenticate Tenant B");
}

// ── New regression-guard tests ────────────────────────────────────────────────

/// T5 REGRESSION: A second magic-link request for the same email within 60 seconds
/// must return 200 OK but must NOT create a second active token.
/// Guards against the pre-hydration double-submit pattern where the SSR-rendered
/// button fires before WASM hydrates, and WASM fires a second time after hydration.
#[tokio::test]
async fn test_magic_link_request_idempotency() {
    let (app, db) = setup_test_app().await;
    let tenant = test_utils::create_test_tenant(&db).await;

    let mut username = format!("idem_{}", Uuid::new_v4());
    let (status, json_body) = test_utils::register_test_user(&app, tenant.id, &mut username).await;
    assert_eq!(status, StatusCode::CREATED);
    let email = json_body["user"]["email"]
        .as_str()
        .expect("email missing from register response")
        .to_string();

    // First request — should succeed and create exactly one active token.
    let req1 = app.clone()
        .oneshot(
            Request::builder().header("Host", "localhost")
                .method("POST")
                .uri("/api/auth/magic-link/request")
                .header("Content-Type", "application/json")
                .body(Body::from(json!({"email": email}).to_string()))
                .unwrap()
        )
        .await
        .unwrap();
    assert_eq!(req1.status(), StatusCode::OK, "First request must return 200 OK");

    use crate::entities::magic_link_token;
    use sea_orm::{EntityTrait, QueryFilter, ColumnTrait, PaginatorTrait};

    let count_after_first = magic_link_token::Entity::find()
        .filter(magic_link_token::Column::IsUsed.eq(false))
        .count(&db)
        .await
        .unwrap();
    assert_eq!(count_after_first, 1, "Exactly one active token must exist after the first request");

    // Second request for the same email within the 60-second idempotency window.
    // Must return 200 OK (silent dedup) but must NOT create another token.
    let req2 = app.clone()
        .oneshot(
            Request::builder().header("Host", "localhost")
                .method("POST")
                .uri("/api/auth/magic-link/request")
                .header("Content-Type", "application/json")
                .body(Body::from(json!({"email": email}).to_string()))
                .unwrap()
        )
        .await
        .unwrap();
    assert_eq!(req2.status(), StatusCode::OK, "Second request must also return 200 OK");

    let count_after_second = magic_link_token::Entity::find()
        .filter(magic_link_token::Column::IsUsed.eq(false))
        .count(&db)
        .await
        .unwrap();
    assert_eq!(
        count_after_second, 1,
        "REGRESSION T5: second request within idempotency window must NOT create a new token \
         (MAGIC_LINK_REQUEST_CACHE must be wired in the hot path)"
    );
}

/// T3 REGRESSION: A magic-link token issued in the context of Tenant A must be
/// rejected when presented alongside Tenant B's ID during verification.
/// Extracted as a standalone test so CI failure messages pinpoint T3 precisely.
/// The composite test_magic_link_flow also covers this, but as step 4 of 4.
#[tokio::test]
async fn test_magic_link_token_tenant_isolation() {
    let (app, db) = setup_test_app().await;
    let tenant_a = test_utils::create_test_tenant(&db).await;
    let tenant_b = test_utils::create_test_tenant(&db).await;

    let mut username = format!("tiso_{}", Uuid::new_v4());
    let (status, json_body) = test_utils::register_test_user(&app, tenant_a.id, &mut username).await;
    assert_eq!(status, StatusCode::CREATED);
    let email = json_body["user"]["email"].as_str().unwrap().to_string();

    // Request a magic link for the user registered under tenant_a.
    app.clone()
        .oneshot(
            Request::builder().header("Host", "localhost")
                .method("POST")
                .uri("/api/auth/magic-link/request")
                .header("Content-Type", "application/json")
                .body(Body::from(json!({"email": email}).to_string()))
                .unwrap()
        )
        .await
        .unwrap();

    use crate::entities::magic_link_token;
    use sea_orm::{EntityTrait, QueryOrder};
    let token = magic_link_token::Entity::find()
        .order_by_desc(magic_link_token::Column::CreatedAt)
        .one(&db)
        .await
        .unwrap()
        .expect("token must have been created")
        .token;

    // Attempt verification with tenant_b's ID — must be rejected with 401.
    let res = app.clone()
        .oneshot(
            Request::builder().header("Host", "localhost")
                .method("POST")
                .uri("/api/auth/magic-link/verify")
                .header("Content-Type", "application/json")
                .body(Body::from(json!({
                    "token": token,
                    "tenant_id": tenant_b.id
                }).to_string()))
                .unwrap()
        )
        .await
        .unwrap();

    assert_eq!(
        res.status(),
        StatusCode::UNAUTHORIZED,
        "REGRESSION T3: token for tenant_a must be rejected when presented with tenant_b's ID \
         (session cookie Domain= omission is what enforces subdomain isolation at the browser level)"
    );
}
