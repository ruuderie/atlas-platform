use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;
use chrono::{DateTime, Utc};
use crate::models::listing::ListingStatus;
use crate::services::search_sync;
use serde_json::json;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "listing")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: Uuid,
    pub profile_id: Uuid,
    pub tenant_id: Uuid,
    pub category_id: Option<Uuid>,
    pub title: String,
    pub description: String,
    pub listing_type: String,
    pub price: Option<f64>,
    pub price_type: Option<String>,
    pub country: Option<String>,
    pub state: Option<String>,
    pub city: Option<String>,
    pub neighborhood: Option<String>,
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,
    pub additional_info: Option<Value>,
    #[sea_orm(column_type = "JsonBinary", nullable)]
    pub properties: Option<Value>,
    #[sea_orm(column_type = "String(StringLen::N(32))")]
    pub status: ListingStatus,
    pub is_featured: bool,
    pub is_based_on_template: bool,
    pub based_on_template_id: Option<Uuid>,
    pub is_ad_placement: bool,
    pub is_active: bool,
    #[sea_orm(column_type = "TimestampWithTimeZone")]
    pub created_at: DateTime<Utc>,
    #[sea_orm(column_type = "TimestampWithTimeZone")]
    pub updated_at: DateTime<Utc>,
    #[sea_orm(column_type = "Text", nullable)]
    pub slug: Option<String>,
}

#[derive(Copy, Clone, Debug, EnumIter)]
pub enum Relation {
    Profile,
    Tenant,
    Category,
    BasedOnTemplate,
    AdPurchase,
}

impl RelationTrait for Relation {
    fn def(&self) -> RelationDef {
        match self {
            Self::Profile => Entity::belongs_to(super::profile::Entity)
                .from(Column::ProfileId)
                .to(super::profile::Column::Id)
                .into(),
            Self::Tenant => Entity::belongs_to(super::tenant::Entity)
                .from(Column::TenantId)
                .to(super::tenant::Column::Id)
                .into(),
            Self::Category => Entity::belongs_to(super::category::Entity)
                .from(Column::CategoryId)
                .to(super::category::Column::Id)
                .into(),
            Self::BasedOnTemplate => Entity::belongs_to(super::template::Entity)
                .from(Column::BasedOnTemplateId)
                .to(super::template::Column::Id)
                .into(),
            Self::AdPurchase => Entity::has_many(super::ad_purchase::Entity).into(),
        }
    }
}

impl Related<super::profile::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Profile.def()
    }
}

impl Related<super::tenant::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Tenant.def()
    }
}
impl Related<super::category::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Category.def()
    }
}

impl Related<super::template::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::BasedOnTemplate.def()
    }
}


impl Related<super::ad_purchase::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::AdPurchase.def()
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
        let text_payload = format!("{} {}", model.title, model.description);
        let metadata = json!({
            "title": model.title.clone(),
            "subtitle": "Listing",
        });

        search_sync::upsert_search_index(
            db,
            "Listing",
            model.id,
            Some(model.tenant_id),
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
            search_sync::remove_from_search_index(db, "Listing", id).await?;
        } else if let sea_orm::ActiveValue::Unchanged(id) = self.id {
            search_sync::remove_from_search_index(db, "Listing", id).await?;
        }
        Ok(self)
    }
}