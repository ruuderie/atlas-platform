#![allow(dead_code, unused_imports)]
use chrono::{DateTime, Utc};
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// GENERIC-14: AtlasDocument
/// Generic, polymorphic document registry with versioning and e-signature support.
///
/// Works in conjunction with the vault (attachments + share tokens).
/// `app_namespace` + `document_category` allow each vertical to define its own taxonomy
/// without polluting the database with enums.
#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "atlas_documents")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub attachment_id: Uuid,
    pub share_token_id: Option<Uuid>,
    pub document_category: String,
    pub app_namespace: String,
    pub related_entity_type: Option<String>,
    pub related_entity_id: Option<Uuid>,
    pub is_counterparty_visible: bool,
    pub requires_signature: bool,
    pub is_signed: bool,
    pub signed_at: Option<DateTime<Utc>>,
    pub signed_by_user_id: Option<Uuid>,
    pub signature_blob: Option<String>,
    pub version_number: i32,
    pub supersedes_document_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
