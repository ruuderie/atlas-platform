use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhoneNumber {
    pub number: String,
    pub area_code: String,
    // Add other fields as necessary
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CallLog {
    pub to: String,
    pub from: String,
    pub start_time: DateTime<Utc>,
    pub duration_seconds: u32,
    pub status: String,
    // Add other fields as necessary
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CallEvent {
    pub call_id: String,
    pub to: String,
    pub from: String,
    pub status: String,
    pub duration: Option<u32>,
    pub raw_payload: serde_json::Value,
}

pub trait TelephonyProvider: Send + Sync {
    /// Provision a new phone number based on a specific area code
    async fn provision_number(&self, area_code: &str) -> Result<PhoneNumber>;
    
    /// Send an SMS message
    async fn send_sms(&self, to: &str, body: &str) -> Result<()>;
    
    /// Retrieve call logs for a specific number since a specific time
    async fn get_call_logs(&self, number: &str, since: DateTime<Utc>) -> Result<Vec<CallLog>>;
    
    /// Normalize a provider-specific webhook payload into a standard CallEvent
    fn normalize_webhook(&self, payload: &serde_json::Value) -> Result<CallEvent>;
}
