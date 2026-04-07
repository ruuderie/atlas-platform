use chrono::Utc;
use sea_orm::{DatabaseConnection, EntityTrait, QueryFilter, ColumnTrait};
use uuid::Uuid;
use crate::services::audit::AuditService;
use crate::entities::audit_log;
use serde_json::json;
use crate::tests::api_tests::setup_test_app;

#[tokio::test]
async fn test_audit_ledger_immutable_logging() {
    let (_, db) = setup_test_app().await;


    let tenant_id = Uuid::new_v4();
    let actor_id = Uuid::new_v4();
    let entity_id = Uuid::new_v4();

    let old_state = json!({"status": "pending"});
    let new_state = json!({"status": "completed"});

    AuditService::log_action(
        db.clone(),
        Some(tenant_id),
        Some(actor_id),
        "test.action.completed".to_string(),
        "TestEntity".to_string(),
        entity_id,
        Some(old_state.clone()),
        Some(new_state.clone()),
        Some("127.0.0.1".to_string()),
    );

    // Wait for the tokio::spawn thread to finish inserting the log into the database
    tokio::time::sleep(std::time::Duration::from_millis(500)).await;

    // Verify it was persisted correctly
    let log_entry = audit_log::Entity::find()
        .filter(audit_log::Column::TenantId.eq(tenant_id))
        .filter(audit_log::Column::EntityId.eq(entity_id))
        .one(&db)
        .await
        .unwrap()
        .expect("Audit log was not persisted!");

    assert_eq!(log_entry.action_type, "test.action.completed");
    assert_eq!(log_entry.entity_type, "TestEntity");
    assert_eq!(log_entry.old_state, Some(old_state));
    assert_eq!(log_entry.new_state, Some(new_state));
    assert_eq!(log_entry.ip_address.as_deref(), Some("127.0.0.1"));
}

#[tokio::test]
async fn test_audit_logs_tenant_isolation() {
    let (_, db) = setup_test_app().await;

    let tenant_alpha_id = Uuid::new_v4();
    let tenant_beta_id = Uuid::new_v4();

    AuditService::log_action(
        db.clone(),
        Some(tenant_alpha_id),
        None,
        "alpha.action".to_string(),
        "SomeEntity".to_string(),
        Uuid::new_v4(),
        None,
        None,
        None,
    );

    AuditService::log_action(
        db.clone(),
        Some(tenant_beta_id),
        None,
        "beta.action".to_string(),
        "SomeEntity".to_string(),
        Uuid::new_v4(),
        None,
        None,
        None,
    );

    tokio::time::sleep(std::time::Duration::from_millis(500)).await;

    // Verify Tenant Alpha doesn't see Beta's logs
    let alpha_logs = audit_log::Entity::find()
        .filter(audit_log::Column::TenantId.eq(tenant_alpha_id))
        .all(&db)
        .await
        .unwrap();
    
    assert_eq!(alpha_logs.len(), 1);
    assert_eq!(alpha_logs[0].action_type, "alpha.action");

    // Verify Tenant Beta doesn't see Alpha's logs
    let beta_logs = audit_log::Entity::find()
        .filter(audit_log::Column::TenantId.eq(tenant_beta_id))
        .all(&db)
        .await
        .unwrap();

    assert_eq!(beta_logs.len(), 1);
    assert_eq!(beta_logs[0].action_type, "beta.action");
}
