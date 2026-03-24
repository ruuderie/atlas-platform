use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use uuid::Uuid;
use sha2::{Sha256, Digest};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "session")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: Uuid,
    pub user_id: Uuid,
    pub bearer_token: String,
    pub refresh_token: String,
    #[sea_orm(column_type = "TimestampWithTimeZone")]
    pub token_expiration: DateTime<Utc>,
    #[sea_orm(column_type = "TimestampWithTimeZone")]
    pub refresh_token_expiration: DateTime<Utc>,
    #[sea_orm(column_type = "TimestampWithTimeZone")]
    pub created_at: DateTime<Utc>,
    #[sea_orm(column_type = "TimestampWithTimeZone")]
    pub last_accessed_at: DateTime<Utc>,
    #[sea_orm(column_type = "TimestampWithTimeZone")]
    pub last_modified_date: DateTime<Utc>,
    pub is_admin: bool,
    pub is_active: bool,
    pub integrity_hash: String,
}

impl Model {
    pub fn generate_integrity_hash(&self) -> String {
        let mut hasher = Sha256::new();
        hasher.update(self.id.to_string());
        hasher.update(&self.user_id.to_string());
        hasher.update(&self.bearer_token);
        hasher.update(&self.token_expiration.timestamp().to_string());
        hasher.update(&self.is_admin.to_string());
        format!("{:x}", hasher.finalize())
    }

    pub fn verify_integrity(&self) -> bool {
        self.integrity_hash == self.generate_integrity_hash()
    }
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::user::Entity",
        from = "Column::UserId",
        to = "super::user::Column::Id"
    )]
    User,
}

impl Related<super::user::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::User.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}