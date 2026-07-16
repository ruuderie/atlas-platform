# Atlas Platform — Current State (July 14, 2026 — Rev 13)

**This is the single most important document for any new AI or developer to read first.**

It provides a complete, up-to-date snapshot of the platform architecture after the **Folio Deal Ops (Wholesaling + Creative Finance)** work (Rev 13), building on the **Famtasm Creator Paywall Platform Specification + UI Specs** work (Rev 12).

---

> [!IMPORTANT]
> **Launching a new product app requires Phase 1 AND Phase 2.** Phase 1 (infra/CI) gets the pod running. Phase 2 (DB registration) makes the homepage visible. Skipping Phase 2 causes a 404 on the marketing page even when the pod is healthy — this was the root cause of the folio 404 (July 2026).
>
> - Full checklist: [`docs/architecture/adding_a_new_app.md`](architecture/adding_a_new_app.md)
> - Content resolution algorithm: [`docs/architecture/product_page_system.md`](architecture/product_page_system.md)

---

> [!NOTE]
> **Backlog & future work** is tracked separately in [`docs/backlog/README.md`](backlog/README.md). That file lists known gaps, outstanding platform tasks, and exploratory specs that are NOT yet in the schema or codebase. Read it before planning any new feature — the item you want may already be scoped there.

> [!IMPORTANT]
> **Rule 7 — Generic Fitness Test is mandatory** before any net-new table or new G-number.
> Canonical standard: [`docs/architecture/generic_fitness_test.md`](architecture/generic_fitness_test.md)
> (decision diagram: [`architecture/diagrams/generic_fitness_test_flow.mmd`](architecture/diagrams/generic_fitness_test_flow.mmd)).
> This file is the **registry** of existing generics (what is built), not the fitness procedure.

---

## Executive Summary

### What Changed Since Rev 12 (Rev 13)

- **Folio Deal Ops** ✅ **NEW** — Unified Wholesaling + Creative Finance over G-15 (no new G-number). `DealTrack`, expanded `WholesaleStage`, CF acquire/dispose stages, `AcquisitionStructure`, `ExitMode`, `BuyerFit`, extended `PmContractType` / `PmOpportunityType`. Service: `services/pm/deal_ops.rs`. API: `/api/folio/deals*`. Folio: `/l/deals`, `/l/deals/:id`, `/l/deals/:id/structure`, `/l/buyers`, `/t/option`. Wholesale list DTO drift fixed. Stitch: `_deal_ops/README.md` + assignment/territory previews.
- CYA gate on CF convert; title gate on wholesale assignment; lease-option install + convert-to-CF contingency.

### What Changed Since Rev 11 (Rev 12)

- **Famtasm Creator Paywall Platform** ✅ **NEW** — Full product & technical specification (Rev 2, coverage metric raised from 51% → 71%), Atlas integration mapping (Rev 2), and 12-page UI specification suite covering: marketing landing page, creator onboarding wizard (5-step), public creator hub, creator workbench (with video upload sub-flow), secure video player + paywall gate, fan subscription checkout, PPV unlock flow, creator subscriber management, creator earnings dashboard, creator hub settings, fan profile + subscription management, and platform admin ops dashboard.
- **Famtasm Atlas Integration gaps identified:** 3 gaps require build: (1) Cloudflare Stream signed HLS service (`backend/src/services/media/cloudflare_stream.rs`), (2) Creator Workbench Leptos portal (`apps/folio/src/pages/creator/`), (3) B2C subscription webhook handler + `SyncCreatorSubscriptions` OutboxJob.
- **Famtasm pending migrations (5 new):** `m20261102_famtasm_creator_portals`, `m20261103_famtasm_subscription_tiers`, `m20261104_famtasm_subscriber_registry`, `m20261105_famtasm_video_assets`, `m20261106_famtasm_ppv_purchases`.
- **docs/famtasm/** — Updated `atlas_integration_mapping.md` (Rev 2) + `product_technical_specification.md` (Rev 2) + new `ui-spec/` directory with 12 page specifications.

### What Changed Since Rev 10

- **G-06 Verification end-to-end** ✅ **NEW** — `m20261028_g06_verification_reviewer_notes`; create/list APIs; Folio landlord submit (`account_billing.rs`); platform-admin reviewer notes + request-more-info + vault docs + approve side-effects. Frontend: **Partial → Full UI** (admin queue + Folio submit path).
- **Feature flags per-instance enablements** ✅ **NEW** — `m20261027_atlas_flag_instance_enablements` + `atlas_flag_instance_enablement` entity + `FlagService` + Folio `/api/folio/flags` + platform-admin `instance_features_panel.rs` (Features tab on internal instances). Catalog `/flags` remains; instance Features controls enablements.
- **Ops section revamp (platform-admin)** ✅ **NEW** — Product cards link to detail; Support internal notes + operator reply notifications (hide notes from Folio users); Audit log date/actor filters + CSV export; AI tasks `process_ai_tasks` worker + pause/resume (remove fake queue controls); G-27 scorecard-push from instance Feedback tab.
- **Landlord onboarding stitch parity** ✅ **NEW** — `WizardShell` + `landlord_wizard.rs` aligned to `wiz_landlord_onboard`; per-step left-rail copy; magic-link session gate via `peek_auth_session()` (skips OTP when `/api/folio/me` is 403 but cookie is valid).
- **G-37 Ambassadors + F&F vendors** ✅ (landed on `dev`) — `m20261030_campaign_global_name_ff_vendors_g37_ambassadors`; `atlas_ambassador` / `atlas_ambassador_campaign` entities; platform-admin `/ambassadors`; Folio `/refer` + `/refer/vendors` + landlord `referrals.rs`.
- **G-19 F&F campaign seed** ✅ — `m20261029_friends_family_referral_campaign` (landlord) + vendors child via m20261030 `global_name`.
- **G-36 Programs UI** ✅ — platform-admin `/programs` + `instance_programs_panel`; Folio `NetworkInvitePanel` on wizards/dashboards.
- **G-27 template deployments + session instance** ✅ — `m20261016_*`, `m20261017_*` (already noted late Rev 10; confirmed wired).
- **Product plans + Folio marketing seeds** ✅ — `m20261022_platform_product_plans`, `m20261023–26` hero/section/founding-beta/vendor-trade seeds.
- **Migration count** — **234** registered migration files (up from ~180+ in Rev 10). Latest: `m20261030_*`.
- **Folio pages** — **144** page modules (landlord **35** incl. `referrals.rs`; marketing **19**; onboarding **15** wizards).
- **platform-admin** — **113** page modules; new `instance_features_panel`, `instance_programs_panel`, `marketing/ambassadors`.
- **shared-ui** — still **85** UI primitives; `configurator.rs` present; scorecard widgets + auth + 25+ hooks.
- **Test suite** — still **43** test files (27 integration + 16 unit).

### What Changed Since Rev 9 (preserved for history)

- Property Owner Portal, onboarding wizard suite, booking/OTP/invite codes, product seeding, platform-admin session caching — see Rev 10 executive summary below (abbreviated).

### What Changed Since Rev 8–9 (preserved for history)

- Waitlist confirmation email, STR Host Portal, G-35/G-14/G-34 partial UI, PM rails — see prior revisions.


## Platform Generics Registry (Current — G-01 through G-37+)

### Infrastructure Layer (G01–G08) — All Deployed ✅

| ID | Name | Tables | Service | Backend Status | Frontend Status |
|----|------|--------|---------|----------------|-----------------|
| G01 | `atlas_geo` | `geo_service_areas`, `atlas_geo` | `GeoService` | Deployed with API | Partial UI (map/index.rs in platform-admin + admin settings) |
| G02 | `atlas_vault` | `attachment`, `attachment_share_tokens`, `attachment_multipart_uploads` | `VaultService` + `pm/vault.rs` | Deployed with API | Partial UI (file_attachments component + `landlord/digital_vault.rs` in folio NEW) |
| G03 | `atlas_payments` | `atlas_ledger_entries`, `atlas_ledger_splits`, `atlas_payment_credentials` | `LedgerService` + `pm/ledger.rs` + `pm/payment_rail.rs` | Deployed with API | **Full UI** (billing/tenant ledger in platform-admin; landlord billing + tenant payments + vendor invoices + landlord ledger pages in folio) |
| G04 | `atlas_subscriptions` | `atlas_subscriptions` | `SubscriptionService` | Deployed with API | Partial UI (billing dashboard) |
| G05 | `atlas_syndication` | `atlas_syndication_outbox`, `atlas_integration_events`, `atlas_syndication_offer`, `atlas_app_instance_syndication`, `atlas_external_integrations` | `ExternalIntegrationService` + `SyndicationEventBus` | **Deployed with API** | **Full UI** (platform-admin syndication offers + links pages, per-instance syndication panel; `landlord/syndication.rs` + `str_host/syndication.rs` in folio NEW) |
| G06 | `atlas_verification_queue` | `atlas_verification_requests` (+ reviewer notes) | `VerificationService` | Deployed with API | **Full UI** (platform-admin `verification/index.rs` + Folio landlord submit via `account_billing.rs`; vault docs + approve side-effects) |
| G07 | `atlas_realtime` | `atlas_ws_rooms`, `atlas_ws_messages` | `RealtimeService` | Deployed with API | No UI |
| G08 | `atlas_ai_tasks` | `atlas_ai_tasks` | `AITaskService` | Deployed with API | **Full UI** (platform-admin `admin/ai_tasks.rs` — live `process_ai_tasks` worker + pause/resume; audit instrumentation) |


> **Note on G-05**: G-05 scope now spans the full outbound syndication pipeline: offer registry, per-instance linking, transactional outbox, exponential back-off retry, HMAC-signed webhook delivery, and integration event audit log. The STR host portal adds a dedicated `str_host/syndication.rs` page for channel management.

### Domain Object Layer (G09–G18) — All Deployed ✅

| ID | Name | Tables | Service | Backend Status | Frontend Status |
|----|------|--------|---------|----------------|-----------------|
| G09 | `atlas_portfolios` | `atlas_portfolios` | `PortfolioService` + `pm/portfolio.rs` | Deployed with API | **Full UI** (landlord portfolio page + `map_portfolio.rs` in folio; PMC `portfolio_map.rs`) |
| G10 | `atlas_assets` | `atlas_assets` | `AssetService` + `pm/asset.rs` | Deployed with API | **Full UI** (landlord assets + `asset_detail.rs` + `asset_alerts.rs` in folio; listing_mode + asset_lifecycle fields) |
| G11 | `atlas_contracts` | `atlas_contracts` | `ContractService` + `pm/lease.rs` | Deployed with API | **Full UI** (landlord leases + `lease_detail.rs` + tenant my_lease pages in folio) |
| G12 | `atlas_service_providers` | `atlas_service_providers` | `ServiceProviderService` + `pm/vendor.rs` | Deployed with API | **Full UI** (landlord vendors page + `contractor_marketplace.rs` in folio; vendor `network_profile.rs` NEW) |
| G13 | `atlas_cases` | `atlas_cases` | `CaseService` + `pm/maintenance.rs` | Deployed with API | **Full UI** (tenant maintenance + `maintenance_detail.rs` + `maintenance_triage.rs` + vendor work_orders + `landlord/maintenance_queue.rs` in folio NEW; str_host `incidents.rs`) |
| G14 | `atlas_documents` | `atlas_documents` | `DocumentService` | Deployed with API | **Partial UI** (`tenant/documents.rs` + `landlord/digital_vault.rs` in folio NEW) |
| G15 | `atlas_opportunities` | `atlas_opportunities` | `OpportunityService` + `pm/opportunity.rs` | Deployed with API | Partial UI (folio handler) |
| G16 | `atlas_regulatory_registrations` | `atlas_regulatory_registrations` | `RegulatoryRegistrationService` + `pm/str_compliance.rs` | Deployed with API | **Full UI** (landlord str_compliance page in folio; str_host `violation_file.rs`) |
| G17 | `atlas_tax_events` + `atlas_tax_filings` | `atlas_tax_events`, `atlas_tax_filings` | `TaxService` + `pm/tax.rs` | Deployed with API | Partial UI (`landlord/ledger.rs` surfaces tax events; `owner/statements.rs` NEW) |
| G18 | `atlas_applications` | `atlas_applications` | `ApplicationService` + `pm/applications.rs` | Deployed with API | Partial UI (`tenant/application_status.rs` NEW in folio + `marketing/renter_application.rs` public page) |

### Round 1 Additions (G19–G26) — All Deployed ✅

| ID | Name | Tables | Service | Backend Status | Frontend Status |
|----|------|--------|---------|----------------|-----------------|
| G19 | `atlas_campaigns` | `atlas_campaigns` (+ `global_name` UNIQUE), `atlas_campaign_enrollments`, `atlas_campaign_events`, **`atlas_campaign_mail_drops`**, **`atlas_campaign_offer_codes`** | `pm/campaign.rs` + `pm/campaign_dm.rs` + `pm/direct_mail.rs` | Deployed with API | **Full UI** (landlord campaigns + platform-admin campaigns incl. DM drops/spend/attribution; Stitch `_direct_mail/`) |
| G20 | `atlas_attribution` | `atlas_attribution_touchpoints` | `pm/attribution.rs` + waitlist/LP hooks | Deployed with API | **Partial UI** (campaign-scoped attribution panel in platform-admin; global dashboard still backlog) |
| G21 | `atlas_events` | `atlas_events`, `atlas_event_registrations`, `atlas_event_ticket_types` | `pm/event.rs` | Deployed with API | No UI |
| G22 | `atlas_record_relationships` | `atlas_record_relationships` | `pm/record_relationship.rs` | Deployed with API | No UI |
| G23 | `atlas_reservations` | `atlas_reservations`, `atlas_availability` | `ReservationService` + `pm/reservation.rs` | Deployed with API | **Full UI** (landlord + tenant + str_host reservations pages in folio NEW) |
| G24 | `atlas_quotes` | `atlas_quotes`, `atlas_quote_line_items` | `pm/quote.rs` | Deployed with API | No UI |
| G25 | `atlas_commission_plans` | `atlas_commission_plans`, `atlas_commission_plan_splits` | `pm/commission.rs` | Deployed with API | No UI |
| G26 | `atlas_catalog` | `atlas_catalog_entries`, `atlas_catalog_availability`, `atlas_catalog_rate_rules` | `pm/catalog.rs` | Deployed with API | **Full UI** (landlord catalog page + str_host pricing in folio NEW) |

### Round 2 Additions (G27–G31) — Fully Implemented ✅

| ID | Name | Tables | Service | Backend Status | Frontend Status |
|----|------|--------|---------|----------------|-----------------|
| G27 | `atlas_scorecards` | `atlas_scorecard_templates`, `atlas_scorecard_dimensions`, `atlas_scorecard_dimension_options`, `atlas_scorecards`, `atlas_rating_sessions` (+ `app_instance_id`), `atlas_scorecard_entries`, `atlas_scorecard_dimension_aggregates`, `atlas_scorecard_poll_aggregates`, `atlas_scorecard_time_series`, `atlas_scorecard_targets`, `atlas_scorecard_target_criteria`, `atlas_scorecard_display_rules`, `atlas_scorecard_contributor_calibrations`, `atlas_scorecard_template_deployments` | `ScorecardService` + `ScorecardAnalyticsService` + `scorecard_triggers` | **Deployed with API** (`scorecard_admin.rs`, `scorecard_entries.rs` — deployed-only list, tenant Configurator writes, `post_checkout` / `case_resolved` triggers; contract: `docs/contracts/g27_scorecard_platform.md`) | **Full UI** (platform-admin Pilot/Catalog/Customer + Configurator/Deployments; Folio Meridian + `/t/ratings` + `/l/ratings` + NudgePrompt; NI `/dashboard/scorecards` + ScorecardWidget) |
| G28 | `atlas_note` | `atlas_notes` | handler via `notes.rs` | Entity defined | Partial UI (crm_timeline, notes handler) |
| G29 | `atlas_activity` | `activity` | handler via `activities.rs` | Entity defined | Partial UI (crm_timeline_generic, activities handler) |
| G31 | `atlas_lead` | `atlas_lead`, `atlas_lead_compat_view` | `LeadService` + `pm/lead.rs` | Deployed with API | **Full UI** (leads pages in anchor + network-instance + platform-admin + folio landlord + `marketing/lead_portal.rs` public page NEW) |

> **Note:** G-30 was not assigned in this sprint; the numbering reflects the spec document ordering.

### Round 3 Additions (G32–G34) — All Deployed ✅

| ID | Name | Tables | Service | Backend Status | Frontend Status |
|----|------|--------|---------|----------------|-----------------|
| G32 | `atlas_rbac` | `atlas_role_profiles`, `atlas_role_profile_permissions`, `atlas_user_app_roles`, `atlas_user_asset_access` (NEW Rev 10 — per-asset cohost/delegate/vendor scope grants) | `RbacService` (`services/rbac.rs`) | Deployed with API | Partial UI (integrated in folio auth shells + role redirect; property_owner portal uses G-32 for owner-lite role NEW) |
| G33 | `atlas_app_deployment_config` | `atlas_app_deployment_config` (`folio_mode` column: `standard\|pmc\|brokerage`; `deployment_mode`: `standard\|internal_operator`; `account_id` FK added m20260918) | Axum extractor `app_config.rs` + `PATCH /admin/app-instances/{id}/operational-config` | **Deployed with API** | **Partial UI** (InstanceOperationalConfigPanel + InstanceSyndicationPanel + TenantUsersPanel in platform-admin + Internal Instances UI) |
| G34 | `atlas_vendor_marketplace` | `atlas_service_providers` fields (`is_marketplace_visible`, `marketplace_bio`, `marketplace_trade_types`, `marketplace_location`) | none (G-12 database extension) | Deployed | **Partial UI** (`landlord/contractor_marketplace.rs` + `vendor/network_profile.rs` + `property_owner/find_vendor.rs` NEW in folio) |

### Round 4 Additions (G35+) — Emerging ✅

| ID | Name | Tables | Service | Backend Status | Frontend Status |
|----|------|--------|---------|----------------|-----------------|
| G35 | `atlas_notifications` | `atlas_notifications`, `atlas_user_notification_preferences` | `NotificationService` | Deployed with API | **Partial UI** (`landlord/notifications.rs` in folio; other portals still lack notification bell) |
| G36 | `atlas_programs` | `atlas_programs`, `atlas_program_actions`, `atlas_program_outcomes`, `atlas_program_reward_rules`, `atlas_program_reward_grants`, `atlas_program_instance_enablements` | `ProgramService` | Deployed with API (`/api/folio/programs` + `/api/admin/programs*`) | **Full UI** (platform-admin `/programs` + instance Growth panel; Folio NetworkInvite wizards/dashboards) |
| G37 | `atlas_ambassadors` | `atlas_ambassadors`, `atlas_ambassador_campaigns` | admin handlers (`admin/ambassadors.rs`) | Deployed with API (`/api/admin/ambassadors*` + QR PNG + fulfillment stubs) | **Full UI** (platform-admin `/ambassadors`; Folio `/refer`, `/refer/vendors`, `landlord/referrals.rs`) |
| — | `atlas_lp_events` | `atlas_lp_events` | embedded in landing_pages handler | Migration complete | No UI |
| — | `app_utm_presets` | `app_utm_presets` | embedded in marketing handler | Migration complete | **Full UI** (marketing/campaigns.rs in platform-admin) |
| — | `feature_flags` | `feature_flags`, `flag_audit_logs`, `flag_overrides`, `atlas_flag_instance_enablements` (NEW Rev 11) | `FlagService` | **Deployed with API** | **Full UI** (platform-admin `/flags` catalog + `instance_features_panel` Features tab; Folio flag resolution via `/api/folio/flags`) |
| — | `platform_products` | `platform_products`, `product_pages`, `platform_invites` | `product_localization.rs` | Deployed with API | **Full UI** (products/index.rs + products/detail.rs in platform-admin) |
| — | `atlas_bookings` | `atlas_bookings` (NEW Rev 10) | `pm/reservation.rs` (extended) | Migration complete | No dedicated UI (reservations UI covers bookings flows) |
| — | `atlas_otp_tokens` | `atlas_otp_tokens` (NEW Rev 10) | `auth_service.rs` | Migration complete | Partial UI (`onboarding/otp_client.rs` in folio NEW) |
| — | `atlas_invite_codes` | `atlas_invite_codes` (NEW Rev 10, + `platform_invite_code_fk`, `invite_employer_fk`) | embedded in invite handler | Migration complete | Partial UI (`onboarding/invite_codes_client.rs` + `onboarding/invite_join.rs` in folio NEW) |
| — | `atlas_user_asset_access` | `atlas_user_asset_access` (NEW Rev 10) | `rbac.rs` | Entity defined | No dedicated UI yet |
| — | `atlas_asset_value_history` | `atlas_asset_value_history` (NEW Rev 10) | `asset_service.rs` | Migration complete | Partial UI (`property_owner/property_value.rs` NEW) |

### Party Model (replaces legacy CRM)

- **`atlas_accounts`** — `account_type: individual | organization`. 30-field firmographic model (naics_code, sic_code, duns_number, domain, data_source, etc.).
- **`atlas_contacts`** — Lightweight people records belonging to an Account.
- **`AccountService`** — `create_account`, `create_from_lead_conversion`, `find_by_domain`, `search` (LIKE-injection-safe).
- **`ContactService`** — standard CRUD + tenant isolation.
- **Frontend**: Full UI in both anchor (`pages/admin/contacts/`) and network-instance (`pages/admin/contacts.rs`).

---

## Key Service Layer Facts

### LeadService (`backend/src/services/lead_service.rs`)

```
create(db, tenant_id, first, last, email, phone, company, source, listing_id, account_id) → Lead
create_from_import(db, tenant_id, data_source, data_source_id, ..., raw_data, is_duplicate, original_lead_id) → Lead
  └─ auto-provisions scorecard for fmcsa / business_leads_usa / dot_registry sources (G-31 §8.3)
find_duplicate(db, tenant_id, email, domain, duns) → Option<Uuid>
  └─ priority order: DUNS > email > domain
disqualify(db, lead_id, tenant_id, reason) → Result<()>
  └─ guards: terminal-state check (converted/disqualified → Err), tenant isolation (foreign lead → Err)
convert(db, lead_id, tenant_id, ...) → ConversionResult { account_id, contact_id, opportunity_id }
```

### ScorecardService (`backend/src/services/scorecard_service.rs`)

```
get_or_create(db, tenant_id, template_id, entity_type, entity_id) → Uuid   [idempotent]
open_session(db, tenant_id, scorecard_id, rater_id) → RatingSession
submit_entry(db, tenant_id, session_id, dimension_id, value, notes) → Entry
compute_aggregates(db, scorecard_id) → ()   [triggers OutboxWorker job]
find_similar(db, tenant_id, template_id, entity_ids) → Vec<SimilarityResult>
  └─ tenant-scoped — no cross-tenant leakage
```

### ScorecardAnalyticsService (`backend/src/services/scorecard_analytics_service.rs`)

```
recompute_aggregates(db, scorecard_id) — single-scorecard fast path
recompute_tenant_aggregates(db, tenant_id) — tenant-wide sweep
refresh_time_series(db, scorecard_id, dimension_id) — monthly + quarterly rebuild
calibrate_contributors(db, template_id) — bias offset calibration (weekly)
rebuild_portfolio(db, tenant_id) — analytics percentile rebuild
evaluate_nudge(db, scorecard_id) — display rule evaluation → WS push
```

### SyndicationEventBus (`backend/src/services/syndication_event_bus.rs`)

```
enqueue(db, tenant_id, offer_id, link_id, event_type, payload) → ()
  └─ transactional — call inside the same tx as the business event
  └─ only Active links with non-null inbound_webhook_url receive delivery
  └─ skipped links write audit row (skipped) in atlas_integration_events
start_worker(db_pool) → JoinHandle   [10s poll interval, SKIP LOCKED]
handle_retry(db, job) → ()
  └─ exponential back-off: 0 → 30s → 2m → 10m → 1h → dead-letter (attempt 5)
  └─ HMAC-SHA256 signed payload with X-Atlas-Signature header
```

### AccountService (`backend/src/services/account_service.rs`)

```
create_account(db, tenant_id, account_type, name, ...) → Account
create_from_lead_conversion(db, lead_id, tenant_id) → Account
find_by_domain(db, tenant_id, domain) → Option<Account>
search(db, tenant_id, query) → Vec<Account>   [LIKE metachar-safe]
```

### NotificationService (`backend/src/services/notification_service.rs`)

```
create(db, tenant_id, user_id, notification_type, title, body, entity_type?, entity_id?) → Notification
mark_read(db, notification_id, user_id) → ()
list_unread(db, tenant_id, user_id) → Vec<Notification>
get_user_preferences(db, user_id) → UserNotificationPreferences
update_preferences(db, user_id, preferences) → ()
```

### App Instance API (`backend/src/handlers/app_instance.rs`)

```
GET  /admin/app-instances/{id}                    → PublicConfigResponse (includes tenant_name)
PATCH /admin/app-instances/{id}/operational-config → updated config
POST /admin/app-instances/{id}/suspend            → suspend instance
POST /admin/app-instances/{id}/resume             → resume instance
GET  /admin/app-instances                         → list all instances
POST /admin/app-instances/{id}/domain             → assign domain → triggers ingress-sidecar
```

### Ingress Provisioner (`backend/src/services/ingress_provisioner.rs`)

```
provision(tenant_id, domain) → IngressProvisionResult { dns_instructions?, ssl_mode }
  └─ calls ingress-sidecar POST /api/ingress/provision
  └─ returns dns_instructions for custom domains (CNAME/A record to show the operator)
  └─ wildcard domains (*.dev.atlas.oply.co) return ssl_mode: "wildcard" — no DNS action
deprovision(tenant_id, domain) → ()
  └─ calls ingress-sidecar POST /api/ingress/deprovision
  └─ deletes Ingress + cert-manager Certificate
```

### FlagService (`backend/src/services/flag_service.rs`) — NEW Rev 11

```
resolve(db, flag_key, app_instance_id?) → FlagEffect
  └─ catalog + overrides + atlas_flag_instance_enablements
list_instance_enablements / set_instance_enablement — platform-admin Features tab
```

### Folio Handler Layer (`backend/src/handlers/folio/`)

44+ handler modules covering all PM generics. Key handlers include:
`portfolio.rs`, `assets.rs`, `leases.rs`, `vendors.rs`, `maintenance.rs`, `reservations.rs`, `campaigns.rs`, `catalog.rs`, `billing.rs`, `flags.rs` (NEW), `programs.rs`, `referrals.rs`, `notifications.rs`, `scorecards.rs`, plus marketplace/, pm/, vendor/ subtrees.

### PM Service Layer (`backend/src/services/pm/`) — 40+ services

Core PM services: `aggregates.rs`, `appliance.rs`, `applications.rs`, `asset.rs`, `attribution.rs`, `building_system.rs`, `campaign.rs`, `catalog.rs`, `commission.rs`, `condominio.rs`, `event.rs`, `fair_housing.rs`, `household.rs`, `lead.rs`, `lease.rs`, `ledger.rs`, `maintenance.rs`, `opportunity.rs`, `owner.rs`, `payment_rail.rs`, `portfolio.rs`, `quote.rs`, `record_relationship.rs`, `reporting.rs`, `reservation.rs`, `scorecard_provisioner.rs`, `str_compliance.rs`, `str_guest.rs`, `tax.rs`, `vault.rs`, `vendor.rs`, `violation.rs`, `wholesale.rs`.

**PM market sub-services** (`pm/market/`): `brazil.rs`, `market_config.rs`, `miami.rs`, `usvi.rs` — market-specific regulatory and pricing rules.

**PM payment rails** (`pm/rails/`): `bitcoin_onchain.rs`, `infinitepay.rs`, `kelviq.rs`, `lightning.rs`, `stripe_connect.rs` — multi-rail payment processing (Lightning, on-chain Bitcoin, Stripe, InfinitePay, Kelviq).

---

## Background Worker Infrastructure

All workers start automatically in `main.rs`:

| Worker | Mechanism | Purpose |
|--------|-----------|---------| 
| `OutboxWorker` | `SKIP LOCKED` polling (1.5s) | Reliable delivery: email, `recompute_scorecard_aggregates`, `refresh_scorecard_time_series`, `release_expired_reservation_holds`, `localize_product_page` / AI task fan-out, etc. |
| `SyndicationEventBus` | `SKIP LOCKED` polling (10s) | G-05: outbound webhook delivery with exponential back-off, HMAC signing, dead-lettering |
| `AITaskService::process_ai_tasks` | Tokio task (platform-admin pause/resume) | G-08: drains `atlas_ai_tasks` queue (e.g. `localize_product_page`); real log lines in admin UI |
| `DataSyncService` | Tokio task | Idempotent background syncs (Bitcoin mempool, etc.) |
| `TelemetryService` | Hourly interval | Aggregates `platform_metrics_daily` |
| `WebhookSweeper` | Periodic sweep | Retry + audit `webhook_deliveries` |

**G-27 job types wired in `OutboxWorker`:**
- `recompute_scorecard_aggregates` — single-scorecard fast path OR tenant-wide sweep
- `refresh_scorecard_time_series` — per-dimension monthly + quarterly rebuild
- `calibrate_scorecard_contributors` — weekly bias offsets calibration
- `rebuild_scorecard_portfolio` — rebuild analytics percentiles
- `evaluate_scorecard_nudge` — evaluate display rules and nudge raters via WS rooms

**G-23 job types wired in `OutboxWorker`:**
- `release_expired_reservation_holds` — clean up expired inventory reservations

**G-05 (Syndication) event bus back-off schedule:**
- Attempt 0: immediate (0s)
- Attempt 1: 30s
- Attempt 2: 2 minutes (120s)
- Attempt 3: 10 minutes (600s)
- Attempt 4: 1 hour (3600s)
- Attempt 5+: dead-letter (`next_attempt_at = 9999-12-31`)

---

## Migration Registry (as of July 11, 2026 — Rev 11)

Full migration list in chronological order (key migrations only; full list in `migration/mod.rs`):

```
... (legacy + core platform 2023–2025) ...
m20260523_000001_create_outbox_jobs
m20260523_000005_create_crm_activity_and_deep_fields
m20260523_000006_create_headless_email_tables
m20260524_000001_extend_crm_avatar_attachments
m20260525_000001_extend_notes_and_activities
m20260601_g01_geo_postgis                  ← G-01: PostGIS geo extension
m20260601_g02_vault_extension              ← G-02: Vault attachment tables
m20260601_g03_payments                     ← G-03: Ledger entries + payment credentials
m20260601_g04_subscriptions                ← G-04: Atlas subscriptions
m20260601_g05_external_integrations        ← G-05: External integrations + events (initial)
m20260601_g06_verification_queue           ← G-06: Verification requests
m20260601_g07_realtime                     ← G-07: WS rooms + messages
m20260601_g08_ai_tasks                     ← G-08: AI tasks
m20260601_g09_portfolios through g18       ← G-09 through G-18: domain object layer
m20260601_g31_atlas_lead                   ← G-31: atlas_lead + trigger
m20260601_unify_accounts_contacts          ← Party model: atlas_accounts/contacts
m20260701_g19_campaigns                    ← G-19: campaign tables (initial)
m20260701_g23_reservations                 ← G-23: reservations + availability (initial)
m20260701_g25_commission_plans             ← G-25: commission plans + splits
m20260701_g26_catalog                      ← G-26: catalog tables (initial)
m20260701_g27_scorecards                   ← G-27: 11 scorecard tables + triggers
m20260702_gap_fill_accounts_contacts       ← Firmographic promotion
m20260703_g28_atlas_note                   ← G-28: atlas_notes polymorphic promotion
m20260704_g29_atlas_activity               ← G-29: activity polymorphic + backfill
m20260705_drop_legacy_lead                 ← Drops legacy lead, creates compat view
m20260706_seed_g27_background_jobs         ← Seeds scorecard aggregate + time-series jobs
m20260707_g27_is_inverted through 0714     ← G-27: is_inverted, display_rules, data science v1/v2/v3, display_config
m20260801_pm_g27_template_scope            ← G-27: PM-scoped template support
m20260802_g23_atlas_reservations           ← G-23: refined reservation schema
m20260803_g26_atlas_catalog                ← G-26: refined catalog schema
m20260804_g19_atlas_campaigns              ← G-19: refined campaign schema
m20260805_g20_atlas_attribution            ← G-20: marketing attribution touchpoints
m20260806_g21_atlas_events                 ← G-21: events and registration schema
m20260807_g22_atlas_record_relationships   ← G-22: record relationships linkage
m20260808_g24_atlas_quotes                 ← G-24: quotes and line items
m20260809_g26_catalog_forward              ← G-26: catalog forward/refinement
m20260810_add_folio_role_to_user_account   ← Folio role field to user_account
m20260811_g32_atlas_rbac                   ← G-32: platform-generic RBAC tables
m20260812_g32_folio_role_seed              ← G-32: seed landlord role
m20260813_g32_migrate_folio_roles          ← G-32: migrate folio user roles
m20260814_g32_drop_folio_role_column       ← G-32: drop folio_role column from user_account
m20260815_g33_app_deployment_config        ← G-33: app deployment config table
m20260816_g33_folio_pmc_seed               ← G-33: seed PMC rbac role profiles
m20260817_folio_managed_account_id         ← Managed account isolation on portfolios/assets/leases
m20260818_folio_client_role_scope          ← Client scope support on user roles
m20260819_g34_vendor_marketplace           ← G-34: vendor marketplace visibility + geo point matching
m20260900_g10_asset_lifecycle              ← G-10: asset status and condition lifecycle fields
m20260901_platform_products                ← Product Launch Engine: platform product registry
m20260902_app_instance_public_config       ← Public routing: public_slug & custom_domain
m20260903_platform_products_launch_engine  ← Product Launch Engine: page template registries
m20260904_product_page_variants            ← Product Launch Engine: SEO landing variants
m20260905_product_domain_localization      ← Product Launch Engine: variant localization
m20260906_subscription_grace_period        ← Grace period support for G-04 subscriptions
m20260907_feature_flags                    ← Platform feature flag registry
m20260908_platform_invitations             ← Invitation infrastructure
m20260909_folio_instance_mode              ← G-33: folio_mode column (standard|pmc|brokerage) + CHECK
m20260910_folio_instance_syndication       ← G-05: folio instance syndication config
m20260911_asset_listing_mode               ← G-10: asset listing_mode column
m20260912_atlas_syndication_offer          ← G-05: syndication offer registry table
m20260913_atlas_app_instance_syndication   ← G-05: per-instance syndication link table
m20260914_atlas_listing_asset_fk           ← FK enforcement: listing → asset
m20260915_atlas_syndication_outbox         ← G-05: syndication outbox + integration_events tables
m20260916_product_tracking_pixels          ← tracking pixel support on product pages
m20260917_platform_products_app_slug       ← app_slug column on platform_products
m20260918_deployment_config_account_link   ← account_id FK on atlas_app_deployment_config
m20260919_g19_campaigns_parent_id          ← G-19: campaign hierarchy (parent_id)
m20260920_atlas_notifications              ← G-35: notification tables + user preferences
m20261018_g36_atlas_programs               ← G-36: programs, actions, outcomes, reward rules/grants + NetworkInvite seeds
m20261019_g36_network_invite_reward_rules  ← G-36: seed actor subscription_credit_days rules for NetworkInvite (grants only)
m20261020_g36_subscription_credit_ledger   ← G-36: apply subscription_credit_days grants to internal credit ledger
m20261029_friends_family_referral_campaign ← G-19: Friends & Family landlord campaign seed
m20261030_campaign_global_name_ff_vendors_g37_ambassadors ← G-19 global_name + F&F vendors child + G-37 ambassadors
m20260921_platform_invite_enhancements     ← invite enhancements (expiry, role pre-fill)
m20260922_app_pages_app_id                 ← app_id FK on app_pages
m20260923_app_page_variants                ← page variant table for A/B + l10n
m20260924_app_utm_presets                  ← UTM preset management table
m20260925_atlas_lp_events                  ← landing page conversion event tracking
m20260926_folio_product_seed               ← Rev 9: Folio platform_products + product_page_templates seed
m20260927_folio_broker_product_seed        ← NEW Rev 10: Folio broker product seed
m20260928_cohost_marketplace_product_seed  ← NEW Rev 10: Cohost marketplace product seed
m20260929_folio_pm_product_seed            ← NEW Rev 10: Folio PM product seed
m20260930_folio_vendor_product_seed        ← NEW Rev 10: Folio vendor product seed
m20260931_app_pages_locale                 ← NEW Rev 10: App pages locale column
m20261001_fix_existing_ruuderie_domains    ← NEW Rev 10: Fix ruuderie domain records
m20261002_seed_ruuderie_folio_domain       ← NEW Rev 10: Seed ruuderie folio domain
m20261003_platform_invite_account_id       ← NEW Rev 10: account_id on platform_invite
m20261004_platform_invite_asset_lease_scope ← NEW Rev 10: asset/lease scope on invites
m20261005_atlas_user_asset_access          ← NEW Rev 10: G-32 per-asset access grants table
m20261006_folio_missing_role_profiles      ← NEW Rev 10: fix missing folio RBAC role profiles
m20261007_asset_str_traits                 ← NEW Rev 10: G-10 STR-specific traits on assets
m20261008_lease_type                       ← NEW Rev 10: lease_type column on contracts
m20261009_atlas_bookings                   ← NEW Rev 10: G-23 bookings table (separate from reservations)
m20261010_platform_invite_booking_tenancy  ← NEW Rev 10: booking/tenancy scope on platform_invite
m20261011_atlas_invite_codes               ← NEW Rev 10: invite code table
m20261012_platform_invite_code_fk          ← NEW Rev 10: invite_code FK on platform_invite
m20261013_invite_employer_fk               ← NEW Rev 10: employer FK on invites
m20261014_atlas_otp_tokens                 ← NEW Rev 10: OTP token table
m20261015_atlas_asset_status_owner_occupied ← NEW Rev 10: owner-occupied status on assets
m20261015_atlas_asset_value_history        ← NEW Rev 10: property value history table
m20261015_g32_property_owner_lite_seed     ← NEW Rev 10: G-32 property owner portal RBAC seed
m20261015_platform_invite_review_purpose   ← NEW Rev 10: review_purpose on platform_invite
m20261015_rating_sessions_review_fields    ← NEW Rev 10: G-27 review content fields on rating_sessions
m20261016_atlas_scorecard_template_deployments ← G-27 template ↔ app-instance deployments
m20261017_rating_sessions_app_instance_id  ← G-27 Phase C: nullable app_instance_id on rating sessions
m20261018_g36_atlas_programs               ← G-36: programs, actions, outcomes, reward rules/grants
m20261019_g36_network_invite_reward_rules  ← G-36: NetworkInvite subscription_credit_days rules
m20261020_g36_subscription_credit_ledger   ← G-36: apply credit grants to internal ledger
m20261021_g36_program_instance_enablements ← G-36: per-instance program enablements
m20261022_platform_product_plans           ← NEW Rev 11: product plan catalog
m20261023_folio_marketing_hero_seeds       ← NEW Rev 11: Folio marketing hero content seeds
m20261024_folio_marketing_section_blocks   ← NEW Rev 11: Folio marketing section blocks
m20261025_folio_founding_beta_products     ← NEW Rev 11: founding/beta product seeds
m20261026_folio_vendor_trade_categories    ← NEW Rev 11: vendor trade category seeds
m20261027_atlas_flag_instance_enablements  ← NEW Rev 11: per-instance feature-flag enablements
m20261028_g06_verification_reviewer_notes  ← NEW Rev 11: G-06 reviewer notes + request-more-info
m20261029_friends_family_referral_campaign ← G-19: Friends & Family landlord campaign seed
m20261030_campaign_global_name_ff_vendors_g37_ambassadors ← G-19 global_name + F&F vendors + G-37 ambassadors
m20261101_seed_local_dev_domain_aliases ← atlas-local: *.localhost app_domain aliases for Compose
m20261102_g19_direct_mail_drops_offer_codes ← G-19 DM: mail_drops + offer_codes + direct_mail enum
m20261103_acquisition_feature_flags ← acquisition.dm_tracking + acquisition.open_signup seeds
```

**~57 new migrations since Rev 9.** Total: **237** migration files (latest `m20261103_*`).

> See `docs/architecture/product_page_system.md` for the content resolution algorithm.

---

## Security Invariants

Every service enforces these. Tests verify them.

| Service | Invariant |
|---------|-----------|
| `LeadService::disqualify` | Terminal state guard (converted/disqualified → Err); tenant isolation (foreign lead → Err) |
| `ScorecardService::open_session` | Scorecard ownership check (tenant filter before session insert) |
| `ScorecardService::submit_entry` | Session→tenant ownership + session→scorecard coupling check |
| `ScorecardService::find_similar` | Tenant-scoped filter — no cross-tenant vector leakage |
| `AccountService::search` | LIKE metachar escaping — `%`, `_`, `\` treated as literals |
| `SyndicationEventBus::enqueue` | Only enqueues for `Active` links with a non-null `inbound_webhook_url`; skipped links write a `skipped` audit row in `atlas_integration_events` |
| `SyndicationEventBus::handle_retry` | Dead-letter at `MAX_RETRY_COUNT = 5`; no infinite retry loops possible |
| `IngressProvisioner::provision` | Only the platform-admin UI + authenticated admin endpoints can trigger ingress provisioning; the ingress-sidecar itself only accepts calls from the backend service account |

---

## Test Coverage

```
backend/src/tests/
├── mod.rs                              ← registers all test modules
├── api_tests.rs                        ← setup_test_app harness
├── test_utils.rs                       ← singleton DB + tenant helpers
├── account_tests.rs                    ← AccountService CRUD + firmographic tests
├── ad_purchase_tests.rs                ← Ad purchase lifecycle tests
├── admin_module_tests.rs               ← Admin module provisioning tests
├── admin_tests.rs                      ← Admin API tests
├── anchor_pages_tests.rs               ← Anchor CMS page rendering tests
├── audit_tests.rs                      ← Audit log tests
├── billing_tests.rs                    ← Billing + LedgerService tests
├── crm_tests.rs                        ← CRM legacy bridge tests
├── crm_extended_tests.rs               ← Extended CRM (atlas_account/contact) tests
├── domain_provisioning_tests.rs        ← NEW Rev 8: Domain provisioning + ingress lifecycle tests
├── feed_tests.rs                       ← Content feed tests
├── g27_scorecard_tests.rs              ← G-27: template/session/entry/aggregate/find_similar
├── instance_lifecycle_tests.rs         ← NEW Rev 8: App instance lifecycle (suspend/resume/config)
├── landing_page_builder_tests.rs       ← NEW Rev 8: Landing page builder + UTM preset tests
├── lead_account_tests.rs               ← G-31 + AccountService: full lifecycle + security invariants
├── magic_link_tests.rs                 ← Magic link deduplication tests
├── provision_tests.rs                  ← Tenant provisioning tests
├── relational_dependencies_tests.rs    ← Foreign key + cascade tests
├── search_tests.rs                     ← Global search + G-06 index tests
├── services_tests.rs                   ← DB-backed integration tests
├── telemetry_tests.rs                  ← Telemetry aggregation tests
├── template_tests.rs                   ← Template CRUD tests
├── tenant_settings_tests.rs            ← Tenant settings tests
├── waitlist_integration_tests.rs       ← NEW Rev 8: Waitlist/invite integration tests
├── webauthn_registry_tests.rs          ← Multi-tenant WebAuthn registry tests
├── webhook_tests.rs                    ← Webhook delivery + retry tests
└── unit/
    ├── mod.rs
    ├── app_instance_unit_tests.rs      ← App instance logic unit tests
    ├── atlas_activity_unit_tests.rs    ← G-29: primary_subject + is_completed_communication
    ├── atlas_note_unit_tests.rs        ← G-28: entity helper coverage
    ├── folio_routing_unit_tests.rs     ← NEW Rev 8: Folio role routing + shell guard logic
    ├── g27_unit_tests.rs               ← G-27: additional unit coverage
    ├── pm_phase3_unit_tests.rs         ← Phase 3-7 PM services (payment rails, WS rooms, lead rate limiting, geo coordinates)
    ├── pmc_marketplace_unit_tests.rs   ← PMC (G-33) and Vendor Marketplace (G-34) aggregates, invite flow, geo filters, bio validations, client-account role scoping
    ├── scorecard_lead_unit_tests.rs    ← ScorecardService confidence boundaries + LeadModel helpers
    ├── session_unit_tests.rs           ← Session model unit tests
    ├── syndication_unit_tests.rs       ← G-05: 109 pure unit tests covering FolioMode×NI combos, event types, enqueue skip logic, back-off schedule, HMAC signing, mandatory tier matrix, integration event log, OutboxJobType/Status roundtrips
    ├── type_system_unit_tests.rs       ← Type system + enum unit tests
    └── waitlist_unit_tests.rs          ← NEW Rev 8: Waitlist state machine + invite flow unit tests
```

**Total: 43 test files (27 integration + 16 unit).** Up from 42 in Rev 9 (added `provision_unit_tests.rs`).

**Pattern:** `tests/unit/` for pure no-DB tests; `tests/*.rs` (non-unit) for DB-backed integration tests using `setup_test_app`.

**Critical Unit Test Pattern (Rev 5–10):**
- `DeriveActiveEnum::into_value()` returns the DB string value (`"pmc"`, `"brokerage"`) — use this when verifying persistence behavior.
- `strum_macros::Display` renders PascalCase variant names (`"Pmc"`, `"BrandedPortal"`) — do NOT use `.to_string()` for DB value assertions.
- Import real production types (`FolioMode`, `SyndicationStatus`, `event_type`, `MAX_RETRY_COUNT`) rather than mirroring them locally.
- `ContactResponse` (not `Contact` DB model) is the correct type for deserializing `/api/contacts` API responses. `ContactResponse` has `full_name: Option<String>` (built from first + last), NOT a `name` field.

---

## AtlasApp System (Three-Tier Route Architecture)

```
Tier 3 (api.rs)          — Auth, Sessions, Passkeys, Admin, A/B, Setup, RBAC (api/rbac)
       ↓ get_active_apps() loop
       ↓ merge authenticated_routes_raw() for G-32 API routes
Tier 1 (CorePlatformApp) — CMS routes: pages, menus, onboarding, feeds, search
Tier 2 (AnchorApp)       — Listings, CRM, Profiles, Anchor content routes
Tier 2 (FolioApp/PMApp)  — Property Management [backend fully deployed, frontend built inside apps/folio]
```

**Critical**: `with_state(db)` is called EXACTLY ONCE at the `AtlasApp` boundary. Never call it inside handler route constructors that are used inside an `AtlasApp` — Axum silently drops routes from pre-finalized sub-routers.

**Registered Apps (as of July 2026 Rev 8)**:
- `CorePlatformApp` — always registered first
- `AnchorApp` — CMS + listing + CRM domain routes
- `FolioApp` (via `handlers/folio/`) — 24+ handler files for all PM generics (G09–G26) + syndication admin
- `NetworkInstanceApp` — planned Phase 9+

---

## Handler Cutovers (Legacy CRM)

All six legacy CRM handlers have been updated:
- `leads.rs`, `customers.rs`, `contacts.rs`, `deals.rs`, `cases.rs`, `activities.rs`

**Current Pattern (Transition Phase)**:
- Handlers still support the old API shapes for backward compatibility (especially admin UI).
- They now **dual-write**: create/update both legacy rows *and* the new canonical objects via the services.
- Strong deprecation banners have been added.
- Billing paths now route through the unified `LedgerService`.

This is a controlled migration bridge, not the final state.

---

## Multi-Tenant WebAuthn (Passkey) Architecture

- `WebauthnRegistry` — Moka LRU cache keyed by origin URL.
- **eTLD+1 derivation**: `dev.buildwithruud.com` → `rp_id = buildwithruud.com`.
- **DB pre-warm at startup**: Seeds registry for all tenant domains in `app_domain` table.
- **Dynamic `get_or_create()`**: Cache miss → DB verify → build `Webauthn` instance → cache.
- Unauthorized origins (not in `app_domain`) return `HTTP 403`.

---

## Frontend Apps Layer

### anchor (SSR + WASM — Primary CMS + CRM App)

**Route registration**: `apps/anchor/src/app.rs`

**Built Pages** (`apps/anchor/src/pages/`):
| Page File | Purpose | Generics Surfaced |
|---|---|---|
| `admin.rs` | Admin shell + routing | — |
| `admin/admin_tables.rs` | Admin data tables | G-31 |
| `admin/contacts.rs` + `contacts/contacts_views.rs` | Contact management | G-28/G-29/Party |
| `admin/leads.rs` + `leads/leads_views.rs` | Lead management | G-31 |
| `admin/page_editor.rs` | CMS page editor | CMS |
| `admin/pages_list.rs` | CMS pages list | CMS |
| `bitcoin.rs` | Bitcoin mempool view | DataSync |
| `blog.rs` | Blog listing | CMS |
| `book.rs` | Booking page | G-23 |
| `dynamic_entry.rs` | Dynamic CMS entry | CMS |
| `dynamic_landing.rs` | Dynamic CMS landing | CMS |
| `landing.rs` | Default landing page | CMS |
| `legal.rs` | Legal/terms pages | CMS |
| `onboarding.rs` | Tenant onboarding | Platform |
| `setup_passkey.rs` | Passkey setup | WebAuthn |
| `legacy/certifications.rs` | Legacy cert page | — |
| `legacy/projects.rs` | Legacy projects | — |
| `legacy/resume.rs` | Resume page | — |
| `legacy/services.rs` | Services page | — |

**Built Components** (`apps/anchor/src/components/`):
- `admin_modal.rs`, `content_feed.rs`, `design_mode.rs`, `dynamic_header.rs`, `footer.rs`, `nav.rs`, `theme_provider.rs`, `widget_registry.rs`
- **Block components** (CMS content blocks): `accordion`, `badge_list`, `callout`, `content_feed`, `form_builder`, `grid`, `hero`, `profile_header`, `raw_html`, `rich_text`, `stats`, `timeline`

**Generics with Full UI in anchor**: G-31 (leads), G-28/G-29 (contacts/CRM timeline)

---

### network-instance (SSR + WASM — Multi-Tenant Marketplace Frontend)

**Route registration**: `apps/network-instance/src/app.rs`

**Built Pages** (`apps/network-instance/src/pages/`):
| Page File | Purpose | Generics Surfaced |
|---|---|---|
| `admin.rs` | Admin shell | — |
| `admin/contacts.rs` | Contact management | Party model |
| `admin/leads.rs` | Lead management | G-31 |
| `auth/login.rs` | Login page | Auth |
| `auth/register.rs` | Registration | Auth |
| `dashboard/layout.rs` | Dashboard shell | — |
| `dashboard/listings.rs` | Listing management | G-10 |
| `dashboard/settings.rs` | User settings | — |
| `scorecard_mount.rs` | G-27 ScorecardWidget / Configurator mount | G-27 |
| `search.rs` | Directory search | Global Search |

**Built Components**: `category_nav.rs`, `layout.rs`, `login_modal.rs`, `search_ui.rs`, `seo.rs`

> **Rev 11:** NI scorecard mount at `scorecard_mount.rs` confirms G-27 Full UI path for network-instance.

### platform-admin (CSR — Platform Admin UI)

**Route registration**: `apps/platform-admin/src/app.rs`

**Built Pages** (`apps/platform-admin/src/pages/`):
| Page / Section | Purpose | Generics Surfaced |
|---|---|---|
| `dashboard.rs` | Main dashboard | Telemetry |
| Auth pages (`login.rs`, `magic_login.rs`, `verify_token.rs`, `setup.rs`) | Auth flow | WebAuthn |
| `crm/grid.rs`, `crm/create.rs`, `crm/detail.rs`, `crm/leads.rs`, `crm/contacts.rs`, `crm/accounts.rs`, `crm/opportunities.rs` | CRM management | G-31, Party |
| `apps/index.rs`, `apps/create.rs`, `apps/detail.rs`, `apps/instance.rs`, `apps/panel.rs`, `apps/list.rs`, `apps/tenant_detail.rs` | App instance management | Platform + G-33 |
| `apps/instance/anchor_instance.rs` | Anchor-specific instance view (tabs: Overview, Content, Domains, Operational Config, Users) | G-33, G-05, CMS |
| `apps/instance/folio_instance.rs` | Folio-specific instance view | G-33 |
| `apps/instance/network_instance.rs` | Network-specific instance view | G-33 |
| `internal_instances/index.rs` | Internal instance listing + create modal | G-33, Domain Provisioning |
| `internal_instances/config.rs` | Per-instance config: domain assign, SSL status, DNS instructions, users | G-33, Ingress, G-32 |
| `billing/dashboard.rs`, `billing/products.rs`, `billing/tenant.rs`, `billing/scorecards.rs`, `billing/scorecard_session.rs` | Billing UI | G-03, G-04, G-27 |
| `network/index.rs`, `network/create.rs`, `network/detail.rs`, `network/settings.rs`, `network/syndication.rs` | Network management | G-05 (syndication) |
| `network/listings/` (create, detail, index) | Listing management | G-10 |
| `network/categories/` (create, detail, index) | Category management | — |
| `network/templates/` (create, detail, index) | Template management | — |
| `network/types/` (create, detail, index) | Network type management | — |
| `syndication/offers.rs` | Syndication offer registry | **G-05** |
| `syndication/links.rs` | Per-instance syndication links | **G-05** |
| `admin/users.rs`, `admin/ai_tasks.rs`, `admin/compliance.rs`, `admin/developer.rs`, `admin/integrations.rs`, `admin/profile.rs`, `admin/security.rs` | Platform admin settings | G-08 |
| `analytics/index.rs` | Analytics dashboard | Telemetry |
| `clients/index.rs` | Client portfolio listing | G-33 |
| `flags/index.rs` | Feature flag management | Feature Flags |
| `map/index.rs` | Geographic map view | G-01 |
| `marketing/index.rs` | Marketing overview | — |
| `marketing/campaigns.rs` | Campaign management UI | G-19 |
| `marketing/ambassadors.rs` | Ambassadors CRUD + dual QR + fulfillment stubs | G-37 |
| `marketing/landing_pages.rs` | Landing page builder + UTM presets + variant tracking | CMS, G-19 |
| `products/index.rs`, `products/detail.rs` | Product registry | Product Launch Engine |
| `programs/index.rs`, `programs/detail.rs` | G-36 program catalog | G-36 |
| `verification/index.rs` | Verification queue — **Rev 11** reviewer notes / request-more-info / vault | G-06 |
| `support/index.rs` | Support inbox — **Rev 11** internal notes + reply notifications | Support |
| `logs/index.rs` | Audit log viewer — **Rev 11** filters + CSV export | Audit |
| `admin/ai_tasks.rs` | AI task monitor — **Rev 11** live worker + pause | G-08 |
| `shared/profiles.rs`, `shared/svg_charts.rs` | Shared admin views | — |

**Built Components** (`apps/platform-admin/src/components/`):
| Component | Purpose |
|---|---|
| `app_manifest.rs` | App manifest display |
| `callout.rs` | Standardised callout/warning banner component |
| `dynamic_form.rs` | Dynamic form builder |
| `instance_features_panel.rs` | **NEW Rev 11** — per-instance feature-flag enablements (Features tab) |
| `instance_programs_panel.rs` | **NEW Rev 11** — per-instance G-36 program enablements (Growth) |
| `instance_operational_config_panel.rs` | G-33 operational config PATCH UI (folio_mode, billing_tier, portal toggles) |
| `instance_syndication_panel.rs` | G-05 per-instance syndication link management |
| `tenant_users_panel.rs` | User/invite/role management for a given instance |
| `intel_sidebar.rs` | Intelligence sidebar |
| `milestone_modal.rs` | Milestone tracking modal |
| `omnibar.rs` | Command palette / omnibar |
| `onboarding_wizard.rs` | Guided onboarding wizard |
| `recommended_partners.rs` | Partner recommendations |
| `seed_picker.rs` | App seed selection |
| `upsell_banner.rs` | Subscription upsell |
| `gtm_process_strip.rs` | GTM process strip |
| `redirect.rs` | Client redirect helper |

**API Clients** (`apps/platform-admin/src/api/`): Full set including `syndication.rs`.

**Instance UI Architecture Note (Rev 6/7/8):**
- `pages/apps/instance.rs` — thin dispatcher; fetches `public_config`, routes to type-specific subcomponent based on `app_slug`.
- Each type subcomponent (`anchor_instance.rs`, `folio_instance.rs`, `network_instance.rs`) owns its own tabs and content.
- `pages/internal_instances/` — separate section for platform-team managed instances (InternalOperator mode). Separate from the tenant-facing `apps/` pages.
- **Critical**: All root divs must have `w-full` — the sidebar layout is a flex row; without explicit width, content collapses to minimum size.

---

### folio (SSR + WASM — Dedicated Property Management Frontend App)

**Route registration**: `apps/folio/src/app.rs`

**9 distinct role portals**: landlord (`/l`), tenant (`/t`), vendor (`/v`), owner (`/o`), PMC (`/pmc`), agent (`/a`), broker (`/br`), STR host (`/str`), **property owner (`/po`) — NEW Rev 10**.

**Built Pages** (`apps/folio/src/pages/`):
| Portal / Page File | Purpose | Generics Surfaced |
|---|---|---|
| **Auth** | | |
| `login.rs`, `verify.rs`, `not_found.rs` | Authentication flow and redirection | G-32 |
| `auth/passkey_setup.rs` | Passkey registration flow | WebAuthn |
| `dashboard.rs` | General dashboard / routing index | — |
| `onboarding/wizard.rs` | Tenant onboarding wizard (legacy) | Platform |
| `onboarding/landlord_wizard.rs` | **Rev 11 stitch parity** — landlord setup (`/onboarding`) | Platform |
| `onboarding/*_wizard.rs` | Full persona suite: tenant, vendor, agent, broker, pmc, owner, property_owner, cohost, str_guest | Platform |
| `onboarding/invite_join.rs` + `invite_codes_client.rs` + `otp_client.rs` | Join codes + OTP pre-auth | Platform |
| `components/wizard_shell.rs` | **Rev 11** — shared split shell; session peek; stitch tokens | Platform |
| `settings.rs` | User account settings | — |
| **Landlord Portal** (`/l`) — **35 pages** (was 33) | | |
| `landlord/dashboard.rs` | Landlord portfolio metrics & overview | — |
| `landlord/portfolio.rs` | Portfolio layout & group management | G-09 |
| `landlord/assets.rs` | Property and units tracker | G-10 |
| `landlord/asset_detail.rs` | **NEW** Individual asset deep-dive | G-10 |
| `landlord/asset_alerts.rs` | **NEW** Asset condition/lifecycle alerts | G-10 |
| `landlord/leases.rs` | Active tenant agreements and templates | G-11 |
| `landlord/lease_detail.rs` | **NEW** Individual lease detail view | G-11 |
| `landlord/leads.rs` | Prospects pipeline | G-31 |
| `landlord/campaigns.rs` | Vacancy outreach campaigns | G-19 |
| `landlord/billing.rs` | Invoices and ledger splits | G-03 |
| `landlord/ledger.rs` | **NEW** Full ledger + tax event view | G-03, G-17 |
| `landlord/account_billing.rs` | **NEW** Account-level billing management | G-03, G-04 |
| `landlord/str_compliance.rs` | Short-term rental regulatory licenses | G-16 |
| `landlord/catalog.rs` | Product catalog rate plans and seasons | G-26 |
| `landlord/vendors.rs` | Contractor profiles and marketplace opt-in | G-12, G-34 |
| `landlord/contractor_marketplace.rs` | **NEW** G-34 public vendor marketplace browsing | G-34 |
| `landlord/reservations.rs` | Unit and booking calendars | G-23 |
| `landlord/building_systems.rs` | **NEW** HVAC, plumbing, electrical systems | G-10 |
| `landlord/unit_appliances.rs` | **NEW** Per-unit appliance tracker | G-10 |
| `landlord/communications.rs` | **NEW** Tenant communications center | G-28/G-29 |
| `landlord/digital_vault.rs` | **NEW** Document vault/storage browser | G-02, G-14 |
| `landlord/inspections.rs` | **NEW** Property inspection management | G-13 |
| `landlord/maintenance_queue.rs` | **NEW** Work order queue + dispatch | G-13 |
| `landlord/map_portfolio.rs` | **NEW** Geographic portfolio map view | G-01, G-09 |
| `landlord/notifications.rs` | **NEW** Notification center | G-35 |
| `landlord/listing_preview.rs` | **NEW** Listing preview / marketing preview | G-10 |
| `landlord/meridian_config.rs` | Meridian analytics + shared-ui TenantAdmin Configurator | G-27 |
| `landlord/ratings.rs` | Landlord pending contractor ratings (`case_resolved`) | G-27 |
| `landlord/syndication.rs` | **NEW** Channel syndication management | G-05 |
| `landlord/tenant_profile.rs` | **NEW** Individual tenant profile view | Party |
| `landlord/violations.rs` | **NEW** Lease violation tracking | G-16 |
| `landlord/team.rs` | Team member management | G-32 |
| `landlord/referrals.rs` | **NEW Rev 11** — landlord My referrals / F&F share | G-19, G-37 |
| `landlord/wholesaling.rs` | Wholesale property disposition | — |
| **Tenant Portal** (`/t`) — 16 pages | | |
| `tenant/dashboard.rs` | Tenant dashboard | — |
| `tenant/my_lease.rs` | Active lease agreement detail | G-11 |
| `tenant/payments.rs` | Payment portal (ledger transactions) | G-03 |
| `tenant/payment_history.rs` | **NEW** Full payment history log | G-03 |
| `tenant/maintenance.rs` | Submit and track work requests | G-13 |
| `tenant/maintenance_detail.rs` | **NEW** Individual work order detail | G-13 |
| `tenant/maintenance_triage.rs` | **NEW** AI-assisted maintenance triage | G-13, G-08 |
| `tenant/reservations.rs` | Amenity / unit reservations | G-23 |
| `tenant/documents.rs` | **NEW** Document access/download | G-14 |
| `tenant/household.rs` | **NEW** Household members management | — |
| `tenant/inbox.rs` | **NEW** Tenant message inbox | G-28/G-29 |
| `tenant/profile.rs` | **NEW** Tenant profile & preferences | Party |
| `tenant/application_status.rs` | **NEW** Rental application status tracker | G-18 |
| `tenant/reports.rs` | **NEW** Tenant payment/activity reports | G-03 |
| `tenant/violations.rs` | **NEW** Violation notices viewer | G-16 |
| **Vendor Portal** (`/v`) — 8 pages | | |
| `vendor/dashboard.rs` | Vendor dispatch overview | — |
| `vendor/work_orders.rs` | Dispatched work order cases | G-13 |
| `vendor/invoices.rs` | Invoicing ledger records | G-03 |
| `vendor/network_profile.rs` | **NEW** G-34 marketplace vendor profile editor | G-34 |
| `vendor/schedule.rs` | **NEW** Work schedule / calendar | G-23 |
| `vendor/job_link.rs` | **NEW** Public job link / deep-link handler | G-13 |
| `vendor/onboard.rs` | **NEW** Vendor onboarding wizard | — |
| **Owner Portal** (`/o`) — 6 pages | | |
| `owner/dashboard.rs` | Owner equity dashboard | — |
| `owner/property.rs` | **NEW** Property overview for owner | G-09, G-10 |
| `owner/maintenance.rs` | **NEW** Maintenance transparency view | G-13 |
| `owner/statements.rs` | **NEW** Owner financial statements | G-03, G-17 |
| `owner/distributions.rs` | **NEW** Distribution history & schedule | G-03 |
| **PMC Portal** (`/pmc`) — 8 pages | | |
| `pmc/dashboard.rs` | Property management company dashboard | G-33 (PMC mode) |
| `pmc/client_book.rs` | Client portfolio management | G-33 |
| `pmc/client_detail.rs` | **NEW** Individual client deep-dive | G-33 |
| `pmc/maintenance_dispatch.rs` | **NEW** Cross-portfolio maintenance dispatch | G-13 |
| `pmc/onboard.rs` | **NEW** PMC onboarding flow | G-33 |
| `pmc/owner_statements.rs` | **NEW** Owner statement generation | G-03, G-17 |
| `pmc/portfolio_map.rs` | **NEW** Cross-portfolio geographic map | G-01, G-09 |
| **Agent Portal** (`/a`) | | |
| `agent/dashboard.rs` | Agent listings dashboard | G-05 (brokerage) |
| **Broker Portal** (`/br`) | | |
| `broker/dashboard.rs` | Broker transaction overview | G-05 (brokerage) |
| **STR Host Portal** (`/str`) — 13 pages NEW in Rev 8 | | |
| `str_host/dashboard.rs` | STR host operations dashboard | G-23, G-26 |
| `str_host/listing.rs` | STR listing detail management | G-10 |
| `str_host/listing_index.rs` | All STR listings view | G-10 |
| `str_host/reservations.rs` | Booking calendar + reservation management | G-23 |
| `str_host/calendar.rs` | Availability calendar editor | G-23, G-26 |
| `str_host/pricing.rs` | Dynamic pricing / rate rules | G-26 |
| `str_host/channels.rs` | OTA channel connection management | G-05 |
| `str_host/syndication.rs` | Syndication channel settings | G-05 |
| `str_host/messages.rs` | Guest messaging inbox | G-07, G-28 |
| `str_host/reviews.rs` | Guest review management | G-27 |
| `str_host/incidents.rs` | Guest incident reporting | G-13, G-16 |
| `str_host/violation_file.rs` | STR regulatory violation filing | G-16 |
| **Marketing / Public Pages** | | |
| `marketing/ltr_listings.rs` | **NEW** Public long-term rental listings | G-10 |
| `marketing/str_listings.rs` | **NEW** Public STR listings search | G-10, G-26 |
| `marketing/market_landing_page.rs` | **NEW** Market-specific landing page | CMS |
| `marketing/lead_portal.rs` | **NEW** Public lead capture portal | G-31 |
| `marketing/renter_application.rs` | **NEW** Public renter application form | G-18 |
| `marketing/ni_signup.rs` | **NEW** Network instance signup page | — |
| `marketing/inquiry_confirm.rs` | **NEW** Inquiry confirmation page | G-31 |

**Root-level pages**: `leads.rs`, `leases.rs`, `portfolio.rs`, `reservations.rs` (shallow route aliases).

**Layouts & Shells**:
- Layout components: `landlord_layout.rs`, `tenant_layout.rs`, `vendor_layout.rs`, `owner_layout.rs`, `pmc_layout.rs`, `brokerage_layouts.rs` (AgentLayout, BrokerLayout).
- New in Rev 8: STR host layout implied by `str_host/` pages (check `app.rs` for route registration).
- Role-based shells: `LandlordShell`, `TenantShell`, `VendorShell`, `AgentShell`, `BrokerShell` guard access with dynamic redirection based on active user app roles.
- `RoleRedirect` dispatches `/` directly to correct paths based on resolved role details.
- `onboarding_banner.rs` component surfaces onboarding progress inline.
- `sidebar.rs` + `nav.rs` — shared navigation components.

---

### shared-ui Component Library (`apps/shared-ui`)

#### Primitive UI Components (`src/components/ui/`) — 85 components

| Category | Components |
|---|---|
| **Layout** | `accordion`, `aspect_ratio`, `bento_grid`, `card`, `card_carousel`, `carousel`, `collapsible`, `expandable`, `footer`, `header`, `separator`, `sheet`, `sidenav` |
| **Navigation** | `bottom_nav`, `breadcrumb`, `context_menu`, `dropdown_menu`, `menubar`, `navigation_menu`, `pagination`, `tabs` |
| **Forms & Inputs** | `auto_form`, `checkbox`, `chips`, `date_picker`, `date_picker_dual_state`, `date_picker_state`, `field`, `form`, `input`, `input_group`, `input_otp`, `input_phone`, `label`, `multi_select`, `radio_button`, `radio_button_group`, `select`, `select_native`, `slider`, `switch`, `textarea` |
| **Feedback** | `alert`, `alert_dialog`, `animate`, `empty`, `faq_transition`, `progress`, `shimmer`, `skeleton`, `sonner`, `spinner`, `status`, `tooltip` |
| **Display** | `avatar`, `badge`, `callout`, `charts`, `chat`, `data_grid`, `data_table`, `image`, `item`, `kbd`, `link`, `marquee`, `mask`, `related_list`, `scroll_area`, `table` |
| **Interaction** | `action_bar`, `button`, `button_action`, `button_group`, `command`, `dialog`, `drag_and_drop`, `drawer`, `hover_card`, `popover`, `pressable`, `toggle_group` |
| **Utility** | `direction_provider`, `theme_toggle` |

#### Domain-Specific Components (`src/components/`) — Top-Level

| Component | Status | Generics Served |
|---|---|---|
| `admin_module_sidebar.rs` | ✅ Built | Platform admin nav |
| `attribute_icon.rs` | ✅ Built | Listing attributes |
| `badge.rs` | ✅ Built | General |
| `card.rs` | ✅ Built | General |
| `configurator.rs` | ✅ Built | G-27 Scorecard template config |
| `crm_stage_bar.rs` | ✅ Built | G-31, CRM pipeline |
| `crm_timeline.rs` | ✅ Built | G-28 (notes) |
| `crm_timeline_generic.rs` | ✅ Built | G-29 (activity) |
| `data_table.rs` | ✅ Built | General data display |
| `email_composer.rs` | ✅ Built | Email / CRM comms |
| `file_attachments.rs` | ✅ Built | G-02 Vault |
| `icon.rs` | ✅ Built | General |
| `modal.rs` | ✅ Built | General |
| `properties_editor.rs` | ✅ Built | Listing/entity properties |
| `tabs.rs` | ✅ Built | General |
| `theme_provider.rs` | ✅ Built | Design system |
| `version_banner.rs` | ✅ Built | Platform-wide deployment detection banner |

#### Scorecard Components (`src/components/scorecard/`)

| Component | Status | Purpose |
|---|---|---|
| `models.rs` | ✅ Built | Shared scorecard data models for frontend |
| `sections/display_rules.rs` | ✅ Built | G-27 display rule configuration UI |
| `widgets/scorecard_widget.rs` | ✅ Built | G-27 embedded scorecard widget |
| `widgets/nudge_prompt.rs` | ✅ Built | G-27 prompt-to-rate nudge widget |

#### Auth Components (`src/components/auth/`)

| Component | Status | Purpose |
|---|---|---|
| `atlas_login_panel.rs` | ✅ Built | Shared login panel (all apps) |
| `passkey_login.rs` | ✅ Built | Passkey auth flow |
| `passkey_manager.rs` | ✅ Built | Manage registered passkeys |
| `passkey_nudge.rs` | ✅ Built | Prompt to register passkey |

#### Reactive Hooks (`src/components/hooks/`) — 28 hooks

| Hook | Purpose |
|---|---|
| `use_breadcrumb` | Breadcrumb state management |
| `use_can_scroll` / `use_can_scroll_vertical` | Scroll boundary detection |
| `use_cell_edit` / `use_cell_selection` / `use_drag_selection` | Data grid cell interaction |
| `use_click_outside` | Outside-click detection |
| `use_column_state` | Column layout state |
| `use_copy_clipboard` | Clipboard write |
| `use_data_grid_state` / `use_data_scrolled` | Data grid scroll state |
| `use_form` | Form state management |
| `use_handle_day_click` | Date picker day interaction |
| `use_history` | Browser history integration |
| `use_horizontal_scroll` | Horizontal scroll tracking |
| `use_is_mobile` | Responsive breakpoint detection |
| `use_lock_body_scroll` (+ dialog/popover variants) | Modal scroll locking |
| `use_locks` | Shared lock primitives |
| `use_media_query` | CSS media query reactive signal |
| `use_pagination` | Pagination state |
| `use_press_hold` | Long-press interaction |
| `use_random` | Random value generation |
| `use_theme_mode` | Dark/light mode toggle |
| `use_version_check` | Headless deployment detection; polls `/api/version`, returns `ReadSignal<bool>` |
| `use_virtual_scroll` | Virtual list rendering |

#### Missing UI Surfaces (Backend Deployed, No Dedicated Frontend Page)

| Generic | Backend Status | Gap |
|---|---|---|
| G-07 Realtime | Deployed with API | No realtime presence/chat UI surface (str_host/messages.rs uses it but no standalone WS room UI) |
| G-17 Tax | Deployed with API | Partial only — ledger/statements surface tax events but no dedicated tax filing UI |
| G-20 Attribution | Deployed with API + campaign-scoped admin panel | Campaign attribution tab + CAC; waitlist/LP/OTP wired; Stripe checkout conversion |
| G-21 Events | Deployed with API | No event management UI |
| G-22 Record Relationships | Deployed with API | No relationship graph/timeline UI |
| G-24 Quotes | Deployed with API | No quote builder/viewer UI |
| G-25 Commission Plans | Deployed with API | No commission plan editor UI |
| G-28 Notes | Entity defined | Partial — crm_timeline + communications page exists but no standalone notes page |
| G-29 Activities | Entity defined | Partial — crm_timeline_generic exists but no standalone activity log page |
| G-35 Notifications | Deployed with API | Partial — landlord/notifications.rs built; no notification bell for tenant/vendor/str_host portals |
| `atlas_lp_events` | Migration complete | No LP event analytics UI |
| Persona self-serve referrals (all roles) | Stitch Rev 3 | Backlog — unified `/refer/:code` → onboard path; Leptos not started beyond F&F landlord/vendor |

---

## Infrastructure Components

| Component | Technology | Purpose |
|-----------|-----------|---------|
| Backend API | Rust + Axum | Headless REST API, port 8000 |
| Frontend | Leptos (SSR + WASM) | Anchor CMS + Network Instance + Folio PM apps |
| Platform Admin | Leptos CSR | Admin UI for tenant management; **Operations → System Status** (`/ops/status`) — deploy-safe env hierarchy + health + sanitized metrics (`GET /api/admin/system-status`). Command Center (`/`) remains product/fleet telemetry. |
| PM Backend (Folio) | Rust + Axum | 24+ handler files, 40+ PM services |
| Syndication Worker | Rust + Tokio | G-05: 10s polling, HMAC delivery, back-off, dead-letter |
| Database | PostgreSQL (shared) | Multi-tenant, tenant_id scoped |
| Cache | Moka (in-process) | AppInstance + WebAuthn instances |
| Object Storage | Cloudflare R2 | Vault: leases, media, exports |
| Observability | Prometheus `/metrics` | Bearer-token protected scrape endpoint |
| Proxy | K8s Ingress (Traefik) | Host-header-preserving reverse proxy via K3s |
| Local CLI | Rust + Clap (`tools/atlas-local`) | **`atlas-local`** — default **parity** `up` (baked backend ≈ K8s); `up --hot` / `watch` for cargo mounts; `status` (Overview/Resources/Telemetry/**Env** + **Next steps**; Env tab sets SMTP → `.env.local`, **apply** recreates backend); `refresh`; `env` (`.env.local` get/set/edit + `env smtp`); `db info` / `db pull`. Extend this CLI for new local automation ([`docs/architecture/local_development.md`](architecture/local_development.md)). **CI:** post-deploy `validate_atlas_local_cli` with `failure: ignore` (advisory only). |
| Ingress Sidecar | Rust binary (`ingress_sidecar.rs`) | `POST /api/ingress/provision` + `deprovision`; creates K8s Ingress + cert-manager TLS per tenant domain. Supports ANY domain — platform subdomains or fully custom client-owned domains |
| TLS (`*.atlas.oply.co`) | `wildcard-tls-prod` Secret (cert-manager DNS-01) | Issued once via `letsencrypt-cloudflare` ClusterIssuer (Cloudflare API token required). Requires **one-time cluster bootstrap** — see `docs/architecture/tls_and_custom_domains.md`. After bootstrap, every `*.atlas.oply.co` app gets TLS automatically |
| TLS (custom client domains) | Per-domain cert via `letsencrypt-http` (HTTP-01) | Any domain the client owns: `pm.clientco.com`, `tracker.mybusiness.io`, etc. cert-manager auto-issues within ~60s of DNS propagation. Client points A/CNAME at cluster IP; platform-admin shows the exact record |
| CI/CD | Woodpecker CI | Build, test, lint, deploy pipeline |
| CI/CD RBAC | `woodpecker-deployer` SA | ✅ Bootstrapped 2026-07-02 into atlas-dev + atlas-uat; enables automated domain provisioning |
| Deployment Detection | `use_version_check` (shared-ui) | Polls `/api/version` every 5 min; shows `<VersionBanner>` on SHA change |
| Payment Rails | `pm/rails/` | Bitcoin on-chain, Lightning, Stripe Connect, InfinitePay, Kelviq — multi-rail payment support |

---

## Current Status Summary (July 11, 2026 — Rev 11)

| Area | Status |
|------|--------|
| Platform Generics (G01–G08) | ✅ Fully deployed with API |
| G05 Syndication Event Bus | ✅ Fully deployed with API + Full UI |
| G06 Verification | ✅ **Rev 11 Full UI** — admin review + Folio submit + reviewer notes |
| G08 AI Tasks | ✅ **Rev 11 Full UI** — live worker + pause (was Partial) |
| G19 Campaigns | ✅ Fully deployed with API + Full UI (+ F&F landlord/vendor seeds + `global_name`) |
| G20 Attribution | ✅ Fully deployed with API — No UI |
| G21 Events | ✅ Fully deployed with API — No UI |
| G22 Record Relationships | ✅ Fully deployed with API — No UI |
| G23 Reservations | ✅ Fully deployed with API + Full UI (+ `atlas_bookings`) |
| G24 Quotes | ✅ Fully deployed with API — No UI |
| G25 Commission Plans | ✅ Fully deployed with API — No UI |
| G26 Catalog | ✅ Fully deployed with API + Full UI |
| G27 Atlas Scorecards | ✅ Full implementation + analytics + UI + deployments + Feedback push |
| G28 atlas_note | ✅ Entity + migration; crm_timeline UI + landlord communications |
| G29 atlas_activity | ✅ Entity + migration + backfill; crm_timeline_generic UI |
| G31 atlas_lead | ✅ Full lifecycle + Full UI (all 4 apps + public lead_portal) |
| G32 atlas_rbac | ✅ Deployed with API + Partial UI + per-asset access |
| G33 App Deployment Config | ✅ Deployed with API + Internal Instances UI |
| G34 Vendor Marketplace | ✅ Deployed + Partial UI |
| G35 Notifications | ✅ Deployed with API + Partial UI (landlord only) |
| G36 Programs | ✅ Deployed with API + **Full UI** (admin `/programs` + instance Growth panel + Folio NetworkInvite) |
| G37 Ambassadors | ✅ Deployed with API + **Full UI** (admin `/ambassadors` + Folio refer paths) |
| Feature Flags | ✅ **Rev 11 Deployed with API + Full UI** — `FlagService` + instance enablements |
| G14 Documents | ✅ Deployed with API + Partial UI |
| G18 Applications | ✅ Deployed with API + Partial UI |
| Migrations | ✅ **237** files — latest `m20261103_*` (acquisition feature flags + G-19 DM companions) |
| PM Frontend App (Folio) | ✅ **9 role portals**; landlord **35** pages; **144** page modules; marketing **19**; onboarding **15** |
| Folio Landlord Onboarding | ✅ **Rev 11** stitch parity (`wiz_landlord_onboard`) + magic-link session peek |
| Folio Login | ✅ Dark-harmonized `pub_login_v3` |
| platform-admin Ops | ✅ **Rev 11** products click-through, support notes, audit CSV, AI worker |
| platform-admin pages | ✅ **113** page modules |
| network-instance | ✅ **13** pages + G-27 `scorecard_mount` |
| anchor | ✅ **22** pages |
| shared-ui Primitive Count | ✅ **85** primitive UI components |
| shared-ui Configurator | ✅ Built |
| Test Suite | ✅ **43** test files (27 integration + 16 unit) |
| Outbox / Workers | ✅ Active — outbox job types + SyndicationEventBus + **AI task worker** |
| Security hardening | ✅ Invariants verified; RBAC + syndication guards |
| Party Unification | ✅ Schema + handlers + data migration |
| Legacy CRM dual-write | ⏳ Deprecation path still active |
| PostGIS in CI | ⚠️ Recommended but not enforced |

---

## What a New Developer / AI Should Know

- **Local development uses `atlas-local`.** From `atlas-platform/`: `cargo run -p atlas-local -- up` (**parity** by default — same backend shape as K8s). Use `up --hot` only for volume-mounted cargo iteration. `atlas-local status` shows Resources/Telemetry and **Next steps** when something fails. Do **not** add ad-hoc `scripts/*.sh` for local ops — extend [`tools/atlas-local`](../tools/atlas-local) and document in [`docs/architecture/local_development.md`](architecture/local_development.md). Local WebAuthn is isolated (`RP_ID=localhost` in `.env.local`); never copy those values into K8s overlays.
- **Deployed ops visibility uses platform-admin System Status** (`/ops/status`) — Environment → Tenant → App → Domain hierarchy, health, application capacity, sanitized counters. Docker/Compose and `METRICS_TOKEN` stay off the browser; see local_development.md “Platform Admin System Status”.
- **Rule 7 before any new table.** Run the Generic Fitness Test in [`docs/architecture/generic_fitness_test.md`](architecture/generic_fitness_test.md). Prefer USE EXISTING / EXTEND JSONB / EXTEND COMPANION under an existing G-id over inventing a new G-number. Use **this file** as the registry of what already exists.
- **Folio magic-link → onboarding** — Fresh users often have a valid `session` cookie while `GET /api/folio/me` returns 403 (no Folio role yet). WizardShell uses `peek_auth_session()` (`GET /api/auth/session/validate`) so they skip OTP and land on step 1. Do not gate onboarding solely on `/api/folio/me`.
- **Landlord onboarding design source** — stitch `designs/stitch/project_pm/folio/wiz_landlord_onboard/code.html`. Shell + steps must stay in parity.
- **`FlagService`** — resolve flags with instance enablements (`atlas_flag_instance_enablements`). Prefer Features tab on instances over global catalog toggles for per-tenant control.
- **G-06 verification** — mock seed removed; use create API + Folio submit + reviewer notes / request-more-info. Approve updates account status.
- **G-31 is the canonical lead entity.** Do not use `entities/lead.rs` (legacy). Use `entities/atlas_lead.rs` and `services/lead_service.rs`.
- **G-28 and G-29 are the canonical note and activity entities.** Do not extend `entities/note.rs` or `entities/activity.rs`.
- **G-32 RbacService is the authoritative source for roles and permissions.** Do not query `atlas_user_app_roles` directly. Handlers use the `require_rbac_permission()` helper. Role validation supports wildcards (e.g. `billing:*`). `services/rbac.rs` is now a standalone file.
- **G-33 `folio_mode` is a typed DB column** (`standard | pmc | brokerage`) with a CHECK constraint since `m20260909`. The old `pmc_enabled` JSON boolean no longer exists. Use `FolioMode` enum and `ActiveEnum::into_value()` to get DB strings.
- **G-33 `atlas_app_deployment_config` now has an `account_id` FK** (m20260918). This links deployment config directly to a billing account for managed-service billing.
- **G-33 AppDeploymentConfig Axum extractor** is used to resolve multi-tenant deployment modes (e.g. Standard vs InternalOperator) and config payloads from `atlas_app_deployment_config` table. Defaults to `"folio"` app slug.
- **G-34 Vendor Marketplace is an opt-in extension.** Opt-in visibility controlled by `is_marketplace_visible`. PostGIS `ST_DWithin` + `ST_Distance` for geo matching. Three UI surfaces: `landlord/contractor_marketplace.rs` (browse), `vendor/network_profile.rs` (edit), `property_owner/find_vendor.rs` (NEW Rev 10).
- **G-35 NotificationService** — create notifications by `tenant_id` + `user_id` + `entity_type`/`entity_id` polymorphic target. User preferences control delivery channels. Landlord notification center UI is now built (`landlord/notifications.rs`); other portals still lack a notification bell.
- **G-36 ProgramService** — productized growth programs (NetworkInvite first). Optional `campaign_id` links to G-19 for attribution. Actions use invite codes / platform invites as delivery rails. Outcomes and reward grants are first-class; billing application of rewards is not wired in v1.
- **G-37 Ambassadors** — growth partners (referral / influencer / affiliate / recruiter) with UNIQUE codes, M:N campaign attach, dual audience QR PNGs (`ReferAudience`), and JSONB fulfillment stubs. Promoted via Rule 7 ([`generic_fitness_test.md`](architecture/generic_fitness_test.md)); rewards stay G-36. Admin: `/ambassadors` + `/api/admin/ambassadors*`.
- **G-05 SyndicationEventBus** runs as a separate Tokio worker (10s interval). `enqueue()` is transactional — call it inside the same DB transaction as the business event. Only `Active` links with a non-null `inbound_webhook_url` receive delivery attempts. Skipped links are still audited.
- **G-27 ScorecardService** requires `tenant_id` on every call. All `find_*` methods are tenant-scoped.
- **G-19 campaigns now support hierarchy** via `parent_id` (m20260919). A campaign can be a sub-campaign of another.
- **`submit_entry` validates session ownership** before accepting entries — callers must supply `tenant_id`.
- **LeadService::disqualify** will return `Err` if the lead is already in a terminal state (`converted` or `disqualified`).
- **LeadService::create_from_import** auto-provisions a scorecard when `data_source` is `fmcsa`, `business_leads_usa`, or `dot_registry`.
- **`with_state(db)` is called EXACTLY ONCE at the AtlasApp boundary.** Never call it inside handler route constructors used inside an `AtlasApp`.
- **FolioMode and AppDeploymentMode are independent axes.** `FolioMode` = operational identity of a Folio instance. `AppDeploymentMode` = operator topology. They share the string `"standard"` but are type-distinct SeaORM enums.
- **The `brokerage` FolioMode** enables agent (`/a`) and broker (`/br`) portals in `apps/folio`. A Folio instance cannot be `pmc` AND `brokerage` simultaneously — enforced by DB CHECK constraint.
- **The STR host portal** (`/str`) is a new 8th portal in `apps/folio`. It is separate from the landlord portal STR compliance page — `str_host/` covers operational hosting (bookings, pricing, channel management), while `landlord/str_compliance.rs` covers regulatory licensing (G-16).
- **Property owner portal** (`/po`) is the 9th role portal in folio (Rev 10). It surfaces property value, vendor search, and review submission for property owners distinct from the owner equity portal (`/o`).
- **`atlas_user_asset_access`** (Rev 10) — per-asset access grants table for cohost/delegate/vendor scoping. Entity defined at `entities/atlas_user_asset_access.rs`; no management UI yet.
- **`atlas_bookings`** (Rev 10) — a separate table from `atlas_reservations`. Bookings are confirmed; reservations are holds/availability.
- **Platform-admin `get_session()`** — ALWAYS use `get_session()` (not raw `validate_session()`) in platform-admin components. 15-second TTL. Call `cache_clear()` before `logout()`.
- **Tachys hydration in Leptos SSR** — Use `<Show>` component for conditional rendering, NOT `if` blocks returning `Option<impl IntoView>`. Plain `if` blocks cause hydration mismatches (panic at hydration.rs:227 “internal error: entered unreachable code”). See `docs/leptos_architecture_decisions.md §5.5`.
- **`ContactResponse` is the API response shape** for `/api/contacts` — it has `full_name: Option<String>` (built from first + last), NOT a `name` field. Do NOT deserialize contact API responses into the `Contact` DB model in tests.
- **All Leptos instance component root divs MUST have `w-full`.** In the platform-admin sidebar flex layout, a missing `w-full` causes descendants to collapse to content width. This invariant applies to: the `AppInstance` page wrapper div, and each instance subcomponent root view div.
- **Sessions are Postgres-backed.** Pod restarts do NOT expire user sessions. Deployment detection is now handled by `use_version_check` / `<VersionBanner>` in shared-ui.
- **`/api/version` is a public endpoint.** Returns `{ version, build_sha, build_date, environment }`. No auth required. The `X-Atlas-Version` header is also injected on every API response.
- **Internal Instances vs App Instances**: `pages/apps/` is for all tenant app instances (any operator). `pages/internal_instances/` is specifically for `deployment_mode: internal_operator` instances managed by the platform team. They use different provisioning flows and share the same backend, but the UI is separated for clarity.
- **Automated domain provisioning** is operational. The flow is: platform-admin UI → `POST /api/admin/instances/:id/domain` → `IngressProvisioner` → ingress-sidecar → K8s Ingress + cert-manager Certificate. The `bootstrap_rbac` CI step (completed 2026-07-02) is what enabled this.
- **TLS for `*.atlas.oply.co` requires a one-time cluster bootstrap** — the `wildcard-tls-prod` Secret must be created by cert-manager before any `*.atlas.oply.co` Ingress can serve HTTPS. Apply `k8s/cluster-setup/cluster-issuers.yaml` (creates `letsencrypt-cloudflare` + `letsencrypt-http` ClusterIssuers) and `k8s/overlays/{env}/wildcard-cert.yaml` once per cluster. After that, CI manages renewals automatically. See `docs/architecture/tls_and_custom_domains.md`.
- **Custom client domains are fully supported** — any domain (`pm.clientco.com`, `app.mybusiness.io`, etc.) works end-to-end via HTTP-01. The sidecar assigns `cert-manager.io/cluster-issuer: letsencrypt-http` automatically for non-`*.atlas.oply.co` domains.
- **Payment rails are now a distinct PM sub-layer** (`pm/rails/`). Each rail file wraps a payment processor: Stripe Connect (USD), InfinitePay (Brazil), Kelviq, Bitcoin on-chain, Lightning. Use these services — not direct API calls — to process payments.
- **Market configs** (`pm/market/`) contain regulatory and pricing rules for specific geographic markets (Brazil, Miami, USVI). Consult these before implementing market-specific pricing or compliance logic.
- **Launching a product app requires Phase 1 (CI/CD + K8s) AND Phase 2 (DB registration).** Missing Phase 2 causes a 404 on the marketing homepage even when the pod is healthy. See `docs/architecture/adding_a_new_app.md` for the full checklist and `docs/architecture/product_page_system.md` for the two-layer content resolution algorithm.
- **The product page API (`GET /api/pub/products/:slug`) uses a two-layer waterfall**: GTM Landing Page Builder page (if published) → `product_page_templates` fallback → 404. `launch_mode = "draft"` renders `<NotFound/>` on the frontend even on 200. Always set `launch_mode` to `"waitlist"` or `"active"` for live products.
- **`SendWaitlistConfirmation` is the 7th OutboxJobType.** It fires within 1.5s of a new `atlas_lead` insert. The job is non-fatal (fires-and-forgets) and dedup-safe (only enqueued for new leads, not duplicate signups). SMTP routing: tenant settings → `SMTP_*` env vars → localhost mock. From address: `SMTP_FROM` env var.

---

## Key Documents

- `docs/CURRENT_STATE.md` — **This file.** Ground-truth implementation status.
- `docs/famtasm/` — Famtasm creator paywall platform: `product_technical_specification.md` (Rev 2), `atlas_integration_mapping.md` (Rev 2), `ui-spec/` (12-page UI specification suite).
- `docs/architecture/local_development.md` — **Local loop:** `atlas-local` CLI, Compose/Caddy, WebAuthn isolation, `db pull` sandbox, CLI extension policy; **Platform Admin System Status** vs host `atlas-local status`.
- `docs/cicd_security_hardening.md` — CI/CD security architecture + `bootstrap_rbac` completion + automated domain provisioning documentation.
- `docs/architecture/` — Architecture diagrams and system design docs.
- `docs/diagrams/` — ERDs and flow diagrams.
- `docs/reports/` — Market analysis and sales analysis reports.
- `docs/backlog/telegram_miniapp_architecture_guide.md` — TMA architecture guide.
- `docs/backlog/telegram_network_instance_mvp_spec.md` — network-instance TMA MVP spec.
- `.agents/workflows/` — Reusable agent workflows (update-current-state, generate-market-report, generate-technical-sales-analysis, **generate-gtm-strategy**, etc.).
- `docs/prompts/gtm_strategy_prompt.md` + `docs/reports/gtm/` — App beachhead GTM strategy packs -> LaTeX PDF via `generate_market_reports --report-type gtm`. Exemplar operating pack: `gtm/folio-bristol-miami/` (Orbit root).

---

## Recommended Follow-Up Work

### Highest Value Frontend Gaps (Backend Deployed, No UI)

1. **G-24 Quote Builder** — Full quote + line item UI for property deals and service agreements.
2. **G-21 Event Management** — Event creation, ticket management, and registration UI.
3. **G-22 Record Relationship Graph** — Timeline/graph view linking records polymorphically.
4. **G-25 Commission Plan Editor** — Visual editor for commission split configurations (brokerage FolioMode).
5. **G-20 Attribution Dashboard** — Marketing channel attribution reporting UI.
6. **G-35 Notification Bell (tenant/vendor/str_host)** — Landlord notification center exists; extend to other portals.
7. **LP Events Analytics** — Analytics view for `atlas_lp_events` conversion tracking.
8. **G-17 Tax Filing UI** — Dedicated tax event/filing review page.
9. **Persona self-serve referrals (all roles)** — Stitch Rev 3 ready (`_referrals_system.md`); Leptos path beyond F&F landlord/vendor not started.

### Backend Gaps

1. **G-28 / G-29 Standalone Services** — Promote to full `NoteService` / `ActivityService`.
2. **G-05 Inbound Webhook Handler** — Outbound delivery complete; inbound receiver not built.
3. **G-07 Realtime Presence UI** — WS infra deployed; general-purpose presence surface missing.
4. **G-35 Notification Worker** — Delivery worker (push / email digests) not yet in `OutboxWorker`.
5. **G-36 entity modules** — Program tables exist via migration; confirm SeaORM `atlas_program*` entity files if missing from `entities/`.

### Infrastructure

1. **Extend `atlas-local` CLI** — Prefer new subcommands over scripts: `seed network`, `smoke`, `doctor` (PostGIS / port / WebAuthn env). See [`architecture/local_development.md`](architecture/local_development.md).
2. **Remove `bootstrap_rbac` CI step** — Safe once pipeline confirms no RBAC errors.
3. **PostGIS in CI** — Enforce in test matrix for geo-dependent tests.
4. **Syndication Worker Health Check** — Expose dead-letter counter on Prometheus `/metrics`.
5. **`VersionBanner` Adoption** — Adopt in `anchor`, `network-instance`, and `folio` (currently platform-admin).
6. **STR Host Portal Route Registration** — Keep verifying `str_host/` shells in `apps/folio/src/app.rs`.