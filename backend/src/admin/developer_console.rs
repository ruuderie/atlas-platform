use axum::{
    extract::{Path, State, Json},
    http::StatusCode,
};
use sea_orm::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{Utc, Duration};
use argon2::{
    password_hash::{rand_core::OsRng, PasswordHasher, SaltString},
    Argon2,
};
use rand::{distributions::Alphanumeric, Rng};
use sea_orm::ActiveValue::Set;

use crate::entities::{api_token, webhook_endpoint, webhook_delivery};

// --- API TOKENS ---

#[derive(Deserialize)]
pub struct CreateApiTokenRequest {
    pub scopes: serde_json::Value,
    // e.g., ["crm:read", "listing:write"]
}

#[derive(Serialize)]
pub struct CreateApiTokenResponse {
    pub id: Uuid,
    pub token: String, // Explicitly returned ONE TIME.
    pub scopes: serde_json::Value,
    pub expires_at: Option<chrono::DateTime<Utc>>,
}

pub async fn create_api_token(
    State(db): State<DatabaseConnection>,
    Path(tenant_id): Path<Uuid>,
    Json(payload): Json<CreateApiTokenRequest>,
) -> Result<Json<CreateApiTokenResponse>, (StatusCode, String)> {
    
    // Generate raw token
    let raw_token: String = rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(32)
        .map(char::from)
        .collect();
    let prefix = "atls_";
    let token = format!("{}{}", prefix, raw_token);

    // Hash token
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    let hash = argon2.hash_password(token.as_bytes(), &salt)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .to_string();

    let expires_at = Utc::now() + Duration::days(365);

    let new_token = api_token::ActiveModel {
        id: Set(Uuid::new_v4()),
        tenant_id: Set(tenant_id),
        token_hash: Set(hash),
        scopes: Set(payload.scopes.clone()),
        expires_at: Set(Some(expires_at.into())),
        created_at: Set(Some(Utc::now().into())),
        updated_at: Set(Some(Utc::now().into())),
    }
    .insert(&db)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(CreateApiTokenResponse {
        id: new_token.id,
        token, // ONLY RETURNED ONCE
        scopes: new_token.scopes,
        expires_at: Some(expires_at),
    }))
}

pub async fn list_api_tokens(
    State(db): State<DatabaseConnection>,
    Path(tenant_id): Path<Uuid>,
) -> Result<Json<Vec<api_token::Model>>, (StatusCode, String)> {
    let tokens = api_token::Entity::find()
        .filter(api_token::Column::TenantId.eq(tenant_id))
        .all(&db)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    
    Ok(Json(tokens))
}

pub async fn revoke_api_token(
    State(db): State<DatabaseConnection>,
    Path((_tenant_id, token_id)): Path<(Uuid, Uuid)>,
) -> Result<StatusCode, (StatusCode, String)> {
    api_token::Entity::delete_by_id(token_id)
        .exec(&db)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    
    Ok(StatusCode::NO_CONTENT)
}

// --- WEBHOOK ENDPOINTS ---

#[derive(Deserialize)]
pub struct CreateWebhookRequest {
    pub target_url: String,
    pub subscribed_events: serde_json::Value,
}

pub async fn create_webhook_endpoint(
    State(db): State<DatabaseConnection>,
    Path(tenant_id): Path<Uuid>,
    Json(payload): Json<CreateWebhookRequest>,
) -> Result<Json<webhook_endpoint::Model>, (StatusCode, String)> {
    // Generate secret key (whsec_...)
    let raw_secret: String = rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(32)
        .map(char::from)
        .collect();
    let secret_key = format!("whsec_{}", raw_secret);

    let new_endpoint = webhook_endpoint::ActiveModel {
        id: Set(Uuid::new_v4()),
        tenant_id: Set(tenant_id),
        target_url: Set(payload.target_url),
        secret_key: Set(secret_key),
        subscribed_events: Set(payload.subscribed_events),
        is_active: Set(true),
        created_at: Set(Some(Utc::now().into())),
        updated_at: Set(Some(Utc::now().into())),
    }
    .insert(&db)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(new_endpoint))
}

pub async fn list_webhook_endpoints(
    State(db): State<DatabaseConnection>,
    Path(tenant_id): Path<Uuid>,
) -> Result<Json<Vec<webhook_endpoint::Model>>, (StatusCode, String)> {
    let endpoints = webhook_endpoint::Entity::find()
        .filter(webhook_endpoint::Column::TenantId.eq(tenant_id))
        .all(&db)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    
    Ok(Json(endpoints))
}

pub async fn delete_webhook_endpoint(
    State(db): State<DatabaseConnection>,
    Path((_tenant_id, endpoint_id)): Path<(Uuid, Uuid)>,
) -> Result<StatusCode, (StatusCode, String)> {
    webhook_endpoint::Entity::delete_by_id(endpoint_id)
        .exec(&db)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    
    Ok(StatusCode::NO_CONTENT)
}

// --- WEBHOOK DELIVERIES ---

pub async fn list_webhook_deliveries(
    State(db): State<DatabaseConnection>,
    Path(tenant_id): Path<Uuid>,
) -> Result<Json<Vec<webhook_delivery::Model>>, (StatusCode, String)> {
    let deliveries = webhook_delivery::Entity::find()
        .filter(webhook_delivery::Column::TenantId.eq(tenant_id))
        .order_by_desc(webhook_delivery::Column::CreatedAt)
        .limit(100) // latest 100
        .all(&db)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    
    Ok(Json(deliveries))
}
