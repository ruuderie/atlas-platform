/// Typed representation of a Leptos `Resource::get()` or `LocalResource::get()` call.
///
/// # Why this exists
///
/// Leptos resources return `Option<Result<T, E>>` (or `Option<T>` for `LocalResource`).
/// Matching against these directly with a wildcard `_ =>` arm conflates two distinct states:
/// - `None` (still loading — the resource is pending)
/// - `Some(Err(e))` (loaded, but failed)
///
/// When the `None` state is caught by `_ =>` and mapped to an explicit error view, the
/// Leptos SSR streaming engine sees a concrete DOM node emitted while `<Suspense>` is still
/// pending. The client WASM then tries to hydrate that node against the `<Suspense>` fallback
/// wrapper — causing a silent structural mismatch that kills all event listeners on the page.
///
/// `ResourceState` forces exhaustive matching of all three states, eliminating this footgun.
///
/// # Usage (with `Resource::new` — returns `Option<Result<T, E>>`)
///
/// ```rust,ignore
/// use shared_ui::utils::ResourceState;
///
/// // In your component:
/// {move || match ResourceState::from(my_resource.get()) {
///     ResourceState::Loading => view! { <div class="hidden"></div> }.into_any(),
///     ResourceState::Error(_) => view! { <div class="text-error">"Failed to load"</div> }.into_any(),
///     ResourceState::Ready(data) => view! { <MyComponent data=data/> }.into_any(),
/// }}
/// ```
///
/// # Usage (with `LocalResource::new` — returns `Option<T>`)
///
/// ```rust,ignore
/// use shared_ui::utils::ResourceState;
///
/// {move || match ResourceState::from_option(my_local_resource.get()) {
///     ResourceState::Loading => view! { <div class="hidden"></div> }.into_any(),
///     ResourceState::Ready(data) => view! { <MyComponent data=data/> }.into_any(),
///     ResourceState::Error(_) => unreachable!(), // Infallible
/// }}
/// ```
///
/// # Loading arm shape
///
/// Always return an inert, structurally stable view for `Loading`:
/// - Inside `<tbody>`: `<tr class="hidden"></tr>`
/// - Inside `<select>`: `<option class="hidden" disabled=true></option>`
/// - Everywhere else: `<div class="hidden"></div>`
///
/// Do NOT duplicate the `<Suspense>` or `<Transition>` fallback view — that causes the
/// exact structural mismatch this enum is designed to prevent.
#[derive(Clone, Debug, PartialEq)]
pub enum ResourceState<T, E> {
    /// The resource is still pending (`.get()` returned `None`).
    /// Render an inert hidden element — let `<Suspense>` handle the visible loading UI.
    Loading,
    /// The resource resolved with an error.
    Error(E),
    /// The resource resolved successfully.
    Ready(T),
}

impl<T, E> From<Option<Result<T, E>>> for ResourceState<T, E> {
    fn from(opt: Option<Result<T, E>>) -> Self {
        match opt {
            None => ResourceState::Loading,
            Some(Err(e)) => ResourceState::Error(e),
            Some(Ok(v)) => ResourceState::Ready(v),
        }
    }
}

impl<T> ResourceState<T, std::convert::Infallible> {
    /// Construct a `ResourceState` from a `LocalResource::get()` result (`Option<T>`).
    ///
    /// `LocalResource` swallows errors at the fetch site (via `.unwrap_or_default()`),
    /// so the only two states are `Loading` (None) and `Ready(T)` (Some).
    pub fn from_option(opt: Option<T>) -> Self {
        match opt {
            None => ResourceState::Loading,
            Some(v) => ResourceState::Ready(v),
        }
    }
}
