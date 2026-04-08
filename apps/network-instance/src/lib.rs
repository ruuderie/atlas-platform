pub mod app;
pub mod components;
pub mod pages;
pub mod auth;

#[cfg(feature = "hydrate")]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn hydrate() {
    console_error_panic_hook::set_once();
    leptos::mount::hydrate_body(app::App);
}

pub fn get_api_base_url() -> String {
    #[cfg(feature = "ssr")]
    {
        std::env::var("CONTAINER_API_URL").unwrap_or_else(|_| "http://127.0.0.1:8000".to_string())
    }
    #[cfg(not(feature = "ssr"))]
    {
        std::option_env!("BROWSER_API_URL").unwrap_or("http://127.0.0.1:8000").to_string()
    }
}
