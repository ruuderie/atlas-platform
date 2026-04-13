use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};
use crate::services::search_sync;
use serde_json::json;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "tenant")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub name: String,
    pub description: String,
    pub site_status: String,
    #[sea_orm(unique, nullable)]
    pub slug: Option<String>,
    #[sea_orm(nullable)]
    pub logo: Option<String>,
    #[sea_orm(nullable)]
    pub favicon: Option<String>,
    #[sea_orm(column_type = "Text", nullable)]
    pub header_scripts: Option<String>,
    #[sea_orm(column_type = "Text", nullable)]
    pub footer_scripts: Option<String>,
    #[sea_orm(nullable)]
    pub google_analytics_id: Option<String>,
    #[sea_orm(nullable)]
    pub google_site_verification: Option<String>,
    #[sea_orm(column_type = "Text", nullable)]
    pub meta_description: Option<String>,
    #[sea_orm(column_type = "Text", nullable)]
    pub meta_keywords: Option<String>,
    #[sea_orm(column_type = "Text", nullable)]
    pub meta_title: Option<String>,
    #[sea_orm(nullable)]
    pub page_title: Option<String>,
    #[sea_orm(nullable)]
    pub page_description: Option<String>,
    #[sea_orm(nullable)]
    pub page_keywords: Option<String>,
    #[sea_orm(column_type = "Text", nullable)]
    pub canonical_url: Option<String>,
    #[sea_orm(column_type = "TimestampWithTimeZone")]
    pub created_at: DateTime<Utc>,
    #[sea_orm(column_type = "TimestampWithTimeZone")]
    pub updated_at: DateTime<Utc>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::app_instance::Entity")]
    AppInstance,
    #[sea_orm(has_many = "super::profile::Entity")]
    Profile,
    #[sea_orm(has_many = "super::template::Entity")]
    Template,
    #[sea_orm(has_many = "super::listing::Entity")]
    Listing,
    #[sea_orm(has_many = "super::account::Entity")]
    Account,
    #[sea_orm(has_many = "super::tenant_setting::Entity")]
    TenantSetting,
}

impl Related<super::app_instance::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::AppInstance.def()
    }
}
impl Related<super::profile::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Profile.def()
    }
}
impl Related<super::template::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Template.def()
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
impl Related<super::tenant_setting::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::TenantSetting.def()
    }
}

#[async_trait::async_trait]
impl ActiveModelBehavior for ActiveModel {
    async fn after_save<C>(
        model: Model,
        db: &C,
        _insert: bool,
    ) -> Result<Model, DbErr>
    where
        C: ConnectionTrait,
    {
        let text_payload = format!("{} {}", model.name, model.description);
        let metadata = json!({
            "title": model.name.clone(),
            "subtitle": "Network / Workspace",
            "avatar": model.logo,
        });

        search_sync::upsert_search_index(
            db,
            "Network",
            model.id,
            Some(model.id), // A tenant represents itself in the tenant dimension
            &text_payload,
            metadata,
        )
        .await?;

        Ok(model)
    }

    async fn after_delete<C>(
        self,
        db: &C,
    ) -> Result<Self, DbErr>
    where
        C: ConnectionTrait,
    {
        if let sea_orm::ActiveValue::Set(id) = self.id {
            search_sync::remove_from_search_index(db, "Network", id).await?;
        } else if let sea_orm::ActiveValue::Unchanged(id) = self.id {
            search_sync::remove_from_search_index(db, "Network", id).await?;
        }
        Ok(self)
    }
}
