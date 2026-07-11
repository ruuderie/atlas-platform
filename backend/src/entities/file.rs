#![allow(dead_code, unused_imports)]
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use strum_macros::Display;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "files")]
pub struct Model {
    #[sea_orm(primary_key)]
    #[serde(skip_deserializing)]
    pub id: String,
    pub name: String,
    pub size: i64,
    pub mime_type: String,
    pub hash_sha256: String,
    pub storage_type: StorageType,
    pub storage_path: String,
    pub views: i32,
    pub downloads: i32,
    pub bandwidth_used: i64,
    pub bandwidth_used_paid: i64,
    pub date_upload: DateTimeWithTimeZone,
    pub date_last_view: Option<DateTimeWithTimeZone>,
    pub is_anonymous: bool,
    pub user_id: Option<String>,
}

#[derive(Debug, Display, Clone, PartialEq, EnumIter, DeriveActiveEnum, Serialize, Deserialize)]
#[sea_orm(rs_type = "String", db_type = "String(StringLen::N(1))")]
pub enum StorageType {
    #[sea_orm(string_value = "L")]
    Local,
    #[sea_orm(string_value = "S")]
    S3,
    #[sea_orm(string_value = "D")]
    Database,
    #[sea_orm(string_value = "C")]
    Custom,
}

impl FromStr for StorageType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "L" => Ok(StorageType::Local),
            "S" => Ok(StorageType::S3),
            "D" => Ok(StorageType::Database),
            "C" => Ok(StorageType::Custom),
            _ => Err(format!("Invalid storage type: {}", s)),
        }
    }
}

#[derive(Copy, Clone, Debug, EnumIter)]
pub enum Relation {
    FileAssociation,
}

impl RelationTrait for Relation {
    fn def(&self) -> RelationDef {
        match self {
            Self::FileAssociation => Entity::has_many(super::file_association::Entity).into(),
        }
    }
}

impl Related<super::file_association::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::FileAssociation.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
