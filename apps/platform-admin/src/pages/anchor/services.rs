use leptos::prelude::*;

#[component]
pub fn ServicesPanel() -> impl IntoView {
    view! {
        <div class="p-6 bg-white dark:bg-gray-800 rounded-xl shadow-sm border border-gray-100 dark:border-gray-700">
            <h2 class="text-xl font-semibold mb-4 dark:text-white">"Service Offerings"</h2>
            <p class="text-gray-500 dark:text-gray-400">"Manage billable services and one-off products."</p>
        </div>
    }
}
