use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

/// GENERIC-26: atlas_catalog — Product Catalog, Pricebook & Availability Grid
///
/// A structured catalog of what can be sold (room types, package tiers, service
/// slots, coverage options, subscription tiers, add-ons) with associated prices,
/// availability windows, and rate rules.
///
/// This fills the "Browse" step in the commerce chain — "What is for sale and
/// at what price?" — the gap between Assets (G-10, what you own) and Quotes
/// (G-24, the priced proposal sent to a prospect).
///
/// Salesforce analog: Product2 + Pricebook2 + PricebookEntry.
/// Industry analogs: Shopify Product/Variant/Price, Stripe Price, Cloudbeds Room Types.
///
/// Apps benefiting immediately:
///   - Direct Booking Engine: room_types + room_rates (Phase 1 migration names these explicitly)
///   - Event-Aware Revenue Manager: tenant_room_inventories + rate_recommendations
///   - PM (STR mode): availability windows + nightly rates
///   - Famtasm: creator subscription tiers (Free, Standard, Premium)
///   - CoverFlow: coverage options + limits
///   - Flight+Hotel Package Builder: package tiers + add-ons
///
/// Depends on: G-10 (atlas_assets — the underlying physical asset)
/// Referenced by: G-24 (atlas_quotes — line items reference catalog entries)
///                G-23 (atlas_reservations — reserved_asset_type = 'atlas_catalog_entry')
#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let _db = manager.get_connection();

        // ── atlas_catalog_entries ─────────────────────────────────────────────
        manager
            .create_table(
                Table::create()
                    .table(AtlasCatalogEntries::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(AtlasCatalogEntries::Id)
                            .uuid()
                            .not_null()
                            .primary_key()
                            .default(Expr::cust("gen_random_uuid()")),
                    )
                    .col(
                        ColumnDef::new(AtlasCatalogEntries::TenantId)
                            .uuid()
                            .not_null(),
                    )
                    // entry_type discriminator examples:
                    //   'room_type'         — hotel room category (standard, deluxe, suite)
                    //   'str_unit_type'     — STR property variant
                    //   'package_tier'      — bundled travel package (economy, premium, luxury)
                    //   'service_slot'      — time-based service (cleaning, inspection, tour)
                    //   'coverage_option'   — insurance coverage limit/deductible combo
                    //   'subscription_tier' — creator or SaaS subscription (Free, Pro, Elite)
                    //   'add_on'            — optional extra (breakfast, transfer, travel insurance)
                    //   'event_ticket'      — event ticket type (see also G-21)
                    .col(
                        ColumnDef::new(AtlasCatalogEntries::EntryType)
                            .string_len(50)
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(AtlasCatalogEntries::Name)
                            .string_len(255)
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(AtlasCatalogEntries::Slug)
                            .string_len(255)
                            .null(),
                    )
                    .col(
                        ColumnDef::new(AtlasCatalogEntries::Description)
                            .text()
                            .null(),
                    )
                    // Optional link to underlying physical asset (G-10)
                    // e.g. a 'room_type' catalog entry → atlas_assets row for the physical room
                    .col(ColumnDef::new(AtlasCatalogEntries::AssetId).uuid().null()) // FK atlas_assets
                    // Base pricing (can be overridden per-date by rate rules or availability grid)
                    .col(
                        ColumnDef::new(AtlasCatalogEntries::BasePriceCents)
                            .big_integer()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(AtlasCatalogEntries::Currency)
                            .char_len(3)
                            .not_null()
                            .default(Expr::val("USD")),
                    )
                    // NULL = one-time; 'nightly', 'hourly', 'monthly', 'annually'
                    .col(
                        ColumnDef::new(AtlasCatalogEntries::BillingInterval)
                            .string_len(20)
                            .null(),
                    )
                    // Availability
                    .col(
                        ColumnDef::new(AtlasCatalogEntries::IsAvailable)
                            .boolean()
                            .not_null()
                            .default(true),
                    )
                    .col(
                        ColumnDef::new(AtlasCatalogEntries::MinQuantity)
                            .integer()
                            .not_null()
                            .default(1),
                    )
                    .col(
                        ColumnDef::new(AtlasCatalogEntries::MaxQuantity)
                            .integer()
                            .null(),
                    )
                    // Total inventory count (NULL = unlimited / not capacity-tracked)
                    .col(
                        ColumnDef::new(AtlasCatalogEntries::TotalInventory)
                            .integer()
                            .null(),
                    )
                    // App-specific attributes stored as JSONB
                    // room_type: {bed_type, max_occupancy, view_type, amenities[]}
                    // subscription_tier: {feature_flags[], max_uploads, ai_credits}
                    // coverage_option: {limit_cents, deductible_cents, coverages[]}
                    .col(
                        ColumnDef::new(AtlasCatalogEntries::CatalogMetadata)
                            .json_binary()
                            .not_null()
                            .default(Expr::cust("'{}'")),
                    )
                    // Display order within entry_type group
                    .col(
                        ColumnDef::new(AtlasCatalogEntries::SortOrder)
                            .integer()
                            .not_null()
                            .default(0),
                    )
                    .col(
                        ColumnDef::new(AtlasCatalogEntries::CoverImageAttachmentId)
                            .uuid()
                            .null(),
                    )
                    .col(
                        ColumnDef::new(AtlasCatalogEntries::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .col(
                        ColumnDef::new(AtlasCatalogEntries::UpdatedAt)
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
                    .name("idx_atlas_catalog_entries_tenant_type")
                    .table(AtlasCatalogEntries::Table)
                    .col(AtlasCatalogEntries::TenantId)
                    .col(AtlasCatalogEntries::EntryType)
                    .col(AtlasCatalogEntries::IsAvailable)
                    .col(AtlasCatalogEntries::SortOrder)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_atlas_catalog_entries_asset")
                    .table(AtlasCatalogEntries::Table)
                    .col(AtlasCatalogEntries::TenantId)
                    .col(AtlasCatalogEntries::AssetId)
                    .to_owned(),
            )
            .await?;

        // ── atlas_catalog_rate_rules ──────────────────────────────────────────
        // Price overrides and modifiers for specific date ranges, channels, or
        // booking patterns. The Revenue Manager's dynamic pricing writes here.
        manager
            .create_table(
                Table::create()
                    .table(AtlasCatalogRateRules::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(AtlasCatalogRateRules::Id)
                            .uuid()
                            .not_null()
                            .primary_key()
                            .default(Expr::cust("gen_random_uuid()")),
                    )
                    .col(
                        ColumnDef::new(AtlasCatalogRateRules::CatalogEntryId)
                            .uuid()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(AtlasCatalogRateRules::TenantId)
                            .uuid()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(AtlasCatalogRateRules::RuleName)
                            .string_len(100)
                            .null(),
                    )
                    // When this rule applies (date range)
                    .col(
                        ColumnDef::new(AtlasCatalogRateRules::AppliesFrom)
                            .date()
                            .null(),
                    )
                    .col(
                        ColumnDef::new(AtlasCatalogRateRules::AppliesTo)
                            .date()
                            .null(),
                    )
                    // Day of week bitmask: 1=Mon 2=Tue 4=Wed 8=Thu 16=Fri 32=Sat 64=Sun
                    // e.g. 96 = Sat+Sun (weekend premium rule)
                    .col(
                        ColumnDef::new(AtlasCatalogRateRules::DayOfWeekMask)
                            .integer()
                            .null(),
                    )
                    // Booking constraints
                    .col(
                        ColumnDef::new(AtlasCatalogRateRules::MinStayNights)
                            .integer()
                            .null(),
                    )
                    .col(
                        ColumnDef::new(AtlasCatalogRateRules::MinAdvanceDays)
                            .integer()
                            .null(),
                    )
                    // Channel scope (NULL = applies to all channels)
                    // 'direct', 'ota', 'gds', 'corporate', 'member'
                    .col(
                        ColumnDef::new(AtlasCatalogRateRules::Channel)
                            .string_len(50)
                            .null(),
                    )
                    // Pricing — exactly one of these should be set
                    // absolute override: this IS the price for matching slots
                    .col(
                        ColumnDef::new(AtlasCatalogRateRules::PriceOverrideCents)
                            .big_integer()
                            .null(),
                    )
                    // percentage modifier: positive = premium, negative = discount
                    // e.g. 20.00 = +20% weekend premium; -10.00 = -10% early bird
                    .col(
                        ColumnDef::new(AtlasCatalogRateRules::PriceModifierPct)
                            .decimal_len(6, 2)
                            .null(),
                    )
                    // Priority (higher = evaluated first; first matching rule wins)
                    .col(
                        ColumnDef::new(AtlasCatalogRateRules::Priority)
                            .integer()
                            .not_null()
                            .default(0),
                    )
                    .col(
                        ColumnDef::new(AtlasCatalogRateRules::IsActive)
                            .boolean()
                            .not_null()
                            .default(true),
                    )
                    .col(
                        ColumnDef::new(AtlasCatalogRateRules::CreatedByUserId)
                            .uuid()
                            .null(),
                    )
                    .col(
                        ColumnDef::new(AtlasCatalogRateRules::CreatedAt)
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
                    .name("idx_atlas_catalog_rate_rules_entry_dates")
                    .table(AtlasCatalogRateRules::Table)
                    .col(AtlasCatalogRateRules::CatalogEntryId)
                    .col(AtlasCatalogRateRules::IsActive)
                    .col(AtlasCatalogRateRules::AppliesFrom)
                    .col(AtlasCatalogRateRules::AppliesTo)
                    .to_owned(),
            )
            .await?;

        // ── atlas_catalog_availability ────────────────────────────────────────
        // Per-date slot inventory with hard capacity counts.
        // Used by: Direct Booking (room availability calendar), Revenue Manager
        // (availability + rate push target), PM STR (nightly availability).
        // NOT used for unlimited/unconstrained items (e.g. digital subscription tiers).
        manager
            .create_table(
                Table::create()
                    .table(AtlasCatalogAvailability::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(AtlasCatalogAvailability::Id)
                            .uuid()
                            .not_null()
                            .primary_key()
                            .default(Expr::cust("gen_random_uuid()")),
                    )
                    .col(
                        ColumnDef::new(AtlasCatalogAvailability::CatalogEntryId)
                            .uuid()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(AtlasCatalogAvailability::TenantId)
                            .uuid()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(AtlasCatalogAvailability::SlotDate)
                            .date()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(AtlasCatalogAvailability::TotalInventory)
                            .integer()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(AtlasCatalogAvailability::ReservedCount)
                            .integer()
                            .not_null()
                            .default(0),
                    )
                    // is_blocked: manual operator block (maintenance, hold-back, etc.)
                    .col(
                        ColumnDef::new(AtlasCatalogAvailability::IsBlocked)
                            .boolean()
                            .not_null()
                            .default(false),
                    )
                    .col(
                        ColumnDef::new(AtlasCatalogAvailability::BlockReason)
                            .string_len(255)
                            .null(),
                    )
                    // Day-specific price override (takes precedence over base_price_cents and rate rules)
                    .col(
                        ColumnDef::new(AtlasCatalogAvailability::OverridePriceCents)
                            .big_integer()
                            .null(),
                    )
                    // Synced from PMS or Revenue Manager push
                    .col(
                        ColumnDef::new(AtlasCatalogAvailability::LastSyncedAt)
                            .timestamp_with_time_zone()
                            .null(),
                    )
                    .to_owned(),
            )
            .await?;

        // Unique per entry+date — the availability grid is 1 row per (entry, date)
        manager
            .create_index(
                Index::create()
                    .name("idx_atlas_catalog_availability_entry_date")
                    .table(AtlasCatalogAvailability::Table)
                    .col(AtlasCatalogAvailability::CatalogEntryId)
                    .col(AtlasCatalogAvailability::SlotDate)
                    .unique()
                    .to_owned(),
            )
            .await?;

        // Partial index: available inventory lookup (the hot query path for booking search)
        // Query: WHERE catalog_entry_id = $1 AND slot_date BETWEEN $start AND $end
        //        AND NOT is_blocked AND (total_inventory - reserved_count) > 0
        manager
            .create_index(
                Index::create()
                    .name("idx_atlas_catalog_availability_open_slots")
                    .table(AtlasCatalogAvailability::Table)
                    .col(AtlasCatalogAvailability::TenantId)
                    .col(AtlasCatalogAvailability::CatalogEntryId)
                    .col(AtlasCatalogAvailability::SlotDate)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(
                Table::drop()
                    .table(AtlasCatalogAvailability::Table)
                    .to_owned(),
            )
            .await?;
        manager
            .drop_table(Table::drop().table(AtlasCatalogRateRules::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(AtlasCatalogEntries::Table).to_owned())
            .await?;

        Ok(())
    }
}

// ══════════════════════════════════════════════════════════════════════════════
// Iden enums
// ══════════════════════════════════════════════════════════════════════════════

#[derive(DeriveIden)]
enum AtlasCatalogEntries {
    Table,
    Id,
    TenantId,
    EntryType,
    Name,
    Slug,
    Description,
    AssetId,
    BasePriceCents,
    Currency,
    BillingInterval,
    IsAvailable,
    MinQuantity,
    MaxQuantity,
    TotalInventory,
    CatalogMetadata,
    SortOrder,
    CoverImageAttachmentId,
    CreatedAt,
    UpdatedAt,
}

#[derive(DeriveIden)]
enum AtlasCatalogRateRules {
    Table,
    Id,
    CatalogEntryId,
    TenantId,
    RuleName,
    AppliesFrom,
    AppliesTo,
    DayOfWeekMask,
    MinStayNights,
    MinAdvanceDays,
    Channel,
    PriceOverrideCents,
    PriceModifierPct,
    Priority,
    IsActive,
    CreatedByUserId,
    CreatedAt,
}

#[derive(DeriveIden)]
enum AtlasCatalogAvailability {
    Table,
    Id,
    CatalogEntryId,
    TenantId,
    SlotDate,
    TotalInventory,
    ReservedCount,
    IsBlocked,
    BlockReason,
    OverridePriceCents,
    LastSyncedAt,
}
