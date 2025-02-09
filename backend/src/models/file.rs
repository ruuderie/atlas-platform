use crate::entities::file;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};
use sea_orm::{DatabaseConnection, DbErr};

#[derive(Debug, Serialize, Deserialize)]
pub struct FileModel {
    pub id: Uuid,
    pub name: String,
    pub size: i64,
    pub mime_type: String,
    pub hash_sha256: String,
    pub storage_type: String,
    pub storage_path: String,
    pub views: i32,
    pub downloads: i32,
    pub bandwidth_used: i64,
    pub bandwidth_used_paid: i64,
    pub date_upload: DateTime<Utc>,
    pub date_last_view: Option<DateTime<Utc>>,
    pub is_anonymous: bool,
    pub user_id: Option<Uuid>,
}

#[derive(Debug, Deserialize)]
pub struct CreateFileInput {
    pub name: String,
    pub size: i64,
    pub mime_type: String,
    pub hash_sha256: String,
    pub storage_type: String,
    pub storage_path: String,
    pub is_anonymous: bool,
    pub user_id: Option<Uuid>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateFileInput {
    pub name: Option<String>,
    pub views: Option<i32>,
    pub downloads: Option<i32>,
    pub bandwidth_used: Option<i64>,
    pub bandwidth_used_paid: Option<i64>,
    pub date_last_view: Option<DateTime<Utc>>,
}

impl From<file::Model> for FileModel {
    fn from(model: file::Model) -> Self {
        FileModel {
            id: Uuid::parse_str(&model.id).unwrap(),
            name: model.name,
            size: model.size,
            mime_type: model.mime_type,
            hash_sha256: model.hash_sha256,
            storage_type: model.storage_type.to_string(),
            storage_path: model.storage_path,
            views: model.views,
            downloads: model.downloads,
            bandwidth_used: model.bandwidth_used,
            bandwidth_used_paid: model.bandwidth_used_paid,
            date_upload: model.date_upload.with_timezone(&Utc),
            date_last_view: model.date_last_view.map(|dt| dt.with_timezone(&Utc)),
            is_anonymous: model.is_anonymous,
            user_id: model.user_id.map(|id| Uuid::parse_str(&id).unwrap()),
        }
    }
}

pub trait FileAssociation {
    fn add_file(&self, db: &DatabaseConnection, file_id: Uuid) -> impl std::future::Future<Output = Result<(), DbErr>> + Send;
    fn remove_file(&self, db: &DatabaseConnection, file_id: Uuid) -> impl std::future::Future<Output = Result<(), DbErr>> + Send;
    fn get_associated_files(&self, db: &DatabaseConnection) -> impl std::future::Future<Output = Result<Vec<FileModel>, DbErr>> + Send;
}
