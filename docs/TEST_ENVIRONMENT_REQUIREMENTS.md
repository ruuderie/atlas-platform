# Test Environment Requirements

This document outlines the requirements for running the full test suite locally or in CI.

## Overview

The Atlas Platform backend uses a large number of integration tests that spin up a real PostgreSQL database, run all migrations, and execute against live data. These tests are located primarily in `backend/src/tests/`.

## Core Requirements

### 1. PostgreSQL with PostGIS Extension (Strongly Recommended)

Many tests (especially anything touching assets, geo service areas, or the full migration suite) depend on the PostGIS extension.

**Recommended Setup:**
- Use the official PostGIS Docker image: `postgis/postgis:16-3.4` (or a recent 15/16 version).
- In CI (e.g. Woodpecker, GitHub Actions, or your current runner), configure the database service to use a PostGIS-enabled image instead of the plain `postgres` image.

**Minimal docker-compose snippet for local testing:**
```yaml
services:
  db:
    image: postgis/postgis:16-3.4
    environment:
      POSTGRES_USER: postgres
      POSTGRES_PASSWORD: postgres
      POSTGRES_DB: oplydbtest
    ports:
      - "5432:5432"
```

If PostGIS is **not** installed on your test database:
- The `m20260601_g01_geo_postgis` migration will log a warning and skip creating the geo tables.
- Most tests will still pass.
- Any tests that deeply depend on spatial queries or the `geo_service_areas` table may behave differently or be skipped implicitly.

### 2. Database Connection for Tests

Tests use the database URL configured in the test environment. Common locations:

- `backend/.env`
- Environment variables in your CI configuration
- Hardcoded fallbacks in `src/tests/test_utils.rs` and `src/tests/api_tests.rs`

The test helper `initialize_database()` performs a destructive `DROP SCHEMA public CASCADE` + full re-migration on every test run. This is intentional for hermetic tests.

### 3. Other Considerations

- **SeaORM Migrations**: All tests go through `sea_orm_migration::Migrator::up()`.
- **Transaction Safety**: Migrations must be resilient to partial failures (see the hardening done in G01).
- **Enum Types** (G03): The payments migration creates several custom PostgreSQL ENUM types. The `DROP SCHEMA CASCADE` pattern ensures these are cleaned up between test runs.

## Current State (June 2026)

After the Platform Generics v2 merge, the test suite went through a stabilization phase:

- The G01 migration was made tolerant of missing PostGIS.
- The assertion in `core_platform` tests was updated to reflect that `CorePlatformApp` now registers all platform generics migrations.
- As of the latest runs, **all 118 tests pass** when the test database has the necessary extensions.

See `CURRENT_STATE.md` for more details on the history of the `25P02` test failures and their resolution.

## Recommended CI Configuration

For reliable full test coverage, ensure your CI database container uses a PostGIS image:

```yaml
# Example (Woodpecker / Docker Compose style)
services:
  postgres:
    image: postgis/postgis:16-3.4
    ...
```

If you cannot use PostGIS in CI yet, the suite will still run (with warnings), but geo-related functionality will be limited.

---

See [`CURRENT_STATE.md`](CURRENT_STATE.md) for the full history of the test suite stabilization work (including the `25P02` migration abort issues and their resolution).

**Last Updated:** June 2026
