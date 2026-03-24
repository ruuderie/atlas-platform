use leptos::prelude::*;
use crate::app::App;

mod app;
mod pages;
pub mod api;
pub mod components;


#[cfg(test)]
pub mod tests;
fn main() {
    console_error_panic_hook::set_once();
    leptos::mount::mount_to_body(App);
}
