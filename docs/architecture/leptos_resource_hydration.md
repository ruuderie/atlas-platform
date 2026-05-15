# Leptos Resource Hydration & the `ResourceState` Pattern

## The Problem: The Swallowed `None` Footgun

In Leptos, `Resource::new()` runs an async task and exposes its state via `.get()`, which returns `Option<Result<T, E>>`. A very common but highly dangerous rendering pattern is:

```rust
// ❌ ANTI-PATTERN — DO NOT USE
<Suspense fallback=move || view! { <Spinner/> }>
    {move || match resource.get() {
        Some(Ok(data)) => view! { <Dashboard data=data/> }.into_any(),
        _ => view! { <div class="text-error">"Failed to load"</div> }.into_any()
    }}
</Suspense>
```

### Why this breaks SSR hydration

When a resource is pending, `.get()` returns `None`. The `_ =>` arm catches **both** `None` (loading) and `Some(Err(e))` (actual error).

During **SSR streaming**:
1. The closure evaluates. `.get()` returns `None`.
2. The `_ =>` branch executes and emits the explicit error view into the stream.
3. `<Suspense>` detects the pending resource and wraps the fallback (`<Spinner/>`) in the HTML response instead.

During **WASM hydration**:
1. The WASM starts. It evaluates the closure — still pending — and again returns the error view.
2. It tries to mount the error view onto the DOM.
3. The DOM contains `<Spinner/>`, not the error view.
4. **Hydration panic**: event listeners fail to attach silently. Nav tabs, buttons, modals — all dead.

---

## The Solution: `ResourceState<T, E>`

Located in `shared_ui::utils::ResourceState`.

```rust
use shared_ui::utils::ResourceState;

// ✅ REQUIRED PATTERN — exhaustive, hydration-safe
<Transition fallback=move || view! { <Spinner/> }>
    {move || match ResourceState::from(resource.get()) {
        ResourceState::Loading => view! { <div class="hidden"></div> }.into_any(),
        ResourceState::Error(_) => view! { <div class="text-error">"Failed to load"</div> }.into_any(),
        ResourceState::Ready(data) => view! { <Dashboard data=data/> }.into_any(),
    }}
</Transition>
```

### Why this works

By explicitly mapping `None → Loading`, the compiler forces you to handle it. The `Loading` arm returns an inert hidden element, which is structurally stable and won't conflict with what `<Suspense>` or `<Transition>` expects. There is no mismatch for the hydration walker.

---

## `<Suspense>` vs `<Transition>`

Both use `ResourceState` the same way. The behavioral difference is:

| | `<Suspense>` | `<Transition>` |
|---|---|---|
| First render (resource pending) | Shows fallback, blocks render | Shows fallback |
| Refetch (resource pending) | Shows fallback again | Shows **stale content** |

`ResourceState::Loading` fires only when `.get()` returns `None` — which happens on **first render** for `<Transition>` as well. The hidden `Loading` arm is safe in both cases since `<Transition>` shows the fallback, not the closure output, during that initial pending window.

---

## `LocalResource` — from_option()

`LocalResource::new()` returns `Option<T>` (not `Option<Result<T, E>>`). Errors are typically swallowed at the fetch site with `.unwrap_or_default()`. Use the `from_option` constructor:

```rust
use shared_ui::utils::ResourceState;

// LocalResource returns Option<T>, not Option<Result<T, E>>
let local_res = LocalResource::new(|| async move {
    fetch_something().await.unwrap_or_default()
});

{move || match ResourceState::from_option(local_res.get()) {
    ResourceState::Loading => view! { <div class="hidden"></div> }.into_any(),
    ResourceState::Ready(data) => view! { <MyComponent data=data/> }.into_any(),
    ResourceState::Error(_) => unreachable!(), // Infallible — LocalResource swallows errors
}}
```

> [!NOTE]
> `LocalResource` is CSR-only. It only executes after WASM hydration. There is no SSR hydration contract to worry about — the `Loading` arm is still good practice for consistent UI, but it won't cause a hydration panic if missing.

---

## `Loading` Arm Shape

Always return an inert, structurally stable view. Match the DOM context:

| Context | Correct `Loading` view |
|---------|----------------------|
| Inside `<tbody>` | `view! { <tr class="hidden"></tr> }` |
| Inside `<select>` | `view! { <option class="hidden" disabled=true></option> }` |
| General (div context) | `view! { <div class="hidden"></div> }` |
| Inline (span context) | `view! { <span class="hidden"></span> }` |

**Do NOT** duplicate the `<Suspense>` or `<Transition>` fallback. The fallback is what the user sees. The `Loading` arm is what the reactive closure emits — it must be invisible.

---

## Canonical Import

Always import from the utils root, not the full module path:

```rust
// ✅ Correct
use shared_ui::utils::ResourceState;

// ❌ Verbose — avoid
use shared_ui::utils::resource_state::ResourceState;
```

---

## Never Use This Pattern

```rust
// ❌ Wildcards on resource matches — causes hydration panics
match resource.get() {
    Some(Ok(data)) => { ... },
    _ => view! { <div>"error"</div> }.into_any()  // catches None (loading)!
}
```

The `grep` command to audit for remaining violations:
```bash
grep -rn '_ => view!' apps/anchor/src apps/network-instance/src apps/shared-ui/src
```
Only non-resource wildcards should remain (tab switches, prop tuple matches, etc.).
