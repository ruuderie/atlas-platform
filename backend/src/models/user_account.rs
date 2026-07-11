use crate::entities::user_account::UserRole;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Deserialize, Serialize, Debug)]
pub struct UserAccountCreate {
    pub user_id: Uuid,
    pub account_id: Uuid,
    pub role: UserRole,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct UserAccountUpdate {
    pub user_id: Uuid,
    pub account_id: Uuid,
    pub role: UserRole,
    pub is_active: bool,
    pub updated_at: DateTime<Utc>,
}
