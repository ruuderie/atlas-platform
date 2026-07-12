//! Folio persona referrals — ambassador mint, activity, SMS/email invite-out,
//! and signup attribution into Friends & Family campaigns.
//!
//! | Method | Path | Description |
//! |--------|------|-------------|
//! | GET/POST | /api/folio/me/ambassador | Lazy-create personal share code |
//! | GET | /api/folio/me/referrals | Joined users attributed to my code |
//! | POST | /api/folio/me/referrals/send | SMS or email invite with /refer/{code} |
//! | POST | /api/folio/me/referrals/attribute | Record signup credit for ?ref= |

use axum::{
    Json, Router,
    extract::State,
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
    routing::{get, post},
};
use chrono::Utc;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, ConnectionTrait, DatabaseConnection, EntityTrait, QueryFilter,
    QueryOrder, Set, Statement,
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use uuid::Uuid;

use crate::entities::{atlas_ambassador, atlas_ambassador_campaign, atlas_campaign};
use crate::services::notification_service::channels::{ChannelResult, EmailAdapter, SmsAdapter};
use crate::services::pm::campaign::{CampaignService, EnrollContactPayload, RecordEventPayload};
use crate::types::pm::{
    AmbassadorPartnerType, AmbassadorStatus, CampaignChannel, CampaignEventType, ReferralInviteChannel,
};

fn folio_public_base() -> String {
    std::env::var("FOLIO_PUBLIC_URL")
        .or_else(|_| std::env::var("PUBLIC_BASE_URL"))
        .unwrap_or_else(|_| "https://folio1.atlas.oply.co".to_string())
}

/// Resolve public site base for an app slug (admin + portal share URLs).
pub fn public_base_for_app(app_slug: &str) -> String {
    let key = app_slug.trim().to_lowercase();
    let from_env = match key.as_str() {
        "folio" | "folio-landlord" | "" => std::env::var("FOLIO_PUBLIC_URL").ok(),
        "folio-broker" | "broker" => std::env::var("FOLIO_BROKER_PUBLIC_URL").ok(),
        "folio-pm" | "pmc" | "pm" => std::env::var("FOLIO_PM_PUBLIC_URL").ok(),
        "folio-vendor" | "vendor" => std::env::var("FOLIO_VENDOR_PUBLIC_URL").ok(),
        "network" => std::env::var("NETWORK_PUBLIC_URL").ok(),
        "anchor" => std::env::var("ANCHOR_PUBLIC_URL").ok(),
        _ => None,
    };
    from_env
        .or_else(|| std::env::var("PUBLIC_BASE_URL").ok())
        .unwrap_or_else(folio_public_base)
        .trim_end_matches('/')
        .to_string()
}

pub fn refer_url_for(app_slug: &str, code: &str) -> String {
    format!("{}/refer/{}", public_base_for_app(app_slug), code)
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

fn extract_bearer(headers: &HeaderMap) -> Option<String> {
    if let Some(auth) = headers.get("authorization") {
        if let Ok(val) = auth.to_str() {
            let val = val.trim();
            if val.starts_with("Bearer ") {
                return Some(val["Bearer ".len()..].trim().to_string());
            }
        }
    }
    if let Some(cookie) = headers.get("cookie") {
        if let Ok(val) = cookie.to_str() {
            for part in val.split(';') {
                let part = part.trim();
                if part.starts_with("session=") {
                    return Some(part["session=".len()..].trim().to_string());
                }
                if part.starts_with("atlas_session=") {
                    return Some(part["atlas_session=".len()..].trim().to_string());
                }
            }
        }
    }
    None
}

async fn resolve_caller(
    db: &DatabaseConnection,
    token: &str,
) -> Option<(Uuid, String, Option<String>, Option<Uuid>)> {
    let token_hash = crate::auth::hash_token(token);
    let stmt = Statement::from_sql_and_values(
        sea_orm::DatabaseBackend::Postgres,
        r#"SELECT s.user_id, u.email, u.first_name, u.last_name,
                  (SELECT ua.account_id FROM user_account ua
                   WHERE ua.user_id = s.user_id AND ua.is_active = true
                   ORDER BY ua.created_at ASC LIMIT 1) AS account_id
           FROM sessions s
           JOIN "user" u ON u.id = s.user_id
           WHERE s.bearer_token_hash = $1 AND s.is_active = true
             AND s.token_expiration > now()
           LIMIT 1"#,
        [token_hash.into()],
    );
    let row = db.query_one(stmt).await.ok()??;
    let user_id: Uuid = row.try_get("", "user_id").ok()?;
    let email: String = row.try_get("", "email").ok()?;
    let first: String = row.try_get("", "first_name").unwrap_or_default();
    let last: String = row.try_get("", "last_name").unwrap_or_default();
    let display_name = {
        let n = format!("{} {}", first.trim(), last.trim())
            .trim()
            .to_string();
        if n.is_empty() { None } else { Some(n) }
    };
    let account_id: Option<Uuid> = row.try_get("", "account_id").ok();
    Some((user_id, email, display_name, account_id))
}

pub fn authenticated_routes() -> Router<DatabaseConnection> {
    Router::new()
        .route(
            "/api/folio/me/ambassador",
            get(get_or_create_ambassador).post(get_or_create_ambassador),
        )
        .route("/api/folio/me/referrals", get(list_my_referrals))
        .route("/api/folio/me/referrals/send", post(send_my_referral_invite))
        .route(
            "/api/folio/me/referrals/attribute",
            post(attribute_referral_signup),
        )
}

#[derive(Debug, Serialize)]
pub struct MyAmbassadorDto {
    pub id: Uuid,
    pub code: String,
    pub display_name: String,
    pub refer_url: String,
    pub status: String,
}

async fn ensure_ff_campaign_attach(db: &DatabaseConnection, ambassador_id: Uuid) -> Result<(), String> {
    let now = Utc::now();
    let campaigns = atlas_campaign::Entity::find()
        .filter(
            atlas_campaign::Column::GlobalName
                .is_in(["folio_friends_family", "folio_friends_family_vendors"]),
        )
        .all(db)
        .await
        .map_err(|e| e.to_string())?;
    for c in campaigns {
        let exists = atlas_ambassador_campaign::Entity::find()
            .filter(atlas_ambassador_campaign::Column::AmbassadorId.eq(ambassador_id))
            .filter(atlas_ambassador_campaign::Column::CampaignId.eq(c.id))
            .one(db)
            .await
            .map_err(|e| e.to_string())?;
        if exists.is_none() {
            let link = atlas_ambassador_campaign::ActiveModel {
                ambassador_id: Set(ambassador_id),
                campaign_id: Set(c.id),
                created_at: Set(now),
            };
            link.insert(db).await.map_err(|e| e.to_string())?;
        }
    }
    Ok(())
}

async fn get_or_create_ambassador(
    State(db): State<DatabaseConnection>,
    headers: HeaderMap,
) -> impl IntoResponse {
    let Some(token) = extract_bearer(&headers) else {
        return (
            StatusCode::UNAUTHORIZED,
            Json(json!({"error": "Authentication required"})),
        )
            .into_response();
    };
    let Some((user_id, email, display_name, account_id)) = resolve_caller(&db, &token).await else {
        return (
            StatusCode::UNAUTHORIZED,
            Json(json!({"error": "Invalid session"})),
        )
            .into_response();
    };

    // Prefer existing ambassador linked to this account or created by this user.
    let existing = if let Some(aid) = account_id {
        atlas_ambassador::Entity::find()
            .filter(atlas_ambassador::Column::AccountId.eq(aid))
            .one(&db)
            .await
            .ok()
            .flatten()
    } else {
        None
    };
    let existing = match existing {
        Some(m) => Some(m),
        None => atlas_ambassador::Entity::find()
            .filter(atlas_ambassador::Column::CreatedByUserId.eq(user_id))
            .order_by_asc(atlas_ambassador::Column::CreatedAt)
            .one(&db)
            .await
            .ok()
            .flatten(),
    };

    let model = if let Some(m) = existing {
        m
    } else {
        let local = email.split('@').next().unwrap_or("member");
        let mut code = slugify_code(local);
        if code.is_empty() {
            code = format!("m-{}", &user_id.to_string()[..8]);
        }
        // Ensure uniqueness under sentinel tenant
        let tenant_id = Uuid::nil();
        let mut attempt = code.clone();
        let mut n = 0u32;
        loop {
            let clash = atlas_ambassador::Entity::find()
                .filter(atlas_ambassador::Column::TenantId.eq(tenant_id))
                .filter(atlas_ambassador::Column::Code.eq(&attempt))
                .one(&db)
                .await
                .ok()
                .flatten();
            if clash.is_none() {
                code = attempt;
                break;
            }
            n += 1;
            attempt = format!("{code}-{n}");
            if n > 50 {
                code = format!("u-{}", &Uuid::new_v4().to_string()[..8]);
                break;
            }
        }

        let now = Utc::now();
        let id = Uuid::new_v4();
        let name = display_name
            .filter(|s| !s.trim().is_empty())
            .unwrap_or_else(|| local.to_string());
        let active = atlas_ambassador::ActiveModel {
            id: Set(id),
            tenant_id: Set(tenant_id),
            code: Set(code),
            display_name: Set(name),
            partner_type: Set(AmbassadorPartnerType::Referral.to_string()),
            status: Set(AmbassadorStatus::Active.to_string()),
            account_id: Set(account_id),
            contact_id: Set(None),
            notes: Set(Some("persona_self_serve".into())),
            channels: Set(None),
            fulfillment_requests: Set(json!([])),
            created_by_user_id: Set(Some(user_id)),
            created_at: Set(now),
            updated_at: Set(now),
        };
        match active.insert(&db).await {
            Ok(m) => {
                let _ = ensure_ff_campaign_attach(&db, m.id).await;
                m
            }
            Err(e) => {
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(json!({"error": e.to_string()})),
                )
                    .into_response();
            }
        }
    };

    let dto = MyAmbassadorDto {
        id: model.id,
        code: model.code.clone(),
        display_name: model.display_name.clone(),
        refer_url: refer_url_for("folio", &model.code),
        status: model.status,
    };
    (StatusCode::OK, Json(dto)).into_response()
}

#[derive(Debug, Serialize)]
pub struct ReferralJoinedRow {
    pub email_masked: String,
    pub role: Option<String>,
    pub joined_at: String,
}

#[derive(Debug, Serialize)]
pub struct MyReferralsDto {
    pub code: String,
    pub refer_url: String,
    pub joined: Vec<ReferralJoinedRow>,
    pub by_role: serde_json::Value,
    pub total: usize,
}

async fn list_my_referrals(
    State(db): State<DatabaseConnection>,
    headers: HeaderMap,
) -> impl IntoResponse {
    let Some(token) = extract_bearer(&headers) else {
        return (
            StatusCode::UNAUTHORIZED,
            Json(json!({"error": "Authentication required"})),
        )
            .into_response();
    };
    let Some((user_id, _, _, account_id)) = resolve_caller(&db, &token).await else {
        return (
            StatusCode::UNAUTHORIZED,
            Json(json!({"error": "Invalid session"})),
        )
            .into_response();
    };

    let ambassador = if let Some(aid) = account_id {
        atlas_ambassador::Entity::find()
            .filter(atlas_ambassador::Column::AccountId.eq(aid))
            .one(&db)
            .await
            .ok()
            .flatten()
    } else {
        None
    };
    let ambassador = match ambassador {
        Some(m) => m,
        None => match atlas_ambassador::Entity::find()
            .filter(atlas_ambassador::Column::CreatedByUserId.eq(user_id))
            .one(&db)
            .await
            .ok()
            .flatten()
        {
            Some(m) => m,
            None => {
                return (
                    StatusCode::OK,
                    Json(MyReferralsDto {
                        code: String::new(),
                        refer_url: String::new(),
                        joined: vec![],
                        by_role: json!({}),
                        total: 0,
                    }),
                )
                    .into_response();
            }
        },
    };

    let code = ambassador.code.clone();
    let stmt = Statement::from_sql_and_values(
        sea_orm::DatabaseBackend::Postgres,
        r#"SELECT contact_email, contact_metadata, enrolled_at
           FROM atlas_campaign_enrollments
           WHERE contact_metadata->>'referred_by' = $1
           ORDER BY enrolled_at DESC
           LIMIT 200"#,
        [code.clone().into()],
    );
    let rows = db.query_all(stmt).await.unwrap_or_default();

    let mut joined = Vec::new();
    let mut by_role: serde_json::Map<String, serde_json::Value> = serde_json::Map::new();
    for row in rows {
        let email: String = row
            .try_get("", "contact_email")
            .unwrap_or_else(|_| "unknown".into());
        let meta: Option<serde_json::Value> = row.try_get("", "contact_metadata").ok();
        let role = meta
            .as_ref()
            .and_then(|m| m.get("role"))
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());
        if let Some(ref r) = role {
            let entry = by_role.entry(r.clone()).or_insert(json!(0));
            if let Some(n) = entry.as_u64() {
                *entry = json!(n + 1);
            }
        }
        let enrolled_at: chrono::DateTime<Utc> = row
            .try_get("", "enrolled_at")
            .unwrap_or_else(|_| Utc::now());
        joined.push(ReferralJoinedRow {
            email_masked: mask_email(&email),
            role,
            joined_at: enrolled_at.to_rfc3339(),
        });
    }

    let total = joined.len();
    (
        StatusCode::OK,
        Json(MyReferralsDto {
            code: code.clone(),
            refer_url: refer_url_for("folio", &code),
            joined,
            by_role: serde_json::Value::Object(by_role),
            total,
        }),
    )
        .into_response()
}

fn mask_email(email: &str) -> String {
    let Some((local, domain)) = email.split_once('@') else {
        return "•••".into();
    };
    let keep = local.chars().next().unwrap_or('•');
    format!("{keep}•••@{domain}")
}

#[derive(Debug, Deserialize)]
pub struct SendReferralInvitePayload {
    pub channel: ReferralInviteChannel,
    pub to: String,
}

/// Shared send used by Folio me + admin ambassadors.
pub async fn deliver_referral_invite(
    channel: ReferralInviteChannel,
    to: &str,
    refer_url: &str,
    inviter_name: &str,
    app_label: &str,
) -> Result<(), String> {
    let to = to.trim();
    if to.is_empty() {
        return Err("recipient is required".into());
    }
    let body = format!(
        "Hey — {inviter_name} invited you to {app_label}. Create your account: {refer_url}"
    );
    match channel {
        ReferralInviteChannel::Sms => {
            let cfg = json!({ "phone": to });
            match SmsAdapter::send(&cfg, &body).await {
                ChannelResult::Delivered => Ok(()),
                ChannelResult::Skipped { reason } => Err(reason),
                ChannelResult::Failed { error } => Err(error),
            }
        }
        ReferralInviteChannel::Email => {
            let cfg = json!({ "email": to });
            let title = format!("You're invited to {app_label}");
            match EmailAdapter::send(&cfg, &title, &body, Some(refer_url)).await {
                ChannelResult::Delivered => Ok(()),
                ChannelResult::Skipped { reason } => Err(reason),
                ChannelResult::Failed { error } => Err(error),
            }
        }
    }
}

async fn send_my_referral_invite(
    State(db): State<DatabaseConnection>,
    headers: HeaderMap,
    Json(payload): Json<SendReferralInvitePayload>,
) -> impl IntoResponse {
    let Some(token) = extract_bearer(&headers) else {
        return (
            StatusCode::UNAUTHORIZED,
            Json(json!({"error": "Authentication required"})),
        )
            .into_response();
    };
    let Some((user_id, _, display_name, account_id)) = resolve_caller(&db, &token).await else {
        return (
            StatusCode::UNAUTHORIZED,
            Json(json!({"error": "Invalid session"})),
        )
            .into_response();
    };

    let ambassador = if let Some(aid) = account_id {
        atlas_ambassador::Entity::find()
            .filter(atlas_ambassador::Column::AccountId.eq(aid))
            .one(&db)
            .await
            .ok()
            .flatten()
    } else {
        None
    };
    let Some(ambassador) = (match ambassador {
        Some(m) => Some(m),
        None => atlas_ambassador::Entity::find()
            .filter(atlas_ambassador::Column::CreatedByUserId.eq(user_id))
            .one(&db)
            .await
            .ok()
            .flatten(),
    }) else {
        return (
            StatusCode::NOT_FOUND,
            Json(json!({"error": "Generate your share link first"})),
        )
            .into_response();
    };

    let url = refer_url_for("folio", &ambassador.code);
    let name = display_name
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| ambassador.display_name.clone());
    match deliver_referral_invite(payload.channel, &payload.to, &url, &name, "Folio").await {
        Ok(()) => (StatusCode::OK, Json(json!({"ok": true, "refer_url": url}))).into_response(),
        Err(e) => (
            StatusCode::BAD_GATEWAY,
            Json(json!({"error": e})),
        )
            .into_response(),
    }
}

#[derive(Debug, Deserialize)]
pub struct AttributeReferralPayload {
    pub referred_by: String,
    #[serde(default)]
    pub role: Option<String>,
}

/// Credit a signup to an ambassador code (campaign enrollment + event).
pub async fn record_referral_signup(
    db: &DatabaseConnection,
    email: &str,
    display_name: Option<&str>,
    user_id: Option<Uuid>,
    referred_by: &str,
    role: Option<&str>,
) -> Result<(), String> {
    let code = slugify_code(referred_by);
    if code.is_empty() {
        return Ok(());
    }

    // Prefer landlord F&F campaign; fall back to any matching utm.
    let campaign = atlas_campaign::Entity::find()
        .filter(atlas_campaign::Column::GlobalName.eq("folio_friends_family"))
        .filter(atlas_campaign::Column::Status.eq("active"))
        .one(db)
        .await
        .map_err(|e| e.to_string())?;
    let campaign = match campaign {
        Some(c) => c,
        None => {
            atlas_campaign::Entity::find()
                .filter(atlas_campaign::Column::UtmCampaign.eq("friends_family"))
                .filter(atlas_campaign::Column::Status.eq("active"))
                .one(db)
                .await
                .map_err(|e| e.to_string())?
                .ok_or_else(|| "friends_family campaign not found".to_string())?
        }
    };

    if CampaignService::find_enrollment_by_email(db, campaign.id, email)
        .await
        .map_err(|e| e.to_string())?
        .is_some()
    {
        return Ok(());
    }

    let enrollment = CampaignService::enroll(
        db,
        campaign.tenant_id,
        EnrollContactPayload {
            campaign_id: campaign.id,
            contact_user_id: user_id,
            contact_email: Some(email.to_string()),
            contact_name: display_name.map(str::to_string).filter(|s| !s.is_empty()),
            contact_metadata: Some(json!({
                "source": "referral:friends_family",
                "referred_by": code,
                "utm_campaign": "friends_family",
                "role": role,
                "credit": "account_signup",
            })),
            external_enrollment_id: None,
            next_step_at: None,
        },
    )
    .await
    .map_err(|e| e.to_string())?;

    let _ = CampaignService::record_event(
        db,
        campaign.tenant_id,
        RecordEventPayload {
            enrollment_id: enrollment.id,
            event_type: CampaignEventType::FormFill,
            channel: CampaignChannel::Referral,
            sequence_step_id: None,
            link_clicked: None,
            ip_address: None,
            user_agent: None,
            metadata: Some(json!({
                "referred_by": code,
                "role": role,
                "credit": "account_signup",
            })),
            conversion_entity_type: None,
            conversion_entity_id: None,
        },
    )
    .await;

    Ok(())
}

async fn attribute_referral_signup(
    State(db): State<DatabaseConnection>,
    headers: HeaderMap,
    Json(payload): Json<AttributeReferralPayload>,
) -> impl IntoResponse {
    let Some(token) = extract_bearer(&headers) else {
        return (
            StatusCode::UNAUTHORIZED,
            Json(json!({"error": "Authentication required"})),
        )
            .into_response();
    };
    let Some((user_id, email, display_name, _)) = resolve_caller(&db, &token).await else {
        return (
            StatusCode::UNAUTHORIZED,
            Json(json!({"error": "Invalid session"})),
        )
            .into_response();
    };

    match record_referral_signup(
        &db,
        &email,
        display_name.as_deref(),
        Some(user_id),
        &payload.referred_by,
        payload.role.as_deref(),
    )
    .await
    {
        Ok(()) => (StatusCode::OK, Json(json!({"ok": true}))).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": e})),
        )
            .into_response(),
    }
}
