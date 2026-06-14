//! # G21 EventService — Event Management, Ticketing & Check-In
//!
//! ## Scope
//!
//! Manages the three `atlas_event_*` tables covering the full event lifecycle:
//!
//! ```text
//! atlas_events  →  atlas_event_ticket_types  →  atlas_event_registrations
//!      ↑                                              ↓
//!   G19 atlas_campaigns               G20 atlas_attribution_touchpoints
//!                                     G03 atlas_ledger_entries
//! ```
//!
//! ## Enum enforcement
//!
//! | Field | Enum |
//! |-------|------|
//! | `atlas_events.event_type` | `EventType` |
//! | `atlas_events.status` | `EventStatus` |
//! | `atlas_event_registrations.status` | `RegistrationStatus` |
//!
//! ## Status state machine (EventStatus)
//!
//! ```text
//! Draft  →  Published  →  Active  →  RegistrationClosed  →  InProgress  →  Completed
//!                  ↓                          ↓                  ↓
//!               Cancelled               Cancelled            Cancelled
//! ```
//!
//! ## QR check-in flow
//!
//! 1. Registration created → `check_in_token` = random hex (app-generated)
//! 2. Token encoded in QR code shown to attendee
//! 3. Staff scans QR → `check_in(token)` resolves registration, sets `CheckedIn`,
//!    increments `atlas_events.attended_count`
//!
//! ## Capacity management
//!
//! `register()` checks `atlas_event_ticket_types.quantity_available - quantity_sold`
//! before inserting. If at capacity and `waitlist_enabled = true`, status is set to
//! `Waitlisted`. If `waitlist_enabled = false`, registration is rejected.

use anyhow::{anyhow, Result};
use chrono::Utc;
use hex;
use rand::RngCore;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait,
    QueryFilter, QueryOrder, Set, TransactionTrait,
};
use uuid::Uuid;

use crate::{
    entities::{atlas_event, atlas_event_registration, atlas_event_ticket_type},
    types::pm::{EventStatus, EventType, RegistrationStatus},
};

// ── Input payload types ───────────────────────────────────────────────────────

#[derive(Debug, Clone, serde::Deserialize)]
pub struct CreateEventPayload {
    pub name: String,
    pub slug: Option<String>,
    pub event_type: EventType,
    pub is_virtual: bool,
    pub virtual_url: Option<String>,
    pub venue_name: Option<String>,
    pub venue_address: Option<String>,
    pub venue_asset_id: Option<Uuid>,
    pub max_capacity: Option<i32>,
    pub waitlist_enabled: Option<bool>,
    pub starts_at: chrono::DateTime<Utc>,
    pub ends_at: chrono::DateTime<Utc>,
    pub registration_opens_at: Option<chrono::DateTime<Utc>>,
    pub registration_closes_at: Option<chrono::DateTime<Utc>>,
    pub campaign_id: Option<Uuid>,
    pub subject_entity_type: Option<String>,
    pub subject_entity_id: Option<Uuid>,
    pub is_public: Option<bool>,
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct CreateTicketTypePayload {
    pub event_id: Uuid,
    pub name: String,
    pub price_cents: i64,
    pub currency: Option<String>,
    pub quantity_available: Option<i32>,
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct RegistrationPayload {
    pub event_id: Uuid,
    pub ticket_type_id: Uuid,
    pub attendee_email: String,
    pub attendee_name: Option<String>,
    pub attendee_user_id: Option<Uuid>,
    pub quantity: Option<i32>,
    pub attribution_touchpoint_id: Option<Uuid>,
}

#[derive(Debug, Clone, Default, serde::Deserialize)]
pub struct EventFilter {
    pub event_type: Option<EventType>,
    pub status: Option<EventStatus>,
    pub campaign_id: Option<Uuid>,
    pub subject_entity_type: Option<String>,
    pub subject_entity_id: Option<Uuid>,
    /// Filter to events that haven't started yet.
    pub upcoming_only: Option<bool>,
}

// ── Service ───────────────────────────────────────────────────────────────────

pub struct EventService;

impl EventService {
    // ── Event CRUD ────────────────────────────────────────────────────────────

    /// Create a new event in `Draft` status.
    pub async fn create(
        db: &DatabaseConnection,
        tenant_id: Uuid,
        payload: CreateEventPayload,
    ) -> Result<atlas_event::Model> {
        if payload.ends_at <= payload.starts_at {
            return Err(anyhow!("ends_at must be after starts_at"));
        }

        let now = Utc::now();
        let active = atlas_event::ActiveModel {
            id: Set(Uuid::new_v4()),
            tenant_id: Set(tenant_id),
            name: Set(payload.name),
            slug: Set(payload.slug),
            event_type: Set(payload.event_type.to_string()),
            status: Set(EventStatus::Draft.to_string()),
            is_virtual: Set(payload.is_virtual),
            virtual_url: Set(payload.virtual_url),
            venue_name: Set(payload.venue_name),
            venue_address: Set(payload.venue_address),
            venue_asset_id: Set(payload.venue_asset_id),
            max_capacity: Set(payload.max_capacity),
            waitlist_enabled: Set(payload.waitlist_enabled.unwrap_or(true)),
            starts_at: Set(payload.starts_at),
            ends_at: Set(payload.ends_at),
            registration_opens_at: Set(payload.registration_opens_at),
            registration_closes_at: Set(payload.registration_closes_at),
            campaign_id: Set(payload.campaign_id),
            subject_entity_type: Set(payload.subject_entity_type),
            subject_entity_id: Set(payload.subject_entity_id),
            is_public: Set(payload.is_public.unwrap_or(true)),
            registered_count: Set(0),
            attended_count: Set(0),
            revenue_cents: Set(0),
            created_at: Set(now),
            updated_at: Set(now),
        };

        let event = active.insert(db).await?;

        tracing::info!(
            %tenant_id, event_id = %event.id, event_type = %event.event_type,
            "EventService::create: created '{}'", event.name
        );

        Ok(event)
    }

    /// Get an event by ID, verifying tenant ownership.
    pub async fn get(
        db: &DatabaseConnection,
        tenant_id: Uuid,
        event_id: Uuid,
    ) -> Result<atlas_event::Model> {
        atlas_event::Entity::find_by_id(event_id)
            .filter(atlas_event::Column::TenantId.eq(tenant_id))
            .one(db)
            .await?
            .ok_or_else(|| anyhow!("Event {event_id} not found for tenant {tenant_id}"))
    }

    /// List events with optional filters. Ordered by starts_at ascending.
    pub async fn list(
        db: &DatabaseConnection,
        tenant_id: Uuid,
        filter: EventFilter,
    ) -> Result<Vec<atlas_event::Model>> {
        let mut q = atlas_event::Entity::find()
            .filter(atlas_event::Column::TenantId.eq(tenant_id));

        if let Some(et) = filter.event_type {
            q = q.filter(atlas_event::Column::EventType.eq(et.to_string()));
        }
        if let Some(st) = filter.status {
            q = q.filter(atlas_event::Column::Status.eq(st.to_string()));
        }
        if let Some(cid) = filter.campaign_id {
            q = q.filter(atlas_event::Column::CampaignId.eq(cid));
        }
        if let Some(set) = filter.subject_entity_type {
            q = q.filter(atlas_event::Column::SubjectEntityType.eq(set));
        }
        if let Some(sei) = filter.subject_entity_id {
            q = q.filter(atlas_event::Column::SubjectEntityId.eq(sei));
        }
        if filter.upcoming_only == Some(true) {
            q = q.filter(atlas_event::Column::StartsAt.gte(Utc::now()));
        }

        Ok(q.order_by_asc(atlas_event::Column::StartsAt).all(db).await?)
    }

    /// Transition event status with state machine validation.
    ///
    /// Valid transitions:
    /// ```text
    /// Draft        → Published | Cancelled
    /// Published    → Active | Cancelled
    /// Active       → RegistrationClosed | InProgress | Cancelled
    /// RegistClosed → InProgress | Cancelled
    /// InProgress   → Completed | Cancelled
    /// Completed    → (terminal)
    /// Cancelled    → (terminal)
    /// ```
    pub async fn transition_status(
        db: &DatabaseConnection,
        tenant_id: Uuid,
        event_id: Uuid,
        new_status: EventStatus,
    ) -> Result<atlas_event::Model> {
        let event = Self::get(db, tenant_id, event_id).await?;
        let current = EventStatus::try_from(event.status.as_str())
            .map_err(|e| anyhow!("Invalid stored status: {e}"))?;

        let allowed = match &current {
            EventStatus::Draft              => matches!(new_status, EventStatus::Published | EventStatus::Cancelled),
            EventStatus::Published          => matches!(new_status, EventStatus::Active | EventStatus::Cancelled),
            EventStatus::Active             => matches!(new_status, EventStatus::RegistrationClosed | EventStatus::InProgress | EventStatus::Cancelled),
            EventStatus::RegistrationClosed => matches!(new_status, EventStatus::InProgress | EventStatus::Cancelled),
            EventStatus::InProgress         => matches!(new_status, EventStatus::Completed | EventStatus::Cancelled),
            EventStatus::Completed          => false,
            EventStatus::Cancelled          => false,
        };

        if !allowed {
            return Err(anyhow!(
                "Invalid event transition: {current} → {new_status} for event {event_id}"
            ));
        }

        let mut active: atlas_event::ActiveModel = event.into();
        active.status = Set(new_status.to_string());
        active.updated_at = Set(Utc::now());
        Ok(active.update(db).await?)
    }

    /// Find all events tied to a specific platform entity.
    pub async fn find_by_subject(
        db: &DatabaseConnection,
        tenant_id: Uuid,
        entity_type: &str,
        entity_id: Uuid,
    ) -> Result<Vec<atlas_event::Model>> {
        Ok(atlas_event::Entity::find()
            .filter(atlas_event::Column::TenantId.eq(tenant_id))
            .filter(atlas_event::Column::SubjectEntityType.eq(entity_type))
            .filter(atlas_event::Column::SubjectEntityId.eq(entity_id))
            .order_by_asc(atlas_event::Column::StartsAt)
            .all(db)
            .await?)
    }

    // ── Ticket types ──────────────────────────────────────────────────────────

    /// Create a ticket type for an event.
    pub async fn create_ticket_type(
        db: &DatabaseConnection,
        tenant_id: Uuid,
        payload: CreateTicketTypePayload,
    ) -> Result<atlas_event_ticket_type::Model> {
        Self::get(db, tenant_id, payload.event_id).await?;

        Ok(atlas_event_ticket_type::ActiveModel {
            id: Set(Uuid::new_v4()),
            event_id: Set(payload.event_id),
            tenant_id: Set(tenant_id),
            name: Set(payload.name),
            price_cents: Set(payload.price_cents),
            currency: Set(payload.currency.unwrap_or_else(|| "USD".to_string())),
            quantity_available: Set(payload.quantity_available),
            quantity_sold: Set(0),
            is_active: Set(true),
        }
        .insert(db)
        .await?)
    }

    /// List ticket types for an event (active only by default).
    pub async fn list_ticket_types(
        db: &DatabaseConnection,
        tenant_id: Uuid,
        event_id: Uuid,
        active_only: bool,
    ) -> Result<Vec<atlas_event_ticket_type::Model>> {
        Self::get(db, tenant_id, event_id).await?;

        let mut q = atlas_event_ticket_type::Entity::find()
            .filter(atlas_event_ticket_type::Column::EventId.eq(event_id));

        if active_only {
            q = q.filter(atlas_event_ticket_type::Column::IsActive.eq(true));
        }

        Ok(q.all(db).await?)
    }

    // ── Registration ──────────────────────────────────────────────────────────

    /// Register an attendee for an event. Handles capacity checks, waitlisting,
    /// and counter increments in a single transaction.
    pub async fn register(
        db: &DatabaseConnection,
        tenant_id: Uuid,
        payload: RegistrationPayload,
    ) -> Result<atlas_event_registration::Model> {
        let txn = db.begin().await?;

        // Verify event is in a registrable state.
        let event = atlas_event::Entity::find_by_id(payload.event_id)
            .filter(atlas_event::Column::TenantId.eq(tenant_id))
            .one(&txn)
            .await?
            .ok_or_else(|| anyhow!("Event {} not found", payload.event_id))?;

        let event_status = EventStatus::try_from(event.status.as_str())
            .map_err(|e| anyhow!("Invalid stored event status: {e}"))?;

        if !matches!(event_status, EventStatus::Active | EventStatus::Published) {
            return Err(anyhow!(
                "Cannot register for event {}: status is {event_status}",
                payload.event_id
            ));
        }

        // Load and validate ticket type.
        let ticket = atlas_event_ticket_type::Entity::find_by_id(payload.ticket_type_id)
            .filter(atlas_event_ticket_type::Column::EventId.eq(payload.event_id))
            .one(&txn)
            .await?
            .ok_or_else(|| anyhow!("Ticket type {} not found", payload.ticket_type_id))?;

        if !ticket.is_active {
            return Err(anyhow!("Ticket type {} is not active", ticket.id));
        }

        let qty = payload.quantity.unwrap_or(1);

        // Determine registration status based on capacity.
        let reg_status = if let Some(avail) = ticket.quantity_available {
            let remaining = avail - ticket.quantity_sold;
            if remaining <= 0 {
                if event.waitlist_enabled {
                    RegistrationStatus::Waitlisted
                } else {
                    return Err(anyhow!("Event {} is at capacity", payload.event_id));
                }
            } else {
                RegistrationStatus::Confirmed
            }
        } else {
            // Unlimited capacity — always confirmed.
            RegistrationStatus::Confirmed
        };

        // Generate the QR check-in token (32 random bytes → 64 hex chars).
        let check_in_token = {
            let mut bytes = [0u8; 32];
            rand::thread_rng().fill_bytes(&mut bytes);
            hex::encode(bytes)
        };

        let now = Utc::now();
        let confirmed_at = if matches!(reg_status, RegistrationStatus::Confirmed) {
            Some(now)
        } else {
            None
        };

        let reg = atlas_event_registration::ActiveModel {
            id: Set(Uuid::new_v4()),
            event_id: Set(payload.event_id),
            ticket_type_id: Set(payload.ticket_type_id),
            tenant_id: Set(tenant_id),
            attendee_email: Set(payload.attendee_email),
            attendee_name: Set(payload.attendee_name),
            attendee_user_id: Set(payload.attendee_user_id),
            quantity: Set(qty),
            ledger_entry_id: Set(None),
            check_in_token: Set(check_in_token),
            status: Set(reg_status.to_string()),
            confirmed_at: Set(confirmed_at),
            checked_in_at: Set(None),
            attribution_touchpoint_id: Set(payload.attribution_touchpoint_id),
            created_at: Set(now),
        }
        .insert(&txn)
        .await?;

        // Update counters on event and ticket type.
        use sea_orm::ConnectionTrait;
        if matches!(reg_status, RegistrationStatus::Confirmed) {
            txn.execute_unprepared(&format!(
                "UPDATE atlas_events SET registered_count = registered_count + {qty}, updated_at = NOW() WHERE id = '{}'",
                payload.event_id
            ))
            .await
            .map_err(|e| anyhow!("counter update failed: {e:#}"))?;

            txn.execute_unprepared(&format!(
                "UPDATE atlas_event_ticket_types SET quantity_sold = quantity_sold + {qty} WHERE id = '{}'",
                payload.ticket_type_id
            ))
            .await
            .map_err(|e| anyhow!("ticket quantity update failed: {e:#}"))?;
        }

        txn.commit().await?;

        tracing::info!(
            %tenant_id, event_id = %payload.event_id, registration_id = %reg.id,
            status = %reg.status, "EventService::register: attendee registered"
        );

        Ok(reg)
    }

    // ── QR Check-in ───────────────────────────────────────────────────────────

    /// Scan a QR check-in token and mark the attendee as checked in.
    /// Increments `atlas_events.attended_count`.
    ///
    /// Returns an error if:
    ///   - Token is not found
    ///   - Registration is already checked in, cancelled, or no-show
    pub async fn check_in(
        db: &DatabaseConnection,
        check_in_token: &str,
    ) -> Result<atlas_event_registration::Model> {
        let txn = db.begin().await?;

        let reg = atlas_event_registration::Entity::find()
            .filter(atlas_event_registration::Column::CheckInToken.eq(check_in_token))
            .one(&txn)
            .await?
            .ok_or_else(|| anyhow!("Invalid check-in token"))?;

        let current_status = RegistrationStatus::try_from(reg.status.as_str())
            .map_err(|e| anyhow!("Invalid stored registration status: {e}"))?;

        // Only Confirmed and Waitlisted can check in.
        match &current_status {
            RegistrationStatus::Confirmed  => {}
            RegistrationStatus::Waitlisted => {} // walk-up admission
            RegistrationStatus::CheckedIn  => return Err(anyhow!("Already checked in")),
            RegistrationStatus::Cancelled  => return Err(anyhow!("Registration is cancelled")),
            RegistrationStatus::NoShow     => return Err(anyhow!("Marked as no-show")),
            RegistrationStatus::PendingPayment => return Err(anyhow!("Payment not confirmed")),
        }

        let event_id = reg.event_id;
        let tenant_id = reg.tenant_id;
        let now = Utc::now();

        let mut active: atlas_event_registration::ActiveModel = reg.into();
        active.status = Set(RegistrationStatus::CheckedIn.to_string());
        active.checked_in_at = Set(Some(now));
        let updated = active.update(&txn).await?;

        // Increment attended_count on the event.
        use sea_orm::ConnectionTrait;
        txn.execute_unprepared(&format!(
            "UPDATE atlas_events SET attended_count = attended_count + 1, updated_at = NOW() WHERE id = '{event_id}'"
        ))
        .await
        .map_err(|e| anyhow!("attended_count update failed: {e:#}"))?;

        txn.commit().await?;

        tracing::info!(
            %tenant_id, %event_id, registration_id = %updated.id,
            "EventService::check_in: attendee checked in"
        );

        Ok(updated)
    }

    // ── Registration listing ──────────────────────────────────────────────────

    /// List all registrations for an event, optionally filtered by status.
    pub async fn list_registrations(
        db: &DatabaseConnection,
        tenant_id: Uuid,
        event_id: Uuid,
        status_filter: Option<RegistrationStatus>,
    ) -> Result<Vec<atlas_event_registration::Model>> {
        Self::get(db, tenant_id, event_id).await?;

        let mut q = atlas_event_registration::Entity::find()
            .filter(atlas_event_registration::Column::EventId.eq(event_id));

        if let Some(st) = status_filter {
            q = q.filter(atlas_event_registration::Column::Status.eq(st.to_string()));
        }

        Ok(q.order_by_asc(atlas_event_registration::Column::CreatedAt)
            .all(db)
            .await?)
    }
}
