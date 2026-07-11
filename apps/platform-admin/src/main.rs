use crate::app::App;

pub mod api;
mod app;
pub mod components;
mod pages;
pub mod utils;

#[cfg(test)]
pub mod tests;
fn main() {
    console_error_panic_hook::set_once();
    leptos::mount::mount_to_body(App);
}
// Trigger CI

// ci-build-trigger
