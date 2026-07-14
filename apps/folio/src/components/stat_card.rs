use leptos::prelude::*;
use leptos_router::components::A;

/// Stitch-style KPI card — micro-label, tabular value, Material Symbol.
#[component]
pub fn StatCard(
    label: &'static str,
    value: Signal<String>,
    icon: &'static str,
    #[prop(optional)] href: Option<&'static str>,
) -> impl IntoView {
    let body = view! {
        <span class="material-symbols-outlined folio-stat-card__icon">{icon}</span>
        <div class="folio-stat-card__body">
            <p class="folio-stat-card__label">{label}</p>
            <p class="folio-stat-card__value">{move || value.get()}</p>
        </div>
    };

    match href {
        Some(path) => view! {
            <A href=path attr:class="folio-stat-card folio-stat-card--link">
                {body}
            </A>
        }
        .into_any(),
        None => view! {
            <div class="folio-stat-card">
                {body}
            </div>
        }
        .into_any(),
    }
}
