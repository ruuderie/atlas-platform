use anyhow::{anyhow, Result};
use std::env;

use crate::traits::telephony::TelephonyProvider;
use super::telnyx::TelnyxAdapter;
use super::twilio::TwilioAdapter;

pub fn get_telephony_provider() -> Result<Box<dyn TelephonyProvider>> {
    let provider_name = env::var("TELEPHONY_PROVIDER").unwrap_or_else(|_| "twilio".to_string());

    match provider_name.to_lowercase().as_str() {
        "twilio" => {
            let account_sid = env::var("TWILIO_ACCOUNT_SID")
                .map_err(|_| anyhow!("Missing TWILIO_ACCOUNT_SID environment variable"))?;
            let auth_token = env::var("TWILIO_AUTH_TOKEN")
                .map_err(|_| anyhow!("Missing TWILIO_AUTH_TOKEN environment variable"))?;
            
            Ok(Box::new(TwilioAdapter::new(account_sid, auth_token)))
        }
        "telnyx" => {
            let api_key = env::var("TELNYX_API_KEY")
                .map_err(|_| anyhow!("Missing TELNYX_API_KEY environment variable"))?;
                
            Ok(Box::new(TelnyxAdapter::new(api_key)))
        }
        _ => Err(anyhow!("Unsupported configured telephony provider: {}", provider_name)),
    }
}
