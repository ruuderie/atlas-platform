#![allow(dead_code, unused)]
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Serialize, Deserialize, Debug)]
pub struct SessionResponse {
    pub user: Option<UserInfo>,
    #[serde(skip_serializing)]
    pub token: String,
    #[serde(skip_serializing)]
    pub refresh_token: String,
}

/// A flattened view of one `user_app_permission` row, safe to include in the
/// login response so clients can make immediate feature-gate decisions without
/// an extra round-trip.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AppPermission {
    pub tenant_id: Uuid,
    pub app_slug: String,
    /// Raw JSON permissions blob (e.g. `["read","write"]`) from the DB.
    pub permissions: serde_json::Value,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct UserInfo {
    pub id: Uuid,
    pub email: String,
    pub first_name: String,
    pub last_name: String,
    pub is_admin: bool,
    pub app_permissions: Vec<AppPermission>,
}
