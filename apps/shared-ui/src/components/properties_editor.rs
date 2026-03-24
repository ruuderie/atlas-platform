use leptos::prelude::*;
use serde_json::{Value, Map};

#[component]
pub fn PropertiesEditor(
    #[prop(into)] properties: RwSignal<Option<Value>>,
) -> impl IntoView {
    // Convert Value to Vec<(usize, String, String)> for easy iteration and reactivity
    let (pairs, set_pairs) = signal(Vec::<(usize, String, String)>::new());
    let (next_id, set_next_id) = signal(0usize);
    let (initialized, set_initialized) = signal(false);

    // Initialize state
    Effect::new(move |_| {
        // Only run initialization once
        if !initialized.get_untracked() {
            let props = properties.get().unwrap_or(Value::Object(Map::new()));
            if let Value::Object(map) = props {
                let mut new_pairs = Vec::new();
                let mut id = next_id.get_untracked();
                for (key, val) in map {
                    // simple string conversion for now
                    let val_str = match val {
                        Value::String(s) => s.clone(),
                        Value::Number(n) => n.to_string(),
                        Value::Bool(b) => b.to_string(),
                        _ => val.to_string(),
                    };
                    new_pairs.push((id, key.clone(), val_str));
                    id += 1;
                }
                set_pairs.set(new_pairs);
                set_next_id.set(id);
                set_initialized.set(true);
            }
        }
    });

    let sync_to_parent = move |current_pairs: &Vec<(usize, String, String)>| {
        let mut map = Map::new();
        for (_, k, v) in current_pairs {
            if !k.is_empty() {
                // parse value dynamically 
                let parsed_val = if v == "true" {
                    Value::Bool(true)
                } else if v == "false" {
                    Value::Bool(false)
                } else if let Ok(n) = v.parse::<serde_json::Number>() {
                    Value::Number(n)
                } else {
                    Value::String(v.clone())
                };
                map.insert(k.clone(), parsed_val);
            }
        }
        properties.set(Some(Value::Object(map)));
    };

    let add_field = move |_| {
        set_pairs.update(|p| {
            p.push((next_id.get(), "".to_string(), "".to_string()));
            sync_to_parent(p);
        });
        set_next_id.update(|id| *id += 1);
    };

    let remove_field = move |id_to_remove: usize| {
        set_pairs.update(|p| {
            p.retain(|(id, _, _)| *id != id_to_remove);
            sync_to_parent(p);
        });
    };

    let update_key = move |id_to_update: usize, new_key: String| {
        set_pairs.update(|p| {
            if let Some(pair) = p.iter_mut().find(|(id, _, _)| *id == id_to_update) {
                pair.1 = new_key;
            }
            sync_to_parent(p);
        });
    };

    let update_val = move |id_to_update: usize, new_val: String| {
        set_pairs.update(|p| {
            if let Some(pair) = p.iter_mut().find(|(id, _, _)| *id == id_to_update) {
                pair.2 = new_val;
            }
            sync_to_parent(p);
        });
    };

    view! {
        <div class="space-y-4">
            <div class="flex items-center justify-between">
                <h3 class="text-sm font-medium text-gray-900">"Custom Properties"</h3>
                <button 
                    type="button" 
                    on:click=add_field 
                    class="inline-flex items-center px-3 py-1.5 border border-transparent text-xs font-medium rounded-md shadow-sm text-white bg-indigo-600 hover:bg-indigo-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-indigo-500"
                >
                    <span class="material-symbols-outlined text-sm mr-1">"add"</span>
                    "Add Property"
                </button>
            </div>
            
            <div class="bg-gray-50 p-4 rounded-md border border-gray-200 space-y-3">
                <For
                    each=move || pairs.get()
                    key=|(id, _, _)| *id
                    children=move |(id, key, val)| {
                        view! {
                            <div class="flex items-center space-x-2">
                                <div class="w-1/3">
                                    <input 
                                        type="text" 
                                        class="block w-full rounded-md border-gray-300 shadow-sm focus:border-indigo-500 focus:ring-indigo-500 sm:text-sm p-2" 
                                        placeholder="Property Key"
                                        prop:value=key.clone()
                                        on:input=move |ev| update_key(id, event_target_value(&ev))
                                    />
                                </div>
                                <div class="w-1/2">
                                    <input 
                                        type="text" 
                                        class="block w-full rounded-md border-gray-300 shadow-sm focus:border-indigo-500 focus:ring-indigo-500 sm:text-sm p-2" 
                                        placeholder="Property Value"
                                        prop:value=val.clone()
                                        on:input=move |ev| update_val(id, event_target_value(&ev))
                                    />
                                </div>
                                <button 
                                    type="button" 
                                    on:click=move |_| remove_field(id)
                                    class="p-1 text-gray-400 hover:text-red-500 transition-colors"
                                >
                                    <span class="material-symbols-outlined text-lg">"delete"</span>
                                </button>
                            </div>
                        }
                    }
                />
                
                <Show when=move || pairs.get().is_empty()>
                    <div class="text-center py-4 text-sm text-gray-500">
                        "No custom properties defined. Click 'Add Property' to add one."
                    </div>
                </Show>
            </div>
        </div>
    }
}
