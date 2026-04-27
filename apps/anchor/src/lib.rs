pub mod app;
pub mod auth;
pub mod b2b;
pub mod components;
pub mod email;
pub mod pages;
pub mod resume_engine;
#[cfg(feature = "ssr")]
pub mod state;
#[cfg(feature = "ssr")]
pub mod handlers;
#[cfg(feature = "ssr")]
pub mod atlas_client;
#[cfg(feature = "ssr")]
pub use state::AppState;

#[cfg(feature = "hydrate")]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn hydrate() {
    use crate::app::*;

    console_error_panic_hook::set_once();

    leptos::mount_to_body(App);
}
