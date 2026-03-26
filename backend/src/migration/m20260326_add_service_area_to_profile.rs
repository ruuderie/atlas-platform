use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(Profile::Table)
                    .add_column(
                        ColumnDef::new(Profile::ServiceAreaZips)
                            .array(ColumnType::String(StringLen::None))
                            .null(),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(Profile::Table)
                    .drop_column(Profile::ServiceAreaZips)
                    .to_owned(),
            )
            .await
    }
}

#[derive(Iden)]
enum Profile {
    Table,
    ServiceAreaZips,
}
