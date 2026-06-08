use leptos::prelude::*;

#[component]
pub fn Nav() -> impl IntoView {
    view! {
        <nav class="folio-nav">
            <div class="nav-brand">
                <span class="nav-logo">"Folio"</span>
            </div>
            <ul class="nav-links">
                <li><a href="/dashboard"           class="nav-link">"Overview"</a></li>
                <li><a href="/dashboard/portfolio" class="nav-link">"Portfolio"</a></li>
                <li><a href="/dashboard/leads"     class="nav-link">"Leads"</a></li>
                <li><a href="/dashboard/leases"    class="nav-link">"Leases"</a></li>
                <li><a href="/dashboard/reservations" class="nav-link">"Reservations"</a></li>
            </ul>
            <div class="nav-footer">
                <button
                    class="nav-logout"
                    on:click=move |_| {
                        leptos::spawn_local(async {
                            let _ = crate::auth::revoke_session().await;
                            let _ = web_sys::window()
                                .and_then(|w| w.location().href().ok())
                                .map(|_| {
                                    let _ = web_sys::window()
                                        .unwrap()
                                        .location()
                                        .set_href("/login");
                                });
                        });
                    }
                >
                    "Sign out"
                </button>
            </div>
        </nav>
    }
}
