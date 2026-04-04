use axum::{
    extract::{State, Json},
    http::StatusCode,
    routing::post,
    Router,
};
use sea_orm::{DatabaseConnection, EntityTrait, QueryFilter, ColumnTrait};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use lettre::message::{header, MultiPart, SinglePart};
use lettre::transport::smtp::authentication::Credentials;
use lettre::{AsyncSmtpTransport, AsyncTransport, Message, Tokio1Executor};
use crate::entities::tenant_setting::{self, Entity as TenantSetting};

#[derive(Deserialize, Debug)]
pub struct SendEmailPayload {
    pub tenant_id: Uuid,
    pub to_email: String,
    pub subject: String,
    pub body_html: String,
}

#[derive(Serialize)]
pub struct SendEmailResponse {
    pub message: String,
}

pub fn authenticated_routes(db: DatabaseConnection) -> Router<DatabaseConnection> {
    Router::new()
        .route("/api/communications/email", post(send_email_handler))
        .with_state(db)
}

pub async fn send_email_handler(
    State(db): State<DatabaseConnection>,
    Json(payload): Json<SendEmailPayload>,
) -> Result<(StatusCode, Json<SendEmailResponse>), (StatusCode, String)> {
    
    // 1. Fetch Tenant Settings for SMTP override
    let settings = TenantSetting::find()
        .filter(tenant_setting::Column::TenantId.eq(payload.tenant_id))
        .all(&db)
        .await
        .map_err(|e| {
            tracing::error!("DB error fetching tenant settings: {:?}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, "Database error".to_string())
        })?;

    let mut custom_host = None;
    let mut custom_port = 587;
    let mut custom_username = None;
    let mut custom_token = None;
    let mut custom_from = None;

    for setting in settings {
        match setting.key.as_str() {
            "smtp_host" => custom_host = Some(setting.value),
            "smtp_port" => custom_port = setting.value.parse().unwrap_or(587),
            "smtp_username" => custom_username = Some(setting.value),
            "smtp_token" => custom_token = Some(setting.value),
            "smtp_from" => custom_from = Some(setting.value),
            _ => {}
        }
    }

    // 2. Fallback to System environment variables if no Custom settings
    let host = custom_host.unwrap_or_else(|| std::env::var("SMTP_HOST").unwrap_or_else(|_| "localhost".to_string()));
    let port = custom_port;
    let username = custom_username.unwrap_or_else(|| std::env::var("SMTP_USERNAME").unwrap_or_default());
    let token = custom_token.unwrap_or_else(|| std::env::var("SMTP_TOKEN").unwrap_or_default());
    let from_email = custom_from.unwrap_or_else(|| std::env::var("SMTP_FROM").unwrap_or_else(|_| "noreply@atlas-platform.local".to_string()));

    // 3. Construct Message
    let email = Message::builder()
        .from(from_email.parse().map_err(|_| (StatusCode::BAD_REQUEST, "Invalid FROM email".to_string()))?)
        .to(payload.to_email.parse().map_err(|_| (StatusCode::BAD_REQUEST, "Invalid TO email".to_string()))?)
        .subject(&payload.subject)
        .multipart(
            MultiPart::alternative().singlepart(
                SinglePart::builder()
                    .header(header::ContentType::TEXT_HTML)
                    .body(payload.body_html),
            ),
        )
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "Failed to build email message".to_string()))?;

    // If we're mocking local sending
    if host == "localhost" || host.is_empty() {
        tracing::warn!("SMTP Host not configured. Mocking email send to: {}", payload.to_email);
        return Ok((StatusCode::OK, Json(SendEmailResponse { message: "Email mocked successfully".to_string() })));
    }

    let creds = Credentials::new(username, token);

    let mailer: AsyncSmtpTransport<Tokio1Executor> = if port == 465 {
        AsyncSmtpTransport::<Tokio1Executor>::relay(&host)
            .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "Invalid SMTP relay host".to_string()))?
            .port(port)
            .credentials(creds)
            .build()
    } else {
        AsyncSmtpTransport::<Tokio1Executor>::starttls_relay(&host)
            .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "Invalid SMTP STARTTLS host".to_string()))?
            .port(port)
            .credentials(creds)
            .build()
    };

    match mailer.send(email).await {
        Ok(_) => {
            tracing::info!("Email sent successfully to {}", payload.to_email);
            Ok((StatusCode::OK, Json(SendEmailResponse { message: "Email sent successfully".to_string() })))
        }
        Err(e) => {
            tracing::error!("Failed to send email to {}: {:?}", payload.to_email, e);
            Err((StatusCode::INTERNAL_SERVER_ERROR, "Failed to send email over SMTP".to_string()))
        }
    }
}
