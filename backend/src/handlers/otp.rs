// backend/src/handlers/otp.rs
//
// Inline OTP authentication for the onboarding wizard pre-step.
//
// Two public (no-auth) endpoints:
//   POST /api/auth/otp/send   — find-or-create user, generate 6-digit code, send email
//   POST /api/auth/otp/verify — validate code, create session (same as magic link)
//
// Design notes:
//   - Codes are 6-digit numeric (familiar UX, works with iOS/Android autofill)
//   - Stored as SHA-256 hash — no plaintext in DB
//   - 5-minute TTL, single-use
//   - Max 3 sends per email per 10 minutes (rate limiter)
//   - find-or-create user: direct mail recipients won't have an account yet;
//     we create a stub user and enrich it during the wizard steps

use axum::{
    extract::{Json, State},
    http::{StatusCode, header},
    response::{IntoResponse, Response},
    Extension,
    Router,
    routing::post,
};
use axum::http::HeaderMap;
use chrono::{Duration, Utc};
use sea_orm::{ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter,
              QueryOrder, Order, Set};
use serde::{Deserialize, Serialize};
use serde_json::json;
use sha2::{Sha256, Digest};
use uuid::Uuid;
use std::env;
use lettre::{SmtpTransport, Transport, Message, transport::smtp::authentication::Credentials};
use lettre::message::{header as mail_header, MultiPart, SinglePart};

use crate::entities::user;
use crate::middleware::rate_limiter::RateLimiter;

// ── Route registration ─────────────────────────────────────────────────────────

pub fn public_routes() -> Router<DatabaseConnection> {
    Router::new()
        .route("/api/auth/otp/send",   post(send_otp))
        .route("/api/auth/otp/verify", post(verify_otp))
}

// ── Request / response types ───────────────────────────────────────────────────

#[derive(Deserialize)]
pub struct SendOtpRequest {
    pub email: String,
}

#[derive(Deserialize)]
pub struct VerifyOtpRequest {
    pub email: String,
    pub code:  String,
}

// ── Helpers ────────────────────────────────────────────────────────────────────

fn hash_code(code: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(code.as_bytes());
    format!("{:x}", hasher.finalize())
}

fn generate_otp() -> String {
    use std::time::SystemTime;
    // Use time-seeded modulo — sufficient for a 5-minute single-use code.
    // For production, swap to `rand::Rng` crate if added as dependency.
    let nanos = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap_or_default()
        .subsec_nanos();
    // Additional entropy from a UUID to avoid clock collisions in fast tests
    let extra = Uuid::new_v4().as_u128() as u32;
    let code = ((nanos ^ extra) % 900_000) + 100_000; // 100000..=999999
    format!("{:06}", code)
}

async fn find_or_create_user(
    db: &DatabaseConnection,
    email: &str,
) -> Result<user::Model, StatusCode> {
    // Try to find existing user
    if let Some(u) = user::Entity::find()
        .filter(user::Column::Email.eq(email))
        .one(db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    {
        return Ok(u);
    }

    // Create stub user — wizard steps will enrich first_name, last_name, phone
    let now = Utc::now();
    let stub = user::ActiveModel {
        id:            Set(Uuid::new_v4()),
        email:         Set(email.to_string()),
        username:      Set(email.to_string()),
        first_name:    Set(String::new()),
        last_name:     Set(String::new()),
        phone:         Set(String::new()),
        password_hash: Set(String::new()),
        is_active:     Set(true),
        created_at:    Set(now),
        updated_at:    Set(now),
        ..Default::default()
    };

    stub.insert(db).await.map_err(|e| {
        tracing::error!("otp: failed to create stub user for {}: {:?}", email, e);
        StatusCode::INTERNAL_SERVER_ERROR
    })
}

async fn store_otp(
    db: &DatabaseConnection,
    user_id: Uuid,
    code: &str,
) -> Result<(), StatusCode> {
    let expires_at = Utc::now() + Duration::minutes(5);
    db.execute(sea_orm::Statement::from_sql_and_values(
        sea_orm::DatabaseBackend::Postgres,
        "INSERT INTO atlas_otp_tokens (id, user_id, code_hash, expires_at, is_used, created_at) \
         VALUES ($1, $2, $3, $4, false, NOW())",
        [
            Uuid::new_v4().into(),
            user_id.into(),
            hash_code(code).into(),
            expires_at.into(),
        ],
    ))
    .await
    .map(|_| ())
    .map_err(|e| {
        tracing::error!("otp: failed to store token: {:?}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })
}

fn send_otp_email(to_email: &str, code: &str) {
    let smtp_server   = env::var("SMTP_SERVER").unwrap_or_default();
    let smtp_username = env::var("SMTP_USERNAME").unwrap_or_default();
    let smtp_token    = env::var("SMTP_TOKEN").unwrap_or_default();
    let smtp_port     = env::var("SMTP_PORT").unwrap_or("587".to_string())
                            .parse::<u16>().unwrap_or(587);
    let smtp_from     = env::var("SMTP_FROM")
                            .unwrap_or_else(|_| "noreply@folio.app".to_string());

    // Format as "384 921" — two groups of 3 for easy reading on phone screen
    let display_code = format!("{} {}", &code[..3], &code[3..]);

    let html_body = format!(
        r#"<div style="font-family:Inter,sans-serif;max-width:480px;margin:0 auto;padding:32px 24px;">
  <p style="font-size:28px;font-weight:800;color:#0f172a;margin:0 0 8px;">
    Your verification code
  </p>
  <p style="font-size:14px;color:#64748b;margin:0 0 32px;">
    Enter this code in the Folio app to continue your sign-up.
    It expires in 5 minutes.
  </p>
  <div style="background:#f8fafc;border:1px solid #e2e8f0;border-radius:16px;padding:32px;
              text-align:center;margin-bottom:24px;">
    <span style="font-size:48px;font-weight:900;letter-spacing:0.12em;color:#6366f1;
                 font-family:monospace;">{display_code}</span>
  </div>
  <p style="font-size:12px;color:#94a3b8;margin:0;">
    If you didn't request this code, you can safely ignore this email.
  </p>
</div>"#,
        display_code = display_code,
    );

    let email_to   = to_email.to_string();
    let plain_body = format!("Your Folio verification code is: {display_code}\n\nExpires in 5 minutes.");

    let msg = match Message::builder()
        .from(smtp_from.parse().unwrap_or_else(|_| "noreply@folio.app".parse().unwrap()))
        .to(match email_to.parse() { Ok(m) => m, Err(_) => return })
        .subject("Your Folio sign-up code")
        .multipart(
            MultiPart::alternative()
                .singlepart(SinglePart::builder()
                    .header(mail_header::ContentType::TEXT_PLAIN)
                    .body(plain_body))
                .singlepart(SinglePart::builder()
                    .header(mail_header::ContentType::TEXT_HTML)
                    .body(html_body))
        ) {
            Ok(m) => m,
            Err(e) => { tracing::error!("otp: email build error: {:?}", e); return; }
        };

    let creds  = Credentials::new(smtp_username, smtp_token);
    let mailer = match SmtpTransport::relay(&smtp_server) {
        Ok(b) => b.port(smtp_port).credentials(creds).build(),
        Err(e) => { tracing::error!("otp: SMTP relay error: {:?}", e); return; }
    };

    tokio::task::spawn_blocking(move || {
        match mailer.send(&msg) {
            Ok(_)  => tracing::info!("otp: email sent to {}", email_to),
            Err(e) => tracing::error!("otp: email send failed: {:?}", e),
        }
    });
}

// ── Handlers ───────────────────────────────────────────────────────────────────

/// POST /api/auth/otp/send
///
/// Find-or-create user, generate 6-digit OTP, store hash, send email.
/// Rate-limited: 3 attempts per email per 10 minutes.
async fn send_otp(
    State(db):   State<DatabaseConnection>,
    Extension(rate_limiter): Extension<RateLimiter>,
    headers:     HeaderMap,
    Json(req):   Json<SendOtpRequest>,
) -> impl IntoResponse {
    // Rate limit by email address
    let ip = headers
        .get("x-forwarded-for")
        .and_then(|v| v.to_str().ok())
        .and_then(|s| s.split(',').next())
        .unwrap_or("unknown")
        .trim()
        .to_string();

    if rate_limiter.check_rate_limit(&format!("otp:send:{}", req.email.to_lowercase())).await.is_err() {
        return (StatusCode::TOO_MANY_REQUESTS, Json(json!({
            "error": "Too many code requests. Please wait a few minutes."
        }))).into_response();
    }

    let email = req.email.trim().to_lowercase();
    if email.is_empty() || !email.contains('@') {
        return (StatusCode::BAD_REQUEST, Json(json!({ "error": "Invalid email" }))).into_response();
    }

    let user = match find_or_create_user(&db, &email).await {
        Ok(u) => u,
        Err(s) => return (s, Json(json!({ "error": "Could not create account" }))).into_response(),
    };

    let code = generate_otp();

    if let Err(s) = store_otp(&db, user.id, &code).await {
        return (s, Json(json!({ "error": "Could not generate code" }))).into_response();
    }

    // Dispatch email asynchronously — fire and forget
    send_otp_email(&email, &code);

    tracing::info!(event = "otp.sent", user_id = %user.id, ip = %ip);

    (StatusCode::OK, Json(json!({ "message": "Code sent" }))).into_response()
}

/// POST /api/auth/otp/verify
///
/// Validate code, mark used, create session (same cookie as magic link).
async fn verify_otp(
    State(db): State<DatabaseConnection>,
    Json(req): Json<VerifyOtpRequest>,
) -> Result<Response, (StatusCode, Json<serde_json::Value>)> {
    let email    = req.email.trim().to_lowercase();
    let code_raw = req.code.trim().replace(' ', ""); // strip spaces ("384 921" → "384921")

    if code_raw.len() != 6 || code_raw.chars().any(|c| !c.is_ascii_digit()) {
        return Err((StatusCode::BAD_REQUEST, Json(json!({ "error": "Invalid code format" }))));
    }

    // Look up user
    let user = user::Entity::find()
        .filter(user::Column::Email.eq(&email))
        .one(&db)
        .await
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "error": "Database error" }))))?
        .ok_or_else(|| (StatusCode::UNAUTHORIZED, Json(json!({ "error": "Invalid code" }))))?;

    // Find the latest unused, unexpired token for this user
    let hash = hash_code(&code_raw);
    let now  = Utc::now();

    let row = db.query_one(sea_orm::Statement::from_sql_and_values(
        sea_orm::DatabaseBackend::Postgres,
        "SELECT id FROM atlas_otp_tokens \
         WHERE user_id = $1 AND code_hash = $2 AND is_used = false AND expires_at > $3 \
         ORDER BY created_at DESC LIMIT 1",
        [user.id.into(), hash.into(), now.into()],
    ))
    .await
    .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "error": "Database error" }))))?;

    let token_id: Uuid = match row {
        Some(r) => r.try_get("", "id")
            .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "error": "Database error" }))))?,
        None => return Err((StatusCode::UNAUTHORIZED, Json(json!({ "error": "Invalid or expired code" })))),
    };

    // Mark token used
    let _ = db.execute(sea_orm::Statement::from_sql_and_values(
        sea_orm::DatabaseBackend::Postgres,
        "UPDATE atlas_otp_tokens SET is_used = true WHERE id = $1",
        [token_id.into()],
    )).await;

    // Create session — identical path as magic link verify
    let session_response = crate::handlers::sessions::create_session_for_user(&db, &user)
        .await
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "error": "Failed to create session" }))))?;

    use crate::handlers::sessions::session_cookie_header;
    let cookie = session_cookie_header(&session_response.token, 86_400);

    tracing::info!(
        event = "otp.verified",
        user_id = %user.id,
    );

    Ok((
        StatusCode::OK,
        [(header::SET_COOKIE, cookie)],
        Json(session_response),
    ).into_response())
}
