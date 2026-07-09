#![allow(dead_code)]

//! Migration: add `'owner_occupied'` to the `atlas_asset_status` PostgreSQL enum type.
//!
//! `atlas_asset_status` is a native PG enum created in `m20260601_g10_assets`.
//! Existing values: active, inactive, under_maintenance, listed_for_sale,
//!                  decommissioned, pending_inspection.
//!
//! New value: `owner_occupied`
//!   Assigned to assets created by Property Owner Lite users.
//!   Semantics: the owner lives in the property; no active tenancy or lease.
//!   On upgrade to Landlord, the status transitions into the full lifecycle
//!   (vacant, leased, etc.) as configured — the G-10 row is not replaced.
//!
//! Note: PostgreSQL ADD VALUE to an enum cannot be transactional (no rollback).
//! The down() migration is therefore a no-op — the value stays in the type
//! but simply goes unused if the feature is rolled back at the application layer.

use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // ADD VALUE is idempotent via IF NOT EXISTS (PG 9.6+).
        manager
            .get_connection()
            .execute_unprepared(
                "ALTER TYPE atlas_asset_status ADD VALUE IF NOT EXISTS 'owner_occupied';",
            )
            .await?;

        Ok(())
    }

    async fn down(&self, _manager: &SchemaManager) -> Result<(), DbErr> {
        // PostgreSQL does not support removing enum values.
        // The 'owner_occupied' value is left in place; application code
        // simply stops issuing it if the feature is disabled.
        Ok(())
    }
}
