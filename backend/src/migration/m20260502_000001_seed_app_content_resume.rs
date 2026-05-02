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
                SELECT id INTO v_bwr_tenant_id FROM tenant WHERE name ILIKE '%buildwithruud%' LIMIT 1;
                
                IF v_bwr_tenant_id IS NOT NULL THEN
                    
                    -- Check if resume profiles already populated in app_content for this tenant
                    IF NOT EXISTS (SELECT 1 FROM app_content WHERE tenant_id = v_bwr_tenant_id AND collection_type = 'resume_entry' LIMIT 1) THEN

                        -- Insert Resume Entries scoped to tenant
                        INSERT INTO app_content (tenant_id, collection_type, title, payload) VALUES
                        -- Client Projects
                        (v_bwr_tenant_id, 'resume_entry', 'Salesforce Architect/Technical Lead', jsonb_build_object(
                            'category', 'work',
                            'subtitle', 'Enterprise Cloud Solutions Inc.',
                            'date_range', 'August 2024 – Present',
                            'bullets', '["Led two major ERP implementation projects, a commercial heating solution with Field Service Lightning integration and a Salesforce ERP integration with MailChimp, DocuSign, and a custom forecasting model.", "Managed all aspects of the projects, including developer oversight, system design, client relations, and overall project management, ensuring timely delivery and client satisfaction.", "Architected and oversaw complex data migration strategies for the commercial heating project, seamlessly transferring legacy data into the new Salesforce-based ERP system.", "Designed and implemented a custom integration framework to connect Salesforce with external systems (MailChimp, DocuSign) and a bespoke forecasting model, significantly enhancing the client''s operational efficiency and decision-making capabilities."]'::jsonb,
                            'metadata', '{"is_client_project": true}'::jsonb
                        )),

                        (v_bwr_tenant_id, 'resume_entry', 'Salesforce Architect/Engineer', jsonb_build_object(
                            'category', 'work',
                            'subtitle', 'Swan Bitcoin',
                            'date_range', 'April 2023 – August 2024',
                            'bullets', '["Redesigned the application architecture to bolster security measures and enhance data accuracy during data exchange.", "Managed, wrote, and maintained custom API integrations with external systems, leveraging Apex and Rust programming language for integration testing.", "Implemented Salesforce Shield to enhance data security and compliance with industry standards.", "Introduced version control system to manage source code efficiently, enabling easy collaboration and tracking changes.", "Established a streamlined release process for production, reducing deployment errors and enhancing delivery efficiency."]'::jsonb,
                            'metadata', '{"is_client_project": true}'::jsonb
                        )),

                        (v_bwr_tenant_id, 'resume_entry', 'Blockchain Developer', jsonb_build_object(
                            'category', 'work',
                            'subtitle', 'Zing.It',
                            'date_range', 'January 2023 – July 2023',
                            'bullets', '["Developed TypeScript and React unit tests to ensure the reliability and stability of UI components within the blockchain web application.", "Implemented an integration with SendGrid to facilitate the automatic sending of emails for key platform events, enhancing user engagement and communication effectiveness.", "Employed solidity smart contracts to enable secure and transparent transactional processes within the platform.", "Actively participated in code reviews, providing valuable feedback to peers and promoting code quality and best practices."]'::jsonb,
                            'metadata', '{"is_client_project": true}'::jsonb
                        )),

                        (v_bwr_tenant_id, 'resume_entry', 'Salesforce Consultant', jsonb_build_object(
                            'category', 'work',
                            'subtitle', 'Grayscale Investments',
                            'date_range', 'April 2022 - June 2022',
                            'bullets', '["Audited Grayscale''s Salesforce systems and codebase to identify areas for improvement and ensure compliance with best practices.", "Collaborated with the wealth management and advisory team to understand their sales process and develop solutions to enhance efficiency.", "Implemented lead integration from external systems to streamline data capture and management.", "Consolidated Grayscale''s data structure to facilitate seamless management of clients at different stages of the sales process.", "Provided training and guidance to Grayscale''s team on utilizing Salesforce effectively for their sales operations."]'::jsonb,
                            'metadata', '{"is_client_project": true}'::jsonb
                        )),

                        (v_bwr_tenant_id, 'resume_entry', 'Salesforce Architect and Senior Developer', jsonb_build_object(
                            'category', 'work',
                            'subtitle', 'Principle Studios',
                            'date_range', 'April 2022 – December 2022',
                            'bullets', '["Worked with client to build a Transportation Management System using Salesforce from scratch.", "Developed a Trigger Framework to support end to end Load management.", "Developed a security model that required information to be hidden from users working in different offices, managing, and enabling customer credit and the secure tracking and billing of shipper loads.", "Developed capabilities that support the Automatic and Manual Credit Approval, directly reducing the time it takes for agents to complete their work.", "Used SFDX for metadata deployment from environment to environment."]'::jsonb,
                            'metadata', '{"is_client_project": true}'::jsonb
                        )),

                        (v_bwr_tenant_id, 'resume_entry', 'Salesforce Lead Engineer / Implementation Architect', jsonb_build_object(
                            'category', 'work',
                            'subtitle', 'Mercury Healthcare',
                            'date_range', 'April 2020 – April 2022',
                            'bullets', '["Worked with Key Clients including Prisma Healthcare, Ascension Health and Advocate Aurora Health to implement multiple ISV products across the Sales Cloud, Service Cloud and Marketing Cloud.", "Implemented SSO for 50+ projects with Mercury Health Customers connecting Microsoft, Okta, Salesforce, and Appian Cloud.", "Developed Continuous Integration Process using SFDX for feature delivery.", "Configured Tableau and Snowflake for data visualizations used in Salesforce."]'::jsonb,
                            'metadata', '{"is_client_project": true}'::jsonb
                        )),

                        -- Employment
                        (v_bwr_tenant_id, 'resume_entry', 'Salesforce Technical Architect / Lead Software Engineer', jsonb_build_object(
                            'category', 'work',
                            'subtitle', 'Oplyst International, LLC',
                            'date_range', 'January 2012 - Present',
                            'bullets', '["Provide ongoing support & technical consultation for medium to large enterprises & non-profits", "Einstein Analytics and Discovery implementations with large datasets.", "Custom lightning development on Sales Cloud, Service Cloud, Health Cloud, and Nonprofit Starter Pack.", "Integrated salesforce instances with external services using REST, Platform Events & Heroku.", "Building Product APIs using Rust, Python and Deno Backends"]'::jsonb,
                            'metadata', '{"is_client_project": false}'::jsonb
                        )),

                        (v_bwr_tenant_id, 'resume_entry', 'CRM Technical Manager / Salesforce Architect', jsonb_build_object(
                            'category', 'work',
                            'subtitle', 'ZIPARI Inc',
                            'date_range', 'June 2018 - February 2019',
                            'bullets', '["Managing a 5 person team of Senior Engineers, in addition to being the Principal Salesforce Architect across all Salesforce Products", "Providing technical design, architecture & Api specs for new features", "Implemented numerous CI builds for deployment and product packaging using salesforce dx", "Successfully designed and built a call center application on Lightning Service Cloud", "Completed an internal audit and created a strategy to bring code coverage from 0% to 85% across all products in less than 2 months."]'::jsonb,
                            'metadata', '{"is_client_project": false}'::jsonb
                        )),

                        (v_bwr_tenant_id, 'resume_entry', 'Salesforce Architect / Lead Software Engineer', jsonb_build_object(
                            'category', 'work',
                            'subtitle', 'Evariant Inc',
                            'date_range', 'March 2017 - March 2018',
                            'bullets', '["Migrated 30 + companies to lightning to use the latest version of our application.", "Re-built Healthcare marketing lead list builder in using lightning components. List builder is used by 50 + health networks across the United States on a day to day basis.", "Built reusable lightning components used across Salesforce Classic and Lightning.", "Wrote custom REST Api to ingest healthcare case data."]'::jsonb,
                            'metadata', '{"is_client_project": false}'::jsonb
                        )),

                        -- Projects
                        (v_bwr_tenant_id, 'resume_entry', 'RS Business Network', jsonb_build_object(
                            'category', 'project',
                            'subtitle', 'Full-Stack Web Application for Business Listings Management',
                            'date_range', '',
                            'bullets', '["Architected a high-performance business network platform using Rust and Svelte, enabling efficient search and management of business listings for thousands of users.", "Developed RESTful APIs with Rust and the Axum framework, optimizing data handling and ensuring robust security for user and listing data, achieving sub-100ms response times under load.", "Designed a responsive admin interface with Svelte, Vite, and Tailwind CSS, integrating shadcn-svelte components to deliver an intuitive user experience.", "Containerized backend and frontend services using Docker, streamlining deployments across cloud environments and ensuring consistency with zero-downtime updates.", "Implemented a PostgreSQL-backed data layer with sqlx for scalable storage and retrieval, supporting rapid queries on large datasets and enabling future growth.", "Leveraged modern DevOps practices, including CI/CD pipelines with GitHub Actions and pnpm for frontend dependency management, reducing deployment cycles by 40%."]'::jsonb,
                            'metadata', '{"slug": "rs-business-network", "tags": ["Rust", "Svelte", "Axum", "PostgreSQL", "Docker"], "status": "Completed"}'::jsonb
                        )),

                        (v_bwr_tenant_id, 'resume_entry', 'Basecamp', jsonb_build_object(
                            'category', 'project',
                            'subtitle', 'Smart contract deployment on the Starknet blockchain',
                            'date_range', '',
                            'bullets', '["Built a modular Next.js frontend with Web3.js integration, enabling seamless interaction with StarkNet smart contracts, improving user accessibility across diverse devices.", "Authored smart contracts and deployment scripts using snfoundry, optimizing gas efficiency on StarkNet and ensuring compliance with Ethereum security standards.", "Structured the project for scalability, separating frontend (/packages/nextjs) and blockchain components (/packages/snfoundry), facilitating maintenance and future feature additions.", "Integrated Yarn for dependency management and Vitest for unit testing, achieving 90%+ code coverage and ensuring reliability in production environments."]'::jsonb,
                            'metadata', '{"slug": "basecamp", "tags": ["Starknet", "Cairo", "Next.js", "Web3.js", "snfoundry"], "status": "Completed"}'::jsonb
                        )),

                        (v_bwr_tenant_id, 'resume_entry', 'Loan Landscape', jsonb_build_object(
                            'category', 'project',
                            'subtitle', 'Full-Stack Commercial Loan and Property Analysis Platform',
                            'date_range', '',
                            'bullets', '["Developed a scalable property analysis platform using Nuxt.js, TypeScript, and Rust, delivering real-time insights for property investment decisions.", "Implemented a high-throughput data processing pipeline in Rust, handling complex property calculations with sub-second latency, tested up to 10,000 daily queries.", "Created interactive data visualization components with Nuxt.js and D3.js, enabling users to explore property metrics intuitively.", "Designed a responsive frontend with Vue.js-based Nuxt.js, ensuring cross-platform compatibility and accessibility, validated through user testing.", "Architected a modular system separating frontend, backend, and data processing layers, enhancing maintainability and enabling independent scaling of services."]'::jsonb,
                            'metadata', '{"slug": "loan-landscape", "tags": ["Rust", "Nuxt.js", "Vue.js", "D3.js", "TypeScript"], "status": "Completed"}'::jsonb
                        )),

                        -- Certifications
                        (v_bwr_tenant_id, 'resume_entry', 'Salesforce Certified Administrator', jsonb_build_object('category', 'certification', 'bullets', '[]'::jsonb, 'metadata', '{"is_training": false}'::jsonb)),
                        (v_bwr_tenant_id, 'resume_entry', 'Salesforce Certified Platform Developer I', jsonb_build_object('category', 'certification', 'bullets', '[]'::jsonb, 'metadata', '{"is_training": false}'::jsonb)),
                        (v_bwr_tenant_id, 'resume_entry', 'Salesforce Certified Sharing and Visibility Architect', jsonb_build_object('category', 'certification', 'bullets', '[]'::jsonb, 'metadata', '{"is_training": false}'::jsonb)),
                        (v_bwr_tenant_id, 'resume_entry', 'Salesforce Certified Agentforce Specialist', jsonb_build_object('category', 'certification', 'bullets', '[]'::jsonb, 'metadata', '{"is_training": false}'::jsonb)),
                        (v_bwr_tenant_id, 'resume_entry', 'Salesforce Certified Service Cloud Consultant', jsonb_build_object('category', 'certification', 'bullets', '[]'::jsonb, 'metadata', '{"is_training": false}'::jsonb)),
                        (v_bwr_tenant_id, 'resume_entry', 'Salesforce Certified AI Associate', jsonb_build_object('category', 'certification', 'bullets', '[]'::jsonb, 'metadata', '{"is_training": false}'::jsonb)),
                        (v_bwr_tenant_id, 'resume_entry', 'Salesforce Certified Platform App Builder', jsonb_build_object('category', 'certification', 'bullets', '[]'::jsonb, 'metadata', '{"is_training": false}'::jsonb)),
                        (v_bwr_tenant_id, 'resume_entry', 'Einstein Discover Training at Salesforce Office in Herndon, VA', jsonb_build_object('category', 'certification', 'bullets', '[]'::jsonb, 'metadata', '{"is_training": true}'::jsonb)),
                        (v_bwr_tenant_id, 'resume_entry', 'Einstein Analytics Advanced Training at Salesforce Office in Herndon, VA', jsonb_build_object('category', 'certification', 'bullets', '[]'::jsonb, 'metadata', '{"is_training": true}'::jsonb));

                        -- Insert Resume Profile
                        INSERT INTO app_content (tenant_id, collection_type, title, payload) VALUES
                        (v_bwr_tenant_id, 'resume_profile', 'Default', jsonb_build_object(
                            'name', 'Default',
                            'full_name', 'Ruud Salym Erie',
                            'objective', 'A comprehensive log of active systems architecture, smart contract deployments, and client configurations.',
                            'is_public', true,
                            'target_role', 'Systems Architect',
                            'category_visibility', '{"work":true,"education":true,"certification":true,"project":true,"skill":true,"language":true}'::jsonb,
                            'category_order', '["work", "education", "certification", "project", "skill", "volunteer", "extracurricular", "language", "hobby"]'::jsonb
                        ));
                    END IF;

                END IF;

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
