pub mod session_unit_tests;
// G-28 atlas_note entity helper tests
pub mod atlas_note_unit_tests;
// G-29 atlas_activity entity helper tests
pub mod atlas_activity_unit_tests;
// G-27 ScorecardService helpers + G-31 Lead entity helpers
pub mod scorecard_lead_unit_tests;
// G-27 Phase 5: atlas-compute-sdk pure-math unit tests
pub mod g27_unit_tests;
// Type-system: TryFrom/Display roundtrips + entity typed helper methods
pub mod type_system_unit_tests;
// Phase 3–7: InfinitePay/Kelviq rails, WebSocket registry, geo guards, rate limiter
pub mod pm_phase3_unit_tests;
// G-33 PMC + G-34 Vendor Marketplace: aggregates, invite flow, geo SQL, endorsements, RBAC scoping
pub mod pmc_marketplace_unit_tests;
// G-05 Syndication Event Bus + Folio/NI config matrix + operational config semantics
pub mod syndication_unit_tests;
// AppInstance decomposition: stats response shape, slug dispatch, provision logic, DNS instructions
pub mod app_instance_unit_tests;
// Folio routing type system — FolioRole home_path, predicates, serde wire format
pub mod folio_routing_unit_tests;
// Waitlist handler — WaitlistBody serde contract (role, portfolio_size_label, optional fields)
pub mod waitlist_unit_tests;

