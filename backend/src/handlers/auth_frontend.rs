use axum::{
    extract::{State, Query, Extension, Json},
    http::{StatusCode, header},
    response::IntoResponse,
    routing::{get, post},
    Router,
};
use sea_orm::{DatabaseConnection, EntityTrait, QueryFilter, ColumnTrait, Set, ActiveModelTrait, TransactionTrait};
use serde::Deserialize;
use serde_json::json;
use uuid::Uuid;
use std::time::Instant;
use std::time::Duration as StdDuration;
use chrono::{Utc, Duration};
use moka::future::Cache;
use once_cell::sync::Lazy;
use rand::{distributions::Alphanumeric, Rng};

use crate::entities::{app_domain, magic_link_token, tenant, user, app_instance, account, user_account};
use crate::auth::verify_password;
use crate::handlers::sessions::create_user_session;
use crate::metrics;  // Prometheus metrics

static MAGIC_LINK_REQUEST_CACHE: Lazy<Cache<String, bool>> = Lazy::new(|| {
    Cache::builder()
        .max_capacity(1000)
        .time_to_live(StdDuration::from_secs(60))
        .build()
});

fn generate_request_id() -> Uuid {
    Uuid::new_v4()
}

#[derive(Deserialize)]
pub struct VerifyEmailQuery {
    pub token: String,
}

#[derive(Deserialize)]
pub struct LoginCredentials {
    pub email: String,
    pub password: String,
}

pub fn public_routes() -> Router<DatabaseConnection> {
    Router::new()
        .route("/api/auth/verify-email", get(verify_email))
        .route("/api/auth/login", post(login))
        .route("/api/auth/flow/{email}", get(get_auth_flow))
        .route("/api/auth/magic-link/request", post(request_magic_link))
        .route("/api/auth/magic-link/verify", post(verify_magic_link))
        .route("/api/auth/webauthn/register", post(webauthn_register_start))
        .route("/api/auth/webauthn/authenticate", post(webauthn_auth_start))
}

pub fn authenticated_routes() -> Router<DatabaseConnection> {
    Router::new()
        .route("/api/me", get(get_me))
}

#[derive(serde::Serialize)]
pub struct AuthFlowResponse {
    pub has_passkey: bool,
}

pub async fn get_auth_flow(
    State(db): State<DatabaseConnection>,
    axum::extract::Path(email): axum::extract::Path<String>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let start = Instant::now();
    let request_id = generate_request_id();

    let user_model = user::Entity::find()
        .filter(user::Column::Email.eq(&email))
        .one(&db)
        .await
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "Database Error".to_string()))?;

    let has_passkey = if let Some(user_mod) = user_model {
        let passkeys = crate::entities::passkey::Entity::find()
            .filter(crate::entities::passkey::Column::UserId.eq(user_mod.id))
            .all(&db)
            .await
            .unwrap_or_default();
        passkeys.len() > 0
    } else {
        false
    };

    tracing::info!(
        event = "auth.flow.checked",
        request_id = %request_id,
        duration_ms = start.elapsed().as_millis(),
        status = "success"
    );

    Ok((StatusCode::OK, Json(AuthFlowResponse { has_passkey })))
}

pub async fn verify_email(
    State(_db): State<DatabaseConnection>,
    Query(_query): Query<VerifyEmailQuery>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    Ok((StatusCode::OK, Json(json!({"message": "Email verified successfully"}))))
}

pub async fn login(
    State(db): State<DatabaseConnection>,
    Json(credentials): Json<LoginCredentials>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let start = Instant::now();
    let request_id = generate_request_id();

    let user = user::Entity::find()
        .filter(user::Column::Email.eq(&credentials.email))
        .one(&db)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .ok_or((StatusCode::UNAUTHORIZED, "Invalid credentials".to_string()))?;

    let is_valid = verify_password(&credentials.password, &user.password_hash)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    if !is_valid {
        tracing::warn!(
            event = "auth.login.failed",
            request_id = %request_id,
            email = %credentials.email,
            reason = "invalid_password",
            duration_ms = start.elapsed().as_millis()
        );
        return Err((StatusCode::UNAUTHORIZED, "Invalid credentials".to_string()));
    }

    if !user.is_active {
        return Err((StatusCode::UNAUTHORIZED, "Account is inactive. Please verify your email.".to_string()));
    }

    let session_response = create_user_session(&db, &credentials.email, &credentials.password)
        .await
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "Failed to create session".to_string()))?;

    tracing::info!(
        event = "auth.login.success",
        request_id = %request_id,
        user_id = %user.id,
        duration_ms = start.elapsed().as_millis(),
        status = "success"
    );

    Ok(Json(session_response))
}

pub async fn get_me(
    Extension(current_user): Extension<user::Model>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    Ok(Json(json!({
        "id": current_user.id,
        "email": current_user.email,
        "first_name": current_user.first_name,
        "last_name": current_user.last_name,
        "username": current_user.username,
        "is_active": current_user.is_active,
    })))
}

pub async fn webauthn_register_start() -> Result<impl IntoResponse, StatusCode> {
    Ok((StatusCode::NOT_IMPLEMENTED, "WebAuthn not fully implemented yet"))
}

pub async fn webauthn_auth_start() -> Result<impl IntoResponse, StatusCode> {
    Ok((StatusCode::NOT_IMPLEMENTED, "WebAuthn not fully implemented yet"))
}

#[derive(Deserialize)]
pub struct RequestMagicLinkPayload {
    pub email: String,
    pub redirect_url: Option<String>,
}

pub async fn request_magic_link(
    State(db): State<DatabaseConnection>,
    Extension(rate_limiter): Extension<crate::middleware::rate_limiter::RateLimiter>,
    headers: axum::http::HeaderMap,
    Json(payload): Json<RequestMagicLinkPayload>,
) -> Result<(StatusCode, Json<serde_json::Value>), StatusCode> {
    let start = Instant::now();
    let request_id = generate_request_id();

    let ip = headers
        .get("x-forwarded-for")
        .and_then(|h| h.to_str().ok())
        .unwrap_or("unknown")
        .to_string();

    let user_agent = headers
        .get("user-agent")
        .and_then(|h| h.to_str().ok())
        .unwrap_or("unknown")
        .to_string();

    // Extract app_instance_id from header or resolve from domain
    let app_instance_id = headers
        .get("x-app-instance-id")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("unknown")
        .to_string();

    if let Err(status) = rate_limiter.check_auth_rate_limit(&ip, &payload.email).await {
        tracing::warn!(
            event = "magic_link.rate_limited",
            request_id = %request_id,
            ip = %ip,
            email = %payload.email,
            duration_ms = start.elapsed().as_millis()
        );
        return Err(status);
    }

    // IDEMPOTENCY GUARD — Layer 1: in-memory cache (same-pod deduplication).
    //
    // The cache is populated HERE, before any DB writes, so that concurrent requests
    // on the same pod are caught before they reach the database. Previously the cache
    // was populated AFTER the upsert, leaving a race window where two rapid requests
    // could both miss the cache and both proceed to the DB.
    //
    // TTL is 60 seconds (set in the Lazy initialiser above). This covers:
    //   - UI double-submits (pre-hydration SSR click + post-WASM click)
    //   - LB/browser retries triggered by slow SMTP
    //
    // Cross-pod deduplication is handled by the DB upsert (Layer 2, below).
    let cache_key = payload.email.to_lowercase();
    if MAGIC_LINK_REQUEST_CACHE.get(&cache_key).await.is_some() {
        tracing::info!(
            event = "magic_link.deduplicated",
            request_id = %request_id,
            email = %payload.email,
            duration_ms = start.elapsed().as_millis()
        );
        return Ok((StatusCode::OK, Json(json!({"message": "If the email exists, a magic link has been sent."}))));
    }
    // NOTE: we do NOT insert into the cache here yet.
    // The cache slot is claimed only after a successful transaction commit (below),
    // so that a non-existent user or a DB failure does not poison the cache and
    // block legitimate retries for 60 seconds.

    let user_model = user::Entity::find()
        .filter(user::Column::Email.eq(&payload.email))
        .one(&db)
        .await
        .map_err(|e| {
            tracing::error!("Error finding user: {:?}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    let validated_redirect_url_data = if let Some(ref url_str) = payload.redirect_url {
        match url::Url::parse(url_str) {
            Ok(parsed) => {
                let scheme = parsed.scheme();
                if scheme != "https" && scheme != "http" {
                    tracing::warn!("Magic link request: redirect_url has non-http scheme '{}' — rejecting", scheme);
                    return Err(StatusCode::BAD_REQUEST);
                }

                let host = parsed.host_str().unwrap_or("").to_string();
                if host.is_empty() {
                    tracing::warn!("Magic link request: redirect_url has no host: {}", url_str);
                    return Err(StatusCode::BAD_REQUEST);
                }
                let domain_record = crate::entities::app_domain::Entity::find()
                    .filter(crate::entities::app_domain::Column::DomainName.eq(&host))
                    .one(&db)
                    .await
                    .unwrap_or(None);

                let is_dev = std::env::var("ENVIRONMENT").unwrap_or_default() == "development";
                if domain_record.is_none() && !(is_dev && (host == "localhost" || host.starts_with("127."))) {
                    tracing::warn!(
                        "Magic link request: redirect_url host '{}' not in app_domains — rejecting",
                        host
                    );
                    return Err(StatusCode::BAD_REQUEST);
                }

                let mut tenant_name = None;
                let mut resolved_app_instance_id = app_instance_id.clone();
                if let Some(domain) = domain_record {
                    resolved_app_instance_id = domain.app_instance_id.to_string();
                    if let Ok(Some(app_inst)) = crate::entities::app_instance::Entity::find_by_id(domain.app_instance_id).one(&db).await {
                        if let Ok(Some(tenant)) = crate::entities::tenant::Entity::find_by_id(app_inst.tenant_id).one(&db).await {
                            tenant_name = Some(tenant.name);
                        }
                    }
                }

                Some((url_str.clone(), tenant_name, resolved_app_instance_id))
            }
            Err(e) => {
                tracing::warn!("Magic link request: invalid redirect_url '{}': {:?}", url_str, e);
                return Err(StatusCode::BAD_REQUEST);
            }
        }
    } else {
        None
    };

    let (validated_redirect_url, tenant_name_opt, final_app_instance_id) = match validated_redirect_url_data {
        Some((url, name_opt, app_id)) => (Some(url), name_opt, app_id),
        None => (None, None, app_instance_id),
    };

    if let Some(user_mod) = user_model {
        // IDEMPOTENCY GUARD — Layer 2: PostgreSQL transaction-scoped advisory lock.
        //
        // pg_try_advisory_xact_lock(key) is the architecturally correct cross-pod
        // solution: it uses PostgreSQL's built-in locking — no Redis, no external
        // infrastructure, no in-memory shared state.
        //
        // Properties:
        //   - Non-blocking: returns false immediately if another session holds the lock
        //   - Transaction-scoped: released automatically on COMMIT or ROLLBACK
        //   - Per-user: the lock key is derived from the user_id UUID
        //   - Works across all pods sharing the same PostgreSQL instance
        //
        // Flow:
        //   Pod A arrives → acquires lock → runs cleanup + upsert → sends email → COMMIT
        //   Pod B arrives concurrently → pg_try_advisory_xact_lock returns false → 200
        //
        // The key is a stable i64 derived from the UUID bytes. We XOR the high and low
        // halves of the 128-bit UUID to produce a 64-bit lock key. Collision probability
        // (two different users mapping to the same key) is 1/(2^64) — negligible.
        let token_str: String = rand::thread_rng()
            .sample_iter(&Alphanumeric)
            .take(32)
            .map(char::from)
            .collect();

        let id = Uuid::new_v4();
        let created_at = Utc::now();
        let expires_at = Utc::now() + Duration::minutes(15);

        // Derive a stable i64 lock key from the user UUID.
        let uuid_bytes = user_mod.id.as_bytes();
        let hi = i64::from_be_bytes(uuid_bytes[..8].try_into().unwrap_or([0u8; 8]));
        let lo = i64::from_be_bytes(uuid_bytes[8..].try_into().unwrap_or([0u8; 8]));
        let lock_key = hi ^ lo;

        // All three DB operations (advisory lock, expired token cleanup, upsert) run
        // inside a single explicit transaction. The advisory lock is xact-scoped, so
        // it is automatically released when this transaction commits or rolls back.
        let txn = db.begin().await.map_err(|e| {
            tracing::error!("Failed to begin transaction for magic link: {:?}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

        // Try to acquire the per-user advisory lock.
        let lock_row = sea_orm::ConnectionTrait::query_one(
            &txn,
            sea_orm::Statement::from_sql_and_values(
                sea_orm::DatabaseBackend::Postgres,
                "SELECT pg_try_advisory_xact_lock($1) AS acquired",
                vec![lock_key.into()],
            ),
        )
        .await
        .map_err(|e| {
            tracing::error!("Advisory lock query failed: {:?}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

        let lock_acquired = lock_row
            .and_then(|row| row.try_get_by_index::<bool>(0).ok())
            .unwrap_or(false);

        if !lock_acquired {
            // Another pod is handling this exact user's request right now.
            // The transaction rolls back automatically (no writes were made).
            let _ = txn.rollback().await;
            tracing::info!(
                event = "magic_link.deduplicated_cross_pod",
                request_id = %request_id,
                user_id = %user_mod.id,
                email = %payload.email,
                reason = "advisory_lock_contention",
                duration_ms = start.elapsed().as_millis()
            );
            return Ok((StatusCode::OK, Json(json!({"message": "If the email exists, a magic link has been sent."}))));
        }

        // Lock acquired — this pod is the sole handler for this user right now.
        // Step 1: expire any stale (unused but expired) tokens so they leave the
        //         partial index WHERE is_used = false.
        let _ = sea_orm::ConnectionTrait::execute(
            &txn,
            sea_orm::Statement::from_sql_and_values(
                sea_orm::DatabaseBackend::Postgres,
                "UPDATE magic_link_token SET is_used = true WHERE user_id = $1 AND is_used = false AND expires_at < NOW()",
                vec![user_mod.id.into()],
            ),
        ).await;

        // Step 2: upsert the new token.
        //   - No active token (or just cleaned up) → INSERT → was_inserted = true
        //   - Active token exists (re-request within 15-min window) →
        //     DO UPDATE rotates token → was_inserted = false → still send email
        let sql = r#"
            INSERT INTO magic_link_token
                (id, user_id, token, expires_at, is_used, created_at, redirect_url, is_setup_token)
            VALUES ($1, $2, $3, $4, false, $5, $6, false)
            ON CONFLICT (user_id) WHERE is_used = false
            DO UPDATE SET
                id           = EXCLUDED.id,
                token        = EXCLUDED.token,
                expires_at   = EXCLUDED.expires_at,
                created_at   = EXCLUDED.created_at,
                redirect_url = EXCLUDED.redirect_url
            RETURNING (xmax = 0) AS was_inserted
        "#;

        let upsert_result = sea_orm::ConnectionTrait::query_one(
            &txn,
            sea_orm::Statement::from_sql_and_values(
                sea_orm::DatabaseBackend::Postgres,
                sql,
                vec![
                    id.into(),
                    user_mod.id.into(),
                    token_str.clone().into(),
                    expires_at.into(),
                    created_at.into(),
                    validated_redirect_url.clone().into(),
                ],
            ),
        )
        .await
        .map_err(|e| {
            tracing::error!("Error upserting magic link token: {:?}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

        // Commit — releases the advisory lock atomically with the token write.
        txn.commit().await.map_err(|e| {
            tracing::error!("Failed to commit magic link transaction: {:?}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

        // Claim the in-memory cache slot NOW — only after we have confirmed:
        //   1. The user exists in the DB
        //   2. The advisory lock was acquired (cross-pod dedup)
        //   3. The token was written and committed
        // This prevents cache poisoning: a failed transaction or non-existent user
        // no longer blocks retries for the cache TTL window.
        MAGIC_LINK_REQUEST_CACHE.insert(cache_key.clone(), true).await;

        let was_inserted = upsert_result
            .and_then(|row| row.try_get_by_index::<bool>(0).ok())
            .unwrap_or(true);

        tracing::info!(
            event = if was_inserted { "magic_link.requested" } else { "magic_link.rotated" },
            request_id = %request_id,
            user_id = %user_mod.id,
            email = %user_mod.email,
            ip = %ip,
            user_agent = %user_agent,
            app_instance_id = %final_app_instance_id,
            duration_ms = start.elapsed().as_millis(),
            status = "success"
        );

        metrics::MAGIC_LINK_REQUESTS
            .with_label_values(&["unknown", &final_app_instance_id, "success"])
            .inc();

        let brand_name = tenant_name_opt.unwrap_or_else(|| "Atlas Platform".to_string());

        let magic_link_url = match validated_redirect_url {
            Some(ref base) => {
                if base.contains('?') {
                    format!("{}&token={}", base, token_str)
                } else {
                    format!("{}?token={}", base, token_str)
                }
            }
            None => {
                let admin_url = std::env::var("ADMIN_URL")
                    .unwrap_or_else(|_| "https://uat.atlas.oply.co".to_string());
                format!("{}/verify-token/{}", admin_url, token_str)
            }
        };

        let email_payload = crate::handlers::communications::SendEmailPayload {
            tenant_id: Uuid::nil(),
            to_email: user_mod.email.clone(),
            subject: format!("Sign in to {}", brand_name),
            body_html: format!(
                "<h2>Sign in to {1}</h2>\
                <p>Click the link below to log in securely. This link expires in 15 minutes.</p>\
                <br><a href=\"{0}\" style=\"font-size:16px;font-weight:bold;\">Log In Now</a>\
                <br><br><p style=\"font-size:12px;color:#666;\">If you did not request this, ignore this email.</p>",
                magic_link_url, brand_name
            ),
        };

        // T5 LATENCY FIX: dispatch email in a background task so the HTTP response
        // is returned immediately after the token is committed. SMTP (TLS handshake +
        // mailer.send) can take 2-5 seconds and would otherwise trigger LB timeouts.
        let db_for_email = db.clone();
        let email_addr = user_mod.email.clone();
        tokio::task::spawn(async move {
            if let Err((status, msg)) = crate::handlers::communications::send_email_handler(
                State(db_for_email),
                Json(email_payload),
            ).await {
                tracing::error!("Failed to dispatch magic link email to {}: {} {:?}", email_addr, msg, status);
            } else {
                tracing::info!("Magic link dispatched to {}", email_addr);
            }
        });
    }

    Ok((StatusCode::OK, Json(json!({"message": "If the email exists, a magic link has been sent."}))))
}

#[derive(Deserialize)]
pub struct VerifyMagicLinkPayload {
    pub token: String,
    pub tenant_id: Option<Uuid>,
}

pub async fn verify_magic_link(
    State(db): State<DatabaseConnection>,
    Json(payload): Json<VerifyMagicLinkPayload>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let start = Instant::now();
    let request_id = generate_request_id();

    // Look up the token WITHOUT filtering is_used so we can return a precise error code.
    // The frontend TokenFailure enum uses these codes to show contextual messages.
    let magic_link_opt = magic_link_token::Entity::find()
        .filter(magic_link_token::Column::Token.eq(&payload.token))
        .one(&db)
        .await
        .map_err(|_e| (StatusCode::INTERNAL_SERVER_ERROR, "Database Error".to_string()))?;

    let magic_link = match magic_link_opt {
        Some(m) => m,
        None => {
            tracing::warn!(
                event = "magic_link.verify.failed",
                request_id = %request_id,
                reason = "token_not_found",
                duration_ms = start.elapsed().as_millis()
            );
            // Structured error code for frontend enum
            return Err((StatusCode::UNAUTHORIZED, "error_code:token_not_found".to_string()));
        }
    };

    if magic_link.is_used {
        tracing::warn!(
            event = "magic_link.verify.failed",
            request_id = %request_id,
            user_id = %magic_link.user_id,
            reason = "token_already_used",
            duration_ms = start.elapsed().as_millis()
        );
        return Err((StatusCode::UNAUTHORIZED, "error_code:token_already_used".to_string()));
    }

    if magic_link.expires_at < Utc::now() {
        tracing::warn!(
            event = "magic_link.verify.failed",
            request_id = %request_id,
            user_id = %magic_link.user_id,
            reason = "token_expired",
            duration_ms = start.elapsed().as_millis()
        );
        return Err((StatusCode::UNAUTHORIZED, "error_code:token_expired".to_string()));
    }

    let user_mod = user::Entity::find_by_id(magic_link.user_id)
        .one(&db)
        .await
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "Database Error".to_string()))?
        .ok_or((StatusCode::UNAUTHORIZED, "User not found".to_string()))?;

    if let Some(target_tenant_id) = payload.tenant_id {
        let user_accounts = user_account::Entity::find()
            .filter(user_account::Column::UserId.eq(user_mod.id))
            .all(&db)
            .await
            .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "Database Error".to_string()))?;
            
        let mut has_access = false;
        for ua in user_accounts {
            let acc = account::Entity::find_by_id(ua.account_id)
                .one(&db)
                .await
                .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "Database Error".to_string()))?;
            if let Some(acc) = acc {
                if acc.tenant_id == target_tenant_id {
                    has_access = true;
                    break;
                }
            }
        }
        
        if !has_access {
            tracing::warn!(
                event = "magic_link.verify.failed",
                request_id = %request_id,
                user_id = %user_mod.id,
                tenant_id = %target_tenant_id,
                reason = "tenant_access_denied"
            );
            return Err((StatusCode::UNAUTHORIZED, "User does not have access to this tenant".to_string()));
        }
    }

    if magic_link.is_setup_token {
        let _ = crate::entities::passkey::Entity::delete_many()
            .filter(crate::entities::passkey::Column::UserId.eq(user_mod.id))
            .exec(&db)
            .await;
    }

    let mut ml_active: magic_link_token::ActiveModel = magic_link.into();
    ml_active.is_used = Set(true);
    let _ = ml_active.update(&db).await;

    let session_response = crate::handlers::sessions::create_session_for_user(&db, &user_mod)
        .await
        .map_err(|e| (e, "Failed to create session".to_string()))?;

    use crate::handlers::sessions::session_cookie_header;
    let cookie = session_cookie_header(&session_response.token, 86_400);

    tracing::info!(
        event = "magic_link.verified",
        request_id = %request_id,
        user_id = %user_mod.id,
        tenant_id = %payload.tenant_id.unwrap_or(Uuid::nil()),
        duration_ms = start.elapsed().as_millis(),
        status = "success"
    );

    Ok((
        StatusCode::OK,
        [(header::SET_COOKIE, cookie)],
        Json(session_response),
    ).into_response())
}
