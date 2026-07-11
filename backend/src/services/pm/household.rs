//! Folio — Household Service
//!
//! Manages lease-scoped tenant declarations: registered vehicles and household
//! members. Stored in G-22 `atlas_record_relationships` linked to the lease
//! contract — never in `atlas_assets` (which is the landlord's property register).
//!
//! # Type safety philosophy
//!
//! Validated newtypes (`LicensePlate`, `ModelYear`, `CountryCode`) enforce
//! invariants at the boundary so invalid data is rejected before reaching the DB.
//!
//! Occupant registration uses an enum (`RegisterOccupantInput`) to make
//! `date_of_birth` **required** for Child/Dependent at compile time — not optional.
//!
//! Return types split `ActiveOccupant` from `FormerOccupant` so callers never
//! have to null-check `removed_at` — the type carries that fact.

use anyhow::{Result, anyhow, bail};
use chrono::{Datelike, NaiveDate, Utc};
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, QueryOrder, Set,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::entities::{atlas_contract, atlas_record_relationship};

// ── Relationship type constants ───────────────────────────────────────────────

const REL_VEHICLE: &str = "registered_vehicle";
const REL_HOUSEHOLD: &str = "household_member";
const TARGET_VEHICLE: &str = "tenant_vehicle";
const TARGET_ADULT: &str = "household_adult";
const TARGET_MINOR: &str = "household_minor";
const SOURCE_CONTRACT: &str = "atlas_contract";

// ═══════════════════════════════════════════════════════════════════════════════
// Validated newtypes — invalid data is rejected at construction, never at the DB
// ═══════════════════════════════════════════════════════════════════════════════

/// A non-empty, trimmed, uppercase license plate (max 15 chars).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LicensePlate(String);

impl LicensePlate {
    pub fn new(s: impl Into<String>) -> Result<Self> {
        let s = s.into().trim().to_uppercase();
        if s.is_empty() {
            bail!("license plate cannot be empty");
        }
        if s.len() > 15 {
            bail!("license plate '{}' exceeds 15-character maximum", s);
        }
        // Alphanumeric + hyphens only
        if !s.chars().all(|c| c.is_ascii_alphanumeric() || c == '-') {
            bail!("license plate '{}' contains invalid characters", s);
        }
        Ok(Self(s))
    }
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for LicensePlate {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

/// A vehicle model year — must be between 1886 (first automobile) and next year.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct ModelYear(u16);

impl ModelYear {
    pub fn new(year: i32) -> Result<Self> {
        let next_year = Utc::now().year() + 2; // allow pre-orders
        if year < 1886 {
            bail!(
                "model year {} is before the automobile was invented (1886)",
                year
            );
        }
        if year > next_year {
            bail!(
                "model year {} is too far in the future (max {})",
                year,
                next_year
            );
        }
        Ok(Self(year as u16))
    }
    pub fn value(self) -> i32 {
        self.0 as i32
    }
}

impl std::fmt::Display for ModelYear {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// ISO 3166-1 alpha-2 country code (two uppercase ASCII letters).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CountryCode(String);

impl CountryCode {
    pub fn new(s: impl Into<String>) -> Result<Self> {
        let s = s.into().trim().to_uppercase();
        if s.len() != 2 || !s.chars().all(|c| c.is_ascii_alphabetic()) {
            bail!("'{}' is not a valid ISO 3166-1 alpha-2 country code", s);
        }
        Ok(Self(s))
    }
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for CountryCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// Vehicle types
// ═══════════════════════════════════════════════════════════════════════════════

#[derive(Debug, Clone)]
pub struct RegisterVehicleInput {
    pub make: String,
    pub model: String,
    pub year: ModelYear,
    pub color: String,
    pub license_plate: LicensePlate,
    pub state: String,
    pub country: CountryCode,
    pub parking_spot: Option<String>,
    pub registration_expiry: Option<NaiveDate>,
}

/// At least one field must be `Some` — enforced by `UpdateVehicleInput::new()`.
/// The `make` + `model` + `year` + `color` describe the *new* car.
/// `license_plate` is what changes when they switch cars mid-lease.
#[derive(Debug, Clone)]
pub struct UpdateVehicleInput {
    pub make: Option<String>,
    pub model: Option<String>,
    pub year: Option<ModelYear>,
    pub color: Option<String>,
    pub license_plate: Option<LicensePlate>,
    pub state: Option<String>,
    pub country: Option<CountryCode>,
    pub parking_spot: Option<String>,
    pub registration_expiry: Option<NaiveDate>,
}

impl UpdateVehicleInput {
    /// Returns `Err` if every field is `None` — a no-op patch is a caller error.
    pub fn validate(&self) -> Result<()> {
        let any_set = self.make.is_some()
            || self.model.is_some()
            || self.year.is_some()
            || self.color.is_some()
            || self.license_plate.is_some()
            || self.state.is_some()
            || self.country.is_some()
            || self.parking_spot.is_some()
            || self.registration_expiry.is_some();
        if !any_set {
            bail!("UpdateVehicleInput: at least one field must be provided");
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct VehicleRecord {
    /// The stable ID for this vehicle entry (`target_entity_id` in the relationship).
    pub id: Uuid,
    pub lease_id: Uuid,
    pub make: String,
    pub model: String,
    pub year: i32,
    pub color: String,
    pub license_plate: String,
    pub state: String,
    pub country: String,
    pub parking_spot: Option<String>,
    pub registration_expiry: Option<NaiveDate>,
    pub registered_at: chrono::DateTime<Utc>,
    /// Present if vehicle details were updated mid-lease (e.g. tenant got a new car).
    pub updated_at: Option<chrono::DateTime<Utc>>,
}

// ═══════════════════════════════════════════════════════════════════════════════
// Occupant types — type-safe adult vs minor distinction
// ═══════════════════════════════════════════════════════════════════════════════

/// Adult occupant relationships. Do NOT include Child/Dependent here —
/// those require a DOB and go through `RegisterMinorInput`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum AdultRelationship {
    CoTenant,
    Roommate,
    Spouse,
    Partner,
    Other,
}

impl std::fmt::Display for AdultRelationship {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::CoTenant => "co_tenant",
            Self::Roommate => "roommate",
            Self::Spouse => "spouse",
            Self::Partner => "partner",
            Self::Other => "other",
        })
    }
}

/// Minor occupant types. Separated from AdultRelationship so that
/// `date_of_birth` is **required at compile time** — not an `Option`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum MinorRelationship {
    Child,
    Dependent,
}

impl std::fmt::Display for MinorRelationship {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::Child => "child",
            Self::Dependent => "dependent",
        })
    }
}

/// Registering an adult: no DOB required.
#[derive(Debug, Clone)]
pub struct RegisterAdultInput {
    pub full_name: String,
    pub relationship: AdultRelationship,
    /// If this person is also a platform user, link their profile.
    pub profile_id: Option<Uuid>,
    pub id_document_type: Option<String>,
    pub id_document_number: Option<String>,
    pub notes: Option<String>,
}

/// Registering a minor: `date_of_birth` is NOT `Option` — it is always required.
/// This is enforced at the type level: you cannot construct this struct without a DOB.
#[derive(Debug, Clone)]
pub struct RegisterMinorInput {
    pub full_name: String,
    pub relationship: MinorRelationship,
    /// Required for children/dependents — not optional.
    pub date_of_birth: NaiveDate,
    pub notes: Option<String>,
}

/// Sealed enum makes the adult/minor distinction exhaustive at the call site.
/// Callers must handle both cases — they cannot accidentally treat a minor as adult.
#[derive(Debug, Clone)]
pub enum RegisterOccupantInput {
    Adult(RegisterAdultInput),
    Minor(RegisterMinorInput),
}

/// Partial update — corrections only (typo in name, added doc number).
/// Use `remove_occupant` for departures.
#[derive(Debug, Clone)]
pub struct UpdateOccupantInput {
    pub full_name: Option<String>,
    pub id_document_type: Option<String>,
    pub id_document_number: Option<String>,
    pub notes: Option<String>,
}

impl UpdateOccupantInput {
    pub fn validate(&self) -> Result<()> {
        let any = self.full_name.is_some()
            || self.id_document_type.is_some()
            || self.id_document_number.is_some()
            || self.notes.is_some();
        if !any {
            bail!("UpdateOccupantInput: at least one field must be provided");
        }
        Ok(())
    }
}

/// Why an occupant left. Stored permanently in relationship_metadata —
/// this is a legal/liability record that must never be deleted.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum DepartureReason {
    MovedOut,
    RelationshipEnded,
    LeaseViolation,
    Deceased,
    Other,
}

impl std::fmt::Display for DepartureReason {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::MovedOut => "moved_out",
            Self::RelationshipEnded => "relationship_ended",
            Self::LeaseViolation => "lease_violation",
            Self::Deceased => "deceased",
            Self::Other => "other",
        })
    }
}

// ── Occupant return types — active and former are distinct ────────────────────

/// An occupant currently living in the unit.
/// `removed_at` is absent at the type level — callers cannot mistake this for a
/// former occupant and don't need to null-check anything.
#[derive(Debug, Clone, Serialize)]
pub struct ActiveOccupant {
    pub id: Uuid,
    pub lease_id: Uuid,
    pub full_name: String,
    pub relationship: String,
    pub is_minor: bool,
    pub date_of_birth: Option<NaiveDate>, // Some for minors, None for adults
    pub profile_id: Option<Uuid>,
    pub id_document_type: Option<String>,
    pub registered_at: chrono::DateTime<Utc>,
}

/// An occupant who has departed mid-lease (soft-deleted).
/// `removed_at` and `removal_reason` are NOT `Option` here — they are always
/// present because this type can only be constructed from a departed record.
#[derive(Debug, Clone, Serialize)]
pub struct FormerOccupant {
    pub id: Uuid,
    pub lease_id: Uuid,
    pub full_name: String,
    pub relationship: String,
    pub is_minor: bool,
    pub date_of_birth: Option<NaiveDate>,
    pub profile_id: Option<Uuid>,
    pub id_document_type: Option<String>,
    pub registered_at: chrono::DateTime<Utc>,
    /// Always present — this is why you use `FormerOccupant` not `ActiveOccupant`.
    pub removed_at: chrono::DateTime<Utc>,
    pub removal_reason: DepartureReason,
    pub departure_notes: Option<String>,
}

// ═══════════════════════════════════════════════════════════════════════════════
// HouseholdService
// ═══════════════════════════════════════════════════════════════════════════════

pub struct HouseholdService;

impl HouseholdService {
    // ── Vehicles ──────────────────────────────────────────────────────────────

    /// Register a vehicle under an active lease.
    /// `lease_id` is a function parameter, not embedded in `input`, to avoid
    /// the ambiguity of "which lease_id wins?" at the call site.
    pub async fn register_vehicle(
        db: &DatabaseConnection,
        tenant_id: Uuid,
        user_id: Uuid,
        lease_id: Uuid,
        input: RegisterVehicleInput,
    ) -> Result<VehicleRecord> {
        let lease = Self::assert_lease_access(db, tenant_id, user_id, lease_id).await?;
        let entry_id = Uuid::new_v4();
        let now = Utc::now();

        let meta = serde_json::json!({
            "make":                input.make,
            "model":               input.model,
            "year":                input.year.value(),
            "color":               input.color,
            "license_plate":       input.license_plate.as_str(),
            "state":               input.state,
            "country":             input.country.as_str(),
            "parking_spot":        input.parking_spot,
            "registration_expiry": input.registration_expiry.map(|d| d.to_string()),
        });

        atlas_record_relationship::ActiveModel {
            id: Set(Uuid::new_v4()),
            tenant_id: Set(tenant_id),
            source_entity_type: Set(SOURCE_CONTRACT.to_string()),
            source_entity_id: Set(lease.id),
            target_entity_type: Set(TARGET_VEHICLE.to_string()),
            target_entity_id: Set(entry_id),
            relationship_type: Set(REL_VEHICLE.to_string()),
            inverse_label: Set(None),
            relationship_metadata: Set(Some(meta)),
            created_by_user_id: Set(Some(user_id)),
            created_at: Set(now),
        }
        .insert(db)
        .await?;

        tracing::info!(%tenant_id, %user_id, %lease_id, plate = %input.license_plate,
            "HouseholdService: vehicle registered");

        Ok(VehicleRecord {
            id: entry_id,
            lease_id,
            make: input.make,
            model: input.model,
            year: input.year.value(),
            color: input.color,
            license_plate: input.license_plate.to_string(),
            state: input.state,
            country: input.country.to_string(),
            parking_spot: input.parking_spot,
            registration_expiry: input.registration_expiry,
            registered_at: now,
            updated_at: None,
        })
    }

    pub async fn list_vehicles(
        db: &DatabaseConnection,
        tenant_id: Uuid,
        lease_id: Uuid,
    ) -> Result<Vec<VehicleRecord>> {
        let rows = atlas_record_relationship::Entity::find()
            .filter(atlas_record_relationship::Column::TenantId.eq(tenant_id))
            .filter(atlas_record_relationship::Column::SourceEntityType.eq(SOURCE_CONTRACT))
            .filter(atlas_record_relationship::Column::SourceEntityId.eq(lease_id))
            .filter(atlas_record_relationship::Column::RelationshipType.eq(REL_VEHICLE))
            .order_by_asc(atlas_record_relationship::Column::CreatedAt)
            .all(db)
            .await?;

        Ok(rows
            .into_iter()
            .filter_map(|r| parse_vehicle(lease_id, r))
            .collect())
    }

    /// Update a vehicle mid-lease (tenant got a new car, parking spot changed, etc.)
    /// Validates the patch is non-empty before touching the DB.
    pub async fn update_vehicle(
        db: &DatabaseConnection,
        tenant_id: Uuid,
        user_id: Uuid,
        lease_id: Uuid,
        entry_id: Uuid,
        patch: UpdateVehicleInput,
    ) -> Result<VehicleRecord> {
        patch.validate()?;
        Self::assert_lease_access(db, tenant_id, user_id, lease_id).await?;
        let now = Utc::now();

        let row = atlas_record_relationship::Entity::find()
            .filter(atlas_record_relationship::Column::TenantId.eq(tenant_id))
            .filter(atlas_record_relationship::Column::SourceEntityId.eq(lease_id))
            .filter(atlas_record_relationship::Column::TargetEntityId.eq(entry_id))
            .filter(atlas_record_relationship::Column::RelationshipType.eq(REL_VEHICLE))
            .one(db)
            .await?
            .ok_or_else(|| anyhow!("vehicle {entry_id} not found on lease {lease_id}"))?;

        let mut meta = row
            .relationship_metadata
            .clone()
            .unwrap_or_else(|| serde_json::json!({}));

        macro_rules! patch_str {
            ($field:literal, $val:expr) => {
                if let Some(v) = $val {
                    meta[$field] = serde_json::json!(v);
                }
            };
        }
        patch_str!("make", patch.make);
        patch_str!("model", patch.model);
        patch_str!("color", patch.color);
        patch_str!("state", patch.state);
        if let Some(p) = patch.license_plate {
            meta["license_plate"] = serde_json::json!(p.as_str());
        }
        if let Some(c) = patch.country {
            meta["country"] = serde_json::json!(c.as_str());
        }
        if let Some(y) = patch.year {
            meta["year"] = serde_json::json!(y.value());
        }
        if let Some(ps) = patch.parking_spot {
            meta["parking_spot"] = serde_json::json!(ps);
        }
        if let Some(e) = patch.registration_expiry {
            meta["registration_expiry"] = serde_json::json!(e.to_string());
        }
        meta["updated_at"] = serde_json::json!(now.to_rfc3339());

        let mut active: atlas_record_relationship::ActiveModel = row.into();
        active.relationship_metadata = Set(Some(meta));
        let updated = active.update(db).await?;

        tracing::info!(%tenant_id, %user_id, %lease_id, vehicle_id = %entry_id,
            "HouseholdService: vehicle updated");
        parse_vehicle(lease_id, updated).ok_or_else(|| anyhow!("failed to parse updated vehicle"))
    }

    /// Hard-delete a vehicle. Vehicles have no legal retention requirement.
    /// For occupant departures, use the soft-delete `remove_occupant` instead.
    pub async fn remove_vehicle(
        db: &DatabaseConnection,
        tenant_id: Uuid,
        user_id: Uuid,
        lease_id: Uuid,
        entry_id: Uuid,
    ) -> Result<bool> {
        Self::assert_lease_access(db, tenant_id, user_id, lease_id).await?;
        let result = atlas_record_relationship::Entity::delete_many()
            .filter(atlas_record_relationship::Column::TenantId.eq(tenant_id))
            .filter(atlas_record_relationship::Column::SourceEntityType.eq(SOURCE_CONTRACT))
            .filter(atlas_record_relationship::Column::SourceEntityId.eq(lease_id))
            .filter(atlas_record_relationship::Column::TargetEntityId.eq(entry_id))
            .filter(atlas_record_relationship::Column::RelationshipType.eq(REL_VEHICLE))
            .exec(db)
            .await?;
        Ok(result.rows_affected > 0)
    }

    // ── Household members ─────────────────────────────────────────────────────

    /// Register a household member.
    ///
    /// The `RegisterOccupantInput` enum enforces at compile time that:
    /// - Adults don't require a date of birth
    /// - Minors (Child/Dependent) **always** have a date of birth — it is not `Option`
    pub async fn register_occupant(
        db: &DatabaseConnection,
        tenant_id: Uuid,
        user_id: Uuid,
        lease_id: Uuid,
        input: RegisterOccupantInput,
    ) -> Result<ActiveOccupant> {
        let lease = Self::assert_lease_access(db, tenant_id, user_id, lease_id).await?;
        let entry_id = Uuid::new_v4();
        let now = Utc::now();

        let (target_type, meta, full_name, relationship, is_minor, dob, profile_id, doc_type) =
            match &input {
                RegisterOccupantInput::Adult(a) => {
                    let meta = serde_json::json!({
                        "full_name":            a.full_name,
                        "relationship":         a.relationship.to_string(),
                        "is_minor":             false,
                        "profile_id":           a.profile_id,
                        "id_document_type":     a.id_document_type,
                        "id_document_number":   a.id_document_number,
                        "notes":                a.notes,
                    });
                    (
                        TARGET_ADULT,
                        meta,
                        a.full_name.clone(),
                        a.relationship.to_string(),
                        false,
                        None,
                        a.profile_id,
                        a.id_document_type.clone(),
                    )
                }
                RegisterOccupantInput::Minor(m) => {
                    let meta = serde_json::json!({
                        "full_name":    m.full_name,
                        "relationship": m.relationship.to_string(),
                        "is_minor":     true,
                        "date_of_birth": m.date_of_birth.to_string(),
                        "notes":         m.notes,
                    });
                    (
                        TARGET_MINOR,
                        meta,
                        m.full_name.clone(),
                        m.relationship.to_string(),
                        true,
                        Some(m.date_of_birth),
                        None,
                        None,
                    )
                }
            };

        // Validate minor DOB is actually in the past
        if let RegisterOccupantInput::Minor(m) = &input {
            if m.date_of_birth >= Utc::now().date_naive() {
                bail!("date_of_birth for minor must be in the past");
            }
        }

        atlas_record_relationship::ActiveModel {
            id: Set(Uuid::new_v4()),
            tenant_id: Set(tenant_id),
            source_entity_type: Set(SOURCE_CONTRACT.to_string()),
            source_entity_id: Set(lease.id),
            target_entity_type: Set(target_type.to_string()),
            target_entity_id: Set(entry_id),
            relationship_type: Set(REL_HOUSEHOLD.to_string()),
            inverse_label: Set(None),
            relationship_metadata: Set(Some(meta)),
            created_by_user_id: Set(Some(user_id)),
            created_at: Set(now),
        }
        .insert(db)
        .await?;

        tracing::info!(%tenant_id, %user_id, %lease_id, name = %full_name,
            %is_minor, "HouseholdService: occupant registered");

        Ok(ActiveOccupant {
            id: entry_id,
            lease_id,
            full_name,
            relationship,
            is_minor,
            date_of_birth: dob,
            profile_id,
            id_document_type: doc_type,
            registered_at: now,
        })
    }

    /// List currently active occupants on a lease (departed occupants excluded).
    /// Returns `Vec<ActiveOccupant>` — the return type guarantees no `removed_at` field.
    pub async fn list_active_occupants(
        db: &DatabaseConnection,
        tenant_id: Uuid,
        lease_id: Uuid,
    ) -> Result<Vec<ActiveOccupant>> {
        let rows = Self::fetch_occupant_rows(db, tenant_id, lease_id).await?;
        Ok(rows
            .into_iter()
            .filter_map(|r| try_parse_active(lease_id, r))
            .collect())
    }

    /// List former occupants who departed mid-lease (soft-deleted).
    /// Returns `Vec<FormerOccupant>` — `removed_at` and `removal_reason` are always present.
    pub async fn list_former_occupants(
        db: &DatabaseConnection,
        tenant_id: Uuid,
        lease_id: Uuid,
    ) -> Result<Vec<FormerOccupant>> {
        let rows = Self::fetch_occupant_rows(db, tenant_id, lease_id).await?;
        Ok(rows
            .into_iter()
            .filter_map(|r| try_parse_former(lease_id, r))
            .collect())
    }

    /// Correction-only update — for typos, adding a document number, updating notes.
    /// Cannot change the occupant type (adult ↔ minor), relationship enum, or DOB
    /// through this method — those changes require remove + re-register for audit integrity.
    /// Errors if the occupant has already departed.
    pub async fn update_occupant(
        db: &DatabaseConnection,
        tenant_id: Uuid,
        user_id: Uuid,
        lease_id: Uuid,
        entry_id: Uuid,
        patch: UpdateOccupantInput,
    ) -> Result<ActiveOccupant> {
        patch.validate()?;
        Self::assert_lease_access(db, tenant_id, user_id, lease_id).await?;

        let row = Self::fetch_one_occupant_row(db, tenant_id, lease_id, entry_id).await?;

        // Reject updates to already-departed occupants at the type level:
        // if removed_at is in the metadata, this occupant is a FormerOccupant.
        if let Some(meta) = &row.relationship_metadata {
            if meta.get("removed_at").is_some() {
                bail!("cannot update occupant {entry_id}: already departed — use the history view");
            }
        }

        let mut meta = row
            .relationship_metadata
            .clone()
            .unwrap_or_else(|| serde_json::json!({}));

        macro_rules! patch_str {
            ($field:literal, $val:expr) => {
                if let Some(v) = $val {
                    meta[$field] = serde_json::json!(v);
                }
            };
        }
        patch_str!("full_name", patch.full_name);
        patch_str!("id_document_type", patch.id_document_type);
        patch_str!("id_document_number", patch.id_document_number);
        patch_str!("notes", patch.notes);

        let mut active: atlas_record_relationship::ActiveModel = row.into();
        active.relationship_metadata = Set(Some(meta));
        let updated = active.update(db).await?;

        tracing::info!(%tenant_id, %user_id, %lease_id, occupant_id = %entry_id,
            "HouseholdService: occupant corrected");
        try_parse_active(lease_id, updated).ok_or_else(|| {
            anyhow!("occupant {entry_id} could not be re-parsed as active after update")
        })
    }

    /// Record an occupant departure — **soft delete only, never hard delete**.
    ///
    /// Occupant records are permanent legal/liability history. The landlord needs
    /// to know who lived in a unit and when they left. A breakup or eviction is
    /// not a data deletion — it is a life event that gets recorded.
    ///
    /// Returns a `FormerOccupant` — the type carries `removed_at` and
    /// `removal_reason` as non-optional fields, making it impossible for callers
    /// to accidentally treat the record as still-active.
    pub async fn remove_occupant(
        db: &DatabaseConnection,
        tenant_id: Uuid,
        user_id: Uuid,
        lease_id: Uuid,
        entry_id: Uuid,
        reason: DepartureReason,
        notes: Option<String>,
    ) -> Result<FormerOccupant> {
        Self::assert_lease_access(db, tenant_id, user_id, lease_id).await?;
        let now = Utc::now();

        let row = Self::fetch_one_occupant_row(db, tenant_id, lease_id, entry_id).await?;

        // Guard: cannot depart someone who already departed
        if let Some(meta) = &row.relationship_metadata {
            if meta.get("removed_at").is_some() {
                bail!("occupant {entry_id} has already departed");
            }
        }

        let mut meta = row
            .relationship_metadata
            .clone()
            .unwrap_or_else(|| serde_json::json!({}));
        meta["removed_at"] = serde_json::json!(now.to_rfc3339());
        meta["removal_reason"] = serde_json::json!(reason.to_string());
        if let Some(n) = &notes {
            meta["departure_notes"] = serde_json::json!(n);
        }

        let mut active: atlas_record_relationship::ActiveModel = row.into();
        active.relationship_metadata = Set(Some(meta));
        let updated = active.update(db).await?;

        tracing::info!(%tenant_id, %user_id, %lease_id, occupant_id = %entry_id,
            %reason, "HouseholdService: occupant departed (soft-delete)");

        try_parse_former(lease_id, updated)
            .ok_or_else(|| anyhow!("failed to parse departed occupant {entry_id}"))
    }

    /// Landlord view — all active occupants for a given unit (across its active leases).
    pub async fn list_unit_occupants(
        db: &DatabaseConnection,
        tenant_id: Uuid,
        unit_asset_id: Uuid,
    ) -> Result<Vec<(Uuid, Vec<ActiveOccupant>, Vec<VehicleRecord>)>> {
        let leases = atlas_contract::Entity::find()
            .filter(atlas_contract::Column::TenantId.eq(tenant_id))
            .filter(atlas_contract::Column::AssetId.eq(unit_asset_id))
            .filter(atlas_contract::Column::Status.eq("active"))
            .all(db)
            .await?;

        let mut results = Vec::new();
        for lease in leases {
            let occupants = Self::list_active_occupants(db, tenant_id, lease.id).await?;
            let vehicles = Self::list_vehicles(db, tenant_id, lease.id).await?;
            results.push((lease.id, occupants, vehicles));
        }
        Ok(results)
    }

    // ── Private helpers ───────────────────────────────────────────────────────

    async fn assert_lease_access(
        db: &DatabaseConnection,
        tenant_id: Uuid,
        user_id: Uuid,
        lease_id: Uuid,
    ) -> Result<atlas_contract::Model> {
        atlas_contract::Entity::find_by_id(lease_id)
            .filter(atlas_contract::Column::TenantId.eq(tenant_id))
            .filter(atlas_contract::Column::CounterpartyUserId.eq(user_id))
            .one(db)
            .await?
            .ok_or_else(|| anyhow!("lease {lease_id} not found or access denied"))
    }

    async fn fetch_occupant_rows(
        db: &DatabaseConnection,
        tenant_id: Uuid,
        lease_id: Uuid,
    ) -> Result<Vec<atlas_record_relationship::Model>> {
        Ok(atlas_record_relationship::Entity::find()
            .filter(atlas_record_relationship::Column::TenantId.eq(tenant_id))
            .filter(atlas_record_relationship::Column::SourceEntityType.eq(SOURCE_CONTRACT))
            .filter(atlas_record_relationship::Column::SourceEntityId.eq(lease_id))
            .filter(atlas_record_relationship::Column::RelationshipType.eq(REL_HOUSEHOLD))
            .order_by_asc(atlas_record_relationship::Column::CreatedAt)
            .all(db)
            .await?)
    }

    async fn fetch_one_occupant_row(
        db: &DatabaseConnection,
        tenant_id: Uuid,
        lease_id: Uuid,
        entry_id: Uuid,
    ) -> Result<atlas_record_relationship::Model> {
        atlas_record_relationship::Entity::find()
            .filter(atlas_record_relationship::Column::TenantId.eq(tenant_id))
            .filter(atlas_record_relationship::Column::SourceEntityId.eq(lease_id))
            .filter(atlas_record_relationship::Column::TargetEntityId.eq(entry_id))
            .filter(atlas_record_relationship::Column::RelationshipType.eq(REL_HOUSEHOLD))
            .one(db)
            .await?
            .ok_or_else(|| anyhow!("occupant {entry_id} not found on lease {lease_id}"))
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// Private parsers — JSONB → typed structs
// ═══════════════════════════════════════════════════════════════════════════════

fn parse_vehicle(lease_id: Uuid, r: atlas_record_relationship::Model) -> Option<VehicleRecord> {
    let m = r.relationship_metadata.as_ref()?;
    Some(VehicleRecord {
        id: r.target_entity_id,
        lease_id,
        make: m["make"].as_str()?.to_string(),
        model: m["model"].as_str()?.to_string(),
        year: m["year"].as_i64()? as i32,
        color: m["color"].as_str()?.to_string(),
        license_plate: m["license_plate"].as_str()?.to_string(),
        state: m["state"].as_str()?.to_string(),
        country: m["country"].as_str().unwrap_or("US").to_string(),
        parking_spot: m["parking_spot"].as_str().map(|s| s.to_string()),
        registration_expiry: m["registration_expiry"]
            .as_str()
            .and_then(|s| s.parse().ok()),
        registered_at: r.created_at,
        updated_at: m["updated_at"]
            .as_str()
            .and_then(|s| s.parse::<chrono::DateTime<Utc>>().ok()),
    })
}

/// Returns `Some(ActiveOccupant)` only if `removed_at` is absent from metadata.
fn try_parse_active(lease_id: Uuid, r: atlas_record_relationship::Model) -> Option<ActiveOccupant> {
    let m = r.relationship_metadata.as_ref()?;
    // If removed_at is present, this is a FormerOccupant — skip
    if m.get("removed_at").and_then(|v| v.as_str()).is_some() {
        return None;
    }
    let is_minor = m["is_minor"].as_bool().unwrap_or(false);
    let profile_id = if r.target_entity_type == "profile" || r.target_entity_type == TARGET_ADULT {
        m["profile_id"].as_str().and_then(|s| s.parse().ok())
    } else {
        None
    };
    Some(ActiveOccupant {
        id: r.target_entity_id,
        lease_id,
        full_name: m["full_name"].as_str()?.to_string(),
        relationship: m["relationship"].as_str()?.to_string(),
        is_minor,
        date_of_birth: m["date_of_birth"].as_str().and_then(|s| s.parse().ok()),
        profile_id,
        id_document_type: m["id_document_type"].as_str().map(|s| s.to_string()),
        registered_at: r.created_at,
    })
}

/// Returns `Some(FormerOccupant)` only if `removed_at` is present in metadata.
fn try_parse_former(lease_id: Uuid, r: atlas_record_relationship::Model) -> Option<FormerOccupant> {
    let m = r.relationship_metadata.as_ref()?;
    let removed_at_str = m["removed_at"].as_str()?;
    let removed_at: chrono::DateTime<Utc> = removed_at_str.parse().ok()?;
    let reason_str = m["removal_reason"].as_str().unwrap_or("other");
    let removal_reason = match reason_str {
        "moved_out" => DepartureReason::MovedOut,
        "relationship_ended" => DepartureReason::RelationshipEnded,
        "lease_violation" => DepartureReason::LeaseViolation,
        "deceased" => DepartureReason::Deceased,
        _ => DepartureReason::Other,
    };
    let is_minor = m["is_minor"].as_bool().unwrap_or(false);
    Some(FormerOccupant {
        id: r.target_entity_id,
        lease_id,
        full_name: m["full_name"].as_str()?.to_string(),
        relationship: m["relationship"].as_str()?.to_string(),
        is_minor,
        date_of_birth: m["date_of_birth"].as_str().and_then(|s| s.parse().ok()),
        profile_id: m["profile_id"].as_str().and_then(|s| s.parse().ok()),
        id_document_type: m["id_document_type"].as_str().map(|s| s.to_string()),
        registered_at: r.created_at,
        removed_at,
        removal_reason,
        departure_notes: m["departure_notes"].as_str().map(|s| s.to_string()),
    })
}
