use crate::tests::api_tests::setup_test_app;
use crate::tests::test_utils;
use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use serde_json::json;
use tower::ServiceExt;
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
    let user_id = Uuid::parse_str(
        json_body["user"]["id"]
            .as_str()
            .expect("No user id returned"),
    )
    .expect("Invalid user Uuid");

    // 1. Request Magic Link for correct email
    let req_res = app
        .clone()
        .oneshot(
            Request::builder()
                .header("Host", "localhost")
                .method("POST")
                .uri("/api/auth/magic-link/request")
                .header("Content-Type", "application/json")
                .body(Body::from(
                    json!({
                        "email": email,
                        "tenant_id": tenant.id
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(req_res.status(), StatusCode::OK);

    // Read the token straight from DB
    use crate::entities::magic_link_token;
    use chrono::{Duration, Utc};
    use sea_orm::{
        ActiveModelTrait, ColumnTrait, EntityTrait, PaginatorTrait, QueryFilter, QueryOrder, Set,
    };

    let token_model = magic_link_token::Entity::find()
        .filter(magic_link_token::Column::UserId.eq(user_id))
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
    }
    .insert(&db)
    .await
    .unwrap();

    // Verify Passkey actually exists before the fetch
    let count_before = passkey::Entity::find()
        .filter(passkey::Column::UserId.eq(token_model.user_id))
        .count(&db)
        .await
        .unwrap();
    assert_eq!(
        count_before, 1,
        "The mock passkey was not generated correctly"
    );

    // 2. Verify Magic Link successfully
    let ver_res = app
        .clone()
        .oneshot(
            Request::builder()
                .header("Host", "localhost")
                .method("POST")
                .uri("/api/auth/magic-link/verify")
                .header("Content-Type", "application/json")
                .body(Body::from(
                    json!({
                        "token": token_model.token,
                        "tenant_id": tenant.id
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(ver_res.status(), StatusCode::OK);

    // REGRESSION: SessionResponse.token is #[serde(skip_serializing)] so the JSON body
    // never contains the session token. The response MUST carry a Set-Cookie header —
    // without it the browser has no way to receive the session and every verification
    // appears as "expired" to the end-user even though the backend marked the token used.
    let set_cookie = ver_res
        .headers()
        .get("set-cookie")
        .expect("verify_magic_link must respond with a Set-Cookie header")
        .to_str()
        .expect("Set-Cookie header must be valid UTF-8");
    assert!(
        set_cookie.contains("session="),
        "Set-Cookie must set the 'session' cookie"
    );
    assert!(
        set_cookie.contains("HttpOnly"),
        "session cookie must be HttpOnly"
    );
    assert!(
        set_cookie.contains("SameSite=Strict"),
        "session cookie must have SameSite=Strict"
    );

    // Validate that consuming a regular Magic Link DOES NOT eradicate the mock passkey
    let count_after = passkey::Entity::find()
        .filter(passkey::Column::UserId.eq(token_model.user_id))
        .count(&db)
        .await
        .unwrap();
    assert_eq!(
        count_after, 1,
        "The passkey should NOT be purged after regular Magic Link verification"
    );

    // 3. Test Expiration logic (use is_used=true so we don't violate the new partial unique index)
    let expired_token_str = format!("expired_{}", Uuid::new_v4());
    magic_link_token::ActiveModel {
        id: Set(Uuid::new_v4()),
        user_id: Set(token_model.user_id),
        token: Set(expired_token_str.clone()),
        expires_at: Set(Utc::now() - Duration::minutes(30)),
        is_used: Set(true), // Mark used so it doesn't conflict with active-token constraint
        created_at: Set(Utc::now() - Duration::hours(1)),
        is_setup_token: Set(false),
        redirect_url: Set(None),
    }
    .insert(&db)
    .await
    .unwrap();

    let ver_expired_res = app
        .clone()
        .oneshot(
            Request::builder()
                .header("Host", "localhost")
                .method("POST")
                .uri("/api/auth/magic-link/verify")
                .header("Content-Type", "application/json")
                .body(Body::from(
                    json!({
                        "token": expired_token_str,
                        "tenant_id": tenant.id
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(
        ver_expired_res.status(),
        StatusCode::UNAUTHORIZED,
        "Expired tokens should be rejected"
    );

    // 4. Test Tenant Isolation (again use is_used=true for the cross-tenant test token)
    let other_tenant = test_utils::create_test_tenant(&db).await;

    let isolated_token_str = format!("iso_{}", Uuid::new_v4());
    magic_link_token::ActiveModel {
        id: Set(Uuid::new_v4()),
        user_id: Set(token_model.user_id),
        token: Set(isolated_token_str.clone()),
        expires_at: Set(Utc::now() + Duration::minutes(30)),
        is_used: Set(true), // Not an active token for this user — just for isolation test
        created_at: Set(Utc::now()),
        is_setup_token: Set(false),
        redirect_url: Set(None),
    }
    .insert(&db)
    .await
    .unwrap();

    let cross_tenant_res = app
        .clone()
        .oneshot(
            Request::builder()
                .header("Host", "localhost")
                .method("POST")
                .uri("/api/auth/magic-link/verify")
                .header("Content-Type", "application/json")
                .body(Body::from(
                    json!({
                        "token": isolated_token_str,
                        "tenant_id": other_tenant.id
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(
        cross_tenant_res.status(),
        StatusCode::UNAUTHORIZED,
        "Token mapped to Tenant A should not authenticate Tenant B"
    );
}

// ── New regression-guard tests ────────────────────────────────────────────────

/// T5 REGRESSION: A second magic-link request for the same email within 60 seconds
/// must return 200 OK but must NOT create a second active token.
///
/// ## Why the count query MUST be user-scoped
/// Tests run in parallel on a shared DB (`--test-threads=$(nproc)`). A global
/// `SELECT count(*) WHERE is_used = false` will include tokens created by other
/// tests running concurrently, causing a spurious failure even when this test's
/// user has exactly one token. Always filter by `user_id`.
///
/// ## Why the SMTP guard matters for local dev
/// `send_email_handler` reads `SMTP_SERVER` from the environment at runtime.
/// If a developer has a real SMTP server configured locally (e.g. via `.env`
/// loaded by direnv), the fire-and-forget tokio::task will dispatch a real email.
/// This produces two confusing signals:
///   - Developer receives a real "Sign in to Atlas Platform" email → looks like success
///   - Terminal shows `test_magic_link_request_idempotency ... FAILED` → says failure
///   - The disconnect is baffling: "the email arrived, so why did it fail?"
/// The SmtpGuard sets SMTP_SERVER="" for the test duration, forcing the mock path.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_magic_link_request_idempotency() {
    with_smtp_blanked(|| async {
        let (app, db) = setup_test_app().await;
        let tenant = test_utils::create_test_tenant(&db).await;

        let mut username = format!("idem_{}", Uuid::new_v4());
        let (status, json_body) =
            test_utils::register_test_user(&app, tenant.id, &mut username).await;
        assert_eq!(status, StatusCode::CREATED);
        let email = json_body["user"]["email"]
            .as_str()
            .expect("email missing from register response")
            .to_string();

        // Resolve the user_id now so count queries are scoped to this user only.
        // Global counts break when other tests run in parallel on the shared DB.
        use crate::entities::{magic_link_token, user};
        use sea_orm::{ColumnTrait, EntityTrait, PaginatorTrait, QueryFilter};

        let user_id = user::Entity::find()
            .filter(user::Column::Email.eq(&email))
            .one(&db)
            .await
            .unwrap()
            .expect("user not found — registration may have failed silently")
            .id;

        // ── First request ─────────────────────────────────────────────────────────
        let req1 = app
            .clone()
            .oneshot(
                Request::builder()
                    .header("Host", "localhost")
                    .method("POST")
                    .uri("/api/auth/magic-link/request")
                    .header("Content-Type", "application/json")
                    .body(Body::from(json!({"email": email}).to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(
            req1.status(),
            StatusCode::OK,
            "First request must return 200 OK"
        );

        let count_after_first = magic_link_token::Entity::find()
            .filter(magic_link_token::Column::UserId.eq(user_id))
            .filter(magic_link_token::Column::IsUsed.eq(false))
            .count(&db)
            .await
            .unwrap();
        assert_eq!(
            count_after_first, 1,
            "After the first request there must be exactly 1 active token for user {}. \
         Got {}. Check that request_magic_link (auth_frontend.rs) correctly \
         INSERTs a token row after expiring prior ones.",
            user_id, count_after_first
        );

        // ── Second request within idempotency window ───────────────────────────────
        // The MAGIC_LINK_REQUEST_CACHE (60s TTL Moka cache, keyed by email) must
        // return early before reaching the token INSERT. If the cache guard is ever
        // removed or short-circuited, this assertion catches it.
        let req2 = app
            .clone()
            .oneshot(
                Request::builder()
                    .header("Host", "localhost")
                    .method("POST")
                    .uri("/api/auth/magic-link/request")
                    .header("Content-Type", "application/json")
                    .body(Body::from(json!({"email": email}).to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(
            req2.status(),
            StatusCode::OK,
            "Duplicate request must also return 200 OK (silent dedup)"
        );

        let count_after_second = magic_link_token::Entity::find()
            .filter(magic_link_token::Column::UserId.eq(user_id))
            .filter(magic_link_token::Column::IsUsed.eq(false))
            .count(&db)
            .await
            .unwrap();
        assert_eq!(
            count_after_second, 1,
            "REGRESSION T5: active token count for user {} must still be 1 after a \
         duplicate request within the 60s idempotency window. Got {}. \
         The MAGIC_LINK_REQUEST_CACHE.get() guard in request_magic_link \
         (auth_frontend.rs) is either missing or not being reached before \
         the INSERT INTO magic_link_token statement.",
            user_id, count_after_second
        );
    })
    .await;
}

/// T3 REGRESSION: A magic-link token issued in the context of Tenant A must be
/// rejected when presented alongside Tenant B's ID during verification.
/// Extracted as a standalone test so CI failure messages pinpoint T3 precisely.
/// The composite test_magic_link_flow also covers this, but as step 4 of 4.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_magic_link_token_tenant_isolation() {
    with_smtp_blanked(|| async {
        let (app, db) = setup_test_app().await;
        let tenant_a = test_utils::create_test_tenant(&db).await;
        let tenant_b = test_utils::create_test_tenant(&db).await;

        let mut username = format!("tiso_{}", Uuid::new_v4());
        let (status, json_body) =
            test_utils::register_test_user(&app, tenant_a.id, &mut username).await;
        assert_eq!(status, StatusCode::CREATED);
        let email = json_body["user"]["email"].as_str().unwrap().to_string();

        // Request a magic link for the user registered under tenant_a.
        app.clone()
            .oneshot(
                Request::builder()
                    .header("Host", "localhost")
                    .method("POST")
                    .uri("/api/auth/magic-link/request")
                    .header("Content-Type", "application/json")
                    .body(Body::from(json!({"email": email}).to_string()))
                    .unwrap(),
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
            .expect("token must have been created — check request_magic_link handler")
            .token;

        // Attempt verification with tenant_b's ID — must be rejected with 401.
        let res = app
            .clone()
            .oneshot(
                Request::builder()
                    .header("Host", "localhost")
                    .method("POST")
                    .uri("/api/auth/magic-link/verify")
                    .header("Content-Type", "application/json")
                    .body(Body::from(
                        json!({
                            "token": token,
                            "tenant_id": tenant_b.id
                        })
                        .to_string(),
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(
            res.status(),
            StatusCode::UNAUTHORIZED,
            "REGRESSION T3: token created for tenant_a must be rejected when \
         verify_magic_link is called with tenant_b's ID. Got HTTP {}. \
         This means cross-tenant token verification is not being rejected — \
         check the tenant_id validation in verify_magic_link (auth_frontend.rs).",
            res.status()
        );
    })
    .await;
}

// ── Test Helpers ──────────────────────────────────────────────────────────────

/// Runs an async test body with `SMTP_SERVER` forced to `""` for the duration.
///
/// # Why this is needed
/// `send_email_handler` resolves the SMTP host from `std::env::var("SMTP_SERVER")`
/// at runtime. Developers often have a real SMTP server configured in their shell
/// (e.g. via `.env` loaded by direnv). When a test calls the magic-link endpoint the
/// fire-and-forget tokio task connects to the real SMTP server and delivers an actual
/// email, causing two confusing signals:
///
///   1. **False-success**: the developer receives a real email implying success while
///      the terminal shows `FAILED` — baffling.
///   2. **Intermittent timing failures**: real-SMTP latency races with DB assertions.
///
/// # How it works
/// `temp_env::with_var` acquires a process-global `Mutex` before touching the
/// environment, making the mutation safe across concurrent test threads.
///
/// `tokio::task::block_in_place` yields the current tokio thread to a blocking
/// context, which allows `Handle::block_on` to drive the async future to completion
/// without triggering the "cannot start a runtime from within a runtime" panic that
/// a plain `block_on` call inside `#[tokio::test]` would cause.
///
/// # Runtime flavor requirement
/// `tokio::task::block_in_place` **requires the multi-thread runtime** — it panics
/// with "can call blocking only when running on the multi-threaded runtime" on a
/// `current_thread` executor. Tests that call this helper MUST use:
/// ```ignore
/// #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
/// ```
async fn with_smtp_blanked<F, Fut>(f: F)
where
    F: FnOnce() -> Fut,
    Fut: std::future::Future<Output = ()>,
{
    let handle = tokio::runtime::Handle::current();
    let future = f();
    temp_env::with_var("SMTP_SERVER", Some(""), || {
        tokio::task::block_in_place(|| handle.block_on(future));
    });
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_magic_link_outbox_processing() {
    with_smtp_blanked(|| async {
        let (app, db) = setup_test_app().await;
        let tenant = test_utils::create_test_tenant(&db).await;

        let mut username = format!("outbox_{}", Uuid::new_v4());
        let (status, json_body) =
            test_utils::register_test_user(&app, tenant.id, &mut username).await;
        assert_eq!(status, StatusCode::CREATED);
        let email = json_body["user"]["email"]
            .as_str()
            .expect("email missing from register response")
            .to_string();

        // 1. Request magic link. This should atomically insert a pending outbox job.
        let req = app
            .clone()
            .oneshot(
                Request::builder()
                    .header("Host", "localhost")
                    .method("POST")
                    .uri("/api/auth/magic-link/request")
                    .header("Content-Type", "application/json")
                    .body(Body::from(json!({"email": email}).to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(req.status(), StatusCode::OK);

        // 2. Query the database directly to verify a pending job was created.
        use crate::entities::outbox_job;
        use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};

        let pending_jobs = outbox_job::Entity::find()
            .filter(outbox_job::Column::Status.eq("pending"))
            .filter(outbox_job::Column::JobType.eq("send_magic_link_email"))
            .all(&db)
            .await
            .unwrap();

        // Verify exactly one job exists for this request path.
        assert!(
            !pending_jobs.is_empty(),
            "A pending outbox job must have been enqueued"
        );
        let job_id = pending_jobs[0].id;

        // Shared test DB: other suites may enqueue earlier `run_at` jobs. Drain until
        // *this* magic-link job completes (same scoping pattern as user_id filters above).
        for _ in 0..32 {
            let job = outbox_job::Entity::find_by_id(job_id)
                .one(&db)
                .await
                .unwrap()
                .expect("outbox job row");
            if job.status == "completed" {
                return;
            }
            if job.status == "failed" {
                panic!("magic-link outbox job failed: {:?}", job.error_message);
            }
            crate::services::outbox_worker::OutboxWorker::process_next_job(&db)
                .await
                .unwrap();
        }

        let job = outbox_job::Entity::find_by_id(job_id)
            .one(&db)
            .await
            .unwrap()
            .expect("outbox job row");
        assert_eq!(
            job.status, "completed",
            "The outbox job must be marked as 'completed' (got status={}, err={:?})",
            job.status, job.error_message
        );
    })
    .await;
}

// ── Email branding regression tests ───────────────────────────────────────────

/// REGRESSION: `property_management` app_type must trigger Folio branding.
///
/// Incident root cause: `is_folio` only matched `"folio"` but every provisioned
/// Folio instance stores `app_type = "property_management"`. As a result every
/// magic-link email subject line read "Sign in to {tenant_slug}" instead of
/// "Your Folio Magic Link".
///
/// Fix: `is_folio` now also matches `"property_management"` (auth_frontend.rs).
///
/// This test registers a `property_management` app_instance + domain, requests
/// a magic link pointing at that domain, and asserts the outbox job subject is
/// exactly "Your Folio Magic Link".
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_property_management_app_type_triggers_folio_branding() {
    with_smtp_blanked(|| async {
        use crate::entities::{app_domain, app_instance, outbox_job};
        use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, Set};

        let (app, db) = setup_test_app().await;
        let tenant = test_utils::create_test_tenant(&db).await;

        // Register a user in this tenant.
        let mut username = format!("foliobrand_{}", uuid::Uuid::new_v4());
        let (status, json_body) =
            test_utils::register_test_user(&app, tenant.id, &mut username).await;
        assert_eq!(status, StatusCode::CREATED);
        let email = json_body["user"]["email"].as_str().unwrap().to_string();

        // Create a property_management app_instance for this tenant.
        let instance_id = uuid::Uuid::new_v4();
        app_instance::ActiveModel {
            id: Set(instance_id),
            tenant_id: Set(tenant.id),
            app_type: Set("property_management".to_string()),
            created_at: Set(chrono::Utc::now()),
            updated_at: Set(chrono::Utc::now()),
            ..Default::default()
        }
        .insert(&db)
        .await
        .expect("failed to insert app_instance");

        // Register a domain for that instance.
        let domain = format!("folio.test-{}.example.com", uuid::Uuid::new_v4());
        app_domain::ActiveModel {
            id: Set(uuid::Uuid::new_v4()),
            app_instance_id: Set(instance_id),
            domain_name: Set(domain.clone()),
            created_at: Set(chrono::Utc::now()),
        }
        .insert(&db)
        .await
        .expect("failed to insert app_domain");

        // Request a magic link with redirect_url pointing at the property_management domain.
        let redirect_url = format!("https://{}/verify", domain);
        let req = app
            .clone()
            .oneshot(
                axum::http::Request::builder()
                    .method("POST")
                    .uri("/api/auth/magic-link/request")
                    .header("Content-Type", "application/json")
                    .header("Host", "localhost")
                    .body(axum::body::Body::from(
                        serde_json::json!({
                            "email":        email,
                            "redirect_url": redirect_url,
                        })
                        .to_string(),
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(
            req.status(),
            StatusCode::OK,
            "magic link request must return 200"
        );

        // Read the outbox job and verify email subject uses Folio branding.
        let jobs = outbox_job::Entity::find()
            .filter(outbox_job::Column::JobType.eq("send_magic_link_email"))
            .filter(outbox_job::Column::TenantId.eq(tenant.id))
            .all(&db)
            .await
            .unwrap();

        assert!(!jobs.is_empty(), "an outbox job must have been created");

        let subject = jobs
            .last()
            .unwrap()
            .payload
            .get("subject")
            .and_then(|v| v.as_str())
            .unwrap_or("");

        assert_eq!(
            subject, "Your Folio Magic Link",
            "REGRESSION: property_management app_type must produce Folio email branding. \
         Got subject: '{}'. Check is_folio detection in auth_frontend.rs — \
         must match both 'folio' and 'property_management'.",
            subject
        );
    })
    .await;
}

/// REGRESSION: `tenant.page_title` must be used as the email brand name,
/// falling back to `tenant.name` only when `page_title` is NULL or empty.
///
/// Incident: emails said "Sign in to ruuderie" (the internal tenant slug)
/// instead of the operator-configured display name.
///
/// Fix: request_magic_link now reads `tenant.page_title` first, falling back
/// to `tenant.name` (auth_frontend.rs).
///
/// This is the productized way for operators to control what users see in
/// emails and browser titles — set `page_title` in platform-admin tenant settings.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_magic_link_email_uses_page_title_as_brand_name() {
    with_smtp_blanked(|| async {
    use crate::entities::{app_instance, app_domain, outbox_job, tenant};
    use sea_orm::{ActiveModelTrait, EntityTrait, QueryFilter, ColumnTrait, QueryOrder, Set};

    let (app, db) = setup_test_app().await;

    // Create a tenant with a raw internal name but a proper page_title.
    let raw_name = format!("rawslug_{}", &uuid::Uuid::new_v4().to_string()[..8]);
    let display_name = "Acme Property Co";
    let base_tenant = test_utils::create_test_tenant(&db).await;

    // Update the tenant: set name to raw_name and page_title to display_name.
    let mut tenant_active: tenant::ActiveModel = base_tenant.clone().into();
    tenant_active.name       = Set(raw_name.clone());
    tenant_active.page_title = Set(Some(display_name.to_string()));
    let updated_tenant = tenant_active.update(&db).await.expect("failed to update tenant");

    // Register a user.
    let mut username = format!("pagetitle_{}", uuid::Uuid::new_v4());
    let (status, json_body) = test_utils::register_test_user(&app, updated_tenant.id, &mut username).await;
    assert_eq!(status, StatusCode::CREATED);
    let email = json_body["user"]["email"].as_str().unwrap().to_string();

    // Create a non-Folio app_instance (anchor) so brand_name logic runs.
    let instance_id = uuid::Uuid::new_v4();
    app_instance::ActiveModel {
        id:        Set(instance_id),
        tenant_id: Set(updated_tenant.id),
        app_type:  Set("anchor".to_string()),
        created_at: Set(chrono::Utc::now()),
        updated_at: Set(chrono::Utc::now()),
        ..Default::default()
    }
    .insert(&db)
    .await
    .expect("failed to insert app_instance");

    let domain = format!("anchor.test-{}.example.com", uuid::Uuid::new_v4());
    app_domain::ActiveModel {
        id:              Set(uuid::Uuid::new_v4()),
        app_instance_id: Set(instance_id),
        domain_name:     Set(domain.clone()),
        created_at:      Set(chrono::Utc::now()),
    }
    .insert(&db)
    .await
    .expect("failed to insert app_domain");

    let redirect_url = format!("https://{}/verify", domain);
    let req = app.clone()
        .oneshot(
            axum::http::Request::builder()
                .method("POST")
                .uri("/api/auth/magic-link/request")
                .header("Content-Type", "application/json")
                .header("Host", "localhost")
                .body(axum::body::Body::from(serde_json::json!({
                    "email":        email,
                    "redirect_url": redirect_url,
                }).to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(req.status(), StatusCode::OK);

    let jobs = outbox_job::Entity::find()
        .filter(outbox_job::Column::JobType.eq("send_magic_link_email"))
        .filter(outbox_job::Column::TenantId.eq(updated_tenant.id))
        .all(&db)
        .await
        .unwrap();

    assert!(!jobs.is_empty(), "an outbox job must have been created");

    let subject = jobs.last().unwrap()
        .payload
        .get("subject")
        .and_then(|v| v.as_str())
        .unwrap_or("");

    let expected = format!("Sign in to {}", display_name);
    assert_eq!(
        subject, expected,
        "REGRESSION: email subject must use tenant.page_title ('{}') not tenant.name ('{}').\
         Got: '{}'. Check that request_magic_link reads page_title before name in auth_frontend.rs.",
        display_name, raw_name, subject
    );

    // ── Fallback: empty page_title must fall back to tenant.name ──────────────
    //
    // We CANNOT re-use the same email as the first request: the 60-second
    // in-memory idempotency cache (MAGIC_LINK_REQUEST_CACHE, keyed by email)
    // is still hot. A second call for the same address returns 200 OK but
    // does NOT create a new outbox job, so we'd still see the old subject.
    // Solution: register a fresh user whose email is a different cache key.

    let mut tenant_no_title: tenant::ActiveModel = updated_tenant.clone().into();
    tenant_no_title.page_title = Set(Some(String::new())); // empty string → use name
    tenant_no_title.update(&db).await.expect("failed to clear page_title");

    // Fresh user → fresh cache key → idempotency cache doesn't suppress this request.
    let mut username2 = format!("pagetitle2_{}", uuid::Uuid::new_v4());
    let (status2, json_body2) = test_utils::register_test_user(&app, updated_tenant.id, &mut username2).await;
    assert_eq!(status2, StatusCode::CREATED);
    let email2 = json_body2["user"]["email"].as_str().unwrap().to_string();

    let req2 = app.clone()
        .oneshot(
            axum::http::Request::builder()
                .method("POST")
                .uri("/api/auth/magic-link/request")
                .header("Content-Type", "application/json")
                .header("Host", "localhost")
                .body(axum::body::Body::from(serde_json::json!({
                    "email":        email2,
                    "redirect_url": redirect_url,
                }).to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(req2.status(), StatusCode::OK, "second request (fallback user) must return 200");

    let jobs2 = outbox_job::Entity::find()
        .filter(outbox_job::Column::JobType.eq("send_magic_link_email"))
        .filter(outbox_job::Column::TenantId.eq(updated_tenant.id))
        .order_by_desc(outbox_job::Column::Id)
        .all(&db)
        .await
        .unwrap();

    // Pick the job for email2 (the fallback user's job).
    let fallback_job = jobs2.iter().find(|j| {
        j.payload.get("to_email")
            .and_then(|v: &serde_json::Value| v.as_str())
            == Some(email2.as_str())
    });
    let subject2 = fallback_job
        .expect("must have an outbox job for the fallback user")
        .payload
        .get("subject")
        .and_then(|v: &serde_json::Value| v.as_str())
        .unwrap_or("");

    let fallback_expected = format!("Sign in to {}", raw_name);
    assert_eq!(
        subject2, fallback_expected,
        "When page_title is empty, email subject must fall back to tenant.name. \
         Expected: '{}', got: '{}'.",
        fallback_expected, subject2
    );
    }).await;
}
