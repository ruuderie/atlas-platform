use crate::entities::crm_status_option;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CrmStatusOptionModel {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub object_type: String, // "Lead" or "Contact"
    pub status_key: String,
    pub label: String,
    pub color: String,
    pub sort_order: i32,
    pub is_system: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreateCrmStatusOptionInput {
    pub object_type: String,
    pub status_key: String,
    pub label: String,
    pub color: Option<String>,
    pub sort_order: Option<i32>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateCrmStatusOptionInput {
    pub label: Option<String>,
    pub color: Option<String>,
    pub sort_order: Option<i32>,
}

impl From<crm_status_option::Model> for CrmStatusOptionModel {
    fn from(m: crm_status_option::Model) -> Self {
        Self {
            id: m.id,
            tenant_id: m.tenant_id,
            object_type: m.object_type,
            status_key: m.status_key,
            label: m.label,
            color: m.color,
            sort_order: m.sort_order,
            is_system: m.is_system,
            created_at: m.created_at,
            updated_at: m.updated_at,
        }
    }
}
