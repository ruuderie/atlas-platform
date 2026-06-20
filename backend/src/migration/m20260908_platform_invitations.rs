use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .get_connection()
            .execute_unprepared(
                "CREATE TABLE IF NOT EXISTS platform_invite (
                    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
                    email TEXT NOT NULL,
                    role TEXT NOT NULL,
                    tenant_name TEXT NOT NULL,
                    invited_by TEXT NOT NULL,
                    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
                    expires_at TIMESTAMPTZ NOT NULL
                );
                
                CREATE INDEX IF NOT EXISTS idx_platform_invite_email ON platform_invite (email);
                
                INSERT INTO platform_invite (id, email, role, tenant_name, invited_by, created_at, expires_at)
                VALUES 
                    ('f5a25c60-1234-4bc3-9876-000000000001'::uuid, 'jill@newclient.com', 'Admin', 'Folio PM (new tenant)', 'Jamie Delaney', now() - interval '2 days', now() + interval '5 days'),
                    ('f5a25c60-1234-4bc3-9876-000000000002'::uuid, 'pedro@rioverde.br', 'Editor', 'Rio Verde PMC', 'Maria Fernandes', now() - interval '3 days', now() + interval '4 days')
                ON CONFLICT (id) DO NOTHING;"
            )
            .await?;
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .get_connection()
            .execute_unprepared("DROP TABLE IF EXISTS platform_invite;")
            .await?;
        Ok(())
    }
}
