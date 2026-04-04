# Unified Content Architecture Migration
This task checklist tracks the migration of `jobs`, `certifications`, and `projects` into the centralized `resume_entries` table.

- `[x]` Consolidate all 24 migrations into a Greenfield `initial_schema.sql` and `initial_data.sql`.

- `[x]` Backend Implementation
  - `[x]` Update BaseResumeEntry and ResumeEntry structs to support `metadata JSONB`
  - `[x]` Update API Endpoints to persist metadata natively

- `[x]` Admin UI Refactoring
  - `[x]` Add dynamic JSON payload mapping to `BaseResumeEntryForm`
  - `[x]` Refactor `admin.rs` to replace standalone tabs with unified Lists

- `[x]` Cleanup & Verification
  - `[x]` Resolve strict type scoping errors and compilation issues.
  - `[x]` Fix PostgreSQL `resume_category_enum` vs `VARCHAR` mapping panics
  - `[x]` Drop deprecated table components from frontend pages
  - `[x]` Fix public navigation ENUM filters and resolve `/work` 404 Route
  - `[x]` Delete redundant API function logic (`get_jobs`, `get_projects`)

- `[x]` Post-Migration Enhancements
  - `[x]` Default public profile seeded via `initial_data.sql` mapping rule
  - `[x]` Category-specific Quick Action buttons added to Admin Panel

- `[x]` Architecture Enhancements
  - `[x]` Add `full_name` column to `resume_profiles` to isolate internal names from PDF Document headers.
  - `[x]` Add `'certification'` explicitly to `resume_category_enum` mapping (SQL + Rust) and update seed logic.
  - `[x]` Implement `category_order JSONB` on `resume_profiles` to dictate display sequence globally.
  - `[x]` Update Admin UI `AdminEditorModal` forms to persist the new properties (via Interactive Sortable UX arrows).
  - `[x]` Refactor `resume_engine` PDF LaTeX compiler to honor `full_name` and sequential `category_order`.

- `[x]` Profile Overrides & Dynamic Sequence Filtering
  - `[x]` Database: Append `overrides JSONB` column to `resume_profile_entries`.
  - `[x]` Backend Engine: Update `GetResumeEntries`, `AddResumeProfile`, `UpdateResumeProfile` endpoints with the `ProfileEntryMapping` struct to query and persist override mappings.
  - `[x]` Backend Engine: Update `generate_latex_string` to overwrite the `ResumeEntry` variables conditionally if properties exist and aren't blank.
  - `[x]` Admin Interface: Inject the `Edit Overrides` nested form to input strings binding Reactively to the central HashMap, wiping keys on empty strings.
  - `[x]` Admin Interface: Add `.filter(|(_, cat_name)| get_reactive_vis(cat_name))` to Sequence Ordering mapping.

# Infrastructure & Orchestration
- `[x]` Kubernetes Orchestration (OrbStack)
  - `[x]` Define a dedicated `namespace.yaml` to isolate `ruuderie-ai` from other cluster tenants.
  - `[x]` Create `postgres.yaml` for containerized dev database with PVC mounting.
  - `[x]` Create `app.yaml` embedding dynamic ingress (`ruuderie.orb.local`) mapped to the local docker image.
