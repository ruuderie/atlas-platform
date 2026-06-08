//! # G31 LeadService — PM-Tier Lead Lifecycle Management
//!
//! ## Scope
//!
//! Wraps `atlas_lead` (G31 migration) with a full qualify → convert → disqualify
//! lifecycle. This is the **PM-tier service** — distinct from the legacy
//! `src/handlers/leads.rs` which operates on the old `lead` table.
//!
//! ## State machine
//!
//! ```text
//! New → Contacted → Qualifying → Qualified → Converted  (terminal)
//!                             ↘             ↗
//!                              Disqualified             (terminal)
//! ```
//!
//! Terminal state guard: any attempt to transition a `Converted` or `Disqualified`
//! lead returns an error without touching the database.
//!
//! ## Conversion
//!
//! `convert()` stamps:
//! - `is_converted = true`, `converted_at = now`, `lead_status = "converted"`
//! - `converted_opportunity_id` — FK to G15 atlas_opportunity
//! - `converted_account_id` — FK to atlas_account created for this lead
//! - `converted_contact_id` — FK to atlas_contact created for this lead
//!
//! ## Campaign enrollment
//!
//! `enroll_in_campaign()` creates an `atlas_campaign_enrollment` record (G19)
//! linking this lead into an outbound sequence.

use anyhow::{anyhow, Result};
use chrono::Utc;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait,
    QueryFilter, QueryOrder, Set,
};
use uuid::Uuid;

use crate::{
    entities::{atlas_lead, atlas_campaign_enrollment},
    types::lead::LeadStatus,
};

// ── Input types ───────────────────────────────────────────────────────────────

#[derive(Debug, Clone, serde::Deserialize)]
pub struct CreateLeadPayload {
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    /// Maps to `atlas_lead.company`
    pub company: Option<String>,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub source: Option<String>,
    pub data_source: Option<String>,
    pub data_source_id: Option<String>,
    pub listing_id: Option<Uuid>,
    pub message: Option<String>,
    pub lead_metadata: Option<serde_json::Value>,
    pub country: Option<String>,
    pub street_address: Option<String>,
    pub city: Option<String>,
    pub state: Option<String>,
    pub postal_code: Option<String>,
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct LeadFilter {
    pub status: Option<LeadStatus>,
    pub source: Option<String>,
    pub data_source: Option<String>,
    pub is_converted: Option<bool>,
    pub is_duplicate: Option<bool>,
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct ConvertLeadPayload {
    /// G15 atlas_opportunity created for this lead.
    pub converted_opportunity_id: Option<Uuid>,
    /// atlas_account created for this lead.
    pub converted_account_id: Option<Uuid>,
    /// atlas_contact created for this lead.
    pub converted_contact_id: Option<Uuid>,
}

// ── Service ───────────────────────────────────────────────────────────────────

pub struct LeadService;

impl LeadService {
    // ── CRUD ──────────────────────────────────────────────────────────────────

    pub async fn create(
        db: &DatabaseConnection,
        tenant_id: Uuid,
        payload: CreateLeadPayload,
    ) -> Result<atlas_lead::Model> {
        let now = Utc::now();

        // Compute display name from available identity fields.
        let name = atlas_lead::Model::compute_name(
            payload.first_name.as_deref(),
            payload.last_name.as_deref(),
            payload.company.as_deref(),
            payload.email.as_deref(),
        );

        atlas_lead::ActiveModel {
            id: Set(Uuid::new_v4()),
            tenant_id: Set(tenant_id),
            first_name: Set(payload.first_name),
            middle_name: Set(None),
            last_name: Set(payload.last_name),
            name: Set(name),
            title: Set(None),
            email: Set(payload.email),
            email_verified: Set(false),
            phone: Set(payload.phone),
            phone_verified: Set(false),
            fax: Set(None),
            whatsapp: Set(None),
            telegram: Set(None),
            linkedin_url: Set(None),
            twitter: Set(None),
            instagram: Set(None),
            facebook: Set(None),
            avatar_url: Set(None),
            company: Set(payload.company),
            company_dba: Set(None),
            company_website: Set(None),
            domain: Set(None),
            industry: Set(None),
            sub_industry: Set(None),
            num_employees: Set(None),
            annual_revenue: Set(None),
            company_type: Set(None),
            location_type: Set(None),
            year_established: Set(None),
            sic_code: Set(None),
            naics_code: Set(None),
            duns_number: Set(None),
            credit_score_code: Set(None),
            street_address: Set(payload.street_address),
            city: Set(payload.city),
            state: Set(payload.state),
            postal_code: Set(payload.postal_code),
            country: Set(payload.country.unwrap_or_else(|| "US".into())),
            mailing_address: Set(None),
            message: Set(payload.message),
            lead_status: Set(LeadStatus::New.to_string()),
            source: Set(payload.source),
            data_source: Set(payload.data_source),
            data_source_id: Set(payload.data_source_id),
            lead_metadata: Set(payload.lead_metadata),
            is_duplicate: Set(false),
            duplicate_of_lead_id: Set(None),
            listing_id: Set(payload.listing_id),
            account_id: Set(None),
            is_converted: Set(false),
            converted_at: Set(None),
            converted_account_id: Set(None),
            converted_contact_id: Set(None),
            converted_opportunity_id: Set(None),
            disqualified_at: Set(None),
            disqualification_reason: Set(None),
            created_at: Set(now),
            updated_at: Set(now),
        }
        .insert(db)
        .await
        .map_err(|e| anyhow!("create lead: {e:#}"))
    }

    pub async fn get(
        db: &DatabaseConnection,
        tenant_id: Uuid,
        id: Uuid,
    ) -> Result<atlas_lead::Model> {
        atlas_lead::Entity::find_by_id(id)
            .filter(atlas_lead::Column::TenantId.eq(tenant_id))
            .one(db)
            .await?
            .ok_or_else(|| anyhow!("Lead {id} not found"))
    }

    pub async fn list(
        db: &DatabaseConnection,
        tenant_id: Uuid,
        filter: LeadFilter,
    ) -> Result<Vec<atlas_lead::Model>> {
        let mut q = atlas_lead::Entity::find()
            .filter(atlas_lead::Column::TenantId.eq(tenant_id));

        if let Some(status) = filter.status {
            q = q.filter(atlas_lead::Column::LeadStatus.eq(status.to_string()));
        }
        if let Some(src) = filter.source {
            q = q.filter(atlas_lead::Column::Source.eq(src));
        }
        if let Some(ds) = filter.data_source {
            q = q.filter(atlas_lead::Column::DataSource.eq(ds));
        }
        if let Some(converted) = filter.is_converted {
            q = q.filter(atlas_lead::Column::IsConverted.eq(converted));
        }
        if let Some(dup) = filter.is_duplicate {
            q = q.filter(atlas_lead::Column::IsDuplicate.eq(dup));
        }

        Ok(q.order_by_desc(atlas_lead::Column::CreatedAt).all(db).await?)
    }

    // ── Status machine ────────────────────────────────────────────────────────

    /// Advance the pipeline status.
    ///
    /// Valid forward transitions enforced by terminal guard:
    /// `new → contacted → qualifying → qualified`
    /// Any non-terminal → `disqualified` (via `disqualify()`)
    /// `qualified → converted` (via `convert()` which stamps conversion FKs)
    pub async fn advance_status(
        db: &DatabaseConnection,
        tenant_id: Uuid,
        id: Uuid,
        new_status: LeadStatus,
    ) -> Result<atlas_lead::Model> {
        let lead = Self::get(db, tenant_id, id).await?;

        if lead.is_terminal() {
            return Err(anyhow!(
                "Lead {id} is in terminal state '{}' — no further transitions permitted",
                lead.lead_status
            ));
        }

        // Disqualify and Convert have dedicated methods that stamp extra fields.
        match new_status {
            LeadStatus::Converted => {
                return Err(anyhow!("Use LeadService::convert() to mark a lead as converted"))
            }
            LeadStatus::Disqualified => {
                return Err(anyhow!("Use LeadService::disqualify() to disqualify a lead"))
            }
            _ => {}
        }

        let mut active: atlas_lead::ActiveModel = lead.into();
        active.lead_status = Set(new_status.to_string());
        active.updated_at = Set(Utc::now());
        Ok(active.update(db).await?)
    }

    // ── Qualify ───────────────────────────────────────────────────────────────

    pub async fn qualify(
        db: &DatabaseConnection,
        tenant_id: Uuid,
        id: Uuid,
    ) -> Result<atlas_lead::Model> {
        Self::advance_status(db, tenant_id, id, LeadStatus::Qualified).await
    }

    // ── Disqualify ────────────────────────────────────────────────────────────

    /// Mark a lead as disqualified (terminal). Stamps `disqualified_at` + reason.
    pub async fn disqualify(
        db: &DatabaseConnection,
        tenant_id: Uuid,
        id: Uuid,
        reason: Option<String>,
    ) -> Result<atlas_lead::Model> {
        let lead = Self::get(db, tenant_id, id).await?;

        if lead.is_terminal() {
            return Err(anyhow!(
                "Lead {id} is already in terminal state '{}'",
                lead.lead_status
            ));
        }

        let now = Utc::now();
        let mut active: atlas_lead::ActiveModel = lead.into();
        active.lead_status = Set(LeadStatus::Disqualified.to_string());
        active.disqualified_at = Set(Some(now));
        active.disqualification_reason = Set(reason);
        active.updated_at = Set(now);
        Ok(active.update(db).await?)
    }

    // ── Convert ───────────────────────────────────────────────────────────────

    /// Mark a lead as converted. Stamps all conversion FKs and timestamps.
    ///
    /// Caller creates Account + Contact + Opportunity first, then passes IDs here.
    pub async fn convert(
        db: &DatabaseConnection,
        tenant_id: Uuid,
        id: Uuid,
        payload: ConvertLeadPayload,
    ) -> Result<atlas_lead::Model> {
        let lead = Self::get(db, tenant_id, id).await?;

        if lead.is_terminal() {
            return Err(anyhow!(
                "Lead {id} is already in terminal state '{}'",
                lead.lead_status
            ));
        }

        let now = Utc::now();
        let mut active: atlas_lead::ActiveModel = lead.into();
        active.lead_status = Set(LeadStatus::Converted.to_string());
        active.is_converted = Set(true);
        active.converted_at = Set(Some(now));
        active.converted_opportunity_id = Set(payload.converted_opportunity_id);
        active.converted_account_id = Set(payload.converted_account_id);
        active.converted_contact_id = Set(payload.converted_contact_id);
        active.updated_at = Set(now);
        Ok(active.update(db).await?)
    }

    // ── Duplicate marking ─────────────────────────────────────────────────────

    pub async fn mark_duplicate(
        db: &DatabaseConnection,
        tenant_id: Uuid,
        id: Uuid,
        canonical_id: Uuid,
    ) -> Result<atlas_lead::Model> {
        let lead = Self::get(db, tenant_id, id).await?;
        let mut active: atlas_lead::ActiveModel = lead.into();
        active.is_duplicate = Set(true);
        active.duplicate_of_lead_id = Set(Some(canonical_id));
        active.updated_at = Set(Utc::now());
        Ok(active.update(db).await?)
    }

    // ── Campaign enrollment ───────────────────────────────────────────────────

    /// Enroll this lead in a G19 campaign sequence.
    ///
    /// Blocks enrollment if the lead is in a terminal state — converted/disqualified
    /// leads should not receive outbound campaign sequences.
    pub async fn enroll_in_campaign(
        db: &DatabaseConnection,
        tenant_id: Uuid,
        lead_id: Uuid,
        campaign_id: Uuid,
        enrolled_by_user_id: Option<Uuid>,
    ) -> Result<atlas_campaign_enrollment::Model> {
        let lead = Self::get(db, tenant_id, lead_id).await?;
        if lead.is_terminal() {
            return Err(anyhow!(
                "Cannot enroll terminal lead ({}) in a campaign",
                lead.lead_status
            ));
        }

        let now = Utc::now();
        atlas_campaign_enrollment::ActiveModel {
            id: Set(Uuid::new_v4()),
            tenant_id: Set(tenant_id),
            campaign_id: Set(campaign_id),
            // Link the lead as the contact subject via conversion_entity fields.
            contact_user_id: Set(None),
            contact_email: Set(lead.email.clone()),
            contact_name: Set(Some(lead.name.clone())),
            contact_metadata: Set(Some(serde_json::json!({
                "lead_id": lead_id,
                "entity_type": "atlas_lead"
            }))),
            status: Set("active".into()),
            current_step: Set(0),
            exit_reason: Set(None),
            exit_at: Set(None),
            converted_at: Set(None),
            conversion_entity_type: Set(Some("atlas_lead".into())),
            conversion_entity_id: Set(Some(lead_id)),
            external_enrollment_id: Set(enrolled_by_user_id.map(|id| id.to_string())),
            enrolled_at: Set(now),
            next_step_at: Set(None),
        }
        .insert(db)
        .await
        .map_err(|e| anyhow!("enroll lead in campaign: {e:#}"))
    }
}
