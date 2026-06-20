use leptos::prelude::*;

/// A single pill+label pair for a FilterBar.
#[derive(Clone)]
pub struct PillOption {
    pub value: String,
    pub label: String,
}

impl PillOption {
    pub fn new(value: &str, label: &str) -> Self {
        Self { value: value.to_string(), label: label.to_string() }
    }
}

/// Reusable filter bar: pill group + search input.
/// `active` and `search` are owned signals passed in from the parent tab.
#[component]
pub fn FilterBar(
    pills: Vec<PillOption>,
    active: RwSignal<String>,
    search: RwSignal<String>,
    #[prop(into)] search_placeholder: String,
) -> impl IntoView {
    view! {
        <div class="filter-bar">
            <div class="stage-pills">
                {pills.into_iter().map(|opt| {
                    let val = opt.value.clone();
                    let val_click = val.clone();
                    view! {
                        <button
                            class=move || format!("pill {}", if active.get() == val { "active" } else { "" })
                            on:click=move |_| active.set(val_click.clone())
                        >
                            {opt.label}
                        </button>
                    }
                }).collect_view()}
            </div>
            <div class="filter-sep"></div>
            <div class="filter-search">
                <input
                    type="text"
                    placeholder=search_placeholder
                    prop:value=move || search.get()
                    on:input=move |e| search.set(event_target_value(&e))
                />
            </div>
        </div>
    }
}
