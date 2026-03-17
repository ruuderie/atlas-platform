use leptos::prelude::*;
use leptos_router::components::{Route, Router, Routes};
use leptos_router::path;

#[component]
fn App() -> impl IntoView {
    view! {
        <Router>
            <main class="min-h-screen bg-background text-foreground flex flex-col items-center justify-center p-4">
                <Routes fallback=|| view! { "Page not found." }>
                    <Route
                        path=path!("")
                        view=|| view! {
                            <div class="text-center space-y-4 max-w-lg">
                                <h1 class="text-4xl font-bold tracking-tight">"Welcome to the Directory"</h1>
                                <p class="text-muted-foreground text-lg">"This is the initialized public-facing tenant application. It independently consumes the shared-ui package."</p>
                                <shared_ui::components::ui::button::Button variant=shared_ui::components::ui::button::ButtonVariant::Default>"Browse Listings"</shared_ui::components::ui::button::Button>
                            </div>
                        }
                    />
                </Routes>
            </main>
        </Router>
    }
}

fn main() {
    console_error_panic_hook::set_once();
    leptos::mount::mount_to_body(App);
}
