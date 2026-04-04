use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};
use sea_orm::Set;
use uuid::Uuid;
use chrono::{DateTime, Utc};
use crate::traits::file::FileAssociable;
use crate::models::file::{FileAssociation, FileModel};
use crate::entities::{file_association,file}; 


#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "user")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: Uuid,
    pub username: String,
    pub first_name: String,
    pub last_name: String,
    pub email: String,
    pub phone: String,
    #[sea_orm(column_type = "TimestampWithTimeZone")]
    pub last_login: Option<DateTime<Utc>>,
    pub password_hash: String,
    pub is_admin: bool,
    pub is_active: bool,
    #[sea_orm(column_type = "TimestampWithTimeZone")]
    pub created_at: DateTime<Utc>,
    #[sea_orm(column_type = "TimestampWithTimeZone")]
    pub updated_at: DateTime<Utc>,
}

#[derive(Copy, Clone, Debug, EnumIter)]
pub enum Relation {
    UserAccount,
    Session,
    RequestLog,
    FileAssociation,
    Passkey,
    MagicLinkToken,
}

impl RelationTrait for Relation {
    fn def(&self) -> RelationDef {
        match self {
            Self::UserAccount => Entity::has_many(super::user_account::Entity).into(),
            Self::Session => Entity::has_many(super::session::Entity).into(),
            Self::RequestLog => Entity::has_many(super::request_log::Entity).into(), 
            Self::FileAssociation => Entity::has_many(super::file_association::Entity).into(),
            Self::Passkey => Entity::has_many(super::passkey::Entity).into(),
            Self::MagicLinkToken => Entity::has_many(super::magic_link_token::Entity).into(),
        }
    }
}

impl Related<super::account::Entity> for Entity {
    fn to() -> RelationDef {
        super::user_account::Relation::Account.def()
    }

    fn via() -> Option<RelationDef> {
        Some(super::user_account::Relation::User.def().rev())
    }
}

impl Related<super::session::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Session.def()
    }
}

impl Related<super::request_log::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::RequestLog.def()
    }
}

impl Related<super::file_association::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::FileAssociation.def()
    }
}

impl Related<super::passkey::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Passkey.def()
    }
}

impl Related<super::magic_link_token::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::MagicLinkToken.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}


impl FileAssociable for Entity {
    fn entity_type() -> &'static str {
        "User"
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
