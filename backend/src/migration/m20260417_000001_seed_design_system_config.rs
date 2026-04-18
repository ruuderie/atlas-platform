use sea_orm_migration::prelude::*;
use serde_json::json;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();

        // 1. OplystUSA Editorial Monolith Config
        let oplyst_design = json!({
            "design_config": {
                "heading_font": "font-display",
                "body_font": "font-sans",
                "meta_font": "font-sans uppercase tracking-widest",
                "border_radius_base": "rounded-lg",
                "container_strategy": "centered-standard",
                "background_pattern": "radial-glow",
                "elevation_strategy": "tonal-shifts",
                "button_padding": "px-10 py-4",
                "nav_layout": "floating-glass"
            }
        });

        let oplyst_sql = format!(
            "UPDATE app_instances 
             SET settings = COALESCE(settings, '{{}}'::jsonb) || '{}'::jsonb 
             WHERE tenant_id = (SELECT id FROM tenant WHERE slug = 'oplystusa' LIMIT 1);",
            oplyst_design.to_string()
        );
        db.execute_unprepared(&oplyst_sql).await?;

        // 2. Ruuderie Warm Industrial Config
        let ruuderie_design = json!({
            "design_config": {
                "heading_font": "font-sans",
                "body_font": "font-sans",
                "meta_font": "font-mono uppercase tracking-wider",
                "border_radius_base": "rounded-none",
                "container_strategy": "asymmetrical-gutters",
                "background_pattern": "blueprint-grid",
                "elevation_strategy": "flat-ghost",
                "button_padding": "px-12 py-5",
                "nav_layout": "solid-full"
            }
        });

        // "ruud" or "buildwithruud"
        let ruuderie_sql = format!(
            "UPDATE app_instances 
             SET settings = COALESCE(settings, '{{}}'::jsonb) || '{}'::jsonb 
             WHERE tenant_id IN (SELECT id FROM tenant WHERE name ILIKE '%buildwithruud%' OR name ILIKE '%ruud%');",
            ruuderie_design.to_string()
        );
        db.execute_unprepared(&ruuderie_sql).await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();

        let sql = r#"
            UPDATE app_instances 
            SET settings = settings - 'design_config';
        "#;
        db.execute_unprepared(sql).await?;

        Ok(())
    }
}
