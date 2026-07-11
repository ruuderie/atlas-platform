//! G-36 `atlas_programs` — productized growth/incentive programs.
//!
//! See `docs/architecture/g36_atlas_programs_spec.md`.

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
-- ═══════════════════════════════════════════════════════════════════════════
-- G-36: atlas_programs
-- ═══════════════════════════════════════════════════════════════════════════

CREATE TABLE IF NOT EXISTS atlas_programs (
    id                      UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id               UUID REFERENCES account(id) ON DELETE CASCADE,
    slug                    TEXT NOT NULL,
    name                    TEXT NOT NULL,
    description             TEXT,
    program_kind            TEXT NOT NULL
        CHECK (program_kind IN (
            'network_invite', 'referral', 'review_request',
            'waitlist_access', 'lead_capture', 'partner_share'
        )),
    campaign_id             UUID,
    actor_roles             JSONB NOT NULL DEFAULT '[]'::jsonb,
    target_roles            JSONB NOT NULL DEFAULT '[]'::jsonb,
    config                  JSONB NOT NULL DEFAULT '{}'::jsonb,
    default_outcome_type    TEXT NOT NULL DEFAULT 'signup'
        CHECK (default_outcome_type IN (
            'signup', 'wizard_complete', 'form_submit',
            'review_submitted', 'first_job_logged', 'subscription_activated'
        )),
    is_active               BOOLEAN NOT NULL DEFAULT true,
    created_at              TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at              TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE UNIQUE INDEX IF NOT EXISTS uq_atlas_programs_system_slug
    ON atlas_programs (slug) WHERE tenant_id IS NULL;
CREATE UNIQUE INDEX IF NOT EXISTS uq_atlas_programs_tenant_slug
    ON atlas_programs (tenant_id, slug) WHERE tenant_id IS NOT NULL;
CREATE INDEX IF NOT EXISTS idx_atlas_programs_kind
    ON atlas_programs (program_kind) WHERE is_active = true;

CREATE TABLE IF NOT EXISTS atlas_program_actions (
    id                          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    program_id                  UUID NOT NULL REFERENCES atlas_programs(id) ON DELETE CASCADE,
    campaign_enrollment_id      UUID,
    actor_user_id               UUID NOT NULL REFERENCES "user"(id) ON DELETE CASCADE,
    tenant_id                   UUID REFERENCES account(id) ON DELETE SET NULL,
    target_email                TEXT,
    target_user_id              UUID REFERENCES "user"(id) ON DELETE SET NULL,
    target_role                 TEXT,
    delivery_entity_type        TEXT,
    delivery_entity_id          UUID,
    status                      TEXT NOT NULL DEFAULT 'created'
        CHECK (status IN (
            'created', 'sent', 'opened', 'accepted',
            'outcome_complete', 'expired', 'revoked'
        )),
    metadata                    JSONB NOT NULL DEFAULT '{}'::jsonb,
    created_at                  TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at                  TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS idx_atlas_program_actions_actor
    ON atlas_program_actions (actor_user_id, created_at DESC);
CREATE INDEX IF NOT EXISTS idx_atlas_program_actions_program
    ON atlas_program_actions (program_id, status);
CREATE INDEX IF NOT EXISTS idx_atlas_program_actions_delivery
    ON atlas_program_actions (delivery_entity_type, delivery_entity_id)
    WHERE delivery_entity_id IS NOT NULL;

CREATE TABLE IF NOT EXISTS atlas_program_outcomes (
    id                      UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    program_action_id       UUID NOT NULL REFERENCES atlas_program_actions(id) ON DELETE CASCADE,
    outcome_type            TEXT NOT NULL
        CHECK (outcome_type IN (
            'signup', 'wizard_complete', 'form_submit',
            'review_submitted', 'first_job_logged', 'subscription_activated'
        )),
    status                  TEXT NOT NULL DEFAULT 'pending'
        CHECK (status IN ('pending', 'completed', 'failed')),
    completed_at            TIMESTAMPTZ,
    evidence_entity_type    TEXT,
    evidence_entity_id      UUID,
    created_at              TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS idx_atlas_program_outcomes_action
    ON atlas_program_outcomes (program_action_id);

CREATE TABLE IF NOT EXISTS atlas_program_reward_rules (
    id                      UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    program_id              UUID NOT NULL REFERENCES atlas_programs(id) ON DELETE CASCADE,
    beneficiary             TEXT NOT NULL
        CHECK (beneficiary IN ('actor', 'target')),
    reward_type             TEXT NOT NULL
        CHECK (reward_type IN ('subscription_credit_days', 'feature_unlock', 'none')),
    amount                  NUMERIC(12, 2) NOT NULL DEFAULT 0,
    trigger_outcome_type    TEXT NOT NULL
        CHECK (trigger_outcome_type IN (
            'signup', 'wizard_complete', 'form_submit',
            'review_submitted', 'first_job_logged', 'subscription_activated'
        )),
    is_active               BOOLEAN NOT NULL DEFAULT true,
    created_at              TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS idx_atlas_program_reward_rules_program
    ON atlas_program_reward_rules (program_id) WHERE is_active = true;

CREATE TABLE IF NOT EXISTS atlas_program_reward_grants (
    id                      UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    program_action_id       UUID NOT NULL REFERENCES atlas_program_actions(id) ON DELETE CASCADE,
    rule_id                 UUID NOT NULL REFERENCES atlas_program_reward_rules(id) ON DELETE CASCADE,
    beneficiary_user_id     UUID NOT NULL REFERENCES "user"(id) ON DELETE CASCADE,
    status                  TEXT NOT NULL DEFAULT 'pending'
        CHECK (status IN ('pending', 'granted', 'revoked')),
    granted_at              TIMESTAMPTZ,
    created_at              TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS idx_atlas_program_reward_grants_action
    ON atlas_program_reward_grants (program_action_id);
CREATE INDEX IF NOT EXISTS idx_atlas_program_reward_grants_beneficiary
    ON atlas_program_reward_grants (beneficiary_user_id, status);

-- ── NetworkInvite system seeds ───────────────────────────────────────────────
INSERT INTO atlas_programs (id, tenant_id, slug, name, description, program_kind, actor_roles, target_roles, default_outcome_type)
SELECT gen_random_uuid(), NULL, v.slug, v.name, v.description, 'network_invite',
       v.actor_roles::jsonb, v.target_roles::jsonb, v.default_outcome_type
FROM (VALUES
  ('landlord_invite_peers', 'Landlord peer invites',
   'Invite fellow landlords, owners, and trusted contractors onto Folio.',
   '["landlord"]', '["landlord","property_owner","vendor"]', 'wizard_complete'),
  ('vendor_invite_clients', 'Vendor client invites',
   'Invite past clients so jobs and reviews can live on Folio.',
   '["vendor"]', '["property_owner","landlord"]', 'review_submitted'),
  ('vendor_invite_contractors', 'Vendor contractor invites',
   'Invite trades you already collaborate with.',
   '["vendor"]', '["vendor"]', 'signup'),
  ('pmc_invite_clients', 'PMC client invites',
   'Invite owner clients into an Owner portal pre-linked to your PMC.',
   '["property_manager"]', '["owner"]', 'wizard_complete'),
  ('property_owner_invite_peers', 'Property owner peer invites',
   'Invite other owners, landlords, and vendors you trust.',
   '["property_owner"]', '["landlord","property_owner","vendor"]', 'signup'),
  ('owner_invite_peers', 'Owner portal peer invites',
   'Invite fellow managed owners or self-managed landlords.',
   '["owner"]', '["owner","landlord"]', 'signup')
) AS v(slug, name, description, actor_roles, target_roles, default_outcome_type)
WHERE NOT EXISTS (
    SELECT 1 FROM atlas_programs p WHERE p.slug = v.slug AND p.tenant_id IS NULL
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
DROP TABLE IF EXISTS atlas_program_reward_grants;
DROP TABLE IF EXISTS atlas_program_reward_rules;
DROP TABLE IF EXISTS atlas_program_outcomes;
DROP TABLE IF EXISTS atlas_program_actions;
DROP TABLE IF EXISTS atlas_programs;
"#,
            )
            .await?;
        Ok(())
    }
}
