use chrono::{Utc, Datelike};
use sea_orm::{Database, DatabaseConnection, EntityTrait, Set, ActiveModelTrait, QueryFilter, ColumnTrait};
use uuid::Uuid;
use crate::services::telemetry::TelemetryService;
use crate::entities::{telemetry_events, platform_metrics_daily};
use crate::migration;
use sea_orm_migration::MigratorTrait;
use serde_json::json;

async fn setup_db() -> DatabaseConnection {
    let opt_local_machine = sea_orm::ConnectOptions::new("postgres://postgres:postgres@localhost:5432/oplydbtest")
        .connect_timeout(std::time::Duration::from_secs(2))
        .to_owned();

    let db = Database::connect(opt_local_machine).await.expect("Failed to connect to local DB");
    
    // Make sure we have the tables
    crate::tests::test_utils::initialize_database(&db).await;

    db
}

#[tokio::test]
async fn test_telemetry_kpi_engine_aggregates_correctly() {
    // Attempt DB connection. In some environments (like pure CI without DB), this might fail.
    // If it fails, ignore the test or panic cleanly since it's an integration test.
    let db_result = sea_orm::ConnectOptions::new("postgres://postgres:postgres@localhost:5432/oplydbtest")
        .connect_timeout(std::time::Duration::from_secs(1))
        .to_owned();
    
    let db = match Database::connect(db_result).await {
        Ok(db) => {
            crate::tests::test_utils::initialize_database(&db).await;
            db
        },
        Err(_) => {
            println!("Skipping telemetry aggregation test because Postgres is not available on localhost");
            return;
        }
    };

    let tenant_id = Uuid::new_v4();

    // Insert 2 signups and 1 churn
    TelemetryService::log_event(
        db.clone(),
        tenant_id,
        "app:core".to_string(),
        "user_signed_up".to_string(),
        None,
    );

    TelemetryService::log_event(
        db.clone(),
        tenant_id,
        "app:core".to_string(),
        "user_signed_up".to_string(),
        None,
    );
    
    TelemetryService::log_event(
        db.clone(),
        tenant_id,
        "app:core".to_string(),
        "subscription_created".to_string(),
        Some(json!({"mrr": 50.0})),
    );

    // Wait a brief moment for tokio::spawn to finish inserting
    tokio::time::sleep(std::time::Duration::from_millis(50)).await;

    // Run processor
    let process_res = TelemetryService::process_daily_metrics(&db).await;
    assert!(process_res.is_ok(), "Processor failed: {:?}", process_res.unwrap_err());

    // Check output
    let today = Utc::now().date_naive();
    let mrr_metric = platform_metrics_daily::Entity::find()
        .filter(platform_metrics_daily::Column::TenantId.eq(tenant_id))
        .filter(platform_metrics_daily::Column::MetricKey.eq("mrr"))
        .filter(platform_metrics_daily::Column::Date.eq(today))
        .one(&db)
        .await
        .unwrap()
        .expect("No MRR metric created");

    assert_eq!(mrr_metric.metric_value, 50.0);

    let signups_metric = platform_metrics_daily::Entity::find()
        .filter(platform_metrics_daily::Column::TenantId.eq(tenant_id))
        .filter(platform_metrics_daily::Column::MetricKey.eq("user_signed_up"))
        .filter(platform_metrics_daily::Column::Date.eq(today))
        .one(&db)
        .await
        .unwrap()
        .expect("No signup metric created");

    assert_eq!(signups_metric.metric_value, 2.0);

    // Verify raw events are marked processed
    let unprocessed = telemetry_events::Entity::find()
        .filter(telemetry_events::Column::TenantId.eq(tenant_id))
        .filter(telemetry_events::Column::Processed.eq(false))
        .all(&db)
        .await
        .unwrap();
    
    assert!(unprocessed.is_empty(), "Raw events not marked as processed");
}
