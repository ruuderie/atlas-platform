use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();

        // 1. Create email_template table with tenant scoping
        let create_email_template_sql = r#"
            CREATE TABLE IF NOT EXISTS email_template (
                id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
                tenant_id UUID NOT NULL REFERENCES tenant(id) ON DELETE CASCADE,
                template_name VARCHAR(255) NOT NULL,
                subject_line VARCHAR(500) NOT NULL,
                html_body TEXT NOT NULL,
                merge_fields JSONB NOT NULL DEFAULT '[]'::jsonb,
                created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
                updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
                UNIQUE (tenant_id, template_name)
            );
            CREATE INDEX IF NOT EXISTS idx_email_template_tenant ON email_template (tenant_id);
        "#;
        db.execute_unprepared(create_email_template_sql).await?;

        // 2. Create activity_attachment table with foreign keys
        let create_activity_attachment_sql = r#"
            CREATE TABLE IF NOT EXISTS activity_attachment (
                activity_id UUID NOT NULL REFERENCES activity(id) ON DELETE CASCADE,
                file_url VARCHAR(1000) NOT NULL,
                file_name VARCHAR(255) NOT NULL,
                mime_type VARCHAR(100) NOT NULL,
                created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
                PRIMARY KEY (activity_id, file_url)
            );
            CREATE INDEX IF NOT EXISTS idx_activity_attachment_activity ON activity_attachment (activity_id);
        "#;
        db.execute_unprepared(create_activity_attachment_sql).await?;

        // 3. Seed rev_intake form schema for buildwithruud to fix missing page form issue
        let seed_rev_intake_sql = r#"
            DO $$
            DECLARE
                v_ruud_tenant_id UUID;
            BEGIN
                SELECT id INTO v_ruud_tenant_id FROM tenant WHERE slug = 'buildwithruud' OR name ILIKE '%buildwithruud%' LIMIT 1;
                
                IF v_ruud_tenant_id IS NOT NULL THEN
                    INSERT INTO form_schemas (id, tenant_id, name, slug, description, schema_json, created_at, updated_at)
                    VALUES (
                        gen_random_uuid(),
                        v_ruud_tenant_id,
                        'Real Estate Ventures Intake',
                        'rev_intake',
                        'Intake form for motivated sellers, cash buyers, private lenders, and joint venture partners.',
                        '{
                            "steps": [
                                {
                                    "id": "step1",
                                    "title": "Contact Details",
                                    "fields": [
                                        { "id": "first_name", "type": "text", "label": "First Name", "required": true },
                                        { "id": "last_name", "type": "text", "label": "Last Name", "required": true },
                                        { "id": "email", "type": "email", "label": "Email Address", "required": true },
                                        { "id": "phone", "type": "text", "label": "Phone (optional)", "required": false },
                                        { "id": "role", "type": "select", "label": "I am a...", "options": ["Cash Buyer / Investor", "Motivated Seller", "Tenant-Buyer (Lease-Option)", "Private Money Lender", "Joint Venture Partner", "Commercial Deal Referral", "Newsletter Subscriber Only", "Other"], "required": true },
                                        { "id": "message", "type": "textarea", "label": "Tell me more", "required": false }
                                    ]
                                }
                            ]
                        }'::jsonb,
                        NOW(),
                        NOW()
                    )
                    ON CONFLICT (tenant_id, slug) DO NOTHING;
                END IF;
            END $$;
        "#;
        db.execute_unprepared(seed_rev_intake_sql).await?;

        // 4. Seed default email templates for both buildwithruud and oplystusa
        let seed_templates_sql = r#"
            DO $$
            DECLARE
                v_tenant RECORD;
            BEGIN
                FOR v_tenant IN SELECT id FROM tenant LOOP
                    INSERT INTO email_template (id, tenant_id, template_name, subject_line, html_body, merge_fields, created_at, updated_at)
                    VALUES 
                    (
                        gen_random_uuid(),
                        v_tenant.id,
                        'Lead Follow-up',
                        'Thanks for reaching out! - Team',
                        '<p>Hi {{FirstName}},</p><p>Thank you for contacting us. We have received your inquiry and a team member will review it and get back to you shortly.</p><p>Best regards,<br/>Client Services</p>',
                        '["FirstName"]'::jsonb,
                        NOW(),
                        NOW()
                    ),
                    (
                        gen_random_uuid(),
                        v_tenant.id,
                        'Investor Intake Proposal',
                        'Real Estate Investment Opportunity',
                        '<p>Hi {{FirstName}} {{LastName}},</p><p>We are excited to share our latest premium off-market commercial opportunities with you.</p><p>Please review our latest listings at your earliest convenience.</p><p>Best regards,<br/>Investment Relations</p>',
                        '["FirstName", "LastName"]'::jsonb,
                        NOW(),
                        NOW()
                    )
                    ON CONFLICT (tenant_id, template_name) DO NOTHING;
                END LOOP;
            END $$;
        "#;
        db.execute_unprepared(seed_templates_sql).await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();
        db.execute_unprepared("DROP TABLE IF EXISTS activity_attachment;").await?;
        db.execute_unprepared("DROP TABLE IF EXISTS email_template;").await?;
        Ok(())
    }
}
