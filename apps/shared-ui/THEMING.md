# shared-ui — Theming Architecture

> **For engineers new to this codebase**: Read this before creating any new `shared-ui` component or modifying styles in a consuming app.

---

## The Two-Layer Model

The Atlas Platform uses a **two-layer theming system**:

```
Layer 1 (App-Specific)     Layer 2 (shared-ui)
─────────────────────      ──────────────────
CSS Custom Properties  →   Tailwind Semantic Tokens  →  Components
(:root vars)               (bg-primary, bg-card, etc.)   (Button, Card, etc.)
```

### Layer 1 — CSS Custom Properties (the source of truth)

Each consuming app defines its design tokens as CSS custom properties in its root stylesheet. These are the **authoritative design values**:

```css
/* Example: apps/platform-admin/style/index.css */
:root {
  --cobalt: #2079f7;          /* primary action color */
  --bg-elevated: #05183c;     /* card / section background */
  --text-primary: #dee5ff;    /* primary text */
  --border-default: #2b4680;  /* card borders */
  /* ... etc */
}
```

### Layer 2 — Tailwind Semantic Tokens (the bridge)

Each consuming app's `tailwind.config.js` maps Tailwind's semantic token names to CSS vars, **not** to hardcoded hex values:

```js
// apps/platform-admin/tailwind.config.js
theme: {
  extend: {
    colors: {
      primary: {
        DEFAULT: 'var(--color-primary)',          // ← CSS var, not #2079f7
        foreground: 'var(--color-primary-foreground)',
      },
      card: {
        DEFAULT: 'var(--color-card)',
        foreground: 'var(--color-card-foreground)',
      },
      // ...
    }
  }
}
```

And the CSS bridge vars are added to the app's root CSS:

```css
/* shared-ui token bridge — added to each app's root CSS */
:root {
  --color-primary:             var(--cobalt);
  --color-primary-foreground:  #ffffff;
  --color-card:                var(--bg-elevated);
  --color-card-foreground:     var(--text-primary);
  --color-muted:               var(--bg-base);
  --color-muted-foreground:    var(--text-muted);
  /* ... etc */
}
```

**Result**: A `Button` component with `bg-primary` in its class string automatically renders in `var(--cobalt)` in platform-admin, or whatever `--color-primary` maps to in `anchor` or `network-instance`.

---

## ThemeProvider

The `ThemeProvider` component (`components/theme_provider.rs`) dynamically injects CSS vars at runtime. It is designed for **per-tenant theming** — when a logged-in tenant has a custom primary color, `ThemeProvider` overrides `--color-primary` live.

```rust
// In your app's root layout:
<ThemeProvider primary_color=brand_color_signal>
    <App/>
</ThemeProvider>
```

`ThemeProvider` sets `--color-primary` and `--brand-primary`. All other tokens come from the static CSS bridge in Layer 1.

---

## Rules for New Components

### ✅ DO — Use Tailwind Semantic Tokens

```rust
// Good: uses Tailwind semantic tokens that resolve via CSS vars
clx! {MyCard, div, "bg-card text-card-foreground border border-border rounded-xl p-4"}

// Good: direct CSS var reference when no Tailwind token exists
clx! {StatusBadge, span, "data-[status=active]:bg-[var(--green)] data-[status=error]:bg-[var(--red)]"}
```

### ❌ DON'T — Hardcode Colors

```rust
// Bad: hardcoded hex that won't adapt to different apps
clx! {MyCard, div, "bg-[#05183c] text-[#dee5ff] border border-[#2b4680] rounded-xl p-4"}

// Bad: hardcoded Tailwind palette color (can't be overridden)
clx! {MyCard, div, "bg-blue-900 text-blue-100 rounded-xl p-4"}
```

### Tailwind Semantic Token Reference

| Tailwind class | Maps to CSS var | Use case |
|---|---|---|
| `bg-primary` | `--color-primary` | Primary action backgrounds |
| `text-primary-foreground` | `--color-primary-foreground` | Text on primary bg |
| `bg-card` | `--color-card` | Card / panel backgrounds |
| `text-card-foreground` | `--color-card-foreground` | Card text |
| `bg-background` | `--color-background` | Page background |
| `text-foreground` | `--color-foreground` | Default body text |
| `bg-muted` | `--color-muted` | Subtle fill |
| `text-muted-foreground` | `--color-muted-foreground` | Dim / secondary text |
| `bg-accent` | `--color-accent` | Hover states, selected |
| `text-accent-foreground` | `--color-accent-foreground` | Text on accent |
| `bg-destructive` | `--color-destructive` | Danger/error backgrounds |
| `text-destructive` | `--color-destructive` | Danger text |
| `border-border` | `--color-border` | Standard borders |
| `border-input` | `--color-input` | Form input borders |
| `ring-ring` | `--color-ring` | Focus rings |
| `bg-popover` | `--color-popover` | Dropdown / tooltip backgrounds |
| `text-popover-foreground` | `--color-popover-foreground` | Text in popovers |

---

## Adding a New Consuming App

When a new app wants to use `shared-ui` components, it must:

1. **Ensure `shared-ui` is a Cargo dependency**
2. **Add the CSS var bridge to its root stylesheet** (copy from `platform-admin/style/index.css` section `/* shared-ui token bridge */`)
3. **Update `tailwind.config.js`** to map Tailwind tokens to CSS vars (not hex values)
4. **Include `shared-ui` sources in `tailwind.config.js` content scanning**:
   ```js
   content: ["./src/**/*.rs", "../shared-ui/src/**/*.rs", "./index.html"]
   ```

---

## Consuming App Status

| App | Tailwind config | CSS var bridge | ThemeProvider | Status |
|---|---|---|---|---|
| `platform-admin` | ✅ (being migrated to vars) | ✅ | ✅ | In progress |
| `anchor` | ⚠️ (hardcoded hex) | ❌ | ❌ | Needs migration |
| `network-instance` | ⚠️ (hardcoded hex) | ❌ | ❌ | Needs migration |

---

## FAQ

**Q: Can I just use inline styles for a one-off color?**
A: Only for semantic status colors not in the Tailwind token map (e.g. `style="color:var(--amber)"`). Never use hardcoded hex inline.

**Q: What about dark mode?**
A: Dark mode is handled entirely by the `:root` CSS vars in the consuming app. `shared-ui` components use `dark:` Tailwind variants only for extreme contrast overrides (e.g. `dark:bg-destructive/60`) and only when the var-based approach can't handle it.

**Q: My new component needs a color that's not in the token table above.**
A: First check if the color logically maps to an existing semantic token. If not, add a new CSS var to the bridge (e.g. `--color-success`) and add it to the Tailwind config in all consuming apps. Document it in this file.
