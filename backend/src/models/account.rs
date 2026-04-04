
use chrono::{DateTime, Utc};
use uuid::Uuid;
use serde::{Deserialize, Serialize};


#[derive(Debug, Serialize, Deserialize)]
pub struct AccountModel {
    pub id: Uuid,
    pub name: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreateAccountInput {
    pub name: String,
}

#[derive(Debug, Deserialize)]
pub struct UpdateAccountInput {
    pub name: String,
}