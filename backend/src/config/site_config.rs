use serde::{Deserialize, Serialize};
use uuid::Uuid;
use std::collections::HashMap;
use bitflags::bitflags;
use serde_json::Value;

// ModuleFlags defines the available features that can be enabled/disabled per site
// Each flag represents a distinct piece of functionality that can be toggled
bitflags! {
    #[derive(Serialize, Deserialize, Debug, Clone)]
    pub struct ModuleFlags: u32 {
        // Core listing functionality - enables basic directory features
        // Required for creating, viewing, and managing listings
        const LISTINGS = 0b00000001;

        // User profiles and account management functionality
        // Enables user registration, profile creation, and management
        const PROFILES = 0b00000010;

        // Internal messaging system between users
        // Enables direct communication between users within the platform
        const MESSAGING = 0b00000100;

        // Payment processing and subscription management
        // Enables paid listings, subscriptions, and financial transactions
        const PAYMENTS = 0b00001000;

        // Site usage and performance tracking
        // Enables tracking of user behavior, performance metrics, and site statistics
        const ANALYTICS = 0b00010000;

        // User-generated reviews and ratings
        // Enables users to leave feedback and ratings on listings
        const REVIEWS = 0b00100000;

        // Calendar and event management
        // Enables creation and management of events, bookings, and schedules
        const EVENTS = 0b01000000;

        // Customizable fields for listings/profiles
        // Enables custom field definitions for listings and user profiles
        const CUSTOM_FIELDS = 0b10000000;
    }
}

// SiteConfig holds all configuration data for a specific directory site
// This structure is used throughout the application to determine site behavior
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SiteConfig {
    // Unique identifier for the directory/site
    pub tenant_id: Uuid,

    // Display name of the site shown in UI elements
    pub name: String,

    // Primary domain for accessing the site (e.g., "healthcare.example.com")
    pub domain: String,

    // Optional subdomain for the site (e.g., "healthcare" in "healthcare.example.com")
    pub subdomain: Option<String>,

    // Optional custom domain for the site (e.g., "mycustomsite.com")
    pub custom_domain: Option<String>,

    // Bitflag configuration controlling which features are available
    pub enabled_modules: ModuleFlags,

    // Theme identifier (e.g., "default", "dark", "professional")
    pub theme: Option<String>,

    // Current status of the site ("active", "inactive", etc.)
    pub site_status: Option<String>,

    // Flexible JSON storage for site-specific settings
    // Can include:
    // - Visual customization (colors, logos, fonts)
    // - Contact information
    // - Social media links
    // - SEO settings
    // - Custom integration settings
    pub custom_settings: HashMap<String, Value>,
}

impl SiteConfig {
    // Helper method to check if a specific module is enabled for this site
    // Usage: site_config.is_module_enabled(ModuleFlags::LISTINGS)
    pub fn is_module_enabled(&self, module: ModuleFlags) -> bool {
        self.enabled_modules.contains(module)
    }
}
