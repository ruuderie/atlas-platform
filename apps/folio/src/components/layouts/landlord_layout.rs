use leptos::prelude::*;
use leptos_router::components::Outlet;
use crate::components::nav::{SidebarNav, LANDLORD_NAV};
use crate::auth::SessionInfo;

/// Persistent shell for all /l/** landlord routes.
/// Nav items are driven by `LANDLORD_NAV` in `components/nav.rs`.
/// To add/remove/rename nav items, edit that file only.
#[component]
pub fn LandlordLayout() -> impl IntoView {
    let session = use_context::<Resource<Result<SessionInfo, server_fn::error::ServerFnError>>>()
        .expect("Session context missing");

    view! {
        <div class="folio-layout folio-layout--landlord">
            <Suspense fallback=|| view! { <div class="sidebar-loading"/> }>
                {move || session.get().map(|r| {
                    let (name, initials) = match r {
                        Ok(ref info) => (
                            info.display_name.clone(),
                            info.display_name.as_deref().map(user_initials),
                        ),
                        Err(_) => (None, None),
                    };
                    view! {
                        <SidebarNav
                            config=&LANDLORD_NAV
                            user_name=name
                            initials=initials
                        />
                    }
                })}
            </Suspense>
            <main class="folio-main">
                <Outlet/>
            </main>
        </div>
    }
}

fn user_initials(name: &str) -> String {
    name.split_whitespace()
        .filter_map(|w| w.chars().next())
        .take(2)
        .collect::<String>()
        .to_uppercase()
}
