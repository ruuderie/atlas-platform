use leptos::*;
use crate::atlas_client::fetch_atlas_data;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AppPageInfo {
    pub id: i32,
    pub slug: String,
    pub title: String,
    pub is_published: bool,
}

#[component]
pub fn PagesList(
    #[prop(into)] on_edit: Callback<String>,
    #[prop(into)] on_new: Callback<()>,
) -> impl IntoView {
    // In a real app, we would fetch from /api/anchor/pages
    // For now, we simulate with a dummy response or call the actual API if auth is passed.
    
    view! {
        <div class="space-y-6">
            <div class="flex justify-between items-center mb-6">
                <div>
                    <h3 class="text-xl font-bold text-on-surface">"CMS Pages"</h3>
                    <p class="text-sm text-on-surface-variant">"Manage dynamic content blocks for landing pages."</p>
                </div>
                <button 
                    on:click=move |_| on_new(())
                    class="bg-primary text-on-primary px-4 py-2 font-bold uppercase tracking-widest text-xs rounded hover:bg-primary/90 transition-colors">
                    "New Page"
                </button>
            </div>

            <div class="bg-surface-container overflow-hidden border border-outline-variant/30 hidden md:block">
                <table class="w-full text-left border-collapse">
                    <thead>
                        <tr class="bg-surface-container-high border-b border-outline-variant/30 text-xs tracking-wider uppercase text-on-surface-variant jetbrains">
                            <th class="px-6 py-4 font-medium">"Slug"</th>
                            <th class="px-6 py-4 font-medium">"Title"</th>
                            <th class="px-6 py-4 font-medium">"Status"</th>
                            <th class="px-6 py-4 font-medium text-right">"Actions"</th>
                        </tr>
                    </thead>
                    <tbody class="divide-y divide-outline-variant/30">
                        <tr class="hover:bg-surface-container-high/50 transition-colors">
                            <td class="px-6 py-4 jetbrains text-xs text-primary font-bold">"home"</td>
                            <td class="px-6 py-4 text-sm font-medium">"Homepage"</td>
                            <td class="px-6 py-4">
                                <span class="bg-primary/10 text-primary px-2 py-1 text-xs font-bold rounded">"Published"</span>
                            </td>
                            <td class="px-6 py-4 text-right">
                                <button on:click=move |_| on_edit("home".to_string()) class="text-primary hover:underline text-xs jetbrains font-bold uppercase tracking-widest mr-4">"EDIT"</button>
                            </td>
                        </tr>
                    </tbody>
                </table>
            </div>
        </div>
    }
}
