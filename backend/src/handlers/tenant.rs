use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
    response::IntoResponse,
    routing::{get, post, put, delete},
    Router,
};
use sea_orm::{DatabaseConnection, EntityTrait, QueryFilter, ColumnTrait, ActiveModelTrait, Set, NotSet};
use crate::entities::tenant::{self, Entity as Tenant};
use crate::entities::tenant_setting::{self, Entity as TenantSetting};
use crate::models::tenant::{TenantModel, CreateTenant, UpdateTenant};
use chrono::{Utc, DateTime};
use uuid::Uuid;
use crate::config::site_config::{SiteConfig, ModuleFlags};
use serde_json::Value;
use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use crate::services::tenant::TenantService;

pub fn public_routes(db: DatabaseConnection) -> Router<DatabaseConnection> {
    Router::new()
        .route("/tenants", get(get_tenants))
        .route("/tenants/lookup", get(lookup_tenant_by_domain))
        .route("/tenants/{id}", get(get_tenant_by_id))
        .with_state(db)
}

pub fn authenticated_routes(db: DatabaseConnection) -> Router<DatabaseConnection> {
    Router::new()
        .route("/api/tenants", post(create_tenant))
        .route("/api/tenants/{id}", put(update_tenant))
        .route("/api/tenants/{id}", delete(delete_tenant))
        .route("/api/tenants/{id}/settings", get(get_tenant_settings))
        .route("/api/tenants/{id}/settings", post(upsert_tenant_setting))
        .with_state(db)
}

pub async fn get_tenants(
    State(db): State<DatabaseConnection>,
) -> Result<(StatusCode, Json<Vec<TenantModel>>), StatusCode> {
    let tenants = TenantService::list_tenants(&db)
        .await
        .map_err(|err| {
            tracing::error!("Database error: {:?}", err);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    let models: Vec<TenantModel> = tenants
        .into_iter()
        .map(TenantModel::from)
        .collect();

    Ok((StatusCode::OK, Json(models)))
}

pub async fn get_tenant_by_id(
    Path(tenant_id): Path<Uuid>,
    State(db): State<DatabaseConnection>,
) -> Result<(StatusCode, Json<TenantModel>), StatusCode> {
    let tenant = TenantService::get_tenant_by_id(&db, tenant_id)
        .await
        .map_err(|err| {
            tracing::error!("Failed to fetch tenant: {:?}", err);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    if let Some(t) = tenant {
        Ok((StatusCode::OK, Json(TenantModel::from(t))))
    } else {
        Err(StatusCode::NOT_FOUND)
    }
}

pub async fn lookup_tenant_by_domain(
    axum::extract::Query(params): axum::extract::Query<std::collections::HashMap<String, String>>,
    State(db): State<DatabaseConnection>,
) -> Result<(StatusCode, Json<TenantModel>), StatusCode> {
    let domain = params.get("domain").ok_or(StatusCode::BAD_REQUEST)?;
    
    // In multi-tenant architecture, AppDomain identifies the tenant.
    // For now, simulated by getting the first tenant (Will replace when lookup logic finishes)
    // We just return a 404 for now until app_domain entity logic is fully built
    Err(StatusCode::NOT_FOUND)
}

pub async fn create_tenant(
    State(db): State<DatabaseConnection>,
    Json(payload): Json<CreateTenant>,
) -> Result<(StatusCode, Json<TenantModel>), StatusCode> {
    let tenant = TenantService::create_tenant(&db, payload)
        .await
        .map_err(|err| {
            tracing::error!("Failed to create tenant: {:?}", err);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    Ok((StatusCode::CREATED, Json(TenantModel::from(tenant))))
}

pub async fn update_tenant(
    Path(tenant_id): Path<Uuid>,
    State(db): State<DatabaseConnection>,
    Json(payload): Json<UpdateTenant>,
) -> Result<(StatusCode, Json<TenantModel>), StatusCode> {
    let tenant = TenantService::update_tenant(&db, tenant_id, payload)
        .await
        .map_err(|err| {
            tracing::error!("Failed to update tenant: {:?}", err);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    Ok((StatusCode::OK, Json(TenantModel::from(tenant))))
}

pub async fn delete_tenant(
    Path(tenant_id): Path<Uuid>,
    State(db): State<DatabaseConnection>,
) -> Result<StatusCode, StatusCode> {
    TenantService::delete_tenant(&db, tenant_id)
        .await
        .map_err(|err| {
            tracing::error!("Failed to delete tenant: {:?}", err);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    Ok(StatusCode::NO_CONTENT)
}

#[derive(Serialize, Deserialize)]
pub struct UpsertTenantSettingPayload {
    pub key: String,
    pub value: String,
    pub is_encrypted: Option<bool>,
}

pub async fn get_tenant_settings(
    Path(tenant_id): Path<Uuid>,
    State(db): State<DatabaseConnection>,
) -> Result<(StatusCode, Json<Vec<tenant_setting::Model>>), StatusCode> {
    let settings = TenantSetting::find()
        .filter(tenant_setting::Column::TenantId.eq(tenant_id))
        .all(&db)
        .await
        .map_err(|err| {
            tracing::error!("Failed to fetch settings: {:?}", err);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    Ok((StatusCode::OK, Json(settings)))
}

pub async fn upsert_tenant_setting(
    Path(tenant_id): Path<Uuid>,
    State(db): State<DatabaseConnection>,
    Json(payload): Json<UpsertTenantSettingPayload>,
) -> Result<(StatusCode, Json<tenant_setting::Model>), StatusCode> {
    let existing = TenantSetting::find()
        .filter(tenant_setting::Column::TenantId.eq(tenant_id))
        .filter(tenant_setting::Column::Key.eq(&payload.key))
        .one(&db)
        .await
        .map_err(|err| {
            tracing::error!("Database error checking setting: {:?}", err);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    if let Some(setting) = existing {
        let mut active: tenant_setting::ActiveModel = setting.into();
        active.value = Set(payload.value);
        if let Some(enc) = payload.is_encrypted {
            active.is_encrypted = Set(enc);
        }
        active.updated_at = Set(Utc::now());
        let updated = active.update(&db).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        Ok((StatusCode::OK, Json(updated)))
    } else {
        let new_setting = tenant_setting::ActiveModel {
            id: Set(Uuid::new_v4()),
            tenant_id: Set(tenant_id),
            key: Set(payload.key),
            value: Set(payload.value),
            is_encrypted: Set(payload.is_encrypted.unwrap_or(false)),
            updated_at: Set(Utc::now()),
            created_at: Set(Utc::now()),
        };
        let inserted = new_setting.insert(&db).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        Ok((StatusCode::CREATED, Json(inserted)))
    }
}