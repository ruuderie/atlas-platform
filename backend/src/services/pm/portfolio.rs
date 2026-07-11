//! Folio — Portfolio Service (PM wrapper over G-09 `atlas_portfolios`)
//!
//! NAV aggregation (USD/BRL/BTC), asset_code generation, portfolio hierarchy.
//!
//! # Asset Code Format
//! `{COUNTRY}-{STATE_OR_REGION}-{SEQ:03}` e.g. `US-FL-001`, `BR-SP-042`, `VI-STT-001`
//! Stored in `atlas_assets.serial_or_folio_number` alongside the county folio number.

use anyhow::Result;
use sea_orm::DatabaseConnection;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// ── Asset code ────────────────────────────────────────────────────────────────

/// Asset code: `{COUNTRY}-{REGION}-{SEQ:03}` — e.g. `US-FL-001`, `BR-SP-042`.
/// Stored in `atlas_assets.serial_or_folio_number`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetCode {
    pub country_code: String,
    pub region_code: String,
    pub sequence: u32,
}

impl AssetCode {
    /// Formats as `BR-SP-001`.
    pub fn display(&self) -> String {
        format!(
            "{}-{}-{:03}",
            self.country_code, self.region_code, self.sequence
        )
    }

    /// Parses `BR-SP-001` back to struct. Returns `None` on invalid format.
    pub fn parse(s: &str) -> Option<Self> {
        let parts: Vec<&str> = s.split('-').collect();
        if parts.len() < 3 {
            return None;
        }
        let sequence = parts.last()?.parse::<u32>().ok()?;
        Some(Self {
            country_code: parts[0].to_uppercase(),
            region_code: parts[1..parts.len() - 1].join("-").to_uppercase(),
            sequence,
        })
    }
}

// ── NAV snapshot ──────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortfolioNav {
    pub portfolio_id: Uuid,
    pub tenant_id: Uuid,
    pub nav_usd_cents: i64,
    pub nav_brl_cents: Option<i64>,
    pub nav_btc_satoshis: Option<i64>,
    pub unit_count: i32,
    pub occupied_count: i32,
    pub vacant_count: i32,
    pub occupancy_rate: f64,
    pub calculated_at: chrono::DateTime<chrono::Utc>,
}

// ── Service ───────────────────────────────────────────────────────────────────

pub struct PortfolioService;

impl PortfolioService {
    /// Get or create the default portfolio for a Folio tenant.
    ///
    /// `owner_user_id` is required by the entity — we use a nil UUID as the
    /// system owner when provisioning without a specific user context.
    pub async fn get_or_create_default(db: &DatabaseConnection, tenant_id: Uuid) -> Result<Uuid> {
        use chrono::Utc;
        use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, Set};

        let existing = crate::entities::atlas_portfolio::Entity::find()
            .filter(crate::entities::atlas_portfolio::Column::TenantId.eq(tenant_id))
            .one(db)
            .await?;

        if let Some(p) = existing {
            return Ok(p.id);
        }

        let id = Uuid::new_v4();
        let now = Utc::now();

        let model = crate::entities::atlas_portfolio::ActiveModel {
            id: Set(id),
            tenant_id: Set(tenant_id),
            owner_user_id: Set(Uuid::nil()), // system-provisioned
            portfolio_type: Set("real_estate".to_string()),
            name: Set("My Portfolio".to_string()),
            description: Set(Some("Default Folio property portfolio".to_string())),
            managed_account_id: sea_orm::ActiveValue::NotSet, // NULL for standard portfolios
            metadata: Set(None),
            created_at: Set(now),
        };
        model.insert(db).await?;

        tracing::info!(portfolio_id = %id, %tenant_id, "PortfolioService: created default PM portfolio");
        Ok(id)
    }

    /// Generate the next asset code for a (tenant, country, region) combination.
    ///
    /// Counts existing assets whose `serial_or_folio_number` starts with the prefix
    /// `{COUNTRY}-{REGION}-` to determine the next sequence number.
    ///
    /// Note: not transactionally safe at very high concurrency — acceptable at PM scale.
    pub async fn next_asset_code(
        db: &DatabaseConnection,
        tenant_id: Uuid,
        country_code: &str,
        region_code: &str,
    ) -> Result<AssetCode> {
        use sea_orm::{ColumnTrait, EntityTrait, PaginatorTrait, QueryFilter};

        let prefix = format!(
            "{}-{}-",
            country_code.to_uppercase(),
            region_code.to_uppercase()
        );

        let count = crate::entities::atlas_asset::Entity::find()
            .filter(crate::entities::atlas_asset::Column::TenantId.eq(tenant_id))
            .filter(
                crate::entities::atlas_asset::Column::SerialOrFolioNumber
                    .like(format!("{prefix}%")),
            )
            .count(db)
            .await?;

        Ok(AssetCode {
            country_code: country_code.to_uppercase(),
            region_code: region_code.to_uppercase(),
            sequence: (count as u32) + 1,
        })
    }

    /// Compute a NAV snapshot for a portfolio. Phase 3: live ledger aggregation.
    pub async fn compute_nav(
        db: &DatabaseConnection,
        tenant_id: Uuid,
        portfolio_id: Uuid,
    ) -> Result<PortfolioNav> {
        use sea_orm::{ColumnTrait, EntityTrait, PaginatorTrait, QueryFilter};

        let unit_count = crate::entities::atlas_asset::Entity::find()
            .filter(crate::entities::atlas_asset::Column::TenantId.eq(tenant_id))
            .filter(crate::entities::atlas_asset::Column::PortfolioId.eq(portfolio_id))
            .count(db)
            .await? as i32;

        Ok(PortfolioNav {
            portfolio_id,
            tenant_id,
            nav_usd_cents: 0, // Phase 3: aggregate from atlas_ledger_entries
            nav_brl_cents: None,
            nav_btc_satoshis: None,
            unit_count,
            occupied_count: 0, // Phase 3: from active atlas_contracts
            vacant_count: unit_count,
            occupancy_rate: 0.0,
            calculated_at: chrono::Utc::now(),
        })
    }

    /// List all portfolios for a tenant.
    pub async fn list(
        db: &DatabaseConnection,
        tenant_id: Uuid,
    ) -> Result<Vec<crate::entities::atlas_portfolio::Model>> {
        use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};

        Ok(crate::entities::atlas_portfolio::Entity::find()
            .filter(crate::entities::atlas_portfolio::Column::TenantId.eq(tenant_id))
            .all(db)
            .await?)
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_asset_code_display() {
        let c = AssetCode {
            country_code: "BR".into(),
            region_code: "SP".into(),
            sequence: 1,
        };
        assert_eq!(c.display(), "BR-SP-001");
    }

    #[test]
    fn test_asset_code_display_padding() {
        let c = AssetCode {
            country_code: "US".into(),
            region_code: "FL".into(),
            sequence: 42,
        };
        assert_eq!(c.display(), "US-FL-042");
    }

    #[test]
    fn test_asset_code_parse_roundtrip() {
        for s in &["BR-SP-001", "US-FL-042", "DO-SD-001", "VI-STT-001"] {
            let parsed = AssetCode::parse(s).expect("should parse");
            assert_eq!(&parsed.display(), s);
        }
    }

    #[test]
    fn test_asset_code_parse_invalid() {
        assert!(AssetCode::parse("INVALID").is_none());
        assert!(AssetCode::parse("BR-SP-XYZ").is_none());
    }
}
