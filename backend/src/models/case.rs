use uuid::Uuid;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use crate::entities::case;
use crate::models::note::NoteModel;
use crate::models::activity::ActivityModel;
use crate::models::file::FileModel;
use crate::entities::activity::ActivityType;

#[derive(Debug, Serialize, Deserialize)]
pub struct CaseModel {
    pub id: Uuid,
    pub customer_id: Uuid,
    pub title: String,
    pub description: String,
    pub status: String,
    pub priority: String,
    pub assigned_to: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub closed_at: Option<DateTime<Utc>>,
    pub notes: Vec<NoteModel>,
    pub activities: Vec<ActivityModel>,
    pub files: Vec<FileModel>,
}

#[derive(Debug, Deserialize)]
pub struct CreateCaseInput {
    pub customer_id: Uuid,
    pub title: String,
    pub description: String,
    pub priority: String,
    pub assigned_to: Option<Uuid>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateCaseInput {
    pub title: Option<String>,
    pub description: Option<String>,
    pub status: Option<String>,
    pub priority: Option<String>,
    pub assigned_to: Option<Uuid>,
}

#[derive(Debug, Deserialize)]
pub struct CreateNoteInput {
    pub content: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateActivityInput {
    pub title: String,
    pub description: String,
    pub due_date: Option<DateTime<Utc>>,
    pub activity_type: Option<ActivityType>, // Optional, will default to Task if not provided
    pub assigned_to: Option<Uuid>,
}

#[derive(Debug, Deserialize)]
struct CreateCaseActivityInput {
    title: String,
    description: Option<String>,
}

impl From<case::Model> for CaseModel {
    fn from(case: case::Model) -> Self {
        CaseModel {
            id: case.id,
            customer_id: case.customer_id,
            title: case.title,
            description: case.description,
            status: case.status,
            priority: case.priority,
            assigned_to: case.assigned_to,
            created_at: case.created_at,
            updated_at: case.updated_at,
            closed_at: case.closed_at,
            notes: Vec::new(),
            activities: Vec::new(),
            files: Vec::new(),
        }
    }
}
