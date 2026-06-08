use crate::traits::atlas_app::{AtlasApp, BackgroundJob, OnboardingStep, StepCompletionCheck};
use axum::Router;
use sea_orm::DatabaseConnection;
use sea_orm_migration::MigrationTrait;
use async_trait::async_trait;
use uuid::Uuid;

// ══════════════════════════════════════════════════════════════════════════════
// FolioApp — Property Management Application
//
// "Folio" refers to the official property record identifier used in Miami-Dade
// and most real-estate jurisdictions. The name reflects the financial-ledger
// character of property management: rent rolls, NAV, condomínio splits, escrow.
//
// App domain coverage:
//   · Long-term rentals (LTR) — Miami, São Paulo, DR, Haiti, USVI
//   · Short-term rentals (STR) — Miami compliance OS, OTA revenue sync
//   · Wholesaling CRM — MAO calculator, lead pipeline, Kanban
//   · Vendor management — dispatch, G-27 contractor scorecards
//   · Multi-rail payments — Stripe, PIX/InfinitePay, BTC on-chain + Lightning
//   · G-27 scorecards — STR property, rental quality, contractor, lead quality
//
// Zero net-new tables: all data lives in platform generics G01–G18 + G27.
//
// Route namespacing:
//   /api/folio          — landlord authenticated routes
//   /api/folio/tenant   — tenant authenticated routes
//   /api/folio/str      — STR compliance routes
//   /api/folio/scorecards — G-27 scorecard routes scoped to PM entity types
//
// State Binding Contract:
//   .with_state(db) is called EXACTLY ONCE per router here.
//   Never call it inside a handler route constructor used inside this app.
// ══════
pub struct FolioApp;

#[async_trait]
impl AtlasApp for FolioApp {
    fn app_id(&self) -> &'static str {
        "property_management"
    }

    /// Public routes — Stripe webhook (HMAC-verified) + lead ingest (rate-limited).
    fn public_router(&self, db: DatabaseConnection) -> Router<DatabaseConnection> {
        Router::new()
            // ── Stripe webhook: auth via Stripe-Signature HMAC ──────────────
            .merge(crate::handlers::folio::billing::public_routes_raw())
            // ── G31 lead ingest: no session, rate-limited, honeypot-guarded ─
            .merge(crate::handlers::folio::leads::public_routes_raw())
            .with_state(db)
    }

    /// Authenticated routes — landlord, tenant, STR compliance, vault, applications, G-27.
    ///
    /// All handler modules expose state-free `*_routes_raw()` constructors.
    /// `.with_state(db)` is applied exactly once here.
    fn authenticated_router(&self, db: DatabaseConnection) -> Router<DatabaseConnection> {
        Router::new()
            // ── Landlord routes ─────────────────────────────────────────────
            .merge(crate::handlers::folio::portfolio::authenticated_routes_raw())
            .merge(crate::handlers::folio::assets::authenticated_routes_raw())
            .merge(crate::handlers::folio::leases::authenticated_routes_raw())
            .merge(crate::handlers::folio::maintenance::authenticated_routes_raw())
            .merge(crate::handlers::folio::vendors::authenticated_routes_raw())
            .merge(crate::handlers::folio::wholesale::authenticated_routes_raw())
            .merge(crate::handlers::folio::billing::authenticated_routes_raw())
            // ── STR compliance (Phase 4) ────────────────────────────────────
            .merge(crate::handlers::folio::str::authenticated_routes_raw())
            // ── Document vault (Phase 5) ────────────────────────────────────
            .merge(crate::handlers::folio::vault::authenticated_routes_raw())
            // ── Renter applications (Phase 5) ───────────────────────────────
            .merge(crate::handlers::folio::applications::authenticated_routes_raw())
            // ── STR reservations / bookings (Phase 6 — G23) ───────────────────
            .merge(crate::handlers::folio::reservations::authenticated_routes_raw())
            // ── Product catalog / pricebook (Phase 6 — G26) ───────────────────
            .merge(crate::handlers::folio::catalog::authenticated_routes_raw())
            // ── G-27 scorecard routes are registered in the core api.rs router ──
            // crate::handlers::folio::scorecards re-wraps the same handlers as
            // scorecard_entries / scorecard_analytics / scorecard_display_rules which
            // api.rs already merges directly — including it here would cause an Axum
            // "Overlapping method route" panic at startup.
            // ── G19 Campaign management ───────────────────────────────────────
            .merge(crate::handlers::folio::campaigns::authenticated_routes_raw())
            // ── G20 Attribution touchpoints ───────────────────────────────────
            .merge(crate::handlers::folio::attribution::authenticated_routes_raw())
            // ── G21 Event management, ticketing & check-in ────────────────────
            .merge(crate::handlers::folio::events::authenticated_routes_raw())
            // ── G22 Universal M:M record relationships ────────────────────────
            .merge(crate::handlers::folio::relationships::authenticated_routes_raw())
            // ── G24 Pre-purchase pricing proposals ───────────────────────────
            .merge(crate::handlers::folio::quotes::authenticated_routes_raw())
            // ── G15 Sales pipeline & deal management ─────────────────────────
            .merge(crate::handlers::folio::opportunities::authenticated_routes_raw())
            // ── G25 Commission plan application & splits ──────────────────────
            .merge(crate::handlers::folio::commission_plans::authenticated_routes_raw())
            // ── G31 PM-tier lead lifecycle ──────────────────────────────────────────
            .merge(crate::handlers::folio::leads::authenticated_routes_raw())
            // ── G01 PostGIS spatial routes ──────────────────────────────────────────
            .merge(crate::handlers::folio::geo::authenticated_routes_raw())
            .with_state(db)
    }

    /// Zero net-new migrations. All PM data lives in G01–G18 + G27 generics.
    ///
    /// Phase 0 prerequisites (template_scope, is_tenant_extension) are registered
    /// in the base migration vec in migration/mod.rs — they are platform generics,
    /// not app-specific, and must run before any AtlasApp provisions templates.
    fn migrations(&self) -> Vec<Box<dyn MigrationTrait>> {
        vec![] // Zero — by design. Rule 7 Generic Fitness Test.
    }

    fn background_jobs(&self) -> Vec<BackgroundJob> {
        vec![
            // ── pm_btc_mempool_poll — Phase 3 ─────────────────────────────
            // Polls mempool.space every 2 minutes for pending BTC tenant payments.
            // Confirms transactions, suppresses late fees on timely submissions.
            BackgroundJob {
                job_type: "pm_btc_mempool_poll".to_string(),
                default_interval_seconds: 120, // every 2 minutes
                is_active_by_default: true,
                // Config schema (tenant can override via tenant_setting):
                // { "confirmation_threshold": 1, "mempool_host": "https://mempool.space" }
                default_config_payload: Some(serde_json::json!({
                    "confirmation_threshold": 1,
                    "mempool_host": "https://mempool.space"
                })),
                executor: Box::new(|db, tenant_id, config| {
                    Box::pin(async move {
                        use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};

                        let config = config.unwrap_or_else(|| serde_json::json!({
                            "confirmation_threshold": 1,
                            "mempool_host": "https://mempool.space"
                        }));

                        let confirmation_threshold = config
                            .get("confirmation_threshold")
                            .and_then(|v| v.as_u64())
                            .unwrap_or(1) as u32;

                        let mempool_host_from_config = config
                            .get("mempool_host")
                            .and_then(|v| v.as_str())
                            .unwrap_or("https://mempool.space")
                            .to_string();

                        // Prefer mempool_host from the tenant's btc_onchain_address credential
                        // (supports self-hosted mempool.space for privacy-conscious landlords).
                        let mempool_host = {
                            let cred = crate::entities::atlas_payment_credential::Entity::find()
                                .filter(crate::entities::atlas_payment_credential::Column::TenantId.eq(tenant_id))
                                .filter(crate::entities::atlas_payment_credential::Column::CredentialType.eq("btc_onchain_address"))
                                .filter(crate::entities::atlas_payment_credential::Column::IsActive.eq(true))
                                .one(&db)
                                .await
                                .unwrap_or(None);

                            cred.and_then(|c| {
                                c.credentials_encrypted
                                    .get("mempool_host")
                                    .and_then(|v| v.as_str())
                                    .map(|s| s.to_string())
                            })
                            .unwrap_or(mempool_host_from_config)
                        };

                        // Load all BTC on-chain entries: processing + txid submitted.
                        let pending = crate::entities::atlas_ledger_entry::Entity::find()
                            .filter(crate::entities::atlas_ledger_entry::Column::TenantId.eq(tenant_id))
                            .filter(crate::entities::atlas_ledger_entry::Column::PaymentRail.eq("btc_onchain"))
                            .filter(crate::entities::atlas_ledger_entry::Column::Status.eq("processing"))
                            .filter(crate::entities::atlas_ledger_entry::Column::ExternalTxId.is_not_null())
                            .all(&db)
                            .await;

                        let entries = match pending {
                            Ok(rows) => rows,
                            Err(e) => {
                                tracing::error!(%tenant_id, "pm_btc_mempool_poll: DB error: {e:#}");
                                return Ok(()); // non-fatal — poller retries in 2m
                            }
                        };

                        if entries.is_empty() {
                            tracing::debug!(%tenant_id, "pm_btc_mempool_poll: no pending BTC entries");
                            return Ok(());
                        }

                        tracing::info!(
                            %tenant_id, count = entries.len(), %mempool_host,
                            "pm_btc_mempool_poll: polling {} pending entries", entries.len()
                        );

                        // Reuse a single adapter instance (stateless beyond mempool_host).
                        // The on-chain address is irrelevant for txid polling.
                        let rail = crate::services::pm::rails::bitcoin_onchain::BitcoinOnchainRail::with_mempool_host(
                            String::new(),
                            mempool_host,
                        );

                        for entry in entries {
                            let txid = match &entry.external_tx_id {
                                Some(t) => t.clone(),
                                None => continue,
                            };

                            match rail.poll_tx(&txid).await {
                                Some(status) if status.confirmed && status.confirmations >= confirmation_threshold => {
                                    tracing::info!(
                                        ledger_entry_id = %entry.id, %tenant_id, %txid,
                                        confirmations = status.confirmations,
                                        "pm_btc_mempool_poll: confirmed — marking paid"
                                    );
                                    if let Err(e) = crate::services::pm::ledger::PmLedgerService::mark_paid_for_tenant(
                                        &db, entry.id, tenant_id,
                                    ).await {
                                        tracing::error!(
                                            ledger_entry_id = %entry.id, %tenant_id,
                                            "pm_btc_mempool_poll: mark_paid_for_tenant failed (non-fatal): {e:#}"
                                        );
                                    }
                                }
                                Some(status) => {
                                    tracing::debug!(
                                        ledger_entry_id = %entry.id, %txid,
                                        confirmations = status.confirmations,
                                        "pm_btc_mempool_poll: awaiting confirmations"
                                    );
                                }
                                None => {
                                    tracing::warn!(
                                        ledger_entry_id = %entry.id, %txid,
                                        "pm_btc_mempool_poll: tx not yet in mempool (propagating)"
                                    );
                                }
                            }
                        }

                        Ok(())
                    })
                }),
            },
            // ── pm_str_permit_expiry_scanner — Phase 4 ────────────────────
            // Daily scan: creates compliance_violation cases for permits expiring
            // within N days. Source: atlas_regulatory_registrations.
            BackgroundJob {
                job_type: "pm_str_permit_expiry_scanner".to_string(),
                default_interval_seconds: 86400, // daily
                is_active_by_default: true,
                // Config schema:
                // { "warning_days": 30 }  — days before expiry to open a case
                default_config_payload: Some(serde_json::json!({
                    "warning_days": 30
                })),
                executor: Box::new(|db, tenant_id, config| {
                    Box::pin(async move {
                        let config = config.unwrap_or_else(|| serde_json::json!({ "warning_days": 30 }));
                        let warning_days = config
                            .get("warning_days")
                            .and_then(|v| v.as_u64())
                            .unwrap_or(30) as u32;

                        match crate::services::pm::str_compliance::StrComplianceService::scan_expiring_permits(
                            &db, tenant_id, warning_days,
                        ).await {
                            Ok(count) if count > 0 => {
                                tracing::info!(
                                    %tenant_id, cases_opened = count,
                                    "pm_str_permit_expiry_scanner: opened {} compliance_violation case(s)",
                                    count
                                );
                            }
                            Ok(_) => {
                                tracing::debug!(%tenant_id, "pm_str_permit_expiry_scanner: no expiring permits in window");
                            }
                            Err(e) => {
                                // Non-fatal: log and return Ok so poller runs tomorrow.
                                tracing::error!(
                                    %tenant_id,
                                    "pm_str_permit_expiry_scanner: scan failed (non-fatal): {e:#}"
                                );
                            }
                        }

                        Ok(())
                    })
                }),
            },
            // ── pm_ota_revenue_sync — Phase 5 ─────────────────────────────
            // Hourly: syncs Airbnb/VRBO/Booking.com revenue into atlas_tax_events.
            // Activation: enabled per-tenant when OTA integration is configured.
            // Phase 5 will wire to the external OTA integration pull API.
            BackgroundJob {
                job_type: "pm_ota_revenue_sync".to_string(),
                default_interval_seconds: 3600, // hourly
                is_active_by_default: false, // enable per-tenant when OTA integration configured
                // Config schema:
                // {
                //   "ota_integration_id": "<atlas_external_integration.id>",
                //   "lookback_hours": 25
                // }
                default_config_payload: Some(serde_json::json!({
                    "lookback_hours": 25
                })),
                executor: Box::new(|_db, tenant_id, _config| {
                    Box::pin(async move {
                        // Phase 5: pull from atlas_external_integration and fan-out per asset.
                        // Calls PmTaxService::record_ota_revenue_full() for each OTA booking event.
                        tracing::debug!(
                            %tenant_id,
                            "pm_ota_revenue_sync: Phase 5 OTA integration pull pending"
                        );
                        Ok(())
                    })
                }),
            },
            // -- pm_str_hold_expiry_sweeper -- Phase 6 -------------------------
            // Releases held STR reservations that have passed their hold_expires_at
            // deadline by transitioning them to 'hold_expired'.
            // Runs every 5 minutes; idempotent (bulk-update with status = 'hold' guard).
            BackgroundJob {
                job_type: "pm_str_hold_expiry_sweeper".to_string(),
                default_interval_seconds: 300, // every 5 minutes
                is_active_by_default: true,
                default_config_payload: None,
                executor: Box::new(|db, _tenant_id, _config| {
                    Box::pin(async move {
                        match crate::services::pm::reservation::ReservationService::expire_stale_holds(&db).await {
                            Ok(count) if count > 0 => {
                                tracing::info!(count, "pm_str_hold_expiry_sweeper: expired {} stale holds", count);
                            }
                            Ok(_) => {
                                tracing::debug!("pm_str_hold_expiry_sweeper: no stale holds");
                            }
                            Err(e) => {
                                tracing::error!("pm_str_hold_expiry_sweeper: error: {e:#}");
                            }
                        }
                        Ok(())
                    })
                }),
            },
        ]
    }

    fn onboarding_steps(&self) -> Vec<OnboardingStep> {
        vec![
            OnboardingStep {
                id: "jurisdiction".to_string(),
                title: "Jurisdiction Setup".to_string(),
                description: "Configure your operating jurisdiction (US, Brazil, USVI, DR, Haiti) to enable the correct tax, compliance, and payment rails.".to_string(),
                is_required: true,
                position: 1,
                completion_check: StepCompletionCheck::TenantSettingExists {
                    key: "folio_jurisdiction_code".to_string(),
                },
            },
            OnboardingStep {
                id: "first_property".to_string(),
                title: "Add Your First Property".to_string(),
                description: "Register your first property to start managing leases, maintenance, and payments.".to_string(),
                is_required: true,
                position: 2,
                completion_check: StepCompletionCheck::EntityCountGte {
                    table: "atlas_assets",
                    min: 1,
                },
            },
            OnboardingStep {
                id: "payment_rails".to_string(),
                title: "Payment Rails".to_string(),
                description: "Configure at least one payment method (Stripe, PIX, Bitcoin, or Zelle) so tenants can pay rent.".to_string(),
                is_required: false,
                position: 3,
                completion_check: StepCompletionCheck::TenantSettingExists {
                    key: "folio_payment_rails_configured".to_string(),
                },
            },
        ]
    }

    /// Provision a new Folio tenant with:
    ///   1. Jurisdiction setting (defaults to 'US' if not set)
    ///   2. scorecard_display_rules_enabled = true (G-27 nudges active by default)
    ///   3. Four canonical PM scorecard templates
    ///
    /// Idempotent: all inserts use ON CONFLICT DO NOTHING / WHERE NOT EXISTS.
    async fn provision(&self, db: &DatabaseConnection, tenant_id: Uuid) -> Result<(), String> {
        use sea_orm::{ConnectionTrait, Statement};
        use chrono::Utc;

        let now = Utc::now();

        // ── 1. Default jurisdiction setting ───────────────────────────────────
        db.execute(Statement::from_sql_and_values(
            sea_orm::DatabaseBackend::Postgres,
            r#"
            INSERT INTO tenant_setting (tenant_id, key, value, created_at, updated_at)
            SELECT $1, 'folio_jurisdiction_code', 'US', $2, $2
            WHERE NOT EXISTS (
                SELECT 1 FROM tenant_setting
                WHERE tenant_id = $1 AND key = 'folio_jurisdiction_code'
            )
            "#,
            vec![tenant_id.into(), now.into()],
        ))
        .await
        .map_err(|e| format!("folio provision: jurisdiction seed failed: {e}"))?;

        // ── 2. Enable G-27 scorecard display rules ────────────────────────────
        db.execute(Statement::from_sql_and_values(
            sea_orm::DatabaseBackend::Postgres,
            r#"
            INSERT INTO tenant_setting (tenant_id, key, value, created_at, updated_at)
            SELECT $1, 'scorecard_display_rules_enabled', 'true', $2, $2
            WHERE NOT EXISTS (
                SELECT 1 FROM tenant_setting
                WHERE tenant_id = $1 AND key = 'scorecard_display_rules_enabled'
            )
            "#,
            vec![tenant_id.into(), now.into()],
        ))
        .await
        .map_err(|e| format!("folio provision: scorecard_display_rules_enabled seed failed: {e}"))?;

        // ── 3. Seed 4 canonical PM scorecard templates ────────────────────────
        crate::services::pm::scorecard_provisioner::seed_pm_templates(db, tenant_id)
            .await
            .map_err(|e| format!("folio provision: template seed failed: {e}"))?;

        tracing::info!(
            "folio provision: bootstrapped tenant {} with jurisdiction, display rules, and PM scorecard templates",
            tenant_id
        );

        Ok(())
    }
}

// ══════════════════════════════════════════════════════════════════════════════
// UNIT TESTS
// ══════════════════════════════════════════════════════════════════════════════
#[cfg(test)]
mod tests {
    use super::*;
    use crate::traits::atlas_app::AtlasApp;

    #[test]
    fn test_app_id() {
        let app = FolioApp;
        assert_eq!(app.app_id(), "property_management");
    }

    #[test]
    fn test_zero_migrations() {
        let app = FolioApp;
        let migrations = app.migrations();
        assert!(
            migrations.is_empty(),
            "FolioApp must have zero migrations — all PM data lives in platform generics (Rule 7)"
        );
    }

    #[test]
    fn test_background_jobs_registered() {
        let app = FolioApp;
        let jobs = app.background_jobs();
        assert_eq!(
            jobs.len(), 4,
            "Expected 4 Folio background jobs: mempool poll, STR permit scan, OTA sync, STR hold expiry sweeper"
        );

        let job_types: Vec<&str> = jobs.iter().map(|j| j.job_type.as_str()).collect();
        assert!(job_types.contains(&"pm_btc_mempool_poll"));
        assert!(job_types.contains(&"pm_str_permit_expiry_scanner"));
        assert!(job_types.contains(&"pm_ota_revenue_sync"));
        assert!(job_types.contains(&"pm_str_hold_expiry_sweeper"));

        // pm_btc_mempool_poll must have a documented config schema and correct defaults.
        let mempool_job = jobs.iter().find(|j| j.job_type == "pm_btc_mempool_poll").unwrap();
        let config = mempool_job.default_config_payload.as_ref()
            .expect("pm_btc_mempool_poll must have a default_config_payload");
        assert_eq!(config["confirmation_threshold"], 1);
        assert_eq!(config["mempool_host"], "https://mempool.space");
        assert_eq!(mempool_job.default_interval_seconds, 120);
        assert!(mempool_job.is_active_by_default);
    }

    #[test]
    fn test_onboarding_steps() {
        let app = FolioApp;
        let steps = app.onboarding_steps();
        assert_eq!(steps.len(), 3);

        // Jurisdiction is required and comes first
        assert_eq!(steps[0].id, "jurisdiction");
        assert!(steps[0].is_required);

        // First property is required
        assert_eq!(steps[1].id, "first_property");
        assert!(steps[1].is_required);

        // Payment rails are optional
        assert_eq!(steps[2].id, "payment_rails");
        assert!(!steps[2].is_required);
    }

    #[test]
    fn test_positions_are_unique_and_ordered() {
        let app = FolioApp;
        let steps = app.onboarding_steps();
        let positions: Vec<u8> = steps.iter().map(|s| s.position).collect();
        let mut sorted = positions.clone();
        sorted.sort();
        sorted.dedup();
        assert_eq!(positions, sorted, "Onboarding step positions must be unique and ascending");
    }
}
