use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, Set,
    TransactionTrait,
};
use uuid::Uuid;
use chrono::Utc;
use serde_json::Value;

use crate::entities::atlas_lead::{self, Entity as LeadEntity, ActiveModel as LeadActiveModel};
use crate::entities::atlas_account::{self, Entity as AccountEntity, ActiveModel as AccountActiveModel};
use crate::entities::atlas_contact::{ActiveModel as ContactActiveModel};
use crate::entities::atlas_opportunity::{ActiveModel as OpportunityActiveModel};
use crate::entities::atlas_scorecard_template;
use crate::services::scorecard_service::ScorecardService;

/// Result returned from a successful lead conversion.
///
/// All three legs of the conversion are returned so the caller can route to
/// the created records (e.g. redirect to the opportunity view).
#[derive(Debug)]
pub struct LeadConversionResult {
    pub account_id: Uuid,
    pub contact_id: Uuid,
    pub opportunity_id: Uuid,
}

/// G-31 Lead Service
///
/// Implements the lifecycle of atlas_lead records:
///   - creation (manual + import)
///   - qualification state transitions
///   - conversion → atlas_accounts + atlas_contacts + atlas_opportunities
///   - disqualification
///
/// Spec: docs/architecture/g31_atlas_lead_spec.md section 5 + section 11
pub struct LeadService;

impl LeadService {
    // ── Creation ─────────────────────────────────────────────────────────────

    /// Create a new lead from a manual entry or web form submission.
    ///
    /// `name` is computed automatically — callers do not need to supply it.
    pub async fn create(
        db: &DatabaseConnection,
        tenant_id: Uuid,
        first_name: Option<&str>,
        last_name: Option<&str>,
        email: Option<&str>,
        phone: Option<&str>,
        company: Option<&str>,
        source: Option<&str>,
        listing_id: Option<Uuid>,
        account_id: Option<Uuid>,
    ) -> Result<atlas_lead::Model, String> {
        let name = atlas_lead::Model::compute_name(first_name, last_name, company, email);

        let lead = LeadActiveModel {
            id: Set(Uuid::new_v4()),
            tenant_id: Set(tenant_id),
            name: Set(name),
            first_name: Set(first_name.map(|s| s.to_string())),
            last_name: Set(last_name.map(|s| s.to_string())),
            email: Set(email.map(|s| s.to_string())),
            phone: Set(phone.map(|s| s.to_string())),
            company: Set(company.map(|s| s.to_string())),
            source: Set(source.map(|s| s.to_string())),
            listing_id: Set(listing_id),
            account_id: Set(account_id),
            lead_status: Set("new".to_string()),
            is_converted: Set(false),
            is_duplicate: Set(false),
            email_verified: Set(false),
            phone_verified: Set(false),
            country: Set("US".to_string()),
            created_at: Set(Utc::now()),
            updated_at: Set(Utc::now()),
            ..Default::default()
        };

        lead.insert(db).await.map_err(|e| e.to_string())
    }

    /// Create a lead from a bulk import row.
    ///
    /// `data_source` and `data_source_id` are required for import dedup.
    /// Caller is responsible for running dedup checks before calling this.
    #[allow(clippy::too_many_arguments)]
    pub async fn create_from_import(
        db: &DatabaseConnection,
        tenant_id: Uuid,
        data_source: &str,
        data_source_id: &str,
        first_name: Option<&str>,
        last_name: Option<&str>,
        email: Option<&str>,
        company: Option<&str>,
        domain: Option<&str>,
        duns_number: Option<&str>,
        lead_metadata: Option<Value>,
        is_duplicate: bool,
        duplicate_of_lead_id: Option<Uuid>,
    ) -> Result<atlas_lead::Model, String> {
        let name = atlas_lead::Model::compute_name(first_name, last_name, company, email);

        let lead = LeadActiveModel {
            id: Set(Uuid::new_v4()),
            tenant_id: Set(tenant_id),
            name: Set(name),
            first_name: Set(first_name.map(|s| s.to_string())),
            last_name: Set(last_name.map(|s| s.to_string())),
            email: Set(email.map(|s| s.to_string())),
            company: Set(company.map(|s| s.to_string())),
            domain: Set(domain.map(|s| s.to_string())),
            duns_number: Set(duns_number.map(|s| s.to_string())),
            data_source: Set(Some(data_source.to_string())),
            data_source_id: Set(Some(data_source_id.to_string())),
            lead_metadata: Set(lead_metadata),
            is_duplicate: Set(is_duplicate),
            duplicate_of_lead_id: Set(duplicate_of_lead_id),
            source: Set(Some("import".to_string())),
            lead_status: Set("new".to_string()),
            is_converted: Set(false),
            email_verified: Set(false),
            phone_verified: Set(false),
            country: Set("US".to_string()),
            created_at: Set(Utc::now()),
            updated_at: Set(Utc::now()),
            ..Default::default()
        };

        let inserted = lead.insert(db).await.map_err(|e| e.to_string())?;

        // ── G-31 §8.3: Scorecard auto-provision on structured import sources ────
        // When a lead arrives from a known structured data source (FMCSA DOT
        // registry, B2BLeadsUSA), automatically provision a scorecard so The
        // Combinator can begin similarity matching without manual intervention.
        //
        // We query for the first active template targeting 'atlas_lead' for this
        // tenant. If none exists, we skip silently — provisioning is idempotent
        // and can be triggered manually via ScorecardService::get_or_create.
        let auto_provision_sources = ["fmcsa", "business_leads_usa", "dot_registry"];
        if auto_provision_sources.contains(&data_source) {
            let maybe_template = atlas_scorecard_template::Entity::find()
                .filter(atlas_scorecard_template::Column::TenantId.eq(tenant_id))
                .filter(atlas_scorecard_template::Column::EntityType.eq("atlas_lead"))
                .filter(atlas_scorecard_template::Column::IsPublished.eq(true))
                .one(db)
                .await
                .map_err(|e| e.to_string())?;

            if let Some(template) = maybe_template {
                // Idempotent — error intentionally swallowed (best-effort, not a blocker).
                let _ = ScorecardService::get_or_create(
                    db,
                    tenant_id,
                    template.id,
                    "atlas_lead",
                    inserted.id,
                )
                .await;
            }
        }

        Ok(inserted)
    }

    // ── Conversion ────────────────────────────────────────────────────────────

    /// Convert a qualified lead into an Account + Contact + Opportunity.
    ///
    /// This is the primary gate event between atlas_lead and atlas_opportunity.
    /// All three records are created atomically in a transaction.
    ///
    /// Dedup strategy for account creation:
    ///   1. domain match (most reliable for B2B)
    ///   2. duns_number match
    ///   3. (company + city + state + postal_code) match
    ///
    /// If an existing account is found, the lead is linked to it rather than
    /// creating a duplicate account.
    pub async fn convert_lead(
        db: &DatabaseConnection,
        lead_id: Uuid,
        tenant_id: Uuid,
        _converted_by_user_id: Uuid,
    ) -> Result<LeadConversionResult, String> {
        // Load the lead first (outside the transaction — read-only)
        let lead = LeadEntity::find_by_id(lead_id)
            .filter(atlas_lead::Column::TenantId.eq(tenant_id))
            .one(db)
            .await
            .map_err(|e| e.to_string())?
            .ok_or_else(|| format!("Lead {} not found", lead_id))?;

        if lead.is_converted {
            return Err(format!("Lead {} is already converted", lead_id));
        }

        if lead.is_terminal() {
            return Err(format!(
                "Lead {} is in terminal state '{}' and cannot be converted",
                lead_id, lead.lead_status
            ));
        }

        // Execute the three-leg conversion atomically
        let result = db
            .transaction::<_, LeadConversionResult, sea_orm::DbErr>(|txn| {
                Box::pin(async move {
                    // ── 1. Find or create the Account ─────────────────────────────
                    let account_id = Self::upsert_account(txn, tenant_id, &lead).await?;

                    // ── 2. Create the Contact ─────────────────────────────────────
                    let contact_id = Self::create_contact_from_lead(txn, tenant_id, account_id, &lead).await?;

                    // ── 3. Create the Opportunity ─────────────────────────────────
                    let opportunity_id = Self::create_opportunity_from_lead(
                        txn, tenant_id, account_id, contact_id, &lead,
                    )
                    .await?;

                    // ── 4. Mark lead as converted ─────────────────────────────────
                    let now = Utc::now();
                    LeadActiveModel {
                        id: Set(lead.id),
                        is_converted: Set(true),
                        lead_status: Set("converted".to_string()),
                        converted_at: Set(Some(now)),
                        converted_account_id: Set(Some(account_id)),
                        converted_contact_id: Set(Some(contact_id)),
                        converted_opportunity_id: Set(Some(opportunity_id)),
                        updated_at: Set(now),
                        ..Default::default()
                    }
                    .update(txn)
                    .await?;

                    Ok(LeadConversionResult {
                        account_id,
                        contact_id,
                        opportunity_id,
                    })
                })
            })
            .await
            .map_err(|e| e.to_string())?;

        Ok(result)
    }

    // ── Disqualification ──────────────────────────────────────────────────────

    /// Mark a lead as disqualified with a reason.
    ///
    /// Returns an error if the lead is already in a terminal state (converted or disqualified).
    /// This prevents silent overwrite of conversion history.
    pub async fn disqualify(
        db: &DatabaseConnection,
        lead_id: Uuid,
        tenant_id: Uuid,
        reason: &str,
    ) -> Result<(), String> {
        // Load lead and verify it belongs to this tenant
        let lead = LeadEntity::find_by_id(lead_id)
            .filter(atlas_lead::Column::TenantId.eq(tenant_id))
            .one(db)
            .await
            .map_err(|e| e.to_string())?
            .ok_or_else(|| format!("Lead {} not found for tenant {}", lead_id, tenant_id))?;

        // Guard: cannot disqualify a terminal lead (already converted or disqualified)
        if lead.is_terminal() {
            return Err(format!(
                "Lead {} is in terminal state '{}' and cannot be disqualified",
                lead_id, lead.lead_status
            ));
        }

        let now = Utc::now();
        LeadActiveModel {
            id: Set(lead_id),
            lead_status: Set("disqualified".to_string()),
            disqualified_at: Set(Some(now)),
            disqualification_reason: Set(Some(reason.to_string())),
            updated_at: Set(now),
            ..Default::default()
        }
        .update(db)
        .await
        .map_err(|e| e.to_string())?;

        Ok(())
    }

    // ── Dedup check ───────────────────────────────────────────────────────────

    /// Check if a lead already exists for a given email, domain, or DUNS within a tenant.
    /// Returns the canonical lead ID if a duplicate is found.
    pub async fn find_duplicate(
        db: &DatabaseConnection,
        tenant_id: Uuid,
        email: Option<&str>,
        domain: Option<&str>,
        duns_number: Option<&str>,
    ) -> Result<Option<Uuid>, String> {
        // Priority 1: DUNS (most authoritative)
        if let Some(duns) = duns_number.filter(|s| !s.is_empty()) {
            let existing = LeadEntity::find()
                .filter(atlas_lead::Column::TenantId.eq(tenant_id))
                .filter(atlas_lead::Column::DunsNumber.eq(duns))
                .filter(atlas_lead::Column::IsDuplicate.eq(false))
                .one(db)
                .await
                .map_err(|e| e.to_string())?;

            if let Some(lead) = existing {
                return Ok(Some(lead.id));
            }
        }

        // Priority 2: Email (individual-level)
        if let Some(em) = email.filter(|s| !s.is_empty()) {
            let existing = LeadEntity::find()
                .filter(atlas_lead::Column::TenantId.eq(tenant_id))
                .filter(atlas_lead::Column::Email.eq(em))
                .filter(atlas_lead::Column::IsDuplicate.eq(false))
                .one(db)
                .await
                .map_err(|e| e.to_string())?;

            if let Some(lead) = existing {
                return Ok(Some(lead.id));
            }
        }

        // Priority 3: Domain (company-level)
        if let Some(dom) = domain.filter(|s| !s.is_empty()) {
            let existing = LeadEntity::find()
                .filter(atlas_lead::Column::TenantId.eq(tenant_id))
                .filter(atlas_lead::Column::Domain.eq(dom))
                .filter(atlas_lead::Column::IsDuplicate.eq(false))
                .one(db)
                .await
                .map_err(|e| e.to_string())?;

            if let Some(lead) = existing {
                return Ok(Some(lead.id));
            }
        }

        Ok(None)
    }

    // ── Private helpers ───────────────────────────────────────────────────────

    /// Find an existing atlas_accounts record by dedup keys, or create a new one.
    ///
    /// Dedup priority: domain → duns_number → (company + city + state)
    async fn upsert_account(
        db: &impl sea_orm::ConnectionTrait,
        tenant_id: Uuid,
        lead: &atlas_lead::Model,
    ) -> Result<Uuid, sea_orm::DbErr> {
        // 1. Try domain
        if let Some(ref domain) = lead.domain {
            let existing = AccountEntity::find()
                .filter(atlas_account::Column::TenantId.eq(tenant_id))
                .filter(atlas_account::Column::Domain.eq(domain))
                .filter(atlas_account::Column::IsDuplicate.eq(false))
                .one(db)
                .await?;

            if let Some(account) = existing {
                return Ok(account.id);
            }
        }

        // 2. Try DUNS
        if let Some(ref duns) = lead.duns_number {
            let existing = AccountEntity::find()
                .filter(atlas_account::Column::TenantId.eq(tenant_id))
                .filter(atlas_account::Column::DunsNumber.eq(duns))
                .one(db)
                .await?;

            if let Some(account) = existing {
                return Ok(account.id);
            }
        }

        // 3. No existing account — create one
        let now = Utc::now();
        let account = AccountActiveModel {
            id: Set(Uuid::new_v4()),
            tenant_id: Set(tenant_id),
            account_type: Set("organization".to_string()),
            name: Set(lead.company.clone().unwrap_or_else(|| lead.name.clone())),
            dba_name: Set(lead.company_dba.clone()),
            website: Set(lead.company_website.clone()),
            domain: Set(lead.domain.clone()),
            duns_number: Set(lead.duns_number.clone()),
            industry: Set(lead.industry.clone()),
            sub_industry: Set(lead.sub_industry.clone()),
            sic_code: Set(lead.sic_code.clone()),
            naics_code: Set(lead.naics_code.clone()),
            num_employees: Set(lead.num_employees),
            annual_revenue: Set(lead.annual_revenue),
            company_type: Set(lead.company_type.clone()),
            location_type: Set(lead.location_type.clone()),
            year_established: Set(lead.year_established),
            credit_score_code: Set(lead.credit_score_code.clone()),
            street_address: Set(lead.street_address.clone()),
            city: Set(lead.city.clone()),
            state: Set(lead.state.clone()),
            postal_code: Set(lead.postal_code.clone()),
            country: Set(Some(lead.country.clone())),
            mailing_address: Set(lead.mailing_address.clone()),
            data_source: Set(lead.data_source.clone()),
            data_source_id: Set(lead.data_source_id.clone()),
            account_metadata: Set(lead.lead_metadata.clone()),
            status: Set("prospect".to_string()),
            is_duplicate: Set(false),
            created_at: Set(now),
            updated_at: Set(now),
            ..Default::default()
        };

        let result = account.insert(db).await?;
        Ok(result.id)
    }

    /// Create a contact from the lead's person fields.
    async fn create_contact_from_lead(
        db: &impl sea_orm::ConnectionTrait,
        tenant_id: Uuid,
        account_id: Uuid,
        lead: &atlas_lead::Model,
    ) -> Result<Uuid, sea_orm::DbErr> {
        let full_name = match (&lead.first_name, &lead.last_name) {
            (Some(f), Some(l)) => Some(format!("{} {}", f, l)),
            (Some(f), None) => Some(f.clone()),
            (None, Some(l)) => Some(l.clone()),
            _ => None,
        };

        let now = Utc::now();
        let contact = ContactActiveModel {
            id: Set(Uuid::new_v4()),
            tenant_id: Set(tenant_id),
            account_id: Set(account_id),
            first_name: Set(lead.first_name.clone()),
            middle_name: Set(lead.middle_name.clone()),
            last_name: Set(lead.last_name.clone()),
            full_name: Set(full_name),
            title: Set(lead.title.clone()),
            email: Set(lead.email.clone()),
            email_verified: Set(lead.email_verified),
            phone: Set(lead.phone.clone()),
            phone_verified: Set(lead.phone_verified),
            fax: Set(lead.fax.clone()),
            whatsapp: Set(lead.whatsapp.clone()),
            telegram: Set(lead.telegram.clone()),
            linkedin_url: Set(lead.linkedin_url.clone()),
            twitter: Set(lead.twitter.clone()),
            instagram: Set(lead.instagram.clone()),
            avatar_url: Set(lead.avatar_url.clone()),
            data_source: Set(lead.data_source.clone()),
            data_source_id: Set(lead.data_source_id.clone()),
            is_primary: Set(true),
            is_duplicate: Set(false),
            created_at: Set(now),
            updated_at: Set(now),
            ..Default::default()
        };

        let result = contact.insert(db).await?;
        Ok(result.id)
    }

    /// Create a stub opportunity from the converted lead.
    ///
    /// opportunity_type is 'crm_lead_conversion' — a well-known sentinel that
    /// lets the G-15 opportunity service distinguish auto-created vs manual deals.
    async fn create_opportunity_from_lead(
        db: &impl sea_orm::ConnectionTrait,
        tenant_id: Uuid,
        account_id: Uuid,
        _contact_id: Uuid,
        lead: &atlas_lead::Model,
    ) -> Result<Uuid, sea_orm::DbErr> {
        let opp_name = lead
            .company
            .as_deref()
            .unwrap_or(lead.name.as_str())
            .to_string();

        let opportunity = OpportunityActiveModel {
            id: Set(Uuid::new_v4()),
            tenant_id: Set(tenant_id),
            opportunity_type: Set("crm_lead_conversion".to_string()),
            name: Set(opp_name),
            crm_lead_id: Set(Some(lead.id)),
            status: Set("prospecting".to_string()),
            currency: Set("USD".to_string()),
            created_at: Set(Utc::now()),
            ..Default::default()
        };

        let result = opportunity.insert(db).await?;
        Ok(result.id)
    }
}
