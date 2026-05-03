pub mod anchor;
pub mod core_platform;
pub mod network_instance;
pub mod seeds;

use crate::traits::atlas_app::AtlasApp;

/// Returns all active AtlasApp implementations in registration order.
///
/// ORDER MATTERS — Axum merges routes in the order apps are returned.
/// `CorePlatformApp` MUST be first: it owns all cross-cutting CMS and
/// platform service routes that every tenant receives automatically.
/// Domain sub-apps (Anchor, NetworkInstance) are merged after.
pub fn get_active_apps() -> Vec<Box<dyn AtlasApp>> {
    vec![
        Box::new(core_platform::CorePlatformApp), // MUST be first — owns Tier 1 routes
        Box::new(anchor::AnchorApp),
        Box::new(network_instance::NetworkInstanceApp),
    ]
}
