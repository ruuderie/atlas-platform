use serde::{Deserialize, Serialize};
use uuid::Uuid;
use super::client::{api_post};

#[derive(Debug, Serialize)]
pub struct SendEmailPayload {
    pub tenant_id: Uuid,
    pub to_email: String,
    pub subject: String,
    pub body_html: String,
    #[serde(default)]
    pub attachments: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct SendEmailResponse {
    pub message: String,
}

/// Send an email via the platform SMTP gateway.
/// Calls `POST /api/communications/email`.
///
/// SMTP credentials are resolved in order:
///   1. Tenant settings (smtp_server, smtp_token, etc.) from `tenant_setting` table
///   2. Platform-level env vars: SMTP_SERVER, SMTP_TOKEN, SMTP_FROM, SMTP_PORT
///   3. If host is empty/localhost, backend mocks the send and returns 200 "mocked"
pub async fn send_email(payload: SendEmailPayload) -> Result<SendEmailResponse, String> {
    api_post("api/communications/email", &payload).await
}
