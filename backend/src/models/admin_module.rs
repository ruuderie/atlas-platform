//! Admin Module Registry
//!
//! This module is the **single source of truth** for the Atlas Platform's
//! administrative module type system. It is shared across all Atlas apps
//! (`anchor`, `network-instance`, future apps) via the backend crate.
//!
//! # Architecture
//!
//! ```text
//! AdminModuleType (enum)  — compile-time type safety, defines what CAN exist
//! app_instance_module (DB) — runtime config, defines what IS enabled per tenant
//! ```
//!
//! Apps declare their module set via `AtlasApp::default_modules()` and it is
//! seeded into the DB during `AtlasApp::provision()`.

use serde::{Deserialize, Serialize};
use strum_macros::{Display, EnumIter, EnumString};


// ─────────────────────────────────────────────────────────────────────────────
// MODULE CATEGORY
// ─────────────────────────────────────────────────────────────────────────────

/// Logical grouping for admin module types.
///
/// Used for visual grouping in the Platform Admin onboarding UI and
/// for future sidebar section headers in tenant admin pages.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ModuleCategory {
    /// Fixed modules the platform always includes — cannot be hidden.
    Platform,
    /// Content management modules (blog, resume, landing pages, etc.).
    Content,
    /// Site configuration and appearance (nav, footer, page headers).
    Appearance,
    /// CRM and communication modules (leads, contacts, lead options).
    CrmAndComms,
    /// B2B and service offering modules (services, case studies, highlights).
    B2B,
    /// Advanced / tenant-specific or marketplace modules.
    Advanced,
}

// ─────────────────────────────────────────────────────────────────────────────
// ADMIN MODULE TYPE
// ─────────────────────────────────────────────────────────────────────────────

/// The canonical set of admin module types supported by the Atlas Platform.
///
/// This enum is the compile-time registry of all possible admin tab types.
/// The database (`app_instance_module`) records which subset is enabled
/// for each tenant and in what order.
///
/// # Adding a new module type
///
/// 1. Add a variant here with a doc comment.
/// 2. Add it to the relevant `category()` match arm.
/// 3. Add a `default_sort_order()` value (use a gap to allow insertion).
/// 4. Add it to `to_display_name()`.
/// 5. Add it to the relevant `AtlasApp::default_modules()` implementation
///    for any apps that should offer it.
///
/// # Serialization
///
/// Serializes as `SCREAMING_SNAKE_CASE` for DB storage and JSON transport.
/// The `strum` derives allow `from_str` parsing for route/query params.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash,
    Serialize, Deserialize,
    Display, EnumIter, EnumString,
)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
#[strum(serialize_all = "SCREAMING_SNAKE_CASE")]
pub enum AdminModuleType {
    // ── Platform (Fixed — always present) ────────────────────────────────
    /// Main overview dashboard. Always the first tab.
    Dashboard,
    /// Global site/app settings. Always present.
    Settings,
    /// Passkey and admin identity management. Always present.
    Security,

    // ── Content ──────────────────────────────────────────────────────────
    /// Blog post management.
    Blog,
    /// Resume profile management.
    ResumeProfiles,
    /// Resume entry management (experience, projects, etc.).
    ResumeEntries,
    /// CMS landing page management.
    LandingPages,
    /// Form builder / lead origination schema management.
    Webforms,

    // ── Appearance ───────────────────────────────────────────────────────
    /// Site navigation item management.
    Navigation,
    /// Site footer item management.
    Footer,
    /// Page header / hero management.
    PageHeaders,

    // ── CRM & Comms ──────────────────────────────────────────────────────
    /// Inbound unvetted inquiries — raw CRM input, no opt-in required.
    /// Distinct from Contacts: a Lead has not been vetted or onboarded.
    Leads,
    /// Opted-in / vetted contacts. A Contact has an explicit relationship
    /// with the tenant (newsletter subscriber, onboarded user, etc.).
    /// Supports email and SMS communication channels.
    Contacts,
    /// Lead capture form option configuration (form field options, not CRM leads).
    LeadOptions,

    // ── B2B ──────────────────────────────────────────────────────────────
    /// B2B service offering management.
    Services,
    /// Case study / portfolio management.
    CaseStudies,
    /// Highlight / feature snap management.
    Highlights,

    // ── Advanced / Marketplace ───────────────────────────────────────────
    /// Real estate property listings.
    Properties,
    /// Generic directory / marketplace listings.
    Listings,
    /// Escape hatch: fully tenant-configured module with custom display_name.
    Custom,
}

impl AdminModuleType {
    /// Returns the logical category this module belongs to.
    pub fn category(self) -> ModuleCategory {
        match self {
            Self::Dashboard | Self::Settings | Self::Security => ModuleCategory::Platform,
            Self::Blog | Self::ResumeProfiles | Self::ResumeEntries
                | Self::LandingPages | Self::Webforms => ModuleCategory::Content,
            Self::Navigation | Self::Footer | Self::PageHeaders => ModuleCategory::Appearance,
            Self::Leads | Self::Contacts | Self::LeadOptions => ModuleCategory::CrmAndComms,
            Self::Services | Self::CaseStudies | Self::Highlights => ModuleCategory::B2B,
            Self::Properties | Self::Listings | Self::Custom => ModuleCategory::Advanced,
        }
    }

    /// Returns true if this module is a platform-fixed module that cannot be
    /// disabled for any tenant.
    pub fn is_fixed(self) -> bool {
        self.category() == ModuleCategory::Platform
    }

    /// Returns the platform default sort order for this module type.
    /// Preserves the current `buildwithruud` tab ordering for backward compat.
    /// Uses gaps (multiples of 10) to allow insertion without renumbering.
    pub fn default_sort_order(self) -> i32 {
        match self {
            Self::Dashboard      => 0,
            Self::Blog           => 10,
            Self::Services       => 20,
            Self::CaseStudies    => 30,
            Self::Highlights     => 40,
            Self::Contacts       => 50,
            Self::Settings       => 60,
            Self::LeadOptions    => 70,
            Self::Navigation     => 80,
            Self::Footer         => 90,
            Self::PageHeaders    => 100,
            Self::LandingPages   => 110,
            Self::ResumeProfiles => 120,
            Self::ResumeEntries  => 130,
            Self::Webforms       => 140,
            Self::Security       => 150,
            Self::Leads          => 160,
            Self::Properties     => 170,
            Self::Listings       => 180,
            Self::Custom         => 990,
        }
    }

    /// Returns the canonical human-readable display name for this module type.
    /// Tenants may override this via the `display_name` column in `app_instance_module`.
    pub fn to_display_name(self) -> &'static str {
        match self {
            Self::Dashboard      => "Dashboard",
            Self::Settings       => "Settings",
            Self::Security       => "Security",
            Self::Blog           => "Blog",
            Self::ResumeProfiles => "Resume Profiles",
            Self::ResumeEntries  => "Resume Entries",
            Self::LandingPages   => "Landing Pages",
            Self::Webforms       => "Webforms",
            Self::Navigation     => "Navigation",
            Self::Footer         => "Footer",
            Self::PageHeaders    => "Page Headers",
            Self::Leads          => "Leads",
            Self::Contacts       => "Contacts",
            Self::LeadOptions    => "Lead Options",
            Self::Services       => "Services",
            Self::CaseStudies    => "Case Studies",
            Self::Highlights     => "Highlights",
            Self::Properties     => "Properties",
            Self::Listings       => "Listings",
            Self::Custom         => "Custom",
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// WIRE TYPES (API / Leptos serialization boundary)
// ─────────────────────────────────────────────────────────────────────────────

/// The serialized representation of a module sent to and from the frontend.
///
/// This is what `GET /api/admin/modules` returns and what
/// `POST /api/platform/tenants/{id}/modules` accepts.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AdminModuleConfig {
    /// The canonical module type. Used as the tab identity key.
    pub module_type: AdminModuleType,
    /// Human-readable label shown in the sidebar. May be overridden per tenant.
    pub display_name: String,
    /// Optional Material Symbols icon name.
    pub icon: Option<String>,
    /// Position in the sidebar. Lower = higher. Tenant-configurable.
    pub sort_order: i32,
    /// Whether this module is a fixed platform module (cannot be disabled).
    pub is_fixed: bool,
    /// Logical grouping for Platform Admin UI organization.
    pub category: ModuleCategory,
}

impl AdminModuleConfig {
    /// Construct a config from just a type, using all platform defaults.
    pub fn from_type(module_type: AdminModuleType) -> Self {
        Self {
            display_name: module_type.to_display_name().to_string(),
            icon: None,
            sort_order: module_type.default_sort_order(),
            is_fixed: module_type.is_fixed(),
            category: module_type.category(),
            module_type,
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// PLATFORM ADMIN UPSERT INPUT
// ─────────────────────────────────────────────────────────────────────────────

/// Input body for `POST /api/platform/tenants/{id}/modules`.
#[derive(Debug, Deserialize)]
pub struct UpsertModuleInput {
    pub module_type: AdminModuleType,
    pub display_name: Option<String>,
    pub icon: Option<String>,
    pub sort_order: Option<i32>,
    pub is_enabled: bool,
}

// ─────────────────────────────────────────────────────────────────────────────
// UNIT TESTS
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use strum::IntoEnumIterator;
    use super::*;

    #[test]
    fn test_all_variants_have_category() {
        // Every variant must match a category arm (compile-time enforced,
        // but this confirms no panics at runtime for future additions).
        for variant in AdminModuleType::iter() {
            let _ = variant.category();
        }
    }

    #[test]
    fn test_fixed_modules_are_platform_category() {
        assert!(AdminModuleType::Dashboard.is_fixed());
        assert!(AdminModuleType::Settings.is_fixed());
        assert!(AdminModuleType::Security.is_fixed());
        assert!(!AdminModuleType::Blog.is_fixed());
        assert!(!AdminModuleType::Leads.is_fixed());
        assert!(!AdminModuleType::Contacts.is_fixed());
    }

    #[test]
    fn test_category_assignments() {
        assert_eq!(AdminModuleType::Leads.category(),    ModuleCategory::CrmAndComms);
        assert_eq!(AdminModuleType::Contacts.category(), ModuleCategory::CrmAndComms);
        assert_eq!(AdminModuleType::Blog.category(),     ModuleCategory::Content);
        assert_eq!(AdminModuleType::Services.category(), ModuleCategory::B2B);
        assert_eq!(AdminModuleType::Listings.category(), ModuleCategory::Advanced);
    }

    #[test]
    fn test_serde_round_trip() {
        for variant in AdminModuleType::iter() {
            let serialized   = serde_json::to_string(&variant).expect("serialize");
            let deserialized: AdminModuleType = serde_json::from_str(&serialized).expect("deserialize");
            assert_eq!(variant, deserialized, "round-trip failed for {variant:?}");
        }
    }

    #[test]
    fn test_strum_from_str_round_trip() {
        use std::str::FromStr;
        for variant in AdminModuleType::iter() {
            let s = variant.to_string(); // SCREAMING_SNAKE_CASE via strum Display
            let parsed = AdminModuleType::from_str(&s)
                .unwrap_or_else(|_| panic!("from_str failed for {s}"));
            assert_eq!(variant, parsed);
        }
    }

    #[test]
    fn test_from_type_constructor() {
        let cfg = AdminModuleConfig::from_type(AdminModuleType::Blog);
        assert_eq!(cfg.module_type,  AdminModuleType::Blog);
        assert_eq!(cfg.display_name, "Blog");
        assert_eq!(cfg.category,     ModuleCategory::Content);
        assert!(!cfg.is_fixed);
    }

    #[test]
    fn test_dashboard_is_sort_order_zero() {
        assert_eq!(AdminModuleType::Dashboard.default_sort_order(), 0);
    }

    #[test]
    fn test_all_display_names_non_empty() {
        for variant in AdminModuleType::iter() {
            assert!(
                !variant.to_display_name().is_empty(),
                "{variant:?} has empty display name"
            );
        }
    }
}
