//! m20261002_seed_ruuderie_folio_domain
//!
//! Ensures the `ruuderie` tenant, its `property_management` app_instance,
//! and the `folio1.atlas.oply.co` app_domain all exist in the database.
//!
//! The previous migration `m20261001_fix_existing_ruuderie_domains` silently
//! did nothing because the `ruuderie` tenant did not yet exist.  This migration
//! creates it (idempotently) so that the magic-link backend endpoint can
//! resolve `folio1.atlas.oply.co` from `app_domains` and return 200 instead
//! of 400 (which folio's server-fn was surfacing as a 500 to the browser).
//!
//! Idempotent: every INSERT is guarded by IS NULL / NOT EXISTS.

use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();
        let sql = r##"
            DO $$
            DECLARE
                v_ruud_id        UUID;
                v_folio_app_id   UUID;
                v_network_app_id UUID;
            BEGIN
                -- 1. Ensure ruuderie tenant exists
                SELECT id INTO v_ruud_id FROM tenant WHERE name = 'ruuderie' LIMIT 1;
                IF v_ruud_id IS NULL THEN
                    v_ruud_id := gen_random_uuid();
                    INSERT INTO tenant (id, name, description, slug, created_at, updated_at)
                    VALUES (
                        v_ruud_id,
                        'ruuderie',
                        'Ruuderie – Property Management',
                        'ruuderie',
                        NOW(), NOW()
                    );
                END IF;

                -- 2. Ensure property_management app_instance exists for ruuderie
                SELECT id INTO v_folio_app_id
                FROM app_instances
                WHERE tenant_id = v_ruud_id AND app_type = 'property_management'
                LIMIT 1;

                IF v_folio_app_id IS NULL THEN
                    v_folio_app_id := gen_random_uuid();
                    INSERT INTO app_instances (id, tenant_id, app_type, settings, created_at, updated_at)
                    VALUES (
                        v_folio_app_id,
                        v_ruud_id,
                        'property_management',
                        '{"site_title": "Folio", "contact_email": "admin@ruuderie.com"}'::jsonb,
                        NOW(), NOW()
                    );
                END IF;

                -- 3. Register folio1.atlas.oply.co
                IF NOT EXISTS (SELECT 1 FROM app_domains WHERE domain_name = 'folio1.atlas.oply.co') THEN
                    INSERT INTO app_domains (id, app_instance_id, domain_name)
                    VALUES (gen_random_uuid(), v_folio_app_id, 'folio1.atlas.oply.co');
                END IF;

                -- 4. Register folio.ruuderie.dev.atlas.oply.co (dev subdomain)
                IF NOT EXISTS (SELECT 1 FROM app_domains WHERE domain_name = 'folio.ruuderie.dev.atlas.oply.co') THEN
                    INSERT INTO app_domains (id, app_instance_id, domain_name)
                    VALUES (gen_random_uuid(), v_folio_app_id, 'folio.ruuderie.dev.atlas.oply.co');
                END IF;

                -- 5. Ensure network_instance app exists for ruuderie and register its domains
                SELECT id INTO v_network_app_id
                FROM app_instances
                WHERE tenant_id = v_ruud_id AND app_type = 'network_instance'
                LIMIT 1;

                IF v_network_app_id IS NULL THEN
                    v_network_app_id := gen_random_uuid();
                    INSERT INTO app_instances (id, tenant_id, app_type, settings, created_at, updated_at)
                    VALUES (
                        v_network_app_id,
                        v_ruud_id,
                        'network_instance',
                        '{"site_title": "Ruuderie Network"}'::jsonb,
                        NOW(), NOW()
                    );
                END IF;

                IF NOT EXISTS (SELECT 1 FROM app_domains WHERE domain_name = 'network.ruuderie.dev.atlas.oply.co') THEN
                    INSERT INTO app_domains (id, app_instance_id, domain_name)
                    VALUES (gen_random_uuid(), v_network_app_id, 'network.ruuderie.dev.atlas.oply.co');
                END IF;

            END $$;
        "##;

        db.execute_unprepared(sql).await?;
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();
        db.execute_unprepared(
            "DELETE FROM app_domains WHERE domain_name IN (
                'folio1.atlas.oply.co',
                'folio.ruuderie.dev.atlas.oply.co',
                'network.ruuderie.dev.atlas.oply.co'
            );
            DELETE FROM app_instances WHERE tenant_id = (SELECT id FROM tenant WHERE name = 'ruuderie');
            DELETE FROM tenant WHERE name = 'ruuderie';",
        )
        .await?;
        Ok(())
    }
}
