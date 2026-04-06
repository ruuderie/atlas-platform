use leptos::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AppManifest {
    pub app_type_id: String,
    pub name: String,
    pub panels: Vec<PanelConfig>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PanelConfig {
    pub id: String,
    pub title: String,
    pub icon: Option<String>,
}

pub fn get_manifest_for_app_type(type_name: &str) -> AppManifest {
    match type_name {
        "anchor" | "anchor_app" | "Services" | "services" => AppManifest {
            app_type_id: "anchor_app".to_string(),
            name: "Anchor Services Portal".to_string(),
            panels: vec![
                PanelConfig { id: "profiles".to_string(), title: "Identities".to_string(), icon: Some("group".to_string()) },
                PanelConfig { id: "services".to_string(), title: "Service Offerings".to_string(), icon: Some("sell".to_string()) },
                PanelConfig { id: "anchor_settings".to_string(), title: "System Configuration".to_string(), icon: Some("settings".to_string()) },
            ]
        },
        _ => AppManifest { // Default Network Manifest
            app_type_id: "network".to_string(),
            name: "Network Network".to_string(),
            panels: vec![
                PanelConfig { id: "listings".to_string(), title: "Listings".to_string(), icon: Some("list_alt".to_string()) },
                PanelConfig { id: "profiles".to_string(), title: "User Profiles".to_string(), icon: Some("group".to_string()) },
                PanelConfig { id: "categories".to_string(), title: "Categories".to_string(), icon: Some("category".to_string()) },
                PanelConfig { id: "templates".to_string(), title: "Templates".to_string(), icon: Some("draw".to_string()) },
                PanelConfig { id: "network_settings".to_string(), title: "Settings".to_string(), icon: Some("settings".to_string()) },
            ]
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_manifest_for_anchor_app() {
        let manifest = get_manifest_for_app_type("anchor_app");
        assert_eq!(manifest.app_type_id, "anchor_app");
        assert_eq!(manifest.panels.len(), 3);
        assert_eq!(manifest.panels[0].id, "profiles");
        assert_eq!(manifest.panels[1].id, "services");
        assert_eq!(manifest.panels[2].id, "anchor_settings");
    }

    #[test]
    fn test_manifest_for_anchor_alias() {
        let manifest = get_manifest_for_app_type("anchor");
        assert_eq!(manifest.app_type_id, "anchor_app");
    }

    #[test]
    fn test_manifest_for_default_network() {
        let manifest = get_manifest_for_app_type("network");
        assert_eq!(manifest.app_type_id, "network");
        assert_eq!(manifest.panels.len(), 5);
        assert_eq!(manifest.panels[0].id, "listings");
        assert_eq!(manifest.panels[4].id, "network_settings");
    }

    #[test]
    fn test_manifest_for_unknown_fallback() {
        let manifest = get_manifest_for_app_type("something_unknown");
        assert_eq!(manifest.app_type_id, "network"); // Defaults to network
    }
}
