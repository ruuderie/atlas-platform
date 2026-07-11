#![allow(dead_code, unused_imports)]
use crate::entities::{file, file_association};
use crate::models::file::{FileAssociation, FileModel};
use crate::traits::file::FileAssociable;
use chrono::{DateTime, Utc};
use sea_orm::Set;
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "notes")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: Uuid,
    pub content: String,
    pub created_by: Uuid,
    pub entity_type: String,
    pub entity_id: Uuid,
    pub tenant_id: Option<Uuid>,
    pub is_private: bool,
    #[sea_orm(column_type = "TimestampWithTimeZone")]
    pub created_at: DateTime<Utc>,
    #[sea_orm(column_type = "TimestampWithTimeZone")]
    pub updated_at: DateTime<Utc>,
}

#[derive(Copy, Clone, Debug, EnumIter)]
pub enum Relation {
    User,
    File,
    Deal,
    Customer,
    Lead,
    Contact,
    Case,
    Activity,
    Tenant,
}

impl RelationTrait for Relation {
    fn def(&self) -> RelationDef {
        match self {
            Self::User => Entity::belongs_to(super::user::Entity)
                .from(Column::CreatedBy)
                .to(super::user::Column::Id)
                .into(),
            Self::File => Entity::belongs_to(super::file::Entity).into(),
            Self::Deal => Entity::belongs_to(super::deal::Entity)
                .from(Column::EntityId)
                .to(super::deal::Column::Id)
                .into(),
            Self::Customer => Entity::belongs_to(super::customer::Entity)
                .from(Column::EntityId)
                .to(super::customer::Column::Id)
                .into(),
            Self::Lead => Entity::belongs_to(super::lead::Entity)
                .from(Column::EntityId)
                .to(super::lead::Column::Id)
                .into(),
            Self::Contact => Entity::belongs_to(super::contact::Entity)
                .from(Column::EntityId)
                .to(super::contact::Column::Id)
                .into(),
            Self::Case => Entity::belongs_to(super::case::Entity)
                .from(Column::EntityId)
                .to(super::case::Column::Id)
                .into(),
            Self::Activity => Entity::belongs_to(super::activity::Entity)
                .from(Column::EntityId)
                .to(super::activity::Column::Id)
                .into(),
            Self::Tenant => Entity::belongs_to(super::tenant::Entity)
                .from(Column::TenantId)
                .to(super::tenant::Column::Id)
                .into(),
        }
    }
}

impl Related<super::tenant::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Tenant.def()
    }
}

impl Related<super::user::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::User.def()
    }
}

impl Related<super::file::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::File.def()
    }
}

impl Related<super::deal::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Deal.def()
    }
}

impl Related<super::customer::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Customer.def()
    }
}

impl Related<super::lead::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Lead.def()
    }
}

impl Related<super::contact::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Contact.def()
    }
}

impl Related<super::case::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Case.def()
    }
}

impl Related<super::activity::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Activity.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}

impl Model {
    pub fn new(
        content: String,
        created_by: Uuid,
        entity_type: String,
        entity_id: Uuid,
        tenant_id: Option<Uuid>,
        is_private: bool,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            content,
            created_by,
            entity_type,
            entity_id,
            tenant_id,
            is_private,
            created_at: now,
            updated_at: now,
        }
    }
}
impl FileAssociable for Entity {
    fn entity_type() -> &'static str {
        "Note"
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
        }
        .insert(db)
        .await?;

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
