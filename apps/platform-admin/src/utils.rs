use std::collections::BTreeMap;
use crate::api::models::PlatformAppModel;

/// Groups a flat list of PlatformAppModels by their `tenant_id`.
/// Returns a BTreeMap where:
/// - Key is the `tenant_id`.
/// - Value is a tuple containing the `tenant_name` (extracted from the first seen app)
///   and the `Vec<PlatformAppModel>` natively bound to that tenant.
pub fn group_apps_by_tenant(
    apps: Vec<PlatformAppModel>,
) -> BTreeMap<String, (String, Vec<PlatformAppModel>)> {
    let mut grouped = BTreeMap::new();
    for app in apps {
        let entry = grouped
            .entry(app.tenant_id.clone())
            .or_insert_with(|| (app.name.clone(), Vec::new()));
        entry.1.push(app);
    }
    grouped
}

// Build: 2026-06-24T23:57:05Z

/// Renders a subtle loading pulse row — use in place of a blank table or empty list
/// while a `LocalResource` is still in flight.
///
/// # Example
/// ```rust
/// {move || match data_res.get() {
///     None => inline_loading("Loading clients..."),
///     Some(Err(e)) => inline_error(&e),
///     Some(Ok(data)) => /* real view */,
/// }}
/// ```
pub fn inline_loading(msg: &str) -> leptos::prelude::AnyView {
    use leptos::prelude::*;
    let msg = msg.to_string();
    view! {
        <div class="flex items-center gap-2 px-4 py-6 text-on-surface-variant text-xs animate-pulse">
            <svg class="w-3.5 h-3.5 shrink-0 opacity-50" viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.5">
                <path d="M8 2v2M8 12v2M2 8h2M12 8h2M3.5 3.5l1.4 1.4M11.1 11.1l1.4 1.4M3.5 12.5l1.4-1.4M11.1 4.9l1.4-1.4"/>
            </svg>
            {msg}
        </div>
    }.into_any()
}

/// Renders a dismissible error banner — use when a `LocalResource` returns `Err(_)`.
/// Shows the error message and a subtle retry cue.
pub fn inline_error(msg: &str) -> leptos::prelude::AnyView {
    use leptos::prelude::*;
    let msg = msg.to_string();
    view! {
        <div class="flex items-start gap-2.5 px-4 py-3 m-2 rounded-lg bg-error/8 border border-error/20 text-xs text-error">
            <svg class="w-3.5 h-3.5 shrink-0 mt-0.5" viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.5">
                <circle cx="8" cy="8" r="6"/><line x1="8" y1="5" x2="8" y2="8.5"/><circle cx="8" cy="11" r="0.5" fill="currentColor"/>
            </svg>
            <span>{msg} " — refresh the page or check the backend."</span>
        </div>
    }.into_any()
}
