use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
    response::IntoResponse,
    routing::{get, post},
    Router,
};
use sea_orm::{DatabaseConnection, EntityTrait, QueryFilter, ColumnTrait, ActiveModelTrait, Set};
use crate::entities::app_instance::{self, Entity as AppInstanceEntity};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;
use chrono::Utc;
use std::fs;
use std::path::PathBuf;

#[derive(Serialize, Deserialize)]
pub struct CreateAppInstancePayload {
    pub tenant_id: Uuid,
    pub app_type: String, // e.g., "anchor", "directory"
    pub database_url: Option<String>,
    pub data_seed_name: Option<String>,
    pub settings: Option<Value>,
}

pub fn authenticated_routes(db: DatabaseConnection) -> Router<DatabaseConnection> {
    Router::new()
        .route("/api/app-instances", post(create_app_instance))
        .route("/api/app-instances/seeds/:app_type", get(list_data_seeds))
        .route("/api/app-instances/:tenant_id/:app_type", get(get_app_instance))
        .with_state(db)
}

/// Fetches an AppInstance by tenant ID and app type.
pub async fn get_app_instance(
    Path((tenant_id, app_type)): Path<(Uuid, String)>,
    State(db): State<DatabaseConnection>,
) -> Result<Json<app_instance::Model>, StatusCode> {
    let instance = AppInstanceEntity::find()
        .filter(app_instance::Column::TenantId.eq(tenant_id))
        .filter(app_instance::Column::AppType.eq(app_type))
        .one(&db)
        .await
        .map_err(|err| {
            tracing::error!("Database error fetching app instance: {:?}", err);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    if let Some(inst) = instance {
        Ok(Json(inst))
    } else {
        Err(StatusCode::NOT_FOUND)
    }
}

/// Creates a new AppInstance for a given Tenant.
pub async fn create_app_instance(
    State(db): State<DatabaseConnection>,
    Json(payload): Json<CreateAppInstancePayload>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let id = Uuid::new_v4();

    let new_instance = app_instance::ActiveModel {
        id: Set(id),
        tenant_id: Set(payload.tenant_id),
        app_type: Set(payload.app_type.clone()),
        database_url: Set(payload.database_url),
        data_seed_name: Set(payload.data_seed_name),
        settings: Set(payload.settings),
        created_at: Set(Utc::now()),
        updated_at: Set(Utc::now()),
    };

    let inserted = new_instance.insert(&db).await.map_err(|e| {
        tracing::error!("Error creating App Instance: {:?}", e);
        (StatusCode::INTERNAL_SERVER_ERROR, "Failed to create app instance".into())
    })?;

    Ok((StatusCode::CREATED, Json(inserted)))
}

/// Lists available data seeds for a specific application type.
pub async fn list_data_seeds(
    Path(app_type): Path<String>,
) -> Result<Json<Vec<String>>, (StatusCode, String)> {
    // Navigate relative to the backend running directory, usually workspace root is one up.
    let root_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .join("seeds")
        .join(&app_type);

    if !root_path.exists() || !root_path.is_dir() {
        return Ok(Json(vec![]));
    }

    let mut seeds = Vec::new();
    if let Ok(entries) = fs::read_dir(root_path) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file() {
                if let Some(ext) = path.extension() {
                    if ext == "sql" || ext == "json" {
                        if let Some(file_name) = path.file_stem().and_then(|n| n.to_str()) {
                            seeds.push(file_name.to_string());
                        }
                    }
                }
            }
        }
    }

    Ok(Json(seeds))
}
