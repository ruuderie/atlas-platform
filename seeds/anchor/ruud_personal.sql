INSERT INTO site_settings (key, value) VALUES 
    ('current_focus', 'Technical Architect @ Hipten'),
    ('status', 'Unavailable For New Projects'),
    ('hero_quote', 'Vires in Numeris. Systems architecture is not defined by lines, but by cryptographic proofs and immutable data flows.'),
    ('hero_subtitle', 'TECHNICAL ARCHITECT AND BACKEND ENGINEER // SPECIALIZING IN ENTERPRISE CLOUD SOLUTIONS, SALESFORCE, AND RUST APPLICATIONS.'),
    ('site_title', 'BuildWithRuud'),
    ('lead_capture_title', 'Join My Newsletter'),
    ('lead_capture_desc', 'Get insights on AI, Salesforce, Rust, and Real Estate.'),
    ('lead_capture_label', 'Email Address'),
    ('lead_capture_placeholder', 'user@organization.domain'),
    ('lead_capture_btn', 'Subscribe'),
    ('lead_capture_footer', '* Check your email to confirm your subscription.'),
    ('lead_capture_endpoint', '/api/DownloadResume'),
    ('status_color', '#ff5449'),
    ('webhook_url', ''),
    ('admin_email', 'ruud@oply.co'),
    ('booking_url', 'https://cal.com/ruuderie/15min'),
    ('terms_html', '# Terms of Service\n\nPlease review our terms.'),
    ('privacy_html', '# Privacy Policy\n\nWe respect your digital privacy.'),
    ('github_url', 'https://github.com/ruuderie'),
    ('x_url', 'https://x.com/ruud_awakening'),
    ('linkedin_url', 'https://linkedin.com/in/ruudsalymerie'),
    ('meta_title', 'Ruud Salym Erie - Technical Architect'),
    ('meta_description', 'Technical Architect and Software Engineer specializing in Rust, Salesforce, and high-performance enterprise applications.'),
    ('og_image', '')
ON CONFLICT (key) DO UPDATE SET value = EXCLUDED.value;

INSERT INTO nav_items (label, href, display_order) VALUES
('WORK', '#', 10),
('BLOG', '/blog', 30),
('INVESTMENTS', '#', 40);

INSERT INTO nav_items (label, href, parent_id, display_order) VALUES
('EXPERIENCE', '/resume', (SELECT id FROM nav_items WHERE label = 'WORK' LIMIT 1), 10),
('PROJECTS', '/projects', (SELECT id FROM nav_items WHERE label = 'WORK' LIMIT 1), 20),
('REAL ESTATE', '/investments/real-estate', (SELECT id FROM nav_items WHERE label = 'INVESTMENTS' LIMIT 1), 10),
('BITCOIN', '/investments/bitcoin', (SELECT id FROM nav_items WHERE label = 'INVESTMENTS' LIMIT 1), 20);

INSERT INTO nav_items (label, href, display_order, is_visible) VALUES 
('SERVICES', '/services', 15, true),
('BOOK DISCOVERY', '/book', 25, true);

INSERT INTO footer_items (label, href, display_order) VALUES
('TERMS OF SERVICE', '/terms', 10),
('PRIVACY POLICY', '/privacy', 20),
('SITEMAP', '/sitemap', 30);

INSERT INTO landing_pages (slug, title, description, hero_title, hero_subtitle, lead_capture_title, lead_capture_desc, lead_capture_btn, options_json)
VALUES (
    'real-estate-ventures',
    'Real Estate Ventures.',
    'I am an active real estate investor and landlord always looking for the next deal or strategic partnership. Beyond acquiring properties, I leverage my network as a loan broker to structure investment capital.',
    'Real Estate<br/>Ventures.',
    'I am an active real estate investor and landlord always looking for the next deal or strategic partnership. Beyond acquiring properties, I leverage my network as a loan broker to structure investment capital.',
    'Let''s Connect',
    'Join the deal flow or request financing. Select your areas of interest below.',
    'SUBMIT INQUIRY',
    '{"buying": "Buying a Home", "selling": "Selling a Home", "loan": "Getting a real estate investment loan", "networking": "Connecting with other investors"}'
);

INSERT INTO resume_entries (category, title, subtitle, date_range, bullets, metadata) VALUES
-- Client Projects
('work', 'Salesforce Architect/Technical Lead', 'Enterprise Cloud Solutions Inc.', 'August 2024 – Present',
 '["Led two major ERP implementation projects, a commercial heating solution with Field Service Lightning integration and a Salesforce ERP integration with MailChimp, DocuSign, and a custom forecasting model.", "Managed all aspects of the projects, including developer oversight, system design, client relations, and overall project management, ensuring timely delivery and client satisfaction.", "Architected and oversaw complex data migration strategies for the commercial heating project, seamlessly transferring legacy data into the new Salesforce-based ERP system.", "Designed and implemented a custom integration framework to connect Salesforce with external systems (MailChimp, DocuSign) and a bespoke forecasting model, significantly enhancing the client''s operational efficiency and decision-making capabilities."]'::jsonb,
 '{"is_client_project": true}'::jsonb),

('work', 'Salesforce Architect/Engineer', 'Swan Bitcoin', 'April 2023 – August 2024',
 '["Redesigned the application architecture to bolster security measures and enhance data accuracy during data exchange.", "Managed, wrote, and maintained custom API integrations with external systems, leveraging Apex and Rust programming language for integration testing.", "Implemented Salesforce Shield to enhance data security and compliance with industry standards.", "Introduced version control system to manage source code efficiently, enabling easy collaboration and tracking changes.", "Established a streamlined release process for production, reducing deployment errors and enhancing delivery efficiency."]'::jsonb,
 '{"is_client_project": true}'::jsonb),

('work', 'Blockchain Developer', 'Zing.It', 'January 2023 – July 2023',
 '["Developed TypeScript and React unit tests to ensure the reliability and stability of UI components within the blockchain web application.", "Implemented an integration with SendGrid to facilitate the automatic sending of emails for key platform events, enhancing user engagement and communication effectiveness.", "Employed solidity smart contracts to enable secure and transparent transactional processes within the platform.", "Actively participated in code reviews, providing valuable feedback to peers and promoting code quality and best practices."]'::jsonb,
 '{"is_client_project": true}'::jsonb),

('work', 'Salesforce Consultant', 'Grayscale Investments', 'April 2022 - June 2022',
 '["Audited Grayscale''s Salesforce systems and codebase to identify areas for improvement and ensure compliance with best practices.", "Collaborated with the wealth management and advisory team to understand their sales process and develop solutions to enhance efficiency.", "Implemented lead integration from external systems to streamline data capture and management.", "Consolidated Grayscale''s data structure to facilitate seamless management of clients at different stages of the sales process.", "Provided training and guidance to Grayscale''s team on utilizing Salesforce effectively for their sales operations."]'::jsonb,
 '{"is_client_project": true}'::jsonb),

('work', 'Salesforce Architect and Senior Developer', 'Principle Studios', 'April 2022 – December 2022',
 '["Worked with client to build a Transportation Management System using Salesforce from scratch.", "Developed a Trigger Framework to support end to end Load management.", "Developed a security model that required information to be hidden from users working in different offices, managing, and enabling customer credit and the secure tracking and billing of shipper loads.", "Developed capabilities that support the Automatic and Manual Credit Approval, directly reducing the time it takes for agents to complete their work.", "Used SFDX for metadata deployment from environment to environment."]'::jsonb,
 '{"is_client_project": true}'::jsonb),

('work', 'Salesforce Lead Engineer / Implementation Architect', 'Mercury Healthcare', 'April 2020 – April 2022',
 '["Worked with Key Clients including Prisma Healthcare, Ascension Health and Advocate Aurora Health to implement multiple ISV products across the Sales Cloud, Service Cloud and Marketing Cloud.", "Implemented SSO for 50+ projects with Mercury Health Customers connecting Microsoft, Okta, Salesforce, and Appian Cloud.", "Developed Continuous Integration Process using SFDX for feature delivery.", "Configured Tableau and Snowflake for data visualizations used in Salesforce."]'::jsonb,
 '{"is_client_project": true}'::jsonb),

-- Employment
('work', 'Salesforce Technical Architect / Lead Software Engineer', 'Oplyst International, LLC', 'January 2012 - Present',
 '["Provide ongoing support & technical consultation for medium to large enterprises & non-profits", "Einstein Analytics and Discovery implementations with large datasets.", "Custom lightning development on Sales Cloud, Service Cloud, Health Cloud, and Nonprofit Starter Pack.", "Integrated salesforce instances with external services using REST, Platform Events & Heroku.", "Building Product APIs using Rust, Python and Deno Backends"]'::jsonb,
 '{"is_client_project": false}'::jsonb),

('work', 'CRM Technical Manager / Salesforce Architect', 'ZIPARI Inc', 'June 2018 - February 2019',
 '["Managing a 5 person team of Senior Engineers, in addition to being the Principal Salesforce Architect across all Salesforce Products", "Providing technical design, architecture & Api specs for new features", "Implemented numerous CI builds for deployment and product packaging using salesforce dx", "Successfully designed and built a call center application on Lightning Service Cloud", "Completed an internal audit and created a strategy to bring code coverage from 0% to 85% across all products in less than 2 months."]'::jsonb,
 '{"is_client_project": false}'::jsonb),

('work', 'Salesforce Architect / Lead Software Engineer', 'Evariant Inc', 'March 2017 - March 2018',
 '["Migrated 30 + companies to lightning to use the latest version of our application.", "Re-built Healthcare marketing lead list builder in using lightning components. List builder is used by 50 + health networks across the United States on a day to day basis.", "Built reusable lightning components used across Salesforce Classic and Lightning.", "Wrote custom REST Api to ingest healthcare case data."]'::jsonb,
 '{"is_client_project": false}'::jsonb),

-- Projects
('project', 'RS Business Directory', 'Full-Stack Web Application for Business Listings Management', '',
 '["Architected a high-performance business directory platform using Rust and Svelte, enabling efficient search and management of business listings for thousands of users.", "Developed RESTful APIs with Rust and the Axum framework, optimizing data handling and ensuring robust security for user and listing data, achieving sub-100ms response times under load.", "Designed a responsive admin interface with Svelte, Vite, and Tailwind CSS, integrating shadcn-svelte components to deliver an intuitive user experience.", "Containerized backend and frontend services using Docker, streamlining deployments across cloud environments and ensuring consistency with zero-downtime updates.", "Implemented a PostgreSQL-backed data layer with sqlx for scalable storage and retrieval, supporting rapid queries on large datasets and enabling future growth.", "Leveraged modern DevOps practices, including CI/CD pipelines with GitHub Actions and pnpm for frontend dependency management, reducing deployment cycles by 40%."]'::jsonb,
 '{"slug": "rs-business-directory", "tags": ["Rust", "Svelte", "Axum", "PostgreSQL", "Docker"], "status": "Completed"}'::jsonb),

('project', 'Basecamp', 'Smart contract deployment on the Starknet blockchain', '',
 '["Built a modular Next.js frontend with Web3.js integration, enabling seamless interaction with StarkNet smart contracts, improving user accessibility across diverse devices.", "Authored smart contracts and deployment scripts using snfoundry, optimizing gas efficiency on StarkNet and ensuring compliance with Ethereum security standards.", "Structured the project for scalability, separating frontend (/packages/nextjs) and blockchain components (/packages/snfoundry), facilitating maintenance and future feature additions.", "Integrated Yarn for dependency management and Vitest for unit testing, achieving 90%+ code coverage and ensuring reliability in production environments."]'::jsonb,
 '{"slug": "basecamp", "tags": ["Starknet", "Cairo", "Next.js", "Web3.js", "snfoundry"], "status": "Completed"}'::jsonb),

('project', 'Loan Landscape', 'Full-Stack Commercial Loan and Property Analysis Platform', '',
 '["Developed a scalable property analysis platform using Nuxt.js, TypeScript, and Rust, delivering real-time insights for property investment decisions.", "Implemented a high-throughput data processing pipeline in Rust, handling complex property calculations with sub-second latency, tested up to 10,000 daily queries.", "Created interactive data visualization components with Nuxt.js and D3.js, enabling users to explore property metrics intuitively.", "Designed a responsive frontend with Vue.js-based Nuxt.js, ensuring cross-platform compatibility and accessibility, validated through user testing.", "Architected a modular system separating frontend, backend, and data processing layers, enhancing maintainability and enabling independent scaling of services."]'::jsonb,
 '{"slug": "loan-landscape", "tags": ["Rust", "Nuxt.js", "Vue.js", "D3.js", "TypeScript"], "status": "Completed"}'::jsonb),

-- Certifications
('certification', 'Salesforce Certified Administrator', '', '', '[]'::jsonb, '{"is_training": false}'::jsonb),
('certification', 'Salesforce Certified Platform Developer I', '', '', '[]'::jsonb, '{"is_training": false}'::jsonb),
('certification', 'Salesforce Certified Sharing and Visibility Architect', '', '', '[]'::jsonb, '{"is_training": false}'::jsonb),
('certification', 'Salesforce Certified Agentforce Specialist', '', '', '[]'::jsonb, '{"is_training": false}'::jsonb),
('certification', 'Salesforce Certified Service Cloud Consultant', '', '', '[]'::jsonb, '{"is_training": false}'::jsonb),
('certification', 'Salesforce Certified AI Associate', '', '', '[]'::jsonb, '{"is_training": false}'::jsonb),
('certification', 'Salesforce Certified Platform App Builder', '', '', '[]'::jsonb, '{"is_training": false}'::jsonb),
('certification', 'Einstein Discover Training at Salesforce Office in Herndon, VA', '', '', '[]'::jsonb, '{"is_training": true}'::jsonb),
('certification', 'Einstein Analytics Advanced Training at Salesforce Office in Herndon, VA', '', '', '[]'::jsonb, '{"is_training": true}'::jsonb);

INSERT INTO resume_profiles (name, full_name, objective, is_public, target_role, category_visibility, category_order) VALUES
('Default', 'Ruud Salym Erie', 'A comprehensive log of active systems architecture, smart contract deployments, and client configurations.', true, 'Systems Architect', '{"work":true,"education":true,"certification":true,"project":true,"skill":true,"language":true}'::jsonb, '["work", "education", "certification", "project", "skill", "volunteer", "extracurricular", "language", "hobby"]'::jsonb);

INSERT INTO resume_profile_entries (profile_id, entry_id)
SELECT (SELECT id FROM resume_profiles LIMIT 1), id FROM resume_entries;

INSERT INTO page_headers (route_path, badge_text, title, subtitle) VALUES
('/projects', 'CLIENT AND PERSONAL REPOSITORIES', 'TECHNICAL PORTFOLIO', 'Engineering resilient infrastructures across blockchains, decentralized cloud, and sub-millisecond Rust backends.'),
('/certifications', 'INDEX_REF_07 // VERIFIED INFRA', 'CERTIFICATIONS', 'Cryptographically and institutionally verified architecture authorizations.'),
('/blog', 'ENGINEERING DISSERTATIONS', 'TECHNICAL WRITING', 'Documentation on distributed systems architecture, Bitcoin cryptography, Salesforce APEX algorithms, and low-latency infrastructure design.'),
('/resume', 'ARCHITECTURAL OVERVIEW', 'RESUME & CURRICULUM VITAE', 'Technical proficiencies and professional timeline documented for vendor qualification.'),
('/services', 'RUUDERIE.AI // ADVISORY SERVICES', 'ARCHITECTURE & STRATEGY', 'Elite engineering consultation and systems design for the digital frontier.'),
('/book', 'DISCOVERY // B2B', 'CONSULTATION', 'Schedule a preliminary systems architecture review.')
ON CONFLICT (route_path) DO UPDATE SET
    badge_text = EXCLUDED.badge_text,
    title = EXCLUDED.title,
    subtitle = EXCLUDED.subtitle;

INSERT INTO services (title, description, deliverables, price_range, is_visible, display_order) VALUES
('Strategic Architecture & Process Engineering', 'Reimplementing critical business processes using long-term thinking, elegant architectural design, and scalable infrastructure patterns. We dive deep into your Salesforce and distributed environments to establish sustainable technical foundations.', '["Scalable Business Process Re-engineering", "System Landscape Diagrams", "Technical Debt Refactoring Roadmaps"]'::jsonb, 'Custom per project', true, 1),
('Salesforce Security & Threat Modeling', 'Hardening enterprise perimeters with specialized focus on Salesforce native security. Protect critical data flows and prevent automated exploitation through comprehensive system audits and strict access control regimes.', '["Salesforce Shield Implementations", "Identity & Access Management (IAM)", "System and User Access Reviews"]'::jsonb, 'Retainer available', true, 2),
('Rust Systems Engineering', 'Purpose-built, memory-safe microservices designed for maximum throughput and minimal operational cost. We port your slowest, most expensive legacy endpoints into native Rust binaries to accelerate your architecture.', '["API Contract Definitions", "WebAssembly (Wasm) Porting", "Zero-Cost Abstraction Implementations"]'::jsonb, 'Starting at $5,000', true, 3);

UPDATE services 
SET 
    title = 'High-Performance Rust, Data, & AI Systems',
    description = 'Purpose-built, memory-safe microservices designed for maximum throughput and predictable P99 latency. Leveraging my deep experience building critical infrastructure, I use Rust''s strict compiler guarantees to forge the performance engines behind modern AI architectures and high-volume data engineering pipelines. I specialize in replacing unscalable legacy endpoints with native Rust data streams and WebAssembly (Wasm) runtimes, giving your platform an unmatched competitive edge in speed, security, and operational cost savings.',
    deliverables = '["High-Volume Data Engineering Pipelines", "AI Inference Engine Optimization", "API and backend system development using Rust"]'::jsonb
WHERE id = 3;

INSERT INTO services (title, description, deliverables, price_range, is_visible, display_order) VALUES
('Autonomous AI Agents & Data Orchestration', 'Build end-to-end intelligent agent ecosystems that reason, act, and interface natively with your core business platforms. Drawing on my background architecting complex enterprise data flows, I integrate structured and unstructured data streams—from advanced document OCR extraction to continuous CRM ingestion—to power your autonomous AI deployments. I equip your systems with cutting-edge reasoning tools that operate safely and securely across your unified data graph.', '["Multi-Agent Workflow Automation", "Unified Data Strategies", "Complex Salesforce Integrations"]'::jsonb, 'Engagement based', true, 4),
('Cloud-Native Infrastructure & DevOps', 'Accelerate your delivery capabilities with robust, scalable deployment architectures. Having engineered operations across highly distributed environments, I design and implement strictly governed Kubernetes configurations, sophisticated continuous integration pipelines, and immutable infrastructure protocols. I ensure your platform maintains high availability, scales effortlessly under load, and achieves rock-solid deployment reliability.', '["Kubernetes Cluster Architecture", "Zero-Downtime CI/CD Pipelines", "Infrastructure as Code (IaC) Audits"]'::jsonb, 'Tiered Retainers', true, 5);

UPDATE services 
SET 
    price_range = 'Starting at $25,000',
    description = 'Purpose-built, memory-safe microservices designed for maximum throughput and predictable P99 latency. Leveraging my deep experience building critical infrastructure, I use Rust''s strict compiler guarantees to forge the performance engines behind modern AI architectures and high-volume data engineering pipelines. I specialize in replacing unscalable legacy endpoints with native Rust data streams, giving your platform an unmatched competitive edge in speed, security, and operational cost savings.'
WHERE id = 3;

UPDATE site_settings SET value = 'ANCHOR' WHERE key = 'site_title';
UPDATE page_headers SET badge_text = 'ANCHOR // ADVISORY SERVICES' WHERE route_path = '/services';

UPDATE site_settings SET value = 'https://x.com/ruud_awakening' WHERE key = 'x_url';
UPDATE site_settings SET value = 'https://linkedin.com/in/ruudsalymerie' WHERE key = 'linkedin_url';

