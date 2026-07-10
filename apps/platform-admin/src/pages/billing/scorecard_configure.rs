//! Stub: template configurator (G-27 Phase 1c).

use leptos::prelude::*;
use leptos_router::hooks::use_params_map;

#[component]
pub fn ScorecardConfigure() -> impl IntoView {
    let params = use_params_map();
    let template_id = move || {
        params.with(|p| p.get("template_id").unwrap_or_else(|| "new".to_string()))
    };

    view! {
        <div class="w-full space-y-4">
            <div class="bg-surface-container-low border border-outline-variant/20 p-6 rounded-2xl shadow-sm">
                <h1 class="text-2xl font-extrabold tracking-tight text-on-surface">
                    "Configure Scorecard Template"
                </h1>
                <p class="text-xs text-on-surface-variant mt-1 font-mono">
                    {move || format!("template_id: {}", template_id())}
                </p>
                <p class="text-sm text-on-surface-variant mt-4">
                    "Configurator coming soon — shared-ui Configurator mount lands in a follow-up."
                </p>
                <a href="/billing/scorecards" class="inline-block mt-4 text-sm text-primary hover:underline">
                    "← Back to Scorecards"
                </a>
            </div>
        </div>
    }
}
