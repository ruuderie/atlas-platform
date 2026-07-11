//! Vendor review invite handler.
//!
//! Vendors send review invite links to property owners they've worked with.
//! The invite is stored in `platform_invite` with `invite_purpose = 'review_request'`
//! and `context_entity_id = atlas_service_providers.id`.
//!
//! # Routes (authenticated — vendor role required)
//!
//! ```ignore
//! POST /api/folio/review-invites
//!      Vendor creates an invite link for a past client.
//!      Body: CreateReviewInviteInput
//!      -> 201 { "invite_id": uuid, "review_url": "https://..." }
//! ```
//!
//! # Public routes (no auth — cold-traffic)
//!
//! ```ignore
//! GET  /api/pub/review/:invite_id
//!      Validate the invite token. Returns the vendor's G-27 scorecard
//!      + dimensions so the frontend can render the correct input types.
//!      -> 200 ReviewContext  |  404 | 410 (expired)
//!
//! POST /api/pub/review/:invite_id/submit
//!      OTP-verified submission. Opens a G-27 session, writes one entry
//!      per dimension, and sets testimonial on atlas_rating_sessions.
//!      Body: SubmitReviewInput
//!      -> 201 { "session_id": uuid }
//! ```
//!
//! # Public vendor profile route
//!
//! ```ignore
//! GET  /api/pub/vendors/:sp_id
//!      Public vendor profile — G-27 aggregates + published reviews.
//!      -> 200 PublicVendorProfile  |  404
//! ```

use axum::{
    Router,
    extract::{Path, State, Json},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
};
use sea_orm::{DatabaseBackend, DatabaseConnection, Statement, ConnectionTrait};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::Utc;
use std::env;
use axum::extract::Query;

use crate::types::pm::InvitePurpose;
use crate::services::notification_service::{NotificationService, DispatchInput, NotificationPriority};

// ── Route registration ────────────────────────────────────────────────────────

/// Authenticated routes — vendor must be logged in.
pub fn authenticated_routes_raw() -> Router<DatabaseConnection> {
    Router::new()
        .route("/api/folio/review-invites", post(create_review_invite))
}

/// Public routes — no auth header required.
pub fn public_routes_raw() -> Router<DatabaseConnection> {
    Router::new()
        .route("/api/pub/review/{invite_id}",        get(get_review_context))
        .route("/api/pub/review/{invite_id}/submit",  post(submit_review))
        .route("/api/pub/vendors/{sp_id}",            get(get_public_vendor_profile))
        // Renter help — public vendor search + public service request
        .route("/api/pub/vendors",                    get(search_vendors))
        .route("/api/pub/service-requests",           post(create_public_service_request))
}

// ── Types ─────────────────────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct CreateReviewInviteInput {
    /// Email address of the property owner being invited to review.
    pub email:           String,
    pub display_name:    Option<String>,
    /// The atlas_service_providers.id of the vendor creating the invite.
    /// In production this will be read from the auth session; accepted here
    /// for initial wiring before middleware is in place.
    pub vendor_id:       Uuid,
    /// Optional personal message to include in the invite email.
    pub invite_note:     Option<String>,
    /// Expiry in days. Default 14. Max 60.
    pub expires_days:    Option<u32>,
}

#[derive(Debug, Serialize)]
pub struct CreateReviewInviteResponse {
    pub invite_id:  Uuid,
    pub review_url: String,
}

/// G-27 scorecard dimension context returned for the review form.
#[derive(Debug, Serialize)]
pub struct ReviewDimension {
    pub id:          Uuid,
    pub label:       String,
    pub scale_type:  String,  // "rating" | "boolean" | "absolute" | "poll_single" | "poll_multi"
    pub scale_min:   Option<f64>,
    pub scale_max:   Option<f64>,
    pub unit_label:  Option<String>,
    pub description: Option<String>,
}

/// Full context returned for the public review form.
#[derive(Debug, Serialize)]
pub struct ReviewContext {
    pub invite_id:    Uuid,
    pub vendor_name:  String,
    pub scorecard_id: Uuid,
    pub dimensions:   Vec<ReviewDimension>,
}

/// One dimension score submitted by the reviewer.
#[derive(Debug, Deserialize)]
pub struct DimensionScore {
    pub dimension_id: Uuid,
    /// Numeric score (for rating/absolute/boolean dimensions).
    /// Boolean: 1.0 = yes, 0.0 = no.
    pub score:        Option<f64>,
    /// Option ID for poll_single / poll_multi dimensions.
    pub option_id:    Option<Uuid>,
}

#[derive(Debug, Deserialize)]
pub struct SubmitReviewInput {
    /// OTP-verified email of the reviewer.
    pub reviewer_email: String,
    pub scores:         Vec<DimensionScore>,
    /// Free-text review body.
    pub testimonial:    Option<String>,
}

#[derive(Debug, Serialize)]
struct SubmitReviewResponse {
    pub session_id: Uuid,
}

/// Single published review entry.
#[derive(Debug, Serialize)]
pub struct PublishedReview {
    pub session_id:    Uuid,
    pub testimonial:   Option<String>,
    pub published_at:  String,
    pub reviewer_name: Option<String>,
}

/// G-27 aggregate score for one dimension.
#[derive(Debug, Serialize)]
pub struct DimensionAggregate {
    pub dimension_label: String,
    pub scale_type:      String,
    pub avg_score:       Option<f64>,
    pub total_responses: i64,
}

/// Public vendor profile response — shareable, unauthenticated.
#[derive(Debug, Serialize)]
pub struct PublicVendorProfile {
    pub vendor_id:   Uuid,
    pub vendor_name: String,
    pub trade_type:  Option<String>,
    pub bio:         Option<String>,
    pub aggregates:  Vec<DimensionAggregate>,
    pub reviews:     Vec<PublishedReview>,
    pub total_score: Option<f64>,
    pub review_count: i64,
}

// ── Authenticated Handlers ────────────────────────────────────────────────────

/// POST /api/folio/review-invites
///
/// Vendor creates a review invite link. Writes a `platform_invite` row with
/// `invite_purpose = 'review_request'` and `context_entity_id = vendor_id`.
/// Returns the invite ID and the public review URL for sharing.
pub async fn create_review_invite(
    State(db): State<DatabaseConnection>,
    Json(body): Json<CreateReviewInviteInput>,
) -> impl IntoResponse {
    let email_lower  = body.email.to_lowercase();
    let invite_id    = Uuid::new_v4();
    let expires_days = body.expires_days.unwrap_or(14).min(60) as i64;
    let expires_at   = Utc::now() + chrono::Duration::days(expires_days);
    let purpose      = InvitePurpose::ReviewRequest.to_string();

    let insert_sql = format!(
        "INSERT INTO platform_invite \
         (id, email, role, tenant_name, invited_by, display_name, app_role, \
          invite_purpose, context_entity_id, personal_message, created_at, expires_at) \
         VALUES \
         ('{id}'::uuid, '{email}', 'Member', 'Folio', 'vendor', {display_name}, \
          'property_owner_lite', '{purpose}', '{vendor_id}'::uuid, \
          {note}, NOW(), '{expires_at}'::timestamptz) \
         ON CONFLICT DO NOTHING;",
        id           = invite_id,
        email        = email_lower.replace('\'', "''"),
        display_name = body.display_name.as_deref()
            .map(|n| format!("'{}'", n.replace('\'', "''")))
            .unwrap_or_else(|| "NULL".to_string()),
        purpose      = purpose,
        vendor_id    = body.vendor_id,
        note         = body.invite_note.as_deref()
            .map(|n| format!("'{}'", n.replace('\'', "''")))
            .unwrap_or_else(|| "NULL".to_string()),
        expires_at   = expires_at.format("%Y-%m-%dT%H:%M:%SZ"),
    );

    if let Err(e) = db.execute(
        Statement::from_string(DatabaseBackend::Postgres, insert_sql)
    ).await {
        tracing::error!(error = %e, "create_review_invite: insert failed");
        return StatusCode::INTERNAL_SERVER_ERROR.into_response();
    }

    let frontend_url = env::var("FRONTEND_URL")
        .unwrap_or_else(|_| "https://folio.uat.atlas.oply.co".to_string());
    let review_url = format!("{}/review/{}", frontend_url, invite_id);

    tracing::info!(
        event = "review_invite.created",
        invite_id = %invite_id,
        vendor_id = %body.vendor_id,
        email = %email_lower,
    );

    (StatusCode::CREATED, axum::Json(CreateReviewInviteResponse {
        invite_id,
        review_url,
    })).into_response()
}

// ── Public Handlers ───────────────────────────────────────────────────────────

/// GET /api/pub/review/:invite_id
///
/// Validates a review invite token. Returns the vendor G-27 scorecard
/// context needed to render dimension inputs on the public review form.
pub async fn get_review_context(
    State(db): State<DatabaseConnection>,
    Path(invite_id): Path<Uuid>,
) -> impl IntoResponse {
    // 1. Fetch invite — must exist, be unexpired, and be a review_request invite
    let invite_sql = Statement::from_sql_and_values(
        sea_orm::DatabaseBackend::Postgres,
        r#"SELECT pi.id, pi.context_entity_id, pi.expires_at,
                  sp.business_name AS vendor_name
           FROM platform_invite pi
           JOIN atlas_service_providers sp ON sp.id = pi.context_entity_id
           WHERE pi.id = $1
             AND pi.invite_purpose = 'review_request'
             AND pi.expires_at > NOW()"#,
        [invite_id.into()],
    );

    let invite_row = match db.query_one(invite_sql).await {
        Ok(Some(r)) => r,
        Ok(None) => {
            return (StatusCode::NOT_FOUND, axum::Json(serde_json::json!({
                "error": "Invite not found or has expired."
            }))).into_response();
        }
        Err(e) => {
            tracing::error!(error = %e, "get_review_context: invite query failed");
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };

    let vendor_id:   Uuid   = invite_row.try_get("", "context_entity_id").unwrap();
    let vendor_name: String = invite_row.try_get("", "vendor_name").unwrap_or_default();

    // 2. Look up the vendor's G-27 scorecard (entity_type = 'atlas_service_provider')
    let scorecard_sql = Statement::from_sql_and_values(
        sea_orm::DatabaseBackend::Postgres,
        r#"SELECT s.id AS scorecard_id
           FROM atlas_scorecards s
           WHERE s.subject_entity_type = 'atlas_service_provider'
             AND s.subject_entity_id = $1
           ORDER BY s.created_at DESC
           LIMIT 1"#,
        [vendor_id.into()],
    );

    let scorecard_row = match db.query_one(scorecard_sql).await {
        Ok(Some(r)) => r,
        _ => {
            return (StatusCode::NOT_FOUND, axum::Json(serde_json::json!({
                "error": "No scorecard found for this vendor."
            }))).into_response();
        }
    };

    let scorecard_id: Uuid = scorecard_row.try_get("", "scorecard_id").unwrap();

    // 3. Fetch dimensions for this scorecard's template
    let dims_sql = Statement::from_sql_and_values(
        sea_orm::DatabaseBackend::Postgres,
        r#"SELECT sd.id, sd.label, sd.scale_type, sd.scale_min, sd.scale_max,
                  sd.unit_label, sd.description
           FROM atlas_scorecard_dimensions sd
           JOIN atlas_scorecards s ON s.template_id = sd.template_id
           WHERE s.id = $1
             AND sd.is_active = true
           ORDER BY sd.sort_order ASC, sd.created_at ASC"#,
        [scorecard_id.into()],
    );

    let dims = match db.query_all(dims_sql).await {
        Ok(rows) => rows.into_iter().filter_map(|r| {
            Some(ReviewDimension {
                id:          r.try_get("", "id").ok()?,
                label:       r.try_get("", "label").ok()?,
                scale_type:  r.try_get("", "scale_type").ok()?,
                scale_min:   r.try_get("", "scale_min").ok().unwrap_or(None),
                scale_max:   r.try_get("", "scale_max").ok().unwrap_or(None),
                unit_label:  r.try_get("", "unit_label").ok().unwrap_or(None),
                description: r.try_get("", "description").ok().unwrap_or(None),
            })
        }).collect(),
        Err(e) => {
            tracing::error!(error = %e, "get_review_context: dimensions query failed");
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };

    (StatusCode::OK, axum::Json(ReviewContext {
        invite_id,
        vendor_name,
        scorecard_id,
        dimensions: dims,
    })).into_response()
}

/// POST /api/pub/review/:invite_id/submit
///
/// OTP-verified review submission. Opens a G-27 rating session, writes one
/// scorecard entry per dimension score, and stores the testimonial.
/// `published_at` is left NULL — the platform admin queue publishes it.
pub async fn submit_review(
    State(db): State<DatabaseConnection>,
    Path(invite_id): Path<Uuid>,
    Json(body): Json<SubmitReviewInput>,
) -> impl IntoResponse {
    let email_lower = body.reviewer_email.to_lowercase();

    // 1. Validate invite (same as get_review_context — must be unexpired)
    let invite_sql = Statement::from_sql_and_values(
        sea_orm::DatabaseBackend::Postgres,
        r#"SELECT pi.context_entity_id AS vendor_id,
                  s.id AS scorecard_id,
                  s.template_id,
                  asp.user_id      AS vendor_user_id,
                  asp.tenant_id    AS vendor_tenant_id
           FROM platform_invite pi
           JOIN atlas_scorecards s
             ON s.subject_entity_type = 'atlas_service_provider'
            AND s.subject_entity_id = pi.context_entity_id
           JOIN atlas_service_providers asp
             ON asp.id = pi.context_entity_id
           WHERE pi.id = $1
             AND pi.invite_purpose = 'review_request'
             AND pi.expires_at > NOW()
           ORDER BY s.created_at DESC
           LIMIT 1"#,
        [invite_id.into()],
    );

    let ctx = match db.query_one(invite_sql).await {
        Ok(Some(r)) => r,
        Ok(None) => {
            return (StatusCode::GONE, axum::Json(serde_json::json!({
                "error": "This review link has expired or is invalid."
            }))).into_response();
        }
        Err(e) => {
            tracing::error!(error = %e, "submit_review: context query failed");
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };

    let scorecard_id: Uuid = ctx.try_get("", "scorecard_id").unwrap();
    // Vendor identity — needed for G35 notification dispatch.
    let vendor_user_id: Option<Uuid>   = ctx.try_get("", "vendor_user_id").ok();
    let vendor_tenant_id: Option<Uuid> = ctx.try_get("", "vendor_tenant_id").ok();

    // 2. Resolve rater user_id (or create a stub)
    let rater_sql = Statement::from_sql_and_values(
        sea_orm::DatabaseBackend::Postgres,
        "SELECT id FROM users WHERE email = $1 LIMIT 1",
        [email_lower.clone().into()],
    );
    let rater_id: Uuid = match db.query_one(rater_sql).await {
        Ok(Some(r)) => r.try_get("", "id").unwrap_or_else(|_| Uuid::new_v4()),
        _ => Uuid::new_v4(), // stub — real auth middleware supplies this
    };

    // 3. Anti-fraud: check for duplicate session (same scorecard + rater)
    let dup_sql = Statement::from_sql_and_values(
        sea_orm::DatabaseBackend::Postgres,
        "SELECT id FROM atlas_rating_sessions WHERE scorecard_id = $1 AND rater_id = $2 LIMIT 1",
        [scorecard_id.into(), rater_id.into()],
    );
    if let Ok(Some(_)) = db.query_one(dup_sql).await {
        return (StatusCode::CONFLICT, axum::Json(serde_json::json!({
            "error": "You have already submitted a review for this vendor."
        }))).into_response();
    }

    // 4. Insert atlas_rating_sessions row (testimonial set; published_at = NULL → held for moderation)
    let session_id = Uuid::new_v4();
    let testimonial_sql_val = body.testimonial.as_deref()
        .map(|t| format!("'{}'", t.replace('\'', "''")))
        .unwrap_or_else(|| "NULL".to_string());

    let session_sql = format!(
        "INSERT INTO atlas_rating_sessions \
         (id, scorecard_id, rater_id, status, testimonial, is_flagged, published_at, created_at, updated_at) \
         VALUES ('{sid}'::uuid, '{sc}'::uuid, '{rater}'::uuid, 'submitted', \
                 {testimonial}, false, NULL, NOW(), NOW());",
        sid        = session_id,
        sc         = scorecard_id,
        rater      = rater_id,
        testimonial = testimonial_sql_val,
    );

    if let Err(e) = db.execute(
        Statement::from_string(DatabaseBackend::Postgres, session_sql)
    ).await {
        tracing::error!(error = %e, "submit_review: session insert failed");
        return StatusCode::INTERNAL_SERVER_ERROR.into_response();
    }

    // 5. Insert one atlas_scorecard_entries row per dimension score
    for score in &body.scores {
        let score_val = score.score
            .map(|s| format!("{}", s))
            .unwrap_or_else(|| "NULL".to_string());
        let option_id_val = score.option_id
            .map(|o| format!("'{}'::uuid", o))
            .unwrap_or_else(|| "NULL".to_string());

        let entry_sql = format!(
            "INSERT INTO atlas_scorecard_entries \
             (id, session_id, dimension_id, score, option_id, created_at) \
             VALUES (gen_random_uuid(), '{sid}'::uuid, '{dim}'::uuid, {score}, {opt}, NOW());",
            sid   = session_id,
            dim   = score.dimension_id,
            score = score_val,
            opt   = option_id_val,
        );
        if let Err(e) = db.execute(
            Statement::from_string(DatabaseBackend::Postgres, entry_sql)
        ).await {
            tracing::error!(error = %e, dimension_id = %score.dimension_id, "submit_review: entry insert failed");
            // Continue — partial score is better than total failure
        }
    }

    tracing::info!(
        event = "review.submitted",
        session_id = %session_id,
        scorecard_id = %scorecard_id,
        rater_id = %rater_id,
        invite_id = %invite_id,
    );

    // G35 — notify vendor that a review was submitted (held for moderation).
    // Best-effort: errors are logged, never block the 201 response.
    if let (Some(v_user), Some(v_tenant)) = (vendor_user_id, vendor_tenant_id) {
        let dispatch = DispatchInput {
            tenant_id:         v_tenant,
            user_id:           v_user,
            notification_type: "review_received".to_string(),
            title:             "New review submitted".to_string(),
            body:              "A property owner submitted a review. \
                               It will appear on your profile after moderation.".to_string(),
            priority:          NotificationPriority::Normal,
            entity_type:       Some("atlas_rating_sessions".to_string()),
            entity_id:         Some(session_id),
            metadata:          Some(serde_json::json!({
                "invite_id": invite_id,
                "moderation_held": true
            })),
            include_broadcast: false,
        };
        if let Err(e) = NotificationService::dispatch(&db, dispatch).await {
            tracing::warn!(error = %e, vendor_user_id = %v_user, "G35: review_received notify failed (non-fatal)");
        }

        // G-36: complete review_submitted outcomes for the vendor actor's NetworkInvite actions.
        if let Err(e) = crate::services::program_service::ProgramService::complete_outcomes_for_actor_user(
            &db,
            v_user,
            crate::types::pm::ProgramOutcomeType::ReviewSubmitted,
            Some("atlas_rating_sessions"),
            Some(session_id),
        )
        .await
        {
            tracing::warn!(
                error = %e,
                vendor_user_id = %v_user,
                "G-36 review_submitted failed (non-fatal)"
            );
        }
    }

    (StatusCode::CREATED, axum::Json(SubmitReviewResponse { session_id })).into_response()
}

/// GET /api/pub/vendors/:sp_id
///
/// Public vendor profile — unauthenticated, shareable.
/// Returns the vendor's name, trade type, G-27 dimension aggregates, and all
/// published (non-NULL published_at, non-flagged) review testimonials.
pub async fn get_public_vendor_profile(
    State(db): State<DatabaseConnection>,
    Path(sp_id): Path<Uuid>,
) -> impl IntoResponse {
    // 1. Fetch vendor base info
    let vendor_sql = Statement::from_sql_and_values(
        sea_orm::DatabaseBackend::Postgres,
        r#"SELECT id, business_name,
                  provider_metadata->>'trade_type' AS trade_type,
                  provider_metadata->>'bio'        AS bio
           FROM atlas_service_providers
           WHERE id = $1"#,
        [sp_id.into()],
    );

    let vendor = match db.query_one(vendor_sql).await {
        Ok(Some(r)) => r,
        _ => {
            return (StatusCode::NOT_FOUND, axum::Json(serde_json::json!({
                "error": "Vendor not found."
            }))).into_response();
        }
    };

    let vendor_name: String = vendor.try_get("", "business_name").unwrap_or_default();
    let trade_type: Option<String> = vendor.try_get("", "trade_type").ok().unwrap_or(None);
    let bio: Option<String>        = vendor.try_get("", "bio").ok().unwrap_or(None);

    // 2. Fetch G-27 dimension aggregates
    let agg_sql = Statement::from_sql_and_values(
        sea_orm::DatabaseBackend::Postgres,
        r#"SELECT sd.label AS dimension_label, sd.scale_type,
                  ada.avg_score, ada.total_responses
           FROM atlas_scorecard_dimension_aggregates ada
           JOIN atlas_scorecard_dimensions sd ON sd.id = ada.dimension_id
           JOIN atlas_scorecards sc ON sc.id = ada.scorecard_id
           WHERE sc.subject_entity_type = 'atlas_service_provider'
             AND sc.subject_entity_id = $1
           ORDER BY sd.sort_order ASC"#,
        [sp_id.into()],
    );

    let aggregates: Vec<DimensionAggregate> = db.query_all(agg_sql).await
        .unwrap_or_default()
        .into_iter()
        .filter_map(|r| {
            Some(DimensionAggregate {
                dimension_label: r.try_get("", "dimension_label").ok()?,
                scale_type:      r.try_get("", "scale_type").ok()?,
                avg_score:       r.try_get("", "avg_score").ok().unwrap_or(None),
                total_responses: r.try_get("", "total_responses").unwrap_or(0),
            })
        })
        .collect();

    // 3. Fetch published reviews (published_at IS NOT NULL, is_flagged = false)
    let reviews_sql = Statement::from_sql_and_values(
        sea_orm::DatabaseBackend::Postgres,
        r#"SELECT ars.id AS session_id, ars.testimonial, ars.published_at,
                  u.first_name AS reviewer_name
           FROM atlas_rating_sessions ars
           JOIN atlas_scorecards sc ON sc.id = ars.scorecard_id
           LEFT JOIN users u ON u.id = ars.rater_id
           WHERE sc.subject_entity_type = 'atlas_service_provider'
             AND sc.subject_entity_id = $1
             AND ars.published_at IS NOT NULL
             AND ars.is_flagged = false
           ORDER BY ars.published_at DESC
           LIMIT 20"#,
        [sp_id.into()],
    );

    let reviews: Vec<PublishedReview> = db.query_all(reviews_sql).await
        .unwrap_or_default()
        .into_iter()
        .filter_map(|r| {
            let session_id: Uuid = r.try_get("", "session_id").ok()?;
            let published_at: chrono::DateTime<Utc> = r.try_get("", "published_at").ok()?;
            Some(PublishedReview {
                session_id,
                testimonial:   r.try_get("", "testimonial").ok().unwrap_or(None),
                published_at:  published_at.format("%Y-%m-%dT%H:%M:%SZ").to_string(),
                reviewer_name: r.try_get("", "reviewer_name").ok().unwrap_or(None),
            })
        })
        .collect();

    // 4. Compute overall average from aggregates (weighted by total_responses)
    let total_score: Option<f64> = {
        let (sum, count): (f64, f64) = aggregates.iter()
            .filter(|a| a.scale_type == "rating" || a.scale_type == "absolute")
            .filter_map(|a| a.avg_score.map(|s| (s * a.total_responses as f64, a.total_responses as f64)))
            .fold((0.0, 0.0), |(s, c), (sv, cv)| (s + sv, c + cv));
        if count > 0.0 { Some((sum / count * 10.0).round() / 10.0) } else { None }
    };

    let review_count = reviews.len() as i64;

    (StatusCode::OK, axum::Json(PublicVendorProfile {
        vendor_id: sp_id,
        vendor_name,
        trade_type,
        bio,
        aggregates,
        reviews,
        total_score,
        review_count,
    })).into_response()
}

// ── Public vendor search ──────────────────────────────────────────────────────

/// GET /api/pub/vendors?trade=electrical&zip=33101&limit=20
///
/// Zero-auth. Renter help page uses this to populate the vendor grid.
/// Entry points: Google search, vendor-shared link (/help?trade=X), QR codes.
///
/// Filters:
///   trade  — case-insensitive substring match on provider_metadata->>'trade_type'
///   zip    — future: geo filter (currently ignored, returns all matching trade)
///   q      — free-text search against business_name
///   limit  — max results (default 20, max 50)
#[derive(Debug, Deserialize)]
pub struct VendorSearchParams {
    pub trade: Option<String>,
    pub zip:   Option<String>,
    pub q:     Option<String>,
    pub limit: Option<i64>,
}

#[derive(Debug, Serialize)]
pub struct VendorSearchItem {
    pub id:           Uuid,
    pub business_name: String,
    pub trade_type:   Option<String>,
    pub avg_score:    Option<f64>,
    pub review_count: i64,
    pub bio:          Option<String>,
    pub verified:     bool,
}

#[derive(Debug, Serialize)]
pub struct VendorSearchResponse {
    pub vendors: Vec<VendorSearchItem>,
    pub total:   usize,
}

pub async fn search_vendors(
    State(db): State<DatabaseConnection>,
    Query(params): Query<VendorSearchParams>,
) -> impl IntoResponse {
    let limit = params.limit.unwrap_or(20).min(50).max(1);

    // Build dynamic WHERE clause
    let mut conditions = vec!["asp.is_active = true".to_string()];
    let mut bind_values: Vec<sea_orm::Value> = vec![];
    let mut bind_idx = 1usize;

    if let Some(ref trade) = params.trade {
        conditions.push(format!(
            "LOWER(asp.provider_metadata->>'trade_type') LIKE ${bind_idx}"
        ));
        bind_values.push(format!("%{}%", trade.to_lowercase()).into());
        bind_idx += 1;
    }

    if let Some(ref q) = params.q {
        conditions.push(format!(
            "LOWER(asp.business_name) LIKE ${bind_idx}"
        ));
        bind_values.push(format!("%{}%", q.to_lowercase()).into());
        bind_idx += 1;
    }

    let where_clause = conditions.join(" AND ");

    let sql = format!(
        r#"SELECT
               asp.id,
               asp.business_name,
               asp.provider_metadata->>'trade_type' AS trade_type,
               asp.provider_metadata->>'bio'        AS bio,
               asp.provider_metadata->>'verified'   AS verified,
               COALESCE(agg.avg_overall, NULL)      AS avg_score,
               COALESCE(agg.review_count, 0)        AS review_count
           FROM atlas_service_providers asp
           LEFT JOIN LATERAL (
               SELECT
                   ROUND(AVG(e.score)::numeric, 1)::float8 AS avg_overall,
                   COUNT(DISTINCT rs.id)                   AS review_count
               FROM atlas_rating_sessions rs
               JOIN atlas_scorecards sc
                 ON sc.id = rs.scorecard_id
                AND sc.subject_entity_type = 'atlas_service_provider'
                AND sc.subject_entity_id = asp.id
               JOIN atlas_scorecard_entries e ON e.session_id = rs.id
               WHERE rs.published_at IS NOT NULL AND rs.is_flagged = false
           ) agg ON true
           WHERE {where_clause}
           ORDER BY agg.avg_overall DESC NULLS LAST, agg.review_count DESC NULLS LAST
           LIMIT {limit}"#,
        where_clause = where_clause,
        limit = limit,
    );

    let stmt = if bind_values.is_empty() {
        Statement::from_string(DatabaseBackend::Postgres, sql)
    } else {
        Statement::from_sql_and_values(DatabaseBackend::Postgres, &sql, bind_values)
    };

    let rows = match db.query_all(stmt).await {
        Ok(r) => r,
        Err(e) => {
            tracing::error!(error = %e, "search_vendors: query failed");
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };

    let vendors: Vec<VendorSearchItem> = rows.into_iter().filter_map(|r| {
        let id: Uuid   = r.try_get("", "id").ok()?;
        let name: String = r.try_get("", "business_name").ok()?;
        Some(VendorSearchItem {
            id,
            business_name: name,
            trade_type:   r.try_get("", "trade_type").ok().flatten(),
            avg_score:    r.try_get("", "avg_score").ok().flatten(),
            review_count: r.try_get("", "review_count").unwrap_or(0),
            bio:          r.try_get("", "bio").ok().flatten(),
            verified:     r.try_get::<String>("", "verified")
                           .map(|v| v == "true")
                           .unwrap_or(false),
        })
    }).collect();

    let total = vendors.len();
    (StatusCode::OK, axum::Json(VendorSearchResponse { vendors, total })).into_response()
}

// ── Public service request (zero-auth) ───────────────────────────────────────

/// POST /api/pub/service-requests
///
/// Zero-auth version for cold-traffic renters.
/// Accepts renter contact info in the body (no session).
/// Fires G35 notification to vendor identical to authenticated version.
///
/// Rate-limit: upstream nginx / tower middleware should enforce per-IP limits.
#[derive(Debug, Deserialize)]
pub struct PublicServiceRequestInput {
    pub vendor_id:    Uuid,
    pub description:  String,
    pub urgency:      Option<String>,   // "not_urgent" | "this_week" | "emergency"
    pub address:      Option<String>,
    pub renter_name:  Option<String>,
    pub renter_email: Option<String>,
    pub renter_phone: Option<String>,
    /// Optional: UTM / referral source so we know how the renter found us
    pub utm_source:   Option<String>,
    /// Optional: pre-fill from /help?vendor_id=X links
    pub asset_id:     Option<Uuid>,
}

#[derive(Debug, Serialize)]
pub struct PublicServiceRequestResponse {
    pub request_id: Uuid,
}

pub async fn create_public_service_request(
    State(db): State<DatabaseConnection>,
    Json(body): Json<PublicServiceRequestInput>,
) -> impl IntoResponse {
    if body.description.trim().is_empty() {
        return (
            StatusCode::UNPROCESSABLE_ENTITY,
            axum::Json(serde_json::json!({"error": "Description is required."})),
        ).into_response();
    }

    let request_id = Uuid::new_v4();
    let urgency    = body.urgency.as_deref().unwrap_or("not_urgent");
    let desc_esc   = body.description.replace('\'', "''");
    let addr_val   = body.address.as_deref()
        .map(|a| format!("'{}'", a.replace('\'', "''")))
        .unwrap_or_else(|| "NULL".to_string());
    let asset_val  = body.asset_id
        .map(|id| format!("'{id}'::uuid"))
        .unwrap_or_else(|| "NULL".to_string());

    // Renter contact info stored in request_metadata JSON
    let meta = serde_json::json!({
        "renter_name":  body.renter_name,
        "renter_email": body.renter_email,
        "renter_phone": body.renter_phone,
        "utm_source":   body.utm_source,
        "auth":         "public",
    });

    let insert_sql = format!(
        "INSERT INTO atlas_service_requests \
         (id, vendor_id, description, urgency, address, asset_id, request_metadata, status, created_at, updated_at) \
         VALUES ('{req}'::uuid, '{vendor}'::uuid, '{desc}', '{urgency}', {addr}, {asset}, \
                 '{meta}'::jsonb, 'pending', NOW(), NOW());",
        req    = request_id,
        vendor = body.vendor_id,
        desc   = desc_esc,
        urgency = urgency,
        addr   = addr_val,
        asset  = asset_val,
        meta   = meta.to_string().replace('\'', "''"),
    );

    if let Err(e) = db.execute(
        Statement::from_string(DatabaseBackend::Postgres, insert_sql)
    ).await {
        tracing::error!(error = %e, vendor_id = %body.vendor_id, "create_public_service_request: insert failed");
        return StatusCode::INTERNAL_SERVER_ERROR.into_response();
    }

    tracing::info!(
        event       = "service_request.created.public",
        request_id  = %request_id,
        vendor_id   = %body.vendor_id,
        utm_source  = ?body.utm_source,
    );

    // G35 notify vendor — identical best-effort pattern as authenticated flow
    let vendor_sql = Statement::from_sql_and_values(
        DatabaseBackend::Postgres,
        "SELECT user_id, tenant_id, business_name FROM atlas_service_providers WHERE id = $1 LIMIT 1",
        [body.vendor_id.into()],
    );

    if let Ok(Some(row)) = db.query_one(vendor_sql).await {
        let v_user:   Option<Uuid> = row.try_get("", "user_id").ok();
        let v_tenant: Option<Uuid> = row.try_get("", "tenant_id").ok();
        let biz_name: String       = row.try_get("", "business_name").unwrap_or_default();

        if let (Some(v_user), Some(v_tenant)) = (v_user, v_tenant) {
            let renter_label = body.renter_name
                .as_deref()
                .unwrap_or("A renter");
            let dispatch = DispatchInput {
                tenant_id:         v_tenant,
                user_id:           v_user,
                notification_type: "service_request_received".to_string(),
                title:             format!("New request from {renter_label}"),
                body:              format!(
                    "{renter_label} needs help: \"{}\"",
                    body.description.chars().take(80).collect::<String>()
                ),
                priority:          NotificationPriority::High,
                entity_type:       Some("atlas_service_requests".to_string()),
                entity_id:         Some(request_id),
                metadata:          Some(serde_json::json!({
                    "request_id":   request_id,
                    "renter_email": body.renter_email,
                    "renter_phone": body.renter_phone,
                    "urgency":      urgency,
                    "utm_source":   body.utm_source,
                    "vendor_name":  biz_name,
                })),
                include_broadcast: false,
            };
            if let Err(e) = NotificationService::dispatch(&db, dispatch).await {
                tracing::warn!(error = %e, vendor_user_id = %v_user, "G35: public service_request notify failed (non-fatal)");
            }
        }
    }

    (StatusCode::CREATED, axum::Json(PublicServiceRequestResponse { request_id })).into_response()
}
