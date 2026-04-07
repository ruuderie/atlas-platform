use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, Serialize, Deserialize)]
#[sea_orm(table_name = "webhook_deliveries")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub endpoint_id: Uuid,
    pub tenant_id: Uuid,
    pub event_type: String,
    #[sea_orm(column_type = "JsonBinary")]
    pub payload: Value,
    pub status: String,
    pub next_retry_at: Option<DateTimeWithTimeZone>,
    pub attempts: i32,
    pub response_status: Option<i32>,
    #[sea_orm(column_type = "Text", nullable)]
    pub response_body: Option<String>,
    pub created_at: Option<DateTimeWithTimeZone>,
    pub updated_at: Option<DateTimeWithTimeZone>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::webhook_endpoint::Entity",
        from = "Column::EndpointId",
        to = "super::webhook_endpoint::Column::Id",
        on_update = "NoAction",
        on_delete = "Cascade"
    )]
    WebhookEndpoint,
    #[sea_orm(
        belongs_to = "super::tenant::Entity",
        from = "Column::TenantId",
        to = "super::tenant::Column::Id",
        on_update = "NoAction",
        on_delete = "Cascade"
    )]
    Tenant,
}

impl Related<super::webhook_endpoint::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::WebhookEndpoint.def()
    }
}

impl Related<super::tenant::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Tenant.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
