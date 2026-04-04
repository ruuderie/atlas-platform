use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};
use serde_json::Value;
use crate::models::address::{  AddressJson};
use crate::traits::file::FileAssociable;
use crate::models::file::{FileAssociation, FileModel};
use crate::entities::{file_association,file}; 
use sea_orm::Set;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "contact")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: Uuid,
    pub customer_id: Option<Uuid>,
    pub name: String,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub whatsapp: Option<String>,
    pub telegram: Option<String>,
    pub twitter: Option<String>,
    pub instagram: Option<String>,
    pub facebook: Option<String>,
    #[sea_orm(column_type = "Json")]
    pub billing_address: Option<AddressJson>,
    #[sea_orm(column_type = "Json")]
    pub shipping_address: Option<AddressJson>,
    #[sea_orm(column_type = "TimestampWithTimeZone")]
    pub created_at: DateTime<Utc>,
    #[sea_orm(column_type = "TimestampWithTimeZone")]
    pub updated_at: DateTime<Utc>,
    pub tenant_id: Option<Uuid>,
    #[sea_orm(column_type = "JsonBinary", nullable)]
    pub properties: Option<Value>,
}
/*
// Setting an address
contact.billing_address = Some(AddressJson(new_address));

// Getting an address
if let Some(AddressJson(address)) = &contact.billing_address {
    println!("Full address: {}", address.get_full_address().unwrap_or_default());
}
*/

#[derive(Copy, Clone, Debug, EnumIter)]
pub enum Relation {
    Customer,
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
            Self::FileAssociation => Entity::has_many(super::file_association::Entity).into(),
            Self::Note => Entity::has_many(super::note::Entity).into(),
            Self::Activity => Entity::has_many(super::activity::Entity).into(),
        }
    }
}

impl Related<super::file_association::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::FileAssociation.def()
    }
}

impl Related<super::note::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Note.def()
    }
}

impl Related<super::activity::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Activity.def()
    }
}

impl Related<super::customer::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Customer.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}

impl FileAssociable for Entity {
    fn entity_type() -> &'static str {
        "Contact"
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
