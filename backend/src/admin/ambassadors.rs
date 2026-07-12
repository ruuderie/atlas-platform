//! # G-37 Admin Ambassadors
//!
//! Growth partners (referral / influencer / affiliate / recruiter) with campaign
//! attach, QR PNG export by audience, and card-pack fulfillment stubs.

use axum::{
    Json, Router,
    body::Body,
    extract::{Path, Query, State},
    http::{header, StatusCode},
    response::Response,
    routing::{get, post},
};
use chrono::Utc;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, QueryOrder, Set,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::entities::{atlas_ambassador, atlas_ambassador_campaign, atlas_campaign};
use crate::handlers::folio::referrals::{deliver_referral_invite, public_base_for_app, refer_url_for};
use crate::types::pm::{
    AmbassadorFulfillmentKind, AmbassadorFulfillmentStatus, AmbassadorPartnerType,
    AmbassadorStatus, ReferAudience, ReferralInviteChannel,
};

const SENTINEL_TENANT: Uuid = Uuid::nil();

fn folio_public_base() -> String {
    std::env::var("FOLIO_PUBLIC_URL")
        .or_else(|_| std::env::var("PUBLIC_BASE_URL"))
        .unwrap_or_else(|_| "https://folio1.atlas.oply.co".to_string())
}

fn slugify_code(raw: &str) -> String {
    let slug: String = raw
        .trim()
        .to_lowercase()
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() {
                c
            } else if c.is_whitespace() || c == '-' || c == '_' {
                '-'
            } else {
                '\0'
            }
        })
        .filter(|c| *c != '\0')
        .collect();
    slug.split('-')
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join("-")
        .chars()
        .take(64)
        .collect()
}

#[derive(Debug, Serialize, Clone)]
pub struct AmbassadorDto {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub code: String,
    pub display_name: String,
    pub partner_type: String,
    pub status: String,
    pub notes: Option<String>,
    pub campaign_ids: Vec<Uuid>,
    pub fulfillment_requests: serde_json::Value,
    pub landlord_url: String,
    pub vendor_url: String,
    /// Unified invitee landing (preferred).
    pub refer_url: String,
    pub created_at: String,
    pub updated_at: String,
}

impl AmbassadorDto {
    fn from_model(m: atlas_ambassador::Model, campaign_ids: Vec<Uuid>) -> Self {
        let base = folio_public_base().trim_end_matches('/').to_string();
        let refer_url = format!("{base}/refer/{}", m.code);
        Self {
            id: m.id,
            tenant_id: m.tenant_id,
            landlord_url: refer_url.clone(),
            vendor_url: refer_url.clone(),
            refer_url,
            code: m.code,
            display_name: m.display_name,
            partner_type: m.partner_type,
            status: m.status,
            notes: m.notes,
            campaign_ids,
            fulfillment_requests: m.fulfillment_requests,
            created_at: m.created_at.to_rfc3339(),
            updated_at: m.updated_at.to_rfc3339(),
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct CreateAmbassadorPayload {
    pub code: String,
    pub display_name: String,
    pub partner_type: AmbassadorPartnerType,
    #[serde(default)]
    pub notes: Option<String>,
    #[serde(default)]
    pub campaign_ids: Vec<Uuid>,
    #[serde(default)]
    pub tenant_id: Option<Uuid>,
}

#[derive(Debug, Deserialize)]
pub struct AttachCampaignsPayload {
    pub campaign_ids: Vec<Uuid>,
}

#[derive(Debug, Deserialize)]
pub struct QrQuery {
    pub audience: ReferAudience,
}

#[derive(Debug, Deserialize)]
pub struct CreateFulfillmentPayload {
    pub kind: AmbassadorFulfillmentKind,
    #[serde(default = "default_qty")]
    pub landlord_qty: i32,
    #[serde(default = "default_qty")]
    pub vendor_qty: i32,
    #[serde(default)]
    pub ship_to: Option<serde_json::Value>,
}

fn default_qty() -> i32 {
    25
}

async fn load_campaign_ids(
    db: &DatabaseConnection,
    ambassador_id: Uuid,
) -> Result<Vec<Uuid>, (StatusCode, String)> {
    let rows = atlas_ambassador_campaign::Entity::find()
        .filter(atlas_ambassador_campaign::Column::AmbassadorId.eq(ambassador_id))
        .all(db)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(rows.into_iter().map(|r| r.campaign_id).collect())
}

async fn dto_for(
    db: &DatabaseConnection,
    m: atlas_ambassador::Model,
) -> Result<AmbassadorDto, (StatusCode, String)> {
    let ids = load_campaign_ids(db, m.id).await?;
    Ok(AmbassadorDto::from_model(m, ids))
}

/// GET /api/admin/ambassadors
pub async fn list_ambassadors(
    State(db): State<DatabaseConnection>,
) -> Result<Json<Vec<AmbassadorDto>>, (StatusCode, String)> {
    let rows = atlas_ambassador::Entity::find()
        .order_by_desc(atlas_ambassador::Column::CreatedAt)
        .all(&db)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    let mut out = Vec::with_capacity(rows.len());
    for m in rows {
        out.push(dto_for(&db, m).await?);
    }
    Ok(Json(out))
}

/// GET /api/admin/ambassadors/:id
pub async fn get_ambassador(
    State(db): State<DatabaseConnection>,
    Path(id): Path<Uuid>,
) -> Result<Json<AmbassadorDto>, (StatusCode, String)> {
    let m = atlas_ambassador::Entity::find_by_id(id)
        .one(&db)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .ok_or((StatusCode::NOT_FOUND, "Ambassador not found".into()))?;
    Ok(Json(dto_for(&db, m).await?))
}

/// POST /api/admin/ambassadors
pub async fn create_ambassador(
    State(db): State<DatabaseConnection>,
    Json(payload): Json<CreateAmbassadorPayload>,
) -> Result<(StatusCode, Json<AmbassadorDto>), (StatusCode, String)> {
    let code = slugify_code(&payload.code);
    if code.is_empty() {
        return Err((StatusCode::BAD_REQUEST, "code is required".into()));
    }
    let tenant_id = payload.tenant_id.unwrap_or(SENTINEL_TENANT);
    let now = Utc::now();
    let id = Uuid::new_v4();

    if atlas_ambassador::Entity::find()
        .filter(atlas_ambassador::Column::TenantId.eq(tenant_id))
        .filter(atlas_ambassador::Column::Code.eq(&code))
        .one(&db)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .is_some()
    {
        return Err((
            StatusCode::CONFLICT,
            format!("ambassador code '{code}' already exists"),
        ));
    }

    let active = atlas_ambassador::ActiveModel {
        id: Set(id),
        tenant_id: Set(tenant_id),
        code: Set(code),
        display_name: Set(payload.display_name.trim().to_string()),
        partner_type: Set(payload.partner_type.to_string()),
        status: Set(AmbassadorStatus::Active.to_string()),
        account_id: Set(None),
        contact_id: Set(None),
        notes: Set(payload.notes),
        channels: Set(None),
        fulfillment_requests: Set(serde_json::json!([])),
        created_by_user_id: Set(None),
        created_at: Set(now),
        updated_at: Set(now),
    };

    let inserted = active
        .insert(&db)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let campaign_ids = if payload.campaign_ids.is_empty() {
        // Default: attach to both F&F campaigns when present
        let ff: Vec<_> = atlas_campaign::Entity::find()
            .filter(
                atlas_campaign::Column::GlobalName
                    .is_in(["folio_friends_family", "folio_friends_family_vendors"]),
            )
            .all(&db)
            .await
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
            .into_iter()
            .map(|c| c.id)
            .collect();
        ff
    } else {
        payload.campaign_ids
    };

    for cid in &campaign_ids {
        let link = atlas_ambassador_campaign::ActiveModel {
            ambassador_id: Set(id),
            campaign_id: Set(*cid),
            created_at: Set(now),
        };
        link.insert(&db)
            .await
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    }

    Ok((StatusCode::CREATED, Json(dto_for(&db, inserted).await?)))
}

/// POST /api/admin/ambassadors/:id/campaigns — replace/add campaign attaches
pub async fn attach_campaigns(
    State(db): State<DatabaseConnection>,
    Path(id): Path<Uuid>,
    Json(payload): Json<AttachCampaignsPayload>,
) -> Result<Json<AmbassadorDto>, (StatusCode, String)> {
    let m = atlas_ambassador::Entity::find_by_id(id)
        .one(&db)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .ok_or((StatusCode::NOT_FOUND, "Ambassador not found".into()))?;

    let now = Utc::now();
    for cid in payload.campaign_ids {
        let exists = atlas_ambassador_campaign::Entity::find()
            .filter(atlas_ambassador_campaign::Column::AmbassadorId.eq(id))
            .filter(atlas_ambassador_campaign::Column::CampaignId.eq(cid))
            .one(&db)
            .await
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
        if exists.is_none() {
            let link = atlas_ambassador_campaign::ActiveModel {
                ambassador_id: Set(id),
                campaign_id: Set(cid),
                created_at: Set(now),
            };
            link.insert(&db)
                .await
                .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
        }
    }

    Ok(Json(dto_for(&db, m).await?))
}

/// GET /api/admin/ambassadors/:id/qr?audience=landlord|vendor
pub async fn ambassador_qr_png(
    State(db): State<DatabaseConnection>,
    Path(id): Path<Uuid>,
    Query(q): Query<QrQuery>,
) -> Result<Response, (StatusCode, String)> {
    let m = atlas_ambassador::Entity::find_by_id(id)
        .one(&db)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .ok_or((StatusCode::NOT_FOUND, "Ambassador not found".into()))?;

    if m.status != AmbassadorStatus::Active.to_string() {
        return Err((StatusCode::BAD_REQUEST, "ambassador is not active".into()));
    }

    let base = folio_public_base().trim_end_matches('/').to_string();
    // Unified /refer/:code for all audiences (persona chosen on landing).
    let url = format!("{base}/refer/{}", m.code);
    let _ = (q.audience, ReferAudience::Landlord); // keep query for API compat

    let code = qrcode::QrCode::new(url.as_bytes())
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    let img = code
        .render::<image::Luma<u8>>()
        .min_dimensions(512, 512)
        .build();

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
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    }

    let filename = format!("ambassador-{}-{}.png", m.code, q.audience);
    Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "image/png")
        .header(
            header::CONTENT_DISPOSITION,
            format!("attachment; filename=\"{filename}\""),
        )
        .body(Body::from(buf))
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))
}

/// POST /api/admin/ambassadors/:id/fulfillments
pub async fn create_fulfillment(
    State(db): State<DatabaseConnection>,
    Path(id): Path<Uuid>,
    Json(payload): Json<CreateFulfillmentPayload>,
) -> Result<Json<AmbassadorDto>, (StatusCode, String)> {
    let m = atlas_ambassador::Entity::find_by_id(id)
        .one(&db)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .ok_or((StatusCode::NOT_FOUND, "Ambassador not found".into()))?;

    let mut reqs = m.fulfillment_requests.clone();
    if !reqs.is_array() {
        reqs = serde_json::json!([]);
    }
    let entry = serde_json::json!({
        "id": Uuid::new_v4().to_string(),
        "kind": payload.kind.to_string(),
        "landlord_qty": payload.landlord_qty,
        "vendor_qty": payload.vendor_qty,
        "ship_to": payload.ship_to,
        "status": AmbassadorFulfillmentStatus::Requested.to_string(),
        "requested_at": Utc::now().to_rfc3339(),
    });
    reqs.as_array_mut()
        .ok_or((
            StatusCode::INTERNAL_SERVER_ERROR,
            "fulfillment_requests corrupt".into(),
        ))?
        .push(entry);

    let mut active: atlas_ambassador::ActiveModel = m.into();
    active.fulfillment_requests = Set(reqs);
    active.updated_at = Set(Utc::now());
    let updated = active
        .update(&db)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(dto_for(&db, updated).await?))
}

#[derive(Debug, Deserialize)]
pub struct SendAmbassadorInvitePayload {
    pub channel: ReferralInviteChannel,
    pub to: String,
    /// App slug for public base (folio, folio-broker, folio-pm, folio-vendor, network, anchor).
    #[serde(default = "default_app_slug")]
    pub app_slug: String,
}

fn default_app_slug() -> String {
    "folio".into()
}

/// POST /api/admin/ambassadors/:id/send — SMS or email invite-out for any app.
pub async fn send_ambassador_invite(
    State(db): State<DatabaseConnection>,
    Path(id): Path<Uuid>,
    Json(payload): Json<SendAmbassadorInvitePayload>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let m = atlas_ambassador::Entity::find_by_id(id)
        .one(&db)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .ok_or((StatusCode::NOT_FOUND, "Ambassador not found".into()))?;

    if m.status != AmbassadorStatus::Active.to_string() {
        return Err((StatusCode::BAD_REQUEST, "ambassador is not active".into()));
    }

    let url = refer_url_for(&payload.app_slug, &m.code);
    let _ = public_base_for_app(&payload.app_slug);
    let app_label = match payload.app_slug.to_lowercase().as_str() {
        "folio-broker" | "broker" => "Folio Broker",
        "folio-pm" | "pmc" | "pm" => "Folio Property Manager",
        "folio-vendor" | "vendor" => "Folio Vendor",
        "network" => "Network",
        "anchor" => "Anchor",
        _ => "Folio",
    };

    deliver_referral_invite(
        payload.channel,
        &payload.to,
        &url,
        &m.display_name,
        app_label,
    )
    .await
    .map_err(|e| (StatusCode::BAD_GATEWAY, e))?;

    Ok(Json(serde_json::json!({
        "ok": true,
        "refer_url": url,
        "app_slug": payload.app_slug,
        "channel": payload.channel.to_string(),
    })))
}

pub fn routes_raw() -> Router<DatabaseConnection> {
    Router::new()
        .route(
            "/api/admin/ambassadors",
            get(list_ambassadors).post(create_ambassador),
        )
        .route("/api/admin/ambassadors/{id}", get(get_ambassador))
        .route(
            "/api/admin/ambassadors/{id}/campaigns",
            post(attach_campaigns),
        )
        .route("/api/admin/ambassadors/{id}/qr", get(ambassador_qr_png))
        .route(
            "/api/admin/ambassadors/{id}/fulfillments",
            post(create_fulfillment),
        )
        .route(
            "/api/admin/ambassadors/{id}/send",
            post(send_ambassador_invite),
        )
}
