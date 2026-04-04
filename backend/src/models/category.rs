use crate::entities::category;
use chrono::{DateTime, Utc};
use uuid::Uuid;
use serde::{Deserialize, Serialize};


#[derive(Debug, Serialize, Deserialize)]
pub struct CategoryModel {
    pub id: Uuid,

    pub tenant_id: Option<Uuid>,
    pub parent_category_id: Option<Uuid>,
    pub name: String,
    pub description: String,
    pub is_custom: bool,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct CreateCategory {
    pub tenant_id: Option<Uuid>,
    pub parent_category_id: Option<Uuid>,
    pub name: String,
    pub description: String,
    pub is_custom: bool,
    pub is_active: bool,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct UpdateCategory {

    pub tenant_id: Option<Uuid>,
    pub parent_category_id: Option<Uuid>,
    pub name: Option<String>,
    pub description: Option<String>,
    pub is_custom: Option<bool>,
    pub is_active: Option<bool>,
}

impl From<category::Model> for CategoryModel {
    fn from(category: category::Model) -> Self {
        Self {
            id: category.id,

            tenant_id: category.tenant_id,
            parent_category_id: category.parent_category_id,
            name: category.name,
            description: category.description,
            is_custom: category.is_custom,
            is_active: category.is_active,
            created_at: category.created_at,
            updated_at: category.updated_at,
        }
    }
}

