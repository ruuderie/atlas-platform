use sea_orm::entity::prelude::*;
use sea_orm::FromJsonQueryResult;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};
use serde_json::Value;
use crate::models::address::AddressJson;
use crate::traits::file::FileAssociable;
use crate::models::file::{FileAssociation, FileModel};
use crate::entities::{file_association,file}; 
use sea_orm::Set;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "customer")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: Uuid,
    pub name: String,
    pub primary_contact_id: Option<Uuid>,
    pub customer_type: CustomerType,
    #[sea_orm(column_type = "Json")]
    pub attributes: CustomerAttributes,
    pub cpf: Option<String>,
    pub cnpj: Option<String>,
    pub tin: Option<String>,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub whatsapp: Option<String>,
    pub telegram: Option<String>,
    pub twitter: Option<String>,
    pub instagram: Option<String>,
    pub facebook: Option<String>,
    pub website: Option<String>,
    pub annual_revenue: Option<f64>,
    pub employee_count: Option<i32>,
    pub is_active: bool,
    #[sea_orm(column_type = "TimestampWithTimeZone")]
    pub created_at: DateTime<Utc>,
    #[sea_orm(column_type = "TimestampWithTimeZone")]
    pub updated_at: DateTime<Utc>,
    #[sea_orm(column_type = "Json")]
    pub billing_address: Option<AddressJson>,
    #[sea_orm(column_type = "Json")]
    pub shipping_address: Option<AddressJson>,
    pub tenant_id: Option<Uuid>,
    #[sea_orm(column_type = "JsonBinary", nullable)]
    pub properties: Option<Value>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, FromJsonQueryResult)]
pub struct CustomerAttributes {
    pub shipper: bool,
    pub carrier: bool,
    pub loan_seeker: bool,
    pub loan_broker: bool,
    pub software_vendor: bool,
    pub tenant: bool,
    pub software_development_client: bool,
    pub salesforce_client: bool,
    pub web3_client: bool,
    pub bitcoiner: bool,
    pub zk: bool,
    pub lender: bool,
    pub advertiser: bool,
    pub gp: bool,
    pub construction_contractor: bool,
    pub construction_client: bool,
    pub landlord: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, EnumIter, DeriveActiveEnum, Serialize, Deserialize)]
#[sea_orm(rs_type = "String", db_type = "String(StringLen::N(50))")]
pub enum CustomerType {
    #[sea_orm(string_value = "Household")]
    Household,
    #[sea_orm(string_value = "BusinessEntity")]
    BusinessEntity,
    #[sea_orm(string_value = "Person")]
    Person,
}

impl std::fmt::Display for CustomerType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CustomerType::Household => write!(f, "Household"),
            CustomerType::BusinessEntity => write!(f, "BusinessEntity"),
            CustomerType::Person => write!(f, "Person"),
        }
    }
}

#[derive(Copy, Clone, Debug, EnumIter)]
pub enum Relation {
    Deal,
    Case,
    FileAssociation,
    Note,
    Activity,
    Contact,
}


impl RelationTrait for Relation {
    fn def(&self) -> RelationDef {
        match self {
            Self::Deal => Entity::has_many(super::deal::Entity).into(),
            Self::Contact => Entity::has_many(super::contact::Entity).into(),
            Self::FileAssociation => Entity::has_many(super::file_association::Entity).into(),
            Self::Note => Entity::has_many(super::note::Entity).into(),
            Self::Activity => Entity::has_many(super::activity::Entity).into(),
            Self::Case => Entity::has_many(super::case::Entity).into(),
        }
    }
}


impl Related<super::deal::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Deal.def()
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

impl ActiveModelBehavior for ActiveModel {}

impl FileAssociable for Entity {
    fn entity_type() -> &'static str {
        "Customer"
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
// Setting an address
contact.billing_address = Some(AddressJson(new_address));

// Getting an address
if let Some(AddressJson(address)) = &contact.billing_address {
    println!("Full address: {}", address.get_full_address().unwrap_or_default());
}
*/

