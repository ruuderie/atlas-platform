use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "platform_invite")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub email: String,
    pub role: String,
    pub tenant_name: String,
    pub invited_by: String,
    /// Pre-filled display name for the new user (first + last or full name)
    pub display_name: Option<String>,
    /// Role within the target app instance: "landlord", "pmc", "tenant", "contributor", "editor", etc.
    /// Interpreted by the receiving app — not specific to any single app type.
    pub app_role: Option<String>,
    /// The specific app instance this invite is scoped to
    pub app_instance_id: Option<Uuid>,
    /// When set: link the accepted user to this existing atlas_accounts row instead of
    /// creating a new account. Enables adding a user to an existing workspace.
    pub account_id: Option<Uuid>,
    /// When set: link the accepted user to these existing atlas_assets rows.
    /// Enables cohost/vendor/delegate invites scoped to specific properties.
    /// NULL = no asset restriction (full account access for the granted role).
    /// SeaORM does not natively support UUID arrays — stored as JSONB for now,
    /// deserialized as Vec<Uuid> in application logic.
    #[sea_orm(column_type = "JsonBinary", nullable)]
    pub asset_ids: Option<serde_json::Value>,
    /// When set (tenant invites only): automatically links the accepted user to
    /// this specific lease via `atlas_leases.tenant_user_id = user_id`.
    pub lease_id: Option<Uuid>,
    /// URL override for the magic link landing page (per-instance domain)
    pub target_app_url: Option<String>,
    /// Optional personal message from the operator, included in the invite email
    pub personal_message: Option<String>,
    #[sea_orm(column_type = "TimestampWithTimeZone")]
    pub created_at: chrono::DateTime<chrono::Utc>,
    #[sea_orm(column_type = "TimestampWithTimeZone")]
    pub expires_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
