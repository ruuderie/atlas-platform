use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

/// GENERIC-07: atlas_realtime — WebSocket Room Infrastructure
///
/// Provides the database foundation for real-time entity-scoped communication
/// (chat threads on maintenance tickets, entity comments, campaign leaderboards, etc.).
///
/// The actual WebSocket connection handling and broadcast logic lives in the
/// application layer (handlers/realtime.rs), but the persistent storage and
/// room/message model is defined here as a platform generic.
#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(AtlasWsRoom::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(AtlasWsRoom::Id)
                            .uuid()
                            .not_null()
                            .primary_key()
                            .default(Expr::cust("gen_random_uuid()")),
                    )
                    .col(ColumnDef::new(AtlasWsRoom::TenantId).uuid().not_null())
                    .col(ColumnDef::new(AtlasWsRoom::RoomType).string().not_null())
                    .col(ColumnDef::new(AtlasWsRoom::EntityType).string().not_null())
                    .col(ColumnDef::new(AtlasWsRoom::EntityId).uuid().not_null())
                    .col(ColumnDef::new(AtlasWsRoom::IsActive).boolean().not_null().default(true))
                    .col(
                        ColumnDef::new(AtlasWsRoom::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .to_owned(),
            )
            .await?;

        // Unique constraint on (tenant, room_type, entity_type, entity_id)
        manager
            .create_index(
                Index::create()
                    .name("uq_atlas_ws_rooms_scope")
                    .table(AtlasWsRoom::Table)
                    .col(AtlasWsRoom::TenantId)
                    .col(AtlasWsRoom::RoomType)
                    .col(AtlasWsRoom::EntityType)
                    .col(AtlasWsRoom::EntityId)
                    .unique()
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(AtlasWsMessage::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(AtlasWsMessage::Id)
                            .uuid()
                            .not_null()
                            .primary_key()
                            .default(Expr::cust("gen_random_uuid()")),
                    )
                    .col(ColumnDef::new(AtlasWsMessage::RoomId).uuid().not_null())
                    .col(ColumnDef::new(AtlasWsMessage::SenderUserId).uuid().null())
                    .col(ColumnDef::new(AtlasWsMessage::MessageType).string().not_null().default(Expr::val("text")))
                    .col(ColumnDef::new(AtlasWsMessage::Content).text().not_null())
                    .col(ColumnDef::new(AtlasWsMessage::TranslatedContent).json_binary().null())
                    .col(ColumnDef::new(AtlasWsMessage::AttachmentId).uuid().null())
                    .col(
                        ColumnDef::new(AtlasWsMessage::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_atlas_ws_messages_room")
                    .table(AtlasWsMessage::Table)
                    .col(AtlasWsMessage::RoomId)
                    .col(AtlasWsMessage::CreatedAt)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(AtlasWsMessage::Table).to_owned())
            .await?;

        manager
            .drop_table(Table::drop().table(AtlasWsRoom::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum AtlasWsRoom {
    Table,
    Id,
    TenantId,
    RoomType,
    EntityType,
    EntityId,
    IsActive,
    CreatedAt,
}

#[derive(DeriveIden)]
enum AtlasWsMessage {
    Table,
    Id,
    RoomId,
    SenderUserId,
    MessageType,
    Content,
    TranslatedContent,
    AttachmentId,
    CreatedAt,
}
