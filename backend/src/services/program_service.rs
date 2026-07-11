//! G-36 ProgramService — productized growth/incentive programs.
//!
//! See `docs/architecture/g36_atlas_programs_spec.md`.

use chrono::Utc;
use rust_decimal::Decimal;
use sea_orm::{
    ActiveModelTrait, ConnectionTrait, DatabaseConnection, DbErr, Set, Statement, TransactionTrait,
    Value,
};
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use uuid::Uuid;

use crate::entities::outbox_job;
use crate::types::outbox::OutboxJobType;
use crate::types::pm::{
    ProgramActionStatus, ProgramKind, ProgramOutcomeType, ProgramRewardBeneficiary,
    ProgramRewardType,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProgramRow {
    pub id: Uuid,
    pub slug: String,
    pub name: String,
    pub description: Option<String>,
    pub program_kind: String,
    pub campaign_id: Option<Uuid>,
    pub actor_roles: JsonValue,
    pub target_roles: JsonValue,
    pub config: JsonValue,
    pub default_outcome_type: String,
    pub is_active: bool,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProgramCreateInput {
    pub slug: String,
    pub name: String,
    pub description: Option<String>,
    pub program_kind: ProgramKind,
    pub campaign_id: Option<Uuid>,
    pub actor_roles: Option<JsonValue>,
    pub target_roles: Option<JsonValue>,
    pub config: Option<JsonValue>,
    pub default_outcome_type: Option<ProgramOutcomeType>,
    pub is_active: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ProgramUpdatePatch {
    pub is_active: Option<bool>,
    pub name: Option<String>,
    pub description: Option<Option<String>>,
    pub campaign_id: Option<Option<Uuid>>,
    pub config: Option<JsonValue>,
    pub actor_roles: Option<JsonValue>,
    pub target_roles: Option<JsonValue>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RewardRuleInput {
    pub beneficiary: ProgramRewardBeneficiary,
    pub reward_type: ProgramRewardType,
    pub amount: Decimal,
    pub trigger_outcome_type: ProgramOutcomeType,
    pub is_active: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RewardRuleRow {
    pub id: Uuid,
    pub program_id: Uuid,
    pub beneficiary: String,
    pub reward_type: String,
    pub amount: Decimal,
    pub trigger_outcome_type: String,
    pub is_active: bool,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProgramGrantRow {
    pub id: Uuid,
    pub program_action_id: Uuid,
    pub rule_id: Uuid,
    pub beneficiary_user_id: Uuid,
    pub status: String,
    pub reward_type: Option<String>,
    pub amount: Option<Decimal>,
    pub granted_at: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatusCount {
    pub status: String,
    pub count: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProgramAnalyticsSummary {
    pub total_actions: i64,
    pub total_grants: i64,
    pub actions_by_status: Vec<StatusCount>,
    pub outcomes_by_status: Vec<StatusCount>,
    pub grants_by_status: Vec<StatusCount>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProgramInstanceEnablementRow {
    pub id: Uuid,
    pub program_id: Uuid,
    pub app_instance_id: Uuid,
    pub is_enabled: bool,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstanceProgramRow {
    #[serde(flatten)]
    pub program: ProgramRow,
    pub enabled: bool,
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
            SELECT id, slug, name, description, program_kind, campaign_id,
                   actor_roles, target_roles, config, default_outcome_type,
                   is_active, created_at::text AS created_at, updated_at::text AS updated_at
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
            out.push(program_row_from_query(&row)?);
        }
        Ok(out)
    }

    pub async fn list_programs_admin(
        db: &DatabaseConnection,
        kind: Option<ProgramKind>,
        include_inactive: bool,
    ) -> Result<Vec<ProgramRow>, DbErr> {
        let kind = kind.map(|k| k.to_string());
        let stmt = Statement::from_sql_and_values(
            sea_orm::DatabaseBackend::Postgres,
            r#"
            SELECT id, slug, name, description, program_kind, campaign_id,
                   actor_roles, target_roles, config, default_outcome_type,
                   is_active, created_at::text AS created_at, updated_at::text AS updated_at
            FROM atlas_programs
            WHERE tenant_id IS NULL
              AND ($1::text IS NULL OR program_kind = $1)
              AND ($2::boolean = true OR is_active = true)
            ORDER BY name
            "#,
            [kind.into(), include_inactive.into()],
        );
        rows_to_programs(db.query_all(stmt).await?)
    }

    pub async fn get_program(
        db: &DatabaseConnection,
        id: Uuid,
    ) -> Result<Option<ProgramRow>, DbErr> {
        let stmt = Statement::from_sql_and_values(
            sea_orm::DatabaseBackend::Postgres,
            r#"
            SELECT id, slug, name, description, program_kind, campaign_id,
                   actor_roles, target_roles, config, default_outcome_type,
                   is_active, created_at::text AS created_at, updated_at::text AS updated_at
            FROM atlas_programs
            WHERE id = $1
            "#,
            [id.into()],
        );
        db.query_one(stmt)
            .await?
            .map(|row| program_row_from_query(&row))
            .transpose()
    }

    pub async fn create_program(
        db: &DatabaseConnection,
        input: ProgramCreateInput,
    ) -> Result<ProgramRow, String> {
        validate_json_array(input.actor_roles.as_ref(), "actor_roles")?;
        validate_json_array(input.target_roles.as_ref(), "target_roles")?;

        let stmt = Statement::from_sql_and_values(
            sea_orm::DatabaseBackend::Postgres,
            r#"
            INSERT INTO atlas_programs (
                id, tenant_id, slug, name, description, program_kind, campaign_id,
                actor_roles, target_roles, config, default_outcome_type, is_active,
                created_at, updated_at
            )
            VALUES (
                gen_random_uuid(), NULL, $1, $2, $3, $4, $5,
                COALESCE($6::jsonb, '[]'::jsonb),
                COALESCE($7::jsonb, '[]'::jsonb),
                COALESCE($8::jsonb, '{}'::jsonb),
                $9,
                COALESCE($10, true),
                now(), now()
            )
            RETURNING id, slug, name, description, program_kind, campaign_id,
                      actor_roles, target_roles, config, default_outcome_type,
                      is_active, created_at::text AS created_at, updated_at::text AS updated_at
            "#,
            [
                input.slug.trim().to_string().into(),
                input.name.trim().to_string().into(),
                input.description.into(),
                input.program_kind.to_string().into(),
                input.campaign_id.into(),
                input.actor_roles.into(),
                input.target_roles.into(),
                input.config.into(),
                input
                    .default_outcome_type
                    .unwrap_or(ProgramOutcomeType::Signup)
                    .to_string()
                    .into(),
                input.is_active.into(),
            ],
        );
        let row = db
            .query_one(stmt)
            .await
            .map_err(|e| e.to_string())?
            .ok_or_else(|| "Program create returned no row".to_string())?;
        program_row_from_query(&row).map_err(|e| e.to_string())
    }

    pub async fn update_program(
        db: &DatabaseConnection,
        id: Uuid,
        patch: ProgramUpdatePatch,
    ) -> Result<Option<ProgramRow>, String> {
        validate_json_array(patch.actor_roles.as_ref(), "actor_roles")?;
        validate_json_array(patch.target_roles.as_ref(), "target_roles")?;

        let mut sets: Vec<String> = Vec::new();
        let mut values: Vec<Value> = Vec::new();

        if let Some(v) = patch.is_active {
            values.push(v.into());
            sets.push(format!("is_active = ${}", values.len()));
        }
        if let Some(v) = patch.name {
            values.push(v.trim().to_string().into());
            sets.push(format!("name = ${}", values.len()));
        }
        if let Some(v) = patch.description {
            values.push(v.into());
            sets.push(format!("description = ${}", values.len()));
        }
        if let Some(v) = patch.campaign_id {
            values.push(v.into());
            sets.push(format!("campaign_id = ${}", values.len()));
        }
        if let Some(v) = patch.config {
            values.push(v.into());
            sets.push(format!("config = ${}", values.len()));
        }
        if let Some(v) = patch.actor_roles {
            values.push(v.into());
            sets.push(format!("actor_roles = ${}", values.len()));
        }
        if let Some(v) = patch.target_roles {
            values.push(v.into());
            sets.push(format!("target_roles = ${}", values.len()));
        }

        if sets.is_empty() {
            return Self::get_program(db, id).await.map_err(|e| e.to_string());
        }

        values.push(id.into());
        let id_param = values.len();
        let sql = format!(
            r#"
            UPDATE atlas_programs
            SET {}, updated_at = now()
            WHERE id = ${}
            "#,
            sets.join(", "),
            id_param
        );
        let result = db
            .execute(Statement::from_sql_and_values(
                sea_orm::DatabaseBackend::Postgres,
                sql,
                values,
            ))
            .await
            .map_err(|e| e.to_string())?;
        if result.rows_affected() == 0 {
            return Ok(None);
        }
        Self::get_program(db, id).await.map_err(|e| e.to_string())
    }

    pub async fn list_reward_rules(
        db: &DatabaseConnection,
        program_id: Uuid,
    ) -> Result<Vec<RewardRuleRow>, DbErr> {
        let stmt = Statement::from_sql_and_values(
            sea_orm::DatabaseBackend::Postgres,
            r#"
            SELECT id, program_id, beneficiary, reward_type, amount,
                   trigger_outcome_type, is_active, created_at::text AS created_at
            FROM atlas_program_reward_rules
            WHERE program_id = $1
            ORDER BY created_at ASC
            "#,
            [program_id.into()],
        );
        rows_to_reward_rules(db.query_all(stmt).await?)
    }

    pub async fn upsert_reward_rules(
        db: &DatabaseConnection,
        program_id: Uuid,
        rules: Vec<RewardRuleInput>,
    ) -> Result<Vec<RewardRuleRow>, String> {
        let txn = db.begin().await.map_err(|e| e.to_string())?;
        txn.execute(Statement::from_sql_and_values(
            sea_orm::DatabaseBackend::Postgres,
            "UPDATE atlas_program_reward_rules SET is_active = false WHERE program_id = $1",
            [program_id.into()],
        ))
        .await
        .map_err(|e| e.to_string())?;

        for rule in rules {
            let beneficiary = rule.beneficiary.to_string();
            let reward_type = rule.reward_type.to_string();
            let trigger_outcome_type = rule.trigger_outcome_type.to_string();
            let is_active = rule.is_active.unwrap_or(true);
            let updated = txn
                .execute(Statement::from_sql_and_values(
                    sea_orm::DatabaseBackend::Postgres,
                    r#"
                    UPDATE atlas_program_reward_rules
                    SET amount = $5, is_active = $6
                    WHERE program_id = $1
                      AND beneficiary = $2
                      AND reward_type = $3
                      AND trigger_outcome_type = $4
                    "#,
                    [
                        program_id.into(),
                        beneficiary.clone().into(),
                        reward_type.clone().into(),
                        trigger_outcome_type.clone().into(),
                        rule.amount.into(),
                        is_active.into(),
                    ],
                ))
                .await
                .map_err(|e| e.to_string())?;

            if updated.rows_affected() > 0 {
                continue;
            }

            txn.execute(Statement::from_sql_and_values(
                sea_orm::DatabaseBackend::Postgres,
                r#"
                INSERT INTO atlas_program_reward_rules (
                    id, program_id, beneficiary, reward_type, amount,
                    trigger_outcome_type, is_active, created_at
                )
                VALUES (gen_random_uuid(), $1, $2, $3, $4, $5, $6, now())
                "#,
                [
                    program_id.into(),
                    beneficiary.into(),
                    reward_type.into(),
                    rule.amount.into(),
                    trigger_outcome_type.into(),
                    is_active.into(),
                ],
            ))
            .await
            .map_err(|e| e.to_string())?;
        }

        let rows = txn
            .query_all(Statement::from_sql_and_values(
                sea_orm::DatabaseBackend::Postgres,
                r#"
                SELECT id, program_id, beneficiary, reward_type, amount,
                       trigger_outcome_type, is_active, created_at::text AS created_at
                FROM atlas_program_reward_rules
                WHERE program_id = $1
                ORDER BY created_at ASC
                "#,
                [program_id.into()],
            ))
            .await
            .map_err(|e| e.to_string())?;
        txn.commit().await.map_err(|e| e.to_string())?;
        rows_to_reward_rules(rows).map_err(|e| e.to_string())
    }

    pub async fn list_actions_for_program(
        db: &DatabaseConnection,
        program_id: Uuid,
        limit: i64,
    ) -> Result<Vec<ProgramActionRow>, DbErr> {
        let limit = limit.clamp(1, 500);
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
            WHERE a.program_id = $1
            ORDER BY a.created_at DESC
            LIMIT $2
            "#,
            [program_id.into(), limit.into()],
        );
        rows_to_actions(db.query_all(stmt).await?)
    }

    pub async fn program_analytics(
        db: &DatabaseConnection,
        program_id: Uuid,
    ) -> Result<ProgramAnalyticsSummary, DbErr> {
        let actions_by_status = count_rows(
            db,
            r#"
            SELECT status, COUNT(*)::bigint AS count
            FROM atlas_program_actions
            WHERE program_id = $1
            GROUP BY status
            ORDER BY status
            "#,
            program_id,
        )
        .await?;
        let outcomes_by_status = count_rows(
            db,
            r#"
            SELECT o.status, COUNT(*)::bigint AS count
            FROM atlas_program_outcomes o
            JOIN atlas_program_actions a ON a.id = o.program_action_id
            WHERE a.program_id = $1
            GROUP BY o.status
            ORDER BY o.status
            "#,
            program_id,
        )
        .await?;
        let grants_by_status = count_rows(
            db,
            r#"
            SELECT g.status, COUNT(*)::bigint AS count
            FROM atlas_program_reward_grants g
            JOIN atlas_program_actions a ON a.id = g.program_action_id
            WHERE a.program_id = $1
            GROUP BY g.status
            ORDER BY g.status
            "#,
            program_id,
        )
        .await?;
        Ok(ProgramAnalyticsSummary {
            total_actions: actions_by_status.iter().map(|r| r.count).sum(),
            total_grants: grants_by_status.iter().map(|r| r.count).sum(),
            actions_by_status,
            outcomes_by_status,
            grants_by_status,
        })
    }

    pub async fn list_grants_for_program(
        db: &DatabaseConnection,
        program_id: Uuid,
    ) -> Result<Vec<ProgramGrantRow>, DbErr> {
        let stmt = Statement::from_sql_and_values(
            sea_orm::DatabaseBackend::Postgres,
            r#"
            SELECT g.id, g.program_action_id, g.rule_id, g.beneficiary_user_id,
                   g.status, r.reward_type, r.amount,
                   g.granted_at::text AS granted_at, g.created_at::text AS created_at
            FROM atlas_program_reward_grants g
            JOIN atlas_program_actions a ON a.id = g.program_action_id
            JOIN atlas_program_reward_rules r ON r.id = g.rule_id
            WHERE a.program_id = $1
            ORDER BY g.created_at DESC
            LIMIT 500
            "#,
            [program_id.into()],
        );
        rows_to_grants(db.query_all(stmt).await?)
    }

    pub async fn list_instance_enablements(
        db: &DatabaseConnection,
        program_id: Uuid,
    ) -> Result<Vec<ProgramInstanceEnablementRow>, DbErr> {
        let stmt = Statement::from_sql_and_values(
            sea_orm::DatabaseBackend::Postgres,
            r#"
            SELECT id, program_id, app_instance_id, is_enabled, updated_at::text AS updated_at
            FROM atlas_program_instance_enablements
            WHERE program_id = $1
            ORDER BY updated_at DESC
            "#,
            [program_id.into()],
        );
        rows_to_enablements(db.query_all(stmt).await?)
    }

    pub async fn set_instance_enablement(
        db: &DatabaseConnection,
        program_id: Uuid,
        app_instance_id: Uuid,
        is_enabled: bool,
    ) -> Result<ProgramInstanceEnablementRow, DbErr> {
        let stmt = Statement::from_sql_and_values(
            sea_orm::DatabaseBackend::Postgres,
            r#"
            INSERT INTO atlas_program_instance_enablements (
                id, program_id, app_instance_id, is_enabled, updated_at
            )
            VALUES (gen_random_uuid(), $1, $2, $3, now())
            ON CONFLICT (program_id, app_instance_id)
            DO UPDATE SET is_enabled = EXCLUDED.is_enabled, updated_at = now()
            RETURNING id, program_id, app_instance_id, is_enabled, updated_at::text AS updated_at
            "#,
            [program_id.into(), app_instance_id.into(), is_enabled.into()],
        );
        let row = db.query_one(stmt).await?.ok_or(DbErr::RecordNotFound(
            "instance enablement upsert returned no row".into(),
        ))?;
        enablement_from_query(&row)
    }

    pub async fn list_programs_for_instance(
        db: &DatabaseConnection,
        app_instance_id: Uuid,
        kind: Option<ProgramKind>,
        actor_role: Option<&str>,
    ) -> Result<Vec<InstanceProgramRow>, DbErr> {
        let kind = kind.map(|k| k.to_string());
        let stmt = Statement::from_sql_and_values(
            sea_orm::DatabaseBackend::Postgres,
            r#"
            WITH has_enablements AS (
                SELECT EXISTS (
                    SELECT 1 FROM atlas_program_instance_enablements
                    WHERE app_instance_id = $1
                ) AS has_rows
            )
            SELECT p.id, p.slug, p.name, p.description, p.program_kind, p.campaign_id,
                   p.actor_roles, p.target_roles, p.config, p.default_outcome_type,
                   p.is_active, p.created_at::text AS created_at, p.updated_at::text AS updated_at,
                   CASE WHEN h.has_rows THEN COALESCE(e.is_enabled, false) ELSE true END AS enabled
            FROM atlas_programs p
            CROSS JOIN has_enablements h
            LEFT JOIN atlas_program_instance_enablements e
              ON e.program_id = p.id AND e.app_instance_id = $1
            WHERE p.tenant_id IS NULL
              AND p.is_active = true
              AND ($2::text IS NULL OR p.program_kind = $2)
              AND ($3::text IS NULL OR p.actor_roles @> jsonb_build_array($3::text))
              AND (h.has_rows = false OR e.is_enabled = true)
            ORDER BY p.name
            "#,
            [
                app_instance_id.into(),
                kind.into(),
                actor_role.map(|s| s.to_string()).into(),
            ],
        );
        let rows = db.query_all(stmt).await?;
        let mut out = Vec::with_capacity(rows.len());
        for row in rows {
            out.push(InstanceProgramRow {
                program: program_row_from_query(&row)?,
                enabled: row.try_get("", "enabled")?,
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

        // Vendors invited onto Folio also track first completed job.
        if target_role == "vendor"
            && default_outcome != ProgramOutcomeType::FirstJobLogged.to_string()
        {
            db.execute(Statement::from_sql_and_values(
                sea_orm::DatabaseBackend::Postgres,
                r#"
                INSERT INTO atlas_program_outcomes
                    (id, program_action_id, outcome_type, status, created_at)
                VALUES (gen_random_uuid(), $1, 'first_job_logged', 'pending', now())
                "#,
                [action_id.into()],
            ))
            .await
            .map_err(|e| format!("first_job outcome create failed: {e}"))?;
        }

        // Best-effort outbound invite email via transactional outbox.
        if let Err(e) = Self::enqueue_network_invite_email(
            db,
            tenant_id.unwrap_or(actor_user_id),
            actor_user_id,
            &target_email,
            &code,
            personal_note.as_deref(),
        )
        .await
        {
            tracing::warn!("G-36 network invite email enqueue failed (non-fatal): {e}");
        }

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

    async fn enqueue_network_invite_email(
        db: &DatabaseConnection,
        tenant_id: Uuid,
        actor_user_id: Uuid,
        target_email: &str,
        invite_code: &str,
        personal_note: Option<&str>,
    ) -> Result<(), String> {
        let inviter = db
            .query_one(Statement::from_sql_and_values(
                sea_orm::DatabaseBackend::Postgres,
                r#"SELECT COALESCE(
                       NULLIF(trim(first_name || ' ' || last_name), ''),
                       NULLIF(trim(email), ''),
                       'A Folio member'
                   ) AS name
                   FROM "user" WHERE id = $1"#,
                [actor_user_id.into()],
            ))
            .await
            .map_err(|e| e.to_string())?;
        let inviter_name: String = inviter
            .and_then(|r| r.try_get::<String>("", "name").ok())
            .unwrap_or_else(|| "A Folio member".into());

        let base = std::env::var("FOLIO_PUBLIC_URL")
            .or_else(|_| std::env::var("PUBLIC_BASE_URL"))
            .unwrap_or_else(|_| "http://localhost:3000".into());
        let join_url = format!("{}/join/{}", base.trim_end_matches('/'), invite_code);

        let note_html = personal_note
            .map(|n| n.trim())
            .filter(|n| !n.is_empty())
            .map(|n| {
                format!(
                    r#"<p style="margin:0 0 20px;font-size:14px;line-height:1.6;color:#475569;border-left:3px solid #cbd5e1;padding-left:12px;">{}</p>"#,
                    html_escape(n)
                )
            })
            .unwrap_or_default();

        let body_html = format!(
            r#"<!DOCTYPE html>
<html lang="en"><head><meta charset="UTF-8"/><meta name="viewport" content="width=device-width,initial-scale=1"/>
<title>You're invited to Folio</title></head>
<body style="margin:0;padding:0;background:#f4f6f9;font-family:'Segoe UI',Helvetica,Arial,sans-serif;color:#0f172a;">
  <table width="100%" cellpadding="0" cellspacing="0" style="padding:40px 0;"><tr><td align="center">
    <table width="560" cellpadding="0" cellspacing="0" style="background:#fff;border:1px solid #e2e8f0;border-radius:12px;overflow:hidden;max-width:560px;width:100%;">
      <tr><td style="padding:28px 32px 8px;">
        <div style="font-size:13px;font-weight:700;letter-spacing:.06em;text-transform:uppercase;color:#64748b;">Folio</div>
        <h1 style="margin:12px 0 8px;font-size:22px;font-weight:800;letter-spacing:-.02em;">You're invited</h1>
        <p style="margin:0 0 16px;font-size:15px;line-height:1.6;color:#475569;">
          {inviter} invited you to join Folio. Open your personal link to get started.
        </p>
        {note}
        <a href="{join}" style="display:inline-block;background:#0f172a;color:#fff;text-decoration:none;font-size:14px;font-weight:700;padding:12px 18px;border-radius:8px;">Accept invite</a>
        <p style="margin:20px 0 0;font-size:12px;color:#94a3b8;line-height:1.5;">
          Or paste this link into your browser:<br/>
          <span style="color:#64748b;word-break:break-all;">{join}</span>
        </p>
      </td></tr>
      <tr><td style="padding:16px 32px 28px;font-size:11px;color:#94a3b8;">
        If you were not expecting this, you can ignore this email.
      </td></tr>
    </table>
  </td></tr></table>
</body></html>"#,
            inviter = html_escape(&inviter_name),
            note = note_html,
            join = join_url,
        );

        let payload = crate::handlers::communications::SendEmailPayload {
            tenant_id,
            to_email: target_email.to_string(),
            subject: format!("{inviter_name} invited you to Folio"),
            body_html,
            attachments: Vec::new(),
        };
        let job_payload = serde_json::to_value(&payload).map_err(|e| e.to_string())?;
        let job = outbox_job::ActiveModel {
            id: Set(Uuid::new_v4()),
            tenant_id: Set(tenant_id),
            job_type: Set(OutboxJobType::SendMagicLinkEmail.to_string()),
            payload: Set(job_payload),
            status: Set("pending".to_string()),
            attempts: Set(0),
            created_at: Set(Utc::now()),
            run_at: Set(Utc::now()),
            ..Default::default()
        };
        job.insert(db).await.map_err(|e| e.to_string())?;
        Ok(())
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

        // Create reward grants for matching rules (then apply subscription credits)
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

        if let Err(e) = Self::apply_subscription_credit_grants(db, action_id).await {
            tracing::warn!("G-36 apply_subscription_credit_grants failed (non-fatal): {e}");
        }

        Ok(())
    }

    /// Apply granted `subscription_credit_days` rows into the internal credit ledger.
    /// Idempotent via unique `grant_id`. Stripe consumption is a later integration.
    pub async fn apply_subscription_credit_grants(
        db: &DatabaseConnection,
        action_id: Uuid,
    ) -> Result<u64, String> {
        let result = db
            .execute(Statement::from_sql_and_values(
                sea_orm::DatabaseBackend::Postgres,
                r#"
                WITH applied AS (
                    INSERT INTO atlas_subscription_credit_ledger (id, user_id, grant_id, days, note, created_at)
                    SELECT gen_random_uuid(), g.beneficiary_user_id, g.id, r.amount,
                           'G-36 ' || COALESCE(p.slug, 'program') || ' reward',
                           now()
                    FROM atlas_program_reward_grants g
                    JOIN atlas_program_reward_rules r ON r.id = g.rule_id
                    JOIN atlas_program_actions a ON a.id = g.program_action_id
                    JOIN atlas_programs p ON p.id = a.program_id
                    WHERE g.program_action_id = $1
                      AND g.status = 'granted'
                      AND r.reward_type = 'subscription_credit_days'
                      AND r.amount > 0
                    ON CONFLICT (grant_id) DO NOTHING
                    RETURNING grant_id
                )
                UPDATE atlas_program_reward_grants g
                SET status = 'applied'
                FROM applied
                WHERE g.id = applied.grant_id
                "#,
                [action_id.into()],
            ))
            .await
            .map_err(|e| e.to_string())?;
        Ok(result.rows_affected())
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

    /// Complete pending outcomes for actions targeting a user (e.g. invited vendor's first job).
    pub async fn complete_outcomes_for_target_user(
        db: &DatabaseConnection,
        target_user_id: Uuid,
        outcome_type: ProgramOutcomeType,
        evidence_entity_type: Option<&str>,
        evidence_entity_id: Option<Uuid>,
    ) -> Result<(), String> {
        let rows = db
            .query_all(Statement::from_sql_and_values(
                sea_orm::DatabaseBackend::Postgres,
                r#"
                SELECT DISTINCT a.id
                FROM atlas_program_actions a
                JOIN atlas_program_outcomes o ON o.program_action_id = a.id
                WHERE a.target_user_id = $1
                  AND o.outcome_type = $2
                  AND o.status = 'pending'
                "#,
                [target_user_id.into(), outcome_type.to_string().into()],
            ))
            .await
            .map_err(|e| e.to_string())?;

        for row in rows {
            let action_id: Uuid = row.try_get("", "id").map_err(|e| e.to_string())?;
            Self::complete_outcome(
                db,
                action_id,
                outcome_type.clone(),
                evidence_entity_type,
                evidence_entity_id,
            )
            .await?;
        }
        Ok(())
    }

    /// Complete pending outcomes where the program actor is this user
    /// (e.g. vendor receives a review from an invited client).
    pub async fn complete_outcomes_for_actor_user(
        db: &DatabaseConnection,
        actor_user_id: Uuid,
        outcome_type: ProgramOutcomeType,
        evidence_entity_type: Option<&str>,
        evidence_entity_id: Option<Uuid>,
    ) -> Result<(), String> {
        let rows = db
            .query_all(Statement::from_sql_and_values(
                sea_orm::DatabaseBackend::Postgres,
                r#"
                SELECT DISTINCT a.id
                FROM atlas_program_actions a
                JOIN atlas_program_outcomes o ON o.program_action_id = a.id
                WHERE a.actor_user_id = $1
                  AND o.outcome_type = $2
                  AND o.status = 'pending'
                "#,
                [actor_user_id.into(), outcome_type.to_string().into()],
            ))
            .await
            .map_err(|e| e.to_string())?;

        for row in rows {
            let action_id: Uuid = row.try_get("", "id").map_err(|e| e.to_string())?;
            Self::complete_outcome(
                db,
                action_id,
                outcome_type.clone(),
                evidence_entity_type,
                evidence_entity_id,
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

fn rows_to_programs(rows: Vec<sea_orm::QueryResult>) -> Result<Vec<ProgramRow>, DbErr> {
    let mut out = Vec::with_capacity(rows.len());
    for row in rows {
        out.push(program_row_from_query(&row)?);
    }
    Ok(out)
}

fn program_row_from_query(row: &sea_orm::QueryResult) -> Result<ProgramRow, DbErr> {
    Ok(ProgramRow {
        id: row.try_get("", "id")?,
        slug: row.try_get("", "slug")?,
        name: row.try_get("", "name")?,
        description: row.try_get("", "description").ok().flatten(),
        program_kind: row.try_get("", "program_kind")?,
        campaign_id: row.try_get("", "campaign_id").ok().flatten(),
        actor_roles: row
            .try_get("", "actor_roles")
            .unwrap_or(JsonValue::Array(vec![])),
        target_roles: row
            .try_get("", "target_roles")
            .unwrap_or(JsonValue::Array(vec![])),
        config: row
            .try_get("", "config")
            .unwrap_or_else(|_| serde_json::json!({})),
        default_outcome_type: row.try_get("", "default_outcome_type")?,
        is_active: row.try_get("", "is_active")?,
        created_at: row.try_get("", "created_at").ok().flatten(),
        updated_at: row.try_get("", "updated_at").ok().flatten(),
    })
}

fn rows_to_actions(rows: Vec<sea_orm::QueryResult>) -> Result<Vec<ProgramActionRow>, DbErr> {
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

fn rows_to_reward_rules(rows: Vec<sea_orm::QueryResult>) -> Result<Vec<RewardRuleRow>, DbErr> {
    let mut out = Vec::with_capacity(rows.len());
    for row in rows {
        out.push(RewardRuleRow {
            id: row.try_get("", "id")?,
            program_id: row.try_get("", "program_id")?,
            beneficiary: row.try_get("", "beneficiary")?,
            reward_type: row.try_get("", "reward_type")?,
            amount: row.try_get("", "amount")?,
            trigger_outcome_type: row.try_get("", "trigger_outcome_type")?,
            is_active: row.try_get("", "is_active")?,
            created_at: row.try_get("", "created_at").unwrap_or_default(),
        });
    }
    Ok(out)
}

fn rows_to_grants(rows: Vec<sea_orm::QueryResult>) -> Result<Vec<ProgramGrantRow>, DbErr> {
    let mut out = Vec::with_capacity(rows.len());
    for row in rows {
        out.push(ProgramGrantRow {
            id: row.try_get("", "id")?,
            program_action_id: row.try_get("", "program_action_id")?,
            rule_id: row.try_get("", "rule_id")?,
            beneficiary_user_id: row.try_get("", "beneficiary_user_id")?,
            status: row.try_get("", "status")?,
            reward_type: row.try_get("", "reward_type").ok().flatten(),
            amount: row.try_get("", "amount").ok().flatten(),
            granted_at: row.try_get("", "granted_at").ok().flatten(),
            created_at: row.try_get("", "created_at").unwrap_or_default(),
        });
    }
    Ok(out)
}

fn rows_to_enablements(
    rows: Vec<sea_orm::QueryResult>,
) -> Result<Vec<ProgramInstanceEnablementRow>, DbErr> {
    let mut out = Vec::with_capacity(rows.len());
    for row in rows {
        out.push(enablement_from_query(&row)?);
    }
    Ok(out)
}

fn enablement_from_query(
    row: &sea_orm::QueryResult,
) -> Result<ProgramInstanceEnablementRow, DbErr> {
    Ok(ProgramInstanceEnablementRow {
        id: row.try_get("", "id")?,
        program_id: row.try_get("", "program_id")?,
        app_instance_id: row.try_get("", "app_instance_id")?,
        is_enabled: row.try_get("", "is_enabled")?,
        updated_at: row.try_get("", "updated_at").unwrap_or_default(),
    })
}

async fn count_rows(
    db: &DatabaseConnection,
    sql: &str,
    program_id: Uuid,
) -> Result<Vec<StatusCount>, DbErr> {
    let rows = db
        .query_all(Statement::from_sql_and_values(
            sea_orm::DatabaseBackend::Postgres,
            sql,
            [program_id.into()],
        ))
        .await?;
    let mut out = Vec::with_capacity(rows.len());
    for row in rows {
        out.push(StatusCount {
            status: row.try_get("", "status")?,
            count: row.try_get("", "count")?,
        });
    }
    Ok(out)
}

fn validate_json_array(value: Option<&JsonValue>, field: &str) -> Result<(), String> {
    if value.is_some_and(|v| !v.is_array()) {
        return Err(format!("{field} must be a JSON array"));
    }
    Ok(())
}

fn html_escape(s: &str) -> String {
    s.chars()
        .map(|c| match c {
            '&' => "&amp;".to_string(),
            '<' => "&lt;".to_string(),
            '>' => "&gt;".to_string(),
            '"' => "&quot;".to_string(),
            '\'' => "&#39;".to_string(),
            _ => c.to_string(),
        })
        .collect()
}
