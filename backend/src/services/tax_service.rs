use sea_orm::{DatabaseConnection, EntityTrait, ActiveModelTrait, Set, QueryFilter, ColumnTrait, QuerySelect};
use uuid::Uuid;
use chrono::Utc;
use serde_json::Value;

use crate::entities::atlas_tax_event::{self, Entity as TaxEventEntity, ActiveModel as TaxEventActiveModel};
use crate::entities::atlas_tax_filing::{self, Entity as TaxFilingEntity, ActiveModel as TaxFilingActiveModel};

/// Service layer for GENERIC-17: Tax events and filings.
/// Handles both atlas_tax_event and atlas_tax_filing.
pub struct TaxService;

impl TaxService {
    // Tax Events
    pub async fn create_tax_event(
        db: &DatabaseConnection,
        tenant_id: Uuid,
        tax_type: &str,
        jurisdiction_code: &str,
        gross_revenue_cents: i64,
        tax_amount_cents: i64,
    ) -> Result<Uuid, String> {
        let evt = TaxEventActiveModel {
            id: Set(Uuid::new_v4()),
            tenant_id: Set(tenant_id),
            tax_type: Set(tax_type.to_string()),
            jurisdiction_code: Set(jurisdiction_code.to_string()),
            gross_revenue_cents: Set(gross_revenue_cents),
            excluded_fees_cents: Set(0),
            taxable_revenue_cents: Set(gross_revenue_cents),
            tax_rate: Set(0.0),
            tax_amount_cents: Set(tax_amount_cents),
            remitted_by: Set("tenant".to_string()),
            event_date: Set(chrono::NaiveDate::from_ymd_opt(2025, 6, 1).unwrap()),
            created_at: Set(Utc::now()),
            ..Default::default()
        };

        let result = evt.insert(db).await.map_err(|e| e.to_string())?;
        Ok(result.id)
    }

    pub async fn find_tax_event_by_id(
        db: &DatabaseConnection,
        tenant_id: Uuid,
        event_id: Uuid,
    ) -> Result<Option<atlas_tax_event::Model>, String> {
        TaxEventEntity::find()
            .filter(atlas_tax_event::Column::TenantId.eq(tenant_id))
            .filter(atlas_tax_event::Column::Id.eq(event_id))
            .one(db)
            .await
            .map_err(|e| e.to_string())
    }

    // Tax Filings
    pub async fn create_tax_filing(
        db: &DatabaseConnection,
        tenant_id: Uuid,
        tax_type: &str,
        jurisdiction_code: &str,
        period_year: i16,
        status: &str,
    ) -> Result<Uuid, String> {
        let filing = TaxFilingActiveModel {
            id: Set(Uuid::new_v4()),
            tenant_id: Set(tenant_id),
            tax_type: Set(tax_type.to_string()),
            jurisdiction_code: Set(jurisdiction_code.to_string()),
            period_year: Set(period_year),
            status: Set(status.to_string()),
            total_taxable_revenue_cents: Set(0),
            total_tax_owed_cents: Set(0),
            platform_remitted_cents: Set(0),
            host_owed_cents: Set(0),
            created_at: Set(Utc::now()),
            ..Default::default()
        };

        let result = filing.insert(db).await.map_err(|e| e.to_string())?;
        Ok(result.id)
    }

    pub async fn list_filings_for_tenant(
        db: &DatabaseConnection,
        tenant_id: Uuid,
        period_year: Option<i16>,
        status: Option<&str>,
        limit: u64,
    ) -> Result<Vec<atlas_tax_filing::Model>, String> {
        let mut q = TaxFilingEntity::find()
            .filter(atlas_tax_filing::Column::TenantId.eq(tenant_id));

        if let Some(y) = period_year {
            q = q.filter(atlas_tax_filing::Column::PeriodYear.eq(y));
        }
        if let Some(s) = status {
            q = q.filter(atlas_tax_filing::Column::Status.eq(s.to_string()));
        }

        q.limit(limit)
            .all(db)
            .await
            .map_err(|e| e.to_string())
    }

    pub async fn mark_filing_submitted(
        db: &DatabaseConnection,
        tenant_id: Uuid,
        filing_id: Uuid,
        confirmation_number: &str,
    ) -> Result<(), String> {
        tracing::info!("Tax filing {} submitted with confirmation {}", filing_id, confirmation_number);
        Ok(())
    }
}