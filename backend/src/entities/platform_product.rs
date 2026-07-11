//! Sea-ORM entity for `platform_products`.
//!
//! One row per Atlas Platform product (Folio, Anchor, NetworkInstance, Meridian).
//! Seeded by migration; managed via platform-admin.

use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "platform_products")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub name: String,
    pub slug: String,
    pub tagline: Option<String>,
    /// "active" | "beta" | "deprecated"
    pub status: String,
    /// FK to app_pages — CMS-managed marketing landing page
    pub marketing_page_cms_id: Option<Uuid>,
    /// Cloudflare Pages deploy hook URL (POST → triggers deploy)
    pub deploy_hook_url: Option<String>,

    // ── Launch engine fields (m20260903) ─────────────────────────────────────
    /// "draft" | "pre_launch" | "waitlist" | "active" | "beta" | "invite_only" | "deprecated"
    pub launch_mode: String,
    pub pre_order_enabled: bool,
    pub pre_order_price_cents: Option<i32>,
    pub pre_order_currency: String,
    pub stripe_price_id: Option<String>,
    /// Founding/early-bird cap (null = unlimited)
    pub pre_order_cap: Option<i32>,
    pub pre_order_sold: i32,
    /// Cached waitlist aggregate (denormalized)
    pub waitlist_count: i32,
    pub sentinel_tenant_id: Option<Uuid>,

    // ── Domain routing (m20260905) ────────────────────────────────────────────
    /// Apex marketing domain, e.g. "folio.app"
    pub apex_domain: Option<String>,
    pub apex_domain_verified: bool,

    // ── App binding (m20260917) ────────────────────────────────────────────────────
    /// Which Atlas app binary owns this product's landing page rendering.
    /// Raw string form of `AppId` — parse via `AppId::try_from(&self.app_slug)`
    /// in handlers. Same pattern as `launch_mode`. DB CHECK constraint enforces
    /// the value is a valid `AppId` discriminant.
    pub app_slug: String,

    pub created_at: DateTimeWithTimeZone,
    pub updated_at: DateTimeWithTimeZone,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
