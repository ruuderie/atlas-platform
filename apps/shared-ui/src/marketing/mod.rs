//! Folio Go-to-Market marketing discriminants shared by Folio SSR and platform-admin.
//!
//! Keep in sync with `atlas_backend::types::gtm` counterparts used at API boundaries.

use serde::{Deserialize, Serialize};
use std::fmt;

// ── MarketingSectionBlockType ─────────────────────────────────────────────────

/// CMS section block `type` / `kind` values in `blocks_payload`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MarketingSectionBlockType {
    Stats,
    FeatureGrid,
    Personas,
    Cta,
    BetaStrip,
    Markets,
    PaymentRails,
    StrSection,
    TenantPortal,
    OwnerPortal,
    Footer,
    NavSections,
    PricingIntro,
    TradeCategories,
    RichText,
    Hero,
    /// Explicit escape hatch: replace the polished Leptos stack with BlockRenderer.
    FullPage,
}

impl MarketingSectionBlockType {
    pub const ALL: &'static [Self] = &[
        Self::Stats,
        Self::FeatureGrid,
        Self::Personas,
        Self::Cta,
        Self::BetaStrip,
        Self::Markets,
        Self::PaymentRails,
        Self::StrSection,
        Self::TenantPortal,
        Self::OwnerPortal,
        Self::Footer,
        Self::NavSections,
        Self::PricingIntro,
        Self::TradeCategories,
        Self::RichText,
        Self::Hero,
        Self::FullPage,
    ];

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Stats => "stats",
            Self::FeatureGrid => "feature_grid",
            Self::Personas => "personas",
            Self::Cta => "cta",
            Self::BetaStrip => "beta_strip",
            Self::Markets => "markets",
            Self::PaymentRails => "payment_rails",
            Self::StrSection => "str_section",
            Self::TenantPortal => "tenant_portal",
            Self::OwnerPortal => "owner_portal",
            Self::Footer => "footer",
            Self::NavSections => "nav_sections",
            Self::PricingIntro => "pricing_intro",
            Self::TradeCategories => "trade_categories",
            Self::RichText => "rich_text",
            Self::Hero => "hero",
            Self::FullPage => "full_page",
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Stats => "Stats",
            Self::FeatureGrid => "Feature grid",
            Self::Personas => "Personas",
            Self::Cta => "CTA",
            Self::BetaStrip => "Beta strip",
            Self::Markets => "Markets",
            Self::PaymentRails => "Payment rails",
            Self::StrSection => "STR section",
            Self::TenantPortal => "Tenant portal",
            Self::OwnerPortal => "Owner portal",
            Self::Footer => "Footer",
            Self::NavSections => "Nav sections",
            Self::PricingIntro => "Pricing intro",
            Self::TradeCategories => "Trade categories",
            Self::RichText => "Rich text",
            Self::Hero => "Hero",
            Self::FullPage => "Full page",
        }
    }

    /// Types shown in the Landing Pages "+ Add section" palette.
    pub fn palette() -> &'static [Self] {
        &[
            Self::Stats,
            Self::FeatureGrid,
            Self::Personas,
            Self::Cta,
            Self::BetaStrip,
            Self::Markets,
            Self::PaymentRails,
            Self::StrSection,
            Self::TenantPortal,
            Self::Footer,
            Self::NavSections,
            Self::PricingIntro,
            Self::TradeCategories,
            Self::RichText,
            Self::Hero,
            Self::FullPage,
        ]
    }
}

impl fmt::Display for MarketingSectionBlockType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl TryFrom<&str> for MarketingSectionBlockType {
    type Error = String;
    fn try_from(s: &str) -> Result<Self, Self::Error> {
        Self::ALL
            .iter()
            .copied()
            .find(|v| v.as_str() == s)
            .ok_or_else(|| format!("unknown MarketingSectionBlockType: '{s}'"))
    }
}

// ── FolioMarketingSlug ────────────────────────────────────────────────────────

/// Product / landing `app_id` slugs that own Folio public marketing surfaces.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum FolioMarketingSlug {
    Folio,
    #[serde(rename = "folio-broker")]
    FolioBroker,
    #[serde(rename = "folio-pm")]
    FolioPm,
    #[serde(rename = "folio-vendor")]
    FolioVendor,
    #[serde(rename = "folio-founding")]
    FolioFounding,
    #[serde(rename = "folio-beta")]
    FolioBeta,
    #[serde(rename = "folio-cohost-market")]
    FolioCohostMarket,
}

impl FolioMarketingSlug {
    pub const ALL: &'static [Self] = &[
        Self::Folio,
        Self::FolioBroker,
        Self::FolioPm,
        Self::FolioVendor,
        Self::FolioFounding,
        Self::FolioBeta,
        Self::FolioCohostMarket,
    ];

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Folio => "folio",
            Self::FolioBroker => "folio-broker",
            Self::FolioPm => "folio-pm",
            Self::FolioVendor => "folio-vendor",
            Self::FolioFounding => "folio-founding",
            Self::FolioBeta => "folio-beta",
            Self::FolioCohostMarket => "folio-cohost-market",
        }
    }

    pub fn public_path(self) -> &'static str {
        match self {
            Self::Folio => "/",
            Self::FolioBroker => "/brokers",
            Self::FolioPm => "/property-managers",
            Self::FolioVendor => "/vendors",
            Self::FolioFounding => "/founding",
            Self::FolioBeta => "/beta",
            Self::FolioCohostMarket => "/cohost-market",
        }
    }

    pub fn waitlist_path(self) -> String {
        format!("/api/pub/products/{}/waitlist", self.as_str())
    }

    pub fn pub_product_path(self) -> String {
        format!("/api/pub/products/{}", self.as_str())
    }

    pub fn waitlist_lead_source(self) -> String {
        format!("waitlist:{}", self.as_str())
    }
}

impl fmt::Display for FolioMarketingSlug {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl TryFrom<&str> for FolioMarketingSlug {
    type Error = String;
    fn try_from(s: &str) -> Result<Self, Self::Error> {
        Self::ALL
            .iter()
            .copied()
            .find(|v| v.as_str() == s)
            .ok_or_else(|| format!("unknown FolioMarketingSlug: '{s}'"))
    }
}

/// Resolve a known Folio marketing public path, or a generic fallback.
pub fn folio_public_path_hint(slug: &str) -> &'static str {
    FolioMarketingSlug::try_from(slug)
        .map(FolioMarketingSlug::public_path)
        .unwrap_or("/products/{slug}")
}

// ── FoundingSpotTier ──────────────────────────────────────────────────────────

/// Keys in `hero_payload.spot_inventory` for `/founding`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum FoundingSpotTier {
    #[serde(rename = "ll-grow")]
    LlGrow,
    #[serde(rename = "ll-pro")]
    LlPro,
    #[serde(rename = "ll-investor")]
    LlInvestor,
    #[serde(rename = "br-solo")]
    BrSolo,
    #[serde(rename = "br-team")]
    BrTeam,
    #[serde(rename = "br-firm")]
    BrFirm,
    #[serde(rename = "pm-starter")]
    PmStarter,
    #[serde(rename = "pm-growth")]
    PmGrowth,
    #[serde(rename = "vd-pro")]
    VdPro,
}

impl FoundingSpotTier {
    pub const ALL: &'static [Self] = &[
        Self::LlGrow,
        Self::LlPro,
        Self::LlInvestor,
        Self::BrSolo,
        Self::BrTeam,
        Self::BrFirm,
        Self::PmStarter,
        Self::PmGrowth,
        Self::VdPro,
    ];

    pub fn as_str(self) -> &'static str {
        match self {
            Self::LlGrow => "ll-grow",
            Self::LlPro => "ll-pro",
            Self::LlInvestor => "ll-investor",
            Self::BrSolo => "br-solo",
            Self::BrTeam => "br-team",
            Self::BrFirm => "br-firm",
            Self::PmStarter => "pm-starter",
            Self::PmGrowth => "pm-growth",
            Self::VdPro => "vd-pro",
        }
    }

    /// Default total/taken used when CMS inventory is missing.
    pub fn default_spots(self) -> (u32, u32) {
        match self {
            Self::LlGrow => (500, 47),
            Self::LlPro => (250, 31),
            Self::LlInvestor => (100, 12),
            Self::BrSolo => (200, 8),
            Self::BrTeam => (100, 4),
            Self::BrFirm => (50, 1),
            Self::PmStarter => (150, 7),
            Self::PmGrowth => (75, 3),
            Self::VdPro => (300, 19),
        }
    }
}

impl fmt::Display for FoundingSpotTier {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl TryFrom<&str> for FoundingSpotTier {
    type Error = String;
    fn try_from(s: &str) -> Result<Self, Self::Error> {
        Self::ALL
            .iter()
            .copied()
            .find(|v| v.as_str() == s)
            .ok_or_else(|| format!("unknown FoundingSpotTier: '{s}'"))
    }
}

// ── CtaAction ─────────────────────────────────────────────────────────────────

/// Template `cta_action` — what the primary marketing CTA should do.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CtaAction {
    Waitlist,
    Signup,
    PreOrder,
    External,
    Anchor,
}

impl CtaAction {
    pub const ALL: &'static [Self] = &[
        Self::Waitlist,
        Self::Signup,
        Self::PreOrder,
        Self::External,
        Self::Anchor,
    ];

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Waitlist => "waitlist",
            Self::Signup => "signup",
            Self::PreOrder => "pre_order",
            Self::External => "external",
            Self::Anchor => "anchor",
        }
    }
}

impl fmt::Display for CtaAction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl TryFrom<&str> for CtaAction {
    type Error = String;
    fn try_from(s: &str) -> Result<Self, Self::Error> {
        // Accept path-like legacy values used in some templates.
        match s {
            "waitlist" => Ok(Self::Waitlist),
            "signup" | "/signup" => Ok(Self::Signup),
            "pre_order" | "pre-order" => Ok(Self::PreOrder),
            "external" => Ok(Self::External),
            "anchor" => Ok(Self::Anchor),
            other => Err(format!("unknown CtaAction: '{other}'")),
        }
    }
}

// ── VendorTradeKey ────────────────────────────────────────────────────────────

/// Canonical vendor marketplace trade keys used on `/vendors` signup.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum VendorTradeKey {
    #[serde(rename = "cleaning")]
    Cleaning,
    #[serde(rename = "handyman")]
    Handyman,
    #[serde(rename = "plumbing")]
    Plumbing,
    #[serde(rename = "electrical")]
    Electrical,
    #[serde(rename = "hvac")]
    Hvac,
    #[serde(rename = "painting")]
    Painting,
    #[serde(rename = "landscaping")]
    Landscaping,
    #[serde(rename = "roofing")]
    Roofing,
    #[serde(rename = "flooring")]
    Flooring,
    #[serde(rename = "pest-control")]
    PestControl,
    #[serde(rename = "appliance")]
    Appliance,
    #[serde(rename = "locksmith")]
    Locksmith,
    #[serde(rename = "inspection")]
    Inspection,
    #[serde(rename = "movers")]
    Movers,
    #[serde(rename = "junk-removal")]
    JunkRemoval,
    #[serde(rename = "pool-spa")]
    PoolSpa,
    #[serde(rename = "security")]
    Security,
    #[serde(rename = "solar")]
    Solar,
    #[serde(rename = "general")]
    General,
}

impl VendorTradeKey {
    pub const ALL: &'static [Self] = &[
        Self::Cleaning,
        Self::Handyman,
        Self::Plumbing,
        Self::Electrical,
        Self::Hvac,
        Self::Painting,
        Self::Landscaping,
        Self::Roofing,
        Self::Flooring,
        Self::PestControl,
        Self::Appliance,
        Self::Locksmith,
        Self::Inspection,
        Self::Movers,
        Self::JunkRemoval,
        Self::PoolSpa,
        Self::Security,
        Self::Solar,
        Self::General,
    ];

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Cleaning => "cleaning",
            Self::Handyman => "handyman",
            Self::Plumbing => "plumbing",
            Self::Electrical => "electrical",
            Self::Hvac => "hvac",
            Self::Painting => "painting",
            Self::Landscaping => "landscaping",
            Self::Roofing => "roofing",
            Self::Flooring => "flooring",
            Self::PestControl => "pest-control",
            Self::Appliance => "appliance",
            Self::Locksmith => "locksmith",
            Self::Inspection => "inspection",
            Self::Movers => "movers",
            Self::JunkRemoval => "junk-removal",
            Self::PoolSpa => "pool-spa",
            Self::Security => "security",
            Self::Solar => "solar",
            Self::General => "general",
        }
    }

    pub fn default_label(self) -> &'static str {
        match self {
            Self::Cleaning => "🧹 Cleaning",
            Self::Handyman => "🔧 Handyman",
            Self::Plumbing => "🚿 Plumbing",
            Self::Electrical => "⚡ Electrical",
            Self::Hvac => "❄️ HVAC",
            Self::Painting => "🖌️ Painting",
            Self::Landscaping => "🌿 Landscaping",
            Self::Roofing => "🏠 Roofing",
            Self::Flooring => "🪵 Flooring",
            Self::PestControl => "🐛 Pest Control",
            Self::Appliance => "🛠️ Appliances",
            Self::Locksmith => "🔐 Locksmith",
            Self::Inspection => "🔍 Inspection",
            Self::Movers => "📦 Movers",
            Self::JunkRemoval => "🗑️ Junk Removal",
            Self::PoolSpa => "🏊 Pool & Spa",
            Self::Security => "📷 Security",
            Self::Solar => "☀️ Solar",
            Self::General => "🏗️ General Contractor",
        }
    }
}

impl fmt::Display for VendorTradeKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl TryFrom<&str> for VendorTradeKey {
    type Error = String;
    fn try_from(s: &str) -> Result<Self, Self::Error> {
        Self::ALL
            .iter()
            .copied()
            .find(|v| v.as_str() == s)
            .ok_or_else(|| format!("unknown VendorTradeKey: '{s}'"))
    }
}

// ── BetaApplicantRole ─────────────────────────────────────────────────────────

/// Self-reported role on the Folio beta application form.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum BetaApplicantRole {
    #[serde(rename = "landlord")]
    Landlord,
    #[serde(rename = "str-host")]
    StrHost,
    #[serde(rename = "broker")]
    Broker,
    #[serde(rename = "property-manager")]
    PropertyManager,
    #[serde(rename = "vendor")]
    Vendor,
}

impl BetaApplicantRole {
    pub const ALL: &'static [Self] = &[
        Self::Landlord,
        Self::StrHost,
        Self::Broker,
        Self::PropertyManager,
        Self::Vendor,
    ];

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Landlord => "landlord",
            Self::StrHost => "str-host",
            Self::Broker => "broker",
            Self::PropertyManager => "property-manager",
            Self::Vendor => "vendor",
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Landlord => "Landlord / Real estate investor",
            Self::StrHost => "Short-term rental (STR) host",
            Self::Broker => "Licensed broker / Real estate agent",
            Self::PropertyManager => "Property manager / Management company",
            Self::Vendor => "Vendor / Contractor / Service provider",
        }
    }
}

impl fmt::Display for BetaApplicantRole {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl TryFrom<&str> for BetaApplicantRole {
    type Error = String;
    fn try_from(s: &str) -> Result<Self, Self::Error> {
        Self::ALL
            .iter()
            .copied()
            .find(|v| v.as_str() == s)
            .ok_or_else(|| format!("unknown BetaApplicantRole: '{s}'"))
    }
}
