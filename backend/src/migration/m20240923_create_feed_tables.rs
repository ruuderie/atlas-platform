use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Create feed table
        manager
            .create_table(
                Table::create()
                    .table(Feed::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(Feed::Id).uuid().not_null().primary_key())
                    .col(ColumnDef::new(Feed::DirectoryId).uuid().not_null())
                    .col(ColumnDef::new(Feed::Title).string().not_null())
                    .col(ColumnDef::new(Feed::Description).string().not_null())
                    .col(ColumnDef::new(Feed::FeedUrl).string().not_null())
                    .col(ColumnDef::new(Feed::HomePageUrl).string().not_null())
                    .col(ColumnDef::new(Feed::Icon).string().null())
                    .col(ColumnDef::new(Feed::Favicon).string().null())
                    .col(ColumnDef::new(Feed::CreatedAt).timestamp_with_time_zone().not_null())
                    .col(ColumnDef::new(Feed::UpdatedAt).timestamp_with_time_zone().not_null())
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-feed-directory_id")
                            .from(Feed::Table, Feed::DirectoryId)
                            .to(Directory::Table, Directory::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                    )
                    .to_owned(),
            )
            .await?;

        // Create feed_item table
        manager
            .create_table(
                Table::create()
                    .table(FeedItem::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(FeedItem::Id).uuid().not_null().primary_key())
                    .col(ColumnDef::new(FeedItem::FeedId).uuid().not_null())
                    .col(ColumnDef::new(FeedItem::Url).string().not_null())
                    .col(ColumnDef::new(FeedItem::ExternalUrl).string().null())
                    .col(ColumnDef::new(FeedItem::Title).string().not_null())
                    .col(ColumnDef::new(FeedItem::ContentHtml).string().not_null())
                    .col(ColumnDef::new(FeedItem::ContentText).string().not_null())
                    .col(ColumnDef::new(FeedItem::Summary).string().null())
                    .col(ColumnDef::new(FeedItem::Image).string().null())
                    .col(ColumnDef::new(FeedItem::PublishedAt).timestamp_with_time_zone().null())
                    .col(ColumnDef::new(FeedItem::CreatedAt).timestamp_with_time_zone().not_null())
                    .col(ColumnDef::new(FeedItem::UpdatedAt).timestamp_with_time_zone().not_null())
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-feed_item-feed_id")
                            .from(FeedItem::Table, FeedItem::FeedId)
                            .to(Feed::Table, Feed::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                    )
                    .to_owned(),
            )
            .await?;

        // Create attachment table
        manager
            .create_table(
                Table::create()
                    .table(Attachment::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(Attachment::Id).uuid().not_null().primary_key())
                    .col(ColumnDef::new(Attachment::FeedItemId).uuid().null())
                    .col(ColumnDef::new(Attachment::Url).string().not_null())
                    .col(ColumnDef::new(Attachment::MimeType).string().not_null())
                    .col(ColumnDef::new(Attachment::Title).string().null())
                    .col(ColumnDef::new(Attachment::SizeInBytes).big_integer().null())
                    .col(ColumnDef::new(Attachment::DurationInSeconds).integer().null())
                    .col(ColumnDef::new(Attachment::CreatedAt).timestamp_with_time_zone().not_null())
                    .col(ColumnDef::new(Attachment::UpdatedAt).timestamp_with_time_zone().not_null())
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-attachment-feed_item_id")
                            .from(Attachment::Table, Attachment::FeedItemId)
                            .to(FeedItem::Table, FeedItem::Id)
                            .on_delete(ForeignKeyAction::SetNull)
                    )
                    .to_owned(),
            )
            .await?;

        // Create files table
        manager
            .create_table(
                Table::create()
                    .table(File::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(File::Id).string().not_null().primary_key())
                    .col(ColumnDef::new(File::Name).string().not_null())
                    .col(ColumnDef::new(File::Size).big_integer().not_null())
                    .col(ColumnDef::new(File::MimeType).string().not_null())
                    .col(ColumnDef::new(File::HashSha256).string().not_null())
                    .col(ColumnDef::new(File::StorageType).string().not_null())
                    .col(ColumnDef::new(File::StoragePath).string().not_null())
                    .col(ColumnDef::new(File::CreatedAt).timestamp_with_time_zone().not_null())
                    .col(ColumnDef::new(File::UpdatedAt).timestamp_with_time_zone().not_null())
                    .to_owned(),
            )
            .await?;

        // Create file_associations table
        manager
            .create_table(
                Table::create()
                    .table(FileAssociation::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(FileAssociation::Id).uuid().not_null().primary_key())
                    .col(ColumnDef::new(FileAssociation::FileId).string().not_null())
                    .col(ColumnDef::new(FileAssociation::AssociatedEntityType).string().not_null())
                    .col(ColumnDef::new(FileAssociation::AssociatedEntityId).uuid().not_null())
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-file_association-file_id")
                            .from(FileAssociation::Table, FileAssociation::FileId)
                            .to(File::Table, File::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                    )
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Drop tables in reverse order of creation
        manager
            .drop_table(Table::drop().table(FileAssociation::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(File::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(Attachment::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(FeedItem::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(Feed::Table).to_owned())
            .await?;

        Ok(())
    }
}

#[derive(Iden)]
enum Feed {
    Table,
    Id,
    DirectoryId,
    Title,
    Description,
    FeedUrl,
    HomePageUrl,
    Icon,
    Favicon,
    CreatedAt,
    UpdatedAt,
}

#[derive(Iden)]
enum FeedItem {
    Table,
    Id,
    FeedId,
    Url,
    ExternalUrl,
    Title,
    ContentHtml,
    ContentText,
    Summary,
    Image,
    PublishedAt,
    CreatedAt,
    UpdatedAt,
}

#[derive(Iden)]
enum Attachment {
    Table,
    Id,
    FeedItemId,
    Url,
    MimeType,
    Title,
    SizeInBytes,
    DurationInSeconds,
    CreatedAt,
    UpdatedAt,
}

#[derive(Iden)]
enum File {
    Table,
    Id,
    Name,
    Size,
    MimeType,
    HashSha256,
    StorageType,
    StoragePath,
    CreatedAt,
    UpdatedAt,
}

#[derive(Iden)]
enum FileAssociation {
    Table,
    Id,
    FileId,
    AssociatedEntityType,
    AssociatedEntityId,
}

#[derive(Iden)]
enum Directory {
    Table,
    Id,
}