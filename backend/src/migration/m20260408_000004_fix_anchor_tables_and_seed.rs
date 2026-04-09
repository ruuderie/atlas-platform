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
                v_bwr_tenant_id UUID;
            BEGIN
                -- 1. Create Core Types and Anchor Legacy Tables
                BEGIN
                    CREATE TYPE resume_category_enum AS ENUM (
                        'work', 
                        'education', 
                        'certification',
                        'skill', 
                        'project', 
                        'language', 
                        'volunteer', 
                        'extracurricular', 
                        'hobby'
                    );
                EXCEPTION
                    WHEN duplicate_object THEN null;
                END;

                CREATE TABLE IF NOT EXISTS page_headers (
                    route_path VARCHAR(255) PRIMARY KEY,
                    badge_text VARCHAR(255),
                    title VARCHAR(255) NOT NULL,
                    subtitle TEXT
                );

                CREATE TABLE IF NOT EXISTS resume_profiles (
                    id SERIAL PRIMARY KEY,
                    tenant_id UUID,
                    name VARCHAR(255) NOT NULL,
                    full_name VARCHAR(255) NOT NULL DEFAULT 'Document Header',
                    objective TEXT NOT NULL,
                    is_public BOOLEAN NOT NULL DEFAULT FALSE,
                    target_role VARCHAR(255),
                    contact_email VARCHAR(255),
                    contact_phone VARCHAR(255),
                    contact_location VARCHAR(255),
                    contact_link VARCHAR(255),
                    category_visibility JSONB DEFAULT '{}'::jsonb,
                    category_order JSONB NOT NULL DEFAULT '["work", "education", "certification", "project", "skill", "volunteer", "extracurricular", "language", "hobby"]'::jsonb,
                    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
                );

                CREATE TABLE IF NOT EXISTS resume_entries (
                    id SERIAL PRIMARY KEY,
                    tenant_id UUID,
                    category resume_category_enum NOT NULL,
                    title VARCHAR(500) NOT NULL,
                    subtitle VARCHAR(500),
                    date_range VARCHAR(255),
                    bullets JSONB NOT NULL DEFAULT '[]'::jsonb,
                    metadata JSONB DEFAULT '{}',
                    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP
                );

                CREATE TABLE IF NOT EXISTS resume_profile_entries (
                    profile_id INTEGER NOT NULL REFERENCES resume_profiles(id) ON DELETE CASCADE,
                    entry_id INTEGER NOT NULL REFERENCES resume_entries(id) ON DELETE CASCADE,
                    display_order INTEGER NOT NULL DEFAULT 0,
                    is_visible BOOLEAN NOT NULL DEFAULT TRUE,
                    overrides JSONB DEFAULT '{}'::jsonb,
                    PRIMARY KEY (profile_id, entry_id)
                );



                CREATE TABLE IF NOT EXISTS lead_capture_options (
                    id SERIAL PRIMARY KEY,
                    tenant_id UUID,
                    value_key VARCHAR(255) NOT NULL,
                    label VARCHAR(255) NOT NULL,
                    is_active BOOLEAN NOT NULL DEFAULT TRUE,
                    display_order INTEGER NOT NULL DEFAULT 0,
                    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
                    CONSTRAINT lead_capture_options_tenant_value_key_key UNIQUE(tenant_id, value_key)
                );

                -- Verify if lead_capture_options has initial data
                IF NOT EXISTS (SELECT 1 FROM lead_capture_options LIMIT 1) THEN
                    INSERT INTO lead_capture_options (value_key, label, is_active, display_order) VALUES
                    ('mailing_list', 'Join Mailing List', true, 10),
                    ('resume', 'Request Tailored CV', true, 20)
                    ON CONFLICT DO NOTHING;
                END IF;

                -- 2. Populate missing Seed data for buildwithruud ONLY
                SELECT id INTO v_bwr_tenant_id FROM tenant WHERE name ILIKE '%buildwithruud%' LIMIT 1;
                
                IF v_bwr_tenant_id IS NOT NULL THEN
                    
                    -- Check if resume profiles already populated for this tenant
                    IF NOT EXISTS (SELECT 1 FROM resume_profiles WHERE tenant_id = v_bwr_tenant_id LIMIT 1) THEN

                        -- Insert Resume Entries scoped to tenant
                        INSERT INTO resume_entries (tenant_id, category, title, subtitle, date_range, bullets, metadata) VALUES
                        -- Client Projects
                        (v_bwr_tenant_id, 'work', 'Salesforce Architect/Technical Lead', 'Enterprise Cloud Solutions Inc.', 'August 2024 – Present',
                         '["Led two major ERP implementation projects, a commercial heating solution with Field Service Lightning integration and a Salesforce ERP integration with MailChimp, DocuSign, and a custom forecasting model.", "Managed all aspects of the projects, including developer oversight, system design, client relations, and overall project management, ensuring timely delivery and client satisfaction.", "Architected and oversaw complex data migration strategies for the commercial heating project, seamlessly transferring legacy data into the new Salesforce-based ERP system.", "Designed and implemented a custom integration framework to connect Salesforce with external systems (MailChimp, DocuSign) and a bespoke forecasting model, significantly enhancing the client''s operational efficiency and decision-making capabilities."]'::jsonb,
                         '{"is_client_project": true}'::jsonb),

                        (v_bwr_tenant_id, 'work', 'Salesforce Architect/Engineer', 'Swan Bitcoin', 'April 2023 – August 2024',
                         '["Redesigned the application architecture to bolster security measures and enhance data accuracy during data exchange.", "Managed, wrote, and maintained custom API integrations with external systems, leveraging Apex and Rust programming language for integration testing.", "Implemented Salesforce Shield to enhance data security and compliance with industry standards.", "Introduced version control system to manage source code efficiently, enabling easy collaboration and tracking changes.", "Established a streamlined release process for production, reducing deployment errors and enhancing delivery efficiency."]'::jsonb,
                         '{"is_client_project": true}'::jsonb),

                        (v_bwr_tenant_id, 'work', 'Blockchain Developer', 'Zing.It', 'January 2023 – July 2023',
                         '["Developed TypeScript and React unit tests to ensure the reliability and stability of UI components within the blockchain web application.", "Implemented an integration with SendGrid to facilitate the automatic sending of emails for key platform events, enhancing user engagement and communication effectiveness.", "Employed solidity smart contracts to enable secure and transparent transactional processes within the platform.", "Actively participated in code reviews, providing valuable feedback to peers and promoting code quality and best practices."]'::jsonb,
                         '{"is_client_project": true}'::jsonb),

                        (v_bwr_tenant_id, 'work', 'Salesforce Consultant', 'Grayscale Investments', 'April 2022 - June 2022',
                         '["Audited Grayscale''s Salesforce systems and codebase to identify areas for improvement and ensure compliance with best practices.", "Collaborated with the wealth management and advisory team to understand their sales process and develop solutions to enhance efficiency.", "Implemented lead integration from external systems to streamline data capture and management.", "Consolidated Grayscale''s data structure to facilitate seamless management of clients at different stages of the sales process.", "Provided training and guidance to Grayscale''s team on utilizing Salesforce effectively for their sales operations."]'::jsonb,
                         '{"is_client_project": true}'::jsonb),

                        (v_bwr_tenant_id, 'work', 'Salesforce Architect and Senior Developer', 'Principle Studios', 'April 2022 – December 2022',
                         '["Worked with client to build a Transportation Management System using Salesforce from scratch.", "Developed a Trigger Framework to support end to end Load management.", "Developed a security model that required information to be hidden from users working in different offices, managing, and enabling customer credit and the secure tracking and billing of shipper loads.", "Developed capabilities that support the Automatic and Manual Credit Approval, directly reducing the time it takes for agents to complete their work.", "Used SFDX for metadata deployment from environment to environment."]'::jsonb,
                         '{"is_client_project": true}'::jsonb),

                        (v_bwr_tenant_id, 'work', 'Salesforce Lead Engineer / Implementation Architect', 'Mercury Healthcare', 'April 2020 – April 2022',
                         '["Worked with Key Clients including Prisma Healthcare, Ascension Health and Advocate Aurora Health to implement multiple ISV products across the Sales Cloud, Service Cloud and Marketing Cloud.", "Implemented SSO for 50+ projects with Mercury Health Customers connecting Microsoft, Okta, Salesforce, and Appian Cloud.", "Developed Continuous Integration Process using SFDX for feature delivery.", "Configured Tableau and Snowflake for data visualizations used in Salesforce."]'::jsonb,
                         '{"is_client_project": true}'::jsonb),

                        -- Employment
                        (v_bwr_tenant_id, 'work', 'Salesforce Technical Architect / Lead Software Engineer', 'Oplyst International, LLC', 'January 2012 - Present',
                         '["Provide ongoing support & technical consultation for medium to large enterprises & non-profits", "Einstein Analytics and Discovery implementations with large datasets.", "Custom lightning development on Sales Cloud, Service Cloud, Health Cloud, and Nonprofit Starter Pack.", "Integrated salesforce instances with external services using REST, Platform Events & Heroku.", "Building Product APIs using Rust, Python and Deno Backends"]'::jsonb,
                         '{"is_client_project": false}'::jsonb),

                        (v_bwr_tenant_id, 'work', 'CRM Technical Manager / Salesforce Architect', 'ZIPARI Inc', 'June 2018 - February 2019',
                         '["Managing a 5 person team of Senior Engineers, in addition to being the Principal Salesforce Architect across all Salesforce Products", "Providing technical design, architecture & Api specs for new features", "Implemented numerous CI builds for deployment and product packaging using salesforce dx", "Successfully designed and built a call center application on Lightning Service Cloud", "Completed an internal audit and created a strategy to bring code coverage from 0% to 85% across all products in less than 2 months."]'::jsonb,
                         '{"is_client_project": false}'::jsonb),

                        (v_bwr_tenant_id, 'work', 'Salesforce Architect / Lead Software Engineer', 'Evariant Inc', 'March 2017 - March 2018',
                         '["Migrated 30 + companies to lightning to use the latest version of our application.", "Re-built Healthcare marketing lead list builder in using lightning components. List builder is used by 50 + health networks across the United States on a day to day basis.", "Built reusable lightning components used across Salesforce Classic and Lightning.", "Wrote custom REST Api to ingest healthcare case data."]'::jsonb,
                         '{"is_client_project": false}'::jsonb),

                        -- Projects
                        (v_bwr_tenant_id, 'project', 'RS Business Network', 'Full-Stack Web Application for Business Listings Management', '',
                         '["Architected a high-performance business network platform using Rust and Svelte, enabling efficient search and management of business listings for thousands of users.", "Developed RESTful APIs with Rust and the Axum framework, optimizing data handling and ensuring robust security for user and listing data, achieving sub-100ms response times under load.", "Designed a responsive admin interface with Svelte, Vite, and Tailwind CSS, integrating shadcn-svelte components to deliver an intuitive user experience.", "Containerized backend and frontend services using Docker, streamlining deployments across cloud environments and ensuring consistency with zero-downtime updates.", "Implemented a PostgreSQL-backed data layer with sqlx for scalable storage and retrieval, supporting rapid queries on large datasets and enabling future growth.", "Leveraged modern DevOps practices, including CI/CD pipelines with GitHub Actions and pnpm for frontend dependency management, reducing deployment cycles by 40%."]'::jsonb,
                         '{"slug": "rs-business-network", "tags": ["Rust", "Svelte", "Axum", "PostgreSQL", "Docker"], "status": "Completed"}'::jsonb),

                        (v_bwr_tenant_id, 'project', 'Basecamp', 'Smart contract deployment on the Starknet blockchain', '',
                         '["Built a modular Next.js frontend with Web3.js integration, enabling seamless interaction with StarkNet smart contracts, improving user accessibility across diverse devices.", "Authored smart contracts and deployment scripts using snfoundry, optimizing gas efficiency on StarkNet and ensuring compliance with Ethereum security standards.", "Structured the project for scalability, separating frontend (/packages/nextjs) and blockchain components (/packages/snfoundry), facilitating maintenance and future feature additions.", "Integrated Yarn for dependency management and Vitest for unit testing, achieving 90%+ code coverage and ensuring reliability in production environments."]'::jsonb,
                         '{"slug": "basecamp", "tags": ["Starknet", "Cairo", "Next.js", "Web3.js", "snfoundry"], "status": "Completed"}'::jsonb),

                        (v_bwr_tenant_id, 'project', 'Loan Landscape', 'Full-Stack Commercial Loan and Property Analysis Platform', '',
                         '["Developed a scalable property analysis platform using Nuxt.js, TypeScript, and Rust, delivering real-time insights for property investment decisions.", "Implemented a high-throughput data processing pipeline in Rust, handling complex property calculations with sub-second latency, tested up to 10,000 daily queries.", "Created interactive data visualization components with Nuxt.js and D3.js, enabling users to explore property metrics intuitively.", "Designed a responsive frontend with Vue.js-based Nuxt.js, ensuring cross-platform compatibility and accessibility, validated through user testing.", "Architected a modular system separating frontend, backend, and data processing layers, enhancing maintainability and enabling independent scaling of services."]'::jsonb,
                         '{"slug": "loan-landscape", "tags": ["Rust", "Nuxt.js", "Vue.js", "D3.js", "TypeScript"], "status": "Completed"}'::jsonb),

                        -- Certifications
                        (v_bwr_tenant_id, 'certification', 'Salesforce Certified Administrator', '', '', '[]'::jsonb, '{"is_training": false}'::jsonb),
                        (v_bwr_tenant_id, 'certification', 'Salesforce Certified Platform Developer I', '', '', '[]'::jsonb, '{"is_training": false}'::jsonb),
                        (v_bwr_tenant_id, 'certification', 'Salesforce Certified Sharing and Visibility Architect', '', '', '[]'::jsonb, '{"is_training": false}'::jsonb),
                        (v_bwr_tenant_id, 'certification', 'Salesforce Certified Agentforce Specialist', '', '', '[]'::jsonb, '{"is_training": false}'::jsonb),
                        (v_bwr_tenant_id, 'certification', 'Salesforce Certified Service Cloud Consultant', '', '', '[]'::jsonb, '{"is_training": false}'::jsonb),
                        (v_bwr_tenant_id, 'certification', 'Salesforce Certified AI Associate', '', '', '[]'::jsonb, '{"is_training": false}'::jsonb),
                        (v_bwr_tenant_id, 'certification', 'Salesforce Certified Platform App Builder', '', '', '[]'::jsonb, '{"is_training": false}'::jsonb),
                        (v_bwr_tenant_id, 'certification', 'Einstein Discover Training at Salesforce Office in Herndon, VA', '', '', '[]'::jsonb, '{"is_training": true}'::jsonb),
                        (v_bwr_tenant_id, 'certification', 'Einstein Analytics Advanced Training at Salesforce Office in Herndon, VA', '', '', '[]'::jsonb, '{"is_training": true}'::jsonb);

                        -- Insert Resume Profile
                        INSERT INTO resume_profiles (tenant_id, name, full_name, objective, is_public, target_role, category_visibility, category_order) VALUES
                        (v_bwr_tenant_id, 'Default', 'Ruud Salym Erie', 'A comprehensive log of active systems architecture, smart contract deployments, and client configurations.', true, 'Systems Architect', '{"work":true,"education":true,"certification":true,"project":true,"skill":true,"language":true}'::jsonb, '["work", "education", "certification", "project", "skill", "volunteer", "extracurricular", "language", "hobby"]'::jsonb);

                        -- Map entries to profile
                        INSERT INTO resume_profile_entries (profile_id, entry_id)
                        SELECT 
                            (SELECT id FROM resume_profiles WHERE tenant_id = v_bwr_tenant_id ORDER BY id DESC LIMIT 1), 
                            id 
                        FROM resume_entries 
                        WHERE tenant_id = v_bwr_tenant_id;

                    END IF;

                END IF;

                -- Insert global page headers
                INSERT INTO page_headers (route_path, badge_text, title, subtitle) VALUES
                ('/projects', 'CLIENT AND PERSONAL REPOSITORIES', 'TECHNICAL PORTFOLIO', 'Engineering resilient infrastructures across blockchains, decentralized cloud, and sub-millisecond Rust backends.'),
                ('/certifications', 'INDEX_REF_07 // VERIFIED INFRA', 'CERTIFICATIONS', 'Cryptographically and institutionally verified architecture authorizations.'),
                ('/blog', 'ENGINEERING DISSERTATIONS', 'TECHNICAL WRITING', 'Documentation on distributed systems architecture, Bitcoin cryptography, Salesforce APEX algorithms, and low-latency infrastructure design.'),
                ('/resume', 'ARCHITECTURAL OVERVIEW', 'RESUME & CURRICULUM VITAE', 'Technical proficiencies and professional timeline documented for vendor qualification.'),
                ('/services', 'ANCHOR // ADVISORY SERVICES', 'ARCHITECTURE & STRATEGY', 'Elite engineering consultation and systems design for the digital frontier.'),
                ('/book', 'DISCOVERY // B2B', 'CONSULTATION', 'Schedule a preliminary systems architecture review.')
                ON CONFLICT (route_path) DO UPDATE SET
                    badge_text = EXCLUDED.badge_text,
                    title = EXCLUDED.title,
                    subtitle = EXCLUDED.subtitle;

            END $$;
        "##;
        
        db.execute_unprepared(sql).await?;
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();
        let sql = r#"
            -- Downgrade logic to drop tables if absolutely necessary
            -- Generally shouldn't aggressively drop these due to data loss
        "#;
        db.execute_unprepared(sql).await?;
        Ok(())
    }
}
