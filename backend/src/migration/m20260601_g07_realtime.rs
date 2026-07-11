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
                    .table(AtlasWsRooms::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(AtlasWsRooms::Id)
                            .uuid()
                            .not_null()
                            .primary_key()
                            .default(Expr::cust("gen_random_uuid()")),
                    )
                    .col(ColumnDef::new(AtlasWsRooms::TenantId).uuid().not_null())
                    .col(ColumnDef::new(AtlasWsRooms::RoomType).string().not_null())
                    .col(ColumnDef::new(AtlasWsRooms::EntityType).string().not_null())
                    .col(ColumnDef::new(AtlasWsRooms::EntityId).uuid().not_null())
                    .col(
                        ColumnDef::new(AtlasWsRooms::IsActive)
                            .boolean()
                            .not_null()
                            .default(true),
                    )
                    .col(
                        ColumnDef::new(AtlasWsRooms::CreatedAt)
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
                    .table(AtlasWsRooms::Table)
                    .col(AtlasWsRooms::TenantId)
                    .col(AtlasWsRooms::RoomType)
                    .col(AtlasWsRooms::EntityType)
                    .col(AtlasWsRooms::EntityId)
                    .unique()
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(AtlasWsMessages::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(AtlasWsMessages::Id)
                            .uuid()
                            .not_null()
                            .primary_key()
                            .default(Expr::cust("gen_random_uuid()")),
                    )
                    .col(ColumnDef::new(AtlasWsMessages::RoomId).uuid().not_null())
                    .col(ColumnDef::new(AtlasWsMessages::SenderUserId).uuid().null())
                    .col(
                        ColumnDef::new(AtlasWsMessages::MessageType)
                            .string()
                            .not_null()
                            .default(Expr::val("text")),
                    )
                    .col(ColumnDef::new(AtlasWsMessages::Content).text().not_null())
                    .col(
                        ColumnDef::new(AtlasWsMessages::TranslatedContent)
                            .json_binary()
                            .null(),
                    )
                    .col(ColumnDef::new(AtlasWsMessages::AttachmentId).uuid().null())
                    .col(
                        ColumnDef::new(AtlasWsMessages::CreatedAt)
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
                    .table(AtlasWsMessages::Table)
                    .col(AtlasWsMessages::RoomId)
                    .col(AtlasWsMessages::CreatedAt)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(AtlasWsMessages::Table).to_owned())
            .await?;

        manager
            .drop_table(Table::drop().table(AtlasWsRooms::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum AtlasWsRooms {
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
enum AtlasWsMessages {
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
