use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Asset {
    pub id: Uuid,
    pub name: String,
    pub asset_type: String,
    pub address_line1: Option<String>,
    pub city: Option<String>,
    pub status: String,
}
