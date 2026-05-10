# Leptos 0.8 SSR Shell Pattern

## Overview

Any Atlas app that uses **server-side rendering (SSR)** via `leptos_axum` **must** provide a
`shell()` function in its `main.rs`. Without it, `leptos_meta` panics on every request and the
pod serves 502s.

This document covers the pattern, why it's needed, how to debug it when it's missing, and the
difference between SSR and CSR apps in this platform.

---

## App Rendering Architectures

| App | Mode | Entry point pattern |
|---|---|---|
| `anchor` | SSR (leptos_axum) | `shell()` + `leptos_routes_with_context` |
| `network-instance` | SSR (leptos_axum) | `shell()` + `leptos_routes` |
| `platform-admin` | CSR (client-only) | `mount_to_body(App)` — no shell needed |

---

## The Shell Function

### What it does

The shell provides the outer HTML document that wraps every SSR response. `leptos_meta`
components (`<Title>`, `<Meta>`, `<Link>`, `<Stylesheet>`, etc.) inject into the `<head>` tag
at render time. Without a `</head>` tag present in the shell, `leptos_meta` panics:

```
thread 'tokio-rt-worker' panicked at leptos_meta-0.8.6/src/lib.rs:250:18:
you are using leptos_meta without a </head> tag
```

### Canonical pattern (copy this for any new SSR app)

```rust
#[cfg(feature = "ssr")]
pub fn shell(options: leptos::prelude::LeptosOptions) -> impl leptos::IntoView {
    use leptos::prelude::*;
    use leptos_meta::MetaTags;

    view! {
        <!DOCTYPE html>
        <html lang="en">
            <head>
                <meta charset="utf-8"/>
                <meta name="viewport" content="width=device-width, initial-scale=1"/>
                <AutoReload options=options.clone() />
                <HydrationScripts options />
                <MetaTags/>
            </head>
            <body>
                <App/>
            </body>
        </html>
    }
}
```

### Wiring it into the router

```rust
let app = Router::new()
    .route("/api/{*fn_name}",
        axum::routing::get(leptos_axum::handle_server_fns)
            .post(leptos_axum::handle_server_fns),
    )
    .leptos_routes_with_context(
        &app_state,
        routes,
        {
            let app_state = app_state.clone();
            move || provide_context(app_state.clone())
        },
        {
            let leptos_options = leptos_options.clone();
            move || shell(leptos_options.clone())   // ← shell goes here, not <App/>
        },
    )
    .fallback(leptos_axum::file_and_error_handler(shell))  // ← also here
    .with_state(app_state);
```

> [!IMPORTANT]
> In Leptos 0.7, `leptos_routes` took an app view closure (`move || view! { <App/> }`).
> In Leptos 0.8, it takes a **shell closure** instead. The `<App/>` goes **inside** the shell,
> not passed directly to the router. Passing `<App/>` directly is a silent migration bug —
> the code compiles, but `leptos_meta` panics at runtime.

---

## `leptos_meta` in the App component

The `app.rs` `<App/>` component calls `provide_meta_context()` and uses meta components like
`<Title>`, `<Meta>`, `<Html>`, `<Stylesheet>`, etc. These work by finding the `<head>` tag
that the shell provides and injecting into it during SSR.

Keep all `leptos_meta` usage inside `<App/>` (or its children) — not in the shell itself,
except for `<MetaTags/>` which is the injection point placeholder.

---

## Debugging: 502 from anchor-app

If `dev.buildwithruud.com` (or any SSR app) returns 502:

```bash
# 1. Check pods are actually running (not crash-looping)
kubectl get pods -n atlas-dev

# 2. Check logs for the panic signature
kubectl logs -n atlas-dev -l app=anchor-app --tail=20

# 3. If you see this panic → missing or broken shell function:
#    "you are using leptos_meta without a </head> tag"

# 4. If pods are running but site is empty → check site directory contents:
kubectl exec -n atlas-dev deployment/anchor-app -- sh -c "find /app/site -type f | sort"
#    Expected: /app/site/pkg/anchor.js, anchor.wasm, anchor.css
#    Missing index.html is normal — cargo-leptos 0.3.6 does NOT generate one.
#    The shell is provided at runtime by the Rust server, not as a static file.
```

### Known panic → root cause mapping

| Panic message | Root cause | Fix |
|---|---|---|
| `you are using leptos_meta without a </head> tag` | Missing `shell()` function, or router wired with `<App/>` directly instead of `shell()` | Add `shell()` and wire into `leptos_routes` |
| `Invalid static segment: {slug}` | Axum route using Leptos `path!` macro syntax instead of Axum syntax | Use `{slug}` in Axum routes, `:slug` in Leptos `path!()` macro |
| `you are reading a resource in hydrate mode outside a <Suspense/>` | `Resource::new` read during SSR without Suspense boundary | Switch to `LocalResource::new` or wrap in `<Suspense>` |

---

## cargo-leptos site directory

`site-root = "target/site"` in `Cargo.toml` resolves **relative to the workspace root**, not
the crate root. In the Docker build (WORKDIR `/app/apps/anchor`), the workspace root is
`/app/apps/`, so the site lands at:

```
/app/apps/target/site/       ← correct COPY source in Dockerfile
/app/apps/anchor/target/site ← WRONG — this does not exist
```

`cargo-leptos` copies `assets-dir` (the `public/` folder) into the site root and outputs
WASM/JS to `target/site/pkg/`. It does **not** write an `index.html` — the HTML shell is
rendered dynamically by the SSR server at request time.

> [!WARNING]
> Do not place an `index.html` in `public/` (the assets-dir). cargo-leptos 0.3.6+ explicitly
> rejects it: `Assets source anchor/public contains path anchor/public/index.html reserved for Leptos.`

---

## Adding a new SSR app

When creating a new app that uses `leptos_axum`:

1. Copy the `shell()` function pattern above into `main.rs`
2. Wire `shell` into `leptos_routes` / `leptos_routes_with_context` (5th argument)  
3. Add `.fallback(leptos_axum::file_and_error_handler(shell))`
4. Add the `/api/{*fn_name}` server function route
5. Verify the Dockerfile `COPY --from=builder` uses the correct workspace-level target path

See `apps/anchor/src/main.rs` and `apps/network-instance/src/main.rs` as reference implementations.
