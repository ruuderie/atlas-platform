#![allow(dead_code)]
use sea_orm_migration::prelude::extension::postgres::Type;
use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

/// GENERIC-03: atlas_payments — Multi-Rail Payment Ledger + Credential Management
///
/// This implements the core payment ledger (entries + splits) plus the extensible
/// payment credential storage system.
///
/// IMPORTANT DESIGN NOTES (per platform philosophy):
/// - No hard attachment to any specific payment provider.
/// - The credential_type and payment_rail values below are **examples** only.
/// - New rails/providers can be added without schema changes via the adapter pattern.
/// - Bitcoin support is designed to allow migration from third-party services
///   (mempool.space, etc.) to self-hosted nodes/infrastructure over time.
#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Payment rails (idempotent enum creation)
        manager
            .get_connection()
            .execute(sea_orm::Statement::from_string(
                manager.get_connection().get_database_backend(),
                r#"
            DO $$ BEGIN
                CREATE TYPE atlas_payment_rail AS ENUM (
                    'stripe', 'stripe_connect', 'btc_onchain', 'btc_lightning',
                    'zelle', 'cash_app', 'apple_pay', 'google_pay', 'pix',
                    'wire', 'ach', 'western_union', 'moneygram', 'cash'
                );
            EXCEPTION WHEN duplicate_object THEN NULL; END $$;
            "#
                .to_owned(),
            ))
            .await?;

        // Ledger entry status (idempotent enum creation)
        manager
            .get_connection()
            .execute(sea_orm::Statement::from_string(
                manager.get_connection().get_database_backend(),
                r#"
            DO $$ BEGIN
                CREATE TYPE atlas_ledger_status AS ENUM (
                    'pending', 'processing', 'paid', 'partially_paid',
                    'late', 'failed', 'refunded', 'waived', 'reconciled'
                );
            EXCEPTION WHEN duplicate_object THEN NULL; END $$;
            "#
                .to_owned(),
            ))
            .await?;

        // Main ledger table
        manager
            .create_table(
                Table::create()
                    .table(AtlasLedgerEntries::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(AtlasLedgerEntries::Id)
                            .uuid()
                            .not_null()
                            .primary_key()
                            .default(Expr::cust("gen_random_uuid()")),
                    )
                    .col(
                        ColumnDef::new(AtlasLedgerEntries::TenantId)
                            .uuid()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(AtlasLedgerEntries::BillableEntityType)
                            .string()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(AtlasLedgerEntries::BillableEntityId)
                            .uuid()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(AtlasLedgerEntries::PayerUserId)
                            .uuid()
                            .null(),
                    )
                    .col(
                        ColumnDef::new(AtlasLedgerEntries::PayerEmail)
                            .string()
                            .null(),
                    )
                    .col(
                        ColumnDef::new(AtlasLedgerEntries::GrossAmountCents)
                            .big_integer()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(AtlasLedgerEntries::FeeAmountCents)
                            .big_integer()
                            .not_null()
                            .default(0),
                    )
                    .col(
                        ColumnDef::new(AtlasLedgerEntries::NetAmountCents)
                            .big_integer()
                            .not_null(),
                    ) // computed in service or trigger
                    .col(
                        ColumnDef::new(AtlasLedgerEntries::Currency)
                            .char_len(3)
                            .not_null()
                            .default(Expr::val("USD")),
                    )
                    .col(
                        ColumnDef::new(AtlasLedgerEntries::PaymentRail)
                            .string_len(30)
                            .null(),
                    )
                    .col(
                        ColumnDef::new(AtlasLedgerEntries::ExternalTxId)
                            .string()
                            .null(),
                    )
                    .col(
                        ColumnDef::new(AtlasLedgerEntries::ReceiptAttachmentId)
                            .uuid()
                            .null(),
                    )
                    .col(
                        ColumnDef::new(AtlasLedgerEntries::Status)
                            .string_len(30)
                            .not_null()
                            .default(Expr::val("pending")),
                    )
                    .col(ColumnDef::new(AtlasLedgerEntries::DueDate).date().null())
                    .col(
                        ColumnDef::new(AtlasLedgerEntries::PaidAt)
                            .timestamp_with_time_zone()
                            .null(),
                    )
                    .col(
                        ColumnDef::new(AtlasLedgerEntries::VerifiedByUserId)
                            .uuid()
                            .null(),
                    )
                    .col(
                        ColumnDef::new(AtlasLedgerEntries::VerifiedAt)
                            .timestamp_with_time_zone()
                            .null(),
                    )
                    .col(
                        ColumnDef::new(AtlasLedgerEntries::ReconciledAt)
                            .timestamp_with_time_zone()
                            .null(),
                    )
                    .col(
                        ColumnDef::new(AtlasLedgerEntries::ReconciliationNote)
                            .text()
                            .null(),
                    )
                    .col(
                        ColumnDef::new(AtlasLedgerEntries::CreatedAt)
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
                    .name("idx_atlas_ledger_entries_entity")
                    .table(AtlasLedgerEntries::Table)
                    .col(AtlasLedgerEntries::TenantId)
                    .col(AtlasLedgerEntries::BillableEntityType)
                    .col(AtlasLedgerEntries::BillableEntityId)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_atlas_ledger_entries_status_due")
                    .table(AtlasLedgerEntries::Table)
                    .col(AtlasLedgerEntries::TenantId)
                    .col(AtlasLedgerEntries::Status)
                    .col(AtlasLedgerEntries::DueDate)
                    .to_owned(),
            )
            .await?;

        // Ledger splits (for multi-party payouts)
        manager
            .create_table(
                Table::create()
                    .table(AtlasLedgerSplits::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(AtlasLedgerSplits::Id)
                            .uuid()
                            .not_null()
                            .primary_key()
                            .default(Expr::cust("gen_random_uuid()")),
                    )
                    .col(
                        ColumnDef::new(AtlasLedgerSplits::LedgerEntryId)
                            .uuid()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(AtlasLedgerSplits::RecipientType)
                            .string()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(AtlasLedgerSplits::RecipientUserId)
                            .uuid()
                            .null(),
                    )
                    .col(
                        ColumnDef::new(AtlasLedgerSplits::RecipientLabel)
                            .string()
                            .null(),
                    )
                    .col(
                        ColumnDef::new(AtlasLedgerSplits::AmountCents)
                            .big_integer()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(AtlasLedgerSplits::PayoutRail)
                            .string_len(30)
                            .null(),
                    )
                    .col(
                        ColumnDef::new(AtlasLedgerSplits::PayoutStatus)
                            .string()
                            .not_null()
                            .default(Expr::val("pending")),
                    )
                    .col(
                        ColumnDef::new(AtlasLedgerSplits::PayoutTxId)
                            .string()
                            .null(),
                    )
                    .col(
                        ColumnDef::new(AtlasLedgerSplits::SettledAt)
                            .timestamp_with_time_zone()
                            .null(),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_atlas_ledger_splits_entry")
                    .table(AtlasLedgerSplits::Table)
                    .col(AtlasLedgerSplits::LedgerEntryId)
                    .to_owned(),
            )
            .await?;

        // === Payment Credentials (extensible, provider-agnostic) ===

        // Payment credential types (idempotent enum creation)
        manager
            .get_connection()
            .execute(sea_orm::Statement::from_string(
                manager.get_connection().get_database_backend(),
                r#"
            DO $$ BEGIN
                CREATE TYPE atlas_payment_credential_type AS ENUM (
                    'stripe_connect_express', 'stripe_connect_standard',
                    'btc_onchain_address', 'btc_lightning_node',
                    'zelle_account', 'pix_key'
                );
            EXCEPTION WHEN duplicate_object THEN NULL; END $$;
            "#
                .to_owned(),
            ))
            .await?;

        // Merchant of Record types (idempotent enum creation)
        manager
            .get_connection()
            .execute(sea_orm::Statement::from_string(
                manager.get_connection().get_database_backend(),
                r#"
            DO $$ BEGIN
                CREATE TYPE atlas_mor_type AS ENUM (
                    'platform', 'client', 'hybrid'
                );
            EXCEPTION WHEN duplicate_object THEN NULL; END $$;
            "#
                .to_owned(),
            ))
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(AtlasPaymentCredentials::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(AtlasPaymentCredentials::Id)
                            .uuid()
                            .not_null()
                            .primary_key()
                            .default(Expr::cust("gen_random_uuid()")),
                    )
                    .col(
                        ColumnDef::new(AtlasPaymentCredentials::TenantId)
                            .uuid()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(AtlasPaymentCredentials::CredentialType)
                            .string_len(50)
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(AtlasPaymentCredentials::MorType)
                            .string_len(30)
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(AtlasPaymentCredentials::Label)
                            .string()
                            .null(),
                    )
                    .col(
                        ColumnDef::new(AtlasPaymentCredentials::CredentialsEncrypted)
                            .json_binary()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(AtlasPaymentCredentials::DisplayIdentifier)
                            .string()
                            .null(),
                    )
                    .col(
                        ColumnDef::new(AtlasPaymentCredentials::DisplayName)
                            .string()
                            .null(),
                    )
                    .col(
                        ColumnDef::new(AtlasPaymentCredentials::IsActive)
                            .boolean()
                            .not_null()
                            .default(true),
                    )
                    .col(
                        ColumnDef::new(AtlasPaymentCredentials::IsDefaultForType)
                            .boolean()
                            .not_null()
                            .default(false),
                    )
                    .col(
                        ColumnDef::new(AtlasPaymentCredentials::IsVerified)
                            .boolean()
                            .not_null()
                            .default(false),
                    )
                    .col(
                        ColumnDef::new(AtlasPaymentCredentials::VerifiedAt)
                            .timestamp_with_time_zone()
                            .null(),
                    )
                    .col(
                        ColumnDef::new(AtlasPaymentCredentials::PayoutCurrency)
                            .char_len(3)
                            .not_null()
                            .default(Expr::val("USD")),
                    )
                    .col(
                        ColumnDef::new(AtlasPaymentCredentials::PayoutMinimumCents)
                            .big_integer()
                            .not_null()
                            .default(0),
                    )
                    .col(
                        ColumnDef::new(AtlasPaymentCredentials::WebhookSecretEnc)
                            .string()
                            .null(),
                    )
                    .col(
                        ColumnDef::new(AtlasPaymentCredentials::CreatedByUserId)
                            .uuid()
                            .null(),
                    )
                    .col(
                        ColumnDef::new(AtlasPaymentCredentials::CreatedAt)
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
                    .name("idx_atlas_payment_credentials_tenant")
                    .table(AtlasPaymentCredentials::Table)
                    .col(AtlasPaymentCredentials::TenantId)
                    .col(AtlasPaymentCredentials::CredentialType)
                    .col(AtlasPaymentCredentials::IsActive)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(
                Table::drop()
                    .table(AtlasPaymentCredentials::Table)
                    .to_owned(),
            )
            .await?;
        manager
            .drop_table(Table::drop().table(AtlasLedgerSplits::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(AtlasLedgerEntries::Table).to_owned())
            .await?;

        manager
            .drop_type(Type::drop().name(AtlasMorType::Table).to_owned())
            .await?;
        manager
            .drop_type(
                Type::drop()
                    .name(AtlasPaymentCredentialType::Table)
                    .to_owned(),
            )
            .await?;
        manager
            .drop_type(Type::drop().name(AtlasLedgerStatus::Table).to_owned())
            .await?;
        manager
            .drop_type(Type::drop().name(AtlasPaymentRail::Table).to_owned())
            .await?;

        Ok(())
    }
}

#[derive(DeriveIden)]
enum AtlasLedgerEntries {
    Table,
    Id,
    TenantId,
    BillableEntityType,
    BillableEntityId,
    PayerUserId,
    PayerEmail,
    GrossAmountCents,
    FeeAmountCents,
    NetAmountCents,
    Currency,
    PaymentRail,
    ExternalTxId,
    ReceiptAttachmentId,
    Status,
    DueDate,
    PaidAt,
    VerifiedByUserId,
    VerifiedAt,
    ReconciledAt,
    ReconciliationNote,
    CreatedAt,
}

#[derive(DeriveIden)]
enum AtlasLedgerSplits {
    Table,
    Id,
    LedgerEntryId,
    RecipientType,
    RecipientUserId,
    RecipientLabel,
    AmountCents,
    PayoutRail,
    PayoutStatus,
    PayoutTxId,
    SettledAt,
}

#[derive(DeriveIden)]
enum AtlasPaymentCredentials {
    Table,
    Id,
    TenantId,
    CredentialType,
    MorType,
    Label,
    CredentialsEncrypted,
    DisplayIdentifier,
    DisplayName,
    IsActive,
    IsDefaultForType,
    IsVerified,
    VerifiedAt,
    PayoutCurrency,
    PayoutMinimumCents,
    WebhookSecretEnc,
    CreatedByUserId,
    CreatedAt,
}

#[derive(DeriveIden)]
enum AtlasPaymentRail {
    Table,
    Stripe,
    StripeConnect,
    BtcOnchain,
    BtcLightning,
    Zelle,
    CashApp,
    ApplePay,
    GooglePay,
    Pix,
    Wire,
    Ach,
    WesternUnion,
    Moneygram,
    Cash,
}

#[derive(DeriveIden)]
enum AtlasLedgerStatus {
    Table,
    Pending,
    Processing,
    Paid,
    PartiallyPaid,
    Late,
    Failed,
    Refunded,
    Waived,
    Reconciled,
}

#[derive(DeriveIden)]
enum AtlasPaymentCredentialType {
    Table,
    StripeConnectExpress,
    StripeConnectStandard,
    BtcOnchainAddress,
    BtcLightningNode,
    ZelleAccount,
    PixKey,
}

#[derive(DeriveIden)]
enum AtlasMorType {
    Table,
    Platform,
    Client,
    Hybrid,
}
