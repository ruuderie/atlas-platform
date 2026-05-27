use sea_orm_migration::prelude::*;
use sea_orm_migration::prelude::extension::postgres::Type;

#[derive(DeriveMigrationName)]
pub struct Migration;

/// GENERIC-16: atlas_regulatory_registrations — Permits, Licenses, Government Registrations
/// STR permits, contractor licenses, insurance licenses, vehicle registrations, etc.
#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_type(
                Type::create()
                    .as_enum(AtlasRegStatus::Table)
                    .values([
                        AtlasRegStatus::PendingApplication,
                        AtlasRegStatus::UnderReview,
                        AtlasRegStatus::Active,
                        AtlasRegStatus::Suspended,
                        AtlasRegStatus::Expired,
                        AtlasRegStatus::Revoked,
                        AtlasRegStatus::Exempt,
                        AtlasRegStatus::NonCompliant,
                    ])
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(AtlasRegulatoryRegistrations::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(AtlasRegulatoryRegistrations::Id)
                            .uuid()
                            .not_null()
                            .primary_key()
                            .default(Expr::cust("gen_random_uuid()")),
                    )
                    .col(ColumnDef::new(AtlasRegulatoryRegistrations::TenantId).uuid().not_null())
                    .col(ColumnDef::new(AtlasRegulatoryRegistrations::RegistrationType).string().not_null())
                    .col(ColumnDef::new(AtlasRegulatoryRegistrations::AssetId).uuid().null())
                    .col(ColumnDef::new(AtlasRegulatoryRegistrations::ServiceProviderId).uuid().null())
                    .col(ColumnDef::new(AtlasRegulatoryRegistrations::JurisdictionCode).string().not_null())
                    .col(ColumnDef::new(AtlasRegulatoryRegistrations::IssuingAuthority).string().null())
                    .col(ColumnDef::new(AtlasRegulatoryRegistrations::RegistrationNumber).string().not_null())
                    .col(ColumnDef::new(AtlasRegulatoryRegistrations::VerificationRequestId).uuid().null())
                    .col(
                        ColumnDef::new(AtlasRegulatoryRegistrations::Status)
                            .string_len(30)
                            .not_null()
                            .default(Expr::val("pending_application")),
                    )
                    .col(ColumnDef::new(AtlasRegulatoryRegistrations::IssuedDate).date().null())
                    .col(ColumnDef::new(AtlasRegulatoryRegistrations::ExpiresAt).date().null())
                    .col(ColumnDef::new(AtlasRegulatoryRegistrations::LastInspectionDate).date().null())
                    .col(ColumnDef::new(AtlasRegulatoryRegistrations::NextInspectionDue).date().null())
                    .col(ColumnDef::new(AtlasRegulatoryRegistrations::AccessToken).string().null())
                    .col(ColumnDef::new(AtlasRegulatoryRegistrations::AccessTokenExpiresAt).timestamp_with_time_zone().null())
                    .col(ColumnDef::new(AtlasRegulatoryRegistrations::JurisdictionMetadata).json_binary().null())
                    .col(
                        ColumnDef::new(AtlasRegulatoryRegistrations::CreatedAt)
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
                    .name("idx_atlas_reg_tenant_type_status")
                    .table(AtlasRegulatoryRegistrations::Table)
                    .col(AtlasRegulatoryRegistrations::TenantId)
                    .col(AtlasRegulatoryRegistrations::RegistrationType)
                    .col(AtlasRegulatoryRegistrations::Status)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_atlas_reg_expiry")
                    .table(AtlasRegulatoryRegistrations::Table)
                    .col(AtlasRegulatoryRegistrations::ExpiresAt)
                    .col(AtlasRegulatoryRegistrations::Status)
                    .to_owned(),
            )
            .await?;

        // Unique constraint
        manager
            .create_index(
                Index::create()
                    .name("uq_atlas_reg_type_number_jurisdiction")
                    .table(AtlasRegulatoryRegistrations::Table)
                    .col(AtlasRegulatoryRegistrations::TenantId)
                    .col(AtlasRegulatoryRegistrations::RegistrationType)
                    .col(AtlasRegulatoryRegistrations::RegistrationNumber)
                    .col(AtlasRegulatoryRegistrations::JurisdictionCode)
                    .unique()
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(AtlasRegulatoryRegistrations::Table).to_owned())
            .await?;

        manager
            .drop_type(Type::drop().name(AtlasRegStatus::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum AtlasRegulatoryRegistrations {
    Table,
    Id,
    TenantId,
    RegistrationType,
    AssetId,
    ServiceProviderId,
    JurisdictionCode,
    IssuingAuthority,
    RegistrationNumber,
    VerificationRequestId,
    Status,
    IssuedDate,
    ExpiresAt,
    LastInspectionDate,
    NextInspectionDue,
    AccessToken,
    AccessTokenExpiresAt,
    JurisdictionMetadata,
    CreatedAt,
}

#[derive(DeriveIden)]
enum AtlasRegStatus {
    Table,
    PendingApplication,
    UnderReview,
    Active,
    Suspended,
    Expired,
    Revoked,
    Exempt,
    NonCompliant,
}
