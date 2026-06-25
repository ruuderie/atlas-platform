//! VersionBanner — detects when a new backend deployment is live and prompts
//! the operator to reload.
//!
//! ## How it works
//!
//! On mount it calls `GET /api/version` once to capture the baseline `build_sha`.
//! It then uses `set_interval` to re-poll every 5 minutes. If the returned SHA
//! ever differs from the baseline, a non-dismissable banner slides in at the top
//! of the viewport asking the user to reload. Clicking "Reload" calls
//! `window.location.reload()`.
//!
//! The WASM itself does not need to know its own SHA — we only compare the
//! backend's sha against itself across time. When CI ships a new image, the
//! backend's `build_sha` changes and the banner fires.
//!
//! This replaces the previous implicit signal (session expiry on pod restart)
//! which was unreliable once sessions moved to Postgres.

use leptos::prelude::*;
use gloo_timers::future::TimeoutFuture;
use crate::api::admin::get_backend_version;

/// How often to poll `/api/version` in milliseconds (5 minutes).
const POLL_INTERVAL_MS: u32 = 5 * 60 * 1000;

#[component]
pub fn VersionBanner() -> impl IntoView {
    // `None`  = baseline not yet captured
    // `Some(sha)` = sha seen at startup; if it changes → show banner
    let baseline_sha: RwSignal<Option<String>> = RwSignal::new(None);
    let new_version_available = RwSignal::new(false);

    // Kick off the poller as a background task.
    leptos::task::spawn_local(async move {
        loop {
            match get_backend_version().await {
                Ok(info) => {
                    match baseline_sha.get_untracked() {
                        None => {
                            // First successful call — record baseline.
                            baseline_sha.set(Some(info.build_sha));
                        }
                        Some(ref baseline) if *baseline != info.build_sha => {
                            // SHA changed — new deployment detected.
                            new_version_available.set(true);
                            // Stop polling once we've shown the banner.
                            break;
                        }
                        _ => {}
                    }
                }
                Err(_) => {
                    // Network blip — ignore and retry next interval.
                }
            }
            TimeoutFuture::new(POLL_INTERVAL_MS).await;
        }
    });

    view! {
        <Show when=move || new_version_available.get()>
            // Fixed top bar — sits above everything, not dismissable.
            <div
                class="fixed top-0 left-0 right-0 z-[10000] flex items-center justify-between px-6 py-2.5 bg-primary text-on-primary text-xs font-semibold shadow-lg"
                role="alert"
            >
                <div class="flex items-center gap-2">
                    <span class="text-sm">{"🚀"}</span>
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
            // Spacer so content doesn't sit behind the banner.
            <div class="h-10" />
        </Show>
    }
}
