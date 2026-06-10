//! Folio — STR Guest Service
//!
//! Manages per-booking guest registration, vehicle declarations, and special
//! requests for Short-Term Rentals (STR). All records are stored in
//! `atlas_record_relationships` (G-22) scoped to a `reservation_id` — no new
//! migrations required.
//!
//! # Why G-22 and not reservation_metadata?
//!
//! Storing guests as G-22 relationships (not JSONB blobs) enables:
//! - Individual guest lookups (compliance audit: "who stayed on Dec 24?")
//! - Cross-booking guest history ("has this passport been flagged?")
//! - Index-efficient queries on `source_entity_id = reservation_id`
//!
//! Special requests and vehicles — which are transient display-only data —
//! are stored in `reservation_metadata` JSONB to avoid over-engineering.
//!
//! # Relationship types used
//!
//! | relationship_type    | Description                         |
//! |----------------------|-------------------------------------|
//! | `"str_guest"`        | A person registered on this booking |
//! | `"str_vehicle"`      | A vehicle declared on this booking  |
//!
//! # Jurisdiction compliance
//!
//! Many markets (ES, PT, FR, MX, IT) require all guests to be registered with
//! full name + passport number before or at check-in. `StrGuestService` provides
//! the storage layer; compliance enforcement is handled by `str_compliance.rs`.
//!
//! # Document type
//!
//! `DocumentType` is a typed enum — cannot register a guest with a raw string
//! document type that doesn't exist in the system.

use anyhow::{bail, Result};
use chrono::{NaiveDate, Utc};
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, Set,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::entities::atlas_record_relationship;

// ── Relationship type constants ───────────────────────────────────────────────

pub const STR_GUEST_RELATIONSHIP: &str = "str_guest";
pub const STR_VEHICLE_RELATIONSHIP: &str = "str_vehicle";

// ═══════════════════════════════════════════════════════════════════════════════
// Newtypes — invalid states are unrepresentable
// ═══════════════════════════════════════════════════════════════════════════════

/// ISO 3166-1 alpha-2 nationality code ("BR", "US", "FR").
/// Validated at construction — cannot be 3-letter or empty.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NationalityCode(String);

impl NationalityCode {
    pub fn new(s: &str) -> Result<Self> {
        let upper = s.trim().to_uppercase();
        if upper.len() != 2 || !upper.chars().all(|c| c.is_ascii_alphabetic()) {
            bail!("nationality code '{}' must be exactly 2 ASCII letters (ISO 3166-1 alpha-2)", s);
        }
        Ok(Self(upper))
    }
    pub fn value(&self) -> &str { &self.0 }
}

impl std::fmt::Display for NationalityCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

/// Government-issued document number.
/// Alphanumeric + hyphens, 4–30 chars (covers passports, national IDs, driving licences).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DocumentNumber(String);

impl DocumentNumber {
    pub fn new(s: &str) -> Result<Self> {
        let trimmed = s.trim();
        if trimmed.len() < 4 || trimmed.len() > 30 {
            bail!("document number '{}' must be 4–30 characters", s);
        }
        if !trimmed.chars().all(|c| c.is_alphanumeric() || c == '-') {
            bail!("document number '{}' must contain only alphanumeric characters and hyphens", s);
        }
        Ok(Self(trimmed.to_uppercase()))
    }
    pub fn value(&self) -> &str { &self.0 }
}

impl std::fmt::Display for DocumentNumber {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// Enums — typed document categories
// ═══════════════════════════════════════════════════════════════════════════════

/// Government-issued identity document types accepted for STR guest registration.
///
/// These align with the document types required by major STR-compliance
/// jurisdictions (Airbnb's Guest ID Verification program, EU Directive 2016/681).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DocumentType {
    /// Internationally recognised travel document.
    Passport,
    /// Government-issued national identity card (EU member states).
    NationalId,
    /// Driver's licence / permis de conduire / carteira de habilitação.
    DriverLicence,
    /// Alien registration card / residence permit.
    ResidencePermit,
    /// Visa / entry permit (used when passport not yet returned by embassy).
    Visa,
    /// Other government-issued photo ID — jurisdiction must accept this type.
    OtherGovId,
}

impl std::fmt::Display for DocumentType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::Passport        => "passport",
            Self::NationalId      => "national_id",
            Self::DriverLicence   => "driver_licence",
            Self::ResidencePermit => "residence_permit",
            Self::Visa            => "visa",
            Self::OtherGovId      => "other_gov_id",
        })
    }
}

impl TryFrom<String> for DocumentType {
    type Error = String;
    fn try_from(s: String) -> Result<Self, Self::Error> {
        match s.as_str() {
            "passport"         => Ok(Self::Passport),
            "national_id"      => Ok(Self::NationalId),
            "driver_licence"   => Ok(Self::DriverLicence),
            "residence_permit" => Ok(Self::ResidencePermit),
            "visa"             => Ok(Self::Visa),
            "other_gov_id"     => Ok(Self::OtherGovId),
            other              => Err(format!("unknown DocumentType: '{other}'")),
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// Input types
// ═══════════════════════════════════════════════════════════════════════════════

/// Input for registering a guest on an STR booking.
///
/// The lead guest (`is_lead_guest = true`) is the booking holder.
/// All additional guests are supporting guests. There can be exactly one
/// lead guest per reservation — enforced at write time.
#[derive(Debug)]
pub struct RegisterStrGuestInput {
    /// Full legal name as it appears on the document.
    pub full_name: String,
    /// Typed nationality (ISO 3166-1 alpha-2) — e.g., `NationalityCode::new("BR")`.
    pub nationality: NationalityCode,
    /// Date of birth — used for minor detection and jurisdiction age checks.
    pub date_of_birth: NaiveDate,
    /// Document type — typed, cannot be an arbitrary string.
    pub document_type: DocumentType,
    /// Validated document number.
    pub document_number: DocumentNumber,
    /// Whether this guest is the primary booking holder.
    pub is_lead_guest: bool,
    /// Optional: user account ID if this guest has a Folio account.
    pub user_account_id: Option<Uuid>,
}

/// Input for registering a vehicle on an STR booking.
///
/// STR vehicle registration is lightweight — just what's needed for
/// parking management and security. No newtype validation for model year
/// (that's for LTR permanent registrations). License plate still validated
/// to prevent garbage data in the parking log.
#[derive(Debug)]
pub struct RegisterStrVehicleInput {
    pub license_plate: String,
    pub make: Option<String>,
    pub model: Option<String>,
    pub color: Option<String>,
    /// Assigned parking spot (e.g., "B3", "Underground Level 2").
    pub parking_spot: Option<String>,
}

impl RegisterStrVehicleInput {
    /// Basic plate validation — non-empty, ≤20 chars, alphanumeric + hyphens/spaces.
    pub fn validate_plate(plate: &str) -> Result<()> {
        let trimmed = plate.trim();
        if trimmed.is_empty() {
            bail!("license plate cannot be empty");
        }
        if trimmed.len() > 20 {
            bail!("license plate '{}' exceeds 20 characters", plate);
        }
        if !trimmed.chars().all(|c| c.is_alphanumeric() || c == '-' || c == ' ') {
            bail!("license plate '{}' must contain only alphanumeric characters, hyphens, or spaces", plate);
        }
        Ok(())
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// Output types
// ═══════════════════════════════════════════════════════════════════════════════

#[derive(Debug, Clone, Serialize)]
pub struct StrGuest {
    pub rel_id: Uuid,
    pub full_name: String,
    pub nationality: String,
    pub date_of_birth: NaiveDate,
    pub document_type: String,
    pub document_number: String,
    pub is_lead_guest: bool,
    pub user_account_id: Option<Uuid>,
    pub registered_at: chrono::DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize)]
pub struct StrVehicle {
    pub rel_id: Uuid,
    pub license_plate: String,
    pub make: Option<String>,
    pub model: Option<String>,
    pub color: Option<String>,
    pub parking_spot: Option<String>,
    pub registered_at: chrono::DateTime<Utc>,
}

/// Reservation-level guest + vehicle manifest — returned by `get_manifest`.
#[derive(Debug, Clone, Serialize)]
pub struct ReservationManifest {
    pub reservation_id: Uuid,
    pub guests: Vec<StrGuest>,
    pub vehicles: Vec<StrVehicle>,
    pub special_requests: Vec<String>,
    pub guest_count: u32,
}

// ═══════════════════════════════════════════════════════════════════════════════
// StrGuestService
// ═══════════════════════════════════════════════════════════════════════════════

pub struct StrGuestService;

impl StrGuestService {
    // ── Guest registration ────────────────────────────────────────────────────

    /// Register a guest on an STR reservation.
    ///
    /// # Lead guest enforcement
    /// If `is_lead_guest = true` and a lead guest already exists on this
    /// reservation, the new guest is rejected. Only one lead guest per booking.
    ///
    /// Returns the G-22 relationship ID.
    pub async fn register_guest(
        db: &DatabaseConnection,
        tenant_id: Uuid,
        reservation_id: Uuid,
        created_by_user_id: Uuid,
        input: RegisterStrGuestInput,
    ) -> Result<Uuid> {
        // Enforce single lead guest per reservation
        if input.is_lead_guest {
            let existing_guests = Self::list_guests(db, tenant_id, reservation_id).await?;
            if existing_guests.iter().any(|g| g.is_lead_guest) {
                bail!(
                    "reservation {} already has a lead guest — remove the existing lead guest before registering a new one",
                    reservation_id
                );
            }
        }

        let rel_id = Uuid::new_v4();
        let now = Utc::now();

        atlas_record_relationship::ActiveModel {
            id: Set(rel_id),
            tenant_id: Set(tenant_id),
            source_entity_type: Set("atlas_reservation".to_string()),
            source_entity_id: Set(reservation_id),
            target_entity_type: Set("str_guest".to_string()),
            target_entity_id: Set(Uuid::new_v4()), // synthetic ID — guest has no FK table
            relationship_type: Set(STR_GUEST_RELATIONSHIP.to_string()),
            inverse_label: Set(None),
            relationship_metadata: Set(Some(serde_json::json!({
                "full_name":       input.full_name,
                "nationality":     input.nationality.value(),
                "date_of_birth":   input.date_of_birth.to_string(),
                "document_type":   input.document_type.to_string(),
                "document_number": input.document_number.value(),
                "is_lead_guest":   input.is_lead_guest,
                "user_account_id": input.user_account_id,
            }))),
            created_by_user_id: Set(Some(created_by_user_id)),
            created_at: Set(now),
        }
        .insert(db)
        .await?;

        tracing::info!(
            %rel_id, %tenant_id, %reservation_id,
            is_lead = input.is_lead_guest,
            "StrGuestService: guest registered"
        );

        Ok(rel_id)
    }

    /// Remove a guest from a reservation (hard delete — no retention requirement for STR guests).
    pub async fn remove_guest(
        db: &DatabaseConnection,
        tenant_id: Uuid,
        reservation_id: Uuid,
        rel_id: Uuid,
    ) -> Result<()> {
        let rel = atlas_record_relationship::Entity::find_by_id(rel_id)
            .filter(atlas_record_relationship::Column::TenantId.eq(tenant_id))
            .filter(atlas_record_relationship::Column::SourceEntityId.eq(reservation_id))
            .filter(atlas_record_relationship::Column::RelationshipType.eq(STR_GUEST_RELATIONSHIP))
            .one(db)
            .await?
            .ok_or_else(|| anyhow::anyhow!("guest registration {} not found on reservation {}", rel_id, reservation_id))?;

        atlas_record_relationship::Entity::delete_by_id(rel.id)
            .exec(db)
            .await?;

        tracing::info!(%rel_id, %reservation_id, %tenant_id, "StrGuestService: guest removed");
        Ok(())
    }

    /// List all registered guests on a reservation.
    pub async fn list_guests(
        db: &DatabaseConnection,
        tenant_id: Uuid,
        reservation_id: Uuid,
    ) -> Result<Vec<StrGuest>> {
        let rels = atlas_record_relationship::Entity::find()
            .filter(atlas_record_relationship::Column::TenantId.eq(tenant_id))
            .filter(atlas_record_relationship::Column::SourceEntityType.eq("atlas_reservation"))
            .filter(atlas_record_relationship::Column::SourceEntityId.eq(reservation_id))
            .filter(atlas_record_relationship::Column::RelationshipType.eq(STR_GUEST_RELATIONSHIP))
            .all(db)
            .await?;

        Ok(rels.into_iter().filter_map(|r| parse_guest(r)).collect())
    }

    // ── Vehicle registration ──────────────────────────────────────────────────

    /// Register a vehicle on an STR reservation.
    /// Validates the license plate before writing.
    /// Returns the G-22 relationship ID.
    pub async fn register_vehicle(
        db: &DatabaseConnection,
        tenant_id: Uuid,
        reservation_id: Uuid,
        created_by_user_id: Uuid,
        input: RegisterStrVehicleInput,
    ) -> Result<Uuid> {
        RegisterStrVehicleInput::validate_plate(&input.license_plate)?;

        let rel_id = Uuid::new_v4();
        let now = Utc::now();

        atlas_record_relationship::ActiveModel {
            id: Set(rel_id),
            tenant_id: Set(tenant_id),
            source_entity_type: Set("atlas_reservation".to_string()),
            source_entity_id: Set(reservation_id),
            target_entity_type: Set("str_vehicle".to_string()),
            target_entity_id: Set(Uuid::new_v4()), // synthetic
            relationship_type: Set(STR_VEHICLE_RELATIONSHIP.to_string()),
            inverse_label: Set(None),
            relationship_metadata: Set(Some(serde_json::json!({
                "license_plate": input.license_plate.trim().to_uppercase(),
                "make":          input.make,
                "model":         input.model,
                "color":         input.color,
                "parking_spot":  input.parking_spot,
            }))),
            created_by_user_id: Set(Some(created_by_user_id)),
            created_at: Set(now),
        }
        .insert(db)
        .await?;

        tracing::info!(
            %rel_id, %tenant_id, %reservation_id,
            "StrGuestService: vehicle registered"
        );

        Ok(rel_id)
    }

    /// Remove a vehicle registration (hard delete).
    pub async fn remove_vehicle(
        db: &DatabaseConnection,
        tenant_id: Uuid,
        reservation_id: Uuid,
        rel_id: Uuid,
    ) -> Result<()> {
        let rel = atlas_record_relationship::Entity::find_by_id(rel_id)
            .filter(atlas_record_relationship::Column::TenantId.eq(tenant_id))
            .filter(atlas_record_relationship::Column::SourceEntityId.eq(reservation_id))
            .filter(atlas_record_relationship::Column::RelationshipType.eq(STR_VEHICLE_RELATIONSHIP))
            .one(db)
            .await?
            .ok_or_else(|| anyhow::anyhow!("vehicle {} not found on reservation {}", rel_id, reservation_id))?;

        atlas_record_relationship::Entity::delete_by_id(rel.id)
            .exec(db)
            .await?;

        tracing::info!(%rel_id, %reservation_id, %tenant_id, "StrGuestService: vehicle removed");
        Ok(())
    }

    /// List all registered vehicles on a reservation.
    pub async fn list_vehicles(
        db: &DatabaseConnection,
        tenant_id: Uuid,
        reservation_id: Uuid,
    ) -> Result<Vec<StrVehicle>> {
        let rels = atlas_record_relationship::Entity::find()
            .filter(atlas_record_relationship::Column::TenantId.eq(tenant_id))
            .filter(atlas_record_relationship::Column::SourceEntityType.eq("atlas_reservation"))
            .filter(atlas_record_relationship::Column::SourceEntityId.eq(reservation_id))
            .filter(atlas_record_relationship::Column::RelationshipType.eq(STR_VEHICLE_RELATIONSHIP))
            .all(db)
            .await?;

        Ok(rels.into_iter().filter_map(|r| parse_vehicle(r)).collect())
    }

    // ── Special requests ──────────────────────────────────────────────────────

    /// Update the special requests list on a reservation.
    ///
    /// Overwrites the `special_requests` key in `reservation_metadata`.
    /// Other metadata keys are preserved via JSONB merge.
    pub async fn set_special_requests(
        db: &DatabaseConnection,
        tenant_id: Uuid,
        reservation_id: Uuid,
        requests: Vec<String>,
    ) -> Result<()> {
        use crate::entities::atlas_reservation;

        let reservation = atlas_reservation::Entity::find_by_id(reservation_id)
            .filter(atlas_reservation::Column::TenantId.eq(tenant_id))
            .one(db)
            .await?
            .ok_or_else(|| anyhow::anyhow!("reservation {} not found", reservation_id))?;

        let mut meta = reservation.reservation_metadata.clone();
        if let Some(obj) = meta.as_object_mut() {
            obj.insert(
                "special_requests".to_string(),
                serde_json::Value::Array(
                    requests.iter().map(|r| serde_json::Value::String(r.clone())).collect()
                ),
            );
        }

        let mut active: atlas_reservation::ActiveModel = reservation.into();
        active.reservation_metadata = Set(meta);
        active.update(db).await?;

        tracing::info!(%reservation_id, %tenant_id, "StrGuestService: special requests updated");
        Ok(())
    }

    // ── Full manifest ─────────────────────────────────────────────────────────

    /// Return the full guest + vehicle + special request manifest for a reservation.
    pub async fn get_manifest(
        db: &DatabaseConnection,
        tenant_id: Uuid,
        reservation_id: Uuid,
    ) -> Result<ReservationManifest> {
        use crate::entities::atlas_reservation;

        let reservation = atlas_reservation::Entity::find_by_id(reservation_id)
            .filter(atlas_reservation::Column::TenantId.eq(tenant_id))
            .one(db)
            .await?
            .ok_or_else(|| anyhow::anyhow!("reservation {} not found", reservation_id))?;

        let guests   = Self::list_guests(db, tenant_id, reservation_id).await?;
        let vehicles = Self::list_vehicles(db, tenant_id, reservation_id).await?;

        let special_requests: Vec<String> = reservation.reservation_metadata
            .get("special_requests")
            .and_then(|v| v.as_array())
            .map(|arr| arr.iter().filter_map(|v| v.as_str().map(String::from)).collect())
            .unwrap_or_default();

        let guest_count = reservation.reservation_metadata
            .get("guest_count")
            .and_then(|v| v.as_u64())
            .unwrap_or(guests.len() as u64) as u32;

        Ok(ReservationManifest {
            reservation_id,
            guests,
            vehicles,
            special_requests,
            guest_count,
        })
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// Helpers — parse G-22 metadata JSONB into typed structs
// ═══════════════════════════════════════════════════════════════════════════════

fn parse_guest(r: atlas_record_relationship::Model) -> Option<StrGuest> {
    let meta = r.relationship_metadata.as_ref()?;
    let full_name       = meta["full_name"].as_str()?.to_string();
    let nationality     = meta["nationality"].as_str()?.to_string();
    let dob_str         = meta["date_of_birth"].as_str()?;
    let date_of_birth   = NaiveDate::parse_from_str(dob_str, "%Y-%m-%d").ok()?;
    let document_type   = meta["document_type"].as_str()?.to_string();
    let document_number = meta["document_number"].as_str()?.to_string();
    let is_lead_guest   = meta["is_lead_guest"].as_bool().unwrap_or(false);
    let user_account_id = meta["user_account_id"].as_str()
        .and_then(|s| Uuid::parse_str(s).ok());

    Some(StrGuest {
        rel_id: r.id,
        full_name,
        nationality,
        date_of_birth,
        document_type,
        document_number,
        is_lead_guest,
        user_account_id,
        registered_at: r.created_at,
    })
}

fn parse_vehicle(r: atlas_record_relationship::Model) -> Option<StrVehicle> {
    let meta = r.relationship_metadata.as_ref()?;
    let license_plate = meta["license_plate"].as_str()?.to_string();

    Some(StrVehicle {
        rel_id: r.id,
        license_plate,
        make:         meta["make"].as_str().map(String::from),
        model:        meta["model"].as_str().map(String::from),
        color:        meta["color"].as_str().map(String::from),
        parking_spot: meta["parking_spot"].as_str().map(String::from),
        registered_at: r.created_at,
    })
}
