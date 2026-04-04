use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};
use crate::traits::file::FileAssociable;
use crate::models::file::{FileAssociation, FileModel};
use crate::entities::{file_association,file}; 
use sea_orm::Set;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "activity")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: Uuid,
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
    #[sea_orm(column_type = "Json")]
    pub associated_entities: Json,
    pub created_by: Uuid,
    pub assigned_to: Option<Uuid>,
    #[sea_orm(column_type = "TimestampWithTimeZone")]
    pub created_at: DateTime<Utc>,
    #[sea_orm(column_type = "TimestampWithTimeZone")]
    pub updated_at: DateTime<Utc>,
}

#[derive(Clone, Debug, PartialEq, Eq, EnumIter, DeriveActiveEnum, Serialize, Deserialize)]
#[sea_orm(rs_type = "String", db_type = "String(StringLen::N(50))")]
pub enum ActivityType {
    #[sea_orm(string_value = "PhoneCall")]
    PhoneCall,
    #[sea_orm(string_value = "Email")]
    Email,
    #[sea_orm(string_value = "Meeting")]
    Meeting,
    #[sea_orm(string_value = "Note")]
    Note,
    #[sea_orm(string_value = "Task")]
    Task,
    #[sea_orm(string_value = "Reminder")]
    Reminder,
    #[sea_orm(string_value = "Other")]
    Other,
}

#[derive(Clone, Debug, PartialEq, Eq, EnumIter, DeriveActiveEnum, Serialize, Deserialize)]
#[sea_orm(rs_type = "String", db_type = "String(StringLen::N(50))")]
pub enum ActivityStatus {
    #[sea_orm(string_value = "Pending")]
    Pending,
    #[sea_orm(string_value = "InProgress")]
    InProgress,
    #[sea_orm(string_value = "Completed")]
    Completed,
    #[sea_orm(string_value = "Cancelled")]
    Cancelled,
}

#[derive(Clone, Debug, PartialEq, Eq, EnumIter, DeriveActiveEnum, Serialize, Deserialize)]
#[sea_orm(rs_type = "String", db_type = "String(StringLen::N(50))")]
pub enum AssociatedEntityType {
    #[sea_orm(string_value = "Account")]
    Account,
    #[sea_orm(string_value = "Customer")]
    Customer,
    #[sea_orm(string_value = "Lead")]
    Lead,
    #[sea_orm(string_value = "Deal")]
    Deal,
    #[sea_orm(string_value = "Case")]
    Case,
    #[sea_orm(string_value = "Contact")]
    Contact,
}

#[derive(Copy, Clone, Debug, EnumIter)]
pub enum Relation {
    FileAssociation,
    Account,
    CreatedBy,
    AssignedTo,
    Deal,
    Customer,
    Lead,
    Contact,
    Case,
}

impl RelationTrait for Relation {
    fn def(&self) -> RelationDef {
        match self {
            Self::Account => Entity::belongs_to(super::account::Entity)
                .from(Column::AccountId)
                .to(super::account::Column::Id)
                .into(),
            Self::CreatedBy => Entity::belongs_to(super::user::Entity)
                .from(Column::CreatedBy)
                .to(super::user::Column::Id)
                .into(),
            Self::AssignedTo => Entity::belongs_to(super::user::Entity)
                .from(Column::AssignedTo)
                .to(super::user::Column::Id)
                .into(),
            Self::Deal => Entity::belongs_to(super::deal::Entity)
                .from(Column::DealId)
                .to(super::deal::Column::Id)
                .into(),
            Self::Customer => Entity::belongs_to(super::customer::Entity)
                .from(Column::CustomerId)
                .to(super::customer::Column::Id)
                .into(),
            Self::Lead => Entity::belongs_to(super::lead::Entity)
                .from(Column::LeadId)
                .to(super::lead::Column::Id)
                .into(),
            Self::Contact => Entity::belongs_to(super::contact::Entity)
                .from(Column::ContactId)
                .to(super::contact::Column::Id)
                .into(),
            Self::Case => Entity::belongs_to(super::case::Entity)
                .from(Column::CaseId)
                .to(super::case::Column::Id)
                .into(),
            Self::FileAssociation => Entity::has_many(super::file_association::Entity).into(),
        }
    }
}

impl Related<super::account::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Account.def()
    }
}
impl Related<super::lead::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Lead.def()
    }
}
impl Related<super::deal::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Deal.def()
    }
}

impl Related<super::user::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::CreatedBy.def()
    }
}

impl Related<super::case::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Case.def()
    }
}

impl Related<super::contact::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Contact.def()
    }
}

impl Related<super::customer::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Customer.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssociatedEntity {
    pub entity_type: AssociatedEntityType,
    pub entity_id: Uuid,
}

// Add methods to the Model for working with associated entities
impl Model {
    pub fn get_associated_entities(&self) -> Result<Vec<AssociatedEntity>, serde_json::Error> {
        serde_json::from_value(self.associated_entities.clone())
    }

    pub fn set_associated_entities(&mut self, entities: Vec<AssociatedEntity>) -> Result<(), serde_json::Error> {
        self.associated_entities = serde_json::to_value(entities)?;
        Ok(())
    }

    pub fn add_associated_entity(&mut self, entity: AssociatedEntity) -> Result<(), serde_json::Error> {
        let mut entities = self.get_associated_entities()?;
        entities.push(entity);
        self.set_associated_entities(entities)
    }
}

impl FileAssociable for Entity {
    fn entity_type() -> &'static str {
        "Activity"
    }
}

impl FileAssociation for Model {
    async fn add_file(&self, db: &DatabaseConnection, file_id: Uuid) -> Result<(), DbErr> {
        let file = file::Entity::find_by_id(file_id.to_string())
            .one(db)
            .await?
            .ok_or_else(|| DbErr::Custom("File not found".to_string()))?;

        file_association::ActiveModel {
            id: Set(Uuid::new_v4()),
            file_id: Set(file.id),
            associated_entity_type: Set(Entity::entity_type().to_string()),
            associated_entity_id: Set(self.id),
        }.insert(db).await?;

        Ok(())
    }

    async fn remove_file(&self, db: &DatabaseConnection, file_id: Uuid) -> Result<(), DbErr> {
        file_association::Entity::delete_many()
            .filter(file_association::Column::FileId.eq(file_id.to_string()))
            .filter(file_association::Column::AssociatedEntityType.eq(Entity::entity_type()))
            .filter(file_association::Column::AssociatedEntityId.eq(self.id))
            .exec(db)
            .await?;
        Ok(())
    }

    async fn get_associated_files(&self, db: &DatabaseConnection) -> Result<Vec<FileModel>, DbErr> {
        let associations = file_association::Entity::find()
            .filter(file_association::Column::AssociatedEntityType.eq(Entity::entity_type()))
            .filter(file_association::Column::AssociatedEntityId.eq(self.id))
            .all(db)
            .await?;

        let file_ids: Vec<String> = associations.into_iter().map(|a| a.file_id).collect();
        let files = file::Entity::find()
            .filter(file::Column::Id.is_in(file_ids))
            .all(db)
            .await?;

        Ok(files.into_iter().map(FileModel::from).collect())
    }
}
