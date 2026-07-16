pub mod aggregates; // G-33/PMC — Per-client aggregate metrics (single CTE SQL query)
pub mod appliance; // G-10 lifecycle — Appliance tracking (ApplianceMetadata + lifecycle alerts)
pub mod applications;
pub mod asset;
pub mod attribution; // Phase 6 — Multi-channel attribution touchpoints (G20)
pub mod attribution_hooks; // Attribution conversion helpers for payment flows
pub mod building_system; // G-10 lifecycle — Building system tracking (elevator, roof, HVAC, fire suppression)
pub mod campaign; // Phase 6 — Multi-channel campaign management (G19)
pub mod campaign_dm; // G-19 Direct Mail campaign helpers (mail drops + offer codes)
pub mod catalog; // Phase 6 — Product catalog, pricebook & availability (G26)
pub mod commission; // Phase 6 — Commission plan application & splits (G25)
pub mod condominio;
pub mod direct_mail; // G-19 DirectMailProvider trait + provider stubs
pub mod event; // Phase 6 — Event management, ticketing & check-in (G21)
pub mod fair_housing;
pub mod household; // G-22 — Lease-scoped tenant declarations: vehicles + occupants (type-safe)
pub mod lead; // Phase 6 — Lead lifecycle: qualify, convert, disqualify (G31)
pub mod lease;
pub mod ledger; // Phase 3 — PM ledger service (atlas_ledger_entries wrapper)
pub mod maintenance;
/// Folio — Property Management Service Layer
///
/// All PM services are thin, domain-specific wrappers over the platform generics
/// (G01–G18 + G27). They add PM-specific business rules, validation, and vocabulary
/// on top of the generic platform models.
///
/// # Zero Net-New Tables
/// Every PM service reads/writes to platform generic tables (atlas_assets,
/// atlas_contracts, atlas_cases, etc.). No new schema is introduced here.
///
/// # Module Structure
/// - `market/`             — Market configuration trait system (MarketConfig + implementors)
///   - `market_config`     — Root trait + sub-traits + MarketRegistry
///   - `brazil`            — BrazilMarket (Lei do Inquilinato, Serasa, BRL, IRRF)
///   - `miami`             — MiamiDadeMarket (FHA, TDT 7%, STR Ordinance 2023-89)
///   - `usvi`              — UsViMarket (FHA, USVI Hotel Room Tax 12.5%)
/// - `portfolio`           — NAV aggregation (USD/BRL/BTC), asset_code generation
/// - `asset`               — Property/unit hierarchy, folio number, geo_point
/// - `lease`               — Condomínio split, guarantee types, auto-renewal
/// - `maintenance`         — Emergency bypass, vendor dispatch, WebSocket threading
/// - `vendor`              — License verification, emergency availability, contractor scoring
/// - `wholesale`           — MAO formula, lead pipeline, Kanban stage transitions
/// - `condominio`          — Lei do Inquilinato expense classification (delegates to market)
/// - `fair_housing`        — FHA compliance filter (delegates to market.anti_discrimination_law)
/// - `applications`        — Renter screening (Serasa/Checkr/TransUnion)
/// - `str_compliance`      — Miami STR zoning, permit expiry, OTA revenue sync
/// - `tax`                 — TDT calculation, monthly reconciliation
/// - `vault`               — PM document taxonomy, presigned R2 URLs
/// - `scorecard_provisioner` — Seeds the 4 canonical PM G-27 templates
/// - `payment_rail`          — `PaymentRailAdapter` trait + `resolve_adapter()` registry
/// - `rails/`                — Per-rail implementations (stripe_connect, infinitepay,
///                             bitcoin_onchain, lightning, kelviq)
/// - `ledger`                — PM ledger service: create/update `atlas_ledger_entries`
pub mod market; // Market configuration system — MUST be first
pub mod opportunity; // Phase 6 — Sales pipeline & deal management (G15)
pub mod owner; // G-22 — Beneficial owner read-only portfolio visibility
pub mod payment_rail; // Phase 3 — PaymentRailAdapter trait + adapter registry
pub mod portfolio;
pub mod quote; // Phase 6 — Pre-purchase pricing proposals (G24)
pub mod rails; // Phase 3 — per-rail adapter implementations
pub mod record_relationship; // Phase 6 — Universal M:M junction table (G22)
pub mod reporting; // Cross-table aggregation: tenant reports, landlord KPIs, vendor analytics
pub mod reservation; // Phase 6 — STR Reservation lifecycle (G23 atlas_reservations)
pub mod scorecard_provisioner;
pub mod str_compliance;
pub mod str_guest;
pub mod tax;
pub mod vault;
pub mod vendor;
pub mod violation; // G-13 — Compliance violation lifecycle (typed category + cure status transitions)
pub mod wholesale; // G-15 wholesale MAO (thin; prefer deal_ops for Deal Ops)
pub mod deal_ops; // G-15 Deal Ops — Wholesaling + Creative Finance
