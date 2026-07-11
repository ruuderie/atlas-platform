//! # Migration: G24 `atlas_quotes` — Pre-purchase Pricing Proposals
//!
//! ## Tables created
//!
//! | Table | Purpose |
//! |-------|---------|
//! | `atlas_quotes` | The proposal header — validity, status, totals, recipient |
//! | `atlas_quote_line_items` | Individual pricing lines referencing G26 catalog entries |
//!
//! ## Commerce chain position
//!
//! ```text
//! G26 atlas_catalog_entries  →  atlas_quotes  →  G23 atlas_reservations
//!                                             ↓
//!                                   atlas_quote_line_items
//! ```
//!
//! Both `atlas_reservations.quote_id` FK references in G23 v1 and v2 migrations
//! point to this table — those are nullable FKs that become valid once G24 is deployed.

use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // ── 1. atlas_quotes (proposal header) ────────────────────────────────
        manager
            .create_table(
                Table::create()
                    .table(Alias::new("atlas_quotes"))
                    .if_not_exists()
                    .col(ColumnDef::new(Alias::new("id")).uuid().primary_key())
                    .col(ColumnDef::new(Alias::new("tenant_id")).uuid().not_null())
                    // Subject entity — the thing being quoted (asset, service, package, etc.)
                    .col(
                        ColumnDef::new(Alias::new("subject_entity_type"))
                            .string()
                            .null(),
                    )
                    .col(
                        ColumnDef::new(Alias::new("subject_entity_id"))
                            .uuid()
                            .null(),
                    )
                    // Recipient
                    .col(
                        ColumnDef::new(Alias::new("recipient_user_id"))
                            .uuid()
                            .null(),
                    )
                    .col(
                        ColumnDef::new(Alias::new("recipient_email"))
                            .string()
                            .null(),
                    )
                    .col(ColumnDef::new(Alias::new("recipient_name")).string().null())
                    // Downstream links
                    .col(ColumnDef::new(Alias::new("campaign_id")).uuid().null()) // G19
                    .col(ColumnDef::new(Alias::new("catalog_entry_id")).uuid().null()) // G26
                    // Quote identity
                    .col(ColumnDef::new(Alias::new("quote_number")).string().null())
                    .col(ColumnDef::new(Alias::new("title")).string().not_null())
                    .col(ColumnDef::new(Alias::new("notes")).text().null())
                    // Status lifecycle: draft → sent → accepted/rejected/expired → converted
                    .col(
                        ColumnDef::new(Alias::new("status"))
                            .string()
                            .not_null()
                            .default("draft"),
                    )
                    // Financial totals (derived from line items — kept in sync by QuoteService)
                    .col(
                        ColumnDef::new(Alias::new("subtotal_cents"))
                            .big_integer()
                            .not_null()
                            .default(0),
                    )
                    .col(
                        ColumnDef::new(Alias::new("discount_cents"))
                            .big_integer()
                            .not_null()
                            .default(0),
                    )
                    .col(
                        ColumnDef::new(Alias::new("tax_cents"))
                            .big_integer()
                            .not_null()
                            .default(0),
                    )
                    .col(
                        ColumnDef::new(Alias::new("total_cents"))
                            .big_integer()
                            .not_null()
                            .default(0),
                    )
                    .col(
                        ColumnDef::new(Alias::new("currency"))
                            .string()
                            .not_null()
                            .default("USD"),
                    )
                    // Validity
                    .col(
                        ColumnDef::new(Alias::new("valid_from"))
                            .timestamp_with_time_zone()
                            .null(),
                    )
                    .col(
                        ColumnDef::new(Alias::new("valid_until"))
                            .timestamp_with_time_zone()
                            .null(),
                    )
                    // Conversion tracking
                    .col(
                        ColumnDef::new(Alias::new("accepted_at"))
                            .timestamp_with_time_zone()
                            .null(),
                    )
                    .col(
                        ColumnDef::new(Alias::new("rejected_at"))
                            .timestamp_with_time_zone()
                            .null(),
                    )
                    .col(
                        ColumnDef::new(Alias::new("converted_reservation_id"))
                            .uuid()
                            .null(),
                    ) // G23
                    .col(
                        ColumnDef::new(Alias::new("revision_number"))
                            .integer()
                            .not_null()
                            .default(1),
                    )
                    .col(ColumnDef::new(Alias::new("superseded_by_id")).uuid().null()) // self-ref
                    // Metadata
                    .col(
                        ColumnDef::new(Alias::new("quote_metadata"))
                            .json_binary()
                            .null(),
                    )
                    .col(
                        ColumnDef::new(Alias::new("created_by_user_id"))
                            .uuid()
                            .null(),
                    )
                    .col(
                        ColumnDef::new(Alias::new("created_at"))
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(Alias::new("updated_at"))
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .to_owned(),
            )
            .await?;

        // ── 2. atlas_quote_line_items ─────────────────────────────────────────
        manager
            .create_table(
                Table::create()
                    .table(Alias::new("atlas_quote_line_items"))
                    .if_not_exists()
                    .col(ColumnDef::new(Alias::new("id")).uuid().primary_key())
                    .col(ColumnDef::new(Alias::new("tenant_id")).uuid().not_null())
                    .col(ColumnDef::new(Alias::new("quote_id")).uuid().not_null())
                    .col(
                        ColumnDef::new(Alias::new("line_item_type"))
                            .string()
                            .not_null(),
                    )
                    .col(ColumnDef::new(Alias::new("catalog_entry_id")).uuid().null()) // G26
                    .col(
                        ColumnDef::new(Alias::new("description"))
                            .string()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(Alias::new("quantity"))
                            .integer()
                            .not_null()
                            .default(1),
                    )
                    .col(
                        ColumnDef::new(Alias::new("unit_price_cents"))
                            .big_integer()
                            .not_null(),
                    )
                    // For percentage_discount: 0-10000 (basis points). Otherwise 0.
                    .col(
                        ColumnDef::new(Alias::new("discount_basis_points"))
                            .integer()
                            .not_null()
                            .default(0),
                    )
                    .col(
                        ColumnDef::new(Alias::new("line_total_cents"))
                            .big_integer()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(Alias::new("sort_order"))
                            .integer()
                            .not_null()
                            .default(0),
                    )
                    .col(
                        ColumnDef::new(Alias::new("line_metadata"))
                            .json_binary()
                            .null(),
                    )
                    .col(
                        ColumnDef::new(Alias::new("created_at"))
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .to_owned(),
            )
            .await?;

        // ── 3. Indexes ────────────────────────────────────────────────────────
        manager
            .create_index(
                Index::create()
                    .table(Alias::new("atlas_quotes"))
                    .name("idx_atlas_quotes_tenant_status")
                    .col(Alias::new("tenant_id"))
                    .col(Alias::new("status"))
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .table(Alias::new("atlas_quotes"))
                    .name("idx_atlas_quotes_subject")
                    .col(Alias::new("tenant_id"))
                    .col(Alias::new("subject_entity_type"))
                    .col(Alias::new("subject_entity_id"))
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .table(Alias::new("atlas_quote_line_items"))
                    .name("idx_atlas_quote_line_items_quote")
                    .col(Alias::new("quote_id"))
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(
                Table::drop()
                    .table(Alias::new("atlas_quote_line_items"))
                    .to_owned(),
            )
            .await?;
        manager
            .drop_table(Table::drop().table(Alias::new("atlas_quotes")).to_owned())
            .await?;
        Ok(())
    }
}
