use axum::{
    extract::{Path, State, Query},
    http::StatusCode,
    Json,
    routing::{get, post},
    Router,
};
use sea_orm::{DatabaseConnection, EntityTrait, QueryFilter, ColumnTrait, ActiveModelTrait, Set};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::Utc;
use std::collections::HashMap;

use crate::entities::onboarding_progress;
use crate::atlas_apps::get_active_apps;
use crate::entities::app_instance;

// ──────────────────────────────────────────────────────────────────────────────
// RESPONSE TYPES
// ──────────────────────────────────────────────────────────────────────────────

/// The status of a single onboarding step as returned to the frontend wizard.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct OnboardingStepStatus {
    pub id: String,
    pub title: String,
    pub description: String,
    pub is_required: bool,
    pub is_complete: bool,
    pub is_skipped: bool,
}

/// Full response for an app instance's onboarding state.
#[derive(Serialize, Deserialize, Debug)]
pub struct OnboardingStatusResponse {
    pub app_instance_id: Uuid,
    pub tenant_id: Uuid,
    pub app_type: String,
    pub steps: Vec<OnboardingStepStatus>,
    /// True if all required steps are complete
    pub is_ready: bool,
    /// Set if the platform admin dismissed the full-page takeover
    pub dismissed_at: Option<chrono::DateTime<Utc>>,
}

// ──────────────────────────────────────────────────────────────────────────────
// REQUEST TYPES
// ──────────────────────────────────────────────────────────────────────────────

#[derive(Deserialize)]
pub struct TokenQuery {
    pub token: Option<String>,
}

#[derive(Deserialize)]
pub struct DismissPayload {
    /// Optional JSON metadata to persist alongside the dismissal
    pub metadata: Option<serde_json::Value>,
}

// ──────────────────────────────────────────────────────────────────────────────
// ROUTES
// ──────────────────────────────────────────────────────────────────────────────

/// Authenticated routes (platform admin)
pub fn authenticated_routes(db: DatabaseConnection) -> Router<DatabaseConnection> {
    Router::new()
        // Get onboarding status for an app instance
        .route(
            "/api/onboarding/:app_instance_id",
            get(get_onboarding_status),
        )
        // Mark a step complete (Custom steps only — data steps resolve automatically)
        .route(
            "/api/onboarding/:app_instance_id/complete/:step_id",
            post(complete_step),
        )
        // Skip an optional step
        .route(
            "/api/onboarding/:app_instance_id/skip/:step_id",
            post(skip_step),
        )
        // Dismiss the full-page takeover ("I'll do this later")
        .route(
            "/api/onboarding/:app_instance_id/dismiss",
            post(dismiss_wizard),
        )
        .with_state(db)
}

/// Public token-gated routes (tenant self-service via magic link)
pub fn public_routes(db: DatabaseConnection) -> Router<DatabaseConnection> {
    Router::new()
        // Get status — tenant reads via ?token=
        .route(
            "/onboarding/status/:app_instance_id",
            get(get_onboarding_status_public),
        )
        // Complete a step — tenant submits via ?token=
        .route(
            "/onboarding/step/:app_instance_id/:step_id",
            post(complete_step_public),
        )
        .with_state(db)
}

// ──────────────────────────────────────────────────────────────────────────────
// HELPERS
// ──────────────────────────────────────────────────────────────────────────────

/// Resolves the AppInstance entity and constructs the full OnboardingStatusResponse.
async fn build_status_response(
    db: &DatabaseConnection,
    app_instance_id: Uuid,
) -> Result<OnboardingStatusResponse, StatusCode> {
    // 1. Load the AppInstance to know app_type and tenant_id
    let instance = app_instance::Entity::find_by_id(app_instance_id)
        .one(db)
        .await
        .map_err(|e| {
            tracing::error!("DB error fetching app_instance: {:?}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?
        .ok_or(StatusCode::NOT_FOUND)?;

    let tenant_id = instance.tenant_id;
    let app_type = instance.app_type.clone();

    // 2. Find the matching AtlasApp implementation
    let apps = get_active_apps();
    let app = apps
        .iter()
        .find(|a| a.app_id() == app_type.as_str())
        .ok_or_else(|| {
            tracing::warn!("No AtlasApp registered for type: {}", app_type);
            StatusCode::UNPROCESSABLE_ENTITY
        })?;

    // 3. Get declared steps from the app
    let steps = app.onboarding_steps();
    if steps.is_empty() {
        return Ok(OnboardingStatusResponse {
            app_instance_id,
            tenant_id,
            app_type,
            steps: vec![],
            is_ready: true,
            dismissed_at: None,
        });
    }

    // 4. Evaluate which required steps are incomplete (server-side data checks)
    let incomplete_ids = app
        .onboarding_readiness(db, tenant_id, app_instance_id)
        .await
        .map_err(|e| {
            tracing::error!("onboarding_readiness error: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    // 5. Load explicit override records (skips, custom completions, dismissal)
    let progress_records = onboarding_progress::Entity::find()
        .filter(onboarding_progress::Column::AppInstanceId.eq(app_instance_id))
        .all(db)
        .await
        .map_err(|e| {
            tracing::error!("DB error fetching onboarding_progress: {:?}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    let progress_map: HashMap<String, onboarding_progress::Model> = progress_records
        .into_iter()
        .map(|r| (r.step_id.clone(), r))
        .collect();

    // 6. Check for a wizard-level dismissal record
    let dismissed_at = progress_map
        .get("__wizard__")
        .and_then(|r| r.dismissed_at);

    // 7. Merge declared steps with readiness and override data
    let step_statuses: Vec<OnboardingStepStatus> = steps
        .iter()
        .map(|step| {
            let data_complete = !incomplete_ids.contains(&step.id);
            let override_record = progress_map.get(&step.id);
            let explicitly_complete =
                override_record.map(|r| r.completed_at.is_some()).unwrap_or(false);
            let skipped = override_record.map(|r| r.skipped).unwrap_or(false);

            OnboardingStepStatus {
                id: step.id.clone(),
                title: step.title.clone(),
                description: step.description.clone(),
                is_required: step.is_required,
                is_complete: data_complete || explicitly_complete,
                is_skipped: skipped,
            }
        })
        .collect();

    let is_ready = step_statuses
        .iter()
        .filter(|s| s.is_required)
        .all(|s| s.is_complete);

    Ok(OnboardingStatusResponse {
        app_instance_id,
        tenant_id,
        app_type,
        steps: step_statuses,
        is_ready,
        dismissed_at,
    })
}

// ──────────────────────────────────────────────────────────────────────────────
// AUTHENTICATED HANDLERS
// ──────────────────────────────────────────────────────────────────────────────

pub async fn get_onboarding_status(
    Path(app_instance_id): Path<Uuid>,
    State(db): State<DatabaseConnection>,
) -> Result<(StatusCode, Json<OnboardingStatusResponse>), StatusCode> {
    let response = build_status_response(&db, app_instance_id).await?;
    Ok((StatusCode::OK, Json(response)))
}

pub async fn complete_step(
    Path((app_instance_id, step_id)): Path<(Uuid, String)>,
    State(db): State<DatabaseConnection>,
) -> Result<StatusCode, StatusCode> {
    let instance = app_instance::Entity::find_by_id(app_instance_id)
        .one(&db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;

    upsert_progress(&db, instance.tenant_id, app_instance_id, &step_id, |m| {
        m.completed_at = Set(Some(Utc::now()));
    })
    .await?;

    Ok(StatusCode::NO_CONTENT)
}

pub async fn skip_step(
    Path((app_instance_id, step_id)): Path<(Uuid, String)>,
    State(db): State<DatabaseConnection>,
) -> Result<StatusCode, StatusCode> {
    let instance = app_instance::Entity::find_by_id(app_instance_id)
        .one(&db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;

    upsert_progress(&db, instance.tenant_id, app_instance_id, &step_id, |m| {
        m.skipped = Set(true);
    })
    .await?;

    Ok(StatusCode::NO_CONTENT)
}

pub async fn dismiss_wizard(
    Path(app_instance_id): Path<Uuid>,
    State(db): State<DatabaseConnection>,
    Json(payload): Json<DismissPayload>,
) -> Result<StatusCode, StatusCode> {
    let instance = app_instance::Entity::find_by_id(app_instance_id)
        .one(&db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;

    let meta = payload.metadata;
    upsert_progress(&db, instance.tenant_id, app_instance_id, "__wizard__", move |m| {
        m.dismissed_at = Set(Some(Utc::now()));
        m.metadata = Set(meta);
    })
    .await?;

    Ok(StatusCode::NO_CONTENT)
}

// ──────────────────────────────────────────────────────────────────────────────
// PUBLIC TOKEN-GATED HANDLERS
// ──────────────────────────────────────────────────────────────────────────────

pub async fn get_onboarding_status_public(
    Path(app_instance_id): Path<Uuid>,
    Query(params): Query<TokenQuery>,
    State(db): State<DatabaseConnection>,
) -> Result<(StatusCode, Json<OnboardingStatusResponse>), StatusCode> {
    validate_setup_token(&db, app_instance_id, &params.token).await?;
    let response = build_status_response(&db, app_instance_id).await?;
    Ok((StatusCode::OK, Json(response)))
}

pub async fn complete_step_public(
    Path((app_instance_id, step_id)): Path<(Uuid, String)>,
    Query(params): Query<TokenQuery>,
    State(db): State<DatabaseConnection>,
) -> Result<StatusCode, StatusCode> {
    let instance = validate_setup_token(&db, app_instance_id, &params.token).await?;

    upsert_progress(&db, instance.tenant_id, app_instance_id, &step_id, |m| {
        m.completed_at = Set(Some(Utc::now()));
    })
    .await?;

    Ok(StatusCode::NO_CONTENT)
}

// ──────────────────────────────────────────────────────────────────────────────
// INTERNAL UTILITIES
// ──────────────────────────────────────────────────────────────────────────────

/// Validates that the provided setup_token matches the one stored in TenantSettings
/// for this app_instance. Returns the AppInstance model on success.
async fn validate_setup_token(
    db: &DatabaseConnection,
    app_instance_id: Uuid,
    token: &Option<String>,
) -> Result<app_instance::Model, StatusCode> {
    use crate::entities::tenant_setting;

    let token = token.as_deref().ok_or(StatusCode::UNAUTHORIZED)?;

    let instance = app_instance::Entity::find_by_id(app_instance_id)
        .one(db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;

    // Validate against the setup_token stored as a TenantSetting
    let stored = tenant_setting::Entity::find()
        .filter(tenant_setting::Column::TenantId.eq(instance.tenant_id))
        .filter(tenant_setting::Column::Key.eq("setup_token"))
        .one(db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::UNAUTHORIZED)?;

    if stored.value != token {
        return Err(StatusCode::UNAUTHORIZED);
    }

    Ok(instance)
}

/// Upserts an onboarding_progress row. Creates a new one if the (app_instance, step_id) pair
/// doesn't yet exist; updates the existing one via the provided mutation closure.
async fn upsert_progress<F>(
    db: &DatabaseConnection,
    tenant_id: Uuid,
    app_instance_id: Uuid,
    step_id: &str,
    mutate: F,
) -> Result<(), StatusCode>
where
    F: FnOnce(&mut onboarding_progress::ActiveModel),
{
    let existing = onboarding_progress::Entity::find()
        .filter(onboarding_progress::Column::AppInstanceId.eq(app_instance_id))
        .filter(onboarding_progress::Column::StepId.eq(step_id))
        .one(db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    if let Some(record) = existing {
        let mut active: onboarding_progress::ActiveModel = record.into();
        active.updated_at = Set(Utc::now());
        mutate(&mut active);
        active.update(db).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    } else {
        let mut new_record = onboarding_progress::ActiveModel {
            id: Set(Uuid::new_v4()),
            tenant_id: Set(tenant_id),
            app_instance_id: Set(app_instance_id),
            step_id: Set(step_id.to_string()),
            completed_at: Set(None),
            skipped: Set(false),
            dismissed_at: Set(None),
            metadata: Set(None),
            created_at: Set(Utc::now()),
            updated_at: Set(Utc::now()),
        };
        mutate(&mut new_record);
        new_record.insert(db).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    }

    Ok(())
}
