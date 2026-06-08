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
use uuid::Uuid;
use serde::{Deserialize, Serialize};

use crate::types::pm::{PmContractType, GuaranteeType, Currency};

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
        use sea_orm::{Set, ActiveModelTrait};
        use chrono::Utc;

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
}
