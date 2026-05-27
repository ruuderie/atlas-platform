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
                    .table(AtlasAccount::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(AtlasAccount::Id)
                            .uuid()
                            .not_null()
                            .primary_key()
                            .default(Expr::cust("gen_random_uuid()")),
                    )
                    .col(ColumnDef::new(AtlasAccount::TenantId).uuid().not_null())
                    .col(ColumnDef::new(AtlasAccount::AccountType).string().not_null())
                    .col(ColumnDef::new(AtlasAccount::Name).string().not_null())
                    .col(ColumnDef::new(AtlasAccount::FirstName).string().null())
                    .col(ColumnDef::new(AtlasAccount::LastName).string().null())
                    .col(ColumnDef::new(AtlasAccount::PrimaryContactId).uuid().null())
                    .col(ColumnDef::new(AtlasAccount::Status).string().not_null().default(Expr::val("active")))
                    .col(ColumnDef::new(AtlasAccount::Attributes).json_binary().null())
                    .col(
                        ColumnDef::new(AtlasAccount::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .col(
                        ColumnDef::new(AtlasAccount::UpdatedAt)
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
                    .table(AtlasAccount::Table)
                    .col(AtlasAccount::TenantId)
                    .col(AtlasAccount::Status)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_atlas_accounts_type")
                    .table(AtlasAccount::Table)
                    .col(AtlasAccount::TenantId)
                    .col(AtlasAccount::AccountType)
                    .to_owned(),
            )
            .await?;

        // === atlas_contacts ===
        manager
            .create_table(
                Table::create()
                    .table(AtlasContact::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(AtlasContact::Id)
                            .uuid()
                            .not_null()
                            .primary_key()
                            .default(Expr::cust("gen_random_uuid()")),
                    )
                    .col(ColumnDef::new(AtlasContact::TenantId).uuid().not_null())
                    .col(ColumnDef::new(AtlasContact::AccountId).uuid().not_null())
                    .col(ColumnDef::new(AtlasContact::FirstName).string().null())
                    .col(ColumnDef::new(AtlasContact::LastName).string().null())
                    .col(ColumnDef::new(AtlasContact::FullName).string().null())
                    .col(ColumnDef::new(AtlasContact::Email).string().null())
                    .col(ColumnDef::new(AtlasContact::Phone).string().null())
                    .col(ColumnDef::new(AtlasContact::Title).string().null())
                    .col(ColumnDef::new(AtlasContact::Department).string().null())
                    .col(ColumnDef::new(AtlasContact::IsPrimary).boolean().not_null().default(false))
                    .col(ColumnDef::new(AtlasContact::ContactMetadata).json_binary().null())
                    .col(
                        ColumnDef::new(AtlasContact::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .col(
                        ColumnDef::new(AtlasContact::UpdatedAt)
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
                    .table(AtlasContact::Table)
                    .col(AtlasContact::AccountId)
                    .col(AtlasContact::IsPrimary)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_atlas_contacts_tenant")
                    .table(AtlasContact::Table)
                    .col(AtlasContact::TenantId)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_atlas_contacts_email")
                    .table(AtlasContact::Table)
                    .col(AtlasContact::TenantId)
                    .col(AtlasContact::Email)
                    .to_owned(),
            )
            .await?;

        // Foreign key from contacts -> accounts (contacts depend on accounts)
        manager
            .alter_table(
                Table::alter()
                    .table(AtlasContact::Table)
                    .add_foreign_key(
                        ForeignKey::create()
                            .name("fk_atlas_contacts_account_id")
                            .from(AtlasContact::Table, AtlasContact::AccountId)
                            .to(AtlasAccount::Table, AtlasAccount::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .to_owned(),
                    )
                    .to_owned(),
            )
            .await?;

        // Circular FK: accounts.primary_contact_id -> contacts (after contacts table exists)
        manager
            .alter_table(
                Table::alter()
                    .table(AtlasAccount::Table)
                    .add_foreign_key(
                        ForeignKey::create()
                            .name("fk_atlas_accounts_primary_contact_id")
                            .from(AtlasAccount::Table, AtlasAccount::PrimaryContactId)
                            .to(AtlasContact::Table, AtlasContact::Id)
                            .on_delete(ForeignKeyAction::SetNull)
                            .to_owned(),
                    )
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(AtlasContact::Table).to_owned())
            .await?;

        manager
            .drop_table(Table::drop().table(AtlasAccount::Table).to_owned())
            .await?;

        Ok(())
    }
}

#[derive(DeriveIden)]
enum AtlasAccount {
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
enum AtlasContact {
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
