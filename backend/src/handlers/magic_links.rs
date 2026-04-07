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

    let new_token_record = match crate::services::auth_service::AuthService::create_magic_link(&db, &req.email).await {
        Ok(t) => t,
        Err((status, message)) => {
            if status == StatusCode::NOT_FOUND {
                // Silently succeed to prevent enumeration
                return Ok((StatusCode::OK, Json(json!({ "message": "If the email is registered, a magic link has been sent." }))));
            } else {
                return Err((status, Json(json!({ "message": message }))));
            }
        }
    };
    
    let token = new_token_record.token;

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
    let user_record = crate::services::auth_service::AuthService::verify_magic_link(&db, &req.token)
        .await
        .map_err(|(status, msg)| (status, Json(json!({ "message": msg }))))?;

    // Create session (this also naturally serves as the verification step completion loop)
    let session_response = create_passwordless_session(&db, &user_record.email)
        .await
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "message": "Failed to establish session" }))))?;

    Ok((StatusCode::OK, Json(session_response)))
}
