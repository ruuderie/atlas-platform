//! Folio — Lease Service (PM wrapper over G-11 `atlas_contracts`)
//!
//! Brazilian guarantee types, condomínio split tagging, auto-renewal logic.
//!
//! # Entity field map (`atlas_contracts`)
//!   `asset_id`               → the property the lease is for
//!   `counterparty_user_id`   → the tenant (lessee)
//!   `currency`               → ISO 4217 code (not `currency_code`)
//!   `recurring_amount_cents` → monthly rent
//!   `terms_metadata`         → JSONB for guarantee type, jurisdiction, condomínio config
//!   `billing_interval`       → "monthly"
//!   `auto_renew`             → defaults false

use anyhow::{anyhow, Result};
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::types::pm::{Currency, GuaranteeType, PmContractType};

/// Lease statuses that mean someone is currently on the unit.
pub fn status_counts_as_occupied(status: &str) -> bool {
    matches!(
        status.to_ascii_lowercase().as_str(),
        "draft" | "active" | "pending"
    )
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateLeaseInput {
    pub asset_id: Uuid,
    pub counterparty_user_id: Uuid,
    pub monthly_rent_cents: i64,
    pub currency: Currency,
    pub start_date: chrono::NaiveDate,
    pub end_date: Option<chrono::NaiveDate>,
    pub auto_renew: bool,
    pub guarantee_type: GuaranteeType,
}

/// Who the lease is with — Atlas account or offline historical person.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CounterpartyKind {
    AtlasUser,
    OfflinePerson,
}

impl CounterpartyKind {
    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "atlas_user" => Some(Self::AtlasUser),
            "offline_person" => Some(Self::OfflinePerson),
            _ => None,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::AtlasUser => "atlas_user",
            Self::OfflinePerson => "offline_person",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OfflinePerson {
    pub name: String,
    pub phone: Option<String>,
    pub email: Option<String>,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateHistoricalLeaseInput {
    pub asset_id: Uuid,
    pub counterparty_kind: CounterpartyKind,
    pub counterparty_user_id: Option<Uuid>,
    pub offline_person: Option<OfflinePerson>,
    pub monthly_rent_cents: i64,
    pub currency: Currency,
    pub start_date: chrono::NaiveDate,
    pub end_date: Option<chrono::NaiveDate>,
    pub guarantee_type: GuaranteeType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateOccupancyInput {
    pub asset_id: Uuid,
    pub offline_person: OfflinePerson,
    pub start_date: Option<chrono::NaiveDate>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivateLeaseInput {
    pub monthly_rent_cents: i64,
    pub currency: Currency,
    pub guarantee_type: GuaranteeType,
    pub start_date: chrono::NaiveDate,
    pub end_date: Option<chrono::NaiveDate>,
    pub auto_renew: bool,
    pub counterparty_user_id: Option<Uuid>,
}

pub struct LeaseService;

impl LeaseService {
    /// Draft occupancy — offline person on unit without commercial terms yet.
    pub async fn create_occupancy_draft(
        db: &DatabaseConnection,
        tenant_id: Uuid,
        input: CreateOccupancyInput,
    ) -> Result<Uuid> {
        use chrono::Utc;
        use sea_orm::{ActiveModelTrait, Set};

        if input.offline_person.name.trim().is_empty() {
            anyhow::bail!("offline_person.name is required");
        }

        if Self::has_current_occupancy(db, tenant_id, input.asset_id).await? {
            anyhow::bail!("unit already has current occupancy");
        }

        let id = Uuid::new_v4();
        let now = Utc::now();
        let start = input.start_date.unwrap_or_else(|| now.date_naive());
        let terms = serde_json::json!({
            "source": "occupancy_draft",
            "counterparty_kind": CounterpartyKind::OfflinePerson.as_str(),
            "offline_person": input.offline_person,
        });

        let model = crate::entities::atlas_contract::ActiveModel {
            id: Set(id),
            tenant_id: Set(tenant_id),
            contract_type: Set(PmContractType::Lease.to_string()),
            asset_id: Set(Some(input.asset_id)),
            counterparty_user_id: Set(None),
            status: Set("draft".to_string()),
            start_date: Set(start),
            end_date: Set(None),
            auto_renew: Set(false),
            currency: Set(Currency::Usd.to_string()),
            recurring_amount_cents: Set(None),
            billing_interval: Set("monthly".to_string()),
            terms_metadata: Set(Some(terms)),
            created_at: Set(now),
            ..Default::default()
        };
        model.insert(db).await?;
        tracing::info!(
            contract_id = %id, %tenant_id,
            asset_id = %input.asset_id,
            "LeaseService: occupancy draft created"
        );
        Ok(id)
    }

    /// Activate a draft occupancy into a live lease with commercial terms.
    pub async fn activate_lease(
        db: &DatabaseConnection,
        tenant_id: Uuid,
        lease_id: Uuid,
        input: ActivateLeaseInput,
    ) -> Result<()> {
        use sea_orm::{ActiveModelTrait, Set};

        let lease = crate::entities::atlas_contract::Entity::find_by_id(lease_id)
            .filter(crate::entities::atlas_contract::Column::TenantId.eq(tenant_id))
            .one(db)
            .await?
            .ok_or_else(|| anyhow!("lease not found"))?;

        if lease.status != "draft" {
            anyhow::bail!("only draft leases can be activated");
        }

        let mut terms = lease
            .terms_metadata
            .clone()
            .unwrap_or_else(|| serde_json::json!({}));
        if let Some(obj) = terms.as_object_mut() {
            obj.insert(
                "guarantee_type".into(),
                serde_json::json!(input.guarantee_type.to_string()),
            );
            obj.insert(
                "monthly_rent_cents".into(),
                serde_json::json!(input.monthly_rent_cents),
            );
        }

        let mut am: crate::entities::atlas_contract::ActiveModel = lease.into();
        am.status = Set("active".to_string());
        am.recurring_amount_cents = Set(Some(input.monthly_rent_cents));
        am.currency = Set(input.currency.to_string());
        am.start_date = Set(input.start_date);
        am.end_date = Set(input.end_date);
        am.auto_renew = Set(input.auto_renew);
        am.terms_metadata = Set(Some(terms));
        if let Some(uid) = input.counterparty_user_id {
            am.counterparty_user_id = Set(Some(uid));
        }
        am.update(db).await?;
        Ok(())
    }

    async fn has_current_occupancy(
        db: &DatabaseConnection,
        tenant_id: Uuid,
        asset_id: Uuid,
    ) -> Result<bool> {
        let rows = crate::entities::atlas_contract::Entity::find()
            .filter(crate::entities::atlas_contract::Column::TenantId.eq(tenant_id))
            .filter(crate::entities::atlas_contract::Column::AssetId.eq(asset_id))
            .filter(
                crate::entities::atlas_contract::Column::ContractType
                    .eq(PmContractType::Lease.to_string()),
            )
            .all(db)
            .await?;
        Ok(rows.iter().any(|r| status_counts_as_occupied(&r.status)))
    }

    /// Asset IDs (and parents will be rolled up by caller) with current occupancy.
    pub async fn occupying_asset_ids(
        db: &DatabaseConnection,
        tenant_id: Uuid,
    ) -> Result<std::collections::HashSet<Uuid>> {
        let rows = crate::entities::atlas_contract::Entity::find()
            .filter(crate::entities::atlas_contract::Column::TenantId.eq(tenant_id))
            .filter(
                crate::entities::atlas_contract::Column::ContractType
                    .eq(PmContractType::Lease.to_string()),
            )
            .all(db)
            .await?;
        Ok(rows
            .into_iter()
            .filter(|r| status_counts_as_occupied(&r.status))
            .filter_map(|r| r.asset_id)
            .collect())
    }

    /// Create a lease contract in `atlas_contracts`.
    ///
    /// BR leases embed `guarantee_type` in `terms_metadata` for downstream
    /// condomínio split classification.
    pub async fn create_lease(
        db: &DatabaseConnection,
        tenant_id: Uuid,
        input: CreateLeaseInput,
    ) -> Result<Uuid> {
        use chrono::Utc;
        use sea_orm::{ActiveModelTrait, Set};

        let id = Uuid::new_v4();
        let now = Utc::now();

        let terms = serde_json::json!({
            "guarantee_type": input.guarantee_type.to_string(),
            "monthly_rent_cents": input.monthly_rent_cents,
        });

        let model = crate::entities::atlas_contract::ActiveModel {
            id: Set(id),
            tenant_id: Set(tenant_id),
            contract_type: Set(PmContractType::Lease.to_string()),
            asset_id: Set(Some(input.asset_id)),
            counterparty_user_id: Set(Some(input.counterparty_user_id)),
            status: Set("active".to_string()),
            start_date: Set(input.start_date),
            end_date: Set(input.end_date),
            auto_renew: Set(input.auto_renew),
            currency: Set(input.currency.to_string()),
            recurring_amount_cents: Set(Some(input.monthly_rent_cents)),
            billing_interval: Set("monthly".to_string()),
            terms_metadata: Set(Some(terms)),
            created_at: Set(now),
            ..Default::default()
        };
        model.insert(db).await?;

        tracing::info!(
            contract_id = %id, %tenant_id,
            asset_id = %input.asset_id,
            guarantee = %input.guarantee_type,
            "LeaseService: lease created"
        );
        Ok(id)
    }

    /// Historical / backfilled lease — may use offline person (no Atlas user_id).
    pub async fn create_historical_lease(
        db: &DatabaseConnection,
        tenant_id: Uuid,
        input: CreateHistoricalLeaseInput,
    ) -> Result<Uuid> {
        use chrono::Utc;
        use sea_orm::{ActiveModelTrait, Set};

        let (counterparty_user_id, offline) = match input.counterparty_kind {
            CounterpartyKind::AtlasUser => {
                let uid = input
                    .counterparty_user_id
                    .ok_or_else(|| anyhow::anyhow!("counterparty_user_id required for atlas_user"))?;
                (Some(uid), None)
            }
            CounterpartyKind::OfflinePerson => {
                let person = input
                    .offline_person
                    .ok_or_else(|| anyhow::anyhow!("offline_person required for offline_person"))?;
                if person.name.trim().is_empty() {
                    anyhow::bail!("offline_person.name is required");
                }
                (None, Some(person))
            }
        };

        let id = Uuid::new_v4();
        let now = Utc::now();
        let mut terms = serde_json::json!({
            "guarantee_type": input.guarantee_type.to_string(),
            "monthly_rent_cents": input.monthly_rent_cents,
            "source": "historical",
            "counterparty_kind": input.counterparty_kind.as_str(),
        });
        if let Some(person) = offline {
            if let Some(obj) = terms.as_object_mut() {
                obj.insert(
                    "offline_person".into(),
                    serde_json::to_value(person)?,
                );
            }
        }

        let model = crate::entities::atlas_contract::ActiveModel {
            id: Set(id),
            tenant_id: Set(tenant_id),
            contract_type: Set(PmContractType::Lease.to_string()),
            asset_id: Set(Some(input.asset_id)),
            counterparty_user_id: Set(counterparty_user_id),
            status: Set("terminated".to_string()), // historical = past occupancy
            start_date: Set(input.start_date),
            end_date: Set(input.end_date),
            auto_renew: Set(false),
            currency: Set(input.currency.to_string()),
            recurring_amount_cents: Set(Some(input.monthly_rent_cents)),
            billing_interval: Set("monthly".to_string()),
            terms_metadata: Set(Some(terms)),
            created_at: Set(now),
            ..Default::default()
        };
        model.insert(db).await?;

        tracing::info!(
            contract_id = %id, %tenant_id,
            asset_id = %input.asset_id,
            kind = %input.counterparty_kind.as_str(),
            "LeaseService: historical lease created"
        );
        Ok(id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn counterparty_kind_rejects_unknown() {
        assert!(CounterpartyKind::parse("ghost").is_none());
        assert_eq!(
            CounterpartyKind::parse("offline_person"),
            Some(CounterpartyKind::OfflinePerson)
        );
    }

    #[test]
    fn occupancy_statuses() {
        assert!(status_counts_as_occupied("draft"));
        assert!(status_counts_as_occupied("active"));
        assert!(status_counts_as_occupied("pending"));
        assert!(!status_counts_as_occupied("terminated"));
        assert!(!status_counts_as_occupied("expired"));
    }
}
