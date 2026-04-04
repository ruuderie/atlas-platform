use chrono::{Utc, DateTime};
use uuid::Uuid;
use serde::{Serialize, Deserialize};
use serde_json::Value;
use sea_orm::prelude::*;

#[derive(Debug, Deserialize, Serialize)]
pub struct TemplateModel {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub category_id: Uuid,
    pub name: String,
    pub description: String,
    pub template_type: String,
    pub is_active: bool,
    pub attributes_schema: Option<Value>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct CreateTemplate {
    pub name: String,
    pub tenant_id: Uuid,
    pub category_id: Uuid,
    pub description: String,
    pub template_type: String,
    pub is_active: bool,
    pub attributes_schema: Option<Value>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct UpdateTemplate {
    pub id: Uuid,
    pub name: String,
    pub tenant_id: Uuid,
    pub description: String,
    pub template_type: String,
    pub is_active: bool,
    pub attributes_schema: Option<Value>,
}

