//! G-36 follow-up: seed NetworkInvite reward rules (grant ledger only; no billing).

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
-- One actor-facing rule per seeded NetworkInvite program.
-- Grants are written when the program's default outcome completes.
-- Billing / credit application is intentionally out of scope for v1.
INSERT INTO atlas_program_reward_rules (
    id, program_id, beneficiary, reward_type, amount,
    trigger_outcome_type, is_active, created_at
)
SELECT
    gen_random_uuid(),
    p.id,
    'actor',
    'subscription_credit_days',
    14,
    p.default_outcome_type,
    true,
    now()
FROM atlas_programs p
WHERE p.tenant_id IS NULL
  AND p.program_kind = 'network_invite'
  AND p.slug IN (
    'landlord_invite_peers',
    'vendor_invite_clients',
    'vendor_invite_contractors',
    'pmc_invite_clients',
    'property_owner_invite_peers',
    'owner_invite_peers'
  )
  AND NOT EXISTS (
      SELECT 1 FROM atlas_program_reward_rules r
      WHERE r.program_id = p.id
        AND r.beneficiary = 'actor'
        AND r.reward_type = 'subscription_credit_days'
        AND r.trigger_outcome_type = p.default_outcome_type
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
DELETE FROM atlas_program_reward_rules r
USING atlas_programs p
WHERE r.program_id = p.id
  AND p.tenant_id IS NULL
  AND p.program_kind = 'network_invite'
  AND r.beneficiary = 'actor'
  AND r.reward_type = 'subscription_credit_days'
  AND r.amount = 14;
"#,
            )
            .await?;
        Ok(())
    }
}
