pub mod portfolio;
pub mod assets;
pub mod leases;
pub mod maintenance;
pub mod vendors;
pub mod wholesale;
pub mod billing;
pub mod scorecards;
pub mod str;
pub mod vault;
pub mod applications;
pub mod reservations; // Phase 6 — STR booking lifecycle (G23 atlas_reservations)
pub mod catalog;      // Phase 6 — Product catalog, pricebook & availability (G26)
pub mod campaigns;    // Phase 6 — Multi-channel campaign management (G19)
pub mod attribution;  // Phase 6 — Multi-channel attribution touchpoints (G20)
pub mod events;       // Phase 6 — Event management, ticketing & check-in (G21)
pub mod relationships; // Phase 6 — Universal M:M junction table (G22)
pub mod quotes;           // Phase 6 — Pre-purchase pricing proposals (G24)
pub mod opportunities;    // Phase 6 — Sales pipeline & deal management (G15)
pub mod commission_plans; // Phase 6 — Commission plan application & splits (G25)
pub mod leads;            // Phase 6 — PM-tier lead lifecycle: qualify, convert, disqualify (G31)
pub mod geo;              // G01  — PostGIS spatial query routes (radius, nearest, containment)
pub mod me;               // GET /api/folio/me — multi-role identity & FolioRole resolution
pub mod vendor;           // G-32 — Vendor-role routes: work orders + invoices
pub mod pm;               // G-33 — PMC routes: clients, client detail, analytics, app config
pub mod marketplace;      // G-34 — Vendor marketplace: discovery, endorsements, listing mgmt
pub mod appliances;       // G-10 lifecycle — Appliance CRUD + lifecycle alerts
pub mod building_systems; // G-10 lifecycle — Building system CRUD + canonical lifecycle alert query
pub mod household;        // G-22 — Vehicle & occupant registration (lease-scoped, type-safe)
pub mod violations;       // G-13 — Compliance violation filing + cure status transitions
pub mod reporting;        // Cross-table — Tenant reports + landlord/vendor analytics
pub mod owner;            // G-22 — Beneficial owner read-only portal + PMC link management
pub mod str_guest;        // G-22 — STR guest/vehicle registration + special requests per booking
