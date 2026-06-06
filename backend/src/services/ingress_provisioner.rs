#![allow(dead_code, unused)]
use reqwest::Client;
use serde::Serialize;

#[derive(Serialize)]
struct IngressPayload<'a> {
    tenant_slug: &'a str,
    domain: &'a str,
}

pub struct IngressProvisioner {
    client: Client,
    sidecar_url: String,
    is_dev: bool,
}

impl IngressProvisioner {
    pub fn new() -> Self {
        let sidecar_url = std::env::var("INGRESS_SIDECAR_URL")
            .unwrap_or_else(|_| "http://localhost:8085".to_string());
        let env = std::env::var("ENVIRONMENT").unwrap_or_else(|_| "production".to_string());
        let is_dev = env == "development";

        Self {
            client: Client::new(),
            sidecar_url,
            is_dev,
        }
    }

    pub async fn provision_domain(&self, tenant_slug: &str, domain: &str) -> Result<(), String> {
        if self.is_dev {
            tracing::info!(
                event = "provision.ingress.bypass_dev",
                tenant_slug = %tenant_slug,
                domain = %domain,
                message = "Local dev mode: Bypassing ingress provision sidecar call (dry-run successfully logged)."
            );
            return Ok(());
        }

        let url = format!("{}/api/ingress/provision", self.sidecar_url);
        let payload = IngressPayload { tenant_slug, domain };

        let res = self.client.post(&url)
            .json(&payload)
            .send()
            .await
            .map_err(|e| format!("Failed to call ingress sidecar: {}", e))?;

        if !res.status().is_success() {
            let status = res.status();
            let body_text = res.text().await.unwrap_or_default();
            return Err(format!("Ingress sidecar returned status {}: {}", status, body_text));
        }

        tracing::info!(
            event = "provision.ingress.success",
            tenant_slug = %tenant_slug,
            domain = %domain
        );
        Ok(())
    }

    pub async fn deprovision_domain(&self, tenant_slug: &str, domain: &str) -> Result<(), String> {
        if self.is_dev {
            tracing::info!(
                event = "deprovision.ingress.bypass_dev",
                tenant_slug = %tenant_slug,
                domain = %domain,
                message = "Local dev mode: Bypassing ingress deprovision sidecar call."
            );
            return Ok(());
        }

        let url = format!("{}/api/ingress/deprovision", self.sidecar_url);
        let payload = IngressPayload { tenant_slug, domain };

        let res = self.client.post(&url)
            .json(&payload)
            .send()
            .await
            .map_err(|e| format!("Failed to call ingress sidecar: {}", e))?;

        if !res.status().is_success() {
            let status = res.status();
            let body_text = res.text().await.unwrap_or_default();
            return Err(format!("Ingress deprovision sidecar returned status {}: {}", status, body_text));
        }

        tracing::info!(
            event = "deprovision.ingress.success",
            tenant_slug = %tenant_slug,
            domain = %domain
        );
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use wiremock::matchers::{method, path, body_json};
    use wiremock::{Mock, MockServer, ResponseTemplate};
    use serde_json::json;

    #[tokio::test]
    async fn test_provision_domain_success() {
        let mock_server = MockServer::start().await;
        
        Mock::given(method("POST"))
            .and(path("/api/ingress/provision"))
            .and(body_json(json!({
                "tenant_slug": "test-tenant",
                "domain": "test.com"
            })))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({"status": "success"})))
            .mount(&mock_server)
            .await;

        let provisioner = IngressProvisioner {
            client: Client::new(),
            sidecar_url: mock_server.uri(),
            is_dev: false,
        };

        let result = provisioner.provision_domain("test-tenant", "test.com").await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_provision_domain_failure() {
        let mock_server = MockServer::start().await;
        
        Mock::given(method("POST"))
            .and(path("/api/ingress/provision"))
            .respond_with(ResponseTemplate::new(500).set_body_string("Internal Error"))
            .mount(&mock_server)
            .await;

        let provisioner = IngressProvisioner {
            client: Client::new(),
            sidecar_url: mock_server.uri(),
            is_dev: false,
        };

        let result = provisioner.provision_domain("test-tenant", "test.com").await;
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Internal Error"));
    }

    #[tokio::test]
    async fn test_provision_domain_bypass_dev() {
        let provisioner = IngressProvisioner {
            client: Client::new(),
            sidecar_url: "http://invalid-url-should-not-be-called".to_string(),
            is_dev: true,
        };

        let result = provisioner.provision_domain("test-tenant", "test.com").await;
        assert!(result.is_ok(), "Dev mode should bypass external calls and return Ok");
    }

    #[tokio::test]
    async fn test_deprovision_domain_success() {
        let mock_server = MockServer::start().await;
        
        Mock::given(method("POST"))
            .and(path("/api/ingress/deprovision"))
            .and(body_json(json!({
                "tenant_slug": "test-tenant",
                "domain": "test.com"
            })))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({"status": "success"})))
            .mount(&mock_server)
            .await;

        let provisioner = IngressProvisioner {
            client: Client::new(),
            sidecar_url: mock_server.uri(),
            is_dev: false,
        };

        let result = provisioner.deprovision_domain("test-tenant", "test.com").await;
        assert!(result.is_ok());
    }
}
