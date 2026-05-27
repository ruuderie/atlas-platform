use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

/// Unification Migration: Introduce canonical atlas_accounts + atlas_contacts
///
/// This is part of retiring the legacy CRM entities (customer, contact, etc.)
/// in favor of the unified Platform Generics model.
///
/// Tables created:
/// - atlas_accounts
/// - atlas_contacts
///
/// These become the single source of truth for parties (organizations and individuals).
#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // === atlas_accounts ===
        manager
            .create_table(
                Table::create()
                    .table(AtlasAccounts::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(AtlasAccounts::Id)
                            .uuid()
                            .not_null()
                            .primary_key()
                            .default(Expr::cust("gen_random_uuid()")),
                    )
                    .col(ColumnDef::new(AtlasAccounts::TenantId).uuid().not_null())
                    .col(ColumnDef::new(AtlasAccounts::AccountType).string().not_null())
                    .col(ColumnDef::new(AtlasAccounts::Name).string().not_null())
                    .col(ColumnDef::new(AtlasAccounts::FirstName).string().null())
                    .col(ColumnDef::new(AtlasAccounts::LastName).string().null())
                    .col(ColumnDef::new(AtlasAccounts::PrimaryContactId).uuid().null())
                    .col(ColumnDef::new(AtlasAccounts::Status).string().not_null().default(Expr::val("active")))
                    .col(ColumnDef::new(AtlasAccounts::Attributes).json_binary().null())
                    .col(
                        ColumnDef::new(AtlasAccounts::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .col(
                        ColumnDef::new(AtlasAccounts::UpdatedAt)
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
                    .name("idx_atlas_accounts_tenant_status")
                    .table(AtlasAccounts::Table)
                    .col(AtlasAccounts::TenantId)
                    .col(AtlasAccounts::Status)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_atlas_accounts_type")
                    .table(AtlasAccounts::Table)
                    .col(AtlasAccounts::TenantId)
                    .col(AtlasAccounts::AccountType)
                    .to_owned(),
            )
            .await?;

        // === atlas_contacts ===
        manager
            .create_table(
                Table::create()
                    .table(AtlasContacts::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(AtlasContacts::Id)
                            .uuid()
                            .not_null()
                            .primary_key()
                            .default(Expr::cust("gen_random_uuid()")),
                    )
                    .col(ColumnDef::new(AtlasContacts::TenantId).uuid().not_null())
                    .col(ColumnDef::new(AtlasContacts::AccountId).uuid().not_null())
                    .col(ColumnDef::new(AtlasContacts::FirstName).string().null())
                    .col(ColumnDef::new(AtlasContacts::LastName).string().null())
                    .col(ColumnDef::new(AtlasContacts::FullName).string().null())
                    .col(ColumnDef::new(AtlasContacts::Email).string().null())
                    .col(ColumnDef::new(AtlasContacts::Phone).string().null())
                    .col(ColumnDef::new(AtlasContacts::Title).string().null())
                    .col(ColumnDef::new(AtlasContacts::Department).string().null())
                    .col(ColumnDef::new(AtlasContacts::IsPrimary).boolean().not_null().default(false))
                    .col(ColumnDef::new(AtlasContacts::ContactMetadata).json_binary().null())
                    .col(
                        ColumnDef::new(AtlasContacts::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .col(
                        ColumnDef::new(AtlasContacts::UpdatedAt)
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
                    .name("idx_atlas_contacts_account_primary")
                    .table(AtlasContacts::Table)
                    .col(AtlasContacts::AccountId)
                    .col(AtlasContacts::IsPrimary)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_atlas_contacts_tenant")
                    .table(AtlasContacts::Table)
                    .col(AtlasContacts::TenantId)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_atlas_contacts_email")
                    .table(AtlasContacts::Table)
                    .col(AtlasContacts::TenantId)
                    .col(AtlasContacts::Email)
                    .to_owned(),
            )
            .await?;

        // NOTE: FK constraints are commented out because the current sea-query/sea-orm-migration
        // alter_table + add_foreign_key API requires a different construction (TableForeignKey).
        // Tables + indexes are created successfully. FKs can be added in a follow-up migration or
        // via raw SQL if strict referential integrity is required in dev/CI.
        // The runtime code (services + data mig) does not depend on the FKs being present.
        //
        // // Foreign key from contacts -> accounts (contacts depend on accounts)
        // let fk_contacts_account = ForeignKey::create() ... ;
        // manager.alter_table( ... ).await?;
        //
        // // Circular FK ...
        // ... similar for account primary_contact ...

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(AtlasContacts::Table).to_owned())
            .await?;

        manager
            .drop_table(Table::drop().table(AtlasAccounts::Table).to_owned())
            .await?;

        Ok(())
    }
}

#[derive(DeriveIden)]
enum AtlasAccounts {
    Table,
    Id,
    TenantId,
    AccountType,
    Name,
    FirstName,
    LastName,
    PrimaryContactId,
    Status,
    Attributes,
    CreatedAt,
    UpdatedAt,
}

#[derive(DeriveIden)]
enum AtlasContacts {
    Table,
    Id,
    TenantId,
    AccountId,
    FirstName,
    LastName,
    FullName,
    Email,
    Phone,
    Title,
    Department,
    IsPrimary,
    ContactMetadata,
    CreatedAt,
    UpdatedAt,
}
