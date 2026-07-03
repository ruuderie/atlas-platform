#![recursion_limit = "512"]

pub mod app;
pub mod auth;
pub mod components;
pub mod geo;
pub mod models;
pub mod pages;
pub mod utils;

#[cfg(feature = "ssr")]
pub mod atlas_client;
#[cfg(feature = "ssr")]
pub mod state;
#[cfg(feature = "ssr")]
pub use state::AppState;

#[cfg(feature = "hydrate")]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn hydrate() {
    use crate::app::App;
    console_error_panic_hook::set_once();
    leptos::mount::hydrate_body(App);
}
