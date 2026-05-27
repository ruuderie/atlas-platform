use sea_orm_migration::prelude::*;
use sea_orm_migration::prelude::extension::postgres::Type;

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
        // Payment rails (examples — extensible)
        manager
            .create_type(
                Type::create()
                    .as_enum(AtlasPaymentRail::Table)
                    .values([
                        AtlasPaymentRail::Stripe,
                        AtlasPaymentRail::StripeConnect,
                        AtlasPaymentRail::BtcOnchain,
                        AtlasPaymentRail::BtcLightning,
                        AtlasPaymentRail::Zelle,
                        AtlasPaymentRail::CashApp,
                        AtlasPaymentRail::ApplePay,
                        AtlasPaymentRail::GooglePay,
                        AtlasPaymentRail::Pix,
                        AtlasPaymentRail::Wire,
                        AtlasPaymentRail::Ach,
                        AtlasPaymentRail::WesternUnion,
                        AtlasPaymentRail::Moneygram,
                        AtlasPaymentRail::Cash,
                    ])
                    .to_owned(),
            )
            .await?;

        // Ledger entry status
        manager
            .create_type(
                Type::create()
                    .as_enum(AtlasLedgerStatus::Table)
                    .values([
                        AtlasLedgerStatus::Pending,
                        AtlasLedgerStatus::Processing,
                        AtlasLedgerStatus::Paid,
                        AtlasLedgerStatus::PartiallyPaid,
                        AtlasLedgerStatus::Late,
                        AtlasLedgerStatus::Failed,
                        AtlasLedgerStatus::Refunded,
                        AtlasLedgerStatus::Waived,
                        AtlasLedgerStatus::Reconciled,
                    ])
                    .to_owned(),
            )
            .await?;

        // Main ledger table
        manager
            .create_table(
                Table::create()
                    .table(AtlasLedgerEntry::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(AtlasLedgerEntry::Id)
                            .uuid()
                            .not_null()
                            .primary_key()
                            .default(Expr::cust("gen_random_uuid()")),
                    )
                    .col(ColumnDef::new(AtlasLedgerEntry::TenantId).uuid().not_null())
                    .col(ColumnDef::new(AtlasLedgerEntry::BillableEntityType).string().not_null())
                    .col(ColumnDef::new(AtlasLedgerEntry::BillableEntityId).uuid().not_null())
                    .col(ColumnDef::new(AtlasLedgerEntry::PayerUserId).uuid().null())
                    .col(ColumnDef::new(AtlasLedgerEntry::PayerEmail).string().null())
                    .col(ColumnDef::new(AtlasLedgerEntry::GrossAmountCents).big_integer().not_null())
                    .col(ColumnDef::new(AtlasLedgerEntry::FeeAmountCents).big_integer().not_null().default(0))
                    .col(ColumnDef::new(AtlasLedgerEntry::NetAmountCents).big_integer().not_null()) // computed in service or trigger
                    .col(ColumnDef::new(AtlasLedgerEntry::Currency).char_len(3).not_null().default(Expr::val("USD")))
                    .col(ColumnDef::new(AtlasLedgerEntry::PaymentRail).custom(AtlasPaymentRail::Table).null())
                    .col(ColumnDef::new(AtlasLedgerEntry::ExternalTxId).string().null())
                    .col(ColumnDef::new(AtlasLedgerEntry::ReceiptAttachmentId).uuid().null())
                    .col(
                        ColumnDef::new(AtlasLedgerEntry::Status)
                            .custom(AtlasLedgerStatus::Table)
                            .not_null()
                            .default(Expr::val("pending")),
                    )
                    .col(ColumnDef::new(AtlasLedgerEntry::DueDate).date().null())
                    .col(ColumnDef::new(AtlasLedgerEntry::PaidAt).timestamp_with_time_zone().null())
                    .col(ColumnDef::new(AtlasLedgerEntry::VerifiedByUserId).uuid().null())
                    .col(ColumnDef::new(AtlasLedgerEntry::VerifiedAt).timestamp_with_time_zone().null())
                    .col(ColumnDef::new(AtlasLedgerEntry::ReconciledAt).timestamp_with_time_zone().null())
                    .col(ColumnDef::new(AtlasLedgerEntry::ReconciliationNote).text().null())
                    .col(
                        ColumnDef::new(AtlasLedgerEntry::CreatedAt)
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
                    .table(AtlasLedgerEntry::Table)
                    .col(AtlasLedgerEntry::TenantId)
                    .col(AtlasLedgerEntry::BillableEntityType)
                    .col(AtlasLedgerEntry::BillableEntityId)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_atlas_ledger_entries_status_due")
                    .table(AtlasLedgerEntry::Table)
                    .col(AtlasLedgerEntry::TenantId)
                    .col(AtlasLedgerEntry::Status)
                    .col(AtlasLedgerEntry::DueDate)
                    .to_owned(),
            )
            .await?;

        // Ledger splits (for multi-party payouts)
        manager
            .create_table(
                Table::create()
                    .table(AtlasLedgerSplit::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(AtlasLedgerSplit::Id)
                            .uuid()
                            .not_null()
                            .primary_key()
                            .default(Expr::cust("gen_random_uuid()")),
                    )
                    .col(ColumnDef::new(AtlasLedgerSplit::LedgerEntryId).uuid().not_null())
                    .col(ColumnDef::new(AtlasLedgerSplit::RecipientType).string().not_null())
                    .col(ColumnDef::new(AtlasLedgerSplit::RecipientUserId).uuid().null())
                    .col(ColumnDef::new(AtlasLedgerSplit::RecipientLabel).string().null())
                    .col(ColumnDef::new(AtlasLedgerSplit::AmountCents).big_integer().not_null())
                    .col(ColumnDef::new(AtlasLedgerSplit::PayoutRail).custom(AtlasPaymentRail::Table).null())
                    .col(ColumnDef::new(AtlasLedgerSplit::PayoutStatus).string().not_null().default(Expr::val("pending")))
                    .col(ColumnDef::new(AtlasLedgerSplit::PayoutTxId).string().null())
                    .col(ColumnDef::new(AtlasLedgerSplit::SettledAt).timestamp_with_time_zone().null())
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_atlas_ledger_splits_entry")
                    .table(AtlasLedgerSplit::Table)
                    .col(AtlasLedgerSplit::LedgerEntryId)
                    .to_owned(),
            )
            .await?;

        // === Payment Credentials (extensible, provider-agnostic) ===

        manager
            .create_type(
                Type::create()
                    .as_enum(AtlasPaymentCredentialType::Table)
                    .values([
                        // The values below are illustrative examples only.
                        // The platform is deliberately not tied to any specific provider.
                        AtlasPaymentCredentialType::StripeConnectExpress,
                        AtlasPaymentCredentialType::StripeConnectStandard,
                        AtlasPaymentCredentialType::BtcOnchainAddress,
                        AtlasPaymentCredentialType::BtcLightningNode,
                        AtlasPaymentCredentialType::ZelleAccount,
                        AtlasPaymentCredentialType::PixKey,
                        // Add more as needed via future migrations or by treating unknown strings gracefully
                    ])
                    .to_owned(),
            )
            .await?;

        manager
            .create_type(
                Type::create()
                    .as_enum(AtlasMorType::Table)
                    .values([
                        AtlasMorType::Platform,
                        AtlasMorType::Client,
                        AtlasMorType::Hybrid,
                    ])
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(AtlasPaymentCredential::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(AtlasPaymentCredential::Id)
                            .uuid()
                            .not_null()
                            .primary_key()
                            .default(Expr::cust("gen_random_uuid()")),
                    )
                    .col(ColumnDef::new(AtlasPaymentCredential::TenantId).uuid().not_null())
                    .col(ColumnDef::new(AtlasPaymentCredential::CredentialType).custom(AtlasPaymentCredentialType::Table).not_null())
                    .col(ColumnDef::new(AtlasPaymentCredential::MorType).custom(AtlasMorType::Table).not_null())
                    .col(ColumnDef::new(AtlasPaymentCredential::Label).string().null())
                    .col(ColumnDef::new(AtlasPaymentCredential::CredentialsEncrypted).json_binary().not_null())
                    .col(ColumnDef::new(AtlasPaymentCredential::DisplayIdentifier).string().null())
                    .col(ColumnDef::new(AtlasPaymentCredential::DisplayName).string().null())
                    .col(ColumnDef::new(AtlasPaymentCredential::IsActive).boolean().not_null().default(true))
                    .col(ColumnDef::new(AtlasPaymentCredential::IsDefaultForType).boolean().not_null().default(false))
                    .col(ColumnDef::new(AtlasPaymentCredential::IsVerified).boolean().not_null().default(false))
                    .col(ColumnDef::new(AtlasPaymentCredential::VerifiedAt).timestamp_with_time_zone().null())
                    .col(ColumnDef::new(AtlasPaymentCredential::PayoutCurrency).char_len(3).not_null().default(Expr::val("USD")))
                    .col(ColumnDef::new(AtlasPaymentCredential::PayoutMinimumCents).big_integer().not_null().default(0))
                    .col(ColumnDef::new(AtlasPaymentCredential::WebhookSecretEnc).string().null())
                    .col(ColumnDef::new(AtlasPaymentCredential::CreatedByUserId).uuid().null())
                    .col(
                        ColumnDef::new(AtlasPaymentCredential::CreatedAt)
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
                    .table(AtlasPaymentCredential::Table)
                    .col(AtlasPaymentCredential::TenantId)
                    .col(AtlasPaymentCredential::CredentialType)
                    .col(AtlasPaymentCredential::IsActive)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(AtlasPaymentCredential::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(AtlasLedgerSplit::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(AtlasLedgerEntry::Table).to_owned())
            .await?;

        manager
            .drop_type(Type::drop().name(AtlasMorType::Table).to_owned())
            .await?;
        manager
            .drop_type(Type::drop().name(AtlasPaymentCredentialType::Table).to_owned())
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
enum AtlasLedgerEntry {
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
enum AtlasLedgerSplit {
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
enum AtlasPaymentCredential {
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
