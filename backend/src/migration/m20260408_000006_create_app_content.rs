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
                CREATE TABLE IF NOT EXISTS app_content (
                    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
                    tenant_id UUID NOT NULL REFERENCES tenant(id) ON DELETE CASCADE,
                    collection_type VARCHAR(255) NOT NULL,
                    title VARCHAR(500) NOT NULL,
                    payload JSONB NOT NULL DEFAULT '{}'::jsonb,
                    status VARCHAR(50) NOT NULL DEFAULT 'published',
                    display_order INTEGER NOT NULL DEFAULT 0,
                    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
                    updated_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP
                );

                CREATE INDEX IF NOT EXISTS idx_app_content_tenant_collection ON app_content(tenant_id, collection_type);
                CREATE INDEX IF NOT EXISTS idx_app_content_status ON app_content(status);

                SELECT id INTO v_bwr_tenant_id FROM tenant WHERE name ILIKE '%buildwithruud%' LIMIT 1;
                
                IF v_bwr_tenant_id IS NOT NULL THEN
                    IF NOT EXISTS (SELECT 1 FROM app_content WHERE tenant_id = v_bwr_tenant_id AND collection_type = 'service' LIMIT 1) THEN
                        INSERT INTO app_content (tenant_id, collection_type, title, payload, display_order) VALUES
                        (v_bwr_tenant_id, 'service', 'Strategic Architecture & Process Engineering', '{"description": "Reimplementing critical business processes using long-term thinking, elegant architectural design, and scalable infrastructure patterns. We dive deep into your Salesforce and distributed environments to establish sustainable technical foundations.", "deliverables": ["Scalable Business Process Re-engineering", "System Landscape Diagrams", "Technical Debt Refactoring Roadmaps"], "price_range": "Custom per project"}'::jsonb, 1),
                        (v_bwr_tenant_id, 'service', 'Salesforce Security & Threat Modeling', '{"description": "Hardening enterprise perimeters with specialized focus on Salesforce native security. Protect critical data flows and prevent automated exploitation through comprehensive system audits and strict access control regimes.", "deliverables": ["Salesforce Shield Implementations", "Identity & Access Management (IAM)", "System and User Access Reviews"], "price_range": "Retainer available"}'::jsonb, 2),
                        (v_bwr_tenant_id, 'service', 'High-Performance Rust, Data, & AI Systems', '{"description": "Purpose-built, memory-safe microservices designed for maximum throughput and predictable P99 latency. Leveraging my deep experience building critical infrastructure, I use Rust''s strict compiler guarantees to forge the performance engines behind modern AI architectures and high-volume data engineering pipelines. I specialize in replacing unscalable legacy endpoints with native Rust data streams, giving your platform an unmatched competitive edge in speed, security, and operational cost savings.", "deliverables": ["High-Volume Data Engineering Pipelines", "AI Inference Engine Optimization", "API and backend system development using Rust"], "price_range": "Starting at $25,000"}'::jsonb, 3),
                        (v_bwr_tenant_id, 'service', 'Autonomous AI Agents & Data Orchestration', '{"description": "Build end-to-end intelligent agent ecosystems that reason, act, and interface natively with your core business platforms. Drawing on my background architecting complex enterprise data flows, I integrate structured and unstructured data streams—from advanced document OCR extraction to continuous CRM ingestion—to power your autonomous AI deployments. I equip your systems with cutting-edge reasoning tools that operate safely and securely across your unified data graph.", "deliverables": ["Multi-Agent Workflow Automation", "Unified Data Strategies", "Complex Salesforce Integrations"], "price_range": "Engagement based"}'::jsonb, 4),
                        (v_bwr_tenant_id, 'service', 'Cloud-Native Infrastructure & DevOps', '{"description": "Accelerate your delivery capabilities with robust, scalable deployment architectures. Having engineered operations across highly distributed environments, I design and implement strictly governed Kubernetes configurations, sophisticated continuous integration pipelines, and immutable infrastructure protocols. I ensure your platform maintains high availability, scales effortlessly under load, and achieves rock-solid deployment reliability.", "deliverables": ["Kubernetes Cluster Architecture", "Zero-Downtime CI/CD Pipelines", "Infrastructure as Code (IaC) Audits"], "price_range": "Tiered Retainers"}'::jsonb, 5);
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
            DROP TABLE IF EXISTS app_content;
        "#;
        db.execute_unprepared(sql).await?;
        Ok(())
    }
}
