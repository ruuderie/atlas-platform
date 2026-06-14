# Atlas Platform — Maps as a Core Capability
## Architecture Specification · June 2026

---

## 1. What Already Exists

The geo foundation is already in the codebase — maps are a **missing presentation layer**, not a missing backend.

### `listing` entity — `(latitude, longitude)` columns
```rust
// backend/src/entities/listing.rs — lines 28–29
pub latitude: Option<f64>,
pub longitude: Option<f64>,
```
Every rental listing already stores a coordinate pair. This powers the LTR/STR directory map immediately without any schema change.

### `geo_service_area` entity — G01 (PostGIS polygon + point)
```rust
// backend/src/entities/geo_service_area.rs
// GENERIC-01: GeoServiceArea
pub geom: Option<String>,   // PostGIS GEOMETRY(MultiPolygon, 4326) — WKT
pub point: Option<String>,  // PostGIS GEOGRAPHY(Point, 4326) — WKT
pub zip_codes: Option<serde_json::Value>,
pub owner_entity_type: String,  // polymorphic — attaches to any entity
pub owner_entity_id: Uuid,
```
This is the vendor service area polygon, broker territory, and neighborhood boundary primitive. Already defined, just not surfaced in the UI.

### `atlas_events` (G21) — `venue_geo_point GEOGRAPHY(Point, 4326)`
From the gap analysis — already designed with a PostGIS point for venue location. Open houses and events need map display natively.

---

## 2. The 5 Map Surfaces

| Surface | Data Source | What the map shows | Who benefits |
|---|---|---|---|
| **LTR/STR Directory** | `listing.latitude/longitude` | Listing price pins, cluster badges, commute radius | Renter |
| **Listing detail** | `listing.*` + commute geocoding | Commute to work, transit stops, POIs | Renter |
| **Renter shortlist** | Saved `listing` records | All saved apartments on one map vs. workplace | Renter |
| **Vendor job route** | `geo_service_area.point` + job records | Today's jobs optimized route with driving directions | Vendor |
| **Folio portfolio map** | `listing.latitude/longitude` by status | All owned properties, color-coded by occupancy/health | Landlord / PM |
| **Broker territory** | `geo_service_area.geom` | Agent coverage polygons, listing concentration | Broker |

---

## 3. Architecture — How Maps Fit

```
┌─────────────────────────────────────────────────────────┐
│                   ATLAS MAP SERVICE                     │
│                (apps/shared-ui/src/maps/)               │
│                                                         │
│  MapConfig ──── tile provider (Mapbox or OSM/Carto)     │
│  GeoLayer  ──── pins | polygons | heatmaps | routes     │
│  GeoBounds ──── bounding box for API range queries      │
│  CommuteLayer ─ workplace pin + isochrone overlay       │
└──────────────────────────┬──────────────────────────────┘
                           │ consumes
          ┌────────────────┼────────────────┐
          ▼                ▼                ▼
  /api/listings      /api/geo/service  /api/jobs
  ?bbox=...lat/lng   _areas?owner=     ?date=today
  &map=true          vendor_id         &status=open
  (returns flat      (polygon WKT      (returns lat/lng
  GeoJSON)            array)            per job)
```

### API additions required

**1. Listing geo endpoint** — `GET /api/listings?bbox={sw_lat},{sw_lng},{ne_lat},{ne_lng}&format=geojson`

Returns GeoJSON `FeatureCollection`. Map viewport drives the bounding box. Already all data exists — new query shape only.

```rust
// New handler: handlers/listing_map.rs
pub async fn list_listings_in_bbox(
    db: &DatabaseConnection,
    tenant_id: Uuid,
    sw: (f64, f64),
    ne: (f64, f64),
    filters: ListingMapFilter,  // type, beds, price_range, available_from
) -> Result<GeoJson<FeatureCollection>>
```

**2. Geo service area endpoint** — Already has entity, needs handler:

```rust
// Expose existing geo_service_area entity
GET /api/geo/service-areas?owner_entity_type={type}&owner_entity_id={id}
// Returns polygon WKT → frontend converts to Leaflet L.polygon or Mapbox geojson layer
```

**3. Commute proxy** — Thin proxy to Mapbox Directions API or OSRM (open source):

```rust
// handlers/commute.rs
GET /api/commute/isochrone?origin_lat={}&origin_lng={}&minutes={15|30|45}&mode={drive|transit|walk}
// Returns a GeoJSON polygon (isochrone) drawn on the map
```

---

## 4. Frontend Component — `AtlasMap` (shared-ui)

A single reusable map component used across all apps:

```rust
// apps/shared-ui/src/maps/atlas_map.rs (Leptos component)
#[component]
pub fn AtlasMap(
    // Data
    listings: Vec<ListingGeoPin>,       // for directory/shortlist
    jobs: Option<Vec<JobGeoPin>>,       // for vendor route
    properties: Option<Vec<PortfolioPin>>, // for folio portfolio
    polygons: Option<Vec<GeoPolygon>>,  // for territory/service areas
    // Behavior
    on_pin_click: Callback<Uuid>,
    active_id: Option<Uuid>,
    // Overlays
    show_commute_tool: bool,
    show_layer_controls: bool,
    // Config
    tile_provider: TileProvider,        // Mapbox | CartoDB | OSM
    initial_bounds: GeoBounds,
) -> impl IntoView
```

### Tile provider strategy

| Env | Provider | Cost | When |
|---|---|---|---|
| **Prototype / dev** | CartoDB Light (free) | $0 | Now — all prototypes |
| **Production** | Mapbox GL JS | Pay-per-load ~$0.50/1k tiles | At launch |
| **Enterprise tenant** | Mapbox (tenant key) | Tenant-billed | For white-label network instances |

Mapbox is the clear production choice — it supports:
- Custom vector tile styling (match platform brand per network instance)
- Isochrone API (commute time polygons)
- Directions API (vendor route optimization)
- Geocoding API (commute address → lat/lng)

---

## 5. Data Model — Geo Additions Needed

Only one schema addition is needed to unlock all 5 map surfaces:

```sql
-- Add to listing table (or atlas_catalog_entry for STR rates)
-- These already exist! latitude + longitude columns are in listing.rs
-- No migration needed for directory map.

-- For commute workplace storage (renter profile):
ALTER TABLE profile ADD COLUMN IF NOT EXISTS 
  workplace_lat FLOAT,
  workplace_lng FLOAT,
  workplace_label VARCHAR(255);
-- This lets saved commute preference persist across sessions

-- For vendor service area (already exists in geo_service_areas):
-- No change needed — expose via API handler
```

---

## 6. Platform Generics — Where Maps Touch the Generic Layer

| Generic | Map connection |
|---|---|
| **G01 `geo_service_areas`** | Vendor service area polygon, broker territory polygon, neighborhood boundary |
| **G10 `atlas_assets`** | `attributes` JSONB already stores property details; `lat/lng` should be promoted to typed columns to match `listing` |
| **G21 `atlas_events`** | `venue_geo_point GEOGRAPHY(Point, 4326)` — open house map pin |
| **G23 `atlas_reservations`** | STR booking — pin on guest's confirmation map showing property location |

> **Recommendation:** Promote `latitude` / `longitude` from `listing` to `atlas_assets` as typed `FLOAT` columns (not JSONB). This gives every asset in the platform a geographic coordinate — enabling portfolio maps, event venue maps, and service area overlap detection uniformly.

---

## 7. Build Order

| Phase | Work | Unblocked by |
|---|---|---|
| **P1 — Now (prototypes done)** | Leaflet + CartoDB in stitch pages | Nothing — complete |
| **P2 — App integration** | `AtlasMap` shared-ui Leptos component | Leptos component architecture |
| **P3 — Backend geo API** | `list_listings_in_bbox` handler + GeoJSON response | No schema change needed |
| **P4 — Commute overlay** | Mapbox Isochrone API proxy + workplace storage in `profile` | Minor migration |
| **P5 — Vendor route** | Directions API + job sequence optimization endpoint | `atlas_service_areas` + job entities |
| **P6 — Portfolio map** | Folio-scoped asset map with status color coding | G10 lat/lng promotion |

---

## 8. Competitive Advantage

No PM platform has map functionality that crosses consumer and operator views in one system:

| Platform | Renter map | Vendor route map | Portfolio map | Broker territory |
|---|---|---|---|---|
| **Zillow / Apartments.com** | ✓ | ✗ | ✗ | ✗ |
| **AppFolio / Buildium** | ✗ | ✗ | Partial | ✗ |
| **kvCORE** | Partial | ✗ | ✗ | Partial |
| **Atlas Platform** | ✓ | ✓ | ✓ | ✓ |

The vendor job route map specifically has **no direct competitor** in the PM SaaS space. Field service companies (ServiceTitan, Jobber) have it — but they're not property management platforms. This is a genuine moat.
