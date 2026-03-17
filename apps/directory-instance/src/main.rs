use leptos::prelude::*;
use leptos_router::components::Router;

use serde::{Deserialize, Serialize};
use gloo_net::http::Request;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct DirectoryConfig {
    pub id: String,
    pub name: String,
    pub domain: String,
    pub description: String,
    // Add additional theming fields later
}

async fn fetch_directory_config(domain: String) -> Result<DirectoryConfig, String> {
    // In production, the ingress controller unifies routes. Locally, we proxy to the backend API.
    let url = format!("/directories/lookup?domain={}", domain);
    
    let req = Request::get(&url);
    match req.send().await {
        Ok(resp) => {
            if resp.ok() {
                resp.json::<DirectoryConfig>().await.map_err(|e| e.to_string())
            } else {
                Err(format!("Error: {}", resp.status()))
            }
        },
        Err(e) => Err(e.to_string())
    }
}

#[component]
fn App() -> impl IntoView {
    // 1. Extract the Custom Domain from the browser window header natively
    let host = window().location().hostname().unwrap_or_else(|_| "localhost".to_string());
    
    // 2. Fetch the corresponding tenant config based on that specific host
    let fetch_host = host.clone();
    let err_host = host.clone();
    
    // Using LocalResource because gloo_net futures are !Send (WASM main-thread bound)
    let directory_resource = LocalResource::new(move || {
        let h = fetch_host.clone();
        async move { fetch_directory_config(h).await }
    });

    view! {
        <Router>
            <main class="min-h-screen bg-background text-foreground flex flex-col items-center justify-center p-4">
                <Suspense fallback=|| view! { <p class="animate-pulse text-muted-foreground">"Resolving custom domain configuration..."</p> }>
                    {move || match directory_resource.get() {
                        None => view! { <div/> }.into_any(),
                        Some(Ok(config)) => view! {
                            <div class="text-center space-y-4 max-w-lg transition-all animate-in fade-in zoom-in duration-300">
                                <h1 class="text-4xl font-bold tracking-tight">"Welcome to " {config.name}</h1>
                                <p class="text-muted-foreground text-lg">{config.description}</p>
                                <shared_ui::components::ui::badge::Badge variant=shared_ui::components::ui::badge::BadgeVariant::Secondary>
                                    "Connected exactly via: " {config.domain}
                                </shared_ui::components::ui::badge::Badge>
                                <div class="mt-8">
                                    <shared_ui::components::ui::button::Button variant=shared_ui::components::ui::button::ButtonVariant::Default>"Browse Local Listings"</shared_ui::components::ui::button::Button>
                                </div>
                            </div>
                        }.into_any(),
                        Some(Err(e)) => view! {
                            <div class="text-center space-y-4 max-w-lg text-destructive border border-destructive/20 bg-destructive/5 p-8 rounded-xl">
                                <h1 class="text-2xl font-bold">"Directory Not Found"</h1>
                                <p>"We couldn't connect a dedicated database record for the domain [" {err_host.clone()} "]."</p>
                                <p class="text-xs opacity-50 mt-4">{e}</p>
                            </div>
                        }.into_any(),
                    }}
                </Suspense>
            </main>
        </Router>
    }
}

fn main() {
    console_error_panic_hook::set_once();
    leptos::mount::mount_to_body(App);
}
