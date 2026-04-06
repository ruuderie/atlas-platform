use leptos::prelude::*;
use leptos_router::hooks::use_params_map;

// Component Panel Imports
// In the future, this static mapping can be replaced with a dynamic Remote Schema JSON parser
// that builds data-driven forms and tables based entirely on server-sent UI schematics.
// For V1, we maintain explicit Leptos components for performance and type safety.
use crate::pages::anchor::settings::AnchorSettingsPanel;
use crate::pages::shared::profiles::ProfilesPanel;
use crate::pages::anchor::services::ServicesPanel;
use crate::pages::network::settings::NetworkSettingsPanel;
use crate::pages::network::categories::Categories;
use crate::pages::network::templates::Templates;
use crate::pages::network::listings::Listings;

#[component]
pub fn PlaceholderPanel(panel_id: String) -> impl IntoView {
    view! {
        <div class="p-6 flex flex-col items-center justify-center min-h-[300px] bg-slate-50 dark:bg-slate-900/50 rounded-xl border border-dashed border-slate-200 dark:border-slate-700">
            <span class="material-symbols-outlined text-4xl text-slate-400 mb-3">"construction"</span>
            <h2 class="text-xl font-semibold mb-1 dark:text-white">"Module Under Construction"</h2>
            <p class="text-slate-500 text-sm">"The module '" <span class="font-mono text-indigo-500">{panel_id}</span> "' is not yet implemented."</p>
        </div>
    }
}

/// DynamicPanel maps string flags sent by the AppManifest directly to explicit view trees.
/// 
/// **ROADMAP to V2 Component:**
/// When transitioning to schema-driven rendering, replace this Match statement with a 
/// `SchemaRenderer` component that dynamically constructs tables, forms, and widgets 
/// taking an arbitrary JSON object as its properties. 
#[component]
pub fn DynamicPanel(#[prop(into)] panel_id: Signal<String>) -> impl IntoView {
    view! {
        <div class="w-full animation-fade-in">
            {move || match panel_id.get().as_str() {
                "anchor_settings" => view! { <AnchorSettingsPanel /> }.into_any(),
                "network_settings" => view! { <NetworkSettingsPanel /> }.into_any(),
                "listings" => view! { <Listings /> }.into_any(),
                "categories" => view! { <Categories /> }.into_any(),
                "templates" => view! { <Templates /> }.into_any(),
                "profiles" => view! { <ProfilesPanel /> }.into_any(),
                "services" => view! { <ServicesPanel /> }.into_any(),
                other => view! { <PlaceholderPanel panel_id=other.to_string() /> }.into_any(),
            }}
        </div>
    }
}
