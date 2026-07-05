//! Platform-admin API client — Landing Page Builder
//!
//! Async functions that call the `/api/admin/landing-pages/*` and
//! `/api/admin/utm-presets/*` backend routes from the Leptos WASM frontend.

use super::client::{api_url, create_client, with_credentials, api_request};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;
use chrono::{DateTime, Utc};

// ── Models ─────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct LandingPageSummary {
    pub id: Uuid,
    pub app_id: String,
    pub slug: String,
    pub title: String,
    pub page_type: String,
    pub locale: String,           // "en" | "pt" | "es" | "fr"
    pub is_published: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Full page model — returned by create / get-by-id / update.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct LandingPage {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub app_id: String,
    pub slug: String,
    pub title: String,
    pub description: String,
    pub page_type: String,
    pub locale: String,           // "en" | "pt" | "es" | "fr"
    pub hero_payload: Option<Value>,
    pub blocks_payload: Option<Value>,
    pub is_published: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PageVariant {
    pub id: Uuid,
    pub page_id: Uuid,
    pub name: String,
    pub traffic_pct: i32,
    pub is_control: bool,
    pub blocks_payload: Value,
    pub hero_payload: Option<Value>,
    pub view_count: i32,
    pub lead_count: i32,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct UtmPreset {
    pub id: Uuid,
    pub app_id: String,
    pub name: String,
    pub utm_source: String,
    pub utm_medium: String,
    pub utm_campaign: String,
    pub utm_content: Option<String>,
    pub utm_term: Option<String>,
    pub click_count: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// ── Payloads ───────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateLandingPagePayload {
    pub app_id: String,
    pub slug: String,
    pub title: String,
    pub description: Option<String>,
    pub page_type: Option<String>,
    pub locale: Option<String>,   // defaults to "en" on the backend
    pub hero_payload: Option<Value>,
    pub blocks_payload: Option<Value>,
    pub is_published: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct UpdateLandingPagePayload {
    pub title: Option<String>,
    pub description: Option<String>,
    pub page_type: Option<String>,
    pub locale: Option<String>,   // change locale of an existing page
    pub hero_payload: Option<Value>,
    pub blocks_payload: Option<Value>,
    pub slug: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateVariantPayload {
    pub name: String,
    pub traffic_pct: Option<i32>,
    pub blocks_payload: Option<Value>,
    pub hero_payload: Option<Value>,
    pub is_control: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct UpdateVariantPayload {
    pub name: Option<String>,
    pub traffic_pct: Option<i32>,
    pub blocks_payload: Option<Value>,
    pub hero_payload: Option<Value>,
    pub is_active: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateUtmPresetPayload {
    pub app_id: String,
    pub name: String,
    pub utm_source: String,
    pub utm_medium: String,
    pub utm_campaign: String,
    pub utm_content: Option<String>,
    pub utm_term: Option<String>,
}

// ── Page API functions ─────────────────────────────────────────────────────────

/// `GET /api/admin/landing-pages?app_id={app_id}` — list all pages for an app.
pub async fn list_landing_pages(app_id: &str) -> Result<Vec<LandingPageSummary>, String> {
    let client = create_client();
    let url = api_url(&format!("/api/admin/landing-pages?app_id={}", app_id));
    let req = with_credentials(client.get(&url));
    api_request::<Vec<LandingPageSummary>>(req).await
}

/// `GET /api/admin/landing-pages/{page_id}` — fetch a single page.
pub async fn get_landing_page(page_id: Uuid) -> Result<LandingPage, String> {
    let client = create_client();
    let url = api_url(&format!("/api/admin/landing-pages/{}", page_id));
    let req = with_credentials(client.get(&url));
    api_request::<LandingPage>(req).await
}

/// `POST /api/admin/landing-pages` — create a new page.
pub async fn create_landing_page(payload: CreateLandingPagePayload) -> Result<LandingPage, String> {
    let client = create_client();
    let url = api_url("/api/admin/landing-pages");
    let req = with_credentials(client.post(&url).json(&payload));
    api_request::<LandingPage>(req).await
}

/// `PUT /api/admin/landing-pages/{page_id}` — update page metadata / blocks.
pub async fn update_landing_page(
    page_id: Uuid,
    payload: UpdateLandingPagePayload,
) -> Result<LandingPage, String> {
    let client = create_client();
    let url = api_url(&format!("/api/admin/landing-pages/{}", page_id));
    let req = with_credentials(client.put(&url).json(&payload));
    api_request::<LandingPage>(req).await
}

/// `POST /api/admin/landing-pages/{page_id}/publish` — toggle publish state.
pub async fn toggle_publish(page_id: Uuid) -> Result<LandingPage, String> {
    let client = create_client();
    let url = api_url(&format!("/api/admin/landing-pages/{}/publish", page_id));
    let req = with_credentials(client.post(&url));
    api_request::<LandingPage>(req).await
}

/// `DELETE /api/admin/landing-pages/{page_id}` — delete a page.
pub async fn delete_landing_page(page_id: Uuid) -> Result<(), String> {
    let client = create_client();
    let url = api_url(&format!("/api/admin/landing-pages/{}", page_id));
    let req = with_credentials(client.delete(&url));
    let res = req.send().await.map_err(|e| e.to_string())?;
    if res.status().is_success() { Ok(()) } else { Err(format!("HTTP {}", res.status())) }
}

// ── Variant API functions ──────────────────────────────────────────────────────

/// `GET /api/admin/landing-pages/{page_id}/variants`
pub async fn list_variants(page_id: Uuid) -> Result<Vec<PageVariant>, String> {
    let client = create_client();
    let url = api_url(&format!("/api/admin/landing-pages/{}/variants", page_id));
    let req = with_credentials(client.get(&url));
    api_request::<Vec<PageVariant>>(req).await
}

/// `POST /api/admin/landing-pages/{page_id}/variants`
pub async fn create_variant(page_id: Uuid, payload: CreateVariantPayload) -> Result<PageVariant, String> {
    let client = create_client();
    let url = api_url(&format!("/api/admin/landing-pages/{}/variants", page_id));
    let req = with_credentials(client.post(&url).json(&payload));
    api_request::<PageVariant>(req).await
}

/// `PUT /api/admin/landing-pages/{page_id}/variants/{variant_id}`
pub async fn update_variant(
    page_id: Uuid,
    variant_id: Uuid,
    payload: UpdateVariantPayload,
) -> Result<PageVariant, String> {
    let client = create_client();
    let url = api_url(&format!("/api/admin/landing-pages/{}/variants/{}", page_id, variant_id));
    let req = with_credentials(client.put(&url).json(&payload));
    api_request::<PageVariant>(req).await
}

/// `DELETE /api/admin/landing-pages/{page_id}/variants/{variant_id}`
pub async fn delete_variant(page_id: Uuid, variant_id: Uuid) -> Result<(), String> {
    let client = create_client();
    let url = api_url(&format!("/api/admin/landing-pages/{}/variants/{}", page_id, variant_id));
    let req = with_credentials(client.delete(&url));
    let res = req.send().await.map_err(|e| e.to_string())?;
    if res.status().is_success() { Ok(()) } else { Err(format!("HTTP {}", res.status())) }
}

/// `POST /api/admin/landing-pages/{page_id}/variants/{variant_id}/promote`
pub async fn promote_variant(page_id: Uuid, variant_id: Uuid) -> Result<LandingPage, String> {
    let client = create_client();
    let url = api_url(&format!(
        "/api/admin/landing-pages/{}/variants/{}/promote",
        page_id, variant_id
    ));
    let req = with_credentials(client.post(&url));
    api_request::<LandingPage>(req).await
}

// ── UTM Preset API functions ───────────────────────────────────────────────────

/// `GET /api/admin/utm-presets?app_id={app_id}`
pub async fn list_utm_presets(app_id: &str) -> Result<Vec<UtmPreset>, String> {
    let client = create_client();
    let url = api_url(&format!("/api/admin/utm-presets?app_id={}", app_id));
    let req = with_credentials(client.get(&url));
    api_request::<Vec<UtmPreset>>(req).await
}

/// `POST /api/admin/utm-presets`
pub async fn create_utm_preset(payload: CreateUtmPresetPayload) -> Result<UtmPreset, String> {
    let client = create_client();
    let url = api_url("/api/admin/utm-presets");
    let req = with_credentials(client.post(&url).json(&payload));
    api_request::<UtmPreset>(req).await
}

/// `DELETE /api/admin/utm-presets/{preset_id}`
pub async fn delete_utm_preset(preset_id: Uuid) -> Result<(), String> {
    let client = create_client();
    let url = api_url(&format!("/api/admin/utm-presets/{}", preset_id));
    let req = with_credentials(client.delete(&url));
    let res = req.send().await.map_err(|e| e.to_string())?;
    if res.status().is_success() { Ok(()) } else { Err(format!("HTTP {}", res.status())) }
}

// ── Pixel tracking API ────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct PixelConfig {
    pub enabled: bool,
    pub snippet: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PagePixelConfig {
    pub ga4:      PixelConfig,
    pub meta:     PixelConfig,
    pub linkedin: PixelConfig,
    pub gtm:      PixelConfig,
}

impl Default for PagePixelConfig {
    fn default() -> Self {
        Self {
            ga4:      PixelConfig::default(),
            meta:     PixelConfig::default(),
            linkedin: PixelConfig::default(),
            gtm:      PixelConfig::default(),
        }
    }
}

#[derive(Debug, Serialize)]
pub struct SetPixelPayload {
    pub enabled: bool,
    pub snippet: Option<String>,
}

/// `GET /api/admin/landing-pages/{page_id}/pixels`
pub async fn get_page_pixels(page_id: Uuid) -> Result<PagePixelConfig, String> {
    let client = create_client();
    let url = api_url(&format!("/api/admin/landing-pages/{}/pixels", page_id));
    let req = with_credentials(client.get(&url));
    api_request::<PagePixelConfig>(req).await
}

/// `PUT /api/admin/landing-pages/{page_id}/pixels/{pixel_type}`
pub async fn set_pixel(
    page_id: Uuid,
    pixel_type: &str,
    enabled: bool,
    snippet: Option<String>,
) -> Result<PagePixelConfig, String> {
    let client = create_client();
    let url = api_url(&format!("/api/admin/landing-pages/{}/pixels/{}", page_id, pixel_type));
    let req = with_credentials(client.put(&url).json(&SetPixelPayload { enabled, snippet }));
    api_request::<PagePixelConfig>(req).await
}

// ── Funnel Analytics API ──────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SourceBreakdown {
    pub source: String,
    pub views:  i64,
    pub leads:  i64,
    pub pct:    i32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PageAnalytics {
    pub page_id:       Uuid,
    pub total_views:   i64,
    pub total_leads:   i64,
    pub cta_clicks:    i64,
    pub conv_rate_pct: f64,
    pub sources:       Vec<SourceBreakdown>,
}

/// `GET /api/admin/landing-pages/{page_id}/analytics`
pub async fn get_page_analytics(page_id: Uuid) -> Result<PageAnalytics, String> {
    let client = create_client();
    let url = api_url(&format!("/api/admin/landing-pages/{}/analytics", page_id));
    let req = with_credentials(client.get(&url));
    api_request::<PageAnalytics>(req).await
}
