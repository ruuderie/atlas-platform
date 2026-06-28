//! Folio Notification Handler
//!
//! Routes:
//!   GET    /api/folio/notifications              — paginated list (newest first)
//!   GET    /api/folio/notifications/unread-count — badge count
//!   POST   /api/folio/notifications/{id}/read    — mark one as read
//!   POST   /api/folio/notifications/read-all     — mark all as read
//!   DELETE /api/folio/notifications/{id}         — dismiss (soft-delete)
//!
//!   GET    /api/folio/notification-prefs                    — list user's channel prefs
//!   PUT    /api/folio/notification-prefs/{channel}          — upsert channel pref
//!   DELETE /api/folio/notification-prefs/{channel}          — remove channel pref
//!
//!   GET    /api/folio/notification-channel-settings         — tenant channel config (keys masked)
//!   PUT    /api/folio/notification-channel-settings         — upsert tenant channel setting
//!
//! All routes require authentication via Bearer session token.

use axum::{
    extract::{Extension, Json, Path, Query},
    http::StatusCode,
    response::IntoResponse,
    routing::{delete, get, post, put},
    Router,
};
use chrono::Utc;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, QueryOrder,
    QuerySelect, Set,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    entities::{atlas_notification, atlas_user_notification_pref, tenant_setting, user},
    services::notification_service::NotificationService,
};

// ── Local helper: resolve caller's tenant_id ──────────────────────────────────

async fn resolve_tenant_id(
    db: &DatabaseConnection,
    user_id: Uuid,
) -> Result<Uuid, StatusCode> {
    let user_accounts = crate::entities::user_account::Entity::find()
        .filter(crate::entities::user_account::Column::UserId.eq(user_id))
        .all(db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let account_ids: Vec<Uuid> = user_accounts.into_iter().map(|ua| ua.account_id).collect();

    let profile = crate::entities::profile::Entity::find()
        .filter(crate::entities::profile::Column::AccountId.is_in(account_ids))
        .one(db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::FORBIDDEN)?;

    Ok(profile.tenant_id)
}

// ── Router ────────────────────────────────────────────────────────────────────

pub fn authenticated_routes_raw() -> Router<DatabaseConnection> {
    Router::new()
        // Inbox
        .route("/api/folio/notifications",                get(list_notifications))
        .route("/api/folio/notifications/unread-count",   get(get_unread_count))
        .route("/api/folio/notifications/read-all",       post(mark_all_read))
        .route("/api/folio/notifications/{id}/read",      post(mark_read))
        .route("/api/folio/notifications/{id}",           delete(dismiss))
        // User channel prefs
        .route("/api/folio/notification-prefs",           get(list_prefs))
        .route("/api/folio/notification-prefs/{channel}", put(upsert_pref).delete(delete_pref))
        // Tenant channel settings (operator/admin)
        .route("/api/folio/notification-channel-settings", get(get_channel_settings).put(upsert_channel_setting))
}

// ── Shared helpers ─────────────────────────────────────────────────────────────

fn session_token(headers: &axum::http::HeaderMap) -> Result<String, StatusCode> {
    headers
        .get(axum::http::header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
        .map(|s| s.to_string())
        .ok_or(StatusCode::UNAUTHORIZED)
}

// ── GET /api/folio/notifications ──────────────────────────────────────────────

#[derive(Deserialize)]
struct ListQuery {
    limit:  Option<u64>,
    offset: Option<u64>,
    unread: Option<bool>,
    #[serde(rename = "type")]
    ntype:  Option<String>,
}

#[derive(Serialize)]
struct NotificationRow {
    pub id:                 Uuid,
    pub notification_type:  String,
    pub title:              String,
    pub body:               String,
    pub priority:           String,
    pub entity_type:        Option<String>,
    pub entity_id:          Option<Uuid>,
    pub metadata:           Option<serde_json::Value>,
    pub channels_attempted: serde_json::Value,
    pub read_at:            Option<String>,
    pub created_at:         String,
}

async fn list_notifications(
    Extension(db): Extension<DatabaseConnection>,
    Extension(current_user): Extension<user::Model>,
    Query(q): Query<ListQuery>,
) -> Result<impl IntoResponse, StatusCode> {
    let tenant_id = resolve_tenant_id(&db, current_user.id).await?;
    let limit     = q.limit.unwrap_or(50).min(200);
    let offset    = q.offset.unwrap_or(0);

    let mut query = atlas_notification::Entity::find()
        .filter(atlas_notification::Column::UserId.eq(current_user.id))
        .filter(atlas_notification::Column::TenantId.eq(tenant_id))
        .filter(atlas_notification::Column::DismissedAt.is_null())
        .order_by_desc(atlas_notification::Column::CreatedAt)
        .limit(limit)
        .offset(offset);

    if q.unread.unwrap_or(false) {
        query = query.filter(atlas_notification::Column::ReadAt.is_null());
    }

    if let Some(ref t) = q.ntype {
        query = query.filter(atlas_notification::Column::NotificationType.eq(t.as_str()));
    }

    // query is not Clone — rebuild with same filters for count
    let rows = atlas_notification::Entity::find()
        .filter(atlas_notification::Column::UserId.eq(current_user.id))
        .filter(atlas_notification::Column::TenantId.eq(tenant_id))
        .filter(atlas_notification::Column::DismissedAt.is_null())
        .order_by_desc(atlas_notification::Column::CreatedAt)
        .limit(limit)
        .offset(offset)
        .all(&db)
        .await
        .map_err(|e| {
            tracing::error!("list_notifications: {e:#}");
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    let items: Vec<NotificationRow> = rows.into_iter().map(|n| NotificationRow {
        id:                 n.id,
        notification_type:  n.notification_type,
        title:              n.title,
        body:               n.body,
        priority:           n.priority,
        entity_type:        n.entity_type,
        entity_id:          n.entity_id,
        metadata:           n.metadata,
        channels_attempted: n.channels_attempted,
        read_at:            n.read_at.map(|t| t.to_rfc3339()),
        created_at:         n.created_at.to_rfc3339(),
    }).collect();

    Ok(axum::response::Json(items))
}

// ── GET /api/folio/notifications/unread-count ─────────────────────────────────

#[derive(Serialize)]
struct UnreadCountResponse {
    count: u64,
}

async fn get_unread_count(
    Extension(db): Extension<DatabaseConnection>,
    Extension(current_user): Extension<user::Model>,
) -> Result<impl IntoResponse, StatusCode> {
    let tenant_id = resolve_tenant_id(&db, current_user.id).await?;
    let count = NotificationService::unread_count(&db, current_user.id, tenant_id)
        .await
        .map_err(|e| {
            tracing::error!("get_unread_count: {e:#}");
            StatusCode::INTERNAL_SERVER_ERROR
        })?;
    Ok(axum::response::Json(UnreadCountResponse { count }))
}

// ── POST /api/folio/notifications/:id/read ────────────────────────────────────

async fn mark_read(
    Extension(db): Extension<DatabaseConnection>,
    Extension(current_user): Extension<user::Model>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, StatusCode> {
    NotificationService::mark_read(&db, id, current_user.id)
        .await
        .map_err(|e| {
            tracing::error!("mark_read: {e:#}");
            StatusCode::NOT_FOUND
        })?;
    Ok(StatusCode::NO_CONTENT)
}

// ── POST /api/folio/notifications/read-all ────────────────────────────────────

async fn mark_all_read(
    Extension(db): Extension<DatabaseConnection>,
    Extension(current_user): Extension<user::Model>,
) -> Result<impl IntoResponse, StatusCode> {
    let tenant_id = resolve_tenant_id(&db, current_user.id).await?;
    let count = NotificationService::mark_all_read(&db, current_user.id, tenant_id)
        .await
        .map_err(|e| {
            tracing::error!("mark_all_read: {e:#}");
            StatusCode::INTERNAL_SERVER_ERROR
        })?;
    Ok(axum::response::Json(serde_json::json!({ "marked_read": count })))
}

// ── DELETE /api/folio/notifications/:id ──────────────────────────────────────

async fn dismiss(
    Extension(db): Extension<DatabaseConnection>,
    Extension(current_user): Extension<user::Model>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, StatusCode> {
    NotificationService::dismiss(&db, id, current_user.id)
        .await
        .map_err(|e| {
            tracing::error!("dismiss: {e:#}");
            StatusCode::NOT_FOUND
        })?;
    Ok(StatusCode::NO_CONTENT)
}

// ── GET /api/folio/notification-prefs ─────────────────────────────────────────

#[derive(Serialize)]
struct PrefRow {
    pub id:         Uuid,
    pub channel:    String,
    pub config:     serde_json::Value,
    pub enabled:    bool,
    pub applies_to: Vec<String>,
    pub updated_at: String,
}

async fn list_prefs(
    Extension(db): Extension<DatabaseConnection>,
    Extension(current_user): Extension<user::Model>,
) -> Result<impl IntoResponse, StatusCode> {
    let tenant_id = resolve_tenant_id(&db, current_user.id).await?;

    let rows = atlas_user_notification_pref::Entity::find()
        .filter(atlas_user_notification_pref::Column::UserId.eq(current_user.id))
        .filter(atlas_user_notification_pref::Column::TenantId.eq(tenant_id))
        .all(&db)
        .await
        .map_err(|e| {
            tracing::error!("list_prefs: {e:#}");
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    let items: Vec<PrefRow> = rows.into_iter().map(|p| PrefRow {
        id:         p.id,
        channel:    p.channel,
        config:     p.config,
        enabled:    p.enabled,
        applies_to: p.applies_to,
        updated_at: p.updated_at.to_rfc3339(),
    }).collect();

    Ok(axum::response::Json(items))
}

// ── PUT /api/folio/notification-prefs/:channel ───────────────────────────────

#[derive(Deserialize)]
struct UpsertPrefInput {
    config:     serde_json::Value,
    enabled:    Option<bool>,
    applies_to: Option<Vec<String>>,
}

const ALLOWED_CHANNELS: &[&str] = &["in_app", "sms", "email", "telegram", "whatsapp"];

async fn upsert_pref(
    Extension(db): Extension<DatabaseConnection>,
    Extension(current_user): Extension<user::Model>,
    Path(channel): Path<String>,
    Json(input): Json<UpsertPrefInput>,
) -> Result<impl IntoResponse, StatusCode> {
    if !ALLOWED_CHANNELS.contains(&channel.as_str()) {
        return Err(StatusCode::BAD_REQUEST);
    }

    let tenant_id = resolve_tenant_id(&db, current_user.id).await?;

    // Check for existing pref
    let existing = atlas_user_notification_pref::Entity::find()
        .filter(atlas_user_notification_pref::Column::UserId.eq(current_user.id))
        .filter(atlas_user_notification_pref::Column::TenantId.eq(tenant_id))
        .filter(atlas_user_notification_pref::Column::Channel.eq(channel.as_str()))
        .one(&db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    match existing {
        Some(p) => {
            let mut active: atlas_user_notification_pref::ActiveModel = p.into();
            active.config     = Set(input.config);
            active.enabled    = Set(input.enabled.unwrap_or(true));
            active.applies_to = Set(input.applies_to.unwrap_or_default());
            active.update(&db).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        }
        None => {
            atlas_user_notification_pref::ActiveModel {
                id:         Set(Uuid::new_v4()),
                user_id:    Set(current_user.id),
                tenant_id:  Set(tenant_id),
                channel:    Set(channel),
                config:     Set(input.config),
                enabled:    Set(input.enabled.unwrap_or(true)),
                applies_to: Set(input.applies_to.unwrap_or_default()),
                created_at: Set(Utc::now()),
                updated_at: Set(Utc::now()),
            }.insert(&db).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        }
    }

    Ok(StatusCode::NO_CONTENT)
}

// ── DELETE /api/folio/notification-prefs/:channel ────────────────────────────

async fn delete_pref(
    Extension(db): Extension<DatabaseConnection>,
    Extension(current_user): Extension<user::Model>,
    Path(channel): Path<String>,
) -> Result<impl IntoResponse, StatusCode> {
    let tenant_id = resolve_tenant_id(&db, current_user.id).await?;

    atlas_user_notification_pref::Entity::delete_many()
        .filter(atlas_user_notification_pref::Column::UserId.eq(current_user.id))
        .filter(atlas_user_notification_pref::Column::TenantId.eq(tenant_id))
        .filter(atlas_user_notification_pref::Column::Channel.eq(channel.as_str()))
        .exec(&db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(StatusCode::NO_CONTENT)
}

// ── GET /api/folio/notification-channel-settings ─────────────────────────────
//
// Returns tenant-level channel config from tenant_setting.
// Sensitive fields (tokens, passwords) are masked with "••••••••".
// The UI uses this to show what's configured without exposing credentials.

const CHANNEL_SETTING_KEYS: &[&str] = &[
    "notify_channel_telegram_enabled",
    "notify_channel_telegram_bot_token",
    "notify_channel_telegram_default_chat_id",
    "notify_channel_whatsapp_enabled",
    "notify_channel_whatsapp_provider",
    "notify_channel_whatsapp_from",
    "notify_channel_sms_enabled",
    "notify_channel_sms_from",
    "notify_channel_email_enabled",
];

const MASKED_KEYS: &[&str] = &[
    "notify_channel_telegram_bot_token",
];

#[derive(Serialize)]
struct ChannelSettingRow {
    key:      String,
    value:    String,
    is_set:   bool,
    is_masked: bool,
}

async fn get_channel_settings(
    Extension(db): Extension<DatabaseConnection>,
    Extension(current_user): Extension<user::Model>,
) -> Result<impl IntoResponse, StatusCode> {
    let tenant_id = resolve_tenant_id(&db, current_user.id).await?;

    let settings: std::collections::HashMap<String, String> = tenant_setting::Entity::find()
        .filter(tenant_setting::Column::TenantId.eq(tenant_id))
        .all(&db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .into_iter()
        .map(|s| (s.key, s.value))
        .collect();

    let rows: Vec<ChannelSettingRow> = CHANNEL_SETTING_KEYS.iter().map(|&key| {
        let value   = settings.get(key).cloned().unwrap_or_default();
        let is_set  = !value.is_empty();
        let is_masked = MASKED_KEYS.contains(&key);
        ChannelSettingRow {
            key:       key.to_string(),
            value:     if is_masked && is_set { "••••••••".to_string() } else { value },
            is_set,
            is_masked,
        }
    }).collect();

    Ok(axum::response::Json(rows))
}

// ── PUT /api/folio/notification-channel-settings ─────────────────────────────

#[derive(Deserialize)]
struct UpsertSettingInput {
    key:   String,
    value: String,
}

async fn upsert_channel_setting(
    Extension(db): Extension<DatabaseConnection>,
    Extension(current_user): Extension<user::Model>,
    Json(input): Json<UpsertSettingInput>,
) -> Result<impl IntoResponse, StatusCode> {
    // Only allow whitelisted keys
    if !CHANNEL_SETTING_KEYS.contains(&input.key.as_str()) {
        return Err(StatusCode::BAD_REQUEST);
    }

    let tenant_id = resolve_tenant_id(&db, current_user.id).await?;

    // Upsert into tenant_setting
    let existing = tenant_setting::Entity::find()
        .filter(tenant_setting::Column::TenantId.eq(tenant_id))
        .filter(tenant_setting::Column::Key.eq(input.key.as_str()))
        .one(&db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    match existing {
        Some(s) => {
            let mut active: tenant_setting::ActiveModel = s.into();
            active.value = Set(input.value);
            active.update(&db).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        }
        None => {
            tenant_setting::ActiveModel {
                id:           Set(Uuid::new_v4()),
                tenant_id:    Set(tenant_id),
                key:          Set(input.key),
                value:        Set(input.value),
                is_encrypted: Set(false),
                updated_at:   Set(Utc::now()),
                created_at:   Set(Utc::now()),
            }.insert(&db).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        }
    }

    Ok(StatusCode::NO_CONTENT)
}
