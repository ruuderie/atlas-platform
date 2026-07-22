//! Same-tenant property manager hire (landlord → PM on their book).
//!
//! Reuses G-11 (`management_agreement`), G-32 (`atlas_user_app_roles` +
//! `atlas_user_asset_access`), and Folio invite codes. No new G-number.

use chrono::Utc;
use sea_orm::{ConnectionTrait, DatabaseConnection, DbBackend, Statement};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::services::rbac::RbacService;
use crate::types::pm::PmContractType;

/// Scope stamped into `atlas_contracts.terms_metadata`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DelegationScope {
    Asset,
    Portfolio,
}

impl DelegationScope {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Asset => "asset",
            Self::Portfolio => "portfolio",
        }
    }
}

impl TryFrom<&str> for DelegationScope {
    type Error = String;
    fn try_from(s: &str) -> Result<Self, Self::Error> {
        match s {
            "asset" => Ok(Self::Asset),
            "portfolio" => Ok(Self::Portfolio),
            other => Err(format!("unknown DelegationScope: '{other}'")),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ManagerPerson {
    pub user_id: Uuid,
    pub name: String,
    pub email: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PendingManagerInvite {
    pub invite_id: Uuid,
    pub code: String,
    pub join_url: String,
    pub label: Option<String>,
    pub scope: DelegationScope,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ManagerStatus {
    None,
    Pending,
    Active,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetManagerState {
    pub status: ManagerStatus,
    pub scope: Option<DelegationScope>,
    pub contract_id: Option<Uuid>,
    pub manager: Option<ManagerPerson>,
    pub invite: Option<PendingManagerInvite>,
}

pub struct ManagementDelegationService;

impl ManagementDelegationService {
    /// Parse asset UUIDs from invite `asset_id` + `asset_ids_csv`.
    pub fn parse_invite_asset_ids(
        asset_id: Option<Uuid>,
        asset_ids_csv: Option<&str>,
    ) -> Vec<Uuid> {
        let mut ids = Vec::new();
        if let Some(id) = asset_id {
            ids.push(id);
        }
        if let Some(csv) = asset_ids_csv {
            for part in csv.split(',') {
                let t = part.trim();
                if t.is_empty() {
                    continue;
                }
                if let Ok(u) = Uuid::parse_str(t) {
                    if !ids.contains(&u) {
                        ids.push(u);
                    }
                }
            }
        }
        ids
    }

    pub fn build_terms_metadata(
        scope: DelegationScope,
        asset_ids: &[Uuid],
        invite_code_id: Uuid,
    ) -> serde_json::Value {
        serde_json::json!({
            "scope": scope.as_str(),
            "asset_ids": asset_ids.iter().map(|u| u.to_string()).collect::<Vec<_>>(),
            "invite_code_id": invite_code_id.to_string(),
            "is_employer_admin": true,
        })
    }

    /// After invite accept: assign PM role, write G-11 agreement, grant asset access.
    pub async fn complete_pm_hire(
        db: &DatabaseConnection,
        accepting_user_id: Uuid,
        employer_user_id: Uuid,
        invite_code_id: Uuid,
        asset_id: Option<Uuid>,
        asset_ids_csv: Option<&str>,
    ) -> Result<Uuid, String> {
        let (tenant_id, employer_account_id) = Self::resolve_employer_account(db, employer_user_id)
            .await
            .map_err(|e| e.to_string())?;

        let asset_ids = Self::parse_invite_asset_ids(asset_id, asset_ids_csv);
        let scope = if asset_ids.is_empty() {
            DelegationScope::Portfolio
        } else {
            DelegationScope::Asset
        };

        // G-32 role (landlord book tenant).
        RbacService::assign_role(
            db,
            accepting_user_id,
            tenant_id,
            "folio",
            "property_manager",
            Some(employer_user_id),
        )
        .await
        .map_err(|e| format!("assign_role: {e}"))?;

        // Scope PM to employer's account (client book semantics within same tenant).
        db.execute(Statement::from_sql_and_values(
            DbBackend::Postgres,
            r#"UPDATE atlas_user_app_roles
                   SET client_account_id = $1
                 WHERE user_id = $2
                   AND tenant_id = $3
                   AND app_slug = 'folio'
                   AND is_active = true"#,
            [
                employer_account_id.into(),
                accepting_user_id.into(),
                tenant_id.into(),
            ],
        ))
        .await
        .map_err(|e| format!("client_account_id: {e}"))?;

        // Workspace membership so tenant resolution lands on the employer book.
        db.execute(Statement::from_sql_and_values(
            DbBackend::Postgres,
            r#"INSERT INTO user_account
                   (id, user_id, account_id, role, is_active, created_at, updated_at)
               VALUES ($1, $2, $3, 'Member', true, NOW(), NOW())
               ON CONFLICT (user_id, account_id)
               DO UPDATE SET is_active = true, updated_at = NOW()"#,
            [
                Uuid::new_v4().into(),
                accepting_user_id.into(),
                employer_account_id.into(),
            ],
        ))
        .await
        .map_err(|e| format!("user_account Member: {e}"))?;

        let contract_type = PmContractType::ManagementAgreement.to_string();
        let terms = Self::build_terms_metadata(scope, &asset_ids, invite_code_id);
        let primary_asset: Option<Uuid> = match scope {
            DelegationScope::Asset => asset_ids.first().copied(),
            DelegationScope::Portfolio => None,
        };

        let contract_id = Uuid::new_v4();
        let today = Utc::now().date_naive();
        db.execute(Statement::from_sql_and_values(
            DbBackend::Postgres,
            r#"INSERT INTO atlas_contracts
                   (id, tenant_id, contract_type, counterparty_user_id, asset_id,
                    start_date, status, terms_metadata, managed_account_id, created_at)
               VALUES
                   ($1, $2, $3, $4, $5,
                    $6, 'active', $7::jsonb, $8, now())"#,
            [
                contract_id.into(),
                tenant_id.into(),
                contract_type.into(),
                accepting_user_id.into(),
                primary_asset
                    .map(|u| sea_orm::Value::Uuid(Some(Box::new(u))))
                    .unwrap_or(sea_orm::Value::Uuid(None)),
                today.into(),
                terms.to_string().into(),
                employer_account_id.into(),
            ],
        ))
        .await
        .map_err(|e| format!("management_agreement insert: {e}"))?;

        if scope == DelegationScope::Asset {
            let profile_id = Self::property_manager_profile_id(db, tenant_id)
                .await
                .map_err(|e| e.to_string())?;
            for aid in &asset_ids {
                Self::grant_asset_access(
                    db,
                    accepting_user_id,
                    *aid,
                    profile_id,
                    Some(employer_user_id),
                )
                .await
                .map_err(|e| e.to_string())?;
            }
        }

        Ok(contract_id)
    }

    pub async fn get_manager_for_asset(
        db: &DatabaseConnection,
        employer_user_id: Uuid,
        asset_id: Uuid,
    ) -> Result<AssetManagerState, String> {
        let (tenant_id, employer_account_id) = Self::resolve_employer_account(db, employer_user_id)
            .await
            .map_err(|e| e.to_string())?;

        // Active agreement covering this asset (asset scope) or portfolio.
        if let Some(state) =
            Self::find_active_agreement(db, tenant_id, employer_account_id, asset_id).await?
        {
            return Ok(state);
        }

        // Pending invite for this asset or portfolio-scoped PM invite.
        if let Some(invite) =
            Self::find_pending_invite(db, employer_user_id, Some(asset_id)).await?
        {
            return Ok(AssetManagerState {
                status: ManagerStatus::Pending,
                scope: Some(invite.scope),
                contract_id: None,
                manager: None,
                invite: Some(invite),
            });
        }

        // Portfolio pending (no asset_id on invite).
        if let Some(invite) = Self::find_pending_invite(db, employer_user_id, None).await? {
            if invite.scope == DelegationScope::Portfolio {
                return Ok(AssetManagerState {
                    status: ManagerStatus::Pending,
                    scope: Some(DelegationScope::Portfolio),
                    contract_id: None,
                    manager: None,
                    invite: Some(invite),
                });
            }
        }

        Ok(AssetManagerState {
            status: ManagerStatus::None,
            scope: None,
            contract_id: None,
            manager: None,
            invite: None,
        })
    }

    pub async fn create_pm_invite(
        db: &DatabaseConnection,
        employer_user_id: Uuid,
        asset_id: Uuid,
        portfolio_scope: bool,
        label: Option<String>,
    ) -> Result<PendingManagerInvite, String> {
        // Block if already active/pending for this asset.
        let current = Self::get_manager_for_asset(db, employer_user_id, asset_id).await?;
        match current.status {
            ManagerStatus::Active => {
                return Err("A property manager is already delegated for this property".into());
            }
            ManagerStatus::Pending => {
                return Err("A property manager invite is already pending".into());
            }
            ManagerStatus::None => {}
        }

        let scope = if portfolio_scope {
            DelegationScope::Portfolio
        } else {
            DelegationScope::Asset
        };

        let code = format!("PM-{}", Self::generate_code_suffix());
        let invite_id = Uuid::new_v4();
        let label = label.or_else(|| Some("Property manager".into()));

        let asset_col: sea_orm::Value = if portfolio_scope {
            sea_orm::Value::Uuid(None)
        } else {
            asset_id.into()
        };

        db.execute(Statement::from_sql_and_values(
            DbBackend::Postgres,
            r#"INSERT INTO atlas_invite_codes (
                id, code, workspace_id, role,
                asset_id, employer_user_id,
                created_by, max_uses, uses_count, is_active,
                label, created_at
            ) VALUES (
                $1, $2, $3, 'property_manager',
                $4, $5,
                $5, 1, 0, true,
                $6, now()
            )"#,
            [
                invite_id.into(),
                code.clone().into(),
                employer_user_id.into(),
                asset_col,
                employer_user_id.into(),
                label.clone().into(),
            ],
        ))
        .await
        .map_err(|e| format!("create invite: {e}"))?;

        Ok(PendingManagerInvite {
            invite_id,
            code: code.clone(),
            join_url: format!("/join/{code}"),
            label,
            scope,
        })
    }

    pub async fn cancel_pending_invite(
        db: &DatabaseConnection,
        employer_user_id: Uuid,
        asset_id: Uuid,
    ) -> Result<(), String> {
        let state = Self::get_manager_for_asset(db, employer_user_id, asset_id).await?;
        let Some(invite) = state.invite else {
            return Err("No pending invite for this property".into());
        };

        db.execute(Statement::from_sql_and_values(
            DbBackend::Postgres,
            r#"UPDATE atlas_invite_codes
                   SET is_active = false
                 WHERE id = $1
                   AND created_by = $2
                   AND is_active = true"#,
            [invite.invite_id.into(), employer_user_id.into()],
        ))
        .await
        .map_err(|e| format!("cancel invite: {e}"))?;

        Ok(())
    }

    /// Revoke active delegation for this asset (or whole portfolio agreement).
    pub async fn revoke_manager_for_asset(
        db: &DatabaseConnection,
        employer_user_id: Uuid,
        asset_id: Uuid,
    ) -> Result<(), String> {
        let (tenant_id, employer_account_id) = Self::resolve_employer_account(db, employer_user_id)
            .await
            .map_err(|e| e.to_string())?;

        let row = db
            .query_one(Statement::from_sql_and_values(
                DbBackend::Postgres,
                r#"SELECT id, counterparty_user_id, terms_metadata, asset_id
                   FROM atlas_contracts
                   WHERE tenant_id = $1
                     AND contract_type = $2
                     AND status = 'active'
                     AND managed_account_id = $3
                     AND (
                       asset_id = $4
                       OR (terms_metadata->>'scope') = 'portfolio'
                       OR (terms_metadata->'asset_ids') ? $5
                     )
                   ORDER BY created_at DESC
                   LIMIT 1"#,
                [
                    tenant_id.into(),
                    PmContractType::ManagementAgreement.to_string().into(),
                    employer_account_id.into(),
                    asset_id.into(),
                    asset_id.to_string().into(),
                ],
            ))
            .await
            .map_err(|e| format!("revoke lookup: {e}"))?
            .ok_or_else(|| "No active property manager for this property".to_string())?;

        let contract_id: Uuid = row.try_get("", "id").map_err(|e| e.to_string())?;
        let pm_user_id: Uuid = row
            .try_get("", "counterparty_user_id")
            .map_err(|e| e.to_string())?;
        let terms: Option<serde_json::Value> = row.try_get("", "terms_metadata").ok().flatten();
        let scope = terms
            .as_ref()
            .and_then(|t| t.get("scope"))
            .and_then(|s| s.as_str())
            .and_then(|s| DelegationScope::try_from(s).ok())
            .unwrap_or(DelegationScope::Portfolio);

        match scope {
            DelegationScope::Portfolio => {
                Self::terminate_contract(db, contract_id).await?;
                Self::deactivate_asset_grants_for_user(db, pm_user_id).await?;
                Self::maybe_revoke_pm_role(db, tenant_id, employer_account_id, pm_user_id).await?;
            }
            DelegationScope::Asset => {
                let mut ids = terms
                    .as_ref()
                    .and_then(|t| t.get("asset_ids"))
                    .and_then(|a| a.as_array())
                    .map(|arr| {
                        arr.iter()
                            .filter_map(|v| v.as_str())
                            .filter_map(|s| Uuid::parse_str(s).ok())
                            .collect::<Vec<_>>()
                    })
                    .unwrap_or_default();
                ids.retain(|id| *id != asset_id);
                Self::deactivate_asset_grant(db, pm_user_id, asset_id).await?;

                if ids.is_empty() {
                    Self::terminate_contract(db, contract_id).await?;
                    Self::maybe_revoke_pm_role(db, tenant_id, employer_account_id, pm_user_id)
                        .await?;
                } else {
                    let mut new_terms = terms.unwrap_or_else(|| serde_json::json!({}));
                    if let Some(obj) = new_terms.as_object_mut() {
                        obj.insert(
                            "asset_ids".into(),
                            serde_json::json!(
                                ids.iter().map(|u| u.to_string()).collect::<Vec<_>>()
                            ),
                        );
                    }
                    let new_primary = ids.first().copied();
                    db.execute(Statement::from_sql_and_values(
                        DbBackend::Postgres,
                        r#"UPDATE atlas_contracts
                               SET terms_metadata = $1::jsonb,
                                   asset_id = $2
                             WHERE id = $3"#,
                        [
                            new_terms.to_string().into(),
                            new_primary
                                .map(|u| sea_orm::Value::Uuid(Some(Box::new(u))))
                                .unwrap_or(sea_orm::Value::Uuid(None)),
                            contract_id.into(),
                        ],
                    ))
                    .await
                    .map_err(|e| format!("shrink agreement: {e}"))?;
                }
            }
        }

        Ok(())
    }

    // ── Internals ─────────────────────────────────────────────────────────────

    async fn resolve_employer_account(
        db: &DatabaseConnection,
        employer_user_id: Uuid,
    ) -> Result<(Uuid, Uuid), sea_orm::DbErr> {
        let row = db
            .query_one(Statement::from_sql_and_values(
                DbBackend::Postgres,
                r#"SELECT a.tenant_id, a.id AS account_id
                   FROM user_account ua
                   JOIN account a ON ua.account_id = a.id
                   WHERE ua.user_id = $1
                     AND ua.is_active = true
                   ORDER BY ua.created_at ASC
                   LIMIT 1"#,
                [employer_user_id.into()],
            ))
            .await?
            .ok_or_else(|| {
                sea_orm::DbErr::Custom("Employer has no active account".into())
            })?;
        let tenant_id: Uuid = row.try_get("", "tenant_id")?;
        let account_id: Uuid = row.try_get("", "account_id")?;
        Ok((tenant_id, account_id))
    }

    async fn property_manager_profile_id(
        db: &DatabaseConnection,
        tenant_id: Uuid,
    ) -> Result<Uuid, sea_orm::DbErr> {
        let row = db
            .query_one(Statement::from_sql_and_values(
                DbBackend::Postgres,
                r#"SELECT id FROM atlas_role_profiles
                   WHERE app_slug = 'folio'
                     AND role_slug = 'property_manager'
                     AND (tenant_id = $1 OR is_platform_default = true)
                   ORDER BY (tenant_id = $1) DESC NULLS LAST
                   LIMIT 1"#,
                [tenant_id.into()],
            ))
            .await?
            .ok_or_else(|| {
                sea_orm::DbErr::Custom("property_manager role profile missing".into())
            })?;
        row.try_get("", "id")
    }

    async fn grant_asset_access(
        db: &DatabaseConnection,
        user_id: Uuid,
        asset_id: Uuid,
        role_profile_id: Uuid,
        granted_by: Option<Uuid>,
    ) -> Result<(), sea_orm::DbErr> {
        db.execute(Statement::from_sql_and_values(
            DbBackend::Postgres,
            r#"INSERT INTO atlas_user_asset_access
                   (id, user_id, asset_id, role_profile_id, granted_by, granted_at, is_active)
               VALUES ($1, $2, $3, $4, $5, now(), true)
               ON CONFLICT (user_id, asset_id, role_profile_id)
               DO UPDATE SET is_active = true, granted_by = EXCLUDED.granted_by,
                             granted_at = now()"#,
            [
                Uuid::new_v4().into(),
                user_id.into(),
                asset_id.into(),
                role_profile_id.into(),
                granted_by
                    .map(|u| sea_orm::Value::Uuid(Some(Box::new(u))))
                    .unwrap_or(sea_orm::Value::Uuid(None)),
            ],
        ))
        .await?;
        Ok(())
    }

    async fn find_active_agreement(
        db: &DatabaseConnection,
        tenant_id: Uuid,
        employer_account_id: Uuid,
        asset_id: Uuid,
    ) -> Result<Option<AssetManagerState>, String> {
        let row = db
            .query_one(Statement::from_sql_and_values(
                DbBackend::Postgres,
                r#"SELECT c.id, c.counterparty_user_id, c.terms_metadata,
                          u.first_name, u.last_name, u.email
                   FROM atlas_contracts c
                   JOIN "user" u ON u.id = c.counterparty_user_id
                   WHERE c.tenant_id = $1
                     AND c.contract_type = $2
                     AND c.status = 'active'
                     AND c.managed_account_id = $3
                     AND (
                       c.asset_id = $4
                       OR (c.terms_metadata->>'scope') = 'portfolio'
                       OR (c.terms_metadata->'asset_ids') ? $5
                     )
                   ORDER BY c.created_at DESC
                   LIMIT 1"#,
                [
                    tenant_id.into(),
                    PmContractType::ManagementAgreement.to_string().into(),
                    employer_account_id.into(),
                    asset_id.into(),
                    asset_id.to_string().into(),
                ],
            ))
            .await
            .map_err(|e| e.to_string())?;

        let Some(row) = row else {
            return Ok(None);
        };

        let contract_id: Uuid = row.try_get("", "id").map_err(|e| e.to_string())?;
        let pm_id: Uuid = row
            .try_get("", "counterparty_user_id")
            .map_err(|e| e.to_string())?;
        let first: String = row.try_get("", "first_name").unwrap_or_default();
        let last: String = row.try_get("", "last_name").unwrap_or_default();
        let email: String = row.try_get("", "email").unwrap_or_default();
        let terms: Option<serde_json::Value> = row.try_get("", "terms_metadata").ok().flatten();
        let scope = terms
            .as_ref()
            .and_then(|t| t.get("scope"))
            .and_then(|s| s.as_str())
            .and_then(|s| DelegationScope::try_from(s).ok())
            .unwrap_or(DelegationScope::Portfolio);
        let name = format!("{first} {last}").trim().to_string();
        let name = if name.is_empty() {
            email.clone()
        } else {
            name
        };

        Ok(Some(AssetManagerState {
            status: ManagerStatus::Active,
            scope: Some(scope),
            contract_id: Some(contract_id),
            manager: Some(ManagerPerson {
                user_id: pm_id,
                name,
                email,
            }),
            invite: None,
        }))
    }

    async fn find_pending_invite(
        db: &DatabaseConnection,
        employer_user_id: Uuid,
        asset_id: Option<Uuid>,
    ) -> Result<Option<PendingManagerInvite>, String> {
        let (sql, params): (String, Vec<sea_orm::Value>) = match asset_id {
            Some(aid) => (
                r#"SELECT id, code, label, asset_id
                   FROM atlas_invite_codes
                   WHERE created_by = $1
                     AND role = 'property_manager'
                     AND is_active = true
                     AND employer_user_id = $1
                     AND asset_id = $2
                   ORDER BY created_at DESC
                   LIMIT 1"#
                    .into(),
                vec![employer_user_id.into(), aid.into()],
            ),
            None => (
                r#"SELECT id, code, label, asset_id
                   FROM atlas_invite_codes
                   WHERE created_by = $1
                     AND role = 'property_manager'
                     AND is_active = true
                     AND employer_user_id = $1
                     AND asset_id IS NULL
                   ORDER BY created_at DESC
                   LIMIT 1"#
                    .into(),
                vec![employer_user_id.into()],
            ),
        };

        let row = db
            .query_one(Statement::from_sql_and_values(
                DbBackend::Postgres,
                &sql,
                params,
            ))
            .await
            .map_err(|e| e.to_string())?;

        let Some(row) = row else {
            return Ok(None);
        };

        let invite_id: Uuid = row.try_get("", "id").map_err(|e| e.to_string())?;
        let code: String = row.try_get("", "code").map_err(|e| e.to_string())?;
        let label: Option<String> = row.try_get("", "label").ok().flatten();
        let aid: Option<Uuid> = row.try_get("", "asset_id").ok().flatten();
        let scope = if aid.is_some() {
            DelegationScope::Asset
        } else {
            DelegationScope::Portfolio
        };

        Ok(Some(PendingManagerInvite {
            invite_id,
            code: code.clone(),
            join_url: format!("/join/{code}"),
            label,
            scope,
        }))
    }

    async fn terminate_contract(db: &DatabaseConnection, contract_id: Uuid) -> Result<(), String> {
        db.execute(Statement::from_sql_and_values(
            DbBackend::Postgres,
            r#"UPDATE atlas_contracts
                   SET status = 'terminated',
                       terminated_at = now(),
                       termination_reason = 'revoked_by_landlord'
                 WHERE id = $1"#,
            [contract_id.into()],
        ))
        .await
        .map_err(|e| format!("terminate contract: {e}"))?;
        Ok(())
    }

    async fn deactivate_asset_grant(
        db: &DatabaseConnection,
        user_id: Uuid,
        asset_id: Uuid,
    ) -> Result<(), String> {
        db.execute(Statement::from_sql_and_values(
            DbBackend::Postgres,
            r#"UPDATE atlas_user_asset_access
                   SET is_active = false
                 WHERE user_id = $1 AND asset_id = $2 AND is_active = true"#,
            [user_id.into(), asset_id.into()],
        ))
        .await
        .map_err(|e| format!("deactivate grant: {e}"))?;
        Ok(())
    }

    async fn deactivate_asset_grants_for_user(
        db: &DatabaseConnection,
        user_id: Uuid,
    ) -> Result<(), String> {
        db.execute(Statement::from_sql_and_values(
            DbBackend::Postgres,
            r#"UPDATE atlas_user_asset_access
                   SET is_active = false
                 WHERE user_id = $1 AND is_active = true"#,
            [user_id.into()],
        ))
        .await
        .map_err(|e| format!("deactivate grants: {e}"))?;
        Ok(())
    }

    async fn maybe_revoke_pm_role(
        db: &DatabaseConnection,
        tenant_id: Uuid,
        employer_account_id: Uuid,
        pm_user_id: Uuid,
    ) -> Result<(), String> {
        // Keep role if another active management agreement remains for this employer.
        let other = db
            .query_one(Statement::from_sql_and_values(
                DbBackend::Postgres,
                r#"SELECT id FROM atlas_contracts
                   WHERE tenant_id = $1
                     AND contract_type = $2
                     AND status = 'active'
                     AND managed_account_id = $3
                     AND counterparty_user_id = $4
                   LIMIT 1"#,
                [
                    tenant_id.into(),
                    PmContractType::ManagementAgreement.to_string().into(),
                    employer_account_id.into(),
                    pm_user_id.into(),
                ],
            ))
            .await
            .map_err(|e| e.to_string())?;

        if other.is_some() {
            return Ok(());
        }

        RbacService::revoke_role(db, pm_user_id, tenant_id, "folio")
            .await
            .map_err(|e| format!("revoke_role: {e}"))?;

        // Drop employer-book membership so tenant resolve no longer prefers it.
        db.execute(Statement::from_sql_and_values(
            DbBackend::Postgres,
            r#"UPDATE user_account
                   SET is_active = false, updated_at = NOW()
                 WHERE user_id = $1
                   AND account_id = $2
                   AND is_active = true"#,
            [pm_user_id.into(), employer_account_id.into()],
        ))
        .await
        .map_err(|e| format!("deactivate user_account: {e}"))?;

        Ok(())
    }

    /// Hired PM on a standard (non-PMC) Folio instance: active `property_manager`
    /// role with `client_account_id` set, and tenant `folio_mode != pmc`.
    ///
    /// Returns `(tenant_id, employer_account_id)` when the user is a hired operator.
    pub async fn hired_pm_employer_book(
        db: &DatabaseConnection,
        user_id: Uuid,
    ) -> Result<Option<(Uuid, Uuid)>, sea_orm::DbErr> {
        let row = db
            .query_one(Statement::from_sql_and_values(
                DbBackend::Postgres,
                r#"SELECT a.tenant_id, uar.client_account_id AS account_id
                   FROM atlas_user_app_roles uar
                   JOIN atlas_role_profiles rp ON rp.id = uar.role_profile_id
                   JOIN account a ON a.id = uar.client_account_id
                   LEFT JOIN atlas_app_deployment_config cfg
                     ON cfg.tenant_id = a.tenant_id
                    AND cfg.app_slug = 'property_management'
                   WHERE uar.user_id = $1
                     AND uar.app_slug = 'folio'
                     AND uar.is_active = true
                     AND (uar.expires_at IS NULL OR uar.expires_at > NOW())
                     AND rp.role_slug = 'property_manager'
                     AND uar.client_account_id IS NOT NULL
                     AND COALESCE(cfg.folio_mode, 'standard') <> 'pmc'
                   ORDER BY uar.granted_at DESC
                   LIMIT 1"#,
                [user_id.into()],
            ))
            .await?;

        let Some(row) = row else {
            return Ok(None);
        };
        let tenant_id: Uuid = row.try_get("", "tenant_id")?;
        let account_id: Uuid = row.try_get("", "account_id")?;
        Ok(Some((tenant_id, account_id)))
    }

    pub async fn is_hired_property_manager(
        db: &DatabaseConnection,
        user_id: Uuid,
    ) -> Result<bool, sea_orm::DbErr> {
        Ok(Self::hired_pm_employer_book(db, user_id).await?.is_some())
    }

    /// Asset IDs the hired PM may see. `None` = unrestricted (portfolio hire or not hired).
    /// `Some(ids)` = restrict to those assets and their descendants.
    pub async fn accessible_asset_ids(
        db: &DatabaseConnection,
        user_id: Uuid,
    ) -> Result<Option<Vec<Uuid>>, sea_orm::DbErr> {
        if Self::hired_pm_employer_book(db, user_id).await?.is_none() {
            return Ok(None);
        }

        let rows = db
            .query_all(Statement::from_sql_and_values(
                DbBackend::Postgres,
                r#"SELECT asset_id FROM atlas_user_asset_access
                   WHERE user_id = $1 AND is_active = true"#,
                [user_id.into()],
            ))
            .await?;

        if rows.is_empty() {
            // Portfolio hire — full employer book.
            return Ok(None);
        }

        let ids: Vec<Uuid> = rows
            .into_iter()
            .filter_map(|r| r.try_get::<Uuid>("", "asset_id").ok())
            .collect();
        Ok(Some(ids))
    }

    /// True when `asset_id` is in `grants` or is a descendant of a granted asset.
    pub fn asset_in_grant_scope(
        asset_id: Uuid,
        parent_asset_id: Option<Uuid>,
        grants: &[Uuid],
        parent_by_id: &std::collections::HashMap<Uuid, Option<Uuid>>,
    ) -> bool {
        if grants.contains(&asset_id) {
            return true;
        }
        let mut cursor = parent_asset_id;
        let mut guard = 0usize;
        while let Some(pid) = cursor {
            if grants.contains(&pid) {
                return true;
            }
            cursor = parent_by_id.get(&pid).copied().flatten();
            guard += 1;
            if guard > 32 {
                break;
            }
        }
        false
    }

    /// Display name for the employer (landlord) behind a hired PM's client account.
    pub async fn employer_display_name(
        db: &DatabaseConnection,
        employer_account_id: Uuid,
    ) -> Result<Option<String>, sea_orm::DbErr> {
        let row = db
            .query_one(Statement::from_sql_and_values(
                DbBackend::Postgres,
                r#"SELECT u.first_name, u.last_name, u.email
                   FROM user_account ua
                   JOIN "user" u ON u.id = ua.user_id
                   WHERE ua.account_id = $1
                     AND ua.is_active = true
                     AND ua.role = 'Owner'
                   ORDER BY ua.created_at ASC
                   LIMIT 1"#,
                [employer_account_id.into()],
            ))
            .await?;

        let Some(row) = row else {
            return Ok(None);
        };
        let first: String = row.try_get("", "first_name").unwrap_or_default();
        let last: String = row.try_get("", "last_name").unwrap_or_default();
        let email: String = row.try_get("", "email").unwrap_or_default();
        let name = format!("{first} {last}").trim().to_string();
        Ok(Some(if name.is_empty() { email } else { name }))
    }

    fn generate_code_suffix() -> String {
        use rand::Rng;
        const CHARSET: &[u8] = b"ABCDEFGHJKLMNPQRSTUVWXYZ23456789";
        let mut rng = rand::thread_rng();
        (0..6)
            .map(|_| CHARSET[rng.gen_range(0..CHARSET.len())] as char)
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_invite_assets_single_and_csv() {
        let a = Uuid::parse_str("11111111-1111-1111-1111-111111111111").unwrap();
        let b = Uuid::parse_str("22222222-2222-2222-2222-222222222222").unwrap();
        let ids = ManagementDelegationService::parse_invite_asset_ids(
            Some(a),
            Some(&format!("{b}, {a}")),
        );
        assert_eq!(ids, vec![a, b]);
    }

    #[test]
    fn parse_invite_assets_empty_is_portfolio() {
        let ids = ManagementDelegationService::parse_invite_asset_ids(None, None);
        assert!(ids.is_empty());
    }

    #[test]
    fn terms_metadata_asset_scope() {
        let a = Uuid::parse_str("11111111-1111-1111-1111-111111111111").unwrap();
        let invite = Uuid::parse_str("33333333-3333-3333-3333-333333333333").unwrap();
        let v = ManagementDelegationService::build_terms_metadata(
            DelegationScope::Asset,
            &[a],
            invite,
        );
        assert_eq!(v["scope"], "asset");
        assert_eq!(v["asset_ids"][0], a.to_string());
        assert_eq!(v["invite_code_id"], invite.to_string());
    }

    #[test]
    fn management_agreement_wire_value() {
        assert_eq!(
            PmContractType::ManagementAgreement.to_string(),
            "management_agreement"
        );
        assert!(PmContractType::try_from("property_management_agreement".to_string()).is_err());
    }

    #[test]
    fn delegation_scope_rejects_unknown() {
        assert!(DelegationScope::try_from("global").is_err());
        assert_eq!(
            DelegationScope::try_from("portfolio").unwrap(),
            DelegationScope::Portfolio
        );
    }

    #[test]
    fn asset_in_grant_scope_includes_descendants() {
        use std::collections::HashMap;
        let property = Uuid::parse_str("aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa").unwrap();
        let unit = Uuid::parse_str("bbbbbbbb-bbbb-bbbb-bbbb-bbbbbbbbbbbb").unwrap();
        let other = Uuid::parse_str("cccccccc-cccc-cccc-cccc-cccccccccccc").unwrap();
        let mut parents = HashMap::new();
        parents.insert(property, None);
        parents.insert(unit, Some(property));
        parents.insert(other, None);
        let grants = [property];
        assert!(ManagementDelegationService::asset_in_grant_scope(
            property,
            None,
            &grants,
            &parents
        ));
        assert!(ManagementDelegationService::asset_in_grant_scope(
            unit,
            Some(property),
            &grants,
            &parents
        ));
        assert!(!ManagementDelegationService::asset_in_grant_scope(
            other,
            None,
            &grants,
            &parents
        ));
    }
}
