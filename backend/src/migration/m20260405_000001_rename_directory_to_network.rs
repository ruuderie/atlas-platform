use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let _ = manager.get_connection().execute_unprepared("UPDATE app_instances SET app_type = 'Network' WHERE app_type = 'Directory';").await;
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let _ = manager.get_connection().execute_unprepared("UPDATE app_instances SET app_type = 'Directory' WHERE app_type = 'Network';").await;
        Ok(())
    }
}
