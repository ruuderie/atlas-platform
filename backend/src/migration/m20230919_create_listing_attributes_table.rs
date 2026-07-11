use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(ListingAttribute::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(ListingAttribute::Id)
                            .uuid()
                            .not_null()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(ListingAttribute::ListingId).uuid())
                    .col(ColumnDef::new(ListingAttribute::TemplateId).uuid())
                    .col(
                        ColumnDef::new(ListingAttribute::AttributeType)
                            .string()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(ListingAttribute::AttributeKey)
                            .string()
                            .not_null(),
                    )
                    .col(ColumnDef::new(ListingAttribute::Value).json().not_null())
                    .col(
                        ColumnDef::new(ListingAttribute::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(ListingAttribute::UpdatedAt)
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-listing_attribute-listing_id")
                            .from(ListingAttribute::Table, ListingAttribute::ListingId)
                            .to(Listing::Table, Listing::Id),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-listing_attribute-template_id")
                            .from(ListingAttribute::Table, ListingAttribute::TemplateId)
                            .to(Template::Table, Template::Id),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(ListingAttribute::Table).to_owned())
            .await
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
enum Listing {
    Table,
    Id,
}

#[derive(Iden)]
enum Template {
    Table,
    Id,
}
