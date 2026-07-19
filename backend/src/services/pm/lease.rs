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

use anyhow::Result;
use sea_orm::DatabaseConnection;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::types::pm::{Currency, GuaranteeType, PmContractType};

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

pub struct LeaseService;

impl LeaseService {
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
}
