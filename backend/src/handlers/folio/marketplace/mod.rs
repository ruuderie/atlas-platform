//! Folio Vendor Marketplace — handler module.
//!
//! G-34: A trusted, opt-in vendor network across all Folio tenants.
//!
//! # Design principles
//!
//! - **Opt-in only**: vendors (and their landlords) must explicitly set
//!   `is_marketplace_visible = true` on their service provider profile.
//! - **Privacy preserved**: only public fields are surfaced cross-tenant
//!   (name, bio, trade_types, location, rating_avg, endorsement_count).
//!   PII (notes, payment credentials, stripe_id) never leaves the tenant boundary.
//! - **Endorsement via G-22**: landlords endorse vendors using the existing
//!   `atlas_record_relationships` table with `relationship_type = "marketplace_endorsement"`.
//!   Endorsements are counted cross-tenant, building a network trust signal.
//! - **Proximity via G-01 PostGIS**: `marketplace_location` (GEOGRAPHY Point) enables
//!   radius-based discovery — find vendors within X km of the landlord's properties.
//!
//! # Routes
//!
//! GET  /api/folio/marketplace/vendors             — search vendors (area + trade filter)
//! GET  /api/folio/marketplace/vendors/:id         — vendor detail card
//! POST /api/folio/marketplace/vendors/:id/endorse — landlord endorses a vendor
//! DELETE /api/folio/marketplace/vendors/:id/endorse — retract endorsement
//! PATCH /api/folio/marketplace/my-listing         — publish/update own vendor profile

pub mod endorse;
pub mod listing;
pub mod vendors;
