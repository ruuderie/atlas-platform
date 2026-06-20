use leptos::prelude::*;

/// A single KPI card item.
#[derive(Clone)]
pub struct KpiItem {
    pub label: String,
    pub value: String,
    pub sub: Option<String>,
    pub color: Option<String>, // e.g. "var(--green)"
}

impl KpiItem {
    pub fn new(label: &str, value: &str) -> Self {
        Self { label: label.to_string(), value: value.to_string(), sub: None, color: None }
    }
    pub fn sub(mut self, sub: &str) -> Self { self.sub = Some(sub.to_string()); self }
    pub fn color(mut self, color: &str) -> Self { self.color = Some(color.to_string()); self }
}

/// Horizontal KPI strip. `items` is a reactive closure so the strip re-renders
/// when the underlying resource changes.
#[component]
pub fn KpiStrip(
    #[prop(into)] items: Signal<Vec<KpiItem>>,
) -> impl IntoView {
    view! {
        <div class="kpi-row">
            {move || items.get().into_iter().map(|k| {
                let color_style = k.color.map(|c| format!("color:{}", c)).unwrap_or_default();
                view! {
                    <div class="kpi-card">
                        <span class="kpi-label">{k.label}</span>
                        <span class="kpi-value" style=color_style>{k.value}</span>
                        {k.sub.map(|s| view! { <span class="kpi-delta up">{s}</span> })}
                    </div>
                }
            }).collect_view()}
        </div>
    }
}
