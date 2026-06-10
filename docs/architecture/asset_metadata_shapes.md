# Atlas Asset Lifecycle Metadata Shapes

> **Status:** Living document — update when any AtlasApp defines or modifies a `lifecycle_metadata` shape.  
> **Referenced by:** [`platform_generics_v3.md §8 Risk #8`](./platform_generics_v3.md)  
> **Rule:** JSONB keys are treated as a **stable public API**. Renaming a key requires a versioned backfill migration. Add a `metadata_version` field to every shape.

---

## Overview

`atlas_assets.lifecycle_metadata` is a JSONB column whose shape is **owned by each AtlasApp** and **enforced at the Rust service layer**, not at the DB level. Each AtlasApp must:

1. Define a typed struct (`*Metadata`) that serializes to this JSONB
2. Implement `TryFrom<&AssetModel>` to deserialize and validate
3. Register the shape in this document

The four indexed columns (`scheduled_service_date`, `expiry_date`, `condition`) are **platform-owned** and must be written by every app that sets `lifecycle_metadata`. They are the only `atlas_assets` fields queried platform-wide.

---

## Shape Registry

### `asset_type = "appliance"` — Folio (Property Management)

**Struct:** `folio::services::assets::ApplianceMetadata`  
**Metadata version:** `1`

```json
{
  "metadata_version": 1,
  "appliance_type": "water_heater",
  "make": "Rheem",
  "model": "ProTerra 50gal",
  "year_manufactured": 2021,
  "fuel_type": "electric",
  "serial_number": "RH-2021-4892",
  "installer_sp_id": "uuid | null",
  "warranty_provider": "Rheem Corp | null",
  "warranty_contact": "1-800-555-0100 | null",
  "purchase_price_cents": 120000,
  "replacement_cost_cents": 135000,
  "service_interval_days": 365
}
```

**Indexed platform columns written by this app:**

| Field | Source |
|---|---|
| `scheduled_service_date` | `install_date + service_interval_days` (computed on create/update) |
| `expiry_date` | Manufacturer warranty expiry |
| `condition` | `excellent \| good \| fair \| poor \| retired` |

**Valid `appliance_type` values:**
`refrigerator`, `washer`, `dryer`, `washer_dryer_combo`, `water_heater`, `boiler`, `hvac_unit`, `dishwasher`, `oven_range`, `garbage_disposal`, `garbage_disposal`, `pool_pump`, `garage_door_opener`, `air_handler`, `water_softener`

**Validation rules enforced at service layer:**
- `make` and `model` are required
- `service_interval_days` must be > 0 if set
- `scheduled_service_date` must be set when `service_interval_days` is set

**Parent:** `parent_asset_id` → unit asset (`asset_type = "real_estate_unit"`)

---

### `asset_type = "building_system"` — Folio (Property Management)

**Struct:** `services::pm::building_system::BuildingSystemMetadata`  
**Metadata version:** `1`

```json
{
  "metadata_version": 1,
  "system_type": "elevator",
  "make": "Otis",
  "model": "Gen2 MRL",
  "serial_number": "OT-2019-8821-A",
  "year_installed": 2019,
  "certificate_type": "state_elevator_cert",
  "certificate_issuer": "Miami-Dade County DBPR",
  "certificate_number": "EL-2024-00412",
  "contractor_sp_id": "uuid | null",
  "service_contract_expiry": "2027-03-31",
  "replacement_cost_cents": 8500000,
  "useful_life_years": 25,
  "is_building_wide": true,
  "location_notes": "Main lobby — south shaft"
}
```

**Indexed platform columns written by this app:**

| Field | Source |
|---|---|
| `scheduled_service_date` | Next inspection / maintenance due date |
| `expiry_date` | Certificate / warranty / service contract expiry |
| `condition` | `excellent | good | fair | poor | retired` |

**Valid `system_type` values:**
`elevator`, `escalator`, `fire_suppression`, `fire_alarm`, `emergency_lighting`,
`common_area_hvac`, `boiler`, `cooling_tower`, `chiller`,
`generator`, `electrical_panel`, `transformer_vault`,
`roof_drain_system`, `sewer_lift`, `backflow_preventer`,
`roof`, `facade`, `parking_structure`,
`pool`, `spa`, `gym_equipment`,
`security_system`, `access_control`, `intercom`, `other`

**Valid `certificate_type` values:**
`state_elevator_cert`, `fire_inspection_permit`, `boiler_pressure_vessel_cert`,
`pool_health_permit`, `building_permit`, `epa_608_hvac`, `none`

**Validation rules enforced at service layer:**
- `name` is required
- `useful_life_years` must be > 0 if set
- For systems requiring certs, link to G-16 `atlas_regulatory_registrations`
  via G-22 `atlas_record_relationships` with `relationship_type = "regulatory_cert"`

**Parent:** `parent_asset_id` → property asset (`asset_type = "real_estate_property"`)

---

### `asset_type = "vehicle"` — FleetOps (Fleet Management)

**Struct:** `fleetops::services::assets::VehicleMetadata`  
**Metadata version:** `1`

```json
{
  "metadata_version": 1,
  "vehicle_class": "commercial",
  "make": "Ford",
  "model": "F-650",
  "year": 2022,
  "vin": "1FTWF2CM4AKA12345",
  "odometer_km": 48200,
  "fuel_type": "diesel",
  "dot_number": "US-DOT 3456789",
  "gvwr_kg": 11794,
  "assigned_driver_id": "uuid | null",
  "insurance_policy_id": "uuid | null"
}
```

**Indexed platform columns written by this app:**

| Field | Source |
|---|---|
| `scheduled_service_date` | Next DOT inspection / scheduled PM |
| `expiry_date` | Registration / road-worthiness cert expiry |
| `condition` | `excellent \| good \| fair \| poor \| retired` |

**Validation rules:**
- `vin` is required; must be exactly 17 characters, alphanumeric excluding I/O/Q
- `dot_number` is required when `vehicle_class = "commercial"`
- `gvwr_kg` is required when `vehicle_class = "heavy_duty"` or `"commercial"`

---

### `asset_type = "medical_device"` — MedTrack (Healthcare)

**Struct:** `medtrack::services::assets::MedicalDeviceMetadata`  
**Metadata version:** `1`

```json
{
  "metadata_version": 1,
  "device_name": "MAGNETOM Vida",
  "manufacturer": "Siemens Healthineers",
  "model_number": "VB20A",
  "fda_class": "class_ii",
  "fda_510k_number": "K221234",
  "udi": "00888610008457",
  "sterilization_method": "autoclave",
  "last_sterilized_at": "2026-05-15",
  "biomedical_engineer_id": "uuid | null",
  "recall_status": "none"
}
```

**Indexed platform columns written by this app:**

| Field | Source |
|---|---|
| `scheduled_service_date` | Next calibration / preventive maintenance due |
| `expiry_date` | FDA cert expiry / biomedical certification expiry |
| `condition` | `excellent \| good \| fair \| poor \| decommissioned` |

**Validation rules:**
- `fda_class` is required
- `fda_510k_number` is required when `fda_class = "class_iii"`
- `udi` is required for Class II and III devices (FDA UDI Rule)
- `recall_status` must be one of: `none`, `class_i`, `class_ii`, `class_iii`, `voluntary`

---

### `asset_type = "it_device"` — ITAsset (IT/SaaS)

**Struct:** `itasset::services::assets::ItDeviceMetadata`  
**Metadata version:** `1`

```json
{
  "metadata_version": 1,
  "device_category": "laptop",
  "make": "Apple",
  "model": "MacBook Pro 14",
  "year": 2023,
  "serial_number": "C02XG2JHQ05P",
  "mac_address": "a4:83:e7:ab:12:cd",
  "os": "macOS 14.5",
  "os_eol_date": "2028-09-01",
  "last_patch_date": "2026-06-01",
  "assigned_user_id": "uuid | null",
  "mdm_enrolled": true,
  "disk_encryption": true
}
```

**Indexed platform columns written by this app:**

| Field | Source |
|---|---|
| `scheduled_service_date` | Next patch cycle / MDM compliance check |
| `expiry_date` | Hardware warranty expiry (or OS EOL, whichever is sooner) |
| `condition` | `active \| decommissioned \| lost \| stolen` |

**Validation rules:**
- `serial_number` is required
- `os_eol_date` is required (used for replacement planning alerts)
- `disk_encryption` must be `true` for devices with `device_category = "laptop"` or `"workstation"` (security policy)

---

### `asset_type = "meter"` — EnergyOps (Utilities)

**Struct:** `energyops::services::assets::MeterMetadata`  
**Metadata version:** `1`

```json
{
  "metadata_version": 1,
  "meter_type": "electric",
  "make": "Landis+Gyr",
  "model": "E360",
  "serial_number": "LG-E360-88291",
  "rated_kw": 100,
  "current_reading_kwh": 48291.4,
  "last_reading_at": "2026-06-01",
  "installation_address_asset_id": "uuid",
  "meter_point_id": "MPAN-1234567890"
}
```

**Indexed platform columns written by this app:**

| Field | Source |
|---|---|
| `scheduled_service_date` | Next meter calibration / accuracy test |
| `expiry_date` | Manufacturer warranty / meter certification expiry |
| `condition` | `active \| fault \| decommissioned \| replaced` |

---

## Adding a New Shape

When a new AtlasApp adds a new `asset_type`, the process is:

1. **Define the Rust struct** in `{app}::services::assets::{TypeName}Metadata`
2. **Implement `TryFrom<&AssetModel>`** — must return `AppError::TypeMismatch` if `asset_type` doesn't match
3. **Set the three platform columns** (`scheduled_service_date`, `expiry_date`, `condition`) in every create/update path
4. **Register the shape here** with:
   - JSON example
   - Which platform columns are written and from what source
   - All validation rules enforced at the service layer
5. **Set `metadata_version: 1`** in the initial struct

---

## Renaming a JSONB Key (Backfill Procedure)

JSONB keys are stable public API. Renaming requires:

```sql
-- 1. Add new key, copy values
UPDATE atlas_assets
SET lifecycle_metadata = lifecycle_metadata
    || jsonb_build_object('new_key_name', lifecycle_metadata->>'old_key_name')
WHERE asset_type = '{type}'
  AND lifecycle_metadata ? 'old_key_name';

-- 2. Deploy Rust struct with both keys (Optional<String>) during transition
-- 3. Remove old key after all readers are deployed
UPDATE atlas_assets
SET lifecycle_metadata = lifecycle_metadata - 'old_key_name'
WHERE asset_type = '{type}'
  AND lifecycle_metadata ? 'old_key_name';

-- 4. Bump metadata_version in the Rust struct
-- 5. Update this document
```

---

## Optimization Triggers

See [`platform_generics_v3.md §8 Risk #8`](./platform_generics_v3.md) for the full list. Short form:

> If more than 3 apps share a `lifecycle_metadata` field, that field has proven itself generic. Promote it to a typed column on `atlas_assets` or a typed extension table. The JSONB was the prototype; the column is the production form.
