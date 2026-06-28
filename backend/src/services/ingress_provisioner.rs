#![allow(dead_code, unused)]
use reqwest::Client;
use serde::Serialize;

#[derive(Serialize)]
struct IngressPayload<'a> {
    tenant_slug: &'a str,
    domain:      &'a str,
    app_slug:    &'a str,
}

#[derive(Serialize)]
struct DeprovisionPayload<'a> {
    tenant_slug: &'a str,
    domain:      &'a str,
}

pub struct IngressProvisioner {
    client:      Client,
    sidecar_url: String,
}

impl IngressProvisioner {
    pub fn new() -> Self {
        let sidecar_url = std::env::var("INGRESS_SIDECAR_URL")
            .unwrap_or_else(|_| "http://localhost:8085".to_string());

        Self {
            client: Client::new(),
            sidecar_url,
        }
    }

    /// Create or update the Ingress for `domain`, routing to the correct service for `app_slug`.
    /// The sidecar also handles wildcard-vs-custom TLS strategy automatically.
    pub async fn provision_domain(
        &self,
        tenant_slug: &str,
        domain:      &str,
        app_slug:    &str,
    ) -> Result<(), String> {
        let url     = format!("{}/api/ingress/provision", self.sidecar_url);
        let payload = IngressPayload { tenant_slug, domain, app_slug };

        let res = self.client.post(&url)
            .json(&payload)
            .send()
            .await
            .map_err(|e| format!("Failed to call ingress sidecar: {}", e))?;

        if !res.status().is_success() {
            let status    = res.status();
            let body_text = res.text().await.unwrap_or_default();
            return Err(format!("Ingress sidecar returned status {}: {}", status, body_text));
        }

        tracing::info!(
            event       = "provision.ingress.success",
            tenant_slug = %tenant_slug,
            domain      = %domain,
            app_slug    = %app_slug,
        );
        Ok(())
    }

    pub async fn deprovision_domain(&self, tenant_slug: &str, domain: &str) -> Result<(), String> {
        let url     = format!("{}/api/ingress/deprovision", self.sidecar_url);
        let payload = DeprovisionPayload { tenant_slug, domain };

        let res = self.client.post(&url)
            .json(&payload)
            .send()
            .await
            .map_err(|e| format!("Failed to call ingress sidecar: {}", e))?;

        if !res.status().is_success() {
            let status    = res.status();
            let body_text = res.text().await.unwrap_or_default();
            return Err(format!("Ingress deprovision sidecar returned status {}: {}", status, body_text));
        }

        tracing::info!(
            event       = "deprovision.ingress.success",
            tenant_slug = %tenant_slug,
            domain      = %domain,
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
                "domain":      "test.com",
                "app_slug":    "property_management"
            })))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({"status": "success"})))
            .mount(&mock_server)
            .await;

        let provisioner = IngressProvisioner {
            client:      Client::new(),
            sidecar_url: mock_server.uri(),
        };

        let result = provisioner.provision_domain("test-tenant", "test.com", "property_management").await;
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
            client:      Client::new(),
            sidecar_url: mock_server.uri(),
        };

        let result = provisioner.provision_domain("test-tenant", "test.com", "property_management").await;
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Internal Error"));
    }

    #[tokio::test]
    async fn test_deprovision_domain_success() {
        let mock_server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/api/ingress/deprovision"))
            .and(body_json(json!({
                "tenant_slug": "test-tenant",
                "domain":      "test.com"
            })))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({"status": "success"})))
            .mount(&mock_server)
            .await;

        let provisioner = IngressProvisioner {
            client:      Client::new(),
            sidecar_url: mock_server.uri(),
        };

        let result = provisioner.deprovision_domain("test-tenant", "test.com").await;
        assert!(result.is_ok());
    }
}
