use crate::api::client::{api_get, api_url, create_client, with_credentials};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// ──────────────────────────────────────────────────────────────────────────────
// SHARED TYPES (mirror the backend response structs)
// ──────────────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct OnboardingStepStatus {
    pub id: String,
    pub title: String,
    pub description: String,
    pub is_required: bool,
    /// Explicit display order — sort by this field, not Vec index.
    pub position: u8,
    pub is_complete: bool,
    pub is_skipped: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct OnboardingStatusResponse {
    pub app_instance_id: Uuid,
    pub tenant_id: Uuid,
    pub app_type: String,
    pub steps: Vec<OnboardingStepStatus>,
    pub is_ready: bool,
    pub dismissed_at: Option<String>,
}

// ──────────────────────────────────────────────────────────────────────────────
// PLATFORM ADMIN API CALLS (authenticated session)
// ──────────────────────────────────────────────────────────────────────────────

/// Fetch onboarding status for an app instance (platform admin view).
pub async fn get_onboarding_status(
    app_instance_id: &str,
) -> Result<OnboardingStatusResponse, String> {
    let path = format!("api/onboarding/{}", app_instance_id);
    api_get::<OnboardingStatusResponse>(&path).await
}

/// Mark a custom step complete.
pub async fn complete_step(app_instance_id: &str, step_id: &str) -> Result<(), String> {
    let client = create_client();
    let url = api_url(&format!(
        "api/onboarding/{}/complete/{}",
        app_instance_id, step_id
    ));
    let req = client.post(&url);
    let req = with_credentials(req);
    let res = req.send().await.map_err(|e| e.to_string())?;
    if res.status().is_success() {
        Ok(())
    } else {
        Err(format!("HTTP {}", res.status()))
    }
}

/// Skip an optional step.
pub async fn skip_step(app_instance_id: &str, step_id: &str) -> Result<(), String> {
    let client = create_client();
    let url = api_url(&format!(
        "api/onboarding/{}/skip/{}",
        app_instance_id, step_id
    ));
    let req = client.post(&url);
    let req = with_credentials(req);
    let res = req.send().await.map_err(|e| e.to_string())?;
    if res.status().is_success() {
        Ok(())
    } else {
        Err(format!("HTTP {}", res.status()))
    }
}

/// Dismiss the full-page wizard takeover ("I'll do this later").
pub async fn dismiss_wizard(app_instance_id: &str) -> Result<(), String> {
    let client = create_client();
    let url = api_url(&format!("api/onboarding/{}/dismiss", app_instance_id));
    let req = client.post(&url).json(&serde_json::json!({}));
    let req = with_credentials(req);
    let res = req.send().await.map_err(|e| e.to_string())?;
    if res.status().is_success() {
        Ok(())
    } else {
        Err(format!("HTTP {}", res.status()))
    }
}
