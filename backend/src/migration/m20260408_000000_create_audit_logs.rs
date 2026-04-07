use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Create the audit_logs table
        manager
            .create_table(
                Table::create()
                    .table(AuditLogs::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(AuditLogs::Id)
                            .uuid()
                            .not_null()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(AuditLogs::TenantId).uuid())
                    .col(ColumnDef::new(AuditLogs::ActorId).uuid())
                    .col(ColumnDef::new(AuditLogs::ActionType).string().not_null())
                    .col(ColumnDef::new(AuditLogs::EntityType).string().not_null())
                    .col(ColumnDef::new(AuditLogs::EntityId).uuid().not_null())
                    .col(ColumnDef::new(AuditLogs::OldState).json_binary())
                    .col(ColumnDef::new(AuditLogs::NewState).json_binary())
                    .col(ColumnDef::new(AuditLogs::IpAddress).string())
                    .col(
                        ColumnDef::new(AuditLogs::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .to_owned(),
            )
            .await?;

        // Enforce Append-Only constraints via raw Postgres statements
        let db = manager.get_connection();
        
        // 1. Revoke UPDATE and DELETE permissions from standard roles
        let revoke_sql = "REVOKE UPDATE, DELETE ON audit_logs FROM public;";
        db.execute_unprepared(revoke_sql).await?;
        
        // 2. Create a trigger function that explicitly aborts any UPDATE or DELETE attempt
        let trigger_func_sql = r#"
            CREATE OR REPLACE FUNCTION prevent_audit_log_modification()
            RETURNS TRIGGER AS $$
            BEGIN
                RAISE EXCEPTION 'audit_logs is an append-only ledger. Modifications are forbidden.';
            END;
            $$ LANGUAGE plpgsql;
        "#;
        db.execute_unprepared(trigger_func_sql).await?;

        // 3. Attach the trigger to the table
        let trigger_stmt_sql = r#"
            DROP TRIGGER IF EXISTS enforce_append_only_audit_logs ON audit_logs;
            CREATE TRIGGER enforce_append_only_audit_logs
            BEFORE UPDATE OR DELETE ON audit_logs
            FOR EACH ROW EXECUTE FUNCTION prevent_audit_log_modification();
        "#;
        db.execute_unprepared(trigger_stmt_sql).await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();
        
        let drop_trigger_sql = "DROP TRIGGER IF EXISTS enforce_append_only_audit_logs ON audit_logs;";
        db.execute_unprepared(drop_trigger_sql).await?;

        let drop_function_sql = "DROP FUNCTION IF EXISTS prevent_audit_log_modification();";
        db.execute_unprepared(drop_function_sql).await?;

        manager
            .drop_table(Table::drop().table(AuditLogs::Table).to_owned())
            .await
    }
}

#[derive(Iden)]
enum AuditLogs {
    Table,
    Id,
    TenantId,
    ActorId,
    ActionType,
    EntityType,
    EntityId,
    OldState,
    NewState,
    IpAddress,
    CreatedAt,
}
