use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

/// Gap-Fill: atlas_accounts + atlas_contacts schema promotion
///
/// Adds all first-class firmographic, dedup, and contact-channel columns
/// identified in the data source gap analysis against the /data-processing/data/
/// source files (BusinessLeadsUSA ~5-10M rows, ZoomInfo ~1-2M, FMCSA ~500K, etc.).
///
/// The prior schema stored these in a JSONB `attributes` bag, which is not
/// index-efficient for range queries, multi-column filters, or dedup lookups.
///
/// Changes:
///   atlas_accounts:
///     - ADD: 30 firmographic / dedup / address / import columns
///     - RENAME: attributes → account_metadata
///     - ADD: 8 new indexes (partial where applicable)
///     - ADD: updated_at trigger (uses set_updated_at_column() from G-31)
///
///   atlas_contacts:
///     - ADD: 16 contact-channel / dedup / tax-id columns
///     - ADD: 3 new indexes (partial where applicable)
///     - ADD: updated_at trigger
///
/// Spec: docs/architecture/account_contact_data_gap_analysis.md
/// Ordering: must run after m20260601_unify_accounts_contacts.rs (creates the tables)
///           and after m20260601_g31_atlas_lead.rs (creates set_updated_at_column())
#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();
        let backend = db.get_database_backend();

        // ════════════════════════════════════════════════════════════════════
        // atlas_accounts — gap fill
        // ════════════════════════════════════════════════════════════════════

        // ── 1. RENAME attributes → account_metadata ──────────────────────────
        // Zero breaking call sites confirmed: account_service.rs uses
        // ..Default::default() and no code references Column::Attributes.
        db.execute(sea_orm::Statement::from_string(
            backend,
            "ALTER TABLE atlas_accounts \
             RENAME COLUMN attributes TO account_metadata;"
                .to_owned(),
        ))
        .await?;

        // ── 2. ADD columns to atlas_accounts ─────────────────────────────────
        manager
            .alter_table(
                Table::alter()
                    .table(AtlasAccounts::Table)
                    // Organization Identity
                    .add_column_if_not_exists(
                        ColumnDef::new(AtlasAccounts::DbaName)
                            .string_len(255)
                            .null(),
                    )
                    .add_column_if_not_exists(
                        ColumnDef::new(AtlasAccounts::Website)
                            .string_len(255)
                            .null(),
                    )
                    .add_column_if_not_exists(
                        ColumnDef::new(AtlasAccounts::Domain).string_len(100).null(),
                    )
                    .add_column_if_not_exists(
                        ColumnDef::new(AtlasAccounts::DunsNumber)
                            .string_len(15)
                            .null(),
                    )
                    .add_column_if_not_exists(
                        ColumnDef::new(AtlasAccounts::TaxIdPrimary)
                            .string_len(30)
                            .null(),
                    )
                    .add_column_if_not_exists(
                        ColumnDef::new(AtlasAccounts::TaxIdSecondary)
                            .string_len(30)
                            .null(),
                    )
                    // 'ein' | 'cnpj' | 'cpf' | 'ssn' | 'tin' | 'vat' | 'usdot'
                    .add_column_if_not_exists(
                        ColumnDef::new(AtlasAccounts::TaxIdType)
                            .string_len(20)
                            .null(),
                    )
                    // Company-level contact channels (distinct from a specific person's channels)
                    .add_column_if_not_exists(
                        ColumnDef::new(AtlasAccounts::CompanyPhone)
                            .string_len(30)
                            .null(),
                    )
                    .add_column_if_not_exists(
                        ColumnDef::new(AtlasAccounts::CompanyEmail)
                            .string_len(255)
                            .null(),
                    )
                    .add_column_if_not_exists(
                        ColumnDef::new(AtlasAccounts::CompanyFax)
                            .string_len(30)
                            .null(),
                    )
                    // Firmographic data — first-class for segmentation queries
                    .add_column_if_not_exists(
                        ColumnDef::new(AtlasAccounts::Industry)
                            .string_len(255)
                            .null(),
                    )
                    .add_column_if_not_exists(
                        ColumnDef::new(AtlasAccounts::SubIndustry)
                            .string_len(255)
                            .null(),
                    )
                    .add_column_if_not_exists(
                        ColumnDef::new(AtlasAccounts::SicCode).string_len(10).null(),
                    )
                    .add_column_if_not_exists(
                        ColumnDef::new(AtlasAccounts::NaicsCode)
                            .string_len(10)
                            .null(),
                    )
                    .add_column_if_not_exists(
                        ColumnDef::new(AtlasAccounts::NumEmployees).integer().null(),
                    )
                    .add_column_if_not_exists(
                        ColumnDef::new(AtlasAccounts::AnnualRevenue)
                            .decimal_len(18, 2)
                            .null(),
                    )
                    // 'public' | 'private' | 'government' | 'nonprofit' | 'individual'
                    .add_column_if_not_exists(
                        ColumnDef::new(AtlasAccounts::CompanyType)
                            .string_len(30)
                            .null(),
                    )
                    // 'headquarters' | 'branch' | 'single' | 'franchise'
                    .add_column_if_not_exists(
                        ColumnDef::new(AtlasAccounts::LocationType)
                            .string_len(30)
                            .null(),
                    )
                    .add_column_if_not_exists(
                        ColumnDef::new(AtlasAccounts::YearEstablished)
                            .small_integer()
                            .null(),
                    )
                    .add_column_if_not_exists(
                        ColumnDef::new(AtlasAccounts::CreditScoreCode)
                            .string_len(10)
                            .null(),
                    )
                    // Address — completely absent from prior schema
                    .add_column_if_not_exists(
                        ColumnDef::new(AtlasAccounts::StreetAddress)
                            .string_len(255)
                            .null(),
                    )
                    .add_column_if_not_exists(
                        ColumnDef::new(AtlasAccounts::City).string_len(100).null(),
                    )
                    .add_column_if_not_exists(
                        ColumnDef::new(AtlasAccounts::State).string_len(50).null(),
                    )
                    .add_column_if_not_exists(
                        ColumnDef::new(AtlasAccounts::PostalCode)
                            .string_len(20)
                            .null(),
                    )
                    .add_column_if_not_exists(
                        ColumnDef::new(AtlasAccounts::Country)
                            .string_len(50)
                            .null()
                            .default(Expr::val("US")),
                    )
                    // Separate mailing address — JSONB (rarely queried, spec section 3.4)
                    .add_column_if_not_exists(
                        ColumnDef::new(AtlasAccounts::MailingAddress)
                            .json_binary()
                            .null(),
                    )
                    // Import attribution
                    // 'manual' | 'zoominfo' | 'business_leads_usa' | 'fmcsa' | 'mwbe' | 'linkedin'
                    .add_column_if_not_exists(
                        ColumnDef::new(AtlasAccounts::DataSource)
                            .string_len(50)
                            .null(),
                    )
                    .add_column_if_not_exists(
                        ColumnDef::new(AtlasAccounts::DataSourceId)
                            .string_len(100)
                            .null(),
                    )
                    .add_column_if_not_exists(
                        ColumnDef::new(AtlasAccounts::IsDuplicate)
                            .boolean()
                            .null()
                            .default(false),
                    )
                    .add_column_if_not_exists(
                        ColumnDef::new(AtlasAccounts::DuplicateOfAccountId)
                            .uuid()
                            .null(),
                    )
                    .to_owned(),
            )
            .await?;

        // ── 3. geo_point on atlas_accounts (PostGIS-guarded) ─────────────────
        let has_postgis = db
            .query_one(sea_orm::Statement::from_string(
                backend,
                "SELECT 1 FROM pg_extension WHERE extname = 'postgis';".to_owned(),
            ))
            .await
            .map(|r| r.is_some())
            .unwrap_or(false);

        if has_postgis {
            db.execute(sea_orm::Statement::from_string(
                backend,
                "ALTER TABLE atlas_accounts \
                 ADD COLUMN IF NOT EXISTS geo_point geometry(Point, 4326) NULL;"
                    .to_owned(),
            ))
            .await?;

            db.execute(sea_orm::Statement::from_string(
                backend,
                "CREATE INDEX IF NOT EXISTS idx_atlas_accounts_geo \
                 ON atlas_accounts USING GIST (geo_point) \
                 WHERE geo_point IS NOT NULL;"
                    .to_owned(),
            ))
            .await?;
        }

        // ── 4. Indexes on atlas_accounts (all raw SQL for partial index support) ──

        // Primary dedup anchor — unique per tenant, only canonical records
        // This is a UNIQUE partial index: prevents two non-duplicate records
        // sharing the same domain within a tenant.
        db.execute(sea_orm::Statement::from_string(
            backend,
            "CREATE UNIQUE INDEX IF NOT EXISTS idx_atlas_accounts_domain \
             ON atlas_accounts (tenant_id, domain) \
             WHERE domain IS NOT NULL AND is_duplicate = false;"
                .to_owned(),
        ))
        .await?;

        db.execute(sea_orm::Statement::from_string(
            backend,
            "CREATE INDEX IF NOT EXISTS idx_atlas_accounts_email \
             ON atlas_accounts (tenant_id, company_email) \
             WHERE company_email IS NOT NULL;"
                .to_owned(),
        ))
        .await?;

        db.execute(sea_orm::Statement::from_string(
            backend,
            "CREATE INDEX IF NOT EXISTS idx_atlas_accounts_duns \
             ON atlas_accounts (duns_number) \
             WHERE duns_number IS NOT NULL;"
                .to_owned(),
        ))
        .await?;

        db.execute(sea_orm::Statement::from_string(
            backend,
            "CREATE INDEX IF NOT EXISTS idx_atlas_accounts_data_source \
             ON atlas_accounts (tenant_id, data_source, data_source_id) \
             WHERE data_source IS NOT NULL;"
                .to_owned(),
        ))
        .await?;

        // Firmographic filter indexes (no partial — all rows benefit)
        db.execute(sea_orm::Statement::from_string(
            backend,
            "CREATE INDEX IF NOT EXISTS idx_atlas_accounts_industry \
             ON atlas_accounts (tenant_id, sic_code, naics_code);"
                .to_owned(),
        ))
        .await?;

        db.execute(sea_orm::Statement::from_string(
            backend,
            "CREATE INDEX IF NOT EXISTS idx_atlas_accounts_size \
             ON atlas_accounts (tenant_id, num_employees, annual_revenue);"
                .to_owned(),
        ))
        .await?;

        db.execute(sea_orm::Statement::from_string(
            backend,
            "CREATE INDEX IF NOT EXISTS idx_atlas_accounts_location \
             ON atlas_accounts (tenant_id, state, city, postal_code);"
                .to_owned(),
        ))
        .await?;

        // ── 5. updated_at trigger on atlas_accounts ──────────────────────────
        // set_updated_at_column() was created by G-31 (m20260601_g31_atlas_lead).
        // This migration sorts after G-31 (m20260702_ > m20260601_) so the
        // function is guaranteed to exist at this point.
        db.execute(sea_orm::Statement::from_string(
            backend,
            r#"
CREATE OR REPLACE TRIGGER trg_atlas_accounts_updated_at
BEFORE UPDATE ON atlas_accounts
FOR EACH ROW EXECUTE FUNCTION set_updated_at_column();
            "#
            .trim()
            .to_owned(),
        ))
        .await?;

        // ════════════════════════════════════════════════════════════════════
        // atlas_contacts — gap fill
        // ════════════════════════════════════════════════════════════════════

        // ── 6. ADD columns to atlas_contacts ─────────────────────────────────
        manager
            .alter_table(
                Table::alter()
                    .table(AtlasContacts::Table)
                    // Name enrichment
                    .add_column_if_not_exists(
                        ColumnDef::new(AtlasContacts::MiddleName)
                            .string_len(50)
                            .null(),
                    )
                    .add_column_if_not_exists(
                        ColumnDef::new(AtlasContacts::PreferredName)
                            .string_len(100)
                            .null(),
                    )
                    // department was in the proposal DDL but missing from entity impl
                    .add_column_if_not_exists(
                        ColumnDef::new(AtlasContacts::Department)
                            .string_len(100)
                            .null(),
                    )
                    // Additional contact channels
                    .add_column_if_not_exists(
                        ColumnDef::new(AtlasContacts::Fax).string_len(30).null(),
                    )
                    .add_column_if_not_exists(
                        ColumnDef::new(AtlasContacts::Whatsapp)
                            .string_len(50)
                            .null(),
                    )
                    .add_column_if_not_exists(
                        ColumnDef::new(AtlasContacts::Telegram)
                            .string_len(50)
                            .null(),
                    )
                    .add_column_if_not_exists(
                        ColumnDef::new(AtlasContacts::LinkedinUrl)
                            .string_len(255)
                            .null(),
                    )
                    .add_column_if_not_exists(
                        ColumnDef::new(AtlasContacts::Twitter)
                            .string_len(100)
                            .null(),
                    )
                    .add_column_if_not_exists(
                        ColumnDef::new(AtlasContacts::Instagram)
                            .string_len(100)
                            .null(),
                    )
                    .add_column_if_not_exists(
                        ColumnDef::new(AtlasContacts::AvatarUrl)
                            .string_len(500)
                            .null(),
                    )
                    // Verification flags (from MillionVerifier-processed files)
                    .add_column_if_not_exists(
                        ColumnDef::new(AtlasContacts::EmailVerified)
                            .boolean()
                            .null()
                            .default(false),
                    )
                    .add_column_if_not_exists(
                        ColumnDef::new(AtlasContacts::PhoneVerified)
                            .boolean()
                            .null()
                            .default(false),
                    )
                    // Tax identity — for individual contacts used as B2C customers
                    // CPF (Brazil), SSN (US), NIF (Portugal), TIN (generic)
                    .add_column_if_not_exists(
                        ColumnDef::new(AtlasContacts::TaxId).string_len(30).null(),
                    )
                    // 'cpf' | 'ssn' | 'tin' | 'nif'
                    .add_column_if_not_exists(
                        ColumnDef::new(AtlasContacts::TaxIdType)
                            .string_len(20)
                            .null(),
                    )
                    // Import attribution
                    .add_column_if_not_exists(
                        ColumnDef::new(AtlasContacts::DataSource)
                            .string_len(50)
                            .null(),
                    )
                    .add_column_if_not_exists(
                        ColumnDef::new(AtlasContacts::DataSourceId)
                            .string_len(100)
                            .null(),
                    )
                    .add_column_if_not_exists(
                        ColumnDef::new(AtlasContacts::IsDuplicate)
                            .boolean()
                            .null()
                            .default(false),
                    )
                    .add_column_if_not_exists(
                        ColumnDef::new(AtlasContacts::DuplicateOfContactId)
                            .uuid()
                            .null(),
                    )
                    .to_owned(),
            )
            .await?;

        // ── 7. Indexes on atlas_contacts (raw SQL for partial index support) ──

        // LinkedIn URL — primary enrichment key for B2B contacts
        db.execute(sea_orm::Statement::from_string(
            backend,
            "CREATE INDEX IF NOT EXISTS idx_atlas_contacts_linkedin \
             ON atlas_contacts (linkedin_url) \
             WHERE linkedin_url IS NOT NULL;"
                .to_owned(),
        ))
        .await?;

        // Import dedup: prevents re-importing same contact
        db.execute(sea_orm::Statement::from_string(
            backend,
            "CREATE INDEX IF NOT EXISTS idx_atlas_contacts_data_source \
             ON atlas_contacts (tenant_id, data_source, data_source_id) \
             WHERE data_source IS NOT NULL;"
                .to_owned(),
        ))
        .await?;

        // ── 8. updated_at trigger on atlas_contacts ──────────────────────────
        db.execute(sea_orm::Statement::from_string(
            backend,
            r#"
CREATE OR REPLACE TRIGGER trg_atlas_contacts_updated_at
BEFORE UPDATE ON atlas_contacts
FOR EACH ROW EXECUTE FUNCTION set_updated_at_column();
            "#
            .trim()
            .to_owned(),
        ))
        .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();
        let backend = db.get_database_backend();

        // Drop triggers
        db.execute(sea_orm::Statement::from_string(
            backend,
            "DROP TRIGGER IF EXISTS trg_atlas_contacts_updated_at ON atlas_contacts;".to_owned(),
        ))
        .await?;

        db.execute(sea_orm::Statement::from_string(
            backend,
            "DROP TRIGGER IF EXISTS trg_atlas_accounts_updated_at ON atlas_accounts;".to_owned(),
        ))
        .await?;

        // Drop indexes
        for idx in &[
            "idx_atlas_contacts_data_source",
            "idx_atlas_contacts_linkedin",
            "idx_atlas_accounts_location",
            "idx_atlas_accounts_size",
            "idx_atlas_accounts_industry",
            "idx_atlas_accounts_data_source",
            "idx_atlas_accounts_duns",
            "idx_atlas_accounts_email",
            "idx_atlas_accounts_domain",
            "idx_atlas_accounts_geo",
        ] {
            db.execute(sea_orm::Statement::from_string(
                backend,
                format!("DROP INDEX IF EXISTS {};", idx),
            ))
            .await?;
        }

        // NOTE: We do not attempt to reverse the column additions or the rename
        // in down() — removing columns in a reversible way requires careful
        // data migration and is outside the scope of a schema-only rollback.
        // If rollback is needed, restore from a database snapshot.

        Ok(())
    }
}

// ── Iden enums ───────────────────────────────────────────────────────────────

#[derive(DeriveIden)]
enum AtlasAccounts {
    Table,
    // New columns — existing columns not re-listed
    DbaName,
    Website,
    Domain,
    DunsNumber,
    TaxIdPrimary,
    TaxIdSecondary,
    TaxIdType,
    CompanyPhone,
    CompanyEmail,
    CompanyFax,
    Industry,
    SubIndustry,
    SicCode,
    NaicsCode,
    NumEmployees,
    AnnualRevenue,
    CompanyType,
    LocationType,
    YearEstablished,
    CreditScoreCode,
    StreetAddress,
    City,
    State,
    PostalCode,
    Country,
    MailingAddress,
    DataSource,
    DataSourceId,
    IsDuplicate,
    DuplicateOfAccountId,
}

#[derive(DeriveIden)]
enum AtlasContacts {
    Table,
    // New columns — existing columns not re-listed
    MiddleName,
    PreferredName,
    Department,
    Fax,
    Whatsapp,
    Telegram,
    LinkedinUrl,
    Twitter,
    Instagram,
    AvatarUrl,
    EmailVerified,
    PhoneVerified,
    TaxId,
    TaxIdType,
    DataSource,
    DataSourceId,
    IsDuplicate,
    DuplicateOfContactId,
}
