use crate::entities::outbox_job;
use sea_orm::{
    ActiveModelTrait, DatabaseConnection, EntityTrait, Set
};
use chrono::Utc;
use std::time::Duration;
use tracing::{info, error};

pub struct OutboxWorker;

impl OutboxWorker {
    pub async fn start_worker(db: DatabaseConnection) {
        tokio::spawn(async move {
            info!("Starting OutboxWorker background worker.");
            let mut interval = tokio::time::interval(Duration::from_millis(1500)); // Tick every 1.5s
            
            loop {
                interval.tick().await;
                if let Err(e) = Self::process_next_job(&db).await {
                    error!("OutboxWorker encountered an error: {}", e);
                }
            }
        });
    }

    pub async fn process_next_job(db: &DatabaseConnection) -> Result<(), sea_orm::DbErr> {
        let hostname = std::env::var("HOSTNAME").unwrap_or_else(|_| "unknown-host".to_string());
        // Stale locks: older than 5 minutes can be retried/recovered
        let stale_lock_threshold = Utc::now() - chrono::Duration::minutes(5);

        let query = sea_orm::Statement::from_sql_and_values(
            sea_orm::DatabaseBackend::Postgres,
            r#"
            UPDATE outbox_job
            SET 
                status = 'processing',
                locked_by = $1,
                locked_at = NOW(),
                attempts = attempts + 1
            WHERE id = (
                SELECT id 
                FROM outbox_job 
                WHERE (status = 'pending' 
                       OR (status = 'processing' AND locked_at < $2) 
                       OR (status = 'failed' AND attempts < 5))
                  AND run_at <= NOW()
                ORDER BY run_at ASC
                LIMIT 1
                FOR UPDATE SKIP LOCKED
            )
            RETURNING id, tenant_id, job_type, payload, status, attempts, error_message, locked_by, locked_at, created_at, run_at
            "#,
            vec![hostname.clone().into(), stale_lock_threshold.into()],
        );

        let opt_job = outbox_job::Entity::find()
            .from_raw_sql(query)
            .one(db)
            .await?;

        if let Some(job) = opt_job {
            info!("OutboxWorker: checked out job {} (type: {})", job.id, job.job_type);

            let start_time = std::time::Instant::now();
            let tenant_str = job.tenant_id.to_string();

            let result = match job.job_type.as_str() {
                "send_magic_link_email" => {
                    match serde_json::from_value::<crate::handlers::communications::SendEmailPayload>(job.payload.clone()) {
                        Ok(email_payload) => {
                            match crate::handlers::communications::send_email_handler(
                                axum::extract::State(db.clone()),
                                axum::extract::Json(email_payload),
                            ).await {
                                Ok(_) => Ok(()),
                                Err((status, msg)) => Err(format!("Email send failed with status {:?}: {}", status, msg)),
                            }
                        }
                        Err(e) => Err(format!("Failed to deserialize email payload: {:?}", e)),
                    }
                }

                // G-27: Recompute dimension aggregates, composite score, confidence level,
                // and dimension_vector for all scorecards that have new verified entries
                // since the last run. Runs every 5 minutes (interval_seconds = 300).
                //
                // The payload may contain {"scorecard_id": "<uuid>"} to target a single
                // scorecard, or be absent to scan all stale scorecards for the tenant.
                "recompute_scorecard_aggregates" => {
                    let maybe_id = job.payload
                        .get("scorecard_id")
                        .and_then(|v| v.as_str())
                        .and_then(|s| uuid::Uuid::parse_str(s).ok());

                    if let Some(scorecard_id) = maybe_id {
                        crate::services::scorecard_service::ScorecardService::recompute_aggregates(
                            db,
                            scorecard_id,
                        )
                        .await
                        .map_err(|e| format!("recompute_aggregates({scorecard_id}) failed: {e}"))
                    } else {
                        // Tenant-wide sweep: recompute all stale scorecards for this tenant.
                        use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};
                        use crate::entities::atlas_scorecard;

                        let stale = match atlas_scorecard::Entity::find()
                            .filter(atlas_scorecard::Column::TenantId.eq(job.tenant_id))
                            .all(db)
                            .await
                        {
                            Ok(v) => v,
                            Err(e) => return Err(sea_orm::DbErr::Custom(
                                format!("recompute_scorecard_aggregates: failed to fetch scorecards: {e}")
                            )),
                        };

                        let mut errors: Vec<String> = Vec::new();
                        for sc in stale {
                            if let Err(e) = crate::services::scorecard_service::ScorecardService::recompute_aggregates(db, sc.id).await {
                                errors.push(format!("{}: {}", sc.id, e));
                            }
                        }

                        if errors.is_empty() {
                            Ok(())
                        } else {
                            Err(format!("recompute_aggregates sweep had {} failures: {}", errors.len(), errors.join("; ")))
                        }
                    }
                }

                // G-27: Rebuild monthly + quarterly time-series trend buckets for all
                // scorecard dimensions for this tenant. Runs hourly (interval_seconds = 3600).
                //
                // refresh_time_series_for_dimension internally handles both period types
                // (monthly and quarterly) — no period arg needed.
                "refresh_scorecard_time_series" => {
                    use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};
                    use crate::entities::{atlas_scorecard, atlas_scorecard_dimension};

                    let scorecards = match atlas_scorecard::Entity::find()
                        .filter(atlas_scorecard::Column::TenantId.eq(job.tenant_id))
                        .all(db)
                        .await
                    {
                        Ok(v) => v,
                        Err(e) => return Err(sea_orm::DbErr::Custom(
                            format!("refresh_scorecard_time_series: failed to fetch scorecards: {e}")
                        )),
                    };

                    let mut errors: Vec<String> = Vec::new();

                    for sc in &scorecards {
                        let dimensions = match atlas_scorecard_dimension::Entity::find()
                            .filter(atlas_scorecard_dimension::Column::TemplateId.eq(sc.template_id))
                            .filter(atlas_scorecard_dimension::Column::IsActive.eq(true))
                            .all(db)
                            .await
                        {
                            Ok(v) => v,
                            Err(e) => {
                                errors.push(format!("{}: failed to fetch dimensions: {e}", sc.id));
                                continue;
                            }
                        };

                        for dim in &dimensions {
                            if let Err(e) = crate::services::scorecard_service::ScorecardService::refresh_time_series_for_dimension(
                                db,
                                sc.id,
                                dim.id,
                            )
                            .await
                            {
                                errors.push(format!("{}:{}: {}", sc.id, dim.id, e));
                            }
                        }
                    }

                    if errors.is_empty() {
                        Ok(())
                    } else {
                        Err(format!("refresh_scorecard_time_series had {} failures: {}", errors.len(), errors.join("; ")))
                    }
                }



                _ => Err(format!("Unknown job type: {}", job.job_type)),
            };

            let duration = start_time.elapsed().as_secs_f64();
            crate::metrics::OUTBOX_JOB_LATENCY
                .with_label_values(&[&tenant_str, &job.job_type])
                .observe(duration);

            let job_type = job.job_type.clone();
            let mut active: outbox_job::ActiveModel = job.into();
            let job_id = active.id.as_ref().clone();
            match result {
                Ok(_) => {
                    active.status = Set("completed".to_string());
                    active.error_message = Set(None);
                    active.locked_by = Set(None);
                    active.locked_at = Set(None);
                    active.update(db).await?;
                    info!("OutboxWorker: successfully processed job: {}", job_id);

                    crate::metrics::OUTBOX_JOBS_PROCESSED
                        .with_label_values(&[&tenant_str, &job_type])
                        .inc();
                }
                Err(err_msg) => {
                    error!("OutboxWorker: job execution failed: {}", err_msg);

                    crate::metrics::OUTBOX_JOB_FAILURES
                        .with_label_values(&[&tenant_str, &job_type, &err_msg])
                        .inc();

                    active.status = Set("failed".to_string());
                    active.error_message = Set(Some(err_msg));
                    active.locked_by = Set(None);
                    active.locked_at = Set(None);
                    
                    // Simple exponential backoff for retries: retry after 2^attempts * 10 seconds
                    let attempts = active.attempts.as_ref();
                    let backoff_secs = 2i64.pow(*attempts as u32) * 10;
                    let next_run = Utc::now() + chrono::Duration::seconds(backoff_secs);
                    active.run_at = Set(next_run);
                    
                    active.update(db).await?;
                }
            }
        }

        Ok(())
    }
}
