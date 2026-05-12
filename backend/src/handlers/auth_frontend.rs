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
use chrono::{Utc, Duration};
use rand::{distributions::Alphanumeric, Rng};

use crate::entities::{user, account, user_account, magic_link_token};
use crate::auth::verify_password;
use crate::handlers::sessions::create_user_session;

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

    Ok((StatusCode::OK, Json(AuthFlowResponse { has_passkey })))
}

pub async fn verify_email(
    State(_db): State<DatabaseConnection>,
    Query(_query): Query<VerifyEmailQuery>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    // Stub for email verification logic
    // We would look up the user by the verification token and set is_active = true
    Ok((StatusCode::OK, Json(json!({"message": "Email verified successfully"}))))
}

pub async fn login(
    State(db): State<DatabaseConnection>,
    Json(credentials): Json<LoginCredentials>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let user = user::Entity::find()
        .filter(user::Column::Email.eq(&credentials.email))
        .one(&db)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .ok_or((StatusCode::UNAUTHORIZED, "Invalid credentials".to_string()))?;

    let is_valid = verify_password(&credentials.password, &user.password_hash)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    if !is_valid {
        return Err((StatusCode::UNAUTHORIZED, "Invalid credentials".to_string()));
    }

    if !user.is_active {
        return Err((StatusCode::UNAUTHORIZED, "Account is inactive. Please verify your email.".to_string()));
    }

    let session_response = create_user_session(&db, &credentials.email, &credentials.password)
        .await
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "Failed to create session".to_string()))?;

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
        // is_admin is not stored on the user entity (removed during RBAC migration).
        // Admin claims are embedded in the session JWT and read from session.is_admin.
    })))
}

pub async fn webauthn_register_start() -> Result<impl IntoResponse, StatusCode> {
    // Stub for WebAuthn registration
    Ok((StatusCode::NOT_IMPLEMENTED, "WebAuthn not fully implemented yet"))
}

pub async fn webauthn_auth_start() -> Result<impl IntoResponse, StatusCode> {
    // Stub for WebAuthn authentication
    Ok((StatusCode::NOT_IMPLEMENTED, "WebAuthn not fully implemented yet"))
}

#[derive(Deserialize)]
pub struct RequestMagicLinkPayload {
    pub email: String,
    /// Optional callback URL the magic link email should direct the user to.
    /// Must be a URL whose host is registered in `app_domains`.
    /// When absent, falls back to `ADMIN_URL/verify-token/{token}` (platform-admin flow).
    pub redirect_url: Option<String>,
}

pub async fn request_magic_link(
    State(db): State<DatabaseConnection>,
    Extension(rate_limiter): Extension<crate::middleware::rate_limiter::RateLimiter>,
    headers: axum::http::HeaderMap,
    Json(payload): Json<RequestMagicLinkPayload>,
) -> Result<(StatusCode, Json<serde_json::Value>), StatusCode> {
    let ip_address = headers
        .get("x-forwarded-for")
        .and_then(|h| h.to_str().ok())
        .unwrap_or("Unknown")
        .to_string();

    if let Err(status) = rate_limiter.check_auth_rate_limit(&ip_address, &payload.email).await {
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

    // ── Validate redirect_url against app_domains (open-redirect prevention) ──
    // Parse the host from the supplied redirect_url and confirm it exists in
    // app_domains. Unknown hosts get a 400 — no information about the user is
    // leaked at this point (validation happens before the user lookup result
    // is acted upon, but after a generic check so the response timing is uniform).
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

                // Fetch tenant branding if domain is registered
                let mut tenant_name = None;
                if let Some(domain) = domain_record {
                    if let Ok(Some(app_inst)) = crate::entities::app_instance::Entity::find_by_id(domain.app_instance_id).one(&db).await {
                        if let Ok(Some(tenant)) = crate::entities::tenant::Entity::find_by_id(app_inst.tenant_id).one(&db).await {
                            tenant_name = Some(tenant.name);
                        }
                    }
                }

                tracing::info!("Magic link request: redirect_url '{}' validated against app_domains (or dev bypass)", url_str);
                Some((url_str.clone(), tenant_name))
            }
            Err(e) => {
                tracing::warn!("Magic link request: invalid redirect_url '{}': {:?}", url_str, e);
                return Err(StatusCode::BAD_REQUEST);
            }
        }
    } else {
        None
    };

    let (validated_redirect_url, tenant_name_opt) = match validated_redirect_url_data {
        Some((url, name_opt)) => (Some(url), name_opt),
        None => (None, None),
    };

    if let Some(user_mod) = user_model {
        // ── Idempotency guard ────────────────────────────────────────────────
        // If a valid, unused token for this user already exists that was created
        // within the last 60 seconds, reuse it rather than inserting a second token
        // and firing another email.  This prevents duplicate emails when two HTTP
        // requests reach the handler from the same user action (e.g. a network
        // retry, an in-flight browser double-dispatch before the countdown UI
        // kicks in, or a load-balancer quirk).
        use sea_orm::QueryOrder;
        let recent_cutoff = Utc::now() - Duration::seconds(60);
        let existing_token = magic_link_token::Entity::find()
            .filter(magic_link_token::Column::UserId.eq(user_mod.id))
            .filter(magic_link_token::Column::IsUsed.eq(false))
            .filter(magic_link_token::Column::CreatedAt.gte(recent_cutoff))
            .filter(magic_link_token::Column::ExpiresAt.gt(Utc::now()))
            .order_by_desc(magic_link_token::Column::CreatedAt)
            .one(&db)
            .await
            .unwrap_or(None);

        if let Some(existing) = existing_token {
            tracing::warn!(
                "Idempotency: skipping duplicate magic link for {} — token {}... created <60s ago is still valid",
                user_mod.email,
                &existing.token[..8]
            );
            // Return 200 — the email they already received is still valid.
            return Ok((StatusCode::OK, Json(json!({"message": "If the email exists, a magic link has been sent."}))));
        }
        // ── End idempotency guard ────────────────────────────────────────────

        let token_str: String = rand::thread_rng()
            .sample_iter(&Alphanumeric)
            .take(32)
            .map(char::from)
            .collect();

        let token_entity = magic_link_token::ActiveModel {
            id: Set(Uuid::new_v4()),
            user_id: Set(user_mod.id),
            token: Set(token_str.clone()),
            expires_at: Set(Utc::now() + Duration::minutes(15)),
            is_used: Set(false),
            created_at: Set(Utc::now()),
            redirect_url: Set(validated_redirect_url.clone()),
            is_setup_token: Set(false),
        };

        token_entity.insert(&db).await.map_err(|e| {
            tracing::error!("Error saving magic link: {:?}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

        // Build the magic link URL:
        //   - App-originated: {redirect_url}?token={token}  (e.g. https://uat.buildwithruud.com/admin?token=xxx)
        //   - Platform-admin:  {ADMIN_URL}/verify-token/{token}  (unchanged legacy behaviour)
        let magic_link_url = match validated_redirect_url {
            Some(ref base) => {
                // Append ?token= correctly whether base already has query params or not
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
        tracing::info!("Dispatching magic link to {} (token: {}...) → {}", user_mod.email, token_preview, magic_link_url);

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

    // Always return 200 — prevents email enumeration.
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
    let magic_link_opt = magic_link_token::Entity::find()
        .filter(magic_link_token::Column::Token.eq(&payload.token))
        .filter(magic_link_token::Column::IsUsed.eq(false))
        .one(&db)
        .await
        .map_err(|_e| (StatusCode::INTERNAL_SERVER_ERROR, "Database Error".to_string()))?;

    let magic_link = match magic_link_opt {
        Some(m) => m,
        None => return Err((StatusCode::UNAUTHORIZED, "Invalid or expired token".to_string())),
    };

    if magic_link.expires_at < Utc::now() {
        return Err((StatusCode::UNAUTHORIZED, "Token has expired".to_string()));
    }

    let user_mod = user::Entity::find_by_id(magic_link.user_id)
        .one(&db)
        .await
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "Database Error".to_string()))?
        .ok_or((StatusCode::UNAUTHORIZED, "User not found".to_string()))?;

    // Validate tenant access if tenant_id is provided
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
            return Err((StatusCode::UNAUTHORIZED, "User does not have access to this tenant".to_string()));
        }
    }

    // Mark as used
    // Only wipe passkeys for first-time setup tokens, not regular magic link logins.
    if magic_link.is_setup_token {
        let _ = crate::entities::passkey::Entity::delete_many()
            .filter(crate::entities::passkey::Column::UserId.eq(user_mod.id))
            .exec(&db)
            .await;
    }

    let mut ml_active: magic_link_token::ActiveModel = magic_link.into();
    ml_active.is_used = Set(true);
    let _ = ml_active.update(&db).await;

    // We can use `create_session_for_user`
    let session_response = crate::handlers::sessions::create_session_for_user(&db, &user_mod)
        .await
        .map_err(|e| (e, "Failed to create session".to_string()))?;

    // CRITICAL: SessionResponse.token is #[serde(skip_serializing)] so it is never
    // present in the JSON body. The frontend reads the Set-Cookie header — do NOT
    // change this to a JSON field without a security review.
    use crate::handlers::sessions::session_cookie_header;
    let cookie = session_cookie_header(&session_response.token, 86_400); // 24 h

    Ok((
        StatusCode::OK,
        [(header::SET_COOKIE, cookie)],
        Json(session_response),
    ).into_response())
}

