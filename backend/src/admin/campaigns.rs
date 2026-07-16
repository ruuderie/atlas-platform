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
    extract::{Path, Query, State},
    routing::{get, post, put},
};
use sea_orm::{
    ActiveModelTrait, ColumnTrait, ConnectionTrait, DatabaseConnection, EntityTrait,
    FromQueryResult, QueryFilter, QueryOrder, Set, Statement,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::entities::{atlas_campaign, atlas_campaign_enrollment, atlas_campaign_mail_drop, atlas_campaign_offer_code};
use crate::services::pm::{attribution::AttributionService, campaign_dm};

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

#[derive(Debug, Deserialize)]
pub struct RecordSpendPayload {
    pub cents: i64,
    pub source: String,
    pub external_ref: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MailDropDto {
    pub id: Uuid,
    pub campaign_id: Uuid,
    pub drop_name: String,
    pub creative_variant: Option<String>,
    pub utm_content: Option<String>,
    pub piece_count: i32,
    pub unit_cost_cents: Option<i64>,
    pub provider_job_id: Option<String>,
    pub status: String,
    pub mailed_at: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

impl From<atlas_campaign_mail_drop::Model> for MailDropDto {
    fn from(m: atlas_campaign_mail_drop::Model) -> Self {
        Self {
            id: m.id,
            campaign_id: m.campaign_id,
            drop_name: m.drop_name,
            creative_variant: m.creative_variant,
            utm_content: m.utm_content,
            piece_count: m.piece_count,
            unit_cost_cents: m.unit_cost_cents,
            provider_job_id: m.provider_job_id,
            status: m.status,
            mailed_at: m.mailed_at.map(|d| d.to_rfc3339()),
            created_at: m.created_at.to_rfc3339(),
            updated_at: m.updated_at.to_rfc3339(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct OfferCodeDto {
    pub id: Uuid,
    pub campaign_id: Uuid,
    pub mail_drop_id: Option<Uuid>,
    pub code: String,
    pub is_active: bool,
    pub redemption_count: i32,
    pub created_at: String,
}

impl From<atlas_campaign_offer_code::Model> for OfferCodeDto {
    fn from(m: atlas_campaign_offer_code::Model) -> Self {
        Self {
            id: m.id,
            campaign_id: m.campaign_id,
            mail_drop_id: m.mail_drop_id,
            code: m.code,
            is_active: m.is_active,
            redemption_count: m.redemption_count,
            created_at: m.created_at.to_rfc3339(),
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct QrQuery {
    #[serde(default)]
    pub size: Option<u32>,
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

/// POST /api/admin/campaigns/:id/spend — record manual spend for a campaign.
pub async fn record_campaign_spend(
    State(db): State<DatabaseConnection>,
    Path(id): Path<Uuid>,
    Json(payload): Json<RecordSpendPayload>,
) -> Result<Json<CampaignDto>, (axum::http::StatusCode, String)> {
    use crate::services::pm::campaign::CampaignService;
    
    // Find any tenant that owns this campaign (admin can record spend cross-tenant)
    let campaign = atlas_campaign::Entity::find_by_id(id)
        .one(&db)
        .await
        .map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .ok_or_else(|| (axum::http::StatusCode::NOT_FOUND, "Campaign not found".to_string()))?;

    let updated = CampaignService::record_spend(
        &db,
        campaign.tenant_id,
        id,
        payload.cents,
        &payload.source,
        payload.external_ref.as_deref(),
    )
    .await
    .map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(CampaignDto::from(updated)))
}

/// GET /api/admin/campaigns/:id/mail-drops — list mail drops for a campaign.
pub async fn list_campaign_mail_drops(
    State(db): State<DatabaseConnection>,
    Path(id): Path<Uuid>,
) -> Result<Json<Vec<MailDropDto>>, (axum::http::StatusCode, String)> {
    // Find any tenant that owns this campaign (admin can list cross-tenant)
    let campaign = atlas_campaign::Entity::find_by_id(id)
        .one(&db)
        .await
        .map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .ok_or_else(|| (axum::http::StatusCode::NOT_FOUND, "Campaign not found".to_string()))?;

    let drops = campaign_dm::list_mail_drops(&db, campaign.tenant_id, id)
        .await
        .map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(drops.into_iter().map(MailDropDto::from).collect()))
}

/// POST /api/admin/campaigns/:id/mail-drops — create a mail drop for a campaign.
pub async fn create_campaign_mail_drop(
    State(db): State<DatabaseConnection>,
    Path(id): Path<Uuid>,
    Json(payload): Json<campaign_dm::CreateMailDropPayload>,
) -> Result<Json<MailDropDto>, (axum::http::StatusCode, String)> {
    // Find any tenant that owns this campaign (admin can create cross-tenant)
    let campaign = atlas_campaign::Entity::find_by_id(id)
        .one(&db)
        .await
        .map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .ok_or_else(|| (axum::http::StatusCode::NOT_FOUND, "Campaign not found".to_string()))?;

    let drop = campaign_dm::create_mail_drop(&db, campaign.tenant_id, id, payload)
        .await
        .map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(MailDropDto::from(drop)))
}

/// GET /api/admin/campaigns/:id/offer-codes — list offer codes for a campaign.
pub async fn list_campaign_offer_codes(
    State(db): State<DatabaseConnection>,
    Path(id): Path<Uuid>,
) -> Result<Json<Vec<OfferCodeDto>>, (axum::http::StatusCode, String)> {
    // Find any tenant that owns this campaign (admin can list cross-tenant)
    let campaign = atlas_campaign::Entity::find_by_id(id)
        .one(&db)
        .await
        .map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .ok_or_else(|| (axum::http::StatusCode::NOT_FOUND, "Campaign not found".to_string()))?;

    let codes = campaign_dm::list_offer_codes(&db, campaign.tenant_id, id)
        .await
        .map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(codes.into_iter().map(OfferCodeDto::from).collect()))
}

/// POST /api/admin/campaigns/:id/offer-codes — create an offer code for a campaign.
pub async fn create_campaign_offer_code(
    State(db): State<DatabaseConnection>,
    Path(id): Path<Uuid>,
    Json(payload): Json<campaign_dm::CreateOfferCodePayload>,
) -> Result<Json<OfferCodeDto>, (axum::http::StatusCode, String)> {
    // Find any tenant that owns this campaign (admin can create cross-tenant)
    let campaign = atlas_campaign::Entity::find_by_id(id)
        .one(&db)
        .await
        .map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .ok_or_else(|| (axum::http::StatusCode::NOT_FOUND, "Campaign not found".to_string()))?;

    let code = campaign_dm::create_offer_code(&db, campaign.tenant_id, id, payload)
        .await
        .map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(OfferCodeDto::from(code)))
}

/// GET /api/admin/campaigns/:id/attribution — list attribution touchpoints for a campaign.
pub async fn get_campaign_attribution(
    State(db): State<DatabaseConnection>,
    Path(id): Path<Uuid>,
) -> Result<Json<Vec<crate::entities::atlas_attribution_touchpoint::Model>>, (axum::http::StatusCode, String)> {
    // Find any tenant that owns this campaign (admin can view cross-tenant)
    let campaign = atlas_campaign::Entity::find_by_id(id)
        .one(&db)
        .await
        .map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .ok_or_else(|| (axum::http::StatusCode::NOT_FOUND, "Campaign not found".to_string()))?;

    let touchpoints = AttributionService::get_campaign_touchpoints(&db, campaign.tenant_id, id)
        .await
        .map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(touchpoints))
}

/// GET /api/admin/campaigns/:id/qr — generate QR code PNG for campaign landing page.
pub async fn get_campaign_qr_png(
    State(db): State<DatabaseConnection>,
    Path(id): Path<Uuid>,
    Query(_q): Query<QrQuery>,
) -> Result<axum::response::Response, (axum::http::StatusCode, String)> {
    // Find campaign and get UTM parameters for landing page URL
    let campaign = atlas_campaign::Entity::find_by_id(id)
        .one(&db)
        .await
        .map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .ok_or_else(|| (axum::http::StatusCode::NOT_FOUND, "Campaign not found".to_string()))?;

    // Build landing page URL with UTM parameters
    let base = std::env::var("PUBLIC_BASE_URL")
        .or_else(|_| std::env::var("FOLIO_PUBLIC_URL"))
        .unwrap_or_else(|_| "https://folio1.atlas.oply.co".to_string());
    
    let mut url = format!("{}/products/folio", base.trim_end_matches('/'));
    let mut params = Vec::new();
    
    if let Some(ref source) = campaign.utm_source {
        params.push(format!("utm_source={}", urlencoding::encode(source)));
    }
    if let Some(ref medium) = campaign.utm_medium {
        params.push(format!("utm_medium={}", urlencoding::encode(medium)));
    }
    if let Some(ref campaign_name) = campaign.utm_campaign {
        params.push(format!("utm_campaign={}", urlencoding::encode(campaign_name)));
    }
    
    if !params.is_empty() {
        url.push('?');
        url.push_str(&params.join("&"));
    }

    // Generate QR code
    let code = qrcode::QrCode::new(url.as_bytes())
        .map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    
    let img = code
        .render::<image::Luma<u8>>()
        .min_dimensions(512, 512)
        .build();

    // Encode as PNG
    let mut buf = Vec::new();
    {
        use image::ImageEncoder;
        let encoder = image::codecs::png::PngEncoder::new(&mut buf);
        encoder
            .write_image(
                img.as_raw(),
                img.width(),
                img.height(),
                image::ExtendedColorType::L8,
            )
            .map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    }

    let filename = format!("campaign-{}.png", campaign.global_name);
    Ok(axum::response::Response::builder()
        .status(axum::http::StatusCode::OK)
        .header("Content-Type", "image/png")
        .header(
            "Content-Disposition",
            format!("attachment; filename=\"{}\"", filename),
        )
        .body(axum::body::Body::from(buf))
        .unwrap())
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
            "/api/admin/campaigns/{id}/spend",
            post(record_campaign_spend),
        )
        .route(
            "/api/admin/campaigns/{id}/enrollments",
            get(list_campaign_enrollments),
        )
        .route(
            "/api/admin/campaigns/{id}/referrers",
            get(list_campaign_referrers),
        )
        .route(
            "/api/admin/campaigns/{id}/mail-drops",
            get(list_campaign_mail_drops).post(create_campaign_mail_drop),
        )
        .route(
            "/api/admin/campaigns/{id}/offer-codes",
            get(list_campaign_offer_codes).post(create_campaign_offer_code),
        )
        .route(
            "/api/admin/campaigns/{id}/attribution",
            get(get_campaign_attribution),
        )
        .route(
            "/api/admin/campaigns/{id}/qr",
            get(get_campaign_qr_png),
        )
}
