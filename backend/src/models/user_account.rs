use chrono::{Utc, DateTime};
use uuid::Uuid;
use crate::entities::user_account::UserRole;
use serde::{Serialize, Deserialize};

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