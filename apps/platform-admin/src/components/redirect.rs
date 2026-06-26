use leptos::prelude::*;

/// Imperatively navigates to `to` when mounted. Used to retire old routes
/// without breaking existing bookmarks or deep links.
///
/// ```rust
/// // In your router:
/// <Route path=path!("/old-path") view=|| view! {
///     <Redirect to="/new-path" />
/// } />
/// ```
#[component]
pub fn Redirect(
    /// The URL to redirect to. Must be an absolute path (e.g. "/products").
    to: &'static str,
) -> impl IntoView {
    Effect::new(move |_| {
        if let Some(window) = web_sys::window() {
            let _ = window.location().set_href(to);
        }
    });

    view! {
        // Render nothing — the effect fires the redirect immediately.
        <></>
    }
}
