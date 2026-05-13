use axum::{
    extract::{State, Query, Extension, Json},
    http::{StatusCode, header},
    response::IntoResponse,
    routing::{get, post},
    Router,
};
use sea_orm::{DatabaseConnection, EntityTrait, QueryFilter, ColumnTrait, Set, ActiveModelTrait};
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
        let token_str: String = rand::thread_rng()
            .sample_iter(&Alphanumeric)
            .take(32)
            .map(char::from)
            .collect();

        let id = Uuid::new_v4();
        let created_at = Utc::now();
        let expires_at = Utc::now() + Duration::minutes(15);
        let sql = r#"
            INSERT INTO magic_link_token (id, user_id, token, expires_at, is_used, created_at, redirect_url, is_setup_token)
            VALUES ($1, $2, $3, $4, false, $5, $6, false)
            ON CONFLICT (user_id) WHERE is_used = false
            DO NOTHING
        "#;
        
        let insert_res = sea_orm::ConnectionTrait::execute(
            &db,
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
            tracing::error!("Error saving magic link: {:?}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

        if insert_res.rows_affected() == 0 {
            tracing::warn!(
                event = "magic_link.duplicate_prevented",
                request_id = %request_id,
                user_id = %user_mod.id,
                email = %user_mod.email,
                ip = %ip,
                duration_ms = start.elapsed().as_millis(),
                status = "blocked"
            );

            metrics::MAGIC_LINK_DUPLICATES_PREVENTED
                .with_label_values(&["unknown", &final_app_instance_id])
                .inc();

            return Ok((StatusCode::OK, Json(json!({"message": "If the email exists, a magic link has been sent."}))));
        }

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

        let token_preview = &token_str[..8];
        tracing::info!(
            event = "magic_link.requested",
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

        if let Err((status, msg)) = crate::handlers::communications::send_email_handler(
            State(db.clone()),
            Json(email_payload),
        ).await {
            tracing::error!("Failed to dispatch magic link email: {} {:?}", msg, status);
        } else {
            tracing::info!("Magic link dispatched to {}", user_mod.email);
        }
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

    let magic_link_opt = magic_link_token::Entity::find()
        .filter(magic_link_token::Column::Token.eq(&payload.token))
        .filter(magic_link_token::Column::IsUsed.eq(false))
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
            return Err((StatusCode::UNAUTHORIZED, "Invalid or expired token".to_string()));
        }
    };

    if magic_link.expires_at < Utc::now() {
        tracing::warn!(
            event = "magic_link.verify.failed",
            request_id = %request_id,
            user_id = %magic_link.user_id,
            reason = "token_expired",
            duration_ms = start.elapsed().as_millis()
            );
            return Err((StatusCode::UNAUTHORIZED, "Token has expired".to_string()));
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
