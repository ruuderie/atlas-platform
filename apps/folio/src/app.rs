use leptos::prelude::*;
use leptos_meta::*;
use leptos_router::components::{Redirect, Route, Router, Routes};
use leptos_router::path;

use crate::auth::check_session;
use crate::components::nav::Nav;
use crate::pages::{
    dashboard::Dashboard,
    login::Login,
    not_found::NotFound,
    portfolio::Portfolio,
    leads::Leads,
    leases::Leases,
    reservations::Reservations,
};

#[component]
pub fn App() -> impl IntoView {
    provide_meta_context();

    view! {
        <Router>
            <Routes fallback=|| view! { <NotFound/> }>
                // Public routes
                <Route path=path!("/login")    view=Login/>
                <Route path=path!("/verify")   view=crate::pages::verify::Verify/>

                // Protected dashboard routes — auth guard applied inside each page
                <Route path=path!("/")         view=move || view! { <Redirect path="/dashboard"/> }/>
                <Route path=path!("/dashboard")           view=AuthShell>
                    <Route path=path!("")                 view=Dashboard/>
                    <Route path=path!("/portfolio")       view=Portfolio/>
                    <Route path=path!("/leads")           view=Leads/>
                    <Route path=path!("/leases")          view=Leases/>
                    <Route path=path!("/reservations")    view=Reservations/>
                </Route>
            </Routes>
        </Router>
    }
}

/// Wrapper that checks session before rendering child routes.
/// Redirects to /login if not authenticated.
#[component]
fn AuthShell() -> impl IntoView {
    let session = Resource::new(|| (), |_| check_session());

    view! {
        <Suspense fallback=|| view! { <div class="loading-screen">"Authenticating…"</div> }>
            {move || {
                session.get().map(|result| match result {
                    Ok(_info) => view! {
                        <div class="folio-layout">
                            <Nav/>
                            <main class="folio-main">
                                <leptos_router::components::Outlet/>
                            </main>
                        </div>
                    }.into_any(),
                    Err(_) => view! {
                        <Redirect path="/login"/>
                    }.into_any(),
                })
            }}
        </Suspense>
    }
}
