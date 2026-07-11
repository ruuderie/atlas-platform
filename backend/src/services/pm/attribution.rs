//! # G20 AttributionService — Multi-Channel Attribution
//!
//! ## Scope
//!
//! Records every marketing touchpoint on the path to a conversion and
//! distributes revenue credit across those touchpoints according to a
//! configurable attribution model.
//!
//! ## Identity resolution lifecycle
//!
//! ```text
//! 1. Visitor lands anonymously
//!    → capture_touchpoint(anonymous_id="abc123", user_id=None)
//!
//! 2. Visitor submits lead form
//!    → capture_touchpoint(anonymous_id="abc123", contact_email="bob@co.com")
//!
//! 3. Visitor creates account / logs in
//!    → resolve_identity(anonymous_id="abc123", user_id=UUID)
//!       → UPDATE all prior touchpoints for "abc123" with the resolved user_id
//!
//! 4. Visitor makes a booking
//!    → record_conversion(user_id=UUID, entity="atlas_reservations", model=LastTouch)
//!       → find all touchpoints in attribution window
//!       → match on AttributionModel to distribute credit
//!       → write attributed_revenue_cents + conversion_entity_* to each row
//! ```
//!
//! ## Attribution model dispatch
//!
//! `record_conversion()` matches on `AttributionModel` (exhaustive enum match)
//! to select the credit distribution algorithm. Adding a new model variant
//! requires handling it here — the compiler enforces this.
//!
//! | Model | Credit rule |
//! |-------|-------------|
//! | `FirstTouch` | 100% to oldest touchpoint in window |
//! | `LastTouch` | 100% to most recent touchpoint |
//! | `Linear` | Equal share to all touchpoints |
//! | `TimeDecay` | Exponential decay — more credit to recent touchpoints |
//! | `PositionBased` | 40% first + 40% last + 20% split across middle |

use anyhow::{Result, anyhow};
use chrono::Utc;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, QueryOrder, Set,
};
use uuid::Uuid;

use crate::{
    entities::atlas_attribution_touchpoint,
    types::pm::{AttributionChannel, AttributionModel},
};

// ── Input types ───────────────────────────────────────────────────────────────

/// UTM parameters captured from the request URL. All fields are optional —
/// pass what is available; missing fields are stored as NULL.
#[derive(Debug, Clone, Default, serde::Deserialize)]
pub struct UtmParams {
    pub utm_source: Option<String>,
    pub utm_medium: Option<String>,
    pub utm_campaign: Option<String>,
    pub utm_content: Option<String>,
    pub utm_term: Option<String>,
}

/// Full context for recording a touchpoint.
#[derive(Debug, Clone, serde::Deserialize)]
pub struct CapturePayload {
    pub channel: AttributionChannel,
    pub utm: UtmParams,
    /// Set when the visitor is a known platform user.
    pub user_id: Option<Uuid>,
    /// Email from a form fill (if visitor not yet a user).
    pub contact_email: Option<String>,
    /// Client-side anonymous ID (cookie / device fingerprint).
    pub anonymous_id: Option<String>,
    /// Campaign that drove this touchpoint (if trackable).
    pub campaign_id: Option<Uuid>,
    pub enrollment_id: Option<Uuid>,
    /// Event ID (G21) if this was an event-driven touchpoint.
    pub event_id: Option<Uuid>,
    pub landing_page_url: Option<String>,
    pub referrer_url: Option<String>,
}

/// Payload for recording a conversion against tracked touchpoints.
#[derive(Debug, Clone)]
pub struct ConversionPayload {
    /// Who converted — use the resolved user_id when available.
    pub user_id: Option<Uuid>,
    /// Fallback identity when user_id is not yet resolved.
    pub contact_email: Option<String>,
    /// What was converted (e.g. "atlas_reservations").
    pub conversion_entity_type: String,
    pub conversion_entity_id: Uuid,
    /// GMV of the conversion in cents (used for credit calculation).
    pub conversion_value_cents: i64,
    /// How to distribute credit across the touchpoints in the window.
    pub model: AttributionModel,
    /// How far back to look for touchpoints (in days). Defaults to 30.
    pub attribution_window_days: Option<i64>,
}

// ── Service ───────────────────────────────────────────────────────────────────

pub struct AttributionService;

impl AttributionService {
    // ── Touchpoint capture ────────────────────────────────────────────────────

    /// Record a marketing touchpoint for a visitor. At least one of
    /// `user_id`, `contact_email`, or `anonymous_id` must be present.
    pub async fn capture_touchpoint(
        db: &DatabaseConnection,
        tenant_id: Uuid,
        payload: CapturePayload,
    ) -> Result<atlas_attribution_touchpoint::Model> {
        if payload.user_id.is_none()
            && payload.contact_email.is_none()
            && payload.anonymous_id.is_none()
        {
            return Err(anyhow!(
                "capture_touchpoint requires at least one identity field: user_id, contact_email, or anonymous_id"
            ));
        }

        let active = atlas_attribution_touchpoint::ActiveModel {
            id: Set(Uuid::new_v4()),
            tenant_id: Set(tenant_id),
            user_id: Set(payload.user_id),
            contact_email: Set(payload.contact_email),
            anonymous_id: Set(payload.anonymous_id),
            channel: Set(payload.channel.to_string()),
            utm_source: Set(payload.utm.utm_source),
            utm_medium: Set(payload.utm.utm_medium),
            utm_campaign: Set(payload.utm.utm_campaign),
            utm_content: Set(payload.utm.utm_content),
            utm_term: Set(payload.utm.utm_term),
            campaign_id: Set(payload.campaign_id),
            enrollment_id: Set(payload.enrollment_id),
            event_id: Set(payload.event_id),
            conversion_entity_type: Set(None),
            conversion_entity_id: Set(None),
            conversion_value_cents: Set(None),
            attributed_revenue_cents: Set(None),
            attribution_model: Set(None),
            landing_page_url: Set(payload.landing_page_url),
            referrer_url: Set(payload.referrer_url),
            occurred_at: Set(Utc::now()),
        };

        let tp = active.insert(db).await?;

        tracing::debug!(
            %tenant_id, touchpoint_id = %tp.id,
            channel = %tp.channel,
            "AttributionService::capture_touchpoint: recorded"
        );

        Ok(tp)
    }

    // ── Identity resolution ───────────────────────────────────────────────────

    /// Back-fill all prior touchpoints for `anonymous_id` with the resolved
    /// `user_id`. Called when an anonymous visitor creates an account or logs in.
    ///
    /// Returns the number of rows updated.
    pub async fn resolve_identity(
        db: &DatabaseConnection,
        tenant_id: Uuid,
        anonymous_id: &str,
        user_id: Uuid,
    ) -> Result<u64> {
        use sea_orm::ConnectionTrait;

        let result = db
            .execute_unprepared(&format!(
                "UPDATE atlas_attribution_touchpoints
                 SET user_id = '{user_id}'
                 WHERE tenant_id = '{tenant_id}'
                   AND anonymous_id = '{anonymous_id}'
                   AND user_id IS NULL"
            ))
            .await
            .map_err(|e| anyhow!("resolve_identity failed: {e:#}"))?;

        tracing::info!(
            %tenant_id, %user_id, %anonymous_id,
            rows_updated = result.rows_affected(),
            "AttributionService::resolve_identity: identity resolved"
        );

        Ok(result.rows_affected())
    }

    // ── Conversion recording ──────────────────────────────────────────────────

    /// Find all touchpoints for the visitor within the attribution window and
    /// distribute `conversion_value_cents` credit across them according to
    /// the requested `AttributionModel`.
    ///
    /// Returns the IDs of all credited touchpoints.
    ///
    /// ## Model dispatch
    ///
    /// The match on `AttributionModel` is exhaustive — adding a new model
    /// variant will cause a compiler error here, ensuring it is always handled.
    pub async fn record_conversion(
        db: &DatabaseConnection,
        tenant_id: Uuid,
        payload: ConversionPayload,
    ) -> Result<Vec<Uuid>> {
        let window_days = payload.attribution_window_days.unwrap_or(30);
        let cutoff = Utc::now() - chrono::Duration::days(window_days);

        // Fetch all touchpoints for this visitor within the window.
        let mut q = atlas_attribution_touchpoint::Entity::find()
            .filter(atlas_attribution_touchpoint::Column::TenantId.eq(tenant_id))
            .filter(atlas_attribution_touchpoint::Column::OccurredAt.gte(cutoff))
            .filter(atlas_attribution_touchpoint::Column::ConversionEntityId.is_null())
            .order_by_asc(atlas_attribution_touchpoint::Column::OccurredAt);

        // Filter by visitor identity (try user_id first, fall back to email).
        if let Some(uid) = payload.user_id {
            q = q.filter(atlas_attribution_touchpoint::Column::UserId.eq(uid));
        } else if let Some(ref email) = payload.contact_email {
            q = q.filter(atlas_attribution_touchpoint::Column::ContactEmail.eq(email.as_str()));
        } else {
            return Err(anyhow!(
                "record_conversion requires user_id or contact_email"
            ));
        }

        let touchpoints = q.all(db).await?;

        if touchpoints.is_empty() {
            tracing::warn!(
                %tenant_id, entity_type = %payload.conversion_entity_type,
                entity_id = %payload.conversion_entity_id,
                "AttributionService::record_conversion: no touchpoints found in window"
            );
            return Ok(vec![]);
        }

        let n = touchpoints.len();
        let total = payload.conversion_value_cents;

        // ── Attribution model dispatch ─────────────────────────────────────────
        //
        // Match on AttributionModel to compute per-touchpoint credit in cents.
        // The vector index corresponds to the sorted touchpoint vector (oldest first).
        // Exhaustive match — compiler will reject unhandled new variants.

        let credits: Vec<i64> = match payload.model {
            // 100% to the first (oldest) touchpoint.
            AttributionModel::FirstTouch => {
                let mut v = vec![0i64; n];
                v[0] = total;
                v
            }

            // 100% to the last (most recent) touchpoint.
            AttributionModel::LastTouch => {
                let mut v = vec![0i64; n];
                v[n - 1] = total;
                v
            }

            // Equal share — integer division, remainder to last touchpoint.
            AttributionModel::Linear => {
                let share = total / n as i64;
                let remainder = total - share * n as i64;
                let mut v = vec![share; n];
                v[n - 1] += remainder;
                v
            }

            // Exponential decay: weight_i = 0.5^(n-1-i), normalized to sum=1.
            // More credit to recent touchpoints (higher index = more recent).
            AttributionModel::TimeDecay => {
                let weights: Vec<f64> = (0..n).map(|i| 0.5f64.powi((n - 1 - i) as i32)).collect();
                let weight_sum: f64 = weights.iter().sum();
                let mut v: Vec<i64> = weights
                    .iter()
                    .map(|w| ((w / weight_sum) * total as f64).round() as i64)
                    .collect();
                // Reconcile rounding error against last touchpoint.
                let diff = total - v.iter().sum::<i64>();
                *v.last_mut().unwrap() += diff;
                v
            }

            // 40% first, 40% last, 20% split evenly across middle.
            AttributionModel::PositionBased => {
                if n == 1 {
                    vec![total]
                } else if n == 2 {
                    let half = total / 2;
                    vec![half, total - half]
                } else {
                    let first = (total as f64 * 0.4).round() as i64;
                    let last = (total as f64 * 0.4).round() as i64;
                    let middle_total = total - first - last;
                    let middle_n = n - 2;
                    let middle_share = middle_total / middle_n as i64;
                    let middle_rem = middle_total - middle_share * middle_n as i64;
                    let mut v = vec![middle_share; n];
                    v[0] = first;
                    v[n - 1] = last;
                    v[n - 2] += middle_rem;
                    v
                }
            }
        };

        // Write credit back to each touchpoint row.
        let model_str = payload.model.to_string();
        let mut credited_ids = Vec::with_capacity(n);

        for (tp, credit) in touchpoints.iter().zip(credits.iter()) {
            let mut active: atlas_attribution_touchpoint::ActiveModel = tp.clone().into();
            active.conversion_entity_type = Set(Some(payload.conversion_entity_type.clone()));
            active.conversion_entity_id = Set(Some(payload.conversion_entity_id));
            active.conversion_value_cents = Set(Some(total));
            active.attributed_revenue_cents = Set(Some(*credit));
            active.attribution_model = Set(Some(model_str.clone()));
            active.update(db).await?;
            credited_ids.push(tp.id);
        }

        tracing::info!(
            %tenant_id,
            entity_type = %payload.conversion_entity_type,
            entity_id = %payload.conversion_entity_id,
            model = %model_str,
            touchpoints_credited = n,
            total_cents = total,
            "AttributionService::record_conversion: credit distributed"
        );

        Ok(credited_ids)
    }

    // ── Conversion path lookup ────────────────────────────────────────────────

    /// Return the full ordered touchpoint path that led to a specific conversion.
    /// Used for the "conversion path" report: "Show me every touchpoint that
    /// influenced booking X."
    pub async fn get_conversion_path(
        db: &DatabaseConnection,
        tenant_id: Uuid,
        conversion_entity_id: Uuid,
    ) -> Result<Vec<atlas_attribution_touchpoint::Model>> {
        Ok(atlas_attribution_touchpoint::Entity::find()
            .filter(atlas_attribution_touchpoint::Column::TenantId.eq(tenant_id))
            .filter(
                atlas_attribution_touchpoint::Column::ConversionEntityId.eq(conversion_entity_id),
            )
            .order_by_asc(atlas_attribution_touchpoint::Column::OccurredAt)
            .all(db)
            .await?)
    }

    /// Return all touchpoints for a user, newest first (their full journey).
    pub async fn get_user_journey(
        db: &DatabaseConnection,
        tenant_id: Uuid,
        user_id: Uuid,
    ) -> Result<Vec<atlas_attribution_touchpoint::Model>> {
        Ok(atlas_attribution_touchpoint::Entity::find()
            .filter(atlas_attribution_touchpoint::Column::TenantId.eq(tenant_id))
            .filter(atlas_attribution_touchpoint::Column::UserId.eq(user_id))
            .order_by_desc(atlas_attribution_touchpoint::Column::OccurredAt)
            .all(db)
            .await?)
    }

    /// Return all touchpoints driven by a specific campaign.
    pub async fn get_campaign_touchpoints(
        db: &DatabaseConnection,
        tenant_id: Uuid,
        campaign_id: Uuid,
    ) -> Result<Vec<atlas_attribution_touchpoint::Model>> {
        Ok(atlas_attribution_touchpoint::Entity::find()
            .filter(atlas_attribution_touchpoint::Column::TenantId.eq(tenant_id))
            .filter(atlas_attribution_touchpoint::Column::CampaignId.eq(campaign_id))
            .order_by_desc(atlas_attribution_touchpoint::Column::OccurredAt)
            .all(db)
            .await?)
    }
}
