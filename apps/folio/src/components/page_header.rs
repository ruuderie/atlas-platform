use leptos::prelude::*;

/// Stitch Landlord Portal page header — medium weight display title + muted subtitle.
#[component]
pub fn PageHeader(
    #[prop(into)] title: Signal<String>,
    #[prop(optional, into)] subtitle: Option<Signal<String>>,
    #[prop(optional)] children: Option<Children>,
) -> impl IntoView {
    view! {
        <div class="folio-page-header">
            <div class="folio-page-header__text">
                <h1 class="folio-page-header__title">{move || title.get()}</h1>
                {subtitle.map(|sub| {
                    view! { <p class="folio-page-header__subtitle">{move || sub.get()}</p> }
                })}
            </div>
            {children.map(|c| {
                view! { <div class="folio-page-header__actions">{c()}</div> }
            })}
        </div>
    }
}
