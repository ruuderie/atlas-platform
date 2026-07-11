use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

/// GENERIC-31: atlas_lead — Canonical Lead / Prospect Entity
///
/// Replaces the legacy `lead` table with a production-ready schema that:
///   - Supports bulk import from BusinessLeadsUSA, ZoomInfo, FMCSA, MWBE, Haiti/LinkedIn, SBS
///   - Carries first-class firmographic columns for high-performance segmentation queries
///   - Tracks dedup state across multi-source ingestion pipelines
///   - Records the full conversion flow → atlas_accounts + atlas_contacts + atlas_opportunities
///
/// Design notes:
///   - All indexes with WHERE clauses are emitted as raw SQL; SeaORM's Index builder
///     has no partial index support.
///   - The `idx_atlas_lead_status` index uses DESC on created_at (pipeline feed pattern)
///     which also requires raw SQL.
///   - `geo_point geometry(Point, 4326)` is PostGIS-guarded: added only when the
///     extension is present, matching the G-01 pattern.
///   - A `set_updated_at_column()` Postgres trigger is installed on atlas_lead.
///     This is a platform-wide missing primitive — added here first.
///   - `name` is NOT NULL; the service layer is responsible for computing it as
///     COALESCE(first_name || ' ' || last_name, company, email, 'Unknown').
///   - `converted_account_id` is added (missing from spec DDL but required by the
///     convert_lead flow described in section 5).
///   - Backward-compat view is named `atlas_lead_compat_view` (not `lead`) because
///     the legacy `lead` table still exists during the transition window.
///
/// Spec: docs/architecture/g31_atlas_lead_spec.md
#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();
        let backend = db.get_database_backend();

        // ── 0. Shared updated_at trigger function ────────────────────────────
        //
        // This is a platform-wide primitive that should exist. We create it here
        // idempotently with OR REPLACE. Any table can now attach this trigger.
        db.execute(sea_orm::Statement::from_string(
            backend,
            r#"
CREATE OR REPLACE FUNCTION set_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;
            "#
            .trim()
            .to_owned(),
        ))
        .await?;

        // ── 1. Create atlas_lead table ───────────────────────────────────────
        manager
            .create_table(
                Table::create()
                    .table(AtlasLead::Table)
                    .if_not_exists()
                    // ── Identity ──────────────────────────────────────────────
                    .col(
                        ColumnDef::new(AtlasLead::Id)
                            .uuid()
                            .not_null()
                            .primary_key()
                            .default(Expr::cust("gen_random_uuid()")),
                    )
                    .col(ColumnDef::new(AtlasLead::TenantId).uuid().not_null())
                    // ── Contact — Individual ───────────────────────────────────
                    .col(ColumnDef::new(AtlasLead::FirstName).string_len(100).null())
                    .col(ColumnDef::new(AtlasLead::MiddleName).string_len(50).null())
                    .col(ColumnDef::new(AtlasLead::LastName).string_len(100).null())
                    // NOT NULL: service MUST compute as COALESCE(first+last, company, email, 'Unknown')
                    .col(ColumnDef::new(AtlasLead::Name).string_len(255).not_null())
                    .col(ColumnDef::new(AtlasLead::Title).string_len(150).null())
                    .col(ColumnDef::new(AtlasLead::Email).string_len(255).null())
                    .col(
                        ColumnDef::new(AtlasLead::EmailVerified)
                            .boolean()
                            .not_null()
                            .default(false),
                    )
                    .col(ColumnDef::new(AtlasLead::Phone).string_len(30).null())
                    .col(
                        ColumnDef::new(AtlasLead::PhoneVerified)
                            .boolean()
                            .not_null()
                            .default(false),
                    )
                    .col(ColumnDef::new(AtlasLead::Fax).string_len(30).null())
                    // Social channels
                    .col(ColumnDef::new(AtlasLead::Whatsapp).string_len(50).null())
                    .col(ColumnDef::new(AtlasLead::Telegram).string_len(50).null())
                    .col(
                        ColumnDef::new(AtlasLead::LinkedinUrl)
                            .string_len(255)
                            .null(),
                    )
                    .col(ColumnDef::new(AtlasLead::Twitter).string_len(100).null())
                    .col(ColumnDef::new(AtlasLead::Instagram).string_len(100).null())
                    .col(ColumnDef::new(AtlasLead::Facebook).string_len(100).null())
                    .col(ColumnDef::new(AtlasLead::AvatarUrl).string_len(500).null())
                    // ── Company / Organization ────────────────────────────────
                    .col(ColumnDef::new(AtlasLead::Company).string_len(255).null())
                    .col(ColumnDef::new(AtlasLead::CompanyDba).string_len(255).null())
                    .col(
                        ColumnDef::new(AtlasLead::CompanyWebsite)
                            .string_len(255)
                            .null(),
                    )
                    // extracted domain — primary dedup anchor for company-level dedup
                    .col(ColumnDef::new(AtlasLead::Domain).string_len(100).null())
                    .col(ColumnDef::new(AtlasLead::Industry).string_len(255).null())
                    .col(
                        ColumnDef::new(AtlasLead::SubIndustry)
                            .string_len(255)
                            .null(),
                    )
                    .col(ColumnDef::new(AtlasLead::NumEmployees).integer().null())
                    .col(
                        ColumnDef::new(AtlasLead::AnnualRevenue)
                            .decimal_len(18, 2)
                            .null(),
                    )
                    // 'public' | 'private' | 'government' | 'nonprofit' | 'individual'
                    .col(ColumnDef::new(AtlasLead::CompanyType).string_len(30).null())
                    // 'headquarters' | 'branch' | 'single' | 'franchise'
                    .col(
                        ColumnDef::new(AtlasLead::LocationType)
                            .string_len(30)
                            .null(),
                    )
                    .col(
                        ColumnDef::new(AtlasLead::YearEstablished)
                            .small_integer()
                            .null(),
                    )
                    // ── Industry Classification Codes ─────────────────────────
                    .col(ColumnDef::new(AtlasLead::SicCode).string_len(10).null())
                    .col(ColumnDef::new(AtlasLead::NaicsCode).string_len(10).null())
                    // D-U-N-S — gold-standard business dedup
                    .col(ColumnDef::new(AtlasLead::DunsNumber).string_len(15).null())
                    // ── Credit / Risk Signals ─────────────────────────────────
                    .col(
                        ColumnDef::new(AtlasLead::CreditScoreCode)
                            .string_len(10)
                            .null(),
                    )
                    // ── Address ───────────────────────────────────────────────
                    .col(
                        ColumnDef::new(AtlasLead::StreetAddress)
                            .string_len(255)
                            .null(),
                    )
                    .col(ColumnDef::new(AtlasLead::City).string_len(100).null())
                    .col(ColumnDef::new(AtlasLead::State).string_len(50).null())
                    .col(ColumnDef::new(AtlasLead::PostalCode).string_len(20).null())
                    .col(
                        ColumnDef::new(AtlasLead::Country)
                            .string_len(50)
                            .not_null()
                            .default(Expr::val("US")),
                    )
                    // Separate mailing address — kept as JSONB (rarely queried)
                    // {"street": "...", "city": "...", "state": "...", "zip": "...", "country": "..."}
                    .col(
                        ColumnDef::new(AtlasLead::MailingAddress)
                            .json_binary()
                            .null(),
                    )
                    // ── Lead Metadata ──────────────────────────────────────────
                    .col(ColumnDef::new(AtlasLead::Message).text().null())
                    // 'new' | 'contacted' | 'qualifying' | 'qualified' | 'disqualified' | 'converted'
                    .col(
                        ColumnDef::new(AtlasLead::LeadStatus)
                            .string_len(50)
                            .not_null()
                            .default(Expr::val("new")),
                    )
                    // 'manual' | 'zoominfo' | 'business_leads_usa' | 'fmcsa' | 'mwbe_registry'
                    // 'linkedin' | 'web_form' | 'referral' | 'import'
                    .col(ColumnDef::new(AtlasLead::Source).string_len(50).null())
                    .col(ColumnDef::new(AtlasLead::DataSource).string_len(50).null())
                    .col(
                        ColumnDef::new(AtlasLead::DataSourceId)
                            .string_len(100)
                            .null(),
                    )
                    // ── Industry / Vertical Specific Data ─────────────────────
                    // FMCSA safety data, MWBE certifications, financial health signals,
                    // social profiles — see spec section 4.2 for full structure.
                    .col(ColumnDef::new(AtlasLead::LeadMetadata).json_binary().null())
                    // ── Deduplication ──────────────────────────────────────────
                    .col(
                        ColumnDef::new(AtlasLead::IsDuplicate)
                            .boolean()
                            .not_null()
                            .default(false),
                    )
                    // Self-reference to canonical record this duplicates.
                    // No FK constraint — consistent with platform pattern (see unify_accounts_contacts.rs).
                    .col(ColumnDef::new(AtlasLead::DuplicateOfLeadId).uuid().null())
                    // ── Linked Records ─────────────────────────────────────────
                    .col(ColumnDef::new(AtlasLead::ListingId).uuid().null())
                    // FK to legacy account table (registered user who generated lead via form)
                    .col(ColumnDef::new(AtlasLead::AccountId).uuid().null())
                    // ── Conversion ─────────────────────────────────────────────
                    .col(
                        ColumnDef::new(AtlasLead::IsConverted)
                            .boolean()
                            .not_null()
                            .default(false),
                    )
                    .col(
                        ColumnDef::new(AtlasLead::ConvertedAt)
                            .timestamp_with_time_zone()
                            .null(),
                    )
                    // G-01 atlas_accounts — account created during conversion
                    .col(ColumnDef::new(AtlasLead::ConvertedAccountId).uuid().null())
                    // G-01 atlas_contacts — contact created during conversion
                    .col(ColumnDef::new(AtlasLead::ConvertedContactId).uuid().null())
                    // G-15 atlas_opportunities — opportunity created during conversion
                    .col(
                        ColumnDef::new(AtlasLead::ConvertedOpportunityId)
                            .uuid()
                            .null(),
                    )
                    .col(
                        ColumnDef::new(AtlasLead::DisqualifiedAt)
                            .timestamp_with_time_zone()
                            .null(),
                    )
                    .col(
                        ColumnDef::new(AtlasLead::DisqualificationReason)
                            .text()
                            .null(),
                    )
                    // ── Timestamps ─────────────────────────────────────────────
                    .col(
                        ColumnDef::new(AtlasLead::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .col(
                        ColumnDef::new(AtlasLead::UpdatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .to_owned(),
            )
            .await?;

        // ── 2. updated_at trigger on atlas_lead ──────────────────────────────
        db.execute(sea_orm::Statement::from_string(
            backend,
            r#"
CREATE OR REPLACE TRIGGER trg_atlas_lead_updated_at
BEFORE UPDATE ON atlas_lead
FOR EACH ROW EXECUTE FUNCTION set_updated_at_column();
            "#
            .trim()
            .to_owned(),
        ))
        .await?;

        // ── 3. Indexes — all via raw SQL ─────────────────────────────────────
        //
        // SeaORM Index::create() has no support for:
        //   - Partial indexes (WHERE clause)
        //   - DESC column ordering
        // All indexes below are therefore raw SQL for correctness.

        // Dedup: email within tenant — sparse index on non-null only
        db.execute(sea_orm::Statement::from_string(
            backend,
            "CREATE INDEX IF NOT EXISTS idx_atlas_lead_email \
             ON atlas_lead (tenant_id, email) \
             WHERE email IS NOT NULL;"
                .to_owned(),
        ))
        .await?;

        // Dedup: domain within tenant — sparse, only canonical records
        db.execute(sea_orm::Statement::from_string(
            backend,
            "CREATE INDEX IF NOT EXISTS idx_atlas_lead_domain \
             ON atlas_lead (tenant_id, domain) \
             WHERE domain IS NOT NULL AND is_duplicate = false;"
                .to_owned(),
        ))
        .await?;

        // Dedup: D-U-N-S number (global — DUNS are universal, not tenant-scoped)
        db.execute(sea_orm::Statement::from_string(
            backend,
            "CREATE INDEX IF NOT EXISTS idx_atlas_lead_duns \
             ON atlas_lead (duns_number) \
             WHERE duns_number IS NOT NULL;"
                .to_owned(),
        ))
        .await?;

        // Import attribution dedup: prevents re-importing the same source record
        db.execute(sea_orm::Statement::from_string(
            backend,
            "CREATE INDEX IF NOT EXISTS idx_atlas_lead_data_source \
             ON atlas_lead (tenant_id, data_source, data_source_id) \
             WHERE data_source IS NOT NULL;"
                .to_owned(),
        ))
        .await?;

        // Firmographic filter: industry / SIC / NAICS segmentation
        db.execute(sea_orm::Statement::from_string(
            backend,
            "CREATE INDEX IF NOT EXISTS idx_atlas_lead_industry \
             ON atlas_lead (tenant_id, sic_code, naics_code);"
                .to_owned(),
        ))
        .await?;

        // Firmographic filter: company size bands (SMB vs mid-market vs enterprise)
        db.execute(sea_orm::Statement::from_string(
            backend,
            "CREATE INDEX IF NOT EXISTS idx_atlas_lead_size \
             ON atlas_lead (tenant_id, num_employees, annual_revenue);"
                .to_owned(),
        ))
        .await?;

        // Pipeline status feed — DESC on created_at for newest-first ordering
        db.execute(sea_orm::Statement::from_string(
            backend,
            "CREATE INDEX IF NOT EXISTS idx_atlas_lead_status \
             ON atlas_lead (tenant_id, lead_status, created_at DESC);"
                .to_owned(),
        ))
        .await?;

        // ── 4. geo_point column (PostGIS-guarded) ────────────────────────────
        //
        // Added only when PostGIS is installed so the migration is safe in CI /
        // local environments without the extension. Same guard pattern as G-01.
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
                "ALTER TABLE atlas_lead \
                 ADD COLUMN IF NOT EXISTS geo_point geometry(Point, 4326) NULL;"
                    .to_owned(),
            ))
            .await?;

            // GIST index — partial on non-null only
            db.execute(sea_orm::Statement::from_string(
                backend,
                "CREATE INDEX IF NOT EXISTS idx_atlas_lead_geo \
                 ON atlas_lead USING GIST (geo_point) \
                 WHERE geo_point IS NOT NULL;"
                    .to_owned(),
            ))
            .await?;
        } else {
            tracing::warn!(
                "PostGIS not available — atlas_lead.geo_point column skipped. \
                 Enable PostGIS and re-run this migration to add geo indexing."
            );
        }

        // ── 5. Backward-compat view (spec section 9) ─────────────────────────
        //
        // Named atlas_lead_compat_view (not `lead`) because the legacy `lead`
        // table still exists during the transition window. A follow-up migration
        // will DROP TABLE lead and RENAME VIEW atlas_lead_compat_view TO lead
        // once all code paths reference atlas_lead directly.
        db.execute(sea_orm::Statement::from_string(
            backend,
            r#"
CREATE OR REPLACE VIEW atlas_lead_compat_view AS
SELECT
    id,
    name,
    listing_id,
    account_id,
    first_name,
    last_name,
    email,
    phone,
    whatsapp,
    telegram,
    twitter,
    instagram,
    facebook,
    mailing_address              AS billing_address,
    NULL::jsonb                  AS shipping_address,
    message,
    source,
    is_converted,
    is_converted                 AS converted_to_contact,
    converted_opportunity_id     AS associated_deal_id,
    NULL::uuid                   AS converted_customer_id,
    converted_contact_id,
    company,
    title,
    lead_status,
    created_at,
    updated_at,
    tenant_id,
    lead_metadata                AS properties,
    avatar_url
FROM atlas_lead;
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

        db.execute(sea_orm::Statement::from_string(
            backend,
            "DROP VIEW IF EXISTS atlas_lead_compat_view;".to_owned(),
        ))
        .await?;

        db.execute(sea_orm::Statement::from_string(
            backend,
            "DROP TRIGGER IF EXISTS trg_atlas_lead_updated_at ON atlas_lead;".to_owned(),
        ))
        .await?;

        manager
            .drop_table(Table::drop().table(AtlasLead::Table).to_owned())
            .await?;

        // NOTE: We do NOT drop set_updated_at_column() — it's a shared platform
        // function that may be used by other tables added after this migration.

        Ok(())
    }
}

// ── Iden enum ────────────────────────────────────────────────────────────────

#[derive(DeriveIden)]
enum AtlasLead {
    Table,
    // Identity
    Id,
    TenantId,
    // Contact — Individual
    FirstName,
    MiddleName,
    LastName,
    Name,
    Title,
    Email,
    EmailVerified,
    Phone,
    PhoneVerified,
    Fax,
    Whatsapp,
    Telegram,
    LinkedinUrl,
    Twitter,
    Instagram,
    Facebook,
    AvatarUrl,
    // Company / Organization
    Company,
    CompanyDba,
    CompanyWebsite,
    Domain,
    Industry,
    SubIndustry,
    NumEmployees,
    AnnualRevenue,
    CompanyType,
    LocationType,
    YearEstablished,
    // Classification codes
    SicCode,
    NaicsCode,
    DunsNumber,
    // Credit / risk
    CreditScoreCode,
    // Address
    StreetAddress,
    City,
    State,
    PostalCode,
    Country,
    MailingAddress,
    // Lead metadata
    Message,
    LeadStatus,
    Source,
    DataSource,
    DataSourceId,
    LeadMetadata,
    // Dedup
    IsDuplicate,
    DuplicateOfLeadId,
    // Linked records
    ListingId,
    AccountId,
    // Conversion — all three legs of the conversion output
    IsConverted,
    ConvertedAt,
    ConvertedAccountId,
    ConvertedContactId,
    ConvertedOpportunityId,
    DisqualifiedAt,
    DisqualificationReason,
    // Timestamps
    CreatedAt,
    UpdatedAt,
}
