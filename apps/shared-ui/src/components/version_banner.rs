//! `VersionBanner` — default Atlas-style reload prompt for new deployments.
//!
//! This is the **opinionated** rendering layer on top of the headless
//! `use_version_check` hook. Use it when you want the standard platform
//! look-and-feel. If your app needs custom styling or behaviour, use
//! `use_version_check` directly instead.
//!
//! ## Usage
//!
//! ```rust
//! // In your App root:
//! view! {
//!     <VersionBanner api_base="https://api.dev.atlas.oply.co" />
//!     // ... rest of your app
//! }
//! ```
//!
//! The banner renders a fixed top bar (z-index 10000) with a "Reload" button
//! when a new backend deployment is detected. A spacer `<div class="h-10" />`
//! is also rendered so content does not sit behind the bar.

use leptos::prelude::*;
use super::hooks::use_version_check::use_version_check;

#[component]
pub fn VersionBanner(
    /// Base URL of the Atlas backend for this environment.
    /// Example: `"https://api.dev.atlas.oply.co"`
    api_base: &'static str,
) -> impl IntoView {
    let new_version_available = use_version_check(api_base);

    view! {
        <Show when=move || new_version_available.get()>
            <div
                class="fixed top-0 left-0 right-0 z-[10000] flex items-center justify-between px-6 py-2.5 bg-primary text-on-primary text-xs font-semibold shadow-lg"
                role="alert"
                aria-live="polite"
            >
                <div class="flex items-center gap-2">
                    <span class="text-sm" aria-hidden="true">{"🚀"}</span>
                    "A new version of the platform is available."
                </div>
                <button
                    class="px-4 py-1.5 rounded-lg bg-on-primary text-primary font-bold text-xs hover:opacity-90 transition-all shrink-0"
                    on:click=|_| {
                        let _ = web_sys::window()
                            .expect("window")
                            .location()
                            .reload();
                    }
                >
                    "Reload"
                </button>
            </div>
            // Spacer prevents content from sitting behind the banner.
            <div class="h-10" />
        </Show>
    }
}
