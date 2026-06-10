use sea_orm_migration::prelude::*;

/// Folio: Add `managed_account_id` to core PM tables.
///
/// # Purpose
///
/// Enables a Property Management Company (PMC) running in
/// `mode = 'property_management_co'` to scope assets, leases,
/// portfolios, and leads to a specific client account within their tenant.
///
/// # Existing behavior unchanged
///
/// All columns are nullable and default NULL, which means:
///   - Existing single-landlord deployments: all rows have NULL → no query change
///   - PMC deployments: rows created on behalf of a client carry the client's account_id
///
/// # Query pattern in PMC mode
///
/// When a PM is "acting as" client X (via X-Folio-Client-Account header):
///   WHERE managed_account_id = $client_account_id
///
/// When a PM is viewing aggregate across all clients:
///   WHERE tenant_id = $pm_tenant_id  (no managed_account_id filter)
///
/// # Generality note
///
/// This pattern (scoping rows to sub-tenants within an org) is reusable in
/// any multi-stakeholder app. The column name `managed_account_id` is chosen
/// over `client_id` to stay consistent with the Atlas `account` entity (G-04).
#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .get_connection()
            .execute_unprepared(
                "-- Core Folio tables that need per-client scoping in PMC mode.
                 -- Nullable: NULL = single-landlord (unchanged). UUID = PMC client book.

                 ALTER TABLE atlas_contract
                     ADD COLUMN IF NOT EXISTS managed_account_id UUID
                         REFERENCES account(id) ON DELETE SET NULL;

                 ALTER TABLE atlas_asset
                     ADD COLUMN IF NOT EXISTS managed_account_id UUID
                         REFERENCES account(id) ON DELETE SET NULL;

                 ALTER TABLE atlas_portfolio
                     ADD COLUMN IF NOT EXISTS managed_account_id UUID
                         REFERENCES account(id) ON DELETE SET NULL;

                 ALTER TABLE atlas_lead
                     ADD COLUMN IF NOT EXISTS managed_account_id UUID
                         REFERENCES account(id) ON DELETE SET NULL;

                 -- Partial indexes: only index rows with a client scope (PMC mode).
                 -- Keeps index overhead zero for single-landlord deployments.
                 CREATE INDEX IF NOT EXISTS idx_contract_managed_account
                     ON atlas_contract (tenant_id, managed_account_id)
                     WHERE managed_account_id IS NOT NULL;

                 CREATE INDEX IF NOT EXISTS idx_asset_managed_account
                     ON atlas_asset (tenant_id, managed_account_id)
                     WHERE managed_account_id IS NOT NULL;

                 CREATE INDEX IF NOT EXISTS idx_portfolio_managed_account
                     ON atlas_portfolio (tenant_id, managed_account_id)
                     WHERE managed_account_id IS NOT NULL;

                 CREATE INDEX IF NOT EXISTS idx_lead_managed_account
                     ON atlas_lead (tenant_id, managed_account_id)
                     WHERE managed_account_id IS NOT NULL;",
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .get_connection()
            .execute_unprepared(
                "DROP INDEX IF EXISTS idx_lead_managed_account;
                 DROP INDEX IF EXISTS idx_portfolio_managed_account;
                 DROP INDEX IF EXISTS idx_asset_managed_account;
                 DROP INDEX IF EXISTS idx_contract_managed_account;

                 ALTER TABLE atlas_lead      DROP COLUMN IF EXISTS managed_account_id;
                 ALTER TABLE atlas_portfolio DROP COLUMN IF EXISTS managed_account_id;
                 ALTER TABLE atlas_asset     DROP COLUMN IF EXISTS managed_account_id;
                 ALTER TABLE atlas_contract  DROP COLUMN IF EXISTS managed_account_id;",
            )
            .await?;
        Ok(())
    }
}
