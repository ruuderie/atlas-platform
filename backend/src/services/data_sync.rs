use crate::entities::{bitcoin_block, tenant_background_job};
use sea_orm::{
    ActiveModelTrait, ColumnTrait, ConnectionTrait, DatabaseConnection, EntityTrait, QueryFilter, Set, Condition
};
use sea_orm::sea_query::OnConflict;
use chrono::Utc;
use std::time::Duration;
use tracing::{info, error, debug};
use uuid::Uuid;

pub struct DataSyncService;

impl DataSyncService {
    pub async fn start_worker(db: DatabaseConnection) {
        tokio::spawn(async move {
            info!("Starting DataSyncService background worker.");
            let mut interval = tokio::time::interval(Duration::from_secs(60)); // Check every minute
            
            loop {
                interval.tick().await;
                if let Err(e) = Self::process_due_jobs(&db).await {
                    error!("DataSyncService encountered an error: {}", e);
                }
            }
        });
    }

    async fn process_due_jobs(db: &DatabaseConnection) -> Result<(), sea_orm::DbErr> {
        let now = Utc::now();
        
        // Find all active jobs that are due for execution
        // Since SeaORM doesn't natively map complicated Postgres interval math seamlessly across all adapters efficiently,
        // we fetch active jobs and filter internally unless we write raw SQL. For 100s of configs, mem filtering is perfectly fine.
        let active_jobs = tenant_background_job::Entity::find()
            .filter(tenant_background_job::Column::IsActive.eq(true))
            .all(db)
            .await?;

        for job in active_jobs {
            let should_run = match job.last_run {
                None => true,
                Some(last) => {
                    let diff = now.signed_duration_since(last).num_seconds();
                    diff >= (job.interval_seconds as i64)
                }
            };

            if !should_run {
                continue;
            }

            debug!("Executing job {} / {} for tenant {}", job.id, job.job_type, job.tenant_id);

            // Dispatch based on job type
            if job.job_type == "BitcoinSync" {
                if let Err(e) = Self::sync_bitcoin_blocks(db, job.tenant_id, "https://mempool.space/api/blocks").await {
                    error!("Failed to execute BitcoinSync for tenant {}: {}", job.tenant_id, e);
                }
            } else {
                debug!("Unknown job type: {}", job.job_type);
            }

            // Update last_run
            let mut active_model: tenant_background_job::ActiveModel = job.into();
            active_model.last_run = Set(Some(Utc::now().into()));
            active_model.update(db).await?;
        }

        Ok(())
    }

    pub async fn sync_bitcoin_blocks(db: &DatabaseConnection, tenant_id: Uuid, api_url: &str) -> Result<(), anyhow::Error> {
        let res = reqwest::get(api_url).await?;
        if !res.status().is_success() {
            anyhow::bail!("mempool.space returned error status: {}", res.status());
        }

        let blocks: Vec<serde_json::Value> = res.json().await?;
        let mut inserts = Vec::new();

        for block in blocks {
            let id = block["id"].as_str().unwrap_or_default().to_string();
            let height = block["height"].as_i64().unwrap_or(0);
            let timestamp = block["timestamp"].as_i64().unwrap_or(0);
            let tx_count = block["tx_count"].as_i64().unwrap_or(0) as i32;
            let size = block["size"].as_i64().unwrap_or(0) as i32;
            let weight = block["weight"].as_i64().unwrap_or(0) as i32;
            let difficulty = block["difficulty"].as_f64().unwrap_or(0.0);

            inserts.push(bitcoin_block::ActiveModel {
                id: Set(id),
                tenant_id: Set(tenant_id),
                height: Set(height),
                timestamp: Set(timestamp),
                tx_count: Set(tx_count),
                size: Set(size),
                weight: Set(weight),
                difficulty: Set(difficulty),
                fetched_at: Set(Utc::now().into()),
            });
        }

        if !inserts.is_empty() {
            // ON CONFLICT (tenant_id, height) DO NOTHING
            bitcoin_block::Entity::insert_many(inserts)
                .on_conflict(
                    OnConflict::columns(vec![bitcoin_block::Column::TenantId, bitcoin_block::Column::Height])
                        .do_nothing()
                        .to_owned()
                )
                .exec(db)
                .await?;
        }

        Ok(())
    }
}
