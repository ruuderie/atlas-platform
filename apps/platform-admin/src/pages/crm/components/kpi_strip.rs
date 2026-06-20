use leptos::prelude::*;

/// A single KPI item for the compact CRM strip.
#[derive(Clone)]
pub struct KpiItem {
    pub label: String,
    pub value: String,
    pub sub: Option<String>,
    pub color: Option<String>,
}

impl KpiItem {
    pub fn new(label: &str, value: &str) -> Self {
        Self { label: label.to_string(), value: value.to_string(), sub: None, color: None }
    }
    pub fn sub(mut self, sub: &str) -> Self { self.sub = Some(sub.to_string()); self }
    pub fn color(mut self, color: &str) -> Self { self.color = Some(color.to_string()); self }
}

/// Compact inline KPI strip that matches the CRM detail page design.
/// Uses `.crm-kpi-strip` → `.crm-kpi` layout — NOT the dashboard card layout.
#[component]
pub fn KpiStrip(
    #[prop(into)] items: Signal<Vec<KpiItem>>,
) -> impl IntoView {
    view! {
        <div class="crm-kpi-strip">
            {move || items.get().into_iter().map(|k| {
                let value_style = k.color
                    .map(|c| format!("color:{}", c))
                    .unwrap_or_default();
                view! {
                    <div class="crm-kpi">
                        <span class="crm-kpi-label">{k.label}</span>
                        <span class="crm-kpi-value" style=value_style>{k.value}</span>
                        {k.sub.map(|s| view! { <span class="crm-kpi-sub">{s}</span> })}
                    </div>
                }
            }).collect_view()}
        </div>
    }
}
