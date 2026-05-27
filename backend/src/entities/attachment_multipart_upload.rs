use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};

/// Part of GENERIC-02 (atlas_vault)
/// Tracks in-progress multipart uploads to R2/Cloudflare.
#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "attachment_multipart_uploads")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub attachment_id: Uuid,
    pub r2_upload_id: String,
    pub total_parts: Option<i32>,
    pub completed_parts: i32,
    pub status: String, // 'in_progress', 'finalizing', 'complete', 'aborted'
    pub created_at: DateTime<Utc>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
