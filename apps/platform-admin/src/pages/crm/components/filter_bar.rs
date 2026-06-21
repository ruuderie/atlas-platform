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

/// Reusable CRM filter bar: pill group + search input.
/// Critical layout styles are inlined so the bar renders correctly regardless
/// of external CSS loading state.
#[component]
pub fn FilterBar(
    pills: Vec<PillOption>,
    active: RwSignal<String>,
    search: RwSignal<String>,
    #[prop(into)] search_placeholder: String,
) -> impl IntoView {
    view! {
        <div
            class="crm-filter-bar"
            style="display:flex;align-items:center;gap:10px;padding:10px 14px;border-bottom:1px solid var(--border-default);background:var(--bg-surface);flex-shrink:0;flex-wrap:wrap;"
        >
            <div
                class="crm-pills"
                style="display:flex;gap:4px;flex-wrap:wrap;"
            >
                {pills.into_iter().map(|opt| {
                    let val_for_class = opt.value.clone();
                    let val_for_style = opt.value.clone();
                    let val_click     = opt.value.clone();
                    let label         = opt.label.clone();
                    view! {
                        <button
                            class=move || {
                                if active.get() == val_for_class { "crm-pill active" } else { "crm-pill" }
                            }
                            style=move || {
                                let base = "display:inline-flex;align-items:center;padding:4px 11px;border-radius:4px;border:1px solid;font-size:11px;font-weight:500;font-family:inherit;cursor:pointer;white-space:nowrap;transition:all 0.12s;";
                                if active.get() == val_for_style {
                                    format!("{}background:var(--cobalt-dim);border-color:var(--cobalt);color:var(--cobalt);font-weight:600;", base)
                                } else {
                                    format!("{}background:transparent;border-color:var(--border-default);color:var(--text-muted);", base)
                                }
                            }
                            on:click=move |_| active.set(val_click.clone())
                        >
                            {label}
                        </button>
                    }
                }).collect_view()}
            </div>
            <div
                class="crm-filter-sep"
                style="width:1px;height:20px;background:var(--border-default);flex-shrink:0;"
            ></div>
            <div
                class="crm-search"
                style="flex:1;min-width:180px;max-width:360px;"
            >
                <input
                    type="text"
                    placeholder=search_placeholder
                    prop:value=move || search.get()
                    on:input=move |e| search.set(event_target_value(&e))
                    style="width:100%;background:var(--bg-base);border:1px solid var(--border-strong);border-radius:5px;padding:6px 10px;font-size:12px;color:var(--text-primary);font-family:inherit;outline:none;"
                />
            </div>
        </div>
    }
}
