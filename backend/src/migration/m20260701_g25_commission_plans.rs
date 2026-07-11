use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

/// GENERIC-25: atlas_commission_plans — Commission Plan & Split Governance
///
/// A reusable plan that defines how revenue is split among multiple recipients
/// (platform, broker, agent, carrier, MGA, co-broker) — including tiered rates,
/// overrides, clawback rules, and caps — governing the behavior of
/// `atlas_ledger_splits` (G-03).
///
/// This closes the explicit "Commission" gap in the commerce chain:
/// G-03 `atlas_ledger_splits` records the split, but previously no plan governed
/// what split to compute or who had authority to change it.
///
/// Salesforce analog: Revenue Cloud CommissionSchedule + SalesAgreement.
/// Industry analogs: Xactly Incent, SAP Commissions, Salesforce FSC commission tracking.
///
/// Apps benefiting immediately: CoverFlow (carrier/MGA/broker premium splits),
/// AgentLink (lead referral commissions), Direct Booking Engine (travel agent
/// commissions), Clipping Marketplace (CPM payout rates), PM (property mgmt fees).
///
/// Depends on: G-03 (atlas_ledger_splits receives commission_plan_id FK)
#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();

        // ── Commission basis ENUM ─────────────────────────────────────────────
        db.execute(sea_orm::Statement::from_string(
            db.get_database_backend(),
            r#"
            DO $$ BEGIN
                CREATE TYPE atlas_commission_basis AS ENUM (
                    'percentage',
                    'flat_per_unit',
                    'cpm',
                    'tiered'
                );
            EXCEPTION WHEN duplicate_object THEN NULL; END $$;
            "#
            .to_owned(),
        ))
        .await?;

        // ── atlas_commission_plans ────────────────────────────────────────────
        manager
            .create_table(
                Table::create()
                    .table(AtlasCommissionPlans::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(AtlasCommissionPlans::Id)
                            .uuid()
                            .not_null()
                            .primary_key()
                            .default(Expr::cust("gen_random_uuid()")),
                    )
                    .col(
                        ColumnDef::new(AtlasCommissionPlans::TenantId)
                            .uuid()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(AtlasCommissionPlans::Name)
                            .string_len(255)
                            .not_null(),
                    )
                    // plan_type discriminator examples:
                    //   'broker_split', 'agent_override', 'carrier_remittance',
                    //   'platform_fee', 'cpm_payout', 'referral', 'property_mgmt_fee'
                    .col(
                        ColumnDef::new(AtlasCommissionPlans::PlanType)
                            .string_len(50)
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(AtlasCommissionPlans::IsActive)
                            .boolean()
                            .not_null()
                            .default(true),
                    )
                    // Default split structure
                    .col(
                        ColumnDef::new(AtlasCommissionPlans::CommissionBasis)
                            .string_len(20)
                            .not_null(),
                    )
                    // percentage (e.g. 15.00 = 15%) or flat amount in cents
                    .col(
                        ColumnDef::new(AtlasCommissionPlans::DefaultRate)
                            .decimal_len(8, 4)
                            .null(),
                    )
                    .col(
                        ColumnDef::new(AtlasCommissionPlans::Currency)
                            .char_len(3)
                            .not_null()
                            .default(Expr::val("USD")),
                    )
                    // Tiered rates — used when commission_basis = 'tiered'
                    // Structure: [{min_volume_cents, max_volume_cents, rate}, ...]
                    .col(
                        ColumnDef::new(AtlasCommissionPlans::Tiers)
                            .json_binary()
                            .null(),
                    )
                    // Caps and minimums (per transaction)
                    .col(
                        ColumnDef::new(AtlasCommissionPlans::CapCents)
                            .big_integer()
                            .null(),
                    )
                    .col(
                        ColumnDef::new(AtlasCommissionPlans::MinimumCents)
                            .big_integer()
                            .null(),
                    )
                    // Clawback: if cancelled within N days, commission reversed via G-03 ledger
                    .col(
                        ColumnDef::new(AtlasCommissionPlans::ClawbackDays)
                            .integer()
                            .null(),
                    )
                    // What entity this plan is associated with (polymorphic)
                    // e.g. 'atlas_service_providers', 'atlas_accounts'
                    .col(
                        ColumnDef::new(AtlasCommissionPlans::AppliesToEntityType)
                            .string_len(50)
                            .null(),
                    )
                    .col(
                        ColumnDef::new(AtlasCommissionPlans::AppliesToEntityId)
                            .uuid()
                            .null(),
                    )
                    // What transaction type this plan governs (e.g. 'hotel_booking', 'insurance_policy')
                    .col(
                        ColumnDef::new(AtlasCommissionPlans::AppliesToLedgerType)
                            .string_len(50)
                            .null(),
                    )
                    // Primary recipient
                    // 'platform', 'broker', 'agent', 'carrier', 'creator', 'property_manager'
                    .col(
                        ColumnDef::new(AtlasCommissionPlans::RecipientType)
                            .string_len(50)
                            .null(),
                    )
                    .col(
                        ColumnDef::new(AtlasCommissionPlans::RecipientAccountId)
                            .uuid()
                            .null(),
                    ) // FK atlas_accounts
                    // Effective date range
                    .col(
                        ColumnDef::new(AtlasCommissionPlans::EffectiveFrom)
                            .date()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(AtlasCommissionPlans::EffectiveTo)
                            .date()
                            .null(),
                    )
                    .col(
                        ColumnDef::new(AtlasCommissionPlans::CreatedByUserId)
                            .uuid()
                            .null(),
                    )
                    .col(
                        ColumnDef::new(AtlasCommissionPlans::CreatedAt)
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
                    .name("idx_atlas_commission_plans_tenant")
                    .table(AtlasCommissionPlans::Table)
                    .col(AtlasCommissionPlans::TenantId)
                    .col(AtlasCommissionPlans::PlanType)
                    .col(AtlasCommissionPlans::IsActive)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_atlas_commission_plans_entity")
                    .table(AtlasCommissionPlans::Table)
                    .col(AtlasCommissionPlans::TenantId)
                    .col(AtlasCommissionPlans::AppliesToEntityType)
                    .col(AtlasCommissionPlans::AppliesToEntityId)
                    .to_owned(),
            )
            .await?;

        // ── atlas_commission_plan_splits ──────────────────────────────────────
        // For complex multi-party splits: carrier 70% + MGA 15% + broker 10% + platform 5%
        manager
            .create_table(
                Table::create()
                    .table(AtlasCommissionPlanSplits::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(AtlasCommissionPlanSplits::Id)
                            .uuid()
                            .not_null()
                            .primary_key()
                            .default(Expr::cust("gen_random_uuid()")),
                    )
                    .col(
                        ColumnDef::new(AtlasCommissionPlanSplits::PlanId)
                            .uuid()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(AtlasCommissionPlanSplits::TenantId)
                            .uuid()
                            .not_null(),
                    )
                    // 'platform', 'broker', 'carrier', 'mga', 'agent', 'creator'
                    .col(
                        ColumnDef::new(AtlasCommissionPlanSplits::RecipientType)
                            .string_len(50)
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(AtlasCommissionPlanSplits::RecipientAccountId)
                            .uuid()
                            .null(),
                    )
                    .col(
                        ColumnDef::new(AtlasCommissionPlanSplits::RecipientLabel)
                            .string_len(100)
                            .null(),
                    )
                    .col(
                        ColumnDef::new(AtlasCommissionPlanSplits::SplitBasis)
                            .string_len(20)
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(AtlasCommissionPlanSplits::SplitRate)
                            .decimal_len(8, 4)
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(AtlasCommissionPlanSplits::CapCents)
                            .big_integer()
                            .null(),
                    )
                    // Order of calculation for cascading splits
                    .col(
                        ColumnDef::new(AtlasCommissionPlanSplits::Priority)
                            .integer()
                            .not_null()
                            .default(0),
                    )
                    // TRUE = gets whatever amount remains after other splits are calculated
                    .col(
                        ColumnDef::new(AtlasCommissionPlanSplits::IsRemainder)
                            .boolean()
                            .not_null()
                            .default(false),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_atlas_commission_plan_splits_plan")
                    .table(AtlasCommissionPlanSplits::Table)
                    .col(AtlasCommissionPlanSplits::PlanId)
                    .col(AtlasCommissionPlanSplits::Priority)
                    .to_owned(),
            )
            .await?;

        // ── Backfill: add commission_plan_id to atlas_ledger_splits ───────────
        // This closes the G-03 completeness gap identified in the gap analysis.
        // The column is nullable so existing rows without a plan are unaffected.
        db.execute(sea_orm::Statement::from_string(
            db.get_database_backend(),
            r#"
            ALTER TABLE atlas_ledger_splits
                ADD COLUMN IF NOT EXISTS commission_plan_id UUID REFERENCES atlas_commission_plans(id);
            "#
            .to_owned(),
        ))
        .await?;

        db.execute(sea_orm::Statement::from_string(
            db.get_database_backend(),
            r#"
            CREATE INDEX IF NOT EXISTS idx_atlas_ledger_splits_commission_plan
                ON atlas_ledger_splits(commission_plan_id)
                WHERE commission_plan_id IS NOT NULL;
            "#
            .to_owned(),
        ))
        .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();

        db.execute(sea_orm::Statement::from_string(
            db.get_database_backend(),
            "ALTER TABLE atlas_ledger_splits DROP COLUMN IF EXISTS commission_plan_id;".to_owned(),
        ))
        .await?;

        manager
            .drop_table(
                Table::drop()
                    .table(AtlasCommissionPlanSplits::Table)
                    .to_owned(),
            )
            .await?;
        manager
            .drop_table(Table::drop().table(AtlasCommissionPlans::Table).to_owned())
            .await?;

        db.execute(sea_orm::Statement::from_string(
            db.get_database_backend(),
            "DROP TYPE IF EXISTS atlas_commission_basis;".to_owned(),
        ))
        .await?;

        Ok(())
    }
}

// ══════════════════════════════════════════════════════════════════════════════
// Iden enums
// ══════════════════════════════════════════════════════════════════════════════

#[derive(DeriveIden)]
enum AtlasCommissionPlans {
    Table,
    Id,
    TenantId,
    Name,
    PlanType,
    IsActive,
    CommissionBasis,
    DefaultRate,
    Currency,
    Tiers,
    CapCents,
    MinimumCents,
    ClawbackDays,
    AppliesToEntityType,
    AppliesToEntityId,
    AppliesToLedgerType,
    RecipientType,
    RecipientAccountId,
    EffectiveFrom,
    EffectiveTo,
    CreatedByUserId,
    CreatedAt,
}

#[derive(DeriveIden)]
enum AtlasCommissionPlanSplits {
    Table,
    Id,
    PlanId,
    TenantId,
    RecipientType,
    RecipientAccountId,
    RecipientLabel,
    SplitBasis,
    SplitRate,
    CapCents,
    Priority,
    IsRemainder,
}
