use async_trait::async_trait;
use reqwest::Client;
use std::env;
use serde_json::json;

#[derive(Debug)]
pub enum DnsError {
    ProviderConfigError(String),
    ProvisioningError(String),
}

impl std::fmt::Display for DnsError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ProviderConfigError(msg) => write!(f, "Provider Config Error: {}", msg),
            Self::ProvisioningError(msg) => write!(f, "Provisioning Error: {}", msg),
        }
    }
}

impl std::error::Error for DnsError {}

#[async_trait]
pub trait DnsProvider: Send + Sync {
    async fn provision_hostname(&self, hostname: &str) -> Result<(), DnsError>;
    async fn remove_hostname(&self, hostname: &str) -> Result<(), DnsError>;
}

/// Cloudflare SSL for SaaS Implementation
pub struct CloudflareProvider {
    client: Client,
    api_token: String,
    zone_id: String,
}

impl CloudflareProvider {
    pub fn new() -> Result<Self, DnsError> {
        let api_token = env::var("CLOUDFLARE_API_TOKEN")
            .map_err(|_| DnsError::ProviderConfigError("Missing CLOUDFLARE_API_TOKEN".into()))?;
        let zone_id = env::var("CLOUDFLARE_ZONE_ID")
            .map_err(|_| DnsError::ProviderConfigError("Missing CLOUDFLARE_ZONE_ID".into()))?;

        Ok(Self {
            client: Client::new(),
            api_token,
            zone_id,
        })
    }
}

#[async_trait]
impl DnsProvider for CloudflareProvider {
    async fn provision_hostname(&self, hostname: &str) -> Result<(), DnsError> {
        let url = format!(
            "https://api.cloudflare.com/client/v4/zones/{}/custom_hostnames",
            self.zone_id
        );

        let payload = json!({
            "hostname": hostname,
            "ssl": {
                "method": "http",
                "type": "dv"
            }
        });

        let res = self.client.post(&url)
            .header("Authorization", format!("Bearer {}", self.api_token))
            .header("Content-Type", "application/json")
            .json(&payload)
            .send()
            .await
            .map_err(|e| DnsError::ProvisioningError(e.to_string()))?;

        if !res.status().is_success() {
            let error_text = res.text().await.unwrap_or_default();
            tracing::error!("Cloudflare provisioning failed: {}", error_text);
            return Err(DnsError::ProvisioningError(error_text));
        }

        tracing::info!("Cloudflare custom hostname correctly provisioned for: {}", hostname);
        Ok(())
    }

    async fn remove_hostname(&self, _hostname: &str) -> Result<(), DnsError> {
        // Implementing removal dynamically requires querying the custom_hostname ID first 
        // to pass to the DELETE endpoint. We can stub it for future implementation.
        tracing::warn!("Cloudflare auto-delete domain not implemented natively yet.");
        Ok(())
    }
}

/// Helper function to automatically pick the correct edge orchestration logic.
pub async fn provision_domain(domain: &str) -> Result<(), String> {
    // Check if the administrator activated TLS_PROVIDER=cloudflare
    let provider_name = env::var("TLS_PROVIDER").unwrap_or_else(|_| "cloudflare".to_string());
    
    tracing::info!("Dispatching DNS provision routing via Provider: {}", provider_name);
    
    let provider: Box<dyn DnsProvider> = match provider_name.as_str() {
        "cloudflare" => {
            // Allow failing gracefully if secrets aren't set in local DEV.
            if env::var("CLOUDFLARE_API_TOKEN").is_err() {
                tracing::warn!("DNS Abstraction caught Missing CLOUDFLARE_API_TOKEN inside dev context. Passing through gracefully to allow local testing!");
                return Ok(());
            }
            Box::new(CloudflareProvider::new().map_err(|e| e.to_string())?)
        },
        _ => {
            tracing::warn!("Unknown edge provider: {}, passing through gracefully.", provider_name);
            return Ok(());
        }
    };

    provider.provision_hostname(domain).await.map_err(|e| e.to_string())
}
