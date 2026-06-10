//! PM aggregate metrics service.
//!
//! Computes per-client metrics for the PMC dashboard in a single SQL round-trip.
//! Uses raw SQL via Sea-ORM's `query_all` to express GROUP BY across multiple tables.
//!
//! # Query strategy
//!
//! Instead of N+1 queries (one per client account), we issue one query per metric
//! type using `GROUP BY managed_account_id`, then merge results in Rust.
//!
//! Alternatively, a single CTE-based query merges all metrics:
//!
//! ```sql
//! WITH
//!   asset_counts AS (
//!     SELECT managed_account_id, COUNT(*) AS unit_count
//!     FROM atlas_assets WHERE tenant_id = $1 AND managed_account_id IS NOT NULL
//!     GROUP BY managed_account_id
//!   ),
//!   lease_counts AS (
//!     SELECT managed_account_id,
//!            COUNT(*) AS active_lease_count,
//!            COUNT(*) FILTER (WHERE status != 'active') AS inactive_count
//!     FROM atlas_contracts WHERE tenant_id = $1 AND managed_account_id IS NOT NULL
//!     GROUP BY managed_account_id
//!   ),
//!   portfolio_counts AS (
//!     SELECT managed_account_id, COUNT(*) AS property_count
//!     FROM atlas_portfolios WHERE tenant_id = $1 AND managed_account_id IS NOT NULL
//!     GROUP BY managed_account_id
//!   )
//! SELECT
//!   a.id AS account_id,
//!   COALESCE(pc.property_count, 0) AS property_count,
//!   COALESCE(ac.unit_count, 0) AS unit_count,
//!   COALESCE(lc.active_lease_count, 0) AS active_lease_count,
//!   CASE WHEN COALESCE(ac.unit_count, 0) = 0 THEN 0.0
//!        ELSE COALESCE(lc.active_lease_count, 0)::float / ac.unit_count::float
//!   END AS occupancy_pct
//! FROM account a
//! LEFT JOIN portfolio_counts pc ON pc.managed_account_id = a.id
//! LEFT JOIN asset_counts      ac ON ac.managed_account_id = a.id
//! LEFT JOIN lease_counts       lc ON lc.managed_account_id = a.id
//! WHERE a.tenant_id = $1
//! ```

use sea_orm::{ConnectionTrait, DatabaseConnection, Statement};
use serde::Serialize;
use uuid::Uuid;

/// Aggregate metrics for one client account in the PMC dashboard.
#[derive(Debug, Clone, Serialize)]
pub struct ClientMetrics {
    pub account_id:          Uuid,
    pub property_count:      i64,
    pub unit_count:          i64,
    pub active_lease_count:  i64,
    /// Ratio of active leases to total units. 0.0 if no units.
    pub occupancy_pct:       f64,
}

/// Fetch aggregate metrics for ALL client accounts in the given tenant.
///
/// Returns one `ClientMetrics` row per account that has any managed data.
/// Accounts with zero portfolio rows are still returned (via LEFT JOIN on `account`).
pub async fn fetch_client_metrics(
    db: &DatabaseConnection,
    tenant_id: Uuid,
) -> Result<Vec<ClientMetrics>, sea_orm::DbErr> {
    let sql = format!(
        r#"
        WITH
          portfolio_counts AS (
            SELECT managed_account_id, COUNT(*) AS property_count
            FROM   atlas_portfolio
            WHERE  tenant_id = '{tid}' AND managed_account_id IS NOT NULL
            GROUP  BY managed_account_id
          ),
          asset_counts AS (
            SELECT managed_account_id, COUNT(*) AS unit_count
            FROM   atlas_asset
            WHERE  tenant_id = '{tid}' AND managed_account_id IS NOT NULL
            GROUP  BY managed_account_id
          ),
          lease_counts AS (
            SELECT managed_account_id, COUNT(*) AS active_lease_count
            FROM   atlas_contract
            WHERE  tenant_id = '{tid}'
              AND  managed_account_id IS NOT NULL
              AND  status = 'active'
            GROUP  BY managed_account_id
          )
        SELECT
          a.id                                                         AS account_id,
          COALESCE(pc.property_count,     0)                          AS property_count,
          COALESCE(ac.unit_count,         0)                          AS unit_count,
          COALESCE(lc.active_lease_count, 0)                          AS active_lease_count,
          CASE
            WHEN COALESCE(ac.unit_count, 0) = 0 THEN 0.0
            ELSE COALESCE(lc.active_lease_count, 0)::float / ac.unit_count::float
          END                                                          AS occupancy_pct
        FROM   account a
        LEFT   JOIN portfolio_counts pc ON pc.managed_account_id = a.id
        LEFT   JOIN asset_counts     ac ON ac.managed_account_id = a.id
        LEFT   JOIN lease_counts     lc ON lc.managed_account_id = a.id
        WHERE  a.tenant_id = '{tid}'
        ORDER  BY a.created_at ASC
        "#,
        tid = tenant_id,
    );

    let rows: Vec<sea_orm::QueryResult> = db
        .query_all(Statement::from_string(
            sea_orm::DatabaseBackend::Postgres,
            sql,
        ))
        .await?;

    let mut metrics = Vec::with_capacity(rows.len());
    for row in rows {
        let account_id: Uuid = row.try_get("", "account_id")?;
        let property_count: i64 = row.try_get("", "property_count")?;
        let unit_count: i64 = row.try_get("", "unit_count")?;
        let active_lease_count: i64 = row.try_get("", "active_lease_count")?;
        let occupancy_pct: f64 = row.try_get("", "occupancy_pct")?;

        metrics.push(ClientMetrics {
            account_id,
            property_count,
            unit_count,
            active_lease_count,
            occupancy_pct,
        });
    }

    Ok(metrics)
}
