//! Product-scoped marketing pricing plans.

use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .get_connection()
            .execute_unprepared(
                r#"
CREATE TABLE IF NOT EXISTS platform_product_plans (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  product_id UUID NOT NULL REFERENCES platform_products(id) ON DELETE CASCADE,
  slug TEXT NOT NULL,
  name TEXT NOT NULL,
  tagline TEXT NOT NULL DEFAULT '',
  price_cents INTEGER NOT NULL DEFAULT 0,
  currency TEXT NOT NULL DEFAULT 'USD',
  billing_interval TEXT NOT NULL DEFAULT 'month',
  features JSONB NOT NULL DEFAULT '[]'::jsonb,
  cta_label TEXT NOT NULL DEFAULT 'Get started',
  cta_href TEXT,
  is_featured BOOLEAN NOT NULL DEFAULT false,
  sort_order INTEGER NOT NULL DEFAULT 0,
  is_active BOOLEAN NOT NULL DEFAULT true,
  billing_plan_id UUID NULL REFERENCES billing_plans(id) ON DELETE SET NULL,
  created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  UNIQUE (product_id, slug)
);

CREATE INDEX IF NOT EXISTS idx_product_plans_product
  ON platform_product_plans(product_id) WHERE is_active;

INSERT INTO platform_product_plans
  (product_id, slug, name, tagline, price_cents, currency, billing_interval, features, cta_label, cta_href, is_featured, sort_order)
SELECT p.id, v.slug, v.name, v.tagline, v.price_cents, v.currency, v.billing_interval, v.features::jsonb, v.cta_label, v.cta_href, v.is_featured, v.sort_order
FROM platform_products p
JOIN (
  VALUES
  ('folio', 'free', 'Free', 'Up to 2 units · free forever', 0, 'USD', 'forever',
   '["Landlord dashboard","Lease management","Tenant portal","Maintenance requests"]', 'Join waitlist', '#waitlist-wrap', false, 0),
  ('folio', 'grow', 'Grow', 'Up to 10 units', 2900, 'USD', 'month',
   '["Everything in Free","Rent collection (ACH + card)","Vacancy marketing","Contractor marketplace","Basic analytics"]', 'Join waitlist', '#waitlist-wrap', false, 1),
  ('folio', 'pro', 'Pro', 'Up to 30 units', 7900, 'USD', 'month',
   '["Everything in Grow","Vacation rental calendar & channels","STR compliance & permits","Portfolio analytics","Multi-country (US, Canada, Brazil)","Priority support"]', 'Join waitlist', '#waitlist-wrap', true, 2),
  ('folio', 'investor', 'Investor', 'Unlimited units', 14900, 'USD', 'month',
   '["Everything in Pro","Cohost Network access","Co-host revenue share tracking","Dedicated onboarding","API access"]', 'Join waitlist', '#waitlist-wrap', false, 3),

  ('folio-broker', 'solo', 'Solo', '1 agent seat', 9900, 'USD', 'month',
   '["Active listing management","Buyer & seller CRM","Commission tracking","Transaction timelines"]', 'Join waitlist', '/#waitlist-wrap', false, 0),
  ('folio-broker', 'team', 'Team', 'Up to 5 agent seats', 24900, 'USD', 'month',
   '["Everything in Solo","Agent account management","Agent profiles & bios","Commission tracking","Team analytics dashboard"]', 'Get early access', '/#waitlist-wrap', true, 1),
  ('folio-broker', 'firm', 'Firm', 'Up to 25 agent seats', 49900, 'USD', 'month',
   '["Everything in Team","Branded listing portal","Client management hub","Brokerage analytics","Priority support"]', 'Get early access', '/#waitlist-wrap', false, 2),
  ('folio-broker', 'enterprise', 'Custom', '25+ seats · white-label · SLA', 0, 'USD', 'custom',
   '["Everything in Firm","White-label branding","Dedicated onboarding","API access & SSO","Uptime SLA"]', 'Contact us', '/#waitlist-wrap', false, 3),

  ('folio-pm', 'starter-pm', 'Starter PM', '1 client portfolio · up to 20 units', 9900, 'USD', 'month',
   '["Full landlord platform","1 branded owner portal","Trust accounting ledger","Maintenance dispatch","Requires 2+ owner-clients"]', 'Join waitlist', '#pm-waitlist', false, 0),
  ('folio-pm', 'growth-pm', 'Growth PM', 'Up to 5 client portfolios · 100 units', 19900, 'USD', 'month',
   '["Everything in Starter PM","5 branded owner portals","Auto-disbursement & fee split","Portfolio analytics","Vacancy marketing"]', 'Get early access', '#pm-waitlist', true, 1),
  ('folio-pm', 'scale-pm', 'Scale PM', 'Up to 15 client portfolios · 300 units', 39900, 'USD', 'month',
   '["Everything in Growth PM","Full trust accounting suite","Multi-user team access","Priority support","Advanced reporting"]', 'Get early access', '#pm-waitlist', false, 2),
  ('folio-pm', 'enterprise-pm', 'Enterprise', 'Unlimited portfolios · white-label · API', 0, 'USD', 'custom',
   '["Everything in Scale PM","White-label branding","API access & SSO","Dedicated onboarding","Uptime SLA"]', 'Contact us', '#pm-waitlist', false, 3),

  ('folio-vendor', 'basic', 'Basic', 'Free forever', 0, 'USD', 'forever',
   '["Marketplace profile","Accept & complete jobs","In-platform invoicing","ACH payment in 24h"]', 'Join free', '#vendor-signup', false, 0),
  ('folio-vendor', 'pro-vendor', 'Pro Vendor', 'Priority placement', 2900, 'USD', 'month',
   '["Everything in Basic","Priority search placement","Auto-invoicing templates","Job analytics dashboard","Verified badge"]', 'Get early access', '#vendor-signup', true, 1),
  ('folio-vendor', 'business', 'Business', '0% platform fee', 7900, 'USD', 'month',
   '["Everything in Pro Vendor","0% platform fee on jobs","Multi-tech accounts","Branded company profile","Dedicated account manager"]', 'Get early access', '#vendor-signup', false, 2)
) AS v(product_slug, slug, name, tagline, price_cents, currency, billing_interval, features, cta_label, cta_href, is_featured, sort_order)
  ON p.slug = v.product_slug
WHERE NOT EXISTS (
  SELECT 1
  FROM platform_product_plans existing
  WHERE existing.product_id = p.id
    AND existing.slug = v.slug
);
"#,
            )
            .await?;
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .get_connection()
            .execute_unprepared(
                r#"
DROP TABLE IF EXISTS platform_product_plans;
"#,
            )
            .await?;
        Ok(())
    }
}
