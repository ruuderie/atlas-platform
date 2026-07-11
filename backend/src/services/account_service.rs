#![allow(dead_code)]
use crate::entities::atlas_account::{
    self, ActiveModel as AccountActiveModel, Entity as AccountEntity,
};
use chrono::Utc;
use rust_decimal::Decimal;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, QueryOrder,
    QuerySelect, Set,
    sea_query::{Expr, Func},
};
use serde_json::Value;
use uuid::Uuid;

/// Service layer for the unified Account concept (replaces legacy customer).
///
/// This is the foundation for B2B and B2C party management.
pub struct AccountService;

impl AccountService {
    /// Create a new account (organization or individual).
    pub async fn create_account(
        db: &DatabaseConnection,
        tenant_id: Uuid,
        account_type: &str,
        name: &str,
        first_name: Option<&str>,
        last_name: Option<&str>,
    ) -> Result<Uuid, String> {
        let account = AccountActiveModel {
            id: Set(Uuid::new_v4()),
            tenant_id: Set(tenant_id),
            account_type: Set(account_type.to_string()),
            name: Set(name.to_string()),
            first_name: Set(first_name.map(|s| s.to_string())),
            last_name: Set(last_name.map(|s| s.to_string())),
            status: Set("active".to_string()),
            created_at: Set(Utc::now()),
            updated_at: Set(Utc::now()),
            ..Default::default()
        };

        let result = account.insert(db).await.map_err(|e| e.to_string())?;
        Ok(result.id)
    }

    /// Find an account by ID within a tenant.
    pub async fn find_by_id(
        db: &DatabaseConnection,
        tenant_id: Uuid,
        account_id: Uuid,
    ) -> Result<Option<atlas_account::Model>, String> {
        AccountEntity::find()
            .filter(atlas_account::Column::TenantId.eq(tenant_id))
            .filter(atlas_account::Column::Id.eq(account_id))
            .one(db)
            .await
            .map_err(|e| e.to_string())
    }

    /// List accounts for a tenant.
    pub async fn list_for_tenant(
        db: &DatabaseConnection,
        tenant_id: Uuid,
        limit: u64,
    ) -> Result<Vec<atlas_account::Model>, String> {
        AccountEntity::find()
            .filter(atlas_account::Column::TenantId.eq(tenant_id))
            .limit(limit)
            .all(db)
            .await
            .map_err(|e| e.to_string())
    }

    /// Find or create a default organization account for a tenant (useful during migration).
    pub async fn find_or_create_tenant_account(
        db: &DatabaseConnection,
        tenant_id: Uuid,
        tenant_name: &str,
    ) -> Result<Uuid, String> {
        // In a real implementation we would have a better lookup (e.g. by name + type)
        let existing = Self::list_for_tenant(db, tenant_id, 5).await?;
        if let Some(acc) = existing
            .into_iter()
            .find(|a| a.account_type == "organization")
        {
            return Ok(acc.id);
        }

        Self::create_account(db, tenant_id, "organization", tenant_name, None, None).await
    }

    // ── Firmographic conversion path ───────────────────────────────────────

    /// Full-firmographic account creation from the G-31 lead conversion flow.
    ///
    /// Called exclusively by `LeadService::convert_lead` inside an existing transaction.
    /// All firmographic columns map directly to the `atlas_accounts` entity fields.
    ///
    /// Column semantics mirror `entities/atlas_account.rs`.
    #[allow(clippy::too_many_arguments)]
    pub async fn create_from_lead_conversion(
        db: &DatabaseConnection,
        tenant_id: Uuid,
        // Identity
        account_type: &str, // 'organization' | 'individual'
        name: &str,
        first_name: Option<&str>,
        last_name: Option<&str>,
        // Contact
        company_email: Option<&str>,
        company_phone: Option<&str>,
        website: Option<&str>,
        // Firmographic
        industry: Option<&str>,
        sic_code: Option<&str>,
        naics_code: Option<&str>,
        num_employees: Option<i32>,
        annual_revenue: Option<Decimal>,
        // Address
        street_address: Option<&str>,
        city: Option<&str>,
        state: Option<&str>,
        postal_code: Option<&str>,
        country: Option<&str>,
        // Classification / dedup
        domain: Option<&str>,
        duns_number: Option<&str>,
        // Source tracking
        data_source: Option<&str>,
        data_source_id: Option<&str>,
        account_metadata: Option<Value>,
    ) -> Result<Uuid, String> {
        let now = Utc::now();
        let account = AccountActiveModel {
            id: Set(Uuid::new_v4()),
            tenant_id: Set(tenant_id),
            account_type: Set(account_type.to_string()),
            name: Set(name.to_string()),
            first_name: Set(first_name.map(|s| s.to_string())),
            last_name: Set(last_name.map(|s| s.to_string())),
            status: Set("active".to_string()),
            company_email: Set(company_email.map(|s| s.to_string())),
            company_phone: Set(company_phone.map(|s| s.to_string())),
            website: Set(website.map(|s| s.to_string())),
            industry: Set(industry.map(|s| s.to_string())),
            sic_code: Set(sic_code.map(|s| s.to_string())),
            naics_code: Set(naics_code.map(|s| s.to_string())),
            num_employees: Set(num_employees),
            annual_revenue: Set(annual_revenue),
            street_address: Set(street_address.map(|s| s.to_string())),
            city: Set(city.map(|s| s.to_string())),
            state: Set(state.map(|s| s.to_string())),
            postal_code: Set(postal_code.map(|s| s.to_string())),
            country: Set(country.map(|s| s.to_string())),
            domain: Set(domain.map(|s| s.to_string())),
            duns_number: Set(duns_number.map(|s| s.to_string())),
            data_source: Set(data_source.map(|s| s.to_string())),
            data_source_id: Set(data_source_id.map(|s| s.to_string())),
            account_metadata: Set(account_metadata),
            created_at: Set(now),
            updated_at: Set(now),
            ..Default::default()
        };

        let result = account.insert(db).await.map_err(|e| e.to_string())?;
        Ok(result.id)
    }

    /// Find an existing account by domain within a tenant.
    pub async fn find_by_domain(
        db: &DatabaseConnection,
        tenant_id: Uuid,
        domain: &str,
    ) -> Result<Option<atlas_account::Model>, String> {
        AccountEntity::find()
            .filter(atlas_account::Column::TenantId.eq(tenant_id))
            .filter(atlas_account::Column::Domain.eq(domain))
            .filter(atlas_account::Column::Status.ne("archived"))
            .one(db)
            .await
            .map_err(|e| e.to_string())
    }

    /// Search accounts by name, company email, or domain (case-insensitive).
    ///
    /// The query is sanitized: `%`, `_`, and `\` are escaped before being embedded
    /// in the LIKE expression to prevent wildcard injection (full-table-scan via `_`)
    /// or unintended broad matches.
    pub async fn search(
        db: &DatabaseConnection,
        tenant_id: Uuid,
        query: &str,
        limit: u64,
    ) -> Result<Vec<atlas_account::Model>, String> {
        // Escape LIKE metacharacters so user-supplied characters are treated as literals.
        let escaped = query
            .to_lowercase()
            .replace('\\', "\\\\")
            .replace('%', "\\%")
            .replace('_', "\\_");
        let pattern = format!("%{}%", escaped);
        AccountEntity::find()
            .filter(atlas_account::Column::TenantId.eq(tenant_id))
            .filter(
                sea_orm::Condition::any()
                    .add(
                        Expr::expr(Func::lower(Expr::col(atlas_account::Column::Name)))
                            .like(&pattern),
                    )
                    .add(
                        Expr::expr(Func::lower(Expr::col(atlas_account::Column::CompanyEmail)))
                            .like(&pattern),
                    )
                    .add(
                        Expr::expr(Func::lower(Expr::col(atlas_account::Column::Domain)))
                            .like(&pattern),
                    ),
            )
            .filter(atlas_account::Column::Status.ne("archived"))
            .order_by_asc(atlas_account::Column::Name)
            .limit(limit)
            .all(db)
            .await
            .map_err(|e| e.to_string())
    }
}
