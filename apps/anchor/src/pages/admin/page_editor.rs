use leptos::*;

#[component]
pub fn PageEditor(
    slug: String,
    #[prop(into)] on_cancel: Callback<()>,
    #[prop(into)] on_save: Callback<(String, String)>,
) -> impl IntoView {
    let (current_slug, set_current_slug) = create_signal(slug.clone());
    let (payload, set_payload) = create_signal(String::from("[\n  \n]"));

    view! {
        <div class="space-y-6">
            <div class="flex justify-between items-center mb-6 border-b border-outline-variant/30 pb-4">
                <div>
                    <h3 class="text-xl font-bold text-on-surface">
                        {if slug.is_empty() { "Create New Page".to_string() } else { format!("Edit Page: {}", slug) }}
                    </h3>
                </div>
                <div class="space-x-4">
                    <button 
                        on:click=move |_| on_cancel(())
                        class="text-outline hover:text-on-surface text-xs jetbrains font-bold uppercase tracking-widest transition-colors">
                        "Cancel"
                    </button>
                    <button 
                        on:click=move |_| {
                            on_save((current_slug.get(), payload.get()));
                        }
                        class="bg-primary text-on-primary px-6 py-2 font-bold uppercase tracking-widest text-xs rounded hover:bg-primary/90 transition-colors">
                        "Save"
                    </button>
                </div>
            </div>

            <div class="space-y-4">
                <div>
                    <label class="block text-xs jetbrains text-on-surface-variant mb-2">"Page Slug"</label>
                    <input 
                        type="text" 
                        prop:value=current_slug
                        on:input=move |ev| set_current_slug.set(event_target_value(&ev))
                        class="w-full bg-surface-container border border-outline-variant/30 px-4 py-2 text-on-surface font-mono text-sm focus:border-primary focus:outline-none"
                        placeholder="e.g. about-us"
                    />
                </div>
                
                <div>
                    <label class="block text-xs jetbrains text-on-surface-variant mb-2">"Blocks JSON Payload"</label>
                    <textarea 
                        prop:value=payload
                        on:input=move |ev| set_payload.set(event_target_value(&ev))
                        class="w-full h-96 bg-surface-container border border-outline-variant/30 px-4 py-4 text-on-surface font-mono text-sm focus:border-primary focus:outline-none"
                        placeholder="[ { \"Hero\": ... } ]"
                    ></textarea>
                    <p class="text-xs text-outline mt-2">"Must be a valid JSON array matching the DynamicBlock schema."</p>
                </div>
            </div>
        </div>
    }
}
