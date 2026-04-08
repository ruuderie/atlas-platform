pub mod anchor;
pub mod network_instance;

use crate::traits::atlas_app::AtlasApp;

/// Dynamically returns instantiated applications for global platform ingestion.
pub fn get_active_apps() -> Vec<Box<dyn AtlasApp>> {
    vec![
        Box::new(anchor::AnchorApp),
        Box::new(network_instance::NetworkInstanceApp),
    ]
}
