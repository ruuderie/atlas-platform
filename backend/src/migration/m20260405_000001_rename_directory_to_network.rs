use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let _ = manager.get_connection().execute_unprepared("ALTER TABLE IF EXISTS directory_type RENAME TO network_type;").await;
        let _ = manager.get_connection().execute_unprepared("ALTER TABLE IF EXISTS directory RENAME TO network;").await;
        let _ = manager.get_connection().execute_unprepared("ALTER TABLE IF EXISTS category RENAME COLUMN IF EXISTS directory_type_id TO network_type_id;").await;
        let _ = manager.get_connection().execute_unprepared("ALTER TABLE IF EXISTS category RENAME COLUMN IF EXISTS directory_id TO network_id;").await;
        let _ = manager.get_connection().execute_unprepared("ALTER TABLE IF EXISTS listing RENAME COLUMN IF EXISTS directory_id TO network_id;").await;
        let _ = manager.get_connection().execute_unprepared("ALTER TABLE IF EXISTS template RENAME COLUMN IF EXISTS directory_id TO network_id;").await;
        let _ = manager.get_connection().execute_unprepared("ALTER TABLE IF EXISTS lead RENAME COLUMN IF EXISTS directory_id TO network_id;").await;
        let _ = manager.get_connection().execute_unprepared("ALTER TABLE IF EXISTS contact RENAME COLUMN IF EXISTS directory_id TO network_id;").await;
        let _ = manager.get_connection().execute_unprepared("ALTER TABLE IF EXISTS customer RENAME COLUMN IF EXISTS directory_id TO network_id;").await;
        let _ = manager.get_connection().execute_unprepared("ALTER TABLE IF EXISTS deal RENAME COLUMN IF EXISTS directory_id TO network_id;").await;
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let _ = manager.get_connection().execute_unprepared("ALTER TABLE IF EXISTS deal RENAME COLUMN IF EXISTS network_id TO directory_id;").await;
        let _ = manager.get_connection().execute_unprepared("ALTER TABLE IF EXISTS customer RENAME COLUMN IF EXISTS network_id TO directory_id;").await;
        let _ = manager.get_connection().execute_unprepared("ALTER TABLE IF EXISTS contact RENAME COLUMN IF EXISTS network_id TO directory_id;").await;
        let _ = manager.get_connection().execute_unprepared("ALTER TABLE IF EXISTS lead RENAME COLUMN IF EXISTS network_id TO directory_id;").await;
        let _ = manager.get_connection().execute_unprepared("ALTER TABLE IF EXISTS template RENAME COLUMN IF EXISTS network_id TO directory_id;").await;
        let _ = manager.get_connection().execute_unprepared("ALTER TABLE IF EXISTS listing RENAME COLUMN IF EXISTS network_id TO directory_id;").await;
        let _ = manager.get_connection().execute_unprepared("ALTER TABLE IF EXISTS category RENAME COLUMN IF EXISTS network_id TO directory_id;").await;
        let _ = manager.get_connection().execute_unprepared("ALTER TABLE IF EXISTS category RENAME COLUMN IF EXISTS network_type_id TO directory_type_id;").await;
        let _ = manager.get_connection().execute_unprepared("ALTER TABLE IF EXISTS network RENAME TO directory;").await;
        let _ = manager.get_connection().execute_unprepared("ALTER TABLE IF EXISTS network_type RENAME TO directory_type;").await;
        Ok(())
    }
}
