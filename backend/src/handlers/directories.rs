use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
    response::IntoResponse,
    routing::{get, post, put, delete},
    Router,
};
use sea_orm::{DatabaseConnection, EntityTrait, QueryFilter, ColumnTrait, Set, ActiveModelTrait, DbErr};
use crate::entities::directory::{self, Entity as Directory};
use crate::models::directory::{DirectoryModel, CreateDirectory, UpdateDirectory};
use crate::entities::directory_type::{self, Entity as DirectoryType};
use chrono::Utc;
use uuid::Uuid;
use crate::config::site_config::{SiteConfig, ModuleFlags};
use serde_json::Value;
use std::collections::HashMap;

pub fn public_routes(db: DatabaseConnection) -> Router<DatabaseConnection> {
    Router::new()
        .route("/directories", get(get_directories))
        .route("/directories/:id", get(get_directory_by_id))
        .route("/directories/type/:type_id", get(get_directories_by_type))
        .route("/directories/:id/listings", get(get_directory_listings))
        .with_state(db)
}

pub fn authenticated_routes(db: DatabaseConnection) -> Router<DatabaseConnection> {
    Router::new()
        .route("/directories", post(create_directory))
        .route("/directories/:id", put(update_directory))
        .route("/directories/:id", delete(delete_directory))
        .with_state(db)
}

pub async fn get_directories(
    State(db): State<DatabaseConnection>,
) -> Result<(StatusCode, Json<Vec<DirectoryModel>>), StatusCode> {
    let directories = Directory::find()
        .all(&db)
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

// The rest of your handlers (get_directory_by_id, get_directories_by_type, etc.) are already correct
pub async fn get_directory_by_id(
    Path(directory_id): Path<Uuid>,
    State(db): State<DatabaseConnection>,
) -> Result<(StatusCode, Json<DirectoryModel>), StatusCode> {
    let directory = directory::Entity::find_by_id(directory_id)
        .one(&db)
        .await
        .map_err(|err| {
            tracing::error!("Failed to fetch directory: {:?}", err);
            StatusCode::INTERNAL_SERVER_ERROR
        })?
        .ok_or(StatusCode::NOT_FOUND)?;

    Ok((StatusCode::OK, Json(DirectoryModel::from(directory))))
}

pub async fn get_directories_by_type(
    Path(directory_type_id): Path<Uuid>,
    State(db): State<DatabaseConnection>,
) -> Result<(StatusCode, Json<Vec<DirectoryModel>>), StatusCode> {
    let directories = directory::Entity::find()
        .filter(directory::Column::DirectoryTypeId.eq(directory_type_id))
        .all(&db)
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
    let new_directory = directory::ActiveModel {
        id: Set(Uuid::new_v4()),
        name: Set(input.name),
        description: Set(input.description),
        directory_type_id: Set(input.directory_type_id),
        domain: Set(input.domain),
        created_at: Set(Utc::now()),
        updated_at: Set(Utc::now()),
    };

    let directory = new_directory
        .insert(&db)
        .await
        .map_err(|err| {
            tracing::error!("Error creating directory: {:?}", err);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;
    println!("TEST LOG: from create_directory and directory: {:?}", directory);

    Ok((StatusCode::CREATED, Json(DirectoryModel::from(directory))))
}
pub async fn update_directory(
    Path(directory_id): Path<Uuid>,
    State(db): State<DatabaseConnection>,
    Json(input): Json<UpdateDirectory>,
) -> Result<(StatusCode, Json<DirectoryModel>), StatusCode> {
    let mut directory = directory::Entity::find_by_id(directory_id)
        .one(&db)
        .await
        .map_err(|err| {
            tracing::error!("Error fetching directory: {:?}", err);
            StatusCode::INTERNAL_SERVER_ERROR
        })?
        .ok_or(StatusCode::NOT_FOUND)?;

    let mut active_directory: directory::ActiveModel = directory.clone().into();

    if let Some(name) = input.name {
        active_directory.name = Set(name);
    }
    if let Some(directory_type_id) = input.directory_type_id {
        active_directory.directory_type_id = Set(directory_type_id);
    }
    if let Some(domain) = input.domain {
        active_directory.domain = Set(domain);
    }
    if let Some(description) = input.description {
        active_directory.description = Set(description);
    }
    active_directory.updated_at = Set(Utc::now());

    directory = active_directory
        .update(&db)
        .await
        .map_err(|err| {
            tracing::error!("Error updating directory: {:?}", err);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    Ok((StatusCode::OK, Json(DirectoryModel::from(directory))))
}
pub async fn delete_directory(
    Path(directory_id): Path<Uuid>,
    State(db): State<DatabaseConnection>,
) -> Result<StatusCode, StatusCode> {
    directory::Entity::delete_by_id(directory_id)
        .exec(&db)
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
    let directory = directory::Entity::find_by_id(directory_id)
        .one(&db)
        .await
        .map_err(|err| {
            tracing::error!("Failed to fetch directory: {:?}", err);
            StatusCode::INTERNAL_SERVER_ERROR
        })?
        .ok_or(StatusCode::NOT_FOUND)?;

    let listings = crate::entities::listing::Entity::find()
        .filter(crate::entities::listing::Column::DirectoryId.eq(directory_id))
        .all(&db)
        .await
        .map_err(|err| {
            tracing::error!("Failed to fetch listings: {:?}", err);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    Ok((StatusCode::OK, Json(listings)))
}

// Get site configuration
pub async fn get_site_config(
    Path(directory_id): Path<Uuid>,
    State(db): State<DatabaseConnection>,
) -> Result<impl IntoResponse, StatusCode> {
    let directory = Directory::find_by_id(directory_id)
        .one(&db)
        .await
        .map_err(|err| {
            tracing::error!("Failed to fetch directory: {:?}", err);
            StatusCode::INTERNAL_SERVER_ERROR
        })?
        .ok_or(StatusCode::NOT_FOUND)?;
    
    let site_config = SiteConfig {
        directory_id: directory.id,
        name: directory.name,
        domain: directory.domain,
        subdomain: directory.subdomain,
        custom_domain: directory.custom_domain,
        enabled_modules: ModuleFlags::from_bits_truncate(directory.enabled_modules),
        theme: directory.theme,
        custom_settings: directory.custom_settings
            .map(|v| serde_json::from_value(v).unwrap_or_default())
            .unwrap_or_default(),
        site_status: directory.site_status,
    };
    
    Ok((StatusCode::OK, Json(site_config)))
}

// Get enabled modules
pub async fn get_enabled_modules(
    Path(directory_id): Path<Uuid>,
    State(db): State<DatabaseConnection>,
) -> Result<impl IntoResponse, StatusCode> {
    let directory = Directory::find_by_id(directory_id)
        .one(&db)
        .await
        .map_err(|err| {
            tracing::error!("Failed to fetch directory: {:?}", err);
            StatusCode::INTERNAL_SERVER_ERROR
        })?
        .ok_or(StatusCode::NOT_FOUND)?;
    
    let modules = ModuleFlags::from_bits_truncate(directory.enabled_modules);
    
    #[derive(Serialize)]
    struct ModulesResponse {
        enabled_modules: i32,
        modules: Vec<String>,
    }
    
    // Convert module flags to a list of enabled module names
    let module_names = get_module_names(modules);
    
    let response = ModulesResponse {
        enabled_modules: directory.enabled_modules,
        modules: module_names,
    };
    
    Ok((StatusCode::OK, Json(response)))
}

// Helper function to get module names from flags
fn get_module_names(flags: ModuleFlags) -> Vec<String> {
    let mut modules = Vec::new();
    
    if flags.contains(ModuleFlags::LISTINGS) { modules.push("listings".to_string()); }
    if flags.contains(ModuleFlags::PROFILES) { modules.push("profiles".to_string()); }
    if flags.contains(ModuleFlags::MESSAGING) { modules.push("messaging".to_string()); }
    if flags.contains(ModuleFlags::PAYMENTS) { modules.push("payments".to_string()); }
    if flags.contains(ModuleFlags::ANALYTICS) { modules.push("analytics".to_string()); }
    if flags.contains(ModuleFlags::REVIEWS) { modules.push("reviews".to_string()); }
    if flags.contains(ModuleFlags::EVENTS) { modules.push("events".to_string()); }
    if flags.contains(ModuleFlags::CUSTOM_FIELDS) { modules.push("custom_fields".to_string()); }
    
    modules
}

// Update enabled modules
pub async fn update_enabled_modules(
    Path(directory_id): Path<Uuid>,
    State(db): State<DatabaseConnection>,
    Json(payload): Json<UpdateModulesRequest>,
) -> Result<impl IntoResponse, StatusCode> {
    let directory = Directory::find_by_id(directory_id)
        .one(&db)
        .await
        .map_err(|err| {
            tracing::error!("Failed to fetch directory: {:?}", err);
            StatusCode::INTERNAL_SERVER_ERROR
        })?
        .ok_or(StatusCode::NOT_FOUND)?;
    
    let mut active_model: directory::ActiveModel = directory.clone().into();
    active_model.enabled_modules = Set(payload.enabled_modules);
    active_model.updated_at = Set(Utc::now());
    
    let updated_directory = active_model
        .update(&db)
        .await
        .map_err(|err| {
            tracing::error!("Failed to update directory modules: {:?}", err);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;
    
    let modules = ModuleFlags::from_bits_truncate(updated_directory.enabled_modules);
    let module_names = get_module_names(modules);
    
    #[derive(Serialize)]
    struct ModulesResponse {
        enabled_modules: i32,
        modules: Vec<String>,
    }
    
    let response = ModulesResponse {
        enabled_modules: updated_directory.enabled_modules,
        modules: module_names,
    };
    
    Ok((StatusCode::OK, Json(response)))
}

#[derive(Deserialize)]
pub struct UpdateModulesRequest {
    enabled_modules: i32,
}

// Get site theme
pub async fn get_site_theme(
    Path(directory_id): Path<Uuid>,
    State(db): State<DatabaseConnection>,
) -> Result<impl IntoResponse, StatusCode> {
    let directory = Directory::find_by_id(directory_id)
        .one(&db)
        .await
        .map_err(|err| {
            tracing::error!("Failed to fetch directory: {:?}", err);
            StatusCode::INTERNAL_SERVER_ERROR
        })?
        .ok_or(StatusCode::NOT_FOUND)?;
    
    #[derive(Serialize)]
    struct ThemeResponse {
        theme: Option<String>,
    }
    
    let response = ThemeResponse {
        theme: directory.theme,
    };
    
    Ok((StatusCode::OK, Json(response)))
}

// Update site theme
pub async fn update_site_theme(
    Path(directory_id): Path<Uuid>,
    State(db): State<DatabaseConnection>,
    Json(payload): Json<UpdateThemeRequest>,
) -> Result<impl IntoResponse, StatusCode> {
    let directory = Directory::find_by_id(directory_id)
        .one(&db)
        .await
        .map_err(|err| {
            tracing::error!("Failed to fetch directory: {:?}", err);
            StatusCode::INTERNAL_SERVER_ERROR
        })?
        .ok_or(StatusCode::NOT_FOUND)?;
    
    let mut active_model: directory::ActiveModel = directory.clone().into();
    active_model.theme = Set(payload.theme);
    active_model.updated_at = Set(Utc::now());
    
    let updated_directory = active_model
        .update(&db)
        .await
        .map_err(|err| {
            tracing::error!("Failed to update directory theme: {:?}", err);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;
    
    #[derive(Serialize)]
    struct ThemeResponse {
        theme: Option<String>,
    }
    
    let response = ThemeResponse {
        theme: updated_directory.theme,
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
    let directory = Directory::find_by_id(directory_id)
        .one(&db)
        .await
        .map_err(|err| {
            tracing::error!("Failed to fetch directory: {:?}", err);
            StatusCode::INTERNAL_SERVER_ERROR
        })?
        .ok_or(StatusCode::NOT_FOUND)?;
    
    let custom_settings = directory.custom_settings.unwrap_or_else(|| serde_json::Value::Object(serde_json::Map::new()));
    
    Ok((StatusCode::OK, Json(custom_settings)))
}

// Update custom settings
pub async fn update_custom_settings(
    Path(directory_id): Path<Uuid>,
    State(db): State<DatabaseConnection>,
    Json(payload): Json<Value>,
) -> Result<impl IntoResponse, StatusCode> {
    let directory = Directory::find_by_id(directory_id)
        .one(&db)
        .await
        .map_err(|err| {
            tracing::error!("Failed to fetch directory: {:?}", err);
            StatusCode::INTERNAL_SERVER_ERROR
        })?
        .ok_or(StatusCode::NOT_FOUND)?;
    
    let mut active_model: directory::ActiveModel = directory.clone().into();
    active_model.custom_settings = Set(Some(payload));
    active_model.updated_at = Set(Utc::now());
    
    let updated_directory = active_model
        .update(&db)
        .await
        .map_err(|err| {
            tracing::error!("Failed to update directory custom settings: {:?}", err);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;
    
    let custom_settings = updated_directory.custom_settings.unwrap_or_else(|| serde_json::Value::Object(serde_json::Map::new()));
    
    Ok((StatusCode::OK, Json(custom_settings)))
}

// Update site configuration
pub async fn update_site_config(
    Path(directory_id): Path<Uuid>,
    State(db): State<DatabaseConnection>,
    Json(payload): Json<UpdateSiteConfigRequest>,
) -> Result<impl IntoResponse, StatusCode> {
    let directory = Directory::find_by_id(directory_id)
        .one(&db)
        .await
        .map_err(|err| {
            tracing::error!("Failed to fetch directory: {:?}", err);
            StatusCode::INTERNAL_SERVER_ERROR
        })?
        .ok_or(StatusCode::NOT_FOUND)?;
    
    let mut active_model: directory::ActiveModel = directory.clone().into();
    
    if let Some(name) = payload.name {
        active_model.name = Set(name);
    }
    
    if let Some(domain) = payload.domain {
        active_model.domain = Set(domain);
    }
    
    if let Some(subdomain) = payload.subdomain {
        active_model.subdomain = Set(subdomain);
    }
    
    if let Some(custom_domain) = payload.custom_domain {
        active_model.custom_domain = Set(custom_domain);
    }
    
    if let Some(enabled_modules) = payload.enabled_modules {
        active_model.enabled_modules = Set(enabled_modules);
    }
    
    if let Some(theme) = payload.theme {
        active_model.theme = Set(Some(theme));
    }
    
    if let Some(custom_settings) = payload.custom_settings {
        active_model.custom_settings = Set(Some(serde_json::to_value(custom_settings).unwrap_or_default()));
    }
    
    if let Some(site_status) = payload.site_status {
        active_model.site_status = Set(site_status);
    }
    
    active_model.updated_at = Set(Utc::now());
    
    let updated_directory = active_model
        .update(&db)
        .await
        .map_err(|err| {
            tracing::error!("Failed to update directory configuration: {:?}", err);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;
    
    let site_config = SiteConfig {
        directory_id: updated_directory.id,
        name: updated_directory.name,
        domain: updated_directory.domain,
        subdomain: updated_directory.subdomain,
        custom_domain: updated_directory.custom_domain,
        enabled_modules: ModuleFlags::from_bits_truncate(updated_directory.enabled_modules),
        theme: updated_directory.theme,
        custom_settings: updated_directory.custom_settings
            .map(|v| serde_json::from_value(v).unwrap_or_default())
            .unwrap_or_default(),
        site_status: updated_directory.site_status,
    };
    
    Ok((StatusCode::OK, Json(site_config)))
}

#[derive(Deserialize)]
pub struct UpdateSiteConfigRequest {
    name: Option<String>,
    domain: Option<String>,
    subdomain: Option<Option<String>>,
    custom_domain: Option<Option<String>>,
    enabled_modules: Option<i32>,
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
    pub enabled_modules: i32,
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