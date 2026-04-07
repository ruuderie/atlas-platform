use crate::entities::{webhook_delivery, webhook_endpoint};
use uuid::Uuid;
use sea_orm::prelude::DateTimeWithTimeZone;
use chrono::Utc;
use hmac::{Hmac, Mac};
use reqwest::{Client, StatusCode};
use sea_orm::*;
use sea_orm::ActiveValue::Set;
use serde_json::Value;
use sha2::Sha256;
use std::time::Duration;
use tracing::{error, info, warn};

pub async fn dispatch_event(
    db: &DatabaseConnection,
    tenant_id: Uuid,
    event_type: &str,
    payload: Value,
) -> Result<(), DbErr> {
    // Find active endpoints subscribed to this event
    let endpoints = webhook_endpoint::Entity::find()
        .filter(webhook_endpoint::Column::TenantId.eq(tenant_id))
        .filter(webhook_endpoint::Column::IsActive.eq(true))
        .all(db)
        .await?;

    for endpoint in endpoints {
        // Basic check: is the event_type in the subscribed_events JSON array?
        // (Assuming subscribed_events is an array of strings e.g. ["crm.deal.won", "listing.created"])
        let subscriptions = endpoint.subscribed_events.as_array();
        let is_subscribed = if let Some(subs) = subscriptions {
            subs.iter().any(|v| v.as_str() == Some(event_type))
        } else {
            false
        };

        if !is_subscribed {
            continue;
        }

        // Create webhook delivery
        let delivery = webhook_delivery::ActiveModel {
            id: Set(Uuid::new_v4()),
            endpoint_id: Set(endpoint.id),
            tenant_id: Set(tenant_id),
            event_type: Set(event_type.to_string()),
            payload: Set(payload.clone()),
            status: Set("pending".to_string()),
            next_retry_at: Set(None),
            attempts: Set(0),
            ..Default::default()
        }
        .insert(db)
        .await?;

        // 1. The "Spawn" (Happy Path)
        let db_clone = db.clone();
        let delivery_id = delivery.id;

        tokio::spawn(async move {
            if let Err(e) = process_delivery(&db_clone, delivery_id).await {
                error!("Failed to process webhook delivery {}: {:?}", delivery_id, e);
            }
        });
    }

    Ok(())
}

async fn process_delivery(db: &DatabaseConnection, delivery_id: Uuid) -> Result<(), anyhow::Error> {
    // Fetch delivery and endpoint
    let delivery = webhook_delivery::Entity::find_by_id(delivery_id)
        .one(db)
        .await?
        .ok_or_else(|| anyhow::anyhow!("Delivery not found"))?;

    let endpoint = webhook_endpoint::Entity::find_by_id(delivery.endpoint_id)
        .one(db)
        .await?
        .ok_or_else(|| anyhow::anyhow!("Endpoint not found"))?;

    let payload_str = serde_json::to_string(&delivery.payload)?;
    
    // Create HMAC signature
    let mut mac = Hmac::<Sha256>::new_from_slice(endpoint.secret_key.as_bytes())?;
    mac.update(payload_str.as_bytes());
    let result = mac.finalize();
    let signature = hex::encode(result.into_bytes());

    let client = Client::new();
    let response = client
        .post(&endpoint.target_url)
        .header("Content-Type", "application/json")
        .header("X-Atlas-Signature", signature)
        .header("X-Atlas-Event", &delivery.event_type)
        .body(payload_str)
        .timeout(Duration::from_secs(10))
        .send()
        .await;

    let mut active_delivery: webhook_delivery::ActiveModel = delivery.into();
    active_delivery.attempts = Set(active_delivery.attempts.clone().unwrap() + 1);

    match response {
        Ok(resp) => {
            let status_code = resp.status();
            active_delivery.response_status = Set(Some(status_code.as_u16() as i32));

            // Log response body text if available
            let body_text = resp.text().await.unwrap_or_default();
            active_delivery.response_body = Set(Some(body_text.chars().take(2000).collect())); // truncate if too long

            if status_code.is_success() {
                active_delivery.status = Set("sent".to_string());
                active_delivery.next_retry_at = Set(None);
                info!("Webhook {} delivered successfully", delivery_id);
            } else {
                handle_failure(&mut active_delivery);
                warn!("Webhook {} failed with status {}", delivery_id, status_code);
            }
        }
        Err(e) => {
            active_delivery.status = Set("failed".to_string());
            active_delivery.response_body = Set(Some(e.to_string()));
            handle_failure(&mut active_delivery);
            warn!("Webhook {} failed request: {:?}", delivery_id, e);
        }
    }

    active_delivery.updated_at = Set(Some(Utc::now().into()));
    active_delivery.update(db).await?;

    Ok(())
}

fn handle_failure(delivery: &mut webhook_delivery::ActiveModel) {
    let attempts = delivery.attempts.clone().unwrap();
    if attempts >= 5 {
        delivery.status = Set("failed".to_string());
        delivery.next_retry_at = Set(None); // no more retries
    } else {
        delivery.status = Set("failed".to_string());
        // Exponential backoff: e.g. 5m, 25m, 125m, etc. 
        // 5 minutes * 5 ^ (attempt - 1)
        let delay_mins = 5_i64.pow(attempts as u32 - 1);
        let next_retry = Utc::now() + chrono::Duration::minutes(delay_mins);
        delivery.next_retry_at = Set(Some(next_retry.into()));
    }
}

// 2. The "Sweeper" (Recovery Path)
pub async fn start_webhook_sweeper(db: DatabaseConnection) {
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_secs(60));
        loop {
            interval.tick().await;

            let now: DateTimeWithTimeZone = Utc::now().into();
            
            // Find pendings older than 2 mins
            let two_mins_ago: DateTimeWithTimeZone = (Utc::now() - chrono::Duration::minutes(2)).into();
            let pendings = webhook_delivery::Entity::find()
                .filter(webhook_delivery::Column::Status.eq("pending"))
                .filter(webhook_delivery::Column::UpdatedAt.lt(two_mins_ago))
                .all(&db)
                .await
                .unwrap_or_default();

            for delivery in pendings {
                let db_clone = db.clone();
                let delivery_id = delivery.id;
                tokio::spawn(async move {
                    let _ = process_delivery(&db_clone, delivery_id).await;
                });
            }

            // Find failed that are due for retry
            let due_for_retry = webhook_delivery::Entity::find()
                .filter(webhook_delivery::Column::Status.eq("failed"))
                .filter(webhook_delivery::Column::NextRetryAt.lte(now))
                .filter(webhook_delivery::Column::NextRetryAt.is_not_null())
                .all(&db)
                .await
                .unwrap_or_default();

            for delivery in due_for_retry {
                let db_clone = db.clone();
                let delivery_id = delivery.id;
                tokio::spawn(async move {
                    let _ = process_delivery(&db_clone, delivery_id).await;
                });
            }
        }
    });
}
