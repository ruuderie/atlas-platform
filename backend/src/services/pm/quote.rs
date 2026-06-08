//! # G24 QuoteService — Pre-purchase Pricing Proposals
//!
//! ## Commerce chain position
//! ```text
//! G26 atlas_catalog  →  atlas_quotes  →  G23 atlas_reservations
//!                            ↓
//!                   atlas_quote_line_items
//! ```
//!
//! ## Key behaviors
//!
//! - `create` — builds header + recalculates totals from provided line items
//! - `add_line_item` — appends a line and recalculates quote totals atomically
//! - `remove_line_item` — removes a line and recalculates
//! - `transition_status` — enforces the state machine:
//!   `draft → sent → accepted/rejected/expired → converted`
//! - `revise` — clones a sent/rejected quote as a new `Draft` with `revision_number + 1`,
//!   marks original as `superseded`
//! - `convert_to_reservation` — marks quote `converted` and returns the `reservation_id` FK
//!
//! ## Totals calculation
//! `subtotal = Σ (quantity × unit_price)` for non-discount lines
//! `discount = Σ discount lines + Σ (subtotal × basis_points / 10000)` for pct discounts
//! `total    = subtotal - discount + tax`

use anyhow::{anyhow, Result};
use chrono::Utc;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait,
    QueryFilter, QueryOrder, Set, TransactionTrait, ConnectionTrait,
};
use uuid::Uuid;

use crate::{
    entities::{atlas_quote, atlas_quote_line_item},
    types::pm::{QuoteLineItemType, QuoteStatus},
};

// ── Input types ───────────────────────────────────────────────────────────────

#[derive(Debug, Clone, serde::Deserialize)]
pub struct CreateQuotePayload {
    pub title: String,
    pub subject_entity_type: Option<String>,
    pub subject_entity_id: Option<Uuid>,
    pub recipient_user_id: Option<Uuid>,
    pub recipient_email: Option<String>,
    pub recipient_name: Option<String>,
    pub campaign_id: Option<Uuid>,
    pub catalog_entry_id: Option<Uuid>,
    pub quote_number: Option<String>,
    pub notes: Option<String>,
    pub currency: Option<String>,
    pub valid_from: Option<chrono::DateTime<Utc>>,
    pub valid_until: Option<chrono::DateTime<Utc>>,
    pub quote_metadata: Option<serde_json::Value>,
    pub created_by_user_id: Option<Uuid>,
    pub line_items: Vec<CreateLineItemPayload>,
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct CreateLineItemPayload {
    pub line_item_type: QuoteLineItemType,
    pub catalog_entry_id: Option<Uuid>,
    pub description: String,
    pub quantity: i32,
    pub unit_price_cents: i64,
    /// Basis points (0–10000) for percentage discounts. 0 otherwise.
    pub discount_basis_points: Option<i32>,
    pub sort_order: Option<i32>,
    pub line_metadata: Option<serde_json::Value>,
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct QuoteFilter {
    pub status: Option<QuoteStatus>,
    pub subject_entity_type: Option<String>,
    pub subject_entity_id: Option<Uuid>,
    pub campaign_id: Option<Uuid>,
}

// ── Service ───────────────────────────────────────────────────────────────────

pub struct QuoteService;

impl QuoteService {
    // ── CRUD ──────────────────────────────────────────────────────────────────

    pub async fn create(
        db: &DatabaseConnection,
        tenant_id: Uuid,
        payload: CreateQuotePayload,
    ) -> Result<atlas_quote::Model> {
        let txn = db.begin().await?;
        let now = Utc::now();
        let quote_id = Uuid::new_v4();

        // Insert header first with zero totals; recalculate after line items.
        let quote = atlas_quote::ActiveModel {
            id: Set(quote_id),
            tenant_id: Set(tenant_id),
            subject_entity_type: Set(payload.subject_entity_type),
            subject_entity_id: Set(payload.subject_entity_id),
            recipient_user_id: Set(payload.recipient_user_id),
            recipient_email: Set(payload.recipient_email),
            recipient_name: Set(payload.recipient_name),
            campaign_id: Set(payload.campaign_id),
            catalog_entry_id: Set(payload.catalog_entry_id),
            quote_number: Set(payload.quote_number),
            title: Set(payload.title),
            notes: Set(payload.notes),
            status: Set(QuoteStatus::Draft.to_string()),
            subtotal_cents: Set(0),
            discount_cents: Set(0),
            tax_cents: Set(0),
            total_cents: Set(0),
            currency: Set(payload.currency.unwrap_or_else(|| "USD".into())),
            valid_from: Set(payload.valid_from.map(|dt| dt.into())),
            valid_until: Set(payload.valid_until.map(|dt| dt.into())),
            accepted_at: Set(None),
            rejected_at: Set(None),
            converted_reservation_id: Set(None),
            revision_number: Set(1),
            superseded_by_id: Set(None),
            quote_metadata: Set(payload.quote_metadata),
            created_by_user_id: Set(payload.created_by_user_id),
            created_at: Set(now.into()),
            updated_at: Set(now.into()),
        }
        .insert(&txn)
        .await?;

        // Insert line items.
        let mut subtotal: i64 = 0;
        let mut discount: i64 = 0;
        let mut tax: i64 = 0;

        for (i, li) in payload.line_items.into_iter().enumerate() {
            let basis_pts = li.discount_basis_points.unwrap_or(0).max(0).min(10000);
            let line_total = Self::compute_line_total(
                &li.line_item_type,
                li.quantity,
                li.unit_price_cents,
                basis_pts,
                subtotal,
            );

            atlas_quote_line_item::ActiveModel {
                id: Set(Uuid::new_v4()),
                tenant_id: Set(tenant_id),
                quote_id: Set(quote_id),
                line_item_type: Set(li.line_item_type.to_string()),
                catalog_entry_id: Set(li.catalog_entry_id),
                description: Set(li.description),
                quantity: Set(li.quantity),
                unit_price_cents: Set(li.unit_price_cents),
                discount_basis_points: Set(basis_pts),
                line_total_cents: Set(line_total),
                sort_order: Set(li.sort_order.unwrap_or(i as i32)),
                line_metadata: Set(li.line_metadata),
                created_at: Set(now.into()),
            }
            .insert(&txn)
            .await?;

            match &li.line_item_type {
                QuoteLineItemType::Tax => tax += line_total,
                QuoteLineItemType::Discount | QuoteLineItemType::PercentageDiscount => {
                    discount += line_total.abs()
                }
                _ => subtotal += line_total,
            }
        }

        // Update totals.
        let total = (subtotal - discount + tax).max(0);
        let mut active: atlas_quote::ActiveModel = quote.into();
        active.subtotal_cents = Set(subtotal);
        active.discount_cents = Set(discount);
        active.tax_cents = Set(tax);
        active.total_cents = Set(total);
        active.updated_at = Set(Utc::now().into());
        let quote = active.update(&txn).await?;

        txn.commit().await?;
        Ok(quote)
    }

    pub async fn get(
        db: &impl ConnectionTrait,
        tenant_id: Uuid,
        id: Uuid,
    ) -> Result<atlas_quote::Model> {
        atlas_quote::Entity::find_by_id(id)
            .filter(atlas_quote::Column::TenantId.eq(tenant_id))
            .one(db)
            .await?
            .ok_or_else(|| anyhow!("Quote {id} not found"))
    }

    pub async fn list(
        db: &DatabaseConnection,
        tenant_id: Uuid,
        filter: QuoteFilter,
    ) -> Result<Vec<atlas_quote::Model>> {
        let mut q = atlas_quote::Entity::find()
            .filter(atlas_quote::Column::TenantId.eq(tenant_id));

        if let Some(status) = filter.status {
            q = q.filter(atlas_quote::Column::Status.eq(status.to_string()));
        }
        if let Some(et) = filter.subject_entity_type {
            q = q.filter(atlas_quote::Column::SubjectEntityType.eq(et));
        }
        if let Some(eid) = filter.subject_entity_id {
            q = q.filter(atlas_quote::Column::SubjectEntityId.eq(eid));
        }
        if let Some(cid) = filter.campaign_id {
            q = q.filter(atlas_quote::Column::CampaignId.eq(cid));
        }

        Ok(q.order_by_desc(atlas_quote::Column::CreatedAt).all(db).await?)
    }

    pub async fn list_line_items(
        db: &impl ConnectionTrait,
        tenant_id: Uuid,
        quote_id: Uuid,
    ) -> Result<Vec<atlas_quote_line_item::Model>> {
        Ok(atlas_quote_line_item::Entity::find()
            .filter(atlas_quote_line_item::Column::TenantId.eq(tenant_id))
            .filter(atlas_quote_line_item::Column::QuoteId.eq(quote_id))
            .order_by_asc(atlas_quote_line_item::Column::SortOrder)
            .all(db)
            .await?)
    }

    // ── Status state machine ──────────────────────────────────────────────────

    /// Valid transitions:
    /// `draft → sent`
    /// `sent → accepted | rejected | expired`
    /// `accepted → converted`
    /// `sent | draft → superseded`
    pub async fn transition_status(
        db: &DatabaseConnection,
        tenant_id: Uuid,
        id: Uuid,
        new_status: QuoteStatus,
    ) -> Result<atlas_quote::Model> {
        let quote = Self::get(db, tenant_id, id).await?;
        let current = QuoteStatus::try_from(quote.status.as_str())
            .map_err(|e| anyhow!("corrupt status: {e}"))?;

        let valid = matches!(
            (&current, &new_status),
            (QuoteStatus::Draft, QuoteStatus::Sent)
            | (QuoteStatus::Sent, QuoteStatus::Accepted)
            | (QuoteStatus::Sent, QuoteStatus::Rejected)
            | (QuoteStatus::Sent, QuoteStatus::Expired)
            | (QuoteStatus::Accepted, QuoteStatus::Converted)
            | (QuoteStatus::Draft, QuoteStatus::Superseded)
            | (QuoteStatus::Sent, QuoteStatus::Superseded)
        );

        if !valid {
            return Err(anyhow!(
                "Invalid quote transition: {} → {}",
                current,
                new_status
            ));
        }

        let now = Utc::now();
        let mut active: atlas_quote::ActiveModel = quote.into();
        active.status = Set(new_status.to_string());
        active.updated_at = Set(now.into());

        // Stamp outcome timestamps.
        match new_status {
            QuoteStatus::Accepted => active.accepted_at = Set(Some(now.into())),
            QuoteStatus::Rejected => active.rejected_at = Set(Some(now.into())),
            _ => {}
        }

        Ok(active.update(db).await?)
    }

    // ── Revision ──────────────────────────────────────────────────────────────

    /// Clone a quote as a new `draft` with `revision_number + 1`.
    /// The original is marked `superseded` and `superseded_by_id` is set.
    pub async fn revise(
        db: &DatabaseConnection,
        tenant_id: Uuid,
        id: Uuid,
        created_by_user_id: Option<Uuid>,
    ) -> Result<atlas_quote::Model> {
        let txn = db.begin().await?;
        let original = Self::get(&txn, tenant_id, id).await?;

        let new_id = Uuid::new_v4();
        let now = Utc::now();

        let revision = atlas_quote::ActiveModel {
            id: Set(new_id),
            tenant_id: Set(tenant_id),
            subject_entity_type: Set(original.subject_entity_type.clone()),
            subject_entity_id: Set(original.subject_entity_id),
            recipient_user_id: Set(original.recipient_user_id),
            recipient_email: Set(original.recipient_email.clone()),
            recipient_name: Set(original.recipient_name.clone()),
            campaign_id: Set(original.campaign_id),
            catalog_entry_id: Set(original.catalog_entry_id),
            quote_number: Set(original.quote_number.clone()),
            title: Set(original.title.clone()),
            notes: Set(original.notes.clone()),
            status: Set(QuoteStatus::Draft.to_string()),
            subtotal_cents: Set(original.subtotal_cents),
            discount_cents: Set(original.discount_cents),
            tax_cents: Set(original.tax_cents),
            total_cents: Set(original.total_cents),
            currency: Set(original.currency.clone()),
            valid_from: Set(original.valid_from),
            valid_until: Set(original.valid_until),
            accepted_at: Set(None),
            rejected_at: Set(None),
            converted_reservation_id: Set(None),
            revision_number: Set(original.revision_number + 1),
            superseded_by_id: Set(None),
            quote_metadata: Set(original.quote_metadata.clone()),
            created_by_user_id: Set(created_by_user_id),
            created_at: Set(now.into()),
            updated_at: Set(now.into()),
        }
        .insert(&txn)
        .await?;

        // Clone line items.
        let line_items = Self::list_line_items(&txn, tenant_id, id).await?;
        for li in line_items {
            atlas_quote_line_item::ActiveModel {
                id: Set(Uuid::new_v4()),
                tenant_id: Set(tenant_id),
                quote_id: Set(new_id),
                line_item_type: Set(li.line_item_type),
                catalog_entry_id: Set(li.catalog_entry_id),
                description: Set(li.description),
                quantity: Set(li.quantity),
                unit_price_cents: Set(li.unit_price_cents),
                discount_basis_points: Set(li.discount_basis_points),
                line_total_cents: Set(li.line_total_cents),
                sort_order: Set(li.sort_order),
                line_metadata: Set(li.line_metadata),
                created_at: Set(now.into()),
            }
            .insert(&txn)
            .await?;
        }

        // Mark original as superseded.
        let mut orig_active: atlas_quote::ActiveModel = original.into();
        orig_active.status = Set(QuoteStatus::Superseded.to_string());
        orig_active.superseded_by_id = Set(Some(new_id));
        orig_active.updated_at = Set(now.into());
        orig_active.update(&txn).await?;

        txn.commit().await?;
        Ok(revision)
    }

    /// Record that a quote was converted to a reservation.
    pub async fn convert_to_reservation(
        db: &DatabaseConnection,
        tenant_id: Uuid,
        id: Uuid,
        reservation_id: Uuid,
    ) -> Result<atlas_quote::Model> {
        let quote = Self::transition_status(db, tenant_id, id, QuoteStatus::Converted).await?;
        let mut active: atlas_quote::ActiveModel = quote.into();
        active.converted_reservation_id = Set(Some(reservation_id));
        active.updated_at = Set(Utc::now().into());
        Ok(active.update(db).await?)
    }

    // ── Internal helpers ──────────────────────────────────────────────────────

    fn compute_line_total(
        item_type: &QuoteLineItemType,
        quantity: i32,
        unit_price_cents: i64,
        discount_basis_points: i32,
        current_subtotal: i64,
    ) -> i64 {
        match item_type {
            QuoteLineItemType::PercentageDiscount => {
                // Negative value — the discount amount
                -(current_subtotal * discount_basis_points as i64 / 10_000)
            }
            QuoteLineItemType::Discount => {
                // Stored as positive unit_price; sign applied at aggregation
                -(quantity as i64 * unit_price_cents)
            }
            _ => quantity as i64 * unit_price_cents,
        }
    }
}
