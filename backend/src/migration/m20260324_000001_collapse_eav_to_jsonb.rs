use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // 1. Add `properties` to core tables
        manager.alter_table(
            Table::alter()
                .table(Listing::Table)
                .add_column(ColumnDef::new(Listing::Properties).json_binary())
                .to_owned(),
        ).await?;

        manager.alter_table(
            Table::alter()
                .table(Lead::Table)
                .add_column(ColumnDef::new(Lead::Properties).json_binary())
                .to_owned(),
        ).await?;

        manager.alter_table(
            Table::alter()
                .table(Customer::Table)
                .add_column(ColumnDef::new(Customer::Properties).json_binary())
                .to_owned(),
        ).await?;

        manager.alter_table(
            Table::alter()
                .table(Contact::Table)
                .add_column(ColumnDef::new(Contact::Properties).json_binary())
                .to_owned(),
        ).await?;

        manager.alter_table(
            Table::alter()
                .table(Deal::Table)
                .add_column(ColumnDef::new(Deal::Properties).json_binary())
                .to_owned(),
        ).await?;

        manager.alter_table(
            Table::alter()
                .table(Case::Table)
                .add_column(ColumnDef::new(Case::Properties).json_binary())
                .to_owned(),
        ).await?;

        manager.alter_table(
            Table::alter()
                .table(Profile::Table)
                .add_column(ColumnDef::new(Profile::Properties).json_binary())
                .to_owned(),
        ).await?;

        manager.alter_table(
            Table::alter()
                .table(Template::Table)
                .add_column(ColumnDef::new(Template::AttributesSchema).json_binary())
                .to_owned(),
        ).await?;

        // 2. Drop the listing_attribute table
        manager
            .drop_table(Table::drop().table(ListingAttribute::Table).to_owned())
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Re-create listing_attribute table
        manager
            .create_table(
                Table::create()
                    .table(ListingAttribute::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(ListingAttribute::Id).uuid().not_null().primary_key())
                    .col(ColumnDef::new(ListingAttribute::ListingId).uuid())
                    .col(ColumnDef::new(ListingAttribute::TemplateId).uuid())
                    .col(ColumnDef::new(ListingAttribute::AttributeType).string().not_null())
                    .col(ColumnDef::new(ListingAttribute::AttributeKey).string().not_null())
                    .col(ColumnDef::new(ListingAttribute::Value).json().not_null())
                    .col(ColumnDef::new(ListingAttribute::CreatedAt).timestamp_with_time_zone().not_null())
                    .col(ColumnDef::new(ListingAttribute::UpdatedAt).timestamp_with_time_zone().not_null())
                    .foreign_key(ForeignKey::create()
                        .name("fk-listing_attribute-listing_id")
                        .from(ListingAttribute::Table, ListingAttribute::ListingId)
                        .to(Listing::Table, Listing::Id))
                    .foreign_key(ForeignKey::create()
                        .name("fk-listing_attribute-template_id")
                        .from(ListingAttribute::Table, ListingAttribute::TemplateId)
                        .to(Template::Table, Template::Id))
                    .to_owned(),
            )
            .await?;

        // Drop properties columns
        manager.alter_table(Table::alter().table(Listing::Table).drop_column(Listing::Properties).to_owned()).await?;
        manager.alter_table(Table::alter().table(Lead::Table).drop_column(Lead::Properties).to_owned()).await?;
        manager.alter_table(Table::alter().table(Customer::Table).drop_column(Customer::Properties).to_owned()).await?;
        manager.alter_table(Table::alter().table(Contact::Table).drop_column(Contact::Properties).to_owned()).await?;
        manager.alter_table(Table::alter().table(Deal::Table).drop_column(Deal::Properties).to_owned()).await?;
        manager.alter_table(Table::alter().table(Case::Table).drop_column(Case::Properties).to_owned()).await?;
        manager.alter_table(Table::alter().table(Profile::Table).drop_column(Profile::Properties).to_owned()).await?;
        manager.alter_table(Table::alter().table(Template::Table).drop_column(Template::AttributesSchema).to_owned()).await?;

        Ok(())
    }
}

#[derive(Iden)]
enum ListingAttribute {
    Table,
    Id,
    ListingId,
    TemplateId,
    AttributeType,
    AttributeKey,
    Value,
    CreatedAt,
    UpdatedAt,
}

#[derive(Iden)]
enum Listing { Table, Id, Properties }

#[derive(Iden)]
enum Lead { Table, Properties }

#[derive(Iden)]
enum Customer { Table, Properties }

#[derive(Iden)]
enum Contact { Table, Properties }

#[derive(Iden)]
enum Deal { Table, Properties }

#[derive(Iden)]
enum Case { Table, Properties }

#[derive(Iden)]
enum Profile { Table, Properties }

#[derive(Iden)]
enum Template { Table, Id, AttributesSchema }
