//! Folio — STR Compliance Service (PM wrapper over G-16 `atlas_regulatory_registrations`)
//!
//! Miami STR zoning, permit registration, expiry scanning, OTA revenue sync.
//!
//! # Entity field map (`atlas_regulatory_registrations`)
//!   `asset_id`              → direct FK to the property (no entity_type/entity_id)
//!   `jurisdiction_code`     → SubJurisdiction label e.g. "US-FL-MIAMI-DADE"
//!   `registration_number`   → required String (not Option)
//!   `expires_at`            → NaiveDate (not DateTime<Utc>)
//!   `jurisdiction_metadata` → JSONB for permit_category etc. (not registration_metadata)
//!   No `updated_at` column on this entity
//!
//! # Phase 4: `scan_expiring_permits()`
//!
//! Called daily by the `pm_str_permit_expiry_scanner` background job.
//! Opens one `atlas_cases` row per expiring permit with:
//!   - `case_type = "compliance_violation"`
//!   - `subject = "STR permit expiring in N days: {permit_number}"`
//!   - `case_metadata = { "permit_id", "expires_at", "permit_category", "jurisdiction_code" }`
//!   - `priority = "high"` if expiry within 7 days, else `"medium"`
//!
//! Idempotent: skips permits that already have an open compliance_violation case
//! (checked via `case_metadata->>'permit_id'` in `atlas_cases`).

use anyhow::Result;
use chrono::NaiveDate;
use sea_orm::DatabaseConnection;
use uuid::Uuid;

use crate::types::pm::{PmRegistrationType, StrPermitCategory};

pub struct StrComplianceService;

impl StrComplianceService {
    /// Register an STR operating permit in `atlas_regulatory_registrations`.
    pub async fn register_permit(
        db: &DatabaseConnection,
        tenant_id: Uuid,
        asset_id: Uuid,
        permit_number: &str,
        category: StrPermitCategory,
        expires_at: NaiveDate,
        jurisdiction_code: &str,
    ) -> Result<Uuid> {
        use chrono::Utc;
        use sea_orm::{ActiveModelTrait, Set};

        let id = Uuid::new_v4();
        let now = Utc::now();

        let metadata = serde_json::json!({
            "permit_category": category.to_string(),
        });

        let model = crate::entities::atlas_regulatory_registration::ActiveModel {
            id: Set(id),
            tenant_id: Set(tenant_id),
            asset_id: Set(Some(asset_id)),
            registration_type: Set(PmRegistrationType::StrPermit.to_string()),
            jurisdiction_code: Set(jurisdiction_code.to_string()),
            registration_number: Set(permit_number.to_string()),
            status: Set("active".to_string()),
            expires_at: Set(Some(expires_at)),
            jurisdiction_metadata: Set(Some(metadata)),
            created_at: Set(now),
            ..Default::default()
        };
        model.insert(db).await?;

        tracing::info!(
            permit_id = %id, %tenant_id, %asset_id,
            permit_category = %category,
            %jurisdiction_code,
            "StrComplianceService: STR permit registered"
        );
        Ok(id)
    }

    /// Scan for permits expiring within `warning_days` and open compliance_violation cases.
    ///
    /// Called daily by `pm_str_permit_expiry_scanner`.
    /// `warning_days` should come from `market.str_regulation().expiry_warning_days()`.
    ///
    /// Returns the number of new compliance_violation cases opened.
    ///
    /// # Idempotency
    /// Uses a raw SQL `NOT EXISTS` guard scoped to `permit_id` in `case_metadata`.
    /// Multiple daily runs will not create duplicate cases.
    pub async fn scan_expiring_permits(
        db: &DatabaseConnection,
        tenant_id: Uuid,
        warning_days: u32,
    ) -> Result<u32> {
        use chrono::{Duration, Utc};
        use sea_orm::{ActiveModelTrait, Set};
        use sea_orm::{ColumnTrait, ConnectionTrait, EntityTrait, QueryFilter, Statement};

        let today = Utc::now().date_naive();
        let cutoff = today + Duration::try_days(warning_days as i64).unwrap_or_default();

        // Load permits expiring within warning_days that are still active.
        // Filter: expires_at IS NOT NULL AND expires_at BETWEEN today AND cutoff AND status = 'active'
        let permits = crate::entities::atlas_regulatory_registration::Entity::find()
            .filter(crate::entities::atlas_regulatory_registration::Column::TenantId.eq(tenant_id))
            .filter(
                crate::entities::atlas_regulatory_registration::Column::RegistrationType
                    .eq(PmRegistrationType::StrPermit.to_string()),
            )
            .filter(crate::entities::atlas_regulatory_registration::Column::Status.eq("active"))
            .all(db)
            .await?
            .into_iter()
            // In-memory filter for expires_at range (avoids sea-orm date range query complexity)
            .filter(|p| {
                p.expires_at
                    .map(|exp| exp >= today && exp <= cutoff)
                    .unwrap_or(false)
            })
            .collect::<Vec<_>>();

        if permits.is_empty() {
            tracing::debug!(%tenant_id, warning_days, "StrComplianceService: no expiring permits in window");
            return Ok(0);
        }

        tracing::info!(
            %tenant_id, count = permits.len(), warning_days,
            "StrComplianceService: found {} permits expiring within {} days",
            permits.len(), warning_days
        );

        let mut cases_opened: u32 = 0;
        let now = Utc::now();

        for permit in &permits {
            // Idempotency guard: skip if an open compliance_violation case already exists for this permit.
            // Uses raw SQL because sea-orm JSONB containment is not ergonomic.
            let already_open: bool = db
                .query_one(Statement::from_sql_and_values(
                    sea_orm::DatabaseBackend::Postgres,
                    r#"
                SELECT 1
                FROM atlas_cases
                WHERE tenant_id = $1
                  AND case_type = 'compliance_violation'
                  AND status NOT IN ('resolved', 'closed', 'cancelled')
                  AND case_metadata->>'permit_id' = $2
                LIMIT 1
                "#,
                    vec![tenant_id.into(), permit.id.to_string().into()],
                ))
                .await
                .unwrap_or(None)
                .is_some();

            if already_open {
                tracing::debug!(
                    permit_id = %permit.id,
                    registration_number = %permit.registration_number,
                    "StrComplianceService: compliance_violation already open — skipping"
                );
                continue;
            }

            let expires_at = permit.expires_at.unwrap(); // safe: filtered above
            let days_until_expiry = (expires_at - today).num_days();
            let priority = if days_until_expiry <= 7 {
                "high"
            } else {
                "medium"
            }
            .to_string();

            let subject = format!(
                "STR permit expiring in {} day{}: {}",
                days_until_expiry,
                if days_until_expiry == 1 { "" } else { "s" },
                permit.registration_number,
            );

            let description = format!(
                "STR permit {} (jurisdiction: {}) expires on {}. \
                Renew before expiry to avoid fines under Miami-Dade STR Ordinance 2023-89.",
                permit.registration_number, permit.jurisdiction_code, expires_at,
            );

            let case_metadata = serde_json::json!({
                "permit_id": permit.id.to_string(),
                "permit_number": permit.registration_number,
                "expires_at": expires_at.to_string(),
                "jurisdiction_code": permit.jurisdiction_code,
                "permit_category": permit.jurisdiction_metadata
                    .as_ref()
                    .and_then(|m| m.get("permit_category"))
                    .and_then(|v| v.as_str())
                    .unwrap_or("unknown"),
                "days_until_expiry": days_until_expiry,
            });

            let case_model = crate::entities::atlas_case::ActiveModel {
                id: Set(Uuid::new_v4()),
                tenant_id: Set(tenant_id),
                case_type: Set("compliance_violation".to_string()),
                asset_id: Set(permit.asset_id),
                priority: Set(priority),
                status: Set("open".to_string()),
                subject: Set(subject.clone()),
                description: Set(Some(description)),
                case_metadata: Set(Some(case_metadata)),
                created_at: Set(now),
                // All nullable fields default to None
                reported_by_user_id: Set(None),
                contract_id: Set(None),
                assigned_service_provider_id: Set(None),
                assigned_user_id: Set(None),
                scheduled_at: Set(None),
                completed_at: Set(None),
                estimated_cost_cents: Set(None),
                actual_cost_cents: Set(None),
                ledger_entry_id: Set(None),
                primary_attachment_id: Set(None),
                ws_room_id: Set(None),
            };

            match case_model.insert(db).await {
                Ok(case) => {
                    tracing::info!(
                        case_id = %case.id,
                        permit_id = %permit.id,
                        %tenant_id,
                        %subject,
                        "StrComplianceService: compliance_violation case opened"
                    );
                    cases_opened += 1;
                }
                Err(e) => {
                    tracing::error!(
                        permit_id = %permit.id,
                        %tenant_id,
                        "StrComplianceService: failed to open case (non-fatal): {e:#}"
                    );
                }
            }
        }

        Ok(cases_opened)
    }
}
