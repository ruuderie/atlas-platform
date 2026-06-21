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

/// Compact inline KPI strip.
/// Critical layout is inlined on every element so the strip renders correctly
/// even when the external CSS file is cached/stale.
#[component]
pub fn KpiStrip(
    #[prop(into)] items: Signal<Vec<KpiItem>>,
) -> impl IntoView {
    view! {
        <div
            class="crm-kpi-strip"
            style="display:flex;overflow-x:auto;flex-shrink:0;border-bottom:1px solid var(--border-default);background:var(--bg-surface);"
        >
            {move || items.get().into_iter().enumerate().map(|(i, k)| {
                let is_last = {
                    let total = items.get().len();
                    i == total.saturating_sub(1)
                };
                let value_color = k.color.clone().unwrap_or_else(|| "var(--text-primary)".to_string());
                let border_right = if is_last { "none" } else { "1px solid var(--border-default)" };
                let item_style = format!(
                    "display:flex;flex-direction:column;gap:2px;padding:11px 20px;min-width:110px;flex-shrink:0;border-right:{};",
                    border_right
                );
                let value_style = format!(
                    "font-size:20px;font-weight:700;letter-spacing:-0.5px;font-variant-numeric:tabular-nums;line-height:1.1;color:{};",
                    value_color
                );
                view! {
                    <div class="crm-kpi" style=item_style>
                        <span
                            class="crm-kpi-label"
                            style="font-size:9.5px;font-weight:600;text-transform:uppercase;letter-spacing:0.08em;color:var(--text-muted);"
                        >{k.label}</span>
                        <span class="crm-kpi-value" style=value_style>{k.value}</span>
                        {k.sub.map(|s| view! {
                            <span
                                class="crm-kpi-sub"
                                style="font-size:10px;color:var(--text-muted);"
                            >{s}</span>
                        })}
                    </div>
                }
            }).collect_view()}
        </div>
    }
}
