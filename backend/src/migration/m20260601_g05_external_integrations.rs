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
                    .table(AtlasExternalIntegrations::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(AtlasExternalIntegrations::Id)
                            .uuid()
                            .not_null()
                            .primary_key()
                            .default(Expr::cust("gen_random_uuid()")),
                    )
                    .col(
                        ColumnDef::new(AtlasExternalIntegrations::TenantId)
                            .uuid()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(AtlasExternalIntegrations::IntegrationType)
                            .string()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(AtlasExternalIntegrations::Label)
                            .string()
                            .null(),
                    )
                    .col(
                        ColumnDef::new(AtlasExternalIntegrations::CredentialsEncrypted)
                            .json_binary()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(AtlasExternalIntegrations::WebhookSecret)
                            .string()
                            .null(),
                    )
                    .col(
                        ColumnDef::new(AtlasExternalIntegrations::WebhookUrl)
                            .string()
                            .null(),
                    )
                    .col(
                        ColumnDef::new(AtlasExternalIntegrations::IsActive)
                            .boolean()
                            .not_null()
                            .default(true),
                    )
                    .col(
                        ColumnDef::new(AtlasExternalIntegrations::LastSyncedAt)
                            .timestamp_with_time_zone()
                            .null(),
                    )
                    .col(
                        ColumnDef::new(AtlasExternalIntegrations::LastError)
                            .text()
                            .null(),
                    )
                    .col(
                        ColumnDef::new(AtlasExternalIntegrations::Config)
                            .json_binary()
                            .null(),
                    )
                    .col(
                        ColumnDef::new(AtlasExternalIntegrations::CreatedAt)
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
                    .table(AtlasExternalIntegrations::Table)
                    .col(AtlasExternalIntegrations::TenantId)
                    .col(AtlasExternalIntegrations::IntegrationType)
                    .to_owned(),
            )
            .await?;

        // Unique constraint per tenant + integration_type
        manager
            .create_index(
                Index::create()
                    .name("uq_atlas_external_integrations_tenant_type")
                    .table(AtlasExternalIntegrations::Table)
                    .col(AtlasExternalIntegrations::TenantId)
                    .col(AtlasExternalIntegrations::IntegrationType)
                    .unique()
                    .to_owned(),
            )
            .await?;

        // Integration event log (for auditing, replay, debugging)
        manager
            .create_table(
                Table::create()
                    .table(AtlasIntegrationEvents::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(AtlasIntegrationEvents::Id)
                            .uuid()
                            .not_null()
                            .primary_key()
                            .default(Expr::cust("gen_random_uuid()")),
                    )
                    .col(
                        ColumnDef::new(AtlasIntegrationEvents::TenantId)
                            .uuid()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(AtlasIntegrationEvents::IntegrationId)
                            .uuid()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(AtlasIntegrationEvents::EventType)
                            .string()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(AtlasIntegrationEvents::Direction)
                            .string()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(AtlasIntegrationEvents::Payload)
                            .json_binary()
                            .null(),
                    )
                    .col(
                        ColumnDef::new(AtlasIntegrationEvents::Status)
                            .string()
                            .not_null()
                            .default(Expr::val("pending")),
                    )
                    .col(
                        ColumnDef::new(AtlasIntegrationEvents::ErrorMessage)
                            .text()
                            .null(),
                    )
                    .col(
                        ColumnDef::new(AtlasIntegrationEvents::ProcessedAt)
                            .timestamp_with_time_zone()
                            .null(),
                    )
                    .col(
                        ColumnDef::new(AtlasIntegrationEvents::CreatedAt)
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
                    .table(AtlasIntegrationEvents::Table)
                    .col(AtlasIntegrationEvents::IntegrationId)
                    .col(AtlasIntegrationEvents::Status)
                    .col(AtlasIntegrationEvents::CreatedAt)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Note: atlas_integration_events is now owned by m20260915_atlas_syndication_outbox.
        // That migration's down() drops it first; use IF EXISTS here so rollback is safe
        // regardless of which migration runs first in a partial rollback scenario.
        manager
            .drop_table(
                Table::drop()
                    .table(AtlasIntegrationEvents::Table)
                    .if_exists()
                    .to_owned(),
            )
            .await?;

        manager
            .drop_table(
                Table::drop()
                    .table(AtlasExternalIntegrations::Table)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }
}

#[derive(DeriveIden)]
enum AtlasExternalIntegrations {
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
enum AtlasIntegrationEvents {
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
