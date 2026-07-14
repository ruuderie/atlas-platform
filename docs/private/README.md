# Private docs (local only)

**Everything under `docs/private/` is intentionally not published to GitHub.**

## Public allowlist (everything else → private)

Only these stay in the public `docs/` tree:

| Path | Why public |
|------|------------|
| `docs/folio/` queue/checklist (`README`, `page_queue`, `multi_unit_parity_checklist`) | Public implementation queue only — **prompts stay private** |
| `docs/contracts/` | Shipped platform contracts |
| `docs/grafana/` | Ops dashboards |
| `docs/architecture/` eng how-tos listed below | Build / operate the open codebase |
| Root runbooks + ADRs listed below | Deploy, auth, Leptos, CI |

### Public `docs/architecture/` files

- `adding_a_new_app.md`
- `local_development.md`
- `auth_and_permissions.md`
- `tls_and_custom_domains.md`
- `leptos_resource_hydration.md`
- `domain_operator_runbook.md`
- `generic_fitness_test.md`
- `platform_generics.md` / `platform_generics_v2.md` / `platform_generics_v3.md`
- `platform_layer_map.md`
- `platform_manifest.json`
- `asset_metadata_shapes.md`
- `diagrams/` (engineering diagrams only)

### Public root docs

- `auth-security-observability-runbook.md`
- `cicd_security_hardening.md`
- `cloudflare_sealed_secrets_maintenance.md`
- `deployment_environments.md`
- `TEST_ENVIRONMENT_REQUIREMENTS.md`
- `leptos_architecture_decisions.md`
- `leptos_ssr_shell_pattern.md`
- `atlas_app_integration.md`
- `atlas_app_registry.md`
- `postgres_architecture.md`
- `admin-module-registry.md`
- `apps_walkthrough.md`
- `anchor_blocks_schema.md`
- `private/README.md` (this file)

## Private (examples)

- Future products, UI specs, Stitch dumps
- GTM / market / strategy / reports / backlog
- Research / agent prompts (`docs/private/prompts/` — including stitch→leptos)
- Folio product-surface specs (`docs/private/folio/`)
- Product planning (`g27/` vertical plans, CRM unification plans, CURRENT_STATE dumps, Phase notes)

## Rule for agents

New product, research, GTM, or planning writing goes under **`docs/private/`**, never under public `docs/` roots above.
