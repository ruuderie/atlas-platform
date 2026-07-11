use crate::api::client::{api_get, api_url, create_client, with_credentials};
use serde::{Deserialize, Serialize};

// ──────────────────────────────────────────────────────────────────────────────
// TYPES (mirror backend SeedPackInfo / SeedApplyResponse)
// ──────────────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SeedPackInfo {
    pub id: String,
    pub title: String,
    pub description: String,
    pub content_summary: String,
    /// ISO 8601 timestamp of the most recent application, if ever applied.
    pub last_applied_at: Option<String>,
    /// Number of times this pack has been applied.
    pub apply_count: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SeedApplyResponse {
    pub seed_id: String,
    pub success: bool,
    pub message: String,
}

// ──────────────────────────────────────────────────────────────────────────────
// API CALLS
// ──────────────────────────────────────────────────────────────────────────────

/// Fetch all seed packs available for an app instance, including applied status.
pub async fn get_seed_packs(app_instance_id: &str) -> Result<Vec<SeedPackInfo>, String> {
    let path = format!("api/app-instances/{}/seeds", app_instance_id);
    api_get::<Vec<SeedPackInfo>>(&path).await
}

/// Apply a seed pack to an app instance. Re-application is allowed.
pub async fn apply_seed_pack(
    app_instance_id: &str,
    seed_id: &str,
) -> Result<SeedApplyResponse, String> {
    let client = create_client();
    let url = api_url(&format!(
        "api/app-instances/{}/seeds/{}/apply",
        app_instance_id, seed_id
    ));
    let req = with_credentials(client.post(&url).json(&serde_json::json!({})));
    let res = req.send().await.map_err(|e| e.to_string())?;

    if res.status().is_success() {
        res.json::<SeedApplyResponse>()
            .await
            .map_err(|e| e.to_string())
    } else {
        Err(format!("HTTP {}", res.status()))
    }
}
