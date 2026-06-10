use sea_orm_migration::prelude::*;

/// Extend platform_products with full Product Launch Engine fields:
/// - launch_mode lifecycle state machine
/// - pre-order pricing + Stripe linkage
/// - per-variant pre-order cap support
/// - cached waitlist_count for fast admin display
#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .get_connection()
            .execute_unprepared(
                "-- Launch mode state machine
                 -- draft → pre_launch → waitlist → active | beta → deprecated
                 ALTER TABLE platform_products
                     ADD COLUMN IF NOT EXISTS launch_mode
                         TEXT NOT NULL DEFAULT 'draft'
                         CONSTRAINT chk_platform_product_launch_mode
                         CHECK (launch_mode IN (
                             'draft', 'pre_launch', 'waitlist',
                             'active', 'beta', 'invite_only', 'deprecated'
                         )),

                     -- Pre-order settings (product-level defaults)
                     ADD COLUMN IF NOT EXISTS pre_order_enabled        BOOLEAN NOT NULL DEFAULT false,
                     ADD COLUMN IF NOT EXISTS pre_order_price_cents    INTEGER,
                     ADD COLUMN IF NOT EXISTS pre_order_currency       TEXT NOT NULL DEFAULT 'USD',
                     ADD COLUMN IF NOT EXISTS stripe_price_id          TEXT,

                     -- Founding / early-bird cap (null = unlimited)
                     ADD COLUMN IF NOT EXISTS pre_order_cap            INTEGER,
                     ADD COLUMN IF NOT EXISTS pre_order_sold           INTEGER NOT NULL DEFAULT 0,

                     -- Cached aggregate (denormalized for admin list performance)
                     ADD COLUMN IF NOT EXISTS waitlist_count           INTEGER NOT NULL DEFAULT 0,

                     -- Platform-level tenant_id for lead capture
                     -- nil-UUID = platform sentinel (leads not scoped to any customer tenant)
                     ADD COLUMN IF NOT EXISTS sentinel_tenant_id       UUID;

                 CREATE INDEX IF NOT EXISTS idx_platform_products_launch_mode
                     ON platform_products (launch_mode);",
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .get_connection()
            .execute_unprepared(
                "DROP INDEX IF EXISTS idx_platform_products_launch_mode;
                 ALTER TABLE platform_products
                     DROP COLUMN IF EXISTS launch_mode,
                     DROP COLUMN IF EXISTS pre_order_enabled,
                     DROP COLUMN IF EXISTS pre_order_price_cents,
                     DROP COLUMN IF EXISTS pre_order_currency,
                     DROP COLUMN IF EXISTS stripe_price_id,
                     DROP COLUMN IF EXISTS pre_order_cap,
                     DROP COLUMN IF EXISTS pre_order_sold,
                     DROP COLUMN IF EXISTS waitlist_count,
                     DROP COLUMN IF EXISTS sentinel_tenant_id;",
            )
            .await?;
        Ok(())
    }
}
