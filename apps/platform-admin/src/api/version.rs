use super::client::{api_url, create_client, api_request};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct VersionInfo {
    pub version: String,
    pub build_sha: String,
    pub build_date: String,
}

/// `GET /api/version` — no auth required.
pub async fn get_version() -> Result<VersionInfo, String> {
    let client = create_client();
    let url = api_url("/api/version");
    let req = client.get(&url);
    api_request::<VersionInfo>(req).await
}
