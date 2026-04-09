DO $$
DECLARE
    v_bwr_tenant_id UUID;
    v_bwr_app_id UUID;
BEGIN
    SELECT id INTO v_bwr_tenant_id FROM tenant WHERE name = 'buildwithruud' LIMIT 1;
    
    IF v_bwr_tenant_id IS NULL THEN
        RAISE EXCEPTION 'Tenant buildwithruud not found. Please ensure the backend migrations have run successfully first.';
    END IF;

    SELECT id INTO v_bwr_app_id FROM app_instances WHERE tenant_id = v_bwr_tenant_id AND app_type = 'anchor' LIMIT 1;

    -- Update Site Settings via app_instances.settings payload directly
    UPDATE app_instances
    SET settings = '{
        "current_focus": "Technical Architect @ Hipten",
        "status": "Unavailable For New Projects",
        "hero_quote": "Vires in Numeris. Systems architecture is not defined by lines, but by cryptographic proofs and immutable data flows.",
        "hero_subtitle": "TECHNICAL ARCHITECT AND BACKEND ENGINEER // SPECIALIZING IN ENTERPRISE CLOUD SOLUTIONS, SALESFORCE, AND RUST APPLICATIONS.",
        "site_title": "BuildWithRuud",
        "lc_title": "Join My Newsletter",
        "lc_desc": "Get insights on AI, Salesforce, Rust, and Real Estate.",
        "lc_label": "Email Address",
        "lc_placeholder": "user@organization.domain",
        "lc_btn": "Subscribe",
        "lc_footer": "* Check your email to confirm your subscription.",
        "lc_endpoint": "/api/DownloadResume",
        "status_color": "#ff5449",
        "webhook_url": "",
        "admin_email": "ruud@oply.co",
        "booking_url": "https://cal.com/ruuderie/15min",
        "terms_html": "# Terms of Service\n\nPlease review our terms.",
        "privacy_html": "# Privacy Policy\n\nWe respect your digital privacy.",
        "github_url": "https://github.com/ruuderie",
        "x_url": "https://x.com/ruud_awakening",
        "linkedin_url": "https://linkedin.com/in/ruudsalymerie",
        "meta_title": "Ruud Salym Erie - Technical Architect",
        "meta_description": "Technical Architect and Software Engineer specializing in Rust, Salesforce, and high-performance enterprise applications.",
        "og_image": ""
    }'::jsonb
    WHERE id = v_bwr_app_id;

    -- Update global page headers
    INSERT INTO page_headers (route_path, badge_text, title, subtitle) VALUES
    ('/projects', 'CLIENT AND PERSONAL REPOSITORIES', 'TECHNICAL PORTFOLIO', 'Engineering resilient infrastructures across blockchains, decentralized cloud, and sub-millisecond Rust backends.'),
    ('/certifications', 'INDEX_REF_07 // VERIFIED INFRA', 'CERTIFICATIONS', 'Cryptographically and institutionally verified architecture authorizations.'),
    ('/blog', 'ENGINEERING DISSERTATIONS', 'TECHNICAL WRITING', 'Documentation on distributed systems architecture, Bitcoin cryptography, Salesforce APEX algorithms, and low-latency infrastructure design.'),
    ('/resume', 'ARCHITECTURAL OVERVIEW', 'RESUME & CURRICULUM VITAE', 'Technical proficiencies and professional timeline documented for vendor qualification.'),
    ('/services', 'BUILDWITHRUUD // ADVISORY SERVICES', 'ARCHITECTURE & STRATEGY', 'Elite engineering consultation and systems design for the digital frontier.'),
    ('/book', 'DISCOVERY // B2B', 'CONSULTATION', 'Schedule a preliminary systems architecture review.')
    ON CONFLICT (route_path) DO UPDATE SET
        badge_text = EXCLUDED.badge_text,
        title = EXCLUDED.title,
        subtitle = EXCLUDED.subtitle;

    -- Clear old app content to prevent duplicates if running multiple times
    DELETE FROM app_content WHERE tenant_id = v_bwr_tenant_id AND collection_type IN ('service', 'highlight');

    -- Insert Services
    INSERT INTO app_content (tenant_id, collection_type, title, payload, display_order) VALUES
    (v_bwr_tenant_id, 'service', 'Strategic Architecture & Process Engineering', '{"description": "Reimplementing critical business processes using long-term thinking, elegant architectural design, and scalable infrastructure patterns. We dive deep into your Salesforce and distributed environments to establish sustainable technical foundations.", "deliverables": ["Scalable Business Process Re-engineering", "System Landscape Diagrams", "Technical Debt Refactoring Roadmaps"], "price_range": "Custom per project"}'::jsonb, 1),
    (v_bwr_tenant_id, 'service', 'Salesforce Security & Threat Modeling', '{"description": "Hardening enterprise perimeters with specialized focus on Salesforce native security. Protect critical data flows and prevent automated exploitation through comprehensive system audits and strict access control regimes.", "deliverables": ["Salesforce Shield Implementations", "Identity & Access Management (IAM)", "System and User Access Reviews"], "price_range": "Retainer available"}'::jsonb, 2),
    (v_bwr_tenant_id, 'service', 'High-Performance Rust, Data, & AI Systems', '{"description": "Purpose-built, memory-safe microservices designed for maximum throughput and predictable P99 latency. Leveraging my deep experience building critical infrastructure, I use Rust''s strict compiler guarantees to forge the performance engines behind modern AI architectures and high-volume data engineering pipelines. I specialize in replacing unscalable legacy endpoints with native Rust data streams, giving your platform an unmatched competitive edge in speed, security, and operational cost savings.", "deliverables": ["High-Volume Data Engineering Pipelines", "AI Inference Engine Optimization", "API and backend system development using Rust"], "price_range": "Starting at $25,000"}'::jsonb, 3),
    (v_bwr_tenant_id, 'service', 'Autonomous AI Agents & Data Orchestration', '{"description": "Build end-to-end intelligent agent ecosystems that reason, act, and interface natively with your core business platforms. Drawing on my background architecting complex enterprise data flows, I integrate structured and unstructured data streams—from advanced document OCR extraction to continuous CRM ingestion—to power your autonomous AI deployments. I equip your systems with cutting-edge reasoning tools that operate safely and securely across your unified data graph.", "deliverables": ["Multi-Agent Workflow Automation", "Unified Data Strategies", "Complex Salesforce Integrations"], "price_range": "Engagement based"}'::jsonb, 4),
    (v_bwr_tenant_id, 'service', 'Cloud-Native Infrastructure & DevOps', '{"description": "Accelerate your delivery capabilities with robust, scalable deployment architectures. Having engineered operations across highly distributed environments, I design and implement strictly governed Kubernetes configurations, sophisticated continuous integration pipelines, and immutable infrastructure protocols. I ensure your platform maintains high availability, scales effortlessly under load, and achieves rock-solid deployment reliability.", "deliverables": ["Kubernetes Cluster Architecture", "Zero-Downtime CI/CD Pipelines", "Infrastructure as Code (IaC) Audits"], "price_range": "Tiered Retainers"}'::jsonb, 5);

    -- Insert Highlights
    INSERT INTO app_content (tenant_id, collection_type, title, payload, display_order) VALUES
    (v_bwr_tenant_id, 'highlight', 'Enterprise Salesforce Reimagined', '{"description": "Architected custom ERP endpoints in Salesforce to migrate legacy frameworks completely, optimizing batch processing out of synchronous limits.", "technologies": ["Apex", "LWC", "Rust", "Integration"], "link": "/blog/enterprise-salesforce", "image_url": "https://images.unsplash.com/photo-1551288049-bebda4e38f71?auto=format&fit=crop&q=80"}'::jsonb, 1),
    (v_bwr_tenant_id, 'highlight', 'High Speed Rust Relays', '{"description": "Built resilient, zero-cost abstraction WebAssembly pipelines routing critical state without standard node.js sluggishness.", "technologies": ["Rust", "WASM", "Tokio", "SeaORM"], "link": "/blog/rust-relays", "image_url": "https://images.unsplash.com/photo-1526374965328-7f61d4dc18c5?auto=format&fit=crop&q=80"}'::jsonb, 2),
    (v_bwr_tenant_id, 'highlight', 'Smart Contract Architecture', '{"description": "Led optimization vectors on the Starknet network, reducing overall interaction footprints on rollups by 30%.", "technologies": ["Cairo", "Solidity", "Starknet"], "link": "/blog/smart-contracts", "image_url": "https://images.unsplash.com/photo-1621504450181-5d356f61d307?auto=format&fit=crop&q=80"}'::jsonb, 3);

    -- Enable BitcoinSync background job if it doesn't exist
    IF NOT EXISTS (SELECT 1 FROM tenant_background_jobs WHERE tenant_id = v_bwr_tenant_id AND job_type = 'BitcoinSync') THEN
        INSERT INTO tenant_background_jobs (id, tenant_id, job_type, config, interval_seconds, last_run, is_active) 
        VALUES (gen_random_uuid(), v_bwr_tenant_id, 'BitcoinSync', '{"api_url": "https://mempool.space/api/v1/blocks"}'::jsonb, 600, NULL, true);
    END IF;

    -- Clear old menus to prevent duplication
    DELETE FROM app_menus WHERE tenant_id = v_bwr_tenant_id;

    -- Insert menus
    DECLARE
        v_parent_work UUID := gen_random_uuid();
        v_parent_investments UUID := gen_random_uuid();
    BEGIN
        INSERT INTO app_menus (id, tenant_id, menu_type, label, href, display_order, is_visible) VALUES 
        (v_parent_work, v_bwr_tenant_id, 'header', 'WORK', '#', 10, true),
        (v_parent_investments, v_bwr_tenant_id, 'header', 'INVESTMENTS', '#', 40, true),
        (gen_random_uuid(), v_bwr_tenant_id, 'header', 'BLOG', '/blog', 30, true),
        (gen_random_uuid(), v_bwr_tenant_id, 'header', 'SERVICES', '/services', 15, true),
        (gen_random_uuid(), v_bwr_tenant_id, 'header', 'BOOK DISCOVERY', '/book', 25, true);

        INSERT INTO app_menus (id, tenant_id, menu_type, label, href, parent_id, display_order, is_visible) VALUES
        (gen_random_uuid(), v_bwr_tenant_id, 'header', 'EXPERIENCE', '/resume', v_parent_work, 10, true),
        (gen_random_uuid(), v_bwr_tenant_id, 'header', 'PROJECTS', '/projects', v_parent_work, 20, true),
        (gen_random_uuid(), v_bwr_tenant_id, 'header', 'REAL ESTATE', '/investments/real-estate', v_parent_investments, 10, true),
        (gen_random_uuid(), v_bwr_tenant_id, 'header', 'BITCOIN', '/investments/bitcoin', v_parent_investments, 20, true);

        -- Insert footer menus
        INSERT INTO app_menus (id, tenant_id, menu_type, label, href, display_order, is_visible) VALUES
        (gen_random_uuid(), v_bwr_tenant_id, 'footer', 'TERMS OF SERVICE', '/terms', 10, true),
        (gen_random_uuid(), v_bwr_tenant_id, 'footer', 'PRIVACY POLICY', '/privacy', 20, true),
        (gen_random_uuid(), v_bwr_tenant_id, 'footer', 'SITEMAP', '/sitemap', 30, true);
    END;

    -- Replace landing_pages map into app_pages 
    DELETE FROM app_pages WHERE tenant_id = v_bwr_tenant_id AND slug IN ('real-estate-ventures');

    INSERT INTO app_pages (id, tenant_id, slug, title, description, page_type, hero_payload, blocks_payload, is_published)
    VALUES (
        gen_random_uuid(),
        v_bwr_tenant_id,
        'real-estate-ventures',
        'Real Estate Ventures.',
        'I am an active real estate investor and landlord always looking for the next deal or strategic partnership. Beyond acquiring properties, I leverage my network as a loan broker to structure investment capital.',
        'landing',
        '{"hero_title": "Real Estate<br/>Ventures.", "hero_subtitle": "I am an active real estate investor and landlord always looking for the next deal or strategic partnership. Beyond acquiring properties, I leverage my network as a loan broker to structure investment capital."}'::jsonb,
        '{"lead_capture_title": "Let''s Connect", "lead_capture_desc": "Join the deal flow or request financing. Select your areas of interest below.", "lead_capture_btn": "SUBMIT INQUIRY", "options_json": "{\"buying\": \"Buying a Home\", \"selling\": \"Selling a Home\", \"loan\": \"Getting a real estate investment loan\", \"networking\": \"Connecting with other investors\"}"}'::jsonb,
        true
    );

END $$;
