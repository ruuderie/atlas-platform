use anyhow::Result;
use chrono::{DateTime, Utc};
use crate::traits::telephony::{CallEvent, CallLog, PhoneNumber, TelephonyProvider};

pub struct TwilioAdapter {
    pub account_sid: String,
    pub auth_token: String,
}

impl TwilioAdapter {
    pub fn new(account_sid: String, auth_token: String) -> Self {
        Self { account_sid, auth_token }
    }
}

impl TelephonyProvider for TwilioAdapter {
    async fn provision_number(&self, area_code: &str) -> Result<PhoneNumber> {
        // Implement Twilio API call to provision a number
        Ok(PhoneNumber {
            number: format!("+1{}5550199", area_code),
            area_code: area_code.to_string(),
        })
    }

    async fn send_sms(&self, to: &str, body: &str) -> Result<()> {
        // Implement Twilio API call to send an SMS
        println!("Sending SMS via Twilio to {}: {}", to, body);
        Ok(())
    }

    async fn get_call_logs(&self, number: &str, _since: DateTime<Utc>) -> Result<Vec<CallLog>> {
        // Implement Twilio API call to retrieve call logs for a specific number
        println!("Retrieving Twilio call logs for {}", number);
        Ok(vec![])
    }

    fn normalize_webhook(&self, payload: &serde_json::Value) -> Result<CallEvent> {
        // Extract required fields from Twilio's specific webhook payload format
        Ok(CallEvent {
            call_id: payload.get("CallSid").and_then(|v| v.as_str()).unwrap_or_default().to_string(),
            to: payload.get("To").and_then(|v| v.as_str()).unwrap_or_default().to_string(),
            from: payload.get("From").and_then(|v| v.as_str()).unwrap_or_default().to_string(),
            status: payload.get("CallStatus").and_then(|v| v.as_str()).unwrap_or_default().to_string(),
            duration: payload.get("DialCallDuration").and_then(|v| v.as_str()).and_then(|s| s.parse().ok()),
            raw_payload: payload.clone(),
        })
    }
}
