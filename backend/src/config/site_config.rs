use serde::{Deserialize, Serialize};
use uuid::Uuid;
use std::collections::HashMap;
use bitflags::bitflags;
use serde_json::Value;

bitflags! {
    #[derive(Serialize, Deserialize)]
    pub struct ModuleFlags: i32 {
        const LISTINGS = 0b00000001;
        const PROFILES = 0b00000010;
        const MESSAGING = 0b00000100;
        const PAYMENTS = 0b00001000;
        const ANALYTICS = 0b00010000;
        const REVIEWS = 0b00100000;
        const EVENTS = 0b01000000;
        const CUSTOM_FIELDS = 0b10000000;
        // Add more modules as needed
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SiteConfig {
    pub directory_id: Uuid,
    pub name: String,
    pub domain: String,
    pub subdomain: Option<String>,
    pub custom_domain: Option<String>,
    pub enabled_modules: ModuleFlags,
    pub theme: Option<String>,
    pub custom_settings: HashMap<String, Value>,
    pub site_status: String,
}

impl SiteConfig {
    pub fn is_module_enabled(&self, module: ModuleFlags) -> bool {
        self.enabled_modules.contains(module)
    }
    
    pub fn is_active(&self) -> bool {
        self.site_status == "active"
    }
}
