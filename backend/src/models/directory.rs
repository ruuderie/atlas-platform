use chrono::{Utc, DateTime, Duration};
use uuid::Uuid;
use serde::{Serialize, Deserialize};
use sea_orm::prelude::*;
use sea_orm::ActiveValue::{Set, NotSet};
use crate::entities::directory;

#[derive(Debug, Deserialize, Serialize)]
pub struct DirectoryModel {
    pub id: Uuid,
    pub name: String,
    pub directory_type_id: Uuid,  // Added
    pub domain: String,
    pub description: String,  // Added
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<directory::Model> for DirectoryModel {
    fn from(model: directory::Model) -> Self {
        Self {
            id: model.id,
            name: model.name,
            directory_type_id: model.directory_type_id,
            domain: model.domain,
            description: model.description,
            created_at: model.created_at,
            updated_at: model.updated_at,
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct CreateDirectory {
    pub name: String,
    pub directory_type_id: Uuid,
    pub domain: String,
    pub description: String,
}

impl From<CreateDirectory> for directory::ActiveModel {
    fn from(input: CreateDirectory) -> Self {
        Self {
            id: Set(Uuid::new_v4()),
            name: Set(input.name),
            directory_type_id: Set(input.directory_type_id),
            domain: Set(input.domain),
            description: Set(input.description),
            created_at: Set(Utc::now()),
            updated_at: Set(Utc::now()),
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct UpdateDirectory {
    pub name: Option<String>,
    pub directory_type_id: Option<Uuid>,
    pub domain: Option<String>,
    pub description: Option<String>,
}
/* Not Needed anymore
impl From<UpdateDirectory> for directory::ActiveModel {
    fn from(input: UpdateDirectory) -> Self {
        Self {
            id: Set(input.id),
            name: Set(input.name.unwrap_or_default()),
            directory_type_id: Set(input.directory_type_id.unwrap_or_default()),
            domain: Set(input.domain.unwrap_or_default()),
            description: Set(input.description.unwrap_or_default()),
            updated_at: Set(Utc::now()),
            created_at: NotSet,  // Add this line
        }
    }
} */