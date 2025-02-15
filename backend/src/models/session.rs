use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Serialize, Debug)]
pub struct SessionResponse {
    pub token: String,
    pub refresh_token: String,
    pub user: Option<UserInfo>,
}

#[derive(Serialize, Debug)]
pub struct UserInfo {
    pub id: Uuid,
    pub email: String,
    pub first_name: String,
    pub last_name: String,
    pub is_admin: bool,
}