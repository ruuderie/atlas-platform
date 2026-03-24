use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};
use serde_json::Value;
use crate::traits::file::FileAssociable;
use crate::models::file::{FileAssociation, FileModel};
use crate::entities::{file_association,file, deal_contact, contact}; 
use sea_orm::Set;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "deal")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: Uuid,
    pub customer_id: Uuid,  // Reference to the customer
    pub name: String,
    pub amount: f64,  // Deal amount
    pub status: String,  // e.g., "Prospecting", "Qualification", "Closed Won", "Closed Lost"
    pub stage: String,  // Current stage in the sales process
    pub close_date: Option<DateTime<Utc>>,  // Expected or actual close date
    pub is_active: bool,
    #[sea_orm(column_type = "TimestampWithTimeZone")]
    pub created_at: DateTime<Utc>,
    #[sea_orm(column_type = "TimestampWithTimeZone")]
    pub updated_at: DateTime<Utc>,
    pub directory_id: Option<Uuid>,
    #[sea_orm(column_type = "JsonBinary", nullable)]
    pub properties: Option<Value>,
}

#[derive(Copy, Clone, Debug, EnumIter)]
pub enum Relation {
    Customer,
    DealContact,
    FileAssociation,
    Note,
    Activity,
}

impl RelationTrait for Relation {
    fn def(&self) -> RelationDef {
        match self {
            Self::Customer => Entity::belongs_to(super::customer::Entity)
                .from(Column::CustomerId)
                .to(super::customer::Column::Id)
                .into(),
            Self::DealContact => Entity::has_many(super::deal_contact::Entity).into(),
            Self::FileAssociation => Entity::has_many(super::file_association::Entity).into(),
            Self::Note => Entity::has_many(super::note::Entity).into(),
            Self::Activity => Entity::has_many(super::activity::Entity).into(),
        }
    }
}

impl Related<super::customer::Entity> for Entity {
    fn to() -> RelationDef {
        Entity::belongs_to(super::customer::Entity)
            .from(Column::CustomerId)
            .to(super::customer::Column::Id)
            .into()
    }
}

impl Related<super::note::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Note.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}

impl FileAssociable for Entity {
    fn entity_type() -> &'static str {
        "Deal"
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

impl Model {
    pub async fn add_contact(&self, db: &DatabaseConnection, contact: &contact::Model) -> Result<(), DbErr> {
        let deal_contact = deal_contact::ActiveModel {
            deal_id: Set(self.id),
            contact_id: Set(contact.id),
        };
        deal_contact.insert(db).await?;
        Ok(())
    }

    pub async fn remove_contact(&self, db: &DatabaseConnection, contact: &contact::Model) -> Result<(), DbErr> {
        deal_contact::Entity::delete_many()
            .filter(deal_contact::Column::DealId.eq(self.id))
            .filter(deal_contact::Column::ContactId.eq(contact.id))
            .exec(db)
            .await?;
        Ok(())
    }

    pub async fn get_contacts(&self, db: &DatabaseConnection) -> Result<Vec<contact::Model>, DbErr> {
        let deal_contacts = deal_contact::Entity::find()
            .filter(deal_contact::Column::DealId.eq(self.id))
            .all(db)
            .await?;

        let contact_ids: Vec<Uuid> = deal_contacts.into_iter()
            .map(|dc| dc.contact_id)
            .collect();

        let contacts = contact::Entity::find()
            .filter(contact::Column::Id.is_in(contact_ids))
            .all(db)
            .await?;

        Ok(contacts)
    }
}

impl Related<super::activity::Entity> for Entity {
    fn to() -> RelationDef {
        Entity::has_many(super::activity::Entity)
            .from(Column::Id)
            .to(super::activity::Column::DealId)
            .into()
    }
}
