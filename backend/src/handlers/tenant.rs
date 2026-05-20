use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
    routing::{get, post, put, delete},
    Router,
};
use sea_orm::{DatabaseConnection, EntityTrait, QueryFilter, ColumnTrait, ActiveModelTrait, Set};
use crate::entities::tenant_setting::{self, Entity as TenantSetting};
use crate::models::tenant::{TenantModel, CreateTenant, UpdateTenant};
use chrono::Utc;
use uuid::Uuid;
use serde::{Serialize, Deserialize};
use crate::services::tenant::TenantService;

/// State-free public route definitions.
/// Use inside `AtlasApp::public_router()`. Never call `.with_state()` here.
pub fn public_routes_raw() -> Router<DatabaseConnection> {
    Router::new()
        .route("/tenants", get(get_tenants))
        .route("/tenants/lookup", get(lookup_tenant_by_domain))
        .route("/tenants/{id}", get(get_tenant_by_id))
}

/// State-free authenticated route definitions.
/// Use inside `AtlasApp::authenticated_router()`. Never call `.with_state()` here.
pub fn authenticated_routes_raw() -> Router<DatabaseConnection> {
    Router::new()
        .route("/api/tenants", post(create_tenant))
        .route("/api/tenants/{id}", put(update_tenant))
        .route("/api/tenants/{id}", delete(delete_tenant))
        .route("/api/tenants/{id}/provision-admin", post(provision_admin)) // deprecated: use /api/admin/tenants/provision
        .route("/api/admin/tenants/provision", post(crate::handlers::admin_provision::provision_tenant))
        .route("/api/tenants/{id}/settings", get(get_tenant_settings))
        .route("/api/tenants/{id}/settings", post(upsert_tenant_setting))
}

/// Legacy state-finalized constructor. Used by api.rs during transition period.
/// Remove after CorePlatformApp is active and api.rs is cleaned up (Phase 3).
pub fn public_routes(db: DatabaseConnection) -> Router<DatabaseConnection> {
    public_routes_raw().with_state(db)
}

/// Legacy state-finalized constructor. Used by api.rs during transition period.
/// Remove after CorePlatformApp is active and api.rs is cleaned up (Phase 3).
pub fn authenticated_routes(db: DatabaseConnection) -> Router<DatabaseConnection> {
    authenticated_routes_raw().with_state(db)
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
    State(_db): State<DatabaseConnection>,
) -> Result<(StatusCode, Json<TenantModel>), StatusCode> {
    let _domain = params.get("domain").ok_or(StatusCode::BAD_REQUEST)?;
    
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

#[derive(Deserialize)]
pub struct ProvisionAdminPayload {
    pub email: String,
    pub first_name: String,
    pub last_name: String,
}

#[deprecated(since = "0.1.0", note = "Use POST /api/admin/tenants/provision instead")]
pub async fn provision_admin(
    Path(tenant_id): Path<Uuid>,
    State(db): State<DatabaseConnection>,
    Json(payload): Json<ProvisionAdminPayload>,
) -> Result<(StatusCode, Json<serde_json::Value>), StatusCode> {
    use crate::entities::user;
    use crate::services::auth_service::AuthService;

    tracing::warn!("Hit deprecated endpoint /api/tenants/{}/provision-admin", tenant_id);

    // 1. Check if user exists
    let existing_user = user::Entity::find()
        .filter(user::Column::Email.eq(&payload.email))
        .one(&db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let user_id = if let Some(u) = existing_user {
        u.id
    } else {
        // Create new user
        let new_user = user::ActiveModel {
            id: Set(Uuid::new_v4()),
            email: Set(payload.email.clone()),
            username: Set(payload.email.clone()),
            first_name: Set(payload.first_name),
            last_name: Set(payload.last_name),
            phone: Set("".to_string()),
            password_hash: Set("".to_string()),
            is_active: Set(true),
            created_at: Set(Utc::now()),
            updated_at: Set(Utc::now()),
            ..Default::default()
        };
        let u = new_user.insert(&db).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        
        // Phase 2: Create a user_account as Owner for this tenant
        // First, find the account associated with the tenant
        use crate::entities::{user_account, account};
        let account = account::Entity::find()
            .filter(account::Column::TenantId.eq(tenant_id))
            .one(&db)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
            
        if let Some(account) = account {
            let new_user_account = user_account::ActiveModel {
                id: Set(Uuid::new_v4()),
                user_id: Set(u.id),
                account_id: Set(account.id),
                role: Set(user_account::UserRole::Owner),
                is_active: Set(true),
                created_at: Set(Utc::now()),
                updated_at: Set(Utc::now()),
            };
            let _ = new_user_account.insert(&db).await;
        }
        
        u.id
    };

    // 2. Generate setup token (24h validity)
    let setup_token = AuthService::create_setup_token(&db, user_id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Return the token so the frontend can display the setup link or send an email.
    Ok((
        StatusCode::OK,
        Json(serde_json::json!({
            "message": "Admin provisioned successfully",
            "setup_token": setup_token.token,
            "setup_url": format!("/setup-passkey?token={}", setup_token.token)
        })),
    ))
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