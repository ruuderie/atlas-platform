use axum::{
    extract::{Json, Query, State},
    http::StatusCode,
    response::IntoResponse,
    routing::post,
    Router,
};
use sea_orm::{ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, Set};
use serde::{Deserialize, Serialize};
use serde_json::json;
use uuid::Uuid;
use chrono::{Duration, Utc};
use lettre::{SmtpTransport, Transport, Message, transport::smtp::authentication::Credentials};
use lettre::message::{header, MultiPart, SinglePart};
use std::env;

use crate::entities::{user, magic_link_token};
use crate::models::session::SessionResponse;
use crate::handlers::sessions::create_passwordless_session;

#[derive(Deserialize)]
pub struct MagicLinkRequest {
    pub email: String,
}

#[derive(Deserialize)]
pub struct MagicLinkVerifyRequest {
    pub token: String,
}

pub fn public_routes() -> Router<DatabaseConnection> {
    Router::new()
        .route("/magic-links/request", post(request_magic_link))
        .route("/magic-links/verify", post(verify_magic_link))
}

pub async fn request_magic_link(
    State(db): State<DatabaseConnection>,
    Json(req): Json<MagicLinkRequest>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    tracing::info!("Received magic link request for email: {}", req.email);

    let user = user::Entity::find()
        .filter(user::Column::Email.eq(&req.email))
        .one(&db)
        .await
        .map_err(|e| {
            tracing::error!("Database error: {:?}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "message": "Database error" })))
        })?;

    let user = match user {
        Some(u) => u,
        None => {
            // Silently succeed to prevent email enumeration
            return Ok((StatusCode::OK, Json(json!({ "message": "If the email is registered, a magic link has been sent." }))));
        }
    };

    // Generate token
    let token = Uuid::new_v4().to_string();
    let expires_at = Utc::now() + Duration::minutes(15);

    let new_token = magic_link_token::ActiveModel {
        id: Set(Uuid::new_v4()),
        user_id: Set(user.id),
        token: Set(token.clone()),
        expires_at: Set(expires_at),
        is_used: Set(false),
        created_at: Set(Utc::now()),
    };

    new_token.insert(&db).await.map_err(|e| {
        tracing::error!("Failed to create token: {:?}", e);
        (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "message": "Failed to generate magic link" })))
    })?;

    // Prepare Email
    let smtp_server = env::var("SMTP_SERVER").unwrap_or_default();
    let smtp_username = env::var("SMTP_USERNAME").unwrap_or_default();
    let smtp_token = env::var("SMTP_TOKEN").unwrap_or_default();
    let smtp_port = env::var("SMTP_PORT").unwrap_or("587".to_string()).parse::<u16>().unwrap_or(587);
    let smtp_from = env::var("SMTP_FROM").unwrap_or_else(|_| "noreply@atlas.oply.co".to_string());
    
    // Determine frontend route securely
    let frontend_url = env::var("FRONTEND_URL").unwrap_or_else(|_| "https://network.uat.atlas.oply.co".to_string());
    let magic_link_url = format!("{}/magic-login?token={}", frontend_url, token);

    let email = Message::builder()
        .from(smtp_from.parse().unwrap())
        .to(req.email.parse().unwrap())
        .subject("Your Atlas Platform Magic Link")
        .multipart(
            MultiPart::alternative()
                .singlepart(
                    SinglePart::builder()
                        .header(header::ContentType::TEXT_PLAIN)
                        .body(format!("Click the following link to log in securely: {}", magic_link_url)),
                )
                .singlepart(
                    SinglePart::builder()
                        .header(header::ContentType::TEXT_HTML)
                        .body(format!(
                            "<p>Click the link below to securely log into your Atlas Platform account.</p>\
                            <p><a href=\"{0}\">Log In Now</a></p>\
                            <p><i>If you did not request this, please ignore this email.</i></p>",
                            magic_link_url
                        )),
                ),
        )
        .map_err(|e| {
            tracing::error!("Failed to build email: {:?}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "message": "Failed to compose email payload" })))
        })?;

    let creds = Credentials::new(smtp_username, smtp_token);
    let mailer = SmtpTransport::relay(&smtp_server)
        .unwrap()
        .port(smtp_port)
        .credentials(creds)
        .build();

    // Spawn email dispatch dynamically so it doesn't block the request wrapper
    tokio::task::spawn_blocking(move || {
        match mailer.send(&email) {
            Ok(_) => tracing::info!("Magic link email sent successfully"),
            Err(e) => tracing::error!("Could not send email out natively: {:?}", e),
        }
    });

    Ok((StatusCode::OK, Json(json!({ "message": "If the email is registered, a magic link has been sent." }))))
}

pub async fn verify_magic_link(
    State(db): State<DatabaseConnection>,
    Json(req): Json<MagicLinkVerifyRequest>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    let token_record = magic_link_token::Entity::find()
        .filter(magic_link_token::Column::Token.eq(&req.token))
        .filter(magic_link_token::Column::IsUsed.eq(false))
        .one(&db)
        .await
        .map_err(|e| {
            tracing::error!("Database error: {:?}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "message": "Database query error" })))
        })?;

    let token_record = match token_record {
        Some(t) => t,
        None => return Err((StatusCode::BAD_REQUEST, Json(json!({ "message": "Invalid or expired magic link" })))),
    };

    if token_record.expires_at < Utc::now() {
        return Err((StatusCode::BAD_REQUEST, Json(json!({ "message": "Magic link has expired" }))));
    }

    let user_record = user::Entity::find_by_id(token_record.user_id)
        .one(&db)
        .await
        .map_err(|e| {
            tracing::error!("Database error: {:?}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "message": "User query error" })))
        })?
        .ok_or((StatusCode::NOT_FOUND, Json(json!({ "message": "User not found" }))))?;

    // Mark token as used
    let mut updated_token: magic_link_token::ActiveModel = token_record.into();
    updated_token.is_used = Set(true);
    updated_token.update(&db).await.map_err(|e| {
        tracing::error!("Failed to update token: {:?}", e);
        (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "message": "Failed to consume token" })))
    })?;

    // Create session (this also naturally serves as the verification step completion loop)
    let session_response = create_passwordless_session(&db, &user_record.email)
        .await
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "message": "Failed to establish session" }))))?;

    Ok((StatusCode::OK, Json(session_response)))
}
