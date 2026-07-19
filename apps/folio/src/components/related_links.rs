//! Compact “Related” row under page headers for hierarchical nesting.

use leptos::prelude::*;
use leptos_router::components::A;

use crate::components::nav::FolioRoute;

#[derive(Clone, Copy)]
pub struct RelatedLink {
    pub route: FolioRoute,
    pub label: &'static str,
}

#[component]
pub fn RelatedLinks(links: &'static [RelatedLink]) -> impl IntoView {
    view! {
        <nav class="folio-related" aria-label="Related">
            <span class="folio-related__label">"Related"</span>
            <ul class="folio-related__list">
                {links.iter().map(|link| {
                    view! {
                        <li>
                            <A href=link.route.path() attr:class="folio-related__link press">
                                {link.label}
                            </A>
                        </li>
                    }
                }).collect_view()}
            </ul>
        </nav>
    }
}
