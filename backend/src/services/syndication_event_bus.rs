//! G-05 Syndication Event Bus
//!
//! Implements the transactional outbox pattern for all outbound syndication
//! events fired from a Folio (or any Atlas) app instance to linked Network
//! Instance deployments.
//!
//! # Architecture
//!
//! ```text
//!  ┌─────────────────────────────────────────────────────────────────┐
//!  │  App Handler (e.g. publish_listing)                             │
//!  │                                                                 │
//!  │  BEGIN TRANSACTION                                              │
//!  │    UPDATE atlas_listing SET status = 'published' …             │
//!  │    SyndicationEventBus::enqueue(tx, …)  ←── writes outbox row  │
//!  │  COMMIT                                                         │
//!  └─────────────────────────────────────────────────────────────────┘
//!           │
//!           ▼  (separate process, polls every 10 s)
//!  ┌─────────────────────────────────────────────────────────────────┐
//!  │  SyndicationEventBus::start_worker(db)                          │
//!  │                                                                 │
//!  │  loop:                                                          │
//!  │    SELECT pending rows WHERE next_attempt_at <= now()           │
//!  │    For each row:                                                │
//!  │      1. Mark processing                                         │
//!  │      2. Load atlas_app_instance_syndication to get NI URL       │
//!  │      3. POST payload to ni.inbound_webhook_url                  │
//!  │      4. On success → mark delivered, write integration_event    │
//!  │      5. On failure → exponential back-off, increment retry      │
//!  │                      mark failed if retry_count >= MAX          │
//!  └─────────────────────────────────────────────────────────────────┘
//! ```
//!
//! # Dead-letter
//! Rows with `status = 'failed'` can be replayed via
//! `SyndicationEventBus::replay(db, outbox_id)` from the platform-admin API.

use std::time::Duration;

use chrono::Utc;
use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter,
    QueryOrder, QuerySelect,
};
use serde_json::json;
use uuid::Uuid;

use crate::entities::{
    atlas_app_instance_syndication, atlas_integration_events,
    atlas_syndication_outbox::{self, MAX_RETRY_COUNT},
};

/// Back-off schedule (seconds) indexed by retry_count (0-based).
/// Retries: immediate → 30s → 2m → 10m → 1h → dead-letter
const BACKOFF_SECS: [u64; 5] = [0, 30, 120, 600, 3600];

// ── Public API ────────────────────────────────────────────────────────────────

pub struct SyndicationEventBus;

impl SyndicationEventBus {
    // ── Enqueue ───────────────────────────────────────────────────────────────

    /// Enqueue a syndication event for all **active** links on the given
    /// source app instance.  Call this inside the same DB transaction that
    /// mutates the entity being syndicated so delivery is atomic.
    ///
    /// # Arguments
    /// * `db`               – active `DatabaseConnection` (or transaction)
    /// * `source_config_id` – UUID of the `atlas_app_deployment_config` row for the source app
    /// * `event_type`       – one of the constants in `atlas_syndication_outbox::event_type`
    /// * `entity_id`        – UUID of the entity (listing / asset / inquiry …)
    /// * `entity_type`      – human-readable type label ("listing", "asset", …)
    /// * `data`             – arbitrary JSON payload (e.g. `serde_json::to_value(&listing)?`)
    pub async fn enqueue(
        db: &DatabaseConnection,
        source_config_id: Uuid,
        event_type: &str,
        entity_id: Uuid,
        entity_type: &str,
        data: serde_json::Value,
    ) -> Result<Vec<Uuid>, sea_orm::DbErr> {
        // Fetch all active links for this source instance
        let links = atlas_app_instance_syndication::Entity::find()
            .filter(atlas_app_instance_syndication::Column::SourceConfigId.eq(source_config_id))
            .filter(atlas_app_instance_syndication::Column::Status.eq("active"))
            .all(db)
            .await?;

        let mut enqueued = Vec::with_capacity(links.len());

        for link in links {
            // Skip links with no inbound webhook URL configured on the NI side
            if link.inbound_webhook_url.is_none() {
                continue;
            }

            let payload = json!({
                "event_type": event_type,
                "entity_id":  entity_id,
                "entity_type": entity_type,
                "source_config_id": source_config_id,
                "link_id":    link.id,
                "data":       data.clone(),
                "timestamp":  Utc::now().to_rfc3339(),
            });

            let row = atlas_syndication_outbox::ActiveModel {
                id: Set(Uuid::new_v4()),
                link_id: Set(link.id),
                source_config_id: Set(source_config_id),
                event_type: Set(event_type.to_string()),
                payload: Set(payload),
                status: Set("pending".to_string()),
                retry_count: Set(0),
                last_http_status: Set(None),
                last_error: Set(None),
                next_attempt_at: Set(Utc::now().into()),
                created_at: Set(Utc::now().into()),
                updated_at: Set(Utc::now().into()),
            };

            let inserted = row.insert(db).await?;
            enqueued.push(inserted.id);
        }

        tracing::debug!(
            source_config_id = %source_config_id,
            event_type,
            enqueued = enqueued.len(),
            "syndication outbox: enqueued events",
        );

        Ok(enqueued)
    }

    // ── Worker ────────────────────────────────────────────────────────────────

    /// Spawn the background worker that polls `atlas_syndication_outbox`
    /// and dispatches events to NI webhook URLs.  Called once at startup.
    pub async fn start_worker(db: DatabaseConnection) {
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(10));
            loop {
                interval.tick().await;
                if let Err(e) = Self::process_pending(&db).await {
                    tracing::error!("syndication_event_bus: worker error: {e:#}");
                }
            }
        });
        tracing::info!("syndication_event_bus: worker started (10s poll interval)");
    }

    // ── Replay ────────────────────────────────────────────────────────────────

    /// Reset a dead-letter row back to `pending` so the worker picks it up
    /// again.  Exposed via `POST /api/admin/syndication/outbox/{id}/replay`.
    pub async fn replay(db: &DatabaseConnection, outbox_id: Uuid) -> Result<(), sea_orm::DbErr> {
        let existing = match atlas_syndication_outbox::Entity::find_by_id(outbox_id)
            .one(db)
            .await?
        {
            Some(r) => r,
            None => return Err(sea_orm::DbErr::RecordNotFound(outbox_id.to_string())),
        };

        let mut active: atlas_syndication_outbox::ActiveModel = existing.into();
        active.status = Set("pending".to_string());
        active.retry_count = Set(0);
        active.last_error = Set(None);
        active.next_attempt_at = Set(Utc::now().into());
        active.update(db).await?;

        tracing::info!(outbox_id = %outbox_id, "syndication outbox: row replayed");
        Ok(())
    }

    // ── Internal dispatch loop ────────────────────────────────────────────────

    async fn process_pending(db: &DatabaseConnection) -> Result<(), sea_orm::DbErr> {
        let now = Utc::now();

        // Grab up to 50 pending rows whose next_attempt_at is due
        let now_str = now.to_rfc3339();
        let rows = atlas_syndication_outbox::Entity::find()
            .filter(atlas_syndication_outbox::Column::Status.eq("pending"))
            .filter(sea_orm::Condition::all().add(
                sea_orm::sea_query::Expr::col(atlas_syndication_outbox::Column::NextAttemptAt).lte(
                    sea_orm::sea_query::Expr::cust(format!("'{}'::timestamptz", now_str)),
                ),
            ))
            .order_by_asc(atlas_syndication_outbox::Column::NextAttemptAt)
            .limit(50)
            .all(db)
            .await?;

        for row in rows {
            // Mark in-flight
            let mut active: atlas_syndication_outbox::ActiveModel = row.clone().into();
            active.status = Set("processing".to_string());
            active.clone().update(db).await?;

            // Resolve the NI webhook URL from the link
            let link = match atlas_app_instance_syndication::Entity::find_by_id(row.link_id)
                .one(db)
                .await?
            {
                Some(l) => l,
                None => {
                    // Link was deleted — mark as skipped
                    Self::mark_failed(db, &row, "link_deleted", None, None).await?;
                    continue;
                }
            };

            let webhook_url = match link.inbound_webhook_url {
                Some(ref url) => url.clone(),
                None => {
                    Self::mark_failed(db, &row, "no_webhook_url", None, None).await?;
                    continue;
                }
            };

            // Build HMAC-SHA256 signature header if secret is configured
            let attempt_start = std::time::Instant::now();
            let payload_bytes = serde_json::to_vec(&row.payload).unwrap_or_default();

            let sig_header = link.inbound_webhook_secret.as_deref().map(|secret| {
                use hmac::{Hmac, Mac};
                use sha2::Sha256;
                type HmacSha256 = Hmac<Sha256>;
                let mut mac = HmacSha256::new_from_slice(secret.as_bytes())
                    .expect("HMAC accepts any key size");
                mac.update(&payload_bytes);
                format!("sha256={}", hex::encode(mac.finalize().into_bytes()))
            });

            // Dispatch HTTP POST
            let client = reqwest::Client::builder()
                .timeout(Duration::from_secs(15))
                .build()
                .unwrap_or_default();

            let mut req = client
                .post(&webhook_url)
                .header("Content-Type", "application/json")
                .header("X-Atlas-Event", &row.event_type)
                .header("X-Atlas-Delivery", row.id.to_string());

            if let Some(sig) = sig_header {
                req = req.header("X-Atlas-Signature-256", sig);
            }

            let result = req.body(payload_bytes).send().await;
            let latency_ms = attempt_start.elapsed().as_millis() as i32;

            match result {
                Ok(resp) => {
                    let status_code = resp.status().as_u16() as i32;
                    let body = resp.text().await.unwrap_or_default();
                    let truncated = body.chars().take(2048).collect::<String>();

                    if status_code < 300 {
                        // Successful delivery
                        let mut a: atlas_syndication_outbox::ActiveModel = row.clone().into();
                        a.status = Set("delivered".to_string());
                        a.last_http_status = Set(Some(status_code));
                        a.last_error = Set(None);
                        a.update(db).await?;

                        Self::log_event(
                            db,
                            &row,
                            "success",
                            Some(status_code),
                            Some(truncated),
                            latency_ms,
                        )
                        .await?;
                    } else {
                        Self::handle_retry(
                            db,
                            &row,
                            Some(status_code),
                            &format!("HTTP {status_code}: {truncated}"),
                            latency_ms,
                        )
                        .await?;
                    }
                }
                Err(e) => {
                    Self::handle_retry(db, &row, None, &format!("request error: {e}"), latency_ms)
                        .await?;
                }
            }
        }

        Ok(())
    }

    async fn handle_retry(
        db: &DatabaseConnection,
        row: &atlas_syndication_outbox::Model,
        http_status: Option<i32>,
        error: &str,
        latency_ms: i32,
    ) -> Result<(), sea_orm::DbErr> {
        let new_retry = row.retry_count + 1;
        let (new_status, next_attempt) = if new_retry >= MAX_RETRY_COUNT {
            (
                "failed".to_string(),
                // Use year 9999 as a sentinel "never retry" timestamp
                chrono::DateTime::parse_from_rfc3339("9999-12-31T23:59:59+00:00")
                    .unwrap()
                    .into(),
            )
        } else {
            let backoff_idx = (new_retry as usize).min(BACKOFF_SECS.len() - 1);
            let secs = BACKOFF_SECS[backoff_idx];
            (
                "pending".to_string(),
                (Utc::now() + chrono::Duration::seconds(secs as i64)).into(),
            )
        };

        let mut a: atlas_syndication_outbox::ActiveModel = row.clone().into();
        a.status = Set(new_status.clone());
        a.retry_count = Set(new_retry);
        a.last_http_status = Set(http_status);
        a.last_error = Set(Some(error.chars().take(1024).collect()));
        a.next_attempt_at = Set(next_attempt);
        a.update(db).await?;

        let outcome = if new_status == "failed" {
            "failed"
        } else {
            "failed"
        }; // logged as failed either way
        Self::log_event(
            db,
            row,
            outcome,
            http_status,
            Some(error.to_string()),
            latency_ms,
        )
        .await?;

        if new_status == "failed" {
            tracing::warn!(
                outbox_id = %row.id,
                link_id   = %row.link_id,
                event_type = %row.event_type,
                retry_count = new_retry,
                "syndication outbox: dead-letter after {} attempts", new_retry,
            );
        }

        Ok(())
    }

    async fn mark_failed(
        db: &DatabaseConnection,
        row: &atlas_syndication_outbox::Model,
        reason: &str,
        http_status: Option<i32>,
        latency_ms: Option<i32>,
    ) -> Result<(), sea_orm::DbErr> {
        let mut a: atlas_syndication_outbox::ActiveModel = row.clone().into();
        a.status = Set("failed".to_string());
        a.last_error = Set(Some(reason.to_string()));
        a.update(db).await?;
        Self::log_event(
            db,
            row,
            "skipped",
            http_status,
            Some(reason.to_string()),
            latency_ms.unwrap_or(0),
        )
        .await
    }

    async fn log_event(
        db: &DatabaseConnection,
        row: &atlas_syndication_outbox::Model,
        outcome: &str,
        http_status: Option<i32>,
        response_body: Option<String>,
        latency_ms: i32,
    ) -> Result<(), sea_orm::DbErr> {
        let ev = atlas_integration_events::ActiveModel {
            id: Set(Uuid::new_v4()),
            outbox_id: Set(Some(row.id)),
            link_id: Set(row.link_id),
            source_config_id: Set(row.source_config_id),
            event_type: Set(row.event_type.clone()),
            direction: Set("outbound".to_string()),
            outcome: Set(outcome.to_string()),
            http_status: Set(http_status),
            response_body: Set(response_body.map(|b| b.chars().take(2048).collect())),
            latency_ms: Set(Some(latency_ms)),
            attempt_number: Set(row.retry_count + 1),
            created_at: Set(Utc::now().into()),
        };
        ev.insert(db).await?;
        Ok(())
    }
}
