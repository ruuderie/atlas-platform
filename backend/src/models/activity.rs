use uuid::Uuid;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sea_orm::DatabaseConnection;
use crate::entities::activity::{ActivityType, ActivityStatus, AssociatedEntity, AssociatedEntityType};
use crate::models::file::{FileModel, FileAssociation};
use crate::traits::file::FileAssociable;
#[derive(Debug, Serialize, Deserialize)]
pub struct ActivityModel {
    pub id: Uuid,
    pub tenant_id: Option<Uuid>,
    pub account_id: Option<Uuid>,
    pub deal_id: Option<Uuid>,
    pub customer_id: Option<Uuid>,
    pub lead_id: Option<Uuid>,
    pub contact_id: Option<Uuid>,
    pub case_id: Option<Uuid>,
    pub activity_type: ActivityType,
    pub title: String,
    pub description: Option<String>,
    pub status: ActivityStatus,
    pub due_date: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub associated_entities: Vec<AssociatedEntity>,
    pub created_by: Uuid,
    pub assigned_to: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub files: Vec<FileModel>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AssociatedEntityModel {
    pub entity_type: AssociatedEntityType,
    pub entity_id: Uuid,
}

#[derive(Debug, Deserialize)]
pub struct CreateActivityInput {
    pub account_id: Option<Uuid>,
    pub deal_id: Option<Uuid>,
    pub customer_id: Option<Uuid>,
    pub lead_id: Option<Uuid>,
    pub contact_id: Option<Uuid>,
    pub case_id: Option<Uuid>,
    pub activity_type: ActivityType,
    pub title: String,
    pub description: Option<String>,
    pub status: ActivityStatus,
    pub due_date: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub associated_entities: Vec<AssociatedEntity>,
    pub assigned_to: Option<Uuid>,
    pub files: Vec<FileModel>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateActivityInput {
    pub deal_id: Option<Uuid>,
    pub customer_id: Option<Uuid>,
    pub lead_id: Option<Uuid>,
    pub contact_id: Option<Uuid>,
    pub case_id: Option<Uuid>,
    pub activity_type: Option<ActivityType>,
    pub title: Option<String>,
    pub description: Option<String>,
    pub status: Option<ActivityStatus>,
    pub due_date: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub associated_entities: Option<Vec<AssociatedEntity>>,
    pub assigned_to: Option<Uuid>,
    pub files: Option<Vec<Uuid>>,
}

impl From<crate::entities::activity::Model> for ActivityModel {
    fn from(activity: crate::entities::activity::Model) -> Self {
        let associated_entities = activity.get_associated_entities().unwrap_or_default();
        Self {
            id: activity.id,
            tenant_id: activity.tenant_id,
            account_id: activity.account_id,
            deal_id: activity.deal_id,
            customer_id: activity.customer_id,
            lead_id: activity.lead_id,
            contact_id: activity.contact_id,
            case_id: activity.case_id,
            activity_type: activity.activity_type,
            title: activity.title,
            description: activity.description,
            status: activity.status,
            due_date: activity.due_date,
            completed_at: activity.completed_at,
            associated_entities: associated_entities.into_iter().map(|entity| entity.into()).collect(),
            created_by: activity.created_by,
            assigned_to: activity.assigned_to,
            created_at: activity.created_at,
            updated_at: activity.updated_at,
            files: Vec::new(), // Initialize with empty vec
        }
    }
}

impl ActivityModel {
    pub async fn from_with_files(activity: crate::entities::activity::Model, db: &DatabaseConnection) -> Self {
        let mut model = Self::from(activity.clone());
        model.files = activity.get_associated_files(db).await.unwrap_or_default();
        model
    }
}
