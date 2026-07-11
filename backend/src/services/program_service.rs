//! G-36 ProgramService — productized growth/incentive programs.
//!
//! See `docs/architecture/g36_atlas_programs_spec.md`.

use chrono::Utc;
use sea_orm::{ConnectionTrait, DatabaseConnection, DbErr, Statement};
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use uuid::Uuid;

use crate::types::pm::{ProgramActionStatus, ProgramKind, ProgramOutcomeType};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProgramRow {
    pub id: Uuid,
    pub slug: String,
    pub name: String,
    pub description: Option<String>,
    pub program_kind: String,
    pub actor_roles: JsonValue,
    pub target_roles: JsonValue,
    pub default_outcome_type: String,
    pub is_active: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProgramActionRow {
    pub id: Uuid,
    pub program_id: Uuid,
    pub program_slug: Option<String>,
    pub actor_user_id: Uuid,
    pub target_email: Option<String>,
    pub target_role: Option<String>,
    pub delivery_entity_type: Option<String>,
    pub delivery_entity_id: Option<Uuid>,
    pub status: String,
    pub invite_code: Option<String>,
    pub outcome_type: Option<String>,
    pub outcome_status: Option<String>,
    pub created_at: String,
}

pub struct ProgramService;

impl ProgramService {
    pub async fn list_programs(
        db: &DatabaseConnection,
        kind: Option<&str>,
        actor_role: Option<&str>,
    ) -> Result<Vec<ProgramRow>, DbErr> {
        let stmt = Statement::from_sql_and_values(
            sea_orm::DatabaseBackend::Postgres,
            r#"
            SELECT id, slug, name, description, program_kind, actor_roles, target_roles,
                   default_outcome_type, is_active
            FROM atlas_programs
            WHERE is_active = true
              AND tenant_id IS NULL
              AND ($1::text IS NULL OR program_kind = $1)
              AND ($2::text IS NULL OR actor_roles @> jsonb_build_array($2::text))
            ORDER BY name
            "#,
            [
                kind.map(|s| s.to_string()).into(),
                actor_role.map(|s| s.to_string()).into(),
            ],
        );
        let rows = db.query_all(stmt).await?;
        let mut out = Vec::with_capacity(rows.len());
        for row in rows {
            out.push(ProgramRow {
                id: row.try_get("", "id")?,
                slug: row.try_get("", "slug")?,
                name: row.try_get("", "name")?,
                description: row.try_get("", "description").ok().flatten(),
                program_kind: row.try_get("", "program_kind")?,
                actor_roles: row.try_get("", "actor_roles").unwrap_or(JsonValue::Array(vec![])),
                target_roles: row.try_get("", "target_roles").unwrap_or(JsonValue::Array(vec![])),
                default_outcome_type: row.try_get("", "default_outcome_type")?,
                is_active: row.try_get("", "is_active")?,
            });
        }
        Ok(out)
    }

    /// Create a NetworkInvite action and an underlying invite code delivery rail.
    pub async fn create_network_invite_action(
        db: &DatabaseConnection,
        program_id: Uuid,
        actor_user_id: Uuid,
        tenant_id: Option<Uuid>,
        target_email: String,
        target_role: String,
        personal_note: Option<String>,
    ) -> Result<ProgramActionRow, String> {
        let prog = db
            .query_one(Statement::from_sql_and_values(
                sea_orm::DatabaseBackend::Postgres,
                r#"SELECT id, slug, program_kind, default_outcome_type
                   FROM atlas_programs WHERE id = $1 AND is_active = true"#,
                [program_id.into()],
            ))
            .await
            .map_err(|e| e.to_string())?
            .ok_or_else(|| "Program not found".to_string())?;

        let kind: String = prog.try_get("", "program_kind").unwrap_or_default();
        if kind != ProgramKind::NetworkInvite.to_string() {
            return Err("Program is not a network_invite kind".into());
        }
        let slug: String = prog.try_get("", "slug").unwrap_or_default();
        let default_outcome: String = prog
            .try_get("", "default_outcome_type")
            .unwrap_or_else(|_| ProgramOutcomeType::Signup.to_string());

        let code = format!(
            "NI-{}",
            &Uuid::new_v4().to_string().replace('-', "")[..8].to_uppercase()
        );
        let invite_id = Uuid::new_v4();
        let workspace_id = tenant_id.unwrap_or(actor_user_id);
        // atlas_invite_codes.role CHECK does not include property_owner yet.
        let invite_role = match target_role.as_str() {
            "property_owner" => "landlord".to_string(),
            other => other.to_string(),
        };

        db.execute(Statement::from_sql_and_values(
            sea_orm::DatabaseBackend::Postgres,
            r#"
            INSERT INTO atlas_invite_codes
                (id, code, role, workspace_id, landlord_id, created_by, label, invite_message,
                 max_uses, uses_count, is_active, created_at)
            VALUES
                ($1, $2, $3, $4, $5, $5, $6, $7, 1, 0, true, now())
            "#,
            [
                invite_id.into(),
                code.clone().into(),
                invite_role.into(),
                workspace_id.into(),
                actor_user_id.into(),
                format!("G-36 {slug}").into(),
                personal_note.clone().into(),
            ],
        ))
        .await
        .map_err(|e| format!("invite code create failed: {e}"))?;

        let action_id = Uuid::new_v4();
        let meta = serde_json::json!({ "personal_note": personal_note });
        db.execute(Statement::from_sql_and_values(
            sea_orm::DatabaseBackend::Postgres,
            r#"
            INSERT INTO atlas_program_actions
                (id, program_id, actor_user_id, tenant_id, target_email, target_role,
                 delivery_entity_type, delivery_entity_id, status, metadata, created_at, updated_at)
            VALUES
                ($1, $2, $3, $4, $5, $6, 'invite_code', $7, $8, $9, now(), now())
            "#,
            [
                action_id.into(),
                program_id.into(),
                actor_user_id.into(),
                tenant_id.into(),
                target_email.clone().into(),
                target_role.clone().into(),
                invite_id.into(),
                ProgramActionStatus::Sent.to_string().into(),
                meta.into(),
            ],
        ))
        .await
        .map_err(|e| format!("program action create failed: {e}"))?;

        // Pending outcome row (completed later)
        db.execute(Statement::from_sql_and_values(
            sea_orm::DatabaseBackend::Postgres,
            r#"
            INSERT INTO atlas_program_outcomes
                (id, program_action_id, outcome_type, status, created_at)
            VALUES (gen_random_uuid(), $1, $2, 'pending', now())
            "#,
            [action_id.into(), default_outcome.clone().into()],
        ))
        .await
        .map_err(|e| format!("outcome create failed: {e}"))?;

        Ok(ProgramActionRow {
            id: action_id,
            program_id,
            program_slug: Some(slug),
            actor_user_id,
            target_email: Some(target_email),
            target_role: Some(target_role),
            delivery_entity_type: Some("invite_code".into()),
            delivery_entity_id: Some(invite_id),
            status: ProgramActionStatus::Sent.to_string(),
            invite_code: Some(code),
            outcome_type: Some(default_outcome),
            outcome_status: Some("pending".into()),
            created_at: Utc::now().to_rfc3339(),
        })
    }

    pub async fn mark_action_accepted(
        db: &DatabaseConnection,
        delivery_entity_type: &str,
        delivery_entity_id: Uuid,
        target_user_id: Uuid,
    ) -> Result<(), DbErr> {
        db.execute(Statement::from_sql_and_values(
            sea_orm::DatabaseBackend::Postgres,
            r#"
            UPDATE atlas_program_actions
            SET status = 'accepted',
                target_user_id = $1,
                updated_at = now()
            WHERE delivery_entity_type = $2
              AND delivery_entity_id = $3
              AND status IN ('created', 'sent', 'opened')
            "#,
            [
                target_user_id.into(),
                delivery_entity_type.to_string().into(),
                delivery_entity_id.into(),
            ],
        ))
        .await?;
        Ok(())
    }

    pub async fn complete_outcome(
        db: &DatabaseConnection,
        action_id: Uuid,
        outcome_type: ProgramOutcomeType,
        evidence_entity_type: Option<&str>,
        evidence_entity_id: Option<Uuid>,
    ) -> Result<(), String> {
        db.execute(Statement::from_sql_and_values(
            sea_orm::DatabaseBackend::Postgres,
            r#"
            UPDATE atlas_program_outcomes
            SET status = 'completed',
                completed_at = now(),
                evidence_entity_type = COALESCE($2, evidence_entity_type),
                evidence_entity_id = COALESCE($3, evidence_entity_id)
            WHERE program_action_id = $1
              AND outcome_type = $4
              AND status = 'pending'
            "#,
            [
                action_id.into(),
                evidence_entity_type.map(|s| s.to_string()).into(),
                evidence_entity_id.into(),
                outcome_type.to_string().into(),
            ],
        ))
        .await
        .map_err(|e| e.to_string())?;

        db.execute(Statement::from_sql_and_values(
            sea_orm::DatabaseBackend::Postgres,
            r#"
            UPDATE atlas_program_actions
            SET status = 'outcome_complete', updated_at = now()
            WHERE id = $1
            "#,
            [action_id.into()],
        ))
        .await
        .map_err(|e| e.to_string())?;

        // Create reward grants for matching rules (pending; no billing in v1)
        db.execute(Statement::from_sql_and_values(
            sea_orm::DatabaseBackend::Postgres,
            r#"
            INSERT INTO atlas_program_reward_grants
                (id, program_action_id, rule_id, beneficiary_user_id, status, granted_at, created_at)
            SELECT gen_random_uuid(), a.id, r.id,
                   CASE WHEN r.beneficiary = 'actor' THEN a.actor_user_id
                        ELSE COALESCE(a.target_user_id, a.actor_user_id) END,
                   'granted', now(), now()
            FROM atlas_program_actions a
            JOIN atlas_program_reward_rules r ON r.program_id = a.program_id AND r.is_active = true
            WHERE a.id = $1
              AND r.trigger_outcome_type = $2
              AND r.reward_type <> 'none'
              AND NOT EXISTS (
                  SELECT 1 FROM atlas_program_reward_grants g
                  WHERE g.program_action_id = a.id AND g.rule_id = r.id
              )
            "#,
            [action_id.into(), outcome_type.to_string().into()],
        ))
        .await
        .map_err(|e| e.to_string())?;

        Ok(())
    }

    /// Complete outcomes for actions linked to an invite code (wizard finish hook).
    pub async fn complete_outcomes_for_invite_code(
        db: &DatabaseConnection,
        invite_code_id: Uuid,
        outcome_type: ProgramOutcomeType,
        evidence_user_id: Uuid,
    ) -> Result<(), String> {
        let rows = db
            .query_all(Statement::from_sql_and_values(
                sea_orm::DatabaseBackend::Postgres,
                r#"SELECT id FROM atlas_program_actions
                   WHERE delivery_entity_type = 'invite_code'
                     AND delivery_entity_id = $1"#,
                [invite_code_id.into()],
            ))
            .await
            .map_err(|e| e.to_string())?;

        for row in rows {
            let action_id: Uuid = row.try_get("", "id").map_err(|e| e.to_string())?;
            Self::complete_outcome(
                db,
                action_id,
                outcome_type.clone(),
                Some("user"),
                Some(evidence_user_id),
            )
            .await?;
        }
        Ok(())
    }

    pub async fn list_actions_for_actor(
        db: &DatabaseConnection,
        actor_user_id: Uuid,
    ) -> Result<Vec<ProgramActionRow>, DbErr> {
        let stmt = Statement::from_sql_and_values(
            sea_orm::DatabaseBackend::Postgres,
            r#"
            SELECT a.id, a.program_id, p.slug AS program_slug, a.actor_user_id,
                   a.target_email, a.target_role, a.delivery_entity_type, a.delivery_entity_id,
                   a.status, c.code AS invite_code,
                   o.outcome_type, o.status AS outcome_status,
                   a.created_at::text AS created_at
            FROM atlas_program_actions a
            JOIN atlas_programs p ON p.id = a.program_id
            LEFT JOIN atlas_invite_codes c
              ON a.delivery_entity_type = 'invite_code' AND c.id = a.delivery_entity_id
            LEFT JOIN LATERAL (
                SELECT outcome_type, status FROM atlas_program_outcomes
                WHERE program_action_id = a.id
                ORDER BY created_at DESC LIMIT 1
            ) o ON true
            WHERE a.actor_user_id = $1
            ORDER BY a.created_at DESC
            LIMIT 100
            "#,
            [actor_user_id.into()],
        );
        let rows = db.query_all(stmt).await?;
        let mut out = Vec::with_capacity(rows.len());
        for row in rows {
            out.push(ProgramActionRow {
                id: row.try_get("", "id")?,
                program_id: row.try_get("", "program_id")?,
                program_slug: row.try_get("", "program_slug").ok().flatten(),
                actor_user_id: row.try_get("", "actor_user_id")?,
                target_email: row.try_get("", "target_email").ok().flatten(),
                target_role: row.try_get("", "target_role").ok().flatten(),
                delivery_entity_type: row.try_get("", "delivery_entity_type").ok().flatten(),
                delivery_entity_id: row.try_get("", "delivery_entity_id").ok().flatten(),
                status: row.try_get("", "status")?,
                invite_code: row.try_get("", "invite_code").ok().flatten(),
                outcome_type: row.try_get("", "outcome_type").ok().flatten(),
                outcome_status: row.try_get("", "outcome_status").ok().flatten(),
                created_at: row.try_get("", "created_at").unwrap_or_default(),
            });
        }
        Ok(out)
    }
}
