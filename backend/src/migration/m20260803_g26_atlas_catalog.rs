use sea_orm_migration::prelude::*;
use sea_orm_migration::sea_query::extension::postgres::Type;

// ── G-26: atlas_catalog ───────────────────────────────────────────────────────
//
// Three-table structure mirroring Salesforce Product2 + Pricebook2 + PricebookEntry:
//
//   atlas_catalog_entries      — "what can be sold" (room types, packages, subscriptions)
//   atlas_catalog_rate_rules   — date-range / channel / min-stay pricing overrides
//   atlas_catalog_availability — per-date slot inventory grid with computed availability
//
// Bridges G10 (atlas_assets) → G24 (atlas_quotes) in the commerce chain:
//   Asset (owned) → Catalog Entry (saleable) → Quote Line (priced proposal) → Reservation (booked)
//
// Zero net-new tables outside the generic namespace.

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20260803_g26_atlas_catalog"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // ── 1. atlas_catalog_entry_type enum ─────────────────────────────────
        manager
            .create_type(
                Type::create()
                    .as_enum(Alias::new("atlas_catalog_entry_type"))
                    .values([
                        Alias::new("room_type"),        // hotel/STR room category
                        Alias::new("service_slot"),     // timed service (cleaning, HVAC visit)
                        Alias::new("package_tier"),     // travel bundle tier (Eco/Standard/Premium)
                        Alias::new("subscription_tier"),// SaaS / creator plan (Free/Pro/Enterprise)
                        Alias::new("coverage_option"),  // insurance coverage product
                        Alias::new("add_on"),           // ancillary upsell (parking, breakfast, etc.)
                        Alias::new("equipment_unit"),   // rentable equipment or vehicle
                    ])
                    .to_owned(),
            )
            .await?;

        // ── 2. atlas_catalog_entries ─────────────────────────────────────────
        manager
            .create_table(
                Table::create()
                    .table(Alias::new("atlas_catalog_entries"))
                    .if_not_exists()
                    // ── Identity ──────────────────────────────────────────────
                    .col(
                        ColumnDef::new(Alias::new("id"))
                            .uuid()
                            .not_null()
                            .primary_key()
                            .default(Expr::cust("gen_random_uuid()")),
                    )
                    .col(ColumnDef::new(Alias::new("tenant_id")).uuid().not_null())
                    // ── Product definition ────────────────────────────────────
                    .col(
                        ColumnDef::new(Alias::new("entry_type"))
                            .custom(Alias::new("atlas_catalog_entry_type"))
                            .not_null(),
                    )
                    .col(ColumnDef::new(Alias::new("name")).string_len(255).not_null())
                    .col(ColumnDef::new(Alias::new("description")).text().null())
                    // ── Asset linkage (optional — room type → STR unit) ───────
                    .col(ColumnDef::new(Alias::new("asset_id")).uuid().null())
                    // ── Base pricing ──────────────────────────────────────────
                    .col(
                        ColumnDef::new(Alias::new("base_price_cents"))
                            .big_integer()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(Alias::new("currency"))
                            .char_len(3)
                            .not_null()
                            .default("USD"),
                    )
                    // NULL = one-time; 'nightly', 'monthly', 'annually'
                    .col(ColumnDef::new(Alias::new("billing_interval")).string_len(20).null())
                    // ── Availability ──────────────────────────────────────────
                    .col(
                        ColumnDef::new(Alias::new("is_available"))
                            .boolean()
                            .not_null()
                            .default(true),
                    )
                    .col(
                        ColumnDef::new(Alias::new("min_quantity"))
                            .integer()
                            .not_null()
                            .default(1),
                    )
                    .col(ColumnDef::new(Alias::new("max_quantity")).integer().null())
                    // ── App-specific product attributes ───────────────────────
                    .col(
                        ColumnDef::new(Alias::new("catalog_metadata"))
                            .json_binary()
                            .not_null()
                            .default(Expr::cust("'{}'")),
                    )
                    // ── Display ───────────────────────────────────────────────
                    .col(
                        ColumnDef::new(Alias::new("sort_order"))
                            .integer()
                            .not_null()
                            .default(0),
                    )
                    .col(ColumnDef::new(Alias::new("cover_image_attachment_id")).uuid().null())
                    // ── Timestamps ────────────────────────────────────────────
                    .col(
                        ColumnDef::new(Alias::new("created_at"))
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::cust("now()")),
                    )
                    .col(
                        ColumnDef::new(Alias::new("updated_at"))
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::cust("now()")),
                    )
                    // ── Foreign keys ──────────────────────────────────────────
                    .foreign_key(
                        ForeignKey::create()
                            .from(Alias::new("atlas_catalog_entries"), Alias::new("tenant_id"))
                            .to(Alias::new("tenant"), Alias::new("id"))
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(Alias::new("atlas_catalog_entries"), Alias::new("asset_id"))
                            .to(Alias::new("atlas_assets"), Alias::new("id"))
                            .on_delete(ForeignKeyAction::SetNull),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(
                                Alias::new("atlas_catalog_entries"),
                                Alias::new("cover_image_attachment_id"),
                            )
                            .to(Alias::new("attachment"), Alias::new("id"))
                            .on_delete(ForeignKeyAction::SetNull),
                    )
                    .to_owned(),
            )
            .await?;

        // ── 3. atlas_catalog_entries indexes ─────────────────────────────────
        manager
            .create_index(
                Index::create()
                    .name("atlas_catalog_tenant_type")
                    .table(Alias::new("atlas_catalog_entries"))
                    .col(Alias::new("tenant_id"))
                    .col(Alias::new("entry_type"))
                    .col(Alias::new("is_available"))
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("atlas_catalog_asset")
                    .table(Alias::new("atlas_catalog_entries"))
                    .col(Alias::new("tenant_id"))
                    .col(Alias::new("asset_id"))
                    .to_owned(),
            )
            .await?;

        // updated_at trigger
        manager
            .get_connection()
            .execute_unprepared(
                "CREATE TRIGGER atlas_catalog_entries_updated_at
                 BEFORE UPDATE ON atlas_catalog_entries
                 FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();",
            )
            .await?;

        // ── 4. atlas_catalog_rate_rules ───────────────────────────────────────
        manager
            .create_table(
                Table::create()
                    .table(Alias::new("atlas_catalog_rate_rules"))
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Alias::new("id"))
                            .uuid()
                            .not_null()
                            .primary_key()
                            .default(Expr::cust("gen_random_uuid()")),
                    )
                    .col(ColumnDef::new(Alias::new("catalog_entry_id")).uuid().not_null())
                    .col(ColumnDef::new(Alias::new("tenant_id")).uuid().not_null())
                    .col(ColumnDef::new(Alias::new("rule_name")).string_len(100).null())
                    // ── Applicability window ──────────────────────────────────
                    .col(ColumnDef::new(Alias::new("applies_from")).date().null())
                    .col(ColumnDef::new(Alias::new("applies_to")).date().null())
                    // bitmask: 1=Mon 2=Tue 4=Wed 8=Thu 16=Fri 32=Sat 64=Sun
                    .col(ColumnDef::new(Alias::new("day_of_week_mask")).integer().null())
                    .col(ColumnDef::new(Alias::new("min_duration")).integer().null())
                    // 'direct', 'ota', 'gds', 'corporate' — NULL = all channels
                    .col(ColumnDef::new(Alias::new("channel")).string_len(50).null())
                    // ── Pricing strategy (one or the other, not both) ─────────
                    .col(ColumnDef::new(Alias::new("price_override_cents")).big_integer().null())
                    .col(
                        ColumnDef::new(Alias::new("price_modifier_pct"))
                            .decimal_len(6, 2)
                            .null(),
                    )
                    // ── Rule priority & status ────────────────────────────────
                    .col(
                        ColumnDef::new(Alias::new("priority"))
                            .integer()
                            .not_null()
                            .default(0),
                    )
                    .col(
                        ColumnDef::new(Alias::new("is_active"))
                            .boolean()
                            .not_null()
                            .default(true),
                    )
                    .col(
                        ColumnDef::new(Alias::new("created_at"))
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::cust("now()")),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(
                                Alias::new("atlas_catalog_rate_rules"),
                                Alias::new("catalog_entry_id"),
                            )
                            .to(Alias::new("atlas_catalog_entries"), Alias::new("id"))
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(
                                Alias::new("atlas_catalog_rate_rules"),
                                Alias::new("tenant_id"),
                            )
                            .to(Alias::new("tenant"), Alias::new("id"))
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("atlas_catalog_rate_rules_entry")
                    .table(Alias::new("atlas_catalog_rate_rules"))
                    .col(Alias::new("catalog_entry_id"))
                    .col(Alias::new("applies_from"))
                    .col(Alias::new("applies_to"))
                    .to_owned(),
            )
            .await?;

        // ── 5. atlas_catalog_availability ────────────────────────────────────
        manager
            .create_table(
                Table::create()
                    .table(Alias::new("atlas_catalog_availability"))
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Alias::new("id"))
                            .uuid()
                            .not_null()
                            .primary_key()
                            .default(Expr::cust("gen_random_uuid()")),
                    )
                    .col(ColumnDef::new(Alias::new("catalog_entry_id")).uuid().not_null())
                    .col(ColumnDef::new(Alias::new("tenant_id")).uuid().not_null())
                    .col(ColumnDef::new(Alias::new("slot_date")).date().not_null())
                    .col(
                        ColumnDef::new(Alias::new("total_inventory"))
                            .integer()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(Alias::new("reserved_count"))
                            .integer()
                            .not_null()
                            .default(0),
                    )
                    // available_count = total_inventory - reserved_count (computed/stored)
                    // SeaORM can't express GENERATED ALWAYS AS in schema builder;
                    // we add it via raw SQL after the table creation.
                    .col(
                        ColumnDef::new(Alias::new("is_blocked"))
                            .boolean()
                            .not_null()
                            .default(false),
                    )
                    .col(ColumnDef::new(Alias::new("block_reason")).string_len(255).null())
                    .col(ColumnDef::new(Alias::new("override_price_cents")).big_integer().null())
                    .foreign_key(
                        ForeignKey::create()
                            .from(
                                Alias::new("atlas_catalog_availability"),
                                Alias::new("catalog_entry_id"),
                            )
                            .to(Alias::new("atlas_catalog_entries"), Alias::new("id"))
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(
                                Alias::new("atlas_catalog_availability"),
                                Alias::new("tenant_id"),
                            )
                            .to(Alias::new("tenant"), Alias::new("id"))
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        // Add the GENERATED ALWAYS AS stored column and UNIQUE constraint via raw SQL.
        manager
            .get_connection()
            .execute_unprepared(
                "ALTER TABLE atlas_catalog_availability
                    ADD COLUMN available_count INT
                        GENERATED ALWAYS AS (total_inventory - reserved_count) STORED;

                 ALTER TABLE atlas_catalog_availability
                    ADD CONSTRAINT atlas_catalog_availability_entry_date_unique
                        UNIQUE (catalog_entry_id, slot_date);",
            )
            .await?;

        // Indexes on the availability grid — critical path for booking availability checks.
        manager
            .create_index(
                Index::create()
                    .name("atlas_catalog_availability_entry")
                    .table(Alias::new("atlas_catalog_availability"))
                    .col(Alias::new("catalog_entry_id"))
                    .col(Alias::new("slot_date"))
                    .to_owned(),
            )
            .await?;

        // Partial index — only rows that still have supply and are not manually blocked.
        manager
            .get_connection()
            .execute_unprepared(
                "CREATE INDEX atlas_catalog_availability_available
                    ON atlas_catalog_availability(tenant_id, catalog_entry_id, slot_date)
                    WHERE available_count > 0 AND NOT is_blocked;",
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .get_connection()
            .execute_unprepared(
                "DROP TABLE IF EXISTS atlas_catalog_availability CASCADE;
                 DROP TABLE IF EXISTS atlas_catalog_rate_rules CASCADE;
                 DROP TABLE IF EXISTS atlas_catalog_entries CASCADE;
                 DROP TYPE IF EXISTS atlas_catalog_entry_type CASCADE;",
            )
            .await?;
        Ok(())
    }
}
