use crate::entities::{platform_metrics_daily, telemetry_events};
use sea_orm::{
    ActiveModelTrait, ColumnTrait, ConnectionTrait, DatabaseConnection, EntityTrait, QueryFilter, Set, DatabaseTransaction, TransactionTrait
};
use chrono::{Utc, NaiveDate, Datelike};
use sea_orm::{sea_query::OnConflict, Condition};
use serde_json::Value;
use std::collections::HashMap;
use uuid::Uuid;

pub struct TelemetryService;

impl TelemetryService {
    /// Non-blocking telemetry ingestion
    pub fn log_event(
        db: DatabaseConnection,
        tenant_id: Uuid,
        event_source: String,
        event_type: String,
        event_payload: Option<Value>,
    ) {
        tokio::spawn(async move {
            let active_model = telemetry_events::ActiveModel {
                id: Set(Uuid::new_v4()),
                tenant_id: Set(tenant_id),
                event_source: Set(event_source),
                event_type: Set(event_type),
                event_payload: Set(event_payload),
                timestamp: Set(Utc::now()),
                processed: Set(false),
            };

            if let Err(e) = active_model.insert(&db).await {
                tracing::error!("Failed to asynchronously insert telemetry event: {}", e);
            }
        });
    }

    /// Background cron task to aggregate events into daily KPIs
    pub async fn process_daily_metrics(db: &DatabaseConnection) -> Result<(), sea_orm::DbErr> {
        let txn = db.begin().await?;

        // 1. Fetch unprocessed events
        let unprocessed = telemetry_events::Entity::find()
            .filter(telemetry_events::Column::Processed.eq(false))
            .all(&txn)
            .await?;

        if unprocessed.is_empty() {
            txn.commit().await?;
            return Ok(());
        }

        // 2. Aggregate
        // Key: (Date, TenantId, MetricSource, MetricKey) -> Value
        let mut aggregations: HashMap<(NaiveDate, Uuid, String, String), f32> = HashMap::new();

        let mut processed_ids = Vec::new();

        for event in unprocessed {
            processed_ids.push(event.id);
            let date = event.timestamp.date_naive();
            
            // Basic aggregation: count event occurrence
            // The metric key will be identical to the event_type for generic counts
            let count_key = (
                date,
                event.tenant_id,
                event.event_source.clone(),
                event.event_type.clone(),
            );
            *aggregations.entry(count_key).or_insert(0.0) += 1.0;

            // Optional: business KPI specific parsing based on event_payload
            // For example, if it's "subscription_created" and has "mrr" value
            if let Some(payload) = &event.event_payload {
                if event.event_type == "subscription_created" || event.event_type == "subscription_upgraded" {
                    if let Some(mrr_val) = payload.get("mrr").and_then(|v| v.as_f64()) {
                        let mrr_key = (
                            date,
                            event.tenant_id,
                            event.event_source.clone(),
                            "mrr".to_string(),
                        );
                        *aggregations.entry(mrr_key).or_insert(0.0) += mrr_val as f32;
                    }
                }
            }
        }

        // 3. Upsert aggregations into PlatformMetricsDaily
        let mut inserts = Vec::new();
        for ((date, tenant_id, metric_source, metric_key), value) in aggregations {
            inserts.push(platform_metrics_daily::ActiveModel {
                id: Set(Uuid::new_v4()),
                date: Set(date),
                tenant_id: Set(tenant_id),
                metric_source: Set(metric_source),
                metric_key: Set(metric_key),
                metric_value: Set(value),
            });
        }

        // Execute bulk upsert (ON CONFLICT DO UPDATE)
        if !inserts.is_empty() {
            platform_metrics_daily::Entity::insert_many(inserts)
                .on_conflict(
                    OnConflict::columns([
                        platform_metrics_daily::Column::Date,
                        platform_metrics_daily::Column::TenantId,
                        platform_metrics_daily::Column::MetricSource,
                        platform_metrics_daily::Column::MetricKey,
                    ])
                    .update_expr(
                        platform_metrics_daily::Column::MetricValue,
                        sea_orm::sea_query::Expr::col(platform_metrics_daily::Column::MetricValue)
                            .add(sea_orm::sea_query::Expr::cust("EXCLUDED.metric_value"))
                    )
                    .to_owned()
                )
                .exec(&txn)
                .await?;
        }

        // 4. Mark events as processed
        telemetry_events::Entity::update_many()
            .col_expr(telemetry_events::Column::Processed, sea_orm::sea_query::Expr::value(true))
            .filter(telemetry_events::Column::Id.is_in(processed_ids))
            .exec(&txn)
            .await?;

        txn.commit().await?;
        Ok(())
    }
}
