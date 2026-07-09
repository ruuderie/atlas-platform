#![allow(dead_code)]

//! Migration: create `atlas_asset_value_history` table.
//!
//! G-10 extension — a dedicated ledger of property valuation snapshots over time.
//! Enables the value history chart on the Property Owner Lite and Landlord dashboards.
//!
//! Each row is a point-in-time valuation for a specific asset, typed by source
//! (`crate::types::pm::PropertyValueSource`). Multiple sources can coexist for
//! the same date — e.g. a Zillow AVM estimate alongside a manual owner entry.
//!
//! Columns:
//!   id            UUID PK
//!   asset_id      UUID FK → atlas_assets.id  (CASCADE DELETE)
//!   tenant_id     UUID FK → app_tenants.id   (row-level isolation)
//!   user_id       UUID — who logged this entry
//!   source        TEXT — PropertyValueSource discriminator
//!   source_ref    TEXT NULL — URL, appraiser name, AVM report ID, document ID, etc.
//!   value_cents   BIGINT — property value in minor currency units (avoids Decimal drift)
//!   currency_code CHAR(3) — ISO 4217 (default 'USD')
//!   valued_on     DATE — the date this valuation applies to (not necessarily created_at)
//!   note          TEXT NULL — owner note (e.g. "after roof replacement")
//!   created_at    TIMESTAMPTZ

use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .get_connection()
            .execute_unprepared(
                "CREATE TABLE IF NOT EXISTS atlas_asset_value_history (
                    id            UUID        NOT NULL DEFAULT gen_random_uuid() PRIMARY KEY,
                    asset_id      UUID        NOT NULL,
                    tenant_id     UUID        NOT NULL,
                    user_id       UUID        NOT NULL,
                    source        TEXT        NOT NULL
                                  CHECK (source IN (
                                      'manual',
                                      'purchase_price',
                                      'zillow_avm',
                                      'county_record',
                                      'certified_appraisal',
                                      'bank_appraisal',
                                      'agent_cma'
                                  )),
                    source_ref    TEXT        NULL,
                    value_cents   BIGINT      NOT NULL CHECK (value_cents > 0),
                    currency_code CHAR(3)     NOT NULL DEFAULT 'USD',
                    valued_on     DATE        NOT NULL,
                    note          TEXT        NULL,
                    created_at    TIMESTAMPTZ NOT NULL DEFAULT NOW()
                );",
            )
            .await?;

        // Cascade deletes when the parent asset is removed.
        manager
            .get_connection()
            .execute_unprepared(
                "ALTER TABLE atlas_asset_value_history \
                 ADD CONSTRAINT fk_value_history_asset \
                 FOREIGN KEY (asset_id) REFERENCES atlas_assets(id) ON DELETE CASCADE;",
            )
            .await?;

        // Primary query pattern: time-series for a single asset, ordered by date.
        manager
            .get_connection()
            .execute_unprepared(
                "CREATE INDEX IF NOT EXISTS idx_asset_value_history_asset_date \
                 ON atlas_asset_value_history (asset_id, valued_on DESC);",
            )
            .await?;

        // Secondary: per-tenant listing (for portfolio-level value charts).
        manager
            .get_connection()
            .execute_unprepared(
                "CREATE INDEX IF NOT EXISTS idx_asset_value_history_tenant \
                 ON atlas_asset_value_history (tenant_id, valued_on DESC);",
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .get_connection()
            .execute_unprepared(
                "DROP TABLE IF EXISTS atlas_asset_value_history;",
            )
            .await?;

        Ok(())
    }
}
