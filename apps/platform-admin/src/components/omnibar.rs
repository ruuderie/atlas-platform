use leptos::prelude::*;
use leptos::ev;
use ev::KeyboardEvent;
use crate::api::search::search_global;
use crate::api::models::SearchResult;
use uuid::Uuid;

#[component]
pub fn Omnibar() -> impl IntoView {
    let (is_open, set_is_open) = signal(false);
    let (query, set_query) = signal(String::new());
    let (results, set_results) = signal(Vec::<SearchResult>::new());
    let (is_loading, set_is_loading) = signal(false);
    let active_network = use_context::<ReadSignal<Option<Uuid>>>().expect("active network context");

    let input_ref = NodeRef::<leptos::html::Input>::new();

    window_event_listener(ev::keydown, move |e: KeyboardEvent| {
        if (e.meta_key() || e.ctrl_key()) && e.key() == "k" {
            e.prevent_default();
            set_is_open.update(|open| *open = !*open);
            if is_open.get() {
                if let Some(input) = input_ref.get() {
                    let _ = input.focus();
                }
            }
        }
        if e.key() == "Escape" && is_open.get() {
            set_is_open.set(false);
            set_query.set(String::new());
        }
    });

    Effect::new(move |_| {
        let q = query.get();
        let tid = active_network.get();
        if q.trim().is_empty() {
            set_results.set(Vec::new());
            return;
        }

        set_is_loading.set(true);
        leptos::task::spawn_local(async move {
            if let Ok(res) = search_global(&q, tid).await {
                set_results.set(res);
            }
            set_is_loading.set(false);
        });
    });

    let map_entity_url = |entity_type: &str, entity_id: &Uuid| -> String {
        match entity_type {
            "User" => format!("/admins"),
            "Listing" => format!("/network/listings/{}", entity_id),
            "Deal" => format!("/crm/deal/{}", entity_id),
            "Network" => format!("/apps/{}", entity_id),
            _ => "#".to_string()
        }
    };

    let map_entity_icon = |entity_type: &str| -> &'static str {
        match entity_type {
            "User" => "person",
            "Listing" => "store",
            "Deal" => "handshake",
            "Network" => "schema",
            _ => "search"
        }
    };

    view! {
        <Show when=move || is_open.get()>
            <div class="fixed inset-0 z-[99999] flex items-start justify-center pt-[10vh] px-4"
                 on:click=move |_| set_is_open.set(false)
            >
                <div class="absolute inset-0 bg-black/60 backdrop-blur-sm"></div>
                
                <div class="relative w-full max-w-2xl bg-[#06122d]/90 backdrop-blur-xl border border-[#7bd0ff]/20 rounded-2xl shadow-2xl shadow-black/50 overflow-hidden flex flex-col"
                     on:click=move |e| e.stop_propagation()
                >
                    <div class="relative flex items-center px-4 py-4 border-b border-outline-variant/10">
                        <span class="material-symbols-outlined text-[#7bd0ff] text-2xl mr-3">"search"</span>
                        <input
                            node_ref=input_ref
                            type="text"
                            placeholder="Search everything..."
                            class="flex-1 bg-transparent text-xl text-[#dee5ff] outline-none placeholder:text-[#91aaeb]/50"
                            prop:value=move || query.get()
                            on:input=move |ev| set_query.set(event_target_value(&ev))
                        />
                        <button class="ml-3 px-2 py-1 bg-white/5 rounded text-xs text-[#91aaeb] hover:bg-white/10" on:click=move |_| set_is_open.set(false)>
                            "ESC"
                        </button>
                    </div>

                    <div class="max-h-[60vh] overflow-y-auto min-h-[100px]">
                        <Show when=move || is_loading.get() && results.get().is_empty()>
                            <div class="p-8 text-center text-[#91aaeb]">"Searching..."</div>
                        </Show>
                        <Show when=move || !is_loading.get() && !query.get().is_empty() && results.get().is_empty()>
                            <div class="p-8 text-center text-[#91aaeb]">"No results found for '" {move || query.get()} "'"</div>
                        </Show>
                        <div class="py-2">
                            <For
                                each=move || results.get()
                                key=|r| r.id.to_string()
                                children=move |res| {
                                    let url = map_entity_url(&res.entity_type, &res.entity_id);
                                    let icon = map_entity_icon(&res.entity_type);
                                    let title = res.metadata.get("title").and_then(|v: &serde_json::Value| v.as_str()).unwrap_or("Unknown").to_string();
                                    let subtitle = res.metadata.get("subtitle").and_then(|v: &serde_json::Value| v.as_str()).unwrap_or(&res.entity_type).to_string();
                                    
                                    view! {
                                        <a href=url class="flex items-center px-4 py-3 hover:bg-[#7bd0ff]/10 transition-colors group" on:click=move |_| set_is_open.set(false)>
                                            <div class="w-10 h-10 rounded-lg bg-[#002867]/30 flex items-center justify-center mr-4 group-hover:bg-[#7bd0ff]/20">
                                                <span class="material-symbols-outlined text-[#91aaeb] group-hover:text-[#7bd0ff]">{icon}</span>
                                            </div>
                                            <div class="flex-1">
                                                <div class="text-[#dee5ff] font-medium">{title}</div>
                                                <div class="text-[#91aaeb] text-sm">{subtitle}</div>
                                            </div>
                                            <span class="material-symbols-outlined text-[#91aaeb]/30 group-hover:text-[#7bd0ff] transition-colors">"chevron_right"</span>
                                        </a>
                                    }
                                }
                            />
                        </div>
                    </div>
                </div>
            </div>
        </Show>
    }
}
