//! Platform-admin API client for the syndication offer catalog and active links.
//!
//! Wraps the backend endpoints at:
//!   /api/admin/syndication/offers  (CRUD)
//!   /api/admin/syndication/links   (list, create, revoke)
//!   /api/admin/syndication/offers/:id/auto-provision

use serde::{Deserialize, Serialize};
use uuid::Uuid;
use serde_json::Value;

use super::client::{api_get, api_post, api_put, api_delete};
use super::client::{create_client, api_url, with_credentials, api_request};

// ── Response models ───────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyndicationOfferModel {
    pub id: String,
    pub ni_config_id: String,
    pub display_name: String,
    pub description: Option<String>,
    pub syndication_types: Value,
    pub link_type: String,
    pub is_mandatory_for_tiers: Value,
    pub self_service_allowed: bool,
    pub applies_to_folio_mode: Option<String>,
    pub applies_to_app_slug: Option<String>,
    pub status: String,
    pub created_at: String,
    pub updated_at: String,
}

impl SyndicationOfferModel {
    pub fn types_display(&self) -> String {
        self.syndication_types
            .as_array()
            .map(|arr| arr.iter().filter_map(|v| v.as_str()).collect::<Vec<_>>().join(", "))
            .unwrap_or_else(|| "—".to_string())
    }

    pub fn mandatory_tiers_display(&self) -> String {
        self.is_mandatory_for_tiers
            .as_array()
            .map(|arr| arr.iter().filter_map(|v| v.as_str()).collect::<Vec<_>>().join(", "))
            .unwrap_or_else(|| "none".to_string())
    }

    pub fn link_type_label(&self) -> &str {
        match self.link_type.as_str() {
            "branded_portal" => "Branded Portal",
            _ => "Marketplace Syndication",
        }
    }

    pub fn is_retired(&self) -> bool {
        self.status == "retired"
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyndicationLinkModel {
    pub id: String,
    pub source_config_id: String,
    pub ni_config_id: String,
    pub offer_id: Option<String>,
    pub syndication_types: Value,
    pub link_type: String,
    pub is_mandatory: bool,
    pub status: String,
    pub inbound_webhook_url: Option<String>,
    pub created_by_tenant_id: String,
    pub created_at: String,
}

// ── Request models ────────────────────────────────────────────────────────────

#[derive(Debug, Serialize)]
pub struct CreateOfferInput {
    pub ni_config_id: String,
    pub display_name: String,
    pub description: Option<String>,
    pub syndication_types: Value,
    pub link_type: String,
    pub is_mandatory_for_tiers: Value,
    pub self_service_allowed: bool,
    pub applies_to_folio_mode: Option<String>,
    pub applies_to_app_slug: Option<String>,
}

#[derive(Debug, Serialize, Default)]
pub struct UpdateOfferInput {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub syndication_types: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub link_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_mandatory_for_tiers: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub self_service_allowed: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub applies_to_folio_mode: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct CreateLinkInput {
    pub source_config_id: String,
    pub ni_config_id: String,
    pub offer_id: Option<String>,
    pub syndication_types: Option<Value>,
    pub link_type: Option<String>,
    pub inbound_webhook_url: Option<String>,
    pub created_by_tenant_id: String,
}

#[derive(Debug, Deserialize)]
pub struct AutoProvisionResult {
    pub provisioned: u32,
    pub skipped: u32,
    pub offer_id: String,
    pub mandatory_tiers: Vec<String>,
}

// ── API functions ─────────────────────────────────────────────────────────────

/// Fetch all non-retired syndication offers.
pub async fn list_syndication_offers() -> Result<Vec<SyndicationOfferModel>, String> {
    api_get("/api/admin/syndication/offers").await
}

/// Fetch a single offer by ID.
pub async fn get_syndication_offer(id: &str) -> Result<SyndicationOfferModel, String> {
    api_get(&format!("/api/admin/syndication/offers/{}", id)).await
}

/// Create a new syndication offer (platform admin).
pub async fn create_syndication_offer(input: CreateOfferInput) -> Result<SyndicationOfferModel, String> {
    api_post("/api/admin/syndication/offers", &input).await
}

/// Update an existing syndication offer.
pub async fn update_syndication_offer(id: &str, input: UpdateOfferInput) -> Result<SyndicationOfferModel, String> {
    api_put(&format!("/api/admin/syndication/offers/{}", id), &input).await
}

/// Retire an offer (soft delete — existing links remain, no new activations).
pub async fn retire_syndication_offer(id: &str) -> Result<(), String> {
    let client = create_client();
    let url = api_url(&format!("/api/admin/syndication/offers/{}/retire", id));
    let req = client.post(&url);
    let req = with_credentials(req);
    let res = req.send().await.map_err(|e| e.to_string())?;
    if res.status().is_success() { Ok(()) } else { Err(format!("Retire failed: {}", res.status())) }
}

/// Auto-provision mandatory links for all instances on matching billing tiers.
pub async fn auto_provision_mandatory_links(offer_id: &str) -> Result<AutoProvisionResult, String> {
    let client = create_client();
    let url = api_url(&format!("/api/admin/syndication/offers/{}/auto-provision", offer_id));
    let req = client.post(&url);
    api_request(req).await
}

/// Fetch all active syndication links.
pub async fn list_syndication_links() -> Result<Vec<SyndicationLinkModel>, String> {
    api_get("/api/admin/syndication/links").await
}

/// Create a manual syndication link (admin-only, bypasses self-service gate).
pub async fn create_syndication_link(input: CreateLinkInput) -> Result<SyndicationLinkModel, String> {
    api_post("/api/admin/syndication/links", &input).await
}

/// Revoke an active syndication link.
pub async fn revoke_syndication_link(id: &str) -> Result<(), String> {
    api_delete(&format!("/api/admin/syndication/links/{}", id)).await
}
