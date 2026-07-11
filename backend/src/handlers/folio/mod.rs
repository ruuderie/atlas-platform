pub mod appliances; // G-10 lifecycle — Appliance CRUD + lifecycle alerts
pub mod applications;
pub mod assets;
pub mod attribution; // Phase 6 — Multi-channel attribution touchpoints (G20)
pub mod billing;
pub mod building_systems; // G-10 lifecycle — Building system CRUD + canonical lifecycle alert query
pub mod campaigns; // Phase 6 — Multi-channel campaign management (G19)
pub mod catalog; // Phase 6 — Product catalog, pricebook & availability (G26)
pub mod commission_plans; // Phase 6 — Commission plan application & splits (G25)
pub mod comms; // G-07 — Unified communications: rooms, messages, platform_support
pub mod events; // Phase 6 — Event management, ticketing & check-in (G21)
pub mod flags; // Feature flag resolution for Folio clients
pub mod household; // G-22 — Vehicle & occupant registration (lease-scoped, type-safe)
pub mod invite_codes; // Invite code CRUD + public resolve endpoint for /join/:code flow
pub mod leads; // Phase 6 — PM-tier lead lifecycle: qualify, convert, disqualify (G31)
pub mod leases;
pub mod maintenance;
pub mod marketplace; // G-34 — Vendor marketplace: discovery, endorsements, listing mgmt
pub mod me; // GET /api/folio/me — multi-role identity & FolioRole resolution
pub mod notifications; // G-07 ext — Notification inbox, channel prefs, tenant channel settings
pub mod onboarding_submit; // POST /api/folio/onboarding/submit — atomic first-run wizard save
pub mod opportunities; // Phase 6 — Sales pipeline & deal management (G15)
pub mod owner; // G-22 — Beneficial owner read-only portal + PMC link management
pub mod pm; // G-33 — PMC routes: clients, client detail, analytics, app config
pub mod portfolio;
pub mod programs; // G-36 atlas_programs NetworkInvite API
pub mod property_value; // G-10 ext — POST/GET /api/folio/properties/:id/value[-history]
pub mod provision; // POST /api/folio/provision/invite — unified all-persona provisioning
pub mod quotes; // Phase 6 — Pre-purchase pricing proposals (G24)
pub mod relationships; // Phase 6 — Universal M:M junction table (G22)
pub mod reporting; // Cross-table — Tenant reports + landlord/vendor analytics
pub mod reservations; // Phase 6 — STR booking lifecycle (G23 atlas_reservations)
pub mod review_invite; // G-27 ext — vendor review invites + public review submit + pub vendor profile
pub mod scorecards;
pub mod service_request;
pub mod str;
pub mod str_guest; // G-22 — STR guest/vehicle registration + special requests per booking
pub mod users; // GET /api/folio/users/:id — counterparty user identity lookup
pub mod vault;
pub mod vendor; // G-32 — Vendor-role routes: work orders + invoices
pub mod vendors;
pub mod violations; // G-13 — Compliance violation filing + cure status transitions
pub mod wholesale; // G-35 ext — POST /api/folio/service-requests + vendor notify
