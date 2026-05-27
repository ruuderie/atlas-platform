use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

/// GENERIC-05: atlas_external_integrations — Third-Party API Gateway
///
/// This generic provides the foundation for connecting to external systems
/// (PMS, OTA, AMS, GDS, Telephony, etc.) in a standardized, multi-tenant way.
///
/// Design notes:
/// - `credentials_encrypted` is intended to be encrypted at the application layer
///   (similar to payment credentials).
/// - The `integration_type` values listed are examples only — the platform is
///   deliberately not tied to any specific providers.
/// - `atlas_integration_events` provides an audit + replay log for inbound/outbound traffic.
#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Main integrations registry
        manager
            .create_table(
                Table::create()
                    .table(AtlasExternalIntegration::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(AtlasExternalIntegration::Id)
                            .uuid()
                            .not_null()
                            .primary_key()
                            .default(Expr::cust("gen_random_uuid()")),
                    )
                    .col(ColumnDef::new(AtlasExternalIntegration::TenantId).uuid().not_null())
                    .col(ColumnDef::new(AtlasExternalIntegration::IntegrationType).string().not_null())
                    .col(ColumnDef::new(AtlasExternalIntegration::Label).string().null())
                    .col(ColumnDef::new(AtlasExternalIntegration::CredentialsEncrypted).json_binary().not_null())
                    .col(ColumnDef::new(AtlasExternalIntegration::WebhookSecret).string().null())
                    .col(ColumnDef::new(AtlasExternalIntegration::WebhookUrl).string().null())
                    .col(ColumnDef::new(AtlasExternalIntegration::IsActive).boolean().not_null().default(true))
                    .col(ColumnDef::new(AtlasExternalIntegration::LastSyncedAt).timestamp_with_time_zone().null())
                    .col(ColumnDef::new(AtlasExternalIntegration::LastError).text().null())
                    .col(ColumnDef::new(AtlasExternalIntegration::Config).json_binary().null())
                    .col(
                        ColumnDef::new(AtlasExternalIntegration::CreatedAt)
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
                    .name("idx_atlas_external_integrations_tenant_type")
                    .table(AtlasExternalIntegration::Table)
                    .col(AtlasExternalIntegration::TenantId)
                    .col(AtlasExternalIntegration::IntegrationType)
                    .to_owned(),
            )
            .await?;

        // Unique constraint per tenant + integration_type
        manager
            .create_index(
                Index::create()
                    .name("uq_atlas_external_integrations_tenant_type")
                    .table(AtlasExternalIntegration::Table)
                    .col(AtlasExternalIntegration::TenantId)
                    .col(AtlasExternalIntegration::IntegrationType)
                    .unique()
                    .to_owned(),
            )
            .await?;

        // Integration event log (for auditing, replay, debugging)
        manager
            .create_table(
                Table::create()
                    .table(AtlasIntegrationEvent::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(AtlasIntegrationEvent::Id)
                            .uuid()
                            .not_null()
                            .primary_key()
                            .default(Expr::cust("gen_random_uuid()")),
                    )
                    .col(ColumnDef::new(AtlasIntegrationEvent::TenantId).uuid().not_null())
                    .col(ColumnDef::new(AtlasIntegrationEvent::IntegrationId).uuid().not_null())
                    .col(ColumnDef::new(AtlasIntegrationEvent::EventType).string().not_null())
                    .col(ColumnDef::new(AtlasIntegrationEvent::Direction).string().not_null())
                    .col(ColumnDef::new(AtlasIntegrationEvent::Payload).json_binary().null())
                    .col(ColumnDef::new(AtlasIntegrationEvent::Status).string().not_null().default(Expr::val("pending")))
                    .col(ColumnDef::new(AtlasIntegrationEvent::ErrorMessage).text().null())
                    .col(ColumnDef::new(AtlasIntegrationEvent::ProcessedAt).timestamp_with_time_zone().null())
                    .col(
                        ColumnDef::new(AtlasIntegrationEvent::CreatedAt)
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
                    .name("idx_atlas_integration_events_status")
                    .table(AtlasIntegrationEvent::Table)
                    .col(AtlasIntegrationEvent::IntegrationId)
                    .col(AtlasIntegrationEvent::Status)
                    .col(AtlasIntegrationEvent::CreatedAt)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(AtlasIntegrationEvent::Table).to_owned())
            .await?;

        manager
            .drop_table(Table::drop().table(AtlasExternalIntegration::Table).to_owned())
            .await?;

        Ok(())
    }
}

#[derive(DeriveIden)]
enum AtlasExternalIntegration {
    Table,
    Id,
    TenantId,
    IntegrationType,
    Label,
    CredentialsEncrypted,
    WebhookSecret,
    WebhookUrl,
    IsActive,
    LastSyncedAt,
    LastError,
    Config,
    CreatedAt,
}

#[derive(DeriveIden)]
enum AtlasIntegrationEvent {
    Table,
    Id,
    TenantId,
    IntegrationId,
    EventType,
    Direction,
    Payload,
    Status,
    ErrorMessage,
    ProcessedAt,
    CreatedAt,
}
