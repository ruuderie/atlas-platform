//! Admin — Feature Flags handler
//!
//! Manages the platform feature flag registry:
//! - Global on/off toggle with canary rollout percentages (0-100%)
//! - Plan-gated visibility (Enterprise / Growth / Starter)
//! - Per-tenant (NI) override grants or denies
//! - Per-app-instance enablements (grant / deny / inherit)
//! - Immutable audit trail for every mutation
//!
//! # Routes
//!
//! ```ignore
//! GET  /api/admin/flags
//!      List all flags with their overrides and audit logs eagerly loaded.
//!      -> 200 Vec<FeatureFlagModel>
//!
//! POST /api/admin/flags
//!      Create a new flag. Writes audit log entry.
//!      Body: CreateFlagInput
//!      -> 201 FeatureFlagModel
//!
//! PUT  /api/admin/flags/{key}
//!      Update a flag (is_enabled, global_rollout_pct, plan_gate, description).
//!      Writes audit log entry.
//!      Body: UpdateFlagInput
//!      -> 200 FeatureFlagModel
//!
//! POST /api/admin/flags/{key}/overrides
//!      Add or update a per-tenant NI override. Writes audit log entry.
//!      Body: CreateOverrideInput
//!      -> 201 FlagOverrideModel
//!
//! DELETE /api/admin/flags/{key}/overrides/{tenant_id}
//!      Remove a per-tenant override. Writes audit log entry.
//!      -> 204 No Content
//!
//! GET  /api/admin/app-instances/{id}/feature-flags
//!      Catalog flags with this instance's enablement (or inherit).
//!
//! PUT  /api/admin/app-instances/{id}/feature-flags
//!      Upsert / clear instance enablements.
//!      Body: { updates: [{ flag_key, effect: grant|deny|null }] }
//! ```

use axum::{
    Router,
    extract::{Extension, Json, Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{delete, get, post, put},
};
use chrono::Utc;
use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter,
    QueryOrder,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::entities::{
    app_instance, atlas_flag_instance_enablement, feature_flag, flag_audit_log, flag_override, user,
};
use crate::services::audit::AuditService;
use crate::types::flags::FlagEffect;

// ── Route registration ────────────────────────────────────────────────────────

pub fn routes_raw() -> Router<DatabaseConnection> {
    Router::new()
        .route("/api/admin/flags", get(list_flags).post(create_flag))
        .route("/api/admin/flags/{key}", put(update_flag))
        .route("/api/admin/flags/{key}/overrides", post(add_override))
        .route(
            "/api/admin/flags/{key}/overrides/{tenant_id}",
            delete(remove_override),
        )
        .route(
            "/api/admin/app-instances/{id}/feature-flags",
            get(list_instance_feature_flags).put(update_instance_feature_flags),
        )
}

// ── Response / input types ────────────────────────────────────────────────────

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FlagOverrideModel {
    pub id: Uuid,
    pub flag_id: Uuid,
    pub tenant_id: Uuid,
    pub override_type: String,
    pub rollout_pct: i32,
    pub reason: String,
    pub jira: Option<String>,
    pub changed_by: String,
    pub created_at: chrono::DateTime<Utc>,
}

impl From<flag_override::Model> for FlagOverrideModel {
    fn from(m: flag_override::Model) -> Self {
        Self {
            id: m.id,
            flag_id: m.flag_id,
            tenant_id: m.tenant_id,
            override_type: m.override_type,
            rollout_pct: m.rollout_pct,
            reason: m.reason,
            jira: m.jira,
            changed_by: m.changed_by,
            created_at: m.created_at,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FlagAuditLogModel {
    pub id: Uuid,
    pub flag_id: Uuid,
    pub user_id: String,
    pub action: String,
    pub created_at: chrono::DateTime<Utc>,
}

impl From<flag_audit_log::Model> for FlagAuditLogModel {
    fn from(m: flag_audit_log::Model) -> Self {
        Self {
            id: m.id,
            flag_id: m.flag_id,
            user_id: m.user_id,
            action: m.action,
            created_at: m.created_at,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FeatureFlagModel {
    pub id: Uuid,
    pub key: String,
    pub description: String,
    pub is_enabled: bool,
    pub has_global: bool,
    pub global_rollout_pct: i32,
    pub is_plan_gated: bool,
    pub plan_gate_tier: Option<String>,
    pub jira: Option<String>,
    pub owner: String,
    pub created_at: chrono::DateTime<Utc>,
    pub overrides: Vec<FlagOverrideModel>,
    pub audit_logs: Vec<FlagAuditLogModel>,
}

#[derive(Debug, Deserialize)]
pub struct CreateFlagInput {
    pub key: String,
    pub description: String,
    pub has_global: Option<bool>,
    pub global_rollout_pct: Option<i32>,
    pub is_plan_gated: Option<bool>,
    pub plan_gate_tier: Option<String>,
    pub jira: Option<String>,
    pub owner: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateFlagInput {
    pub description: Option<String>,
    pub is_enabled: Option<bool>,
    pub has_global: Option<bool>,
    pub global_rollout_pct: Option<i32>,
    pub is_plan_gated: Option<bool>,
    pub plan_gate_tier: Option<String>,
    pub jira: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CreateOverrideInput {
    pub tenant_id: Uuid,
    pub override_type: String, // "grant" or "deny"
    pub rollout_pct: Option<i32>,
    pub reason: String,
    pub jira: Option<String>,
    pub changed_by: Option<String>,
}

/// One catalog flag as seen from an app-instance config surface.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct InstanceFlagRow {
    pub flag_key: String,
    pub description: String,
    pub catalog_enabled: bool,
    pub has_global: bool,
    pub global_rollout_pct: i32,
    /// Explicit instance effect, or `null` when inheriting tenant/global resolution.
    pub effect: Option<FlagEffect>,
    pub rollout_pct: Option<i32>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct InstanceFlagsResponse {
    pub flags: Vec<InstanceFlagRow>,
}

#[derive(Debug, Deserialize)]
pub struct InstanceFlagUpdateItem {
    pub flag_key: String,
    /// `grant` | `deny` to upsert; `null` to clear (inherit).
    pub effect: Option<FlagEffect>,
    pub rollout_pct: Option<i32>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateInstanceFlagsInput {
    pub updates: Vec<InstanceFlagUpdateItem>,
}

// ── Helper ────────────────────────────────────────────────────────────────────

async fn write_audit(
    db: &DatabaseConnection,
    flag_id: Uuid,
    user_id: &str,
    action: &str,
) -> Result<(), StatusCode> {
    flag_audit_log::ActiveModel {
        id: Set(Uuid::new_v4()),
        flag_id: Set(flag_id),
        user_id: Set(user_id.to_string()),
        action: Set(action.to_string()),
        created_at: Set(Utc::now()),
    }
    .insert(db)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(())
}

async fn load_full_flag(
    db: &DatabaseConnection,
    flag_id: Uuid,
) -> Result<FeatureFlagModel, StatusCode> {
    let flag = feature_flag::Entity::find_by_id(flag_id)
        .one(db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;

    let overrides = flag_override::Entity::find()
        .filter(flag_override::Column::FlagId.eq(flag_id))
        .order_by_asc(flag_override::Column::CreatedAt)
        .all(db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let audit_logs = flag_audit_log::Entity::find()
        .filter(flag_audit_log::Column::FlagId.eq(flag_id))
        .order_by_desc(flag_audit_log::Column::CreatedAt)
        .all(db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(FeatureFlagModel {
        id: flag.id,
        key: flag.key,
        description: flag.description,
        is_enabled: flag.is_enabled,
        has_global: flag.has_global,
        global_rollout_pct: flag.global_rollout_pct,
        is_plan_gated: flag.is_plan_gated,
        plan_gate_tier: flag.plan_gate_tier,
        jira: flag.jira,
        owner: flag.owner,
        created_at: flag.created_at,
        overrides: overrides.into_iter().map(FlagOverrideModel::from).collect(),
        audit_logs: audit_logs
            .into_iter()
            .map(FlagAuditLogModel::from)
            .collect(),
    })
}

async fn require_app_instance(
    db: &DatabaseConnection,
    id: Uuid,
) -> Result<app_instance::Model, StatusCode> {
    app_instance::Entity::find_by_id(id)
        .one(db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)
}

// ── Handlers ──────────────────────────────────────────────────────────────────

/// GET /api/admin/flags — list all flags with overrides + audit logs
pub async fn list_flags(
    State(db): State<DatabaseConnection>,
    Extension(_current_user): Extension<user::Model>,
) -> Result<impl IntoResponse, StatusCode> {
    let flags = feature_flag::Entity::find()
        .order_by_asc(feature_flag::Column::Key)
        .all(&db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let mut result = Vec::with_capacity(flags.len());
    for flag in flags {
        let flag_id = flag.id;
        let overrides = flag_override::Entity::find()
            .filter(flag_override::Column::FlagId.eq(flag_id))
            .order_by_asc(flag_override::Column::CreatedAt)
            .all(&db)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

        let audit_logs = flag_audit_log::Entity::find()
            .filter(flag_audit_log::Column::FlagId.eq(flag_id))
            .order_by_desc(flag_audit_log::Column::CreatedAt)
            .all(&db)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

        result.push(FeatureFlagModel {
            id: flag.id,
            key: flag.key,
            description: flag.description,
            is_enabled: flag.is_enabled,
            has_global: flag.has_global,
            global_rollout_pct: flag.global_rollout_pct,
            is_plan_gated: flag.is_plan_gated,
            plan_gate_tier: flag.plan_gate_tier,
            jira: flag.jira,
            owner: flag.owner,
            created_at: flag.created_at,
            overrides: overrides.into_iter().map(FlagOverrideModel::from).collect(),
            audit_logs: audit_logs
                .into_iter()
                .map(FlagAuditLogModel::from)
                .collect(),
        });
    }

    Ok(axum::Json(result))
}

/// POST /api/admin/flags — create a new flag
pub async fn create_flag(
    State(db): State<DatabaseConnection>,
    Extension(current_user): Extension<user::Model>,
    Json(input): Json<CreateFlagInput>,
) -> Result<impl IntoResponse, StatusCode> {
    let key = input.key.trim().to_lowercase().replace(' ', "_");
    if key.is_empty() {
        return Err(StatusCode::BAD_REQUEST);
    }

    let has_global = input.has_global.unwrap_or(true);
    let rollout = input
        .global_rollout_pct
        .unwrap_or(if has_global { 100 } else { 0 });
    let owner = input.owner.unwrap_or_else(|| current_user.email.clone());

    let flag_id = Uuid::new_v4();
    feature_flag::ActiveModel {
        id: Set(flag_id),
        key: Set(key.clone()),
        description: Set(input.description.clone()),
        is_enabled: Set(true),
        has_global: Set(has_global),
        global_rollout_pct: Set(rollout),
        is_plan_gated: Set(input.is_plan_gated.unwrap_or(false)),
        plan_gate_tier: Set(input.plan_gate_tier.clone()),
        jira: Set(input.jira.clone()),
        owner: Set(owner.clone()),
        created_at: Set(Utc::now()),
    }
    .insert(&db)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    write_audit(
        &db,
        flag_id,
        &owner,
        &format!(
            "Flag created at {}% rollout{}",
            rollout,
            input
                .jira
                .as_deref()
                .map(|j| format!(" · {}", j))
                .unwrap_or_default()
        ),
    )
    .await?;

    let model = load_full_flag(&db, flag_id).await?;
    Ok((StatusCode::CREATED, axum::Json(model)))
}

/// PUT /api/admin/flags/{key} — update a flag
pub async fn update_flag(
    State(db): State<DatabaseConnection>,
    Extension(current_user): Extension<user::Model>,
    Path(key): Path<String>,
    Json(input): Json<UpdateFlagInput>,
) -> Result<impl IntoResponse, StatusCode> {
    let flag = feature_flag::Entity::find()
        .filter(feature_flag::Column::Key.eq(&key))
        .one(&db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;

    let flag_id = flag.id;
    let prev_enabled = flag.is_enabled;
    let prev_rollout = flag.global_rollout_pct;

    let mut active: feature_flag::ActiveModel = flag.into();
    if let Some(desc) = input.description {
        active.description = Set(desc);
    }
    if let Some(enabled) = input.is_enabled {
        active.is_enabled = Set(enabled);
    }
    if let Some(has_global) = input.has_global {
        active.has_global = Set(has_global);
    }
    if let Some(rollout) = input.global_rollout_pct {
        active.global_rollout_pct = Set(rollout.clamp(0, 100));
    }
    if let Some(gated) = input.is_plan_gated {
        active.is_plan_gated = Set(gated);
    }
    if let Some(tier) = input.plan_gate_tier {
        active.plan_gate_tier = Set(Some(tier));
    }
    if let Some(jira) = input.jira {
        active.jira = Set(Some(jira));
    }
    active
        .update(&db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let actor = &current_user.email;

    // Build audit description
    let new_enabled = input.is_enabled.unwrap_or(prev_enabled);
    let new_rollout = input.global_rollout_pct.unwrap_or(prev_rollout);
    let action = if input.is_enabled.is_some() && new_enabled != prev_enabled {
        format!(
            "Global kill-switch toggled to {}",
            if new_enabled { "ON" } else { "OFF" }
        )
    } else if input.global_rollout_pct.is_some() && new_rollout != prev_rollout {
        format!("Global rollout {}% → {}%", prev_rollout, new_rollout)
    } else {
        "Flag settings updated".to_string()
    };

    write_audit(&db, flag_id, actor, &action).await?;

    AuditService::log_action(
        db.clone(),
        None,
        Some(current_user.id),
        "feature_flag.updated".to_string(),
        "FeatureFlag".to_string(),
        flag_id,
        Some(serde_json::json!({
            "is_enabled": prev_enabled,
            "global_rollout_pct": prev_rollout,
        })),
        Some(serde_json::json!({
            "is_enabled": new_enabled,
            "global_rollout_pct": new_rollout,
            "key": key,
            "action": action,
        })),
        None,
    );

    let model = load_full_flag(&db, flag_id).await?;
    Ok(axum::Json(model))
}

/// POST /api/admin/flags/{key}/overrides — add or update a per-tenant NI override
pub async fn add_override(
    State(db): State<DatabaseConnection>,
    Extension(current_user): Extension<user::Model>,
    Path(key): Path<String>,
    Json(input): Json<CreateOverrideInput>,
) -> Result<impl IntoResponse, StatusCode> {
    let flag = feature_flag::Entity::find()
        .filter(feature_flag::Column::Key.eq(&key))
        .one(&db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;

    let flag_id = flag.id;
    let actor = input
        .changed_by
        .clone()
        .unwrap_or_else(|| current_user.email.clone());

    let effect =
        FlagEffect::try_from(input.override_type.as_str()).map_err(|_| StatusCode::BAD_REQUEST)?;

    // Upsert: remove existing override for same tenant if any
    flag_override::Entity::delete_many()
        .filter(flag_override::Column::FlagId.eq(flag_id))
        .filter(flag_override::Column::TenantId.eq(input.tenant_id))
        .exec(&db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let rollout = if effect == FlagEffect::Deny {
        0
    } else {
        input.rollout_pct.unwrap_or(100).clamp(0, 100)
    };

    let ovr = flag_override::ActiveModel {
        id: Set(Uuid::new_v4()),
        flag_id: Set(flag_id),
        tenant_id: Set(input.tenant_id),
        override_type: Set(effect.to_string()),
        rollout_pct: Set(rollout),
        reason: Set(input.reason.clone()),
        jira: Set(input.jira.clone()),
        changed_by: Set(actor.clone()),
        created_at: Set(Utc::now()),
    }
    .insert(&db)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    write_audit(
        &db,
        flag_id,
        &actor,
        &format!(
            "NI {} added: {} (Reason: {})",
            match effect {
                FlagEffect::Deny => "Deny",
                FlagEffect::Grant => "Grant",
            },
            input.tenant_id,
            input.reason
        ),
    )
    .await?;

    Ok((
        StatusCode::CREATED,
        axum::Json(FlagOverrideModel::from(ovr)),
    ))
}

/// DELETE /api/admin/flags/{key}/overrides/{tenant_id} — remove a per-tenant override
pub async fn remove_override(
    State(db): State<DatabaseConnection>,
    Extension(current_user): Extension<user::Model>,
    Path((key, tenant_id)): Path<(String, Uuid)>,
) -> Result<impl IntoResponse, StatusCode> {
    let flag = feature_flag::Entity::find()
        .filter(feature_flag::Column::Key.eq(&key))
        .one(&db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;

    let flag_id = flag.id;

    flag_override::Entity::delete_many()
        .filter(flag_override::Column::FlagId.eq(flag_id))
        .filter(flag_override::Column::TenantId.eq(tenant_id))
        .exec(&db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let actor = &current_user.email;
    write_audit(
        &db,
        flag_id,
        actor,
        &format!("NI Override removed: {}", tenant_id),
    )
    .await?;

    Ok(StatusCode::NO_CONTENT)
}

/// GET /api/admin/app-instances/{id}/feature-flags
pub async fn list_instance_feature_flags(
    State(db): State<DatabaseConnection>,
    Extension(_current_user): Extension<user::Model>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, StatusCode> {
    require_app_instance(&db, id).await?;

    let flags = feature_flag::Entity::find()
        .order_by_asc(feature_flag::Column::Key)
        .all(&db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let enablements = atlas_flag_instance_enablement::Entity::find()
        .filter(atlas_flag_instance_enablement::Column::AppInstanceId.eq(id))
        .all(&db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let by_key: std::collections::HashMap<String, atlas_flag_instance_enablement::Model> =
        enablements
            .into_iter()
            .map(|e| (e.flag_key.clone(), e))
            .collect();

    let rows: Vec<InstanceFlagRow> = flags
        .into_iter()
        .map(|flag| {
            let en = by_key.get(&flag.key);
            let effect = en.and_then(|e| FlagEffect::try_from(e.effect.as_str()).ok());
            InstanceFlagRow {
                flag_key: flag.key,
                description: flag.description,
                catalog_enabled: flag.is_enabled,
                has_global: flag.has_global,
                global_rollout_pct: flag.global_rollout_pct,
                effect,
                rollout_pct: en.map(|e| e.rollout_pct),
            }
        })
        .collect();

    Ok(axum::Json(InstanceFlagsResponse { flags: rows }))
}

/// PUT /api/admin/app-instances/{id}/feature-flags
pub async fn update_instance_feature_flags(
    State(db): State<DatabaseConnection>,
    Extension(current_user): Extension<user::Model>,
    Path(id): Path<Uuid>,
    Json(input): Json<UpdateInstanceFlagsInput>,
) -> Result<impl IntoResponse, StatusCode> {
    require_app_instance(&db, id).await?;

    let actor_id = current_user.id;
    let now = Utc::now();

    for item in &input.updates {
        let key = item.flag_key.trim();
        if key.is_empty() {
            return Err(StatusCode::BAD_REQUEST);
        }

        // Ensure the key exists in the catalog
        let flag = feature_flag::Entity::find()
            .filter(feature_flag::Column::Key.eq(key))
            .one(&db)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
            .ok_or(StatusCode::BAD_REQUEST)?;

        match item.effect {
            None => {
                atlas_flag_instance_enablement::Entity::delete_many()
                    .filter(atlas_flag_instance_enablement::Column::FlagKey.eq(key))
                    .filter(atlas_flag_instance_enablement::Column::AppInstanceId.eq(id))
                    .exec(&db)
                    .await
                    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

                write_audit(
                    &db,
                    flag.id,
                    &current_user.email,
                    &format!("Instance enablement cleared for {}", id),
                )
                .await?;
            }
            Some(effect) => {
                let rollout = if effect == FlagEffect::Deny {
                    0
                } else {
                    item.rollout_pct.unwrap_or(100).clamp(0, 100)
                };

                let existing = atlas_flag_instance_enablement::Entity::find()
                    .filter(atlas_flag_instance_enablement::Column::FlagKey.eq(key))
                    .filter(atlas_flag_instance_enablement::Column::AppInstanceId.eq(id))
                    .one(&db)
                    .await
                    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

                if let Some(row) = existing {
                    let mut active: atlas_flag_instance_enablement::ActiveModel = row.into();
                    active.effect = Set(effect.to_string());
                    active.rollout_pct = Set(rollout);
                    active.updated_by = Set(Some(actor_id));
                    active.updated_at = Set(now);
                    active
                        .update(&db)
                        .await
                        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
                } else {
                    atlas_flag_instance_enablement::ActiveModel {
                        id: Set(Uuid::new_v4()),
                        flag_key: Set(key.to_string()),
                        app_instance_id: Set(id),
                        effect: Set(effect.to_string()),
                        rollout_pct: Set(rollout),
                        updated_by: Set(Some(actor_id)),
                        created_at: Set(now),
                        updated_at: Set(now),
                    }
                    .insert(&db)
                    .await
                    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
                }

                write_audit(
                    &db,
                    flag.id,
                    &current_user.email,
                    &format!("Instance {} enablement set to {} for {}", effect, id, key),
                )
                .await?;
            }
        }
    }

    // Return refreshed list
    list_instance_feature_flags(State(db), Extension(current_user), Path(id)).await
}
