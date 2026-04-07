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
                -- Create api_tokens table
                CREATE TABLE api_tokens (
                    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
                    tenant_id UUID NOT NULL,
                    token_hash VARCHAR(255) NOT NULL,
                    scopes JSONB NOT NULL DEFAULT '[]',
                    expires_at TIMESTAMP WITH TIME ZONE,
                    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
                    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
                );

                -- Create webhook_endpoints table
                CREATE TABLE webhook_endpoints (
                    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
                    tenant_id UUID NOT NULL,
                    target_url VARCHAR(2048) NOT NULL,
                    secret_key VARCHAR(255) NOT NULL,
                    subscribed_events JSONB NOT NULL DEFAULT '[]',
                    is_active BOOLEAN NOT NULL DEFAULT true,
                    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
                    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
                );

                -- Create webhook_deliveries table
                CREATE TABLE webhook_deliveries (
                    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
                    endpoint_id UUID NOT NULL REFERENCES webhook_endpoints(id) ON DELETE CASCADE,
                    tenant_id UUID NOT NULL,
                    event_type VARCHAR(255) NOT NULL,
                    payload JSONB NOT NULL DEFAULT '{}',
                    status VARCHAR(50) NOT NULL DEFAULT 'pending', -- 'pending', 'sent', 'failed'
                    next_retry_at TIMESTAMP WITH TIME ZONE,
                    attempts INT NOT NULL DEFAULT 0,
                    response_status INT,
                    response_body TEXT,
                    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
                    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
                );

                -- Indexes
                CREATE INDEX idx_api_tokens_tenant_id ON api_tokens (tenant_id);
                CREATE INDEX idx_webhook_endpoints_tenant_id ON webhook_endpoints (tenant_id);
                CREATE INDEX idx_webhook_deliveries_endpoint_id ON webhook_deliveries (endpoint_id);
                CREATE INDEX idx_webhook_deliveries_tenant_id ON webhook_deliveries (tenant_id);
                CREATE INDEX idx_webhook_deliveries_status ON webhook_deliveries (status, next_retry_at);
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
                DROP TABLE IF EXISTS webhook_deliveries CASCADE;
                DROP TABLE IF EXISTS webhook_endpoints CASCADE;
                DROP TABLE IF EXISTS api_tokens CASCADE;
                "#,
            )
            .await?;

        Ok(())
    }
}
