use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager.get_connection().execute_unprepared("ALTER TABLE network_types RENAME TO network_types;").await?;
        manager.get_connection().execute_unprepared("ALTER TABLE categories RENAME COLUMN network_type_id TO network_type_id;").await?;
        manager.get_connection().execute_unprepared("ALTER TABLE listings RENAME COLUMN network_id TO network_id;").await?;
        manager.get_connection().execute_unprepared("ALTER TABLE categories RENAME COLUMN network_id TO network_id;").await?;
        manager.get_connection().execute_unprepared("ALTER TABLE templates RENAME COLUMN network_id TO network_id;").await?;
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager.get_connection().execute_unprepared("ALTER TABLE templates RENAME COLUMN network_id TO network_id;").await?;
        manager.get_connection().execute_unprepared("ALTER TABLE categories RENAME COLUMN network_id TO network_id;").await?;
        manager.get_connection().execute_unprepared("ALTER TABLE listings RENAME COLUMN network_id TO network_id;").await?;
        manager.get_connection().execute_unprepared("ALTER TABLE categories RENAME COLUMN network_type_id TO network_type_id;").await?;
        manager.get_connection().execute_unprepared("ALTER TABLE network_types RENAME TO network_types;").await?;
        Ok(())
    }
}
