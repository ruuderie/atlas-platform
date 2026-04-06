use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .get_connection()
            .execute_unprepared(
                r#"
                -- Create billing_plans table
                CREATE TABLE billing_plans (
                    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
                    name VARCHAR(255) NOT NULL,
                    price BIGINT NOT NULL, -- price in cents
                    currency VARCHAR(10) NOT NULL,
                    interval VARCHAR(50) NOT NULL, -- e.g., 'month', 'year'
                    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
                    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
                );

                -- Create tenant_subscriptions table
                CREATE TABLE tenant_subscriptions (
                    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
                    tenant_id UUID NOT NULL,
                    plan_id UUID NOT NULL REFERENCES billing_plans(id),
                    status VARCHAR(50) NOT NULL, -- 'Active', 'Past_Due', 'Canceled'
                    current_period_end TIMESTAMP WITH TIME ZONE NOT NULL,
                    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
                    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
                );

                -- Create transactions table
                CREATE TABLE transactions (
                    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
                    tenant_id UUID NOT NULL,
                    provider VARCHAR(50) NOT NULL, -- 'Stripe', 'Paddle', 'Zaprite', 'BTCPay'
                    amount BIGINT NOT NULL,
                    currency VARCHAR(10) NOT NULL,
                    provider_tx_id VARCHAR(255),
                    status VARCHAR(50) NOT NULL, -- 'Pending', 'Completed', 'Failed'
                    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
                );

                -- Indexes
                CREATE INDEX idx_tenant_subscriptions_tenant_id ON tenant_subscriptions (tenant_id);
                CREATE INDEX idx_transactions_tenant_id ON transactions (tenant_id);
                CREATE UNIQUE INDEX idx_transactions_provider_tx_id ON transactions (provider, provider_tx_id);

                -- Seed original plans
                INSERT INTO billing_plans (name, price, currency, interval) VALUES
                ('Network Starter', 0, 'USD', 'month'),
                ('Enterprise Anchor', 19900, 'USD', 'month');
                "#,
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .get_connection()
            .execute_unprepared(
                r#"
                DROP TABLE IF EXISTS transactions CASCADE;
                DROP TABLE IF EXISTS tenant_subscriptions CASCADE;
                DROP TABLE IF EXISTS billing_plans CASCADE;
                "#,
            )
            .await?;

        Ok(())
    }
}
