# G-27 Scorecard Platform Contract

**Status:** Binding for stitch, shared-ui, platform-admin, and downstream apps.  
**Ground truth order:** SeaORM entities + `backend/src/types/scorecard.rs` → this contract → stitch HTML → Leptos/shared-ui.  
**Do not invent columns or endpoints.** If UI needs data that is not here, add a thin handler over existing service/SeaORM first, then update this doc.

Related: [`docs/architecture/g27/g27_scorecards_spec.md`](../architecture/g27/g27_scorecards_spec.md), [`docs/backlog/README.md`](../backlog/README.md) items 2–5.

---

## 1. Ownership layers

| Layer | What | Scope |
|-------|------|--------|
| **1. Platform catalog** | Templates with `template_scope=platform` | Benchmark-pool eligible. Still stored as rows with a `tenant_id` (provisioner seeds per tenant). Not a global singleton. |
| **2. Tenant templates** | `template_scope=tenant`, or extension dims (`is_tenant_extension=true`) on a platform copy | Private to that tenant; excluded from cross-tenant pools when scope is tenant / dim is extension. |
| **3. App-instance deployment** | `atlas_scorecard_template_deployments` | Controls which templates an app instance may list/use. |
| **4. Runtime** | `atlas_scorecards` + `atlas_rating_sessions` + entries | Always scoped by **`tenant_id`**. Sessions may stamp nullable **`app_instance_id`** (Folio/trigger paths). |

### Customization paths

1. **Deploy as-is** — enable deployment row; no structural change.
2. **Extend** — add dimensions with `is_tenant_extension=true` on the tenant’s platform template copy.
3. **Fork / custom** — create `template_scope=tenant` template in that tenant.

---

## 2. Platform-admin as pilot tenant

Platform-admin is the **first production consumer** of G-27 (pilot), not only an operator console.

### Two tenant contexts (never conflate)

| Context | Meaning | API usage |
|---------|---------|-----------|
| **Pilot tenant** | Operator org’s own `tenant_id` | CRM / tenant / app_instance **subjects** are rated here; scorecard rows live under pilot `tenant_id`. |
| **Customer tenant** | A selected customer’s `tenant_id` | Manage that customer’s templates, analytics, deployments (billing-style path param). |

### Operator UX lenses

1. **Pilot** — templates + analytics for the operator tenant; rate subjects via ScorecardWidget on detail pages.
2. **Catalog** — platform-scoped templates by product family (CRM, Folio, Network…), including templates the operator manages but does not personally rate.
3. **Customer** — pick customer `tenant_id`; templates / analytics / deployments for that tenant.
4. **Deployments** — per template, which app instances (including platform-admin’s own) have it enabled.

### Subjects you can rate in the pilot

| Subject | `subject_entity_type` | Mount |
|---------|----------------------|--------|
| CRM Account | `atlas_account` | CRM account detail |
| CRM Contact | `atlas_contact` | CRM contact detail |
| CRM Lead | `atlas_lead` | CRM lead detail |
| CRM Opportunity | `atlas_opportunity` | CRM deal detail |
| Customer tenant | `tenant` | `/tenants/:tenant_id` |
| App instance | `app_instance` | Instance detail |

`tenant` and `app_instance` are **additions** to `ScorecardEntityType` (not present until Phase 1 lands). CRM types already exist in `types/scorecard.rs`.

---

## 3. `template_scope` and `is_tenant_extension`

From migration `m20260801_pm_g27_template_scope`:

- **`template_scope=platform`** — canonical template; eligible for cross-tenant benchmark aggregation.
- **`template_scope=tenant`** — private per-tenant template; excluded from cross-tenant pool.
- **`is_tenant_extension=true`** on a dimension — landlord/app-added dim; excluded from cross-tenant benchmark pool.

---

## 4. Resource fields (entity ground truth)

### Template — `atlas_scorecard_templates`

`id`, `tenant_id`, `name`, `entity_type`, `description`, `scoring_method`, `default_scale_min`, `default_scale_max`, `min_entries_to_publish`, `is_published`, `template_scope`, `cold_start_strategy`, `cold_start_saturation_threshold`, `default_bayesian_prior_weight`, `calibration_minimum_entries`, `display_config` (JSONB → `ScorecardTemplateDisplayConfig`), `created_by_user_id`, `created_at`, `updated_at` (+ soft-delete if present).

### Dimension — `atlas_scorecard_dimensions`

`id`, `template_id`, `tenant_id`, `slug`, `name`, `description`, `category`, `weight`, `scale_type`, `scale_min`, `scale_max`, `unit_label`, `benchmark_tiers`, `global_reference_value`, `global_reference_label`, `min_entries_to_show`, `is_community_ratable`, `is_active`, `sort_order`, `is_inverted`, `bayesian_prior_weight`, `is_tenant_extension`.

### Scorecard — `atlas_scorecards`

`id`, `tenant_id`, `template_id`, `subject_entity_type`, `subject_entity_id`, `composite_score`, `confidence_level`, `total_contributors`, `total_sessions`, `total_entries`, `dimension_vector_v2`, `has_data_mask`, `last_computed_at`, `created_at`, `updated_at`, `deleted_at`.

**Not a column:** `trend_direction` on scorecards. Trend lives on `atlas_scorecard_time_series`. Leaderboard must join latest time-series (or omit the field).

### Dimension aggregate — `atlas_scorecard_dimension_aggregates`

`scorecard_id`, `dimension_id`, `mean_score`, `weighted_mean_score`, `percent_true`, `benchmark_label`, `benchmark_color`, `display_value`, `std_deviation`, `consensus_level`, `min_score`, `max_score`, `contributor_count`, `session_count`, `vs_global_delta`, `vs_global_label`, `percentile_rank`, `percentile_cohort_size`, `percentile_band`, `last_computed_at`.

### Time series — `atlas_scorecard_time_series`

PK `(scorecard_id, dimension_id, period_start, period_type)`. Fields: `mean_score`, `session_count`, `contributor_count`, `delta_from_prior`, `trend_direction`, `z_score`, `is_anomaly`, `anomaly_direction`.

`period_type`: `monthly` | `quarterly`.  
`trend_direction`: `improving` | `stable` | `declining` | `insufficient_data`.  
`anomaly_direction`: `spike` | `drop`.

### Session — `atlas_rating_sessions`

`id`, `scorecard_id`, `tenant_id`, `rater_user_id`, `occurred_at`, `session_type`, `context_entity_type`, `context_entity_id`, `session_label`, `status`, `verification_request_id`, `app_instance_id` (nullable FK → `app_instances`), `created_at`.

`status`: `draft` | `submitted` | `verified` | `disputed`.

`app_instance_id` is stamped on tenant/trigger open paths; admin open may pass it optionally.

### Entry — `atlas_scorecard_entries`

`id`, `session_id`, `scorecard_id`, `dimension_id`, `tenant_id`, `contributor_user_id`, `score`, `option_id`, `source_type`, `context`, `note`, `is_verified`, `verification_request_id`, `created_at`.

### Display rule — `atlas_scorecard_display_rules`

As in `DisplayRuleAdminView` / entity: `trigger_category`, `field_reference`, `operator`, `value`, `value_list`, `action`, `alert_message`, `mode_scope`, `priority`, `dimension_id`, `category_target`, `is_active`, …

### Display config — `ScorecardTemplateDisplayConfig`

`show_on_portfolio_table`, `show_on_anomaly_panel`, `show_on_leaderboard`, `show_on_maintenance_queue`, `show_on_property_detail`, `show_on_lead_card`, `show_on_public_listing`, `tenant_visible`, `nudge_on_maintenance_case_close`, `nudge_on_str_checkout`, `min_entries_before_display`, `collapsed_by_default`.

### Deployment (new) — `atlas_scorecard_template_deployments`

`id`, `template_id`, `app_instance_id`, `tenant_id`, `is_enabled`, `trigger_event` (MVP default `manual`), `trigger_context_entity_type` (optional), `created_at`.  
Unique `(template_id, app_instance_id)`.

### Analytics DTOs (existing service)

- **PortfolioStats:** `template_id`, `tenant_id`, `total_scorecards`, `refreshed_at?`, `dimensions: [DimensionPortfolioStats]`
- **DimensionPortfolioStats:** `dimension_id`, `dimension_slug`, `dimension_name`, `cohort_size`, `pool_mean?`, `pool_std_dev?`, `pool_min?`, `pool_p25?`, `pool_p50?`, `pool_p75?`, `pool_p90?`, `pool_max?`, `improving_count`, `declining_count`
- **LeaderboardEntry:** `rank`, `scorecard_id`, `subject_entity_id`, `subject_entity_type`, `composite_score?`, `confidence_level`, `percentile_rank?`, `trend_direction?` (from time-series join)
- **AnomalyAlert:** `scorecard_id`, `dimension_id`, `dimension_slug`, `dimension_name`, `period_start`, `mean_score?`, `z_score?`, `anomaly_direction?`

---

## 5. Entity-type vocabulary

### Global (`types/scorecard.rs` — after Phase 1)

Stored strings include existing variants plus:

- CRM / party: `atlas_lead`, `atlas_opportunity`, `atlas_account`, `atlas_contact`, …
- Pilot ops: **`tenant`**, **`app_instance`**
- PM provisioner also writes: `str_property`, `rental_unit`, `contractor`, `wholesale_lead` (PM enum in `types/pm.rs`)

Stitch filters and Configurator selects must only offer values that can exist in DB for the relevant product family. Do not use mock-only lists.

---

## 6. HTTP contract

### Platform-admin (explicit `tenant_id` in path)

| Method | Path | Purpose |
|--------|------|---------|
| GET/POST | `/api/admin/tenants/{tenant_id}/scorecard-templates` | List / create |
| GET/PATCH | `/api/admin/tenants/{tenant_id}/scorecard-templates/{id}` | Get / update (settings, publish, calibration, `display_config`) |
| GET/POST | `/api/admin/tenants/{tenant_id}/scorecard-templates/{id}/dimensions` | List / create dimensions |
| PATCH | `/api/admin/tenants/{tenant_id}/scorecard-dimensions/{dim_id}` | Update / deactivate (`is_active=false`) |
| GET | `/api/admin/tenants/{tenant_id}/scorecards/{id}` | Detail + aggregates |
| GET | `/api/admin/tenants/{tenant_id}/scorecards/{id}/sessions` | Sessions |
| GET | `/api/admin/tenants/{tenant_id}/scorecards/{id}/sessions/{sid}/entries` | Entries |
| GET | `/api/admin/tenants/{tenant_id}/scorecards/{id}/time-series` | `?dimension_id&period_type` |
| POST | `/api/admin/tenants/{tenant_id}/scorecards/{id}/recompute` | Recompute aggregates |
| POST | `/api/admin/tenants/{tenant_id}/scorecards/get-or-create` | Body: `template_id`, `subject_entity_type`, `subject_entity_id` |
| POST | `/api/admin/tenants/{tenant_id}/scorecards/{id}/sessions` | Open session |
| POST | `/api/admin/tenants/{tenant_id}/scorecard-sessions/{sid}/entries` | Submit entry |
| GET/POST | `/api/admin/tenants/{tenant_id}/scorecard-templates/{id}/analytics` | Portfolio stats |
| GET | `.../leaderboard?limit=` | Leaderboard |
| GET | `.../anomalies?limit=` | Anomalies |
| POST | `.../analytics/refresh` | Refresh |
| GET | `/api/admin/scorecard-templates/catalog` | Catalog lens |
| GET/PUT | `/api/admin/tenants/{tenant_id}/app-instances/{instance_id}/scorecard-deployments` | Instance deployments |
| GET/PATCH | `/api/admin/scorecard-templates/{id}/deployments` | Template deployments |

### Existing (keep)

- Display-rules: `GET /api/admin/scorecard-templates/{id}/display-rules`, `POST/PATCH/DELETE /api/admin/scorecard-display-rules[/{id}]`
- Tenant-facing analytics: `GET/POST /api/scorecard-templates/{id}/analytics|leaderboard|anomalies|refresh`
- `PATCH /api/scorecard-entries/{entry_id}/verify`
- Tenant write path (app-instance runtime): `POST /api/scorecards/get-or-create`, `POST /api/scorecards/{id}/sessions`, `POST /api/scorecard-sessions/{sid}/entries`, `GET /api/scorecard-sessions/pending`, `GET /api/scorecard-templates/{id}/dimensions`

Runtime event map: [`docs/architecture/g27/g27_app_instance_runtime.md`](../architecture/g27/g27_app_instance_runtime.md).

### App-instance list (additive)

Tenant/app Configurator lists **only templates deployed + enabled** for the current `app_instance_id` (header/domain resolution).

| Method | Path | Purpose |
|--------|------|---------|
| GET | `/api/scorecard-templates?app_instance_id=&is_published=` | Deployed+enabled templates for the instance. Resolves instance from query → `x-app-instance-id` → tenant's first `property_management` instance. Foreign instance → 403. |
| GET | `/api/scorecard-templates/{id}` | Deployed+enabled template detail (incl. `display_config`). |
| PATCH | `/api/scorecard-templates/{id}` | TenantAdmin-safe: `display_config`, `description` only. Locked fields (`template_scope`, `is_published`, `entity_type`) → 403. |
| GET/POST | `/api/scorecard-templates/{id}/display-rules` | List active rules / create rule (deployed-gated). |
| PATCH/DELETE | `/api/scorecard-display-rules/{id}` | Update / soft-delete rule (deployed-gated). |

**Auth:** `/api/admin/*` = platform operator session. Every handler verifies resource `tenant_id` matches path (or catalog rules). Foreign tenant → 403/404.

---

## 7. Analytics filters

| Filter | Phase A (ship) | Phase B (later) |
|--------|----------------|-----------------|
| Leaderboard / anomaly `limit` | Server | — |
| Dimension focus | Client on portfolio + anomalies | — |
| Anomaly direction | Client | Optional SQL |
| Entity type / confidence | Client on leaderboard | SQL for anomalies + portfolio |
| Trend | After time-series join fix | Server filter |
| Period type / time window | **Omit from UI** until SQL | Parametrize anomalies |

---

## 8. shared-ui consumer contract

| Component | Consumers | Notes |
|-----------|-----------|--------|
| `Configurator` | platform-admin, folio, network-instance | Modes: `PlatformOperator` \| `TenantAdmin` |
| `ScorecardWidget` | platform-admin (pilot first), folio, … | Props: `template_id`, `entity_type`, `entity_id`, `session_type`, optional `mode` |
| `NudgePrompt` | folio / apps | Activity-triggered |
| `DisplayRulesSection` | inside Configurator | Persist via display-rules API |

### Configurator modes

- **PlatformOperator** — may set `template_scope`, publish, deploy, calibration defaults.
- **TenantAdmin** — may edit tenant templates / extension dims / display rules for **deployed** templates; locked from breaking platform catalog identity fields (exact locks: implementation checklist in shared-ui).

### Save payload

`(TemplateForm, Vec<DimensionForm>, Vec<DisplayRuleForm>, DisplayConfigForm)` — or equivalent `TemplateSavePayload`. Rules and `display_config` must not be dropped on save.

Parent apps pass data + callbacks; shared-ui must not hardcode app API base URLs.

---

## 9. Platform-admin routes (Leptos)

| Route | Page |
|-------|------|
| `/billing/scorecards` | List + analytics (pilot / catalog / customer lenses) |
| `/billing/scorecards/templates/new` | Configure create |
| `/billing/scorecards/templates/:template_id/configure` | Configure edit |
| `/billing/scorecards/:scorecard_id` | Entity detail (read/verify) |
| CRM / tenant / instance detail | Embed `ScorecardWidget` for rating |

Deprecate flat `/billing/scorecards/session` rating wizard.

---

## 10. Stitch citation rule

Stitch HTML API comment blocks must cite **this file** (path + section), not “verified against backend” unless a grep of handler registrations confirms the route exists.

---

## 11. Out of scope (this contract revision)

- Full trigger-event matrix UX (`deal_close`, `lease_end`, …) beyond wired `post_checkout` / `case_resolved`.
- Landlord maintenance `status→closed` API (today `case_resolved` fires on vendor work-order complete).
- Transcript ingest / BYOC compute.
