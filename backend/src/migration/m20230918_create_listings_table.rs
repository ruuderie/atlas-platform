use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Listing::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(Listing::Id).uuid().not_null().primary_key())
                    .col(ColumnDef::new(Listing::ProfileId).uuid().not_null())
                    .col(ColumnDef::new(Listing::DirectoryId).uuid().not_null())
                    .col(ColumnDef::new(Listing::CategoryId).uuid().not_null())
                    .col(ColumnDef::new(Listing::Title).string().not_null())
                    .col(ColumnDef::new(Listing::Description).string().not_null())
                    .col(ColumnDef::new(Listing::ListingType).string().not_null())
                    .col(ColumnDef::new(Listing::Price).big_integer())
                    .col(ColumnDef::new(Listing::PriceType).string())
                    .col(ColumnDef::new(Listing::Country).string().not_null())
                    .col(ColumnDef::new(Listing::State).string().not_null())
                    .col(ColumnDef::new(Listing::City).string().not_null())
                    .col(ColumnDef::new(Listing::Neighborhood).string())
                    .col(ColumnDef::new(Listing::Latitude).double())
                    .col(ColumnDef::new(Listing::Longitude).double())
                    .col(ColumnDef::new(Listing::AdditionalInfo).json())
                    .col(ColumnDef::new(Listing::Status).string().not_null())
                    .col(ColumnDef::new(Listing::IsFeatured).boolean().not_null().default(false))
                    .col(ColumnDef::new(Listing::IsBasedOnTemplate).boolean().not_null().default(false))
                    .col(ColumnDef::new(Listing::BasedOnTemplateId).uuid())
                    .col(ColumnDef::new(Listing::IsAdPlacement).boolean().not_null().default(false))
                    .col(ColumnDef::new(Listing::IsActive).boolean().not_null().default(true))
                    .col(ColumnDef::new(Listing::CreatedAt).timestamp_with_time_zone().not_null())
                    .col(ColumnDef::new(Listing::UpdatedAt).timestamp_with_time_zone().not_null())
                    .foreign_key(ForeignKey::create()
                        .name("fk-listing-profile_id")
                        .from(Listing::Table, Listing::ProfileId)
                        .to(Profile::Table, Profile::Id))
                    .foreign_key(ForeignKey::create()
                        .name("fk-listing-directory_id")
                        .from(Listing::Table, Listing::DirectoryId)
                        .to(Directory::Table, Directory::Id))
                    .foreign_key(ForeignKey::create()
                        .name("fk-listing-category_id")
                        .from(Listing::Table, Listing::CategoryId)
                        .to(Category::Table, Category::Id))
                    .foreign_key(ForeignKey::create()
                        .name("fk-listing-based_on_template_id")
                        .from(Listing::Table, Listing::BasedOnTemplateId)
                        .to(Template::Table, Template::Id))
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Listing::Table).to_owned())
            .await
    }
}

#[derive(Iden)]
enum Listing {
    Table,
    Id,
    ProfileId,
    DirectoryId,
    CategoryId,
    Title,
    Description,
    ListingType,
    Price,
    PriceType,
    Country,
    State,
    City,
    Neighborhood,
    Latitude,
    Longitude,
    AdditionalInfo,
    Status,
    IsFeatured,
    IsBasedOnTemplate,
    BasedOnTemplateId,
    IsAdPlacement,
    IsActive,
    CreatedAt,
    UpdatedAt,
}

#[derive(Iden)]
enum Profile {
    Table,
    Id,
}

#[derive(Iden)]
enum Directory {
    Table,
    Id,
}

#[derive(Iden)]
enum Category {
    Table,
    Id,
}

#[derive(Iden)]
enum Template {
    Table,
    Id,
}