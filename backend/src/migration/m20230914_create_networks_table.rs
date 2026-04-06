use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Network::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(Network::Id).uuid().not_null().primary_key())
                    .col(ColumnDef::new(Network::NetworkTypeId).uuid().not_null())
                    .col(ColumnDef::new(Network::Name).string().not_null())
                    .col(ColumnDef::new(Network::Domain).string().not_null())
                    .col(ColumnDef::new(Network::Description).string().not_null())
                    .col(ColumnDef::new(Network::CreatedAt).timestamp_with_time_zone().not_null())
                    .col(ColumnDef::new(Network::UpdatedAt).timestamp_with_time_zone().not_null())
                    .foreign_key(ForeignKey::create()
                        .name("fk-network-network_type_id")
                        .from(Network::Table, Network::NetworkTypeId)
                        .to(NetworkType::Table, NetworkType::Id))
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Network::Table).to_owned())
            .await
    }
}

#[derive(Iden)]
enum Network {
    Table,
    Id,
    NetworkTypeId,
    Name,
    Domain,
    Description,
    CreatedAt,
    UpdatedAt,
}

#[derive(Iden)]
enum NetworkType {
    Table,
    Id,
}