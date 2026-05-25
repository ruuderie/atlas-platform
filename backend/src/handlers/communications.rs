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

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct SendEmailPayload {
    pub tenant_id: Uuid,
    pub to_email: String,
    pub subject: String,
    pub body_html: String,
    #[serde(default)]
    pub attachments: Vec<String>,
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
    let mut custom_port = None;
    let mut custom_username = None;
    let mut custom_token = None;
    let mut custom_from = None;

    for setting in settings {
        match setting.key.as_str() {
            "smtp_server" => custom_host = Some(setting.value),
            "smtp_port" => custom_port = Some(setting.value.parse().unwrap_or(587)),
            "smtp_username" => custom_username = Some(setting.value),
            "smtp_token" => custom_token = Some(setting.value),
            "smtp_from" => custom_from = Some(setting.value),
            _ => {}
        }
    }

    // 2. Fallback to System environment variables if no Custom settings
    let host = custom_host.unwrap_or_else(|| std::env::var("SMTP_SERVER").unwrap_or_else(|_| "localhost".to_string()));
    let port = custom_port.unwrap_or_else(|| std::env::var("SMTP_PORT").unwrap_or_else(|_| "587".to_string()).parse().unwrap_or(587));
    let username = custom_username.unwrap_or_else(|| std::env::var("SMTP_USERNAME").unwrap_or_default());
    let token = custom_token.unwrap_or_else(|| std::env::var("SMTP_TOKEN").unwrap_or_default());
    let from_email = custom_from.unwrap_or_else(|| std::env::var("SMTP_FROM").unwrap_or_else(|_| "noreply@atlas-platform.local".to_string()));

    // 3. Construct MultiPart Body
    let mut multipart = MultiPart::mixed().singlepart(
        SinglePart::builder()
            .header(header::ContentType::TEXT_HTML)
            .body(payload.body_html.clone()),
    );

    // 4. Download S3 attachments and append to Multipart
    if !payload.attachments.is_empty() {
        let access_key = std::env::var("R2_ACCESS_KEY_ID").unwrap_or_default();
        let secret = std::env::var("R2_SECRET_ACCESS_KEY").unwrap_or_default();
        let endpoint = std::env::var("R2_ENDPOINT").unwrap_or_default();
        let bucket_name = "atlas-tenant-vault".to_string();

        if !access_key.is_empty() && !endpoint.is_empty() {
            let credentials = aws_sdk_s3::config::Credentials::new(
                access_key, secret, None, None, "cloudflare"
            );
            let s3_config = aws_sdk_s3::config::Builder::new()
                .credentials_provider(credentials)
                .region(aws_sdk_s3::config::Region::new("auto"))
                .endpoint_url(endpoint)
                .build();
            let client = aws_sdk_s3::Client::from_conf(s3_config);
            for file_key in &payload.attachments {
                if let Ok(resp) = client.get_object().bucket(&bucket_name).key(file_key).send().await {
                    if let Ok(data) = resp.body.collect().await {
                        let bytes = data.into_bytes().to_vec();
                        let filename = file_key.split('/').last().unwrap_or("attachment").to_string();
                        let ext = filename.split('.').last().unwrap_or("").to_lowercase();
                        let mime = match ext.as_str() {
                            "pdf" => "application/pdf",
                            "png" => "image/png",
                            "jpg" | "jpeg" => "image/jpeg",
                            "gif" => "image/gif",
                            "txt" => "text/plain",
                            "html" => "text/html",
                            "doc" | "docx" => "application/msword",
                            _ => "application/octet-stream",
                        };
                        if let Ok(m_parsed) = mime.parse() {
                            let part = lettre::message::Attachment::new(filename)
                                .body(bytes, m_parsed);
                            multipart = multipart.singlepart(part);
                        }
                    }
                }
            }
        }
    }

    // 5. Construct Email Message
    let email = Message::builder()
        .from(from_email.parse().map_err(|_| (StatusCode::BAD_REQUEST, "Invalid FROM email".to_string()))?)
        .to(payload.to_email.parse().map_err(|_| (StatusCode::BAD_REQUEST, "Invalid TO email".to_string()))?)
        .subject(&payload.subject)
        .multipart(multipart)
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
