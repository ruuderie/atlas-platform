//! G-32 RBAC client for platform-admin Team → Roles tab.

use super::client::{api_request, api_url, create_client, with_credentials};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RoleProfileSummary {
    pub id: Uuid,
    pub role_slug: String,
    pub display_name: String,
    pub description: Option<String>,
    pub is_platform_default: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct UserRoleResponse {
    pub user_id: Uuid,
    pub app_slug: String,
    pub role_slug: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssignRoleInput {
    pub app_slug: String,
    pub role_slug: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssignRoleResponse {
    pub assignment_id: Uuid,
}

pub async fn list_role_profiles(app_slug: &str) -> Result<Vec<RoleProfileSummary>, String> {
    let url = api_url(&format!("/api/rbac/roles?app_slug={}", urlencoding(app_slug)));
    let client = create_client();
    let req = with_credentials(client.get(&url));
    api_request(req).await
}

pub async fn get_user_role(user_id: Uuid, app_slug: &str) -> Result<UserRoleResponse, String> {
    let url = api_url(&format!(
        "/api/rbac/users/{}/roles?app_slug={}",
        user_id,
        urlencoding(app_slug)
    ));
    let client = create_client();
    let req = with_credentials(client.get(&url));
    api_request(req).await
}

pub async fn assign_role(user_id: Uuid, input: AssignRoleInput) -> Result<AssignRoleResponse, String> {
    let url = api_url(&format!("/api/rbac/users/{}/roles", user_id));
    let client = create_client();
    let req = with_credentials(client.post(&url).json(&input));
    api_request(req).await
}

pub async fn revoke_role(user_id: Uuid, app_slug: &str) -> Result<(), String> {
    let url = api_url(&format!(
        "/api/rbac/users/{}/roles/{}",
        user_id,
        urlencoding(app_slug)
    ));
    let client = create_client();
    let req = with_credentials(client.delete(&url));
    let res = req.send().await.map_err(|e| e.to_string())?;
    if res.status().is_success() {
        Ok(())
    } else {
        Err(format!("Failed to revoke role: {}", res.status()))
    }
}

fn urlencoding(s: &str) -> String {
    s.chars()
        .map(|c| match c {
            'A'..='Z' | 'a'..='z' | '0'..='9' | '-' | '_' | '.' | '~' => c.to_string(),
            _ => format!("%{:02X}", c as u8),
        })
        .collect()
}
