use leptos::*;
use crate::pages::landing::DesignConfig;

/// Returns true if the current tenant has the Kami academic design system enabled.
/// Reads from the `DesignConfig` context provided by `App()`.
/// Defaults to `false` for all tenants where `kami_mode` is not explicitly set.
pub fn use_kami_mode() -> bool {
    use_context::<DesignConfig>()
        .map(|cfg| cfg.kami_mode)
        .unwrap_or(false)
}
