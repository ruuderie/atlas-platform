/// # Admin Campaigns — G-19 Go-to-Market Command Center
///
/// Routes (mounted via `routes_raw()` in routes.rs):
///
///   GET  /api/admin/campaigns                        → list all campaigns (cross-tenant)
///   GET  /api/admin/campaigns/:id                    → single campaign detail
///   POST /api/admin/campaigns                        → create a new campaign
///   PUT  /api/admin/campaigns/:id/status             → update status (activate/pause/complete)
///   GET  /api/admin/campaigns/:id/enrollments        → list enrollments for a campaign
///
use axum::{
    Json, Router,
    extract::{Path, State},
    routing::{get, post, put},
};
use sea_orm::{
    ActiveModelTrait, ColumnTrait, ConnectionTrait, DatabaseConnection, EntityTrait,
    FromQueryResult, QueryFilter, QueryOrder, Set, Statement,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::entities::atlas_campaign;
use crate::entities::atlas_campaign_enrollment;

// ── Response DTOs ─────────────────────────────────────────────────────────────

/// Flat campaign summary returned by the list endpoint.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CampaignDto {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub name: String,
    pub global_name: String,
    pub campaign_type: String,
    pub status: String,
    pub goal_type: Option<String>,
    pub budget_cents: Option<i64>,
    pub spent_cents: i64,
    pub total_contacts: i32,
    pub total_opens: i32,
    pub total_clicks: i32,
    pub total_replies: i32,
    pub total_conversions: i32,
    pub utm_source: Option<String>,
    pub utm_medium: Option<String>,
    pub utm_campaign: Option<String>,
    pub starts_at: Option<String>,
    pub ends_at: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

impl From<atlas_campaign::Model> for CampaignDto {
    fn from(m: atlas_campaign::Model) -> Self {
        Self {
            id: m.id,
            tenant_id: m.tenant_id,
            name: m.name,
            global_name: m.global_name,
            campaign_type: m.campaign_type,
            status: m.status,
            goal_type: m.goal_type,
            budget_cents: m.budget_cents,
            spent_cents: m.spent_cents,
            total_contacts: m.total_contacts,
            total_opens: m.total_opens,
            total_clicks: m.total_clicks,
            total_replies: m.total_replies,
            total_conversions: m.total_conversions,
            utm_source: m.utm_source,
            utm_medium: m.utm_medium,
            utm_campaign: m.utm_campaign,
            starts_at: m.starts_at.map(|d| d.to_rfc3339()),
            ends_at: m.ends_at.map(|d| d.to_rfc3339()),
            created_at: m.created_at.to_rfc3339(),
            updated_at: m.updated_at.to_rfc3339(),
        }
    }
}

/// Enrollment summary returned by the enrollments endpoint.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct EnrollmentDto {
    pub id: Uuid,
    pub campaign_id: Uuid,
    pub contact_email: Option<String>,
    pub contact_name: Option<String>,
    pub status: String,
    pub current_step: i32,
    pub exit_reason: Option<String>,
    pub converted_at: Option<String>,
    pub enrolled_at: String,
}

impl From<atlas_campaign_enrollment::Model> for EnrollmentDto {
    fn from(m: atlas_campaign_enrollment::Model) -> Self {
        Self {
            id: m.id,
            campaign_id: m.campaign_id,
            contact_email: m.contact_email,
            contact_name: m.contact_name,
            status: m.status,
            current_step: m.current_step,
            exit_reason: m.exit_reason,
            converted_at: m.converted_at.map(|d| d.to_rfc3339()),
            enrolled_at: m.enrolled_at.to_rfc3339(),
        }
    }
}

// ── Request payloads ──────────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct CreateCampaignPayload {
    pub name: String,
    pub campaign_type: String,
    pub tenant_id: Uuid,
    /// App slug for `global_name` (defaults to `"folio"`).
    #[serde(default)]
    pub app_id: Option<String>,
    pub goal_type: Option<String>,
    pub budget_cents: Option<i64>,
    pub utm_source: Option<String>,
    pub utm_medium: Option<String>,
    pub utm_campaign: Option<String>,
    pub starts_at: Option<String>,
    pub ends_at: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateStatusPayload {
    pub status: String,
}

// ── Handlers ──────────────────────────────────────────────────────────────────

/// GET /api/admin/campaigns — list all campaigns across all tenants, newest first.
pub async fn list_campaigns(
    State(db): State<DatabaseConnection>,
) -> Result<Json<Vec<CampaignDto>>, (axum::http::StatusCode, String)> {
    let campaigns = atlas_campaign::Entity::find()
        .order_by_desc(atlas_campaign::Column::CreatedAt)
        .all(&db)
        .await
        .map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(campaigns.into_iter().map(CampaignDto::from).collect()))
}

/// GET /api/admin/campaigns/:id — single campaign detail.
pub async fn get_campaign(
    State(db): State<DatabaseConnection>,
    Path(id): Path<Uuid>,
) -> Result<Json<CampaignDto>, (axum::http::StatusCode, String)> {
    let campaign = atlas_campaign::Entity::find_by_id(id)
        .one(&db)
        .await
        .map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .ok_or_else(|| {
            (
                axum::http::StatusCode::NOT_FOUND,
                "Campaign not found".to_string(),
            )
        })?;

    Ok(Json(CampaignDto::from(campaign)))
}

/// POST /api/admin/campaigns — create a new campaign record.
pub async fn create_campaign(
    State(db): State<DatabaseConnection>,
    Json(payload): Json<CreateCampaignPayload>,
) -> Result<Json<CampaignDto>, (axum::http::StatusCode, String)> {
    use chrono::Utc;
    use sea_orm::ActiveValue::NotSet;

    let id = Uuid::new_v4();
    let now = Utc::now();

    let starts_at = payload
        .starts_at
        .as_deref()
        .and_then(|s| s.parse::<chrono::DateTime<Utc>>().ok());
    let ends_at = payload
        .ends_at
        .as_deref()
        .and_then(|s| s.parse::<chrono::DateTime<Utc>>().ok());

    let app_id = payload.app_id.as_deref().unwrap_or("folio");
    let global_name = crate::types::pm::campaign_global_name(app_id, &payload.name);

    // Reject duplicate global_name
    if atlas_campaign::Entity::find()
        .filter(atlas_campaign::Column::GlobalName.eq(&global_name))
        .one(&db)
        .await
        .map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .is_some()
    {
        return Err((
            axum::http::StatusCode::CONFLICT,
            format!("global_name '{global_name}' already exists"),
        ));
    }

    let active = atlas_campaign::ActiveModel {
        id: Set(id),
        tenant_id: Set(payload.tenant_id),
        parent_campaign_id: NotSet,
        name: Set(payload.name),
        global_name: Set(global_name),
        campaign_type: Set(payload.campaign_type),
        status: Set("draft".to_string()),
        audience_segment_id: NotSet,
        audience_filter: NotSet,
        goal_type: Set(payload.goal_type),
        goal_entity_type: NotSet,
        target_conversion_count: NotSet,
        budget_cents: Set(payload.budget_cents),
        currency: Set(Some("USD".to_string())),
        spent_cents: Set(0),
        attribution_window_days: Set(30),
        external_campaign_id: NotSet,
        integration_id: NotSet,
        subject_entity_type: NotSet,
        subject_entity_id: NotSet,
        starts_at: Set(starts_at),
        ends_at: Set(ends_at),
        utm_source: Set(payload.utm_source),
        utm_medium: Set(payload.utm_medium),
        utm_campaign: Set(payload.utm_campaign),
        total_contacts: Set(0),
        total_opens: Set(0),
        total_clicks: Set(0),
        total_replies: Set(0),
        total_conversions: Set(0),
        created_by_user_id: NotSet,
        created_at: Set(now),
        updated_at: Set(now),
    };

    let inserted = active
        .insert(&db)
        .await
        .map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(CampaignDto::from(inserted)))
}

/// PUT /api/admin/campaigns/:id/status — update campaign status (active/paused/completed/draft).
pub async fn update_campaign_status(
    State(db): State<DatabaseConnection>,
    Path(id): Path<Uuid>,
    Json(payload): Json<UpdateStatusPayload>,
) -> Result<Json<CampaignDto>, (axum::http::StatusCode, String)> {
    use chrono::Utc;

    let campaign = atlas_campaign::Entity::find_by_id(id)
        .one(&db)
        .await
        .map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .ok_or_else(|| {
            (
                axum::http::StatusCode::NOT_FOUND,
                "Campaign not found".to_string(),
            )
        })?;

    let mut active: atlas_campaign::ActiveModel = campaign.into();
    active.status = Set(payload.status);
    active.updated_at = Set(Utc::now());

    let updated = active
        .update(&db)
        .await
        .map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(CampaignDto::from(updated)))
}

/// GET /api/admin/campaigns/:id/enrollments — list enrollments for a campaign.
pub async fn list_campaign_enrollments(
    State(db): State<DatabaseConnection>,
    Path(id): Path<Uuid>,
) -> Result<Json<Vec<EnrollmentDto>>, (axum::http::StatusCode, String)> {
    let enrollments = atlas_campaign_enrollment::Entity::find()
        .filter(atlas_campaign_enrollment::Column::CampaignId.eq(id))
        .order_by_desc(atlas_campaign_enrollment::Column::EnrolledAt)
        .all(&db)
        .await
        .map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(
        enrollments.into_iter().map(EnrollmentDto::from).collect(),
    ))
}

/// One row in the Friends & Family (or any UTM-tagged) referrer leaderboard.
#[derive(Debug, Serialize, Deserialize, Clone, FromQueryResult)]
pub struct ReferrerLeaderboardRow {
    pub referred_by: String,
    pub signup_count: i64,
    pub latest_signup_at: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ReferrerLeaderboardResponse {
    pub campaign_id: Uuid,
    pub utm_campaign: Option<String>,
    pub total_attributed: i64,
    pub referrers: Vec<ReferrerLeaderboardRow>,
}

/// GET /api/admin/campaigns/:id/referrers — leaderboard of who referred the most.
pub async fn list_campaign_referrers(
    State(db): State<DatabaseConnection>,
    Path(id): Path<Uuid>,
) -> Result<Json<ReferrerLeaderboardResponse>, (axum::http::StatusCode, String)> {
    let campaign = atlas_campaign::Entity::find_by_id(id)
        .one(&db)
        .await
        .map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .ok_or_else(|| {
            (
                axum::http::StatusCode::NOT_FOUND,
                "Campaign not found".to_string(),
            )
        })?;

    let utm = campaign
        .utm_campaign
        .clone()
        .filter(|s| !s.trim().is_empty())
        .ok_or_else(|| {
            (
                axum::http::StatusCode::UNPROCESSABLE_ENTITY,
                "Campaign has no utm_campaign — cannot build referrer leaderboard".to_string(),
            )
        })?;

    let rows = ReferrerLeaderboardRow::find_by_statement(Statement::from_sql_and_values(
        db.get_database_backend(),
        r#"
        SELECT
            COALESCE(
                NULLIF(TRIM(lead_metadata->>'referred_by'), ''),
                NULLIF(TRIM(lead_metadata->>'utm_content'), ''),
                '(unknown)'
            ) AS referred_by,
            COUNT(*)::bigint AS signup_count,
            MAX(created_at)::text AS latest_signup_at
        FROM atlas_lead
        WHERE lead_metadata->>'utm_campaign' = $1
          AND (
                NULLIF(TRIM(lead_metadata->>'referred_by'), '') IS NOT NULL
             OR NULLIF(TRIM(lead_metadata->>'utm_content'), '') IS NOT NULL
          )
        GROUP BY 1
        ORDER BY signup_count DESC, latest_signup_at DESC NULLS LAST
        "#,
        [utm.clone().into()],
    ))
    .all(&db)
    .await
    .map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let total_attributed: i64 = rows.iter().map(|r| r.signup_count).sum();

    Ok(Json(ReferrerLeaderboardResponse {
        campaign_id: id,
        utm_campaign: Some(utm),
        total_attributed,
        referrers: rows,
    }))
}

// ── Route constructor ─────────────────────────────────────────────────────────

pub fn routes_raw() -> Router<DatabaseConnection> {
    Router::new()
        .route(
            "/api/admin/campaigns",
            get(list_campaigns).post(create_campaign),
        )
        .route("/api/admin/campaigns/{id}", get(get_campaign))
        .route(
            "/api/admin/campaigns/{id}/status",
            put(update_campaign_status),
        )
        .route(
            "/api/admin/campaigns/{id}/enrollments",
            get(list_campaign_enrollments),
        )
        .route(
            "/api/admin/campaigns/{id}/referrers",
            get(list_campaign_referrers),
        )
}
