use leptos::prelude::*;
use leptos_router::components::Outlet;
use crate::components::nav::{SidebarNav, TENANT_NAV};
use crate::auth::SessionInfo;

/// Persistent shell for all /t/** tenant routes.
/// Nav items driven by `TENANT_NAV` in `components/nav.rs`.
#[component]
pub fn TenantLayout() -> impl IntoView {
    let session = use_context::<Resource<Result<SessionInfo, server_fn::error::ServerFnError>>>()
        .expect("Session context missing");

    view! {
        <div class="folio-layout folio-layout--tenant">
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
                        <SidebarNav config=&TENANT_NAV user_name=name initials=initials/>
                    }
                })}
            </Suspense>
            <main class="folio-main"><Outlet/></main>
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
