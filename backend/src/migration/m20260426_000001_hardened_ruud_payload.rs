use sea_orm_migration::prelude::*;

/// Hardened canonical home page payload migration for the buildwithruud tenant.
///
/// This is the terminal authoritative migration for the buildwithruud home page.
/// It supersedes the prior chain of silent-failure patches (000003–000006) by
/// enforcing strict failure modes:
///
///   - RAISE EXCEPTION if the tenant cannot be located (never silent skip)
///   - GET DIAGNOSTICS + RAISE EXCEPTION if the UPDATE affects 0 rows
///   - RAISE NOTICE on success so the outcome is visible in backend startup logs
///
/// The payload produced here uses `pt-32` on the first RawHtml block so that the
/// hero section correctly clears the fixed navigation bar. The outer `<main>`
/// wrapper in DynamicHomeLanding renders without padding — each block owns its
/// own layout, enabling full-bleed sections.
#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();

        let sql = r#"
            DO $$
            DECLARE
                v_ruud_id UUID;
                v_rows_affected INT;
            BEGIN
                -- Strategy 1: find by tenant name (most reliable)
                SELECT id INTO v_ruud_id
                FROM tenant
                WHERE name ILIKE '%buildwithruud%' OR name ILIKE '%ruuderie%'
                LIMIT 1;

                -- Strategy 2: fallback via registered app domain
                IF v_ruud_id IS NULL THEN
                    SELECT t.id INTO v_ruud_id
                    FROM tenant t
                    JOIN app_instances ai ON ai.tenant_id = t.id
                    JOIN app_domains ad ON ad.app_instance_id = ai.id
                    WHERE ad.domain_name ILIKE '%buildwithruud%'
                    LIMIT 1;
                END IF;

                -- Hard stop: tenant must exist. If it doesn't, fail loudly.
                IF v_ruud_id IS NULL THEN
                    RAISE EXCEPTION
                        'MIGRATION FAILED [m20260426_000001]: buildwithruud tenant not found. '
                        'Inspect the tenant and app_domains tables. '
                        'Cannot apply canonical home payload without a valid tenant_id.';
                END IF;

                -- Apply the canonical pt-32 home page payload
                UPDATE app_pages
                SET blocks_payload = '[
  {
    "RawHtml": {
      "content": "<div class=\"w-full pt-32 pb-24 px-4 md:px-[8.5rem] bg-surface text-on-surface\"><div class=\"grid grid-cols-1 lg:grid-cols-12 gap-12 lg:gap-8\"><div class=\"lg:col-span-7 flex flex-col justify-start\"><div class=\"inline-block bg-surface-container-high px-3 py-1 mb-8 jetbrains text-[0.625rem] font-medium tracking-widest text-on-surface-variant uppercase w-max\">VER: v1.0.4 // KERNEL_ACTIVE</div><h1 class=\"text-[5rem] lg:text-[7rem] leading-[0.85] font-extrabold tracking-tight text-[#0a385c] uppercase mb-12\">RUUD<br/>ERIE.</h1><p class=\"text-xl lg:text-2xl font-medium text-on-surface uppercase tracking-wide leading-relaxed max-w-2xl mb-16\">SYSTEMS ARCHITECT // SPECIALIZING IN <span class=\"text-[#b83b19]\">DISTRIBUTED INFRASTRUCTURE</span>, HIGH-PERFORMANCE RUST SYSTEMS, AND SCALABLE CLOUD FABRICS.</p><div class=\"flex flex-col md:flex-row gap-8 md:gap-16\"><div><div class=\"jetbrains text-[0.55rem] text-outline tracking-widest uppercase mb-1\">CURRENT FOCUS</div><div class=\"font-bold text-sm\">Sub-millisecond Latency Optimization</div></div><div><div class=\"jetbrains text-[0.55rem] text-outline tracking-widest uppercase mb-1\">STATUS</div><div class=\"font-bold text-sm flex items-center gap-2\"><div class=\"w-1.5 h-1.5 bg-[#b83b19]\"></div>AVAILABLE FOR CRITICAL OPS</div></div></div></div><div class=\"lg:col-span-5 flex flex-col gap-6\"><div class=\"bg-[#f4f5f5] p-8 border-l-4 border-[#b83b19] flex flex-col\"><div class=\"text-[#b83b19] text-4xl leading-none font-serif font-black mb-4\">\"</div><p class=\"italic text-on-surface font-medium text-lg mb-12\">\"Architecture is not about drawing boxes; it''s about defining the physics of the data flow.\"</p><div class=\"space-y-4\"><div><div class=\"flex justify-between items-end mb-1\"><span class=\"jetbrains text-[0.5rem] tracking-widest text-outline uppercase\">SYSTEM EFFICIENCY</span><span class=\"jetbrains text-[0.55rem] text-[#b83b19] font-bold\">99.98%</span></div><div class=\"w-full h-1 bg-outline-variant/30\"><div class=\"h-full bg-[#b83b19]\" style=\"width: 99.98%\"></div></div></div><div><div class=\"flex justify-between items-end mb-1\"><span class=\"jetbrains text-[0.5rem] tracking-widest text-outline uppercase\">RESOURCE OVERHEAD</span><span class=\"jetbrains text-[0.55rem] text-[#0a385c] font-bold\">2.4%</span></div><div class=\"w-full h-1 bg-outline-variant/30\"><div class=\"h-full bg-[#0a385c]\" style=\"width: 2.4%\"></div></div></div></div></div><div class=\"bg-black text-white relative overflow-hidden aspect-[16/9] flex items-end p-4 border border-outline-variant/20\"><div class=\"absolute inset-0 bg-[radial-gradient(circle_at_center,_var(--tw-gradient-stops))] from-white/20 via-black/90 to-black pointer-events-none\"></div><div class=\"absolute inset-0\" style=\"background-image: repeating-radial-gradient(circle at center, transparent, transparent 10px, rgba(255,255,255,0.05) 10px, rgba(255,255,255,0.05) 11px); opacity: 0.5;\"></div><div class=\"relative z-10 bg-white text-black jetbrains text-[0.5rem] px-2 py-1 tracking-widest font-bold\">LOC: /US/NY/BKLYN/11211</div></div></div></div></div>"
    }
  },
  {
    "FormBuilder": {
      "form_id": "rev_intake",
      "title": "Request Tailored CV",
      "description": "Input your protocol for a mission-specific credentials package.",
      "submit_button_text": "Initialize Retrieval",
      "form_classes": "space-y-8 w-full py-8 max-w-2xl mx-auto",
      "container_classes": "w-full px-4 md:px-[8.5rem] py-24 bg-[#f4f5f5] bg-[radial-gradient(#e5e7eb_1px,transparent_1px)] [background-size:16px_16px]",
      "button_classes": "w-full bg-[#b83b19] text-white py-4 jetbrains font-bold text-sm tracking-[0.2em] uppercase hover:bg-[#9a2f13] transition-colors rounded-none outline-none border-none shadow-none",
      "fields": [
        {
          "name": "email",
          "label": "Registry Email Address",
          "field_type": "email",
          "required": true,
          "placeholder": "user@organization.domain",
          "custom_classes": "w-full bg-transparent border-none border-b-2 border-outline-variant focus:border-[#b83b19] focus:ring-0 px-0 py-4 jetbrains text-lg text-outline placeholder:text-outline-variant/70 transition-all rounded-none",
          "label_classes": "jetbrains text-[0.55rem] uppercase tracking-[0.1em] text-outline text-center block mb-2"
        }
      ]
    }
  },
  {
    "RawHtml": {
      "content": "<div class=\"w-full px-4 md:px-[8.5rem] py-24 bg-surface text-on-surface\"><div class=\"grid grid-cols-1 md:grid-cols-3 gap-12\"><div class=\"space-y-4\"><div class=\"jetbrains text-[0.55rem] text-[#b83b19] font-bold tracking-widest uppercase mb-6\">CORE_01</div><h3 class=\"text-xl font-bold text-[#0a385c]\">DISTRIBUTED SYSTEMS</h3><p class=\"text-sm text-on-surface-variant leading-relaxed\">Designing fault-tolerant backends that scale horizontally across global cloud regions without compromising consistency.</p></div><div class=\"space-y-4\"><div class=\"jetbrains text-[0.55rem] text-[#b83b19] font-bold tracking-widest uppercase mb-6\">CORE_02</div><h3 class=\"text-xl font-bold text-[#0a385c]\">RUST PERFORMANCE</h3><p class=\"text-sm text-on-surface-variant leading-relaxed\">Memory-safe, high-concurrency systems built for bare-metal speed and absolute reliability in production.</p></div><div class=\"space-y-4\"><div class=\"jetbrains text-[0.55rem] text-[#b83b19] font-bold tracking-widest uppercase mb-6\">CORE_03</div><h3 class=\"text-xl font-bold text-[#0a385c]\">CLOUD ARCHITECTURE</h3><p class=\"text-sm text-on-surface-variant leading-relaxed\">Strategic infrastructure deployment leveraging Kubernetes, Terraform, and custom automation fabrics.</p></div></div></div>"
    }
  }
]'::jsonb
                WHERE slug = 'home' AND tenant_id = v_ruud_id;

                GET DIAGNOSTICS v_rows_affected = ROW_COUNT;

                -- Hard stop: the home page row must exist and be updated.
                IF v_rows_affected = 0 THEN
                    RAISE EXCEPTION
                        'MIGRATION FAILED [m20260426_000001]: UPDATE affected 0 rows for tenant %. '
                        'The home page row is missing in app_pages. '
                        'Verify the page was seeded by a prior migration.', v_ruud_id;
                END IF;

                RAISE NOTICE 'SUCCESS [m20260426_000001]: Updated % home page row(s) for tenant %.',
                    v_rows_affected, v_ruud_id;
            END $$;
        "#;

        db.execute_unprepared(sql).await?;
        Ok(())
    }

    async fn down(&self, _manager: &SchemaManager) -> Result<(), DbErr> {
        // No down migration — payload rollback is handled by redeploying
        // the previous migration version via a new forward migration.
        Ok(())
    }
}
