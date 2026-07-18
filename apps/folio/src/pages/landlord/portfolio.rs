//! Portfolio route — `/l/portfolio`
//!
//! Backend G-portfolio remains the tenancy/isolation boundary. Operator UX
//! treats Assets as the property list, so this path redirects there.

use leptos::prelude::*;
use leptos_router::components::Redirect;

use crate::components::nav::FolioRoute;

#[component]
pub fn Portfolio() -> impl IntoView {
    view! { <Redirect path=FolioRoute::LandlordAssets.path()/> }
}
