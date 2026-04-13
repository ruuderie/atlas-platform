use sea_orm_migration::prelude::*;
use serde_json::json;
use uuid::Uuid;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();
        
        let oplyst_tenant_id = Uuid::new_v4();
        let oplyst_slug = "oplystusa";
        println!("Seeding OplystUSA Tenant ID: {}", oplyst_tenant_id);

        // Intelligently maps to the new `tenant` schema with deterministic string `slug`
        let raw_insert = format!(
            "INSERT INTO tenant (id, name, description, site_status, slug, created_at, updated_at) 
             VALUES ('{}', '{}', '{}', 'ACTIVE', '{}', NOW(), NOW()) 
             ON CONFLICT (slug) DO UPDATE SET site_status = 'ACTIVE';",
            oplyst_tenant_id, "OplystUSA Commercial Capital", "A national real estate bridge lender", oplyst_slug
        );
        db.execute_unprepared(&raw_insert).await?;

        // Seed site settings for the real "Commercial Capital" copy!
        let settings = vec![
            ("current_focus", "Commercial Real Estate & Bridge Loans"),
            ("status", "Funding Available"),
            ("hero_quote", "Direct non-bank financing for real estate investors. Fast closings and flexible terms."),
            ("hero_subtitle", "DIRECT LENDER // SPECIALIZING IN COMMERCIAL REAL ESTATE, RENTAL PORTFOLIOS, AND FIX-AND-FLIP FINANCING."),
            ("site_title", "COMMERCIAL CAPITAL"),
            ("lc_title", "Get Funded"),
            ("lc_desc", "Submit your basic loan scenario for a rapid term sheet."),
            ("lc_label", "Borrower Email Address"),
            ("lc_placeholder", "investor@example.com"),
            ("lc_btn", "Request Term Sheet"),
            ("lc_footer", "* We will review your scenario within 24 hours."),
            ("lc_endpoint", "/api/contact"),
            ("status_color", "#10b981"),
            ("b2b_enabled", "true"),
            ("meta_title", "Commercial Capital - Direct Lending"),
            ("meta_description", "Non-bank direct lender providing bridge loans, commercial real estate financing, and hard money lending packages."),
        ];

        for (key, val) in settings {
            let escape_val = val.replace("'", "''");
            let sql_set = format!(
                "INSERT INTO site_settings (id, tenant_id, key, value) 
                VALUES (gen_random_uuid(), (SELECT id FROM tenant WHERE slug='{}'), '{}', '{}')
                ON CONFLICT (tenant_id, key) DO UPDATE SET value = EXCLUDED.value;",
                oplyst_slug, key, escape_val
            );
            db.execute_unprepared(&sql_set).await?;
        }

        // SEED THE FORM SCHEMAS (CRE Application and HOA/Condo Application)
        let cre_schema = json!({
            "steps": [
                {
                    "id": "step1",
                    "title": "Loan Scenario",
                    "fields": [
                        { "id": "loan_amount", "type": "currency", "label": "Requested Loan Amount", "required": true },
                        { "id": "property_type", "type": "select", "label": "Property Type", "options": ["Multifamily", "Mixed Use", "Office", "Retail", "Industrial", "Self-Storage"], "required": true },
                        { "id": "loan_purpose", "type": "select", "label": "Loan Purpose", "options": ["Purchase", "Refinance", "Cash-Out Refinance"], "required": true }
                    ]
                },
                {
                    "id": "step2",
                    "title": "Property Details",
                    "fields": [
                        { "id": "property_address", "type": "address", "label": "Subject Property Address", "required": true },
                        { "id": "current_value", "type": "currency", "label": "As-Is Value", "required": true },
                        { "id": "gross_rent", "type": "currency", "label": "Annual Gross Rent", "required": false }
                    ]
                },
                {
                    "id": "step3",
                    "title": "Borrower Info",
                    "fields": [
                        { "id": "borrower_name", "type": "text", "label": "Borrowing Entity or Individual", "required": true },
                        { "id": "contact_email", "type": "email", "label": "Email Address", "required": true },
                        { "id": "contact_phone", "type": "phone", "label": "Phone Number", "required": true }
                    ]
                }
            ]
        });

        let hoa_schema = json!({
            "steps": [
                {
                    "id": "step1",
                    "title": "Association Info",
                    "fields": [
                        { "id": "association_name", "type": "text", "label": "Association Name", "required": true },
                        { "id": "num_units", "type": "number", "label": "Total Number of Units", "required": true },
                        { "id": "loan_amount", "type": "currency", "label": "Requested Loan Amount ($100k-$5M)", "required": true }
                    ]
                },
                {
                    "id": "step2",
                    "title": "Project Details",
                    "fields": [
                        { "id": "project_description", "type": "textarea", "label": "Description of Project/Repairs", "required": true },
                        { "id": "special_assessment", "type": "boolean", "label": "Is there a Special Assessment approved?", "required": true },
                        { "id": "monthly_dues", "type": "currency", "label": "Average Monthly Dues per Unit", "required": true }
                    ]
                }
            ]
        });

        let insert_forms = format!(
            "INSERT INTO form_schemas (id, tenant_id, name, slug, description, schema_json)
             VALUES 
             (gen_random_uuid(), (SELECT id FROM tenant WHERE slug='{}'), 'Commercial Real Estate Loan', 'cre-application', 'Standard CRE loan application for multifamily, retail, office, etc.', '{}'),
             (gen_random_uuid(), (SELECT id FROM tenant WHERE slug='{}'), 'HOA & Condominium Association Loan', 'hoa-condo-application', 'Unsecured lending for condo associations to fund capital improvements.', '{}')
             ON CONFLICT (tenant_id, slug) DO NOTHING;",
             oplyst_slug, cre_schema.to_string().replace("'", "''"), oplyst_slug, hoa_schema.to_string().replace("'", "''")
        );
        
        db.execute_unprepared(&insert_forms).await?;

        // SEED THE DYNAMIC BLOCKS FOR THE LANDING PAGE
        let blocks_json = json!([
            {
                "Hero": {
                    "heading": "Direct Lending for Commercial Real Estate Investors",
                    "subheading": "Fast approvals. Flexible terms. Reliable execution.",
                    "primary_cta_text": "Apply Now",
                    "primary_cta_link": "/apply",
                    "background_image": "/assets/hero-bg.webp"
                }
            },
            {
                "Callout": {
                    "text": "We provide bridge loans, fix-and-flip, and rental portfolio financing nationwide.",
                    "style": "primary"
                }
            },
            {
                "Grid": {
                    "columns": 3,
                    "items": [
                        { "title": "Bridge Loans", "description": "12-24 month terms for acquisitions or refinancing.", "icon": "account_balance" },
                        { "title": "Rental Portfolios", "description": "DSCR loans tailored for landlords.", "icon": "real_estate_agent" },
                        { "title": "HOA Lending", "description": "Capital improvements for condo associations.", "icon": "apartment" }
                    ]
                }
            }
        ]);

        let insert_blocks = format!(
            "UPDATE app_pages SET dynamic_blocks_json = '{}', updated_at = NOW()
             WHERE tenant_id = (SELECT id FROM tenant WHERE slug='{}') AND slug = 'home';",
            blocks_json.to_string().replace("'", "''"),
            oplyst_slug
        );

        db.execute_unprepared(&insert_blocks).await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();
        let oplyst_slug = "oplystusa";
        db.execute_unprepared(&format!("DELETE FROM tenant WHERE slug='{}';", oplyst_slug)).await?;
        Ok(())
    }
}
