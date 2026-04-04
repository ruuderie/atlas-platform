use anyhow::Result;
use chrono::{DateTime, Utc};
use crate::traits::telephony::{CallEvent, CallLog, PhoneNumber, TelephonyProvider};

pub struct TelnyxAdapter {
    pub api_key: String,
}

impl TelnyxAdapter {
    pub fn new(api_key: String) -> Self {
        Self { api_key }
    }
}

#[async_trait::async_trait]
impl TelephonyProvider for TelnyxAdapter {
    async fn provision_number(&self, area_code: &str) -> Result<PhoneNumber> {
        // Implement Telnyx API call to provision a number
        Ok(PhoneNumber {
            number: format!("+1{}5550299", area_code),
            area_code: area_code.to_string(),
        })
    }

    async fn send_sms(&self, to: &str, body: &str) -> Result<()> {
        // Implement Telnyx API call to send an SMS
        println!("Sending SMS via Telnyx to {}: {}", to, body);
        Ok(())
    }

    async fn get_call_logs(&self, number: &str, _since: DateTime<Utc>) -> Result<Vec<CallLog>> {
        // Implement Telnyx API call to retrieve call logs
        println!("Retrieving Telnyx call logs for {}", number);
        Ok(vec![])
    }

    fn normalize_webhook(&self, payload: &serde_json::Value) -> Result<CallEvent> {
        // Extract required fields from Telnyx's specific webhook payload format
        let data = payload.get("data").and_then(|v| v.as_object());
        let payload_type = data.and_then(|d| d.get("record_type")).and_then(|v| v.as_str()).unwrap_or_default();
        let call_session_id = data.and_then(|d| d.get("call_session_id")).and_then(|v| v.as_str()).unwrap_or_default().to_string();
        
        // This is a simplified extraction example
        Ok(CallEvent {
            call_id: call_session_id,
            to: data.and_then(|d| d.get("to")).and_then(|v| v.as_str()).unwrap_or_default().to_string(),
            from: data.and_then(|d| d.get("from")).and_then(|v| v.as_str()).unwrap_or_default().to_string(),
            status: payload_type.to_string(),
            duration: None, // May require additional logic to calculate
            raw_payload: payload.clone(),
        })
    }
}
