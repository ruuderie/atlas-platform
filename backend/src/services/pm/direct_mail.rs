//! Direct-mail provider adapter (plug-ready for Lob / PropertyRadar).
//!
//! v1 ships `ManualCsvProvider` only. Lob and PropertyRadar register as
//! `NotImplemented` stubs so platform-admin can select them after wiring.

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Normalized inbound events from a mail provider webhook or manual ops.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum DirectMailEvent {
    Submitted { provider_job_id: String },
    InTransit { provider_job_id: String },
    Delivered { provider_job_id: String },
    Returned { provider_job_id: String, reason: Option<String> },
    Failed { provider_job_id: String, reason: Option<String> },
    CostIncurred {
        provider_job_id: Option<String>,
        cents: i64,
    },
    QrScan { provider_job_id: Option<String> },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MailRecipient {
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub company: Option<String>,
    pub email: Option<String>,
    pub street_address: Option<String>,
    pub city: Option<String>,
    pub state: Option<String>,
    pub postal_code: Option<String>,
    pub country: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubmitMailJobResult {
    pub provider_job_id: String,
    pub piece_count: i32,
    pub estimated_cost_cents: Option<i64>,
}

#[derive(Debug)]
pub enum DirectMailError {
    NotImplemented(&'static str),
    InvalidCredentials(String),
    Provider(String),
}

impl std::fmt::Display for DirectMailError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            DirectMailError::NotImplemented(provider) => write!(f, "provider not implemented: {}", provider),
            DirectMailError::InvalidCredentials(msg) => write!(f, "invalid credentials: {}", msg),
            DirectMailError::Provider(msg) => write!(f, "provider error: {}", msg),
        }
    }
}

impl std::error::Error for DirectMailError {}

#[async_trait]
pub trait DirectMailProvider: Send + Sync {
    fn provider_id(&self) -> &'static str;

    async fn validate_credentials(
        &self,
        credentials: &serde_json::Value,
    ) -> std::result::Result<bool, DirectMailError>;

    async fn submit_mail_job(
        &self,
        campaign_id: Uuid,
        drop_id: Uuid,
        recipients: &[MailRecipient],
        credentials: &serde_json::Value,
    ) -> std::result::Result<SubmitMailJobResult, DirectMailError>;

    async fn cancel_job(
        &self,
        provider_job_id: &str,
        credentials: &serde_json::Value,
    ) -> std::result::Result<(), DirectMailError>;

    fn parse_webhook(
        &self,
        payload: &serde_json::Value,
    ) -> std::result::Result<Vec<DirectMailEvent>, DirectMailError>;
}

/// Ops export CSV + manual spend — no outbound API.
pub struct ManualCsvProvider;

#[async_trait]
impl DirectMailProvider for ManualCsvProvider {
    fn provider_id(&self) -> &'static str {
        "dm_manual"
    }

    async fn validate_credentials(
        &self,
        _credentials: &serde_json::Value,
    ) -> std::result::Result<bool, DirectMailError> {
        Ok(true)
    }

    async fn submit_mail_job(
        &self,
        campaign_id: Uuid,
        drop_id: Uuid,
        recipients: &[MailRecipient],
        _credentials: &serde_json::Value,
    ) -> std::result::Result<SubmitMailJobResult, DirectMailError> {
        Ok(SubmitMailJobResult {
            provider_job_id: format!("manual-{campaign_id}-{drop_id}"),
            piece_count: recipients.len() as i32,
            estimated_cost_cents: None,
        })
    }

    async fn cancel_job(
        &self,
        _provider_job_id: &str,
        _credentials: &serde_json::Value,
    ) -> std::result::Result<(), DirectMailError> {
        Ok(())
    }

    fn parse_webhook(
        &self,
        _payload: &serde_json::Value,
    ) -> std::result::Result<Vec<DirectMailEvent>, DirectMailError> {
        Ok(vec![])
    }
}

pub struct LobProviderStub;
pub struct PropertyRadarProviderStub;

#[async_trait]
impl DirectMailProvider for LobProviderStub {
    fn provider_id(&self) -> &'static str {
        "dm_lob"
    }

    async fn validate_credentials(
        &self,
        _credentials: &serde_json::Value,
    ) -> std::result::Result<bool, DirectMailError> {
        Err(DirectMailError::NotImplemented("dm_lob"))
    }

    async fn submit_mail_job(
        &self,
        _campaign_id: Uuid,
        _drop_id: Uuid,
        _recipients: &[MailRecipient],
        _credentials: &serde_json::Value,
    ) -> std::result::Result<SubmitMailJobResult, DirectMailError> {
        Err(DirectMailError::NotImplemented("dm_lob"))
    }

    async fn cancel_job(
        &self,
        _provider_job_id: &str,
        _credentials: &serde_json::Value,
    ) -> std::result::Result<(), DirectMailError> {
        Err(DirectMailError::NotImplemented("dm_lob"))
    }

    fn parse_webhook(
        &self,
        _payload: &serde_json::Value,
    ) -> std::result::Result<Vec<DirectMailEvent>, DirectMailError> {
        Err(DirectMailError::NotImplemented("dm_lob"))
    }
}

#[async_trait]
impl DirectMailProvider for PropertyRadarProviderStub {
    fn provider_id(&self) -> &'static str {
        "dm_property_radar"
    }

    async fn validate_credentials(
        &self,
        _credentials: &serde_json::Value,
    ) -> std::result::Result<bool, DirectMailError> {
        Err(DirectMailError::NotImplemented("dm_property_radar"))
    }

    async fn submit_mail_job(
        &self,
        _campaign_id: Uuid,
        _drop_id: Uuid,
        _recipients: &[MailRecipient],
        _credentials: &serde_json::Value,
    ) -> std::result::Result<SubmitMailJobResult, DirectMailError> {
        Err(DirectMailError::NotImplemented("dm_property_radar"))
    }

    async fn cancel_job(
        &self,
        _provider_job_id: &str,
        _credentials: &serde_json::Value,
    ) -> std::result::Result<(), DirectMailError> {
        Err(DirectMailError::NotImplemented("dm_property_radar"))
    }

    fn parse_webhook(
        &self,
        _payload: &serde_json::Value,
    ) -> std::result::Result<Vec<DirectMailEvent>, DirectMailError> {
        Err(DirectMailError::NotImplemented("dm_property_radar"))
    }
}

/// Resolve a provider by `integration_type` / picker id.
pub fn resolve_direct_mail_provider(id: &str) -> Option<&'static dyn DirectMailProvider> {
    match id {
        "dm_manual" | "manual" => Some(&MANUAL_CSV),
        "dm_lob" | "lob" => Some(&LOB_STUB),
        "dm_property_radar" | "property_radar" => Some(&PROPERTY_RADAR_STUB),
        _ => None,
    }
}

static MANUAL_CSV: ManualCsvProvider = ManualCsvProvider;
static LOB_STUB: LobProviderStub = LobProviderStub;
static PROPERTY_RADAR_STUB: PropertyRadarProviderStub = PropertyRadarProviderStub;
