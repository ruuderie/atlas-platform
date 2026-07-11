//! `use_version_check` — headless hook for platform-wide deployment detection.
//!
//! ## Usage
//!
//! ```rust
//! let new_version_available = use_version_check("https://api.dev.atlas.oply.co");
//!
//! view! {
//!     <Show when=move || new_version_available.get()>
//!         // Your app's reload UI here
//!     </Show>
//! }
//! ```
//!
//! Or use the bundled `<VersionBanner api_base=... />` component which renders
//! the default Atlas-style banner for you.
//!
//! ## Contract
//!
//! - Calls `GET {api_base}/api/version` (no auth) on mount.
//! - Records the `build_sha` from the first successful response as the baseline.
//! - Re-polls every `POLL_INTERVAL_MS` (5 minutes).
//! - Returns a `ReadSignal<bool>` that becomes `true` when the SHA changes.
//! - Stops polling once the signal is set to `true`.
//! - Network errors are silently ignored (retry next interval).

use js_sys::Promise;
use leptos::prelude::*;
use serde::Deserialize;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::JsFuture;

const POLL_INTERVAL_MS: i32 = 5 * 60 * 1000; // 5 minutes

#[derive(Clone, Debug, Deserialize, PartialEq)]
struct VersionInfo {
    pub build_sha: String,
}

/// Sleep for `ms` milliseconds using the browser's `setTimeout`.
async fn sleep_ms(ms: i32) {
    let promise = Promise::new(&mut |resolve, _reject| {
        web_sys::window()
            .unwrap()
            .set_timeout_with_callback_and_timeout_and_arguments_0(&resolve, ms)
            .unwrap();
    });
    JsFuture::from(promise).await.unwrap();
}

/// Fetch `{api_base}/api/version` and return the `build_sha` on success.
async fn fetch_build_sha(api_base: &str) -> Option<String> {
    let url = format!("{}/api/version", api_base.trim_end_matches('/'));
    let res = reqwest::Client::new().get(&url).send().await.ok()?;
    if res.status().is_success() {
        res.json::<VersionInfo>().await.ok().map(|v| v.build_sha)
    } else {
        None
    }
}

/// Headless version-check hook.
///
/// Returns a `ReadSignal<bool>` that becomes `true` when a new backend
/// deployment is detected. Pass this into your own `<Show>` or conditional
/// rendering to display a reload prompt in whatever style fits your app.
///
/// `api_base` should be the full base URL of the Atlas backend for this
/// environment, e.g. `"https://api.dev.atlas.oply.co"`.
pub fn use_version_check(api_base: impl Into<String> + 'static) -> ReadSignal<bool> {
    let (new_version_available, set_new_version_available) = signal(false);
    let api_base = api_base.into();

    leptos::task::spawn_local(async move {
        let baseline: std::cell::Cell<Option<String>> = std::cell::Cell::new(None);

        loop {
            if let Some(sha) = fetch_build_sha(&api_base).await {
                match baseline.take() {
                    None => {
                        // First successful call — record baseline.
                        baseline.set(Some(sha));
                    }
                    Some(ref b) if *b != sha => {
                        // SHA changed — new deployment.
                        set_new_version_available.set(true);
                        break; // Stop polling.
                    }
                    Some(b) => {
                        // Same SHA — put it back and continue.
                        baseline.set(Some(b));
                    }
                }
            }
            sleep_ms(POLL_INTERVAL_MS).await;
        }
    });

    new_version_available
}
