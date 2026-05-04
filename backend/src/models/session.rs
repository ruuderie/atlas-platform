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

#[derive(Serialize, Deserialize, Debug)]
pub struct UserInfo {
    pub id: Uuid,
    pub email: String,
    pub first_name: String,
    pub last_name: String,
    pub is_admin: bool,
    pub app_permissions: Vec<String>,
}