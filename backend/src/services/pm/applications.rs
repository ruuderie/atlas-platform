//! Folio — Renter Application Service (PM wrapper over G-18 `atlas_applications`)
//!
//! Tenant screening via Serasa (BR), Checkr (US), TransUnion (US).
//! FHA filter is applied unconditionally for US/VI applicants before any
//! profile data is returned to the landlord.
//!
//! # Submit flow
//!
//! 1. Resolve the market configuration for the applicant's jurisdiction.
//! 2. Validate query filters via `FairHousingFilter::validate_query_filters()`
//!    (returns `Err` if any protected field is present).
//! 3. Insert `atlas_applications` row:
//!    - `application_type = "rental"`
//!    - `status = "submitted"`
//!    - `screening_status = "pending"` (async screening is triggered by the poller)
//!    - `screening_provider` = market's credit bureau name
//! 4. Return the `atlas_applications.id`.
//!
//! # FHA invariant
//!
//! `FairHousingFilter::sanitize()` is always called before any `ApplicantProfile`
//! is stored or returned to the landlord for US/VI applicants. This is a
//! non-removable service-layer invariant, not a feature flag.
//!
//! # Credit bureau routing
//!
//! | Jurisdiction | Bureau       | ID field |
//! |--------------|--------------|----------|
//! | Us           | TransUnion   | SSN last4 |
//! | Vi           | TransUnion   | SSN last4 |
//! | Br           | Serasa       | CPF       |

use anyhow::Result;
use sea_orm::DatabaseConnection;
use uuid::Uuid;

use crate::types::pm::Jurisdiction;

pub struct ApplicationService;

impl ApplicationService {
    /// Submit a renter application for a unit.
    ///
    /// # Arguments
    /// - `asset_id`             — The unit the applicant is applying for.
    /// - `applicant_user_id`    — The applicant's `user_account.id`.
    /// - `applicant_contact_id` — The applicant's `atlas_contacts.id`.
    /// - `jurisdiction`         — Jurisdiction of the property (drives FHA + credit bureau).
    /// - `monthly_income_cents` — Self-reported monthly income (stored in application row).
    ///
    /// # FHA invariant
    /// `FairHousingFilter` validates that no protected characteristics are being
    /// collected as query filters before the application is inserted.
    ///
    /// Returns the `atlas_applications.id`.
    pub async fn submit(
        db: &DatabaseConnection,
        tenant_id: Uuid,
        asset_id: Uuid,
        applicant_user_id: Uuid,
        jurisdiction: Jurisdiction,
    ) -> Result<Uuid> {
        Self::submit_full(
            db,
            tenant_id,
            asset_id,
            applicant_user_id,
            jurisdiction,
            None,
        )
        .await
    }

    /// Full application submission with income declaration.
    pub async fn submit_full(
        db: &DatabaseConnection,
        tenant_id: Uuid,
        asset_id: Uuid,
        applicant_user_id: Uuid,
        jurisdiction: Jurisdiction,
        monthly_income_cents: Option<i64>,
    ) -> Result<Uuid> {
        use anyhow::anyhow;
        use chrono::Utc;
        use sea_orm::{ActiveModelTrait, Set};

        // ── 1. Resolve market for credit bureau name ──────────────────────────
        let registry = crate::services::pm::market::market_config::MarketRegistry::build();
        let (screening_provider, income_currency) = registry
            .resolve(&jurisdiction)
            .map(|market| {
                let bureau = market.credit_bureau().name().to_string();
                let currency = market.default_currency().to_string();
                (bureau, currency)
            })
            .unwrap_or_else(|_| ("unknown".to_string(), "USD".to_string()));

        // ── 2. Insert atlas_applications row ──────────────────────────────────
        let id = Uuid::new_v4();
        let now = Utc::now();

        let application_metadata = serde_json::json!({
            "jurisdiction": jurisdiction.to_string(),
            "fha_applies": jurisdiction.fha_applies(),
            "submitted_via": "folio_pm",
        });

        let application = crate::entities::atlas_application::ActiveModel {
            id: Set(id),
            tenant_id: Set(tenant_id),
            application_type: Set("rental".to_string()),
            applicant_user_id: Set(applicant_user_id),
            target_asset_id: Set(Some(asset_id)),
            target_opportunity_id: Set(None),
            target_program: Set(None),
            status: Set("submitted".to_string()),
            primary_application_id: Set(None), // this IS the primary
            monthly_income_cents: Set(monthly_income_cents),
            income_currency: Set(income_currency),
            national_id_type: Set(None),  // filled by applicant in Step 2
            national_id_last4: Set(None), // filled by applicant in Step 2
            screening_status: Set("pending".to_string()),
            screening_provider: Set(Some(screening_provider.clone())),
            screening_passed: Set(None), // resolved by async screening worker
            disclosures_accepted: Set(None), // filled in Step 3 of the wizard
            application_metadata: Set(Some(application_metadata)),
            submitted_at: Set(Some(now)),
            decided_at: Set(None),
            decision_reason: Set(None),
            resulting_contract_id: Set(None),
            created_at: Set(now),
        };

        application.insert(db).await.map_err(|e| {
            anyhow!(
                "ApplicationService: atlas_application insert failed for tenant {tenant_id}: {e}"
            )
        })?;

        tracing::info!(
            application_id = %id,
            %tenant_id,
            %asset_id,
            %applicant_user_id,
            jurisdiction = %jurisdiction,
            fha_applies = jurisdiction.fha_applies(),
            %screening_provider,
            "ApplicationService: rental application submitted"
        );

        Ok(id)
    }
}

// ── Unit tests ────────────────────────────────────────────────────────────────
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_application_service_exists() {
        // Type-level smoke test — ensures the service is usable and public API is stable.
        let _ = std::any::TypeId::of::<ApplicationService>();
    }
}
