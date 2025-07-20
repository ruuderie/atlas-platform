use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
    response::IntoResponse,
    routing::{get, post, put, delete},
    Router,
};
use sea_orm::{DatabaseConnection, EntityTrait, QueryFilter, ColumnTrait, ActiveModelTrait, Set, NotSet};
use crate::entities::directory::{self, Entity as Directory};
use crate::models::directory::{DirectoryModel, CreateDirectory, UpdateDirectory};
use crate::entities::directory_type::{self, Entity as DirectoryType};
use chrono::{Utc, DateTime};
use uuid::Uuid;
use crate::config::site_config::{SiteConfig, ModuleFlags};
use serde_json::Value;
use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use crate::services::directory::DirectoryService;

pub fn public_routes(db: DatabaseConnection) -> Router<DatabaseConnection> {
    Router::new()
        .route("/directories", get(get_directories))
        .route("/directories/{id}", get(get_directory_by_id))
        .route("/directories/type/{type_id}", get(get_directories_by_type))
        .route("/directories/{id}/listings", get(get_directory_listings))
        .with_state(db)
}

pub fn authenticated_routes(db: DatabaseConnection) -> Router<DatabaseConnection> {
    Router::new()
        .route("/api/directories", post(create_directory))
        .route("/api/directories/{id}", put(update_directory))
        .route("/api/directories/{id}", delete(delete_directory))
        .with_state(db)
}

pub async fn get_directories(
    State(db): State<DatabaseConnection>,
) -> Result<(StatusCode, Json<Vec<DirectoryModel>>), StatusCode> {
    let directories = DirectoryService::list_directories(&db, None, None)
        .await
        .map_err(|err| {
            tracing::error!("Database error: {:?}", err);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    let directory_models: Vec<DirectoryModel> = directories
        .into_iter()
        .map(DirectoryModel::from)
        .collect();

    Ok((StatusCode::OK, Json(directory_models)))
}

pub async fn get_directory_by_id(
    Path(directory_id): Path<Uuid>,
    State(db): State<DatabaseConnection>,
) -> Result<(StatusCode, Json<DirectoryModel>), StatusCode> {
    let directory = DirectoryService::get_directory_by_id(&db, directory_id)
        .await
        .map_err(|err| {
            tracing::error!("Failed to fetch directory: {:?}", err);
            match err.downcast_ref::<anyhow::Error>() {
                Some(e) if e.to_string().contains("not found") => StatusCode::NOT_FOUND,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            }
        })?;

    Ok((StatusCode::OK, Json(DirectoryModel::from(directory))))
}

pub async fn get_directories_by_type(
    Path(directory_type_id): Path<Uuid>,
    State(db): State<DatabaseConnection>,
) -> Result<(StatusCode, Json<Vec<DirectoryModel>>), StatusCode> {
    let directories = DirectoryService::get_directories_by_type(&db, directory_type_id)
        .await
        .map_err(|err| {
            tracing::error!("Database error: {:?}", err);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    let directory_models: Vec<DirectoryModel> = directories
        .into_iter()
        .map(DirectoryModel::from)
        .collect();

    Ok((StatusCode::OK, Json(directory_models)))
}

pub async fn create_directory(
    State(db): State<DatabaseConnection>,
    Json(input): Json<CreateDirectory>,
) -> Result<(StatusCode, Json<DirectoryModel>), StatusCode> {
    let directory = DirectoryService::create_directory(&db, input)
        .await
        .map_err(|err| {
            tracing::error!("Error creating directory: {:?}", err);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    Ok((StatusCode::CREATED, Json(DirectoryModel::from(directory))))
}

pub async fn update_directory(
    Path(directory_id): Path<Uuid>,
    State(db): State<DatabaseConnection>,
    Json(input): Json<UpdateDirectory>,
) -> Result<(StatusCode, Json<DirectoryModel>), StatusCode> {
    let directory = DirectoryService::update_directory(&db, directory_id, input)
        .await
        .map_err(|err| {
            tracing::error!("Error updating directory: {:?}", err);
            match err.downcast_ref::<anyhow::Error>() {
                Some(e) if e.to_string().contains("not found") => StatusCode::NOT_FOUND,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            }
        })?;

    Ok((StatusCode::OK, Json(DirectoryModel::from(directory))))
}

pub async fn delete_directory(
    Path(directory_id): Path<Uuid>,
    State(db): State<DatabaseConnection>,
) -> Result<StatusCode, StatusCode> {
    DirectoryService::delete_directory(&db, directory_id)
        .await
        .map_err(|err| {
            tracing::error!("Error deleting directory: {:?}", err);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    Ok(StatusCode::NO_CONTENT)
}

pub async fn get_directory_listings(
    Path(directory_id): Path<Uuid>,
    State(db): State<DatabaseConnection>,
) -> Result<(StatusCode, Json<Vec<crate::entities::listing::Model>>), StatusCode> {
    let listings = DirectoryService::get_directory_listings(&db, directory_id)
        .await
        .map_err(|err| {
            tracing::error!("Failed to fetch listings: {:?}", err);
            match err.downcast_ref::<anyhow::Error>() {
                Some(e) if e.to_string().contains("not found") => StatusCode::NOT_FOUND,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            }
        })?;

    Ok((StatusCode::OK, Json(listings)))
}

// Get site configuration
pub async fn get_site_config(
    Path(directory_id): Path<Uuid>,
    State(db): State<DatabaseConnection>,
) -> Result<impl IntoResponse, StatusCode> {
    let site_config = DirectoryService::get_site_config(&db, directory_id)
        .await
        .map_err(|err| {
            tracing::error!("Failed to fetch site config: {:?}", err);
            match err.downcast_ref::<anyhow::Error>() {
                Some(e) if e.to_string().contains("not found") => StatusCode::NOT_FOUND,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            }
        })?;
    
    Ok((StatusCode::OK, Json(site_config)))
}

// Get enabled modules
pub async fn get_enabled_modules(
    Path(directory_id): Path<Uuid>,
    State(db): State<DatabaseConnection>,
) -> Result<impl IntoResponse, StatusCode> {
    let (enabled_modules, modules) = DirectoryService::get_enabled_modules(&db, directory_id)
        .await
        .map_err(|err| {
            tracing::error!("Failed to fetch enabled modules: {:?}", err);
            match err.downcast_ref::<anyhow::Error>() {
                Some(e) if e.to_string().contains("not found") => StatusCode::NOT_FOUND,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            }
        })?;
    
    #[derive(Serialize)]
    struct ModulesResponse {
        enabled_modules: u32,
        modules: Vec<String>,
    }
    
    let response = ModulesResponse {
        enabled_modules,
        modules,
    };
    
    Ok((StatusCode::OK, Json(response)))
}

// Update enabled modules
pub async fn update_enabled_modules(
    Path(directory_id): Path<Uuid>,
    State(db): State<DatabaseConnection>,
    Json(payload): Json<UpdateModulesRequest>,
) -> Result<impl IntoResponse, StatusCode> {
    let (enabled_modules, modules) = DirectoryService::update_enabled_modules(&db, directory_id, payload.enabled_modules)
        .await
        .map_err(|err| {
            tracing::error!("Failed to update directory modules: {:?}", err);
            match err.downcast_ref::<anyhow::Error>() {
                Some(e) if e.to_string().contains("not found") => StatusCode::NOT_FOUND,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            }
        })?;
    
    #[derive(Serialize)]
    struct ModulesResponse {
        enabled_modules: u32,
        modules: Vec<String>,
    }
    
    let response = ModulesResponse {
        enabled_modules,
        modules,
    };
    
    Ok((StatusCode::OK, Json(response)))
}

#[derive(Deserialize)]
pub struct UpdateModulesRequest {
    enabled_modules: u32,
}

// Get site theme
pub async fn get_site_theme(
    Path(directory_id): Path<Uuid>,
    State(db): State<DatabaseConnection>,
) -> Result<impl IntoResponse, StatusCode> {
    let theme = DirectoryService::get_site_theme(&db, directory_id)
        .await
        .map_err(|err| {
            tracing::error!("Failed to fetch directory theme: {:?}", err);
            match err.downcast_ref::<anyhow::Error>() {
                Some(e) if e.to_string().contains("not found") => StatusCode::NOT_FOUND,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            }
        })?;
    
    #[derive(Serialize)]
    struct ThemeResponse {
        theme: Option<String>,
    }
    
    let response = ThemeResponse {
        theme,
    };
    
    Ok((StatusCode::OK, Json(response)))
}

// Update site theme
pub async fn update_site_theme(
    Path(directory_id): Path<Uuid>,
    State(db): State<DatabaseConnection>,
    Json(payload): Json<UpdateThemeRequest>,
) -> Result<impl IntoResponse, StatusCode> {
    let theme = DirectoryService::update_site_theme(&db, directory_id, payload.theme)
        .await
        .map_err(|err| {
            tracing::error!("Failed to update directory theme: {:?}", err);
            match err.downcast_ref::<anyhow::Error>() {
                Some(e) if e.to_string().contains("not found") => StatusCode::NOT_FOUND,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            }
        })?;
    
    #[derive(Serialize)]
    struct ThemeResponse {
        theme: Option<String>,
    }
    
    let response = ThemeResponse {
        theme,
    };
    
    Ok((StatusCode::OK, Json(response)))
}

#[derive(Deserialize)]
pub struct UpdateThemeRequest {
    theme: Option<String>,
}

// Get custom settings
pub async fn get_custom_settings(
    Path(directory_id): Path<Uuid>,
    State(db): State<DatabaseConnection>,
) -> Result<impl IntoResponse, StatusCode> {
    let custom_settings = DirectoryService::get_custom_settings(&db, directory_id)
        .await
        .map_err(|err| {
            tracing::error!("Failed to fetch directory custom settings: {:?}", err);
            match err.downcast_ref::<anyhow::Error>() {
                Some(e) if e.to_string().contains("not found") => StatusCode::NOT_FOUND,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            }
        })?;
    
    Ok((StatusCode::OK, Json(custom_settings)))
}

// Update custom settings
pub async fn update_custom_settings(
    Path(directory_id): Path<Uuid>,
    State(db): State<DatabaseConnection>,
    Json(payload): Json<Value>,
) -> Result<impl IntoResponse, StatusCode> {
    let custom_settings = DirectoryService::update_custom_settings(&db, directory_id, payload)
        .await
        .map_err(|err| {
            tracing::error!("Failed to update directory custom settings: {:?}", err);
            match err.downcast_ref::<anyhow::Error>() {
                Some(e) if e.to_string().contains("not found") => StatusCode::NOT_FOUND,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            }
        })?;
    
    Ok((StatusCode::OK, Json(custom_settings)))
}

// Update site configuration
pub async fn update_site_config(
    Path(directory_id): Path<Uuid>,
    State(db): State<DatabaseConnection>,
    Json(payload): Json<UpdateSiteConfigRequest>,
) -> Result<impl IntoResponse, StatusCode> {
    // First get the current config
    let mut config = DirectoryService::get_site_config(&db, directory_id)
        .await
        .map_err(|err| {
            tracing::error!("Failed to fetch site config: {:?}", err);
            match err.downcast_ref::<anyhow::Error>() {
                Some(e) if e.to_string().contains("not found") => StatusCode::NOT_FOUND,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            }
        })?;
    
    // Update with new values
    if let Some(name) = payload.name {
        config.name = name;
    }
    
    if let Some(domain) = payload.domain {
        config.domain = domain;
    }
    
    if let Some(subdomain) = payload.subdomain {
        config.subdomain = subdomain;
    }
    
    if let Some(custom_domain) = payload.custom_domain {
        config.custom_domain = custom_domain;
    }
    
    if let Some(enabled_modules) = payload.enabled_modules {
        config.enabled_modules = ModuleFlags::from_bits_truncate(enabled_modules);
    }
    
    if let Some(theme) = payload.theme {
        config.theme = Some(theme);
    }
    
    if let Some(custom_settings) = payload.custom_settings {
        config.custom_settings = custom_settings;
    }
    
    if let Some(site_status) = payload.site_status {
        config.site_status = Some(site_status);
    }
    
    // Update the config
    let updated_config = DirectoryService::update_directory_config(&db, directory_id, config)
        .await
        .map_err(|err| {
            tracing::error!("Failed to update site config: {:?}", err);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;
    
    Ok((StatusCode::OK, Json(updated_config)))
}

#[derive(Deserialize)]
pub struct UpdateSiteConfigRequest {
    name: Option<String>,
    domain: Option<String>,
    subdomain: Option<Option<String>>,
    custom_domain: Option<Option<String>>,
    enabled_modules: Option<u32>,
    theme: Option<String>,
    custom_settings: Option<HashMap<String, Value>>,
    site_status: Option<String>,
}

// Update the DirectoryModel to include the new fields
#[derive(Debug, Serialize, Deserialize)]
pub struct DirectoryConfigModel {
    pub id: Uuid,
    pub name: String,
    pub directory_type_id: Uuid,
    pub domain: String,
    pub description: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub enabled_modules: u32,
    pub theme: Option<String>,
    pub custom_settings: Option<Value>,
    pub site_status: String,
    pub subdomain: Option<String>,
    pub custom_domain: Option<String>,
}

impl From<directory::Model> for DirectoryConfigModel {
    fn from(model: directory::Model) -> Self {
        Self {
            id: model.id,
            name: model.name,
            directory_type_id: model.directory_type_id,
            domain: model.domain,
            description: model.description,
            created_at: model.created_at,
            updated_at: model.updated_at,
            enabled_modules: model.enabled_modules,
            theme: model.theme,
            custom_settings: model.custom_settings,
            site_status: model.site_status,
            subdomain: model.subdomain,
            custom_domain: model.custom_domain,
        }
    }
}