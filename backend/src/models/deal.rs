use chrono::{DateTime, Utc};
use uuid::Uuid;
use serde::{Serialize, Deserialize};
use crate::entities::deal;
use crate::models::file::FileModel;
#[derive(Debug, Serialize, Deserialize)]
pub struct DealModel {
    pub id: Uuid,
    pub customer_id: Uuid,
    pub name: String,
    pub amount: f64,
    pub status: String,
    pub stage: String,
    pub close_date: Option<DateTime<Utc>>,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub files: Vec<FileModel>,
}

#[derive(Debug, Deserialize)]
pub struct CreateDealInput {
    pub customer_id: Uuid,
    pub name: String,
    pub amount: f64,
    pub status: String,
    pub stage: String,
    pub close_date: Option<DateTime<Utc>>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateDealInput {
    pub name: Option<String>,
    pub amount: Option<f64>,
    pub status: Option<String>,
    pub stage: Option<String>,
    pub close_date: Option<DateTime<Utc>>,
    pub is_active: Option<bool>,
}

impl From<deal::Model> for DealModel {
    fn from(deal: deal::Model) -> Self {
        Self {
            id: deal.id,
            customer_id: deal.customer_id,
            name: deal.name,
            amount: deal.amount,
            status: deal.status,
            stage: deal.stage,
            close_date: deal.close_date,
            is_active: deal.is_active,
            created_at: deal.created_at,
            updated_at: deal.updated_at,
            files: vec![], // This will be populated when needed
        }
    }
}
