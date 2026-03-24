use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};
use serde_json::Value;
use crate::models::address::{ AddressJson};
use crate::traits::file::FileAssociable;
use crate::models::file::{FileAssociation, FileModel};
use crate::entities::{file_association,file}; 
use sea_orm::{Set};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "lead")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: Uuid,
    pub name: String,
    pub listing_id: Option<Uuid>,
    pub account_id: Option<Uuid>,
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
    pub message: Option<String>,
    pub source: Option<String>,
    pub is_converted: bool,
    pub converted_to_contact: bool,
    pub associated_deal_id: Option<Uuid>,
    pub converted_customer_id: Option<Uuid>,
    pub converted_contact_id: Option<Uuid>,
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
    Listing,
    Account,
    Activities,
    Customer,
    Contact,
    Deal,
FileAssociation,
}

impl RelationTrait for Relation {
    fn def(&self) -> RelationDef {
        match self {
            Self::Listing => Entity::belongs_to(super::listing::Entity)
                .from(Column::ListingId)
                .to(super::listing::Column::Id)
                .into(),
            Self::Account => Entity::belongs_to(super::account::Entity)
                .from(Column::AccountId)
                .to(super::account::Column::Id)
                .into(),
            Self::Activities => Entity::has_many(super::activity::Entity).into(),
            Self::Customer => Entity::belongs_to(super::customer::Entity).into(),
            Self::Contact => Entity::belongs_to(super::contact::Entity).into(),
            Self::Deal => Entity::belongs_to(super::deal::Entity)
                .from(Column::AssociatedDealId)
                .to(super::deal::Column::Id)
                .into(),
            Self::FileAssociation => Entity::has_many(super::file_association::Entity).into(),
        }
    }
}

impl Related<super::listing::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Listing.def()
    }
}

impl Related<super::account::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Account.def()
    }
}

impl Related<super::activity::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Activities.def()
    }
}

impl Related<super::customer::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Customer.def()
    }
}

impl Related<super::contact::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Contact.def()
    }
}

impl Related<super::deal::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Deal.def()
    }
}

impl Related<file::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::FileAssociation.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}

impl Model {
    pub fn convert_to_customer(&mut self) {
        self.is_converted = true;
    }

    pub fn associate_deal(&mut self, deal_id: Uuid) {
        self.associated_deal_id = Some(deal_id);
    }
}
impl FileAssociable for Entity {
    fn entity_type() -> &'static str {
        "Lead"
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

/*
    // For setting an address
    let mut contact: contact::ActiveModel = // ... get or create contact ...
    let address = Address {
        street_address: Some("123 Main St".to_string()),
        city: Some("Anytown".to_string()),
        // ... other fields ...
    };
    contact.set_billing_address(Some(address)).map_err(|e| {
        // Handle validation error
    })?;

    // For getting an address
    if let Some(contact) = contact::Entity::find_by_id(id).one(&db).await? {
        if let Some(billing_address) = contact.get_billing_address() {
            println!("Billing Address: {}", billing_address.get_full_address().unwrap_or_default());
        }
    }
*/
