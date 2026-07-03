# Platform Admin — Style Guide

> **Reference standard**: `apps/platform-admin/src/pages/dashboard.rs` (Command Center)
> **Token source**: `apps/platform-admin/style/index.css`

---

## 0. The Golden Rule

**Every component on every page must use a named design-system class from this guide.
Raw Tailwind utility strings are only permitted inside those named classes' own CSS definitions.
Never compose buttons, cards, inputs, modals, or tabs from scratch in a page file.**

---

## 1. Page Layout

Every **top-level route component** follows this shell:

```rust
view! {
    <div class="main-canvas">               // scroll container
        <div class="page-header">
            <div>
                <div class="page-title">"Page Name"</div>
                <div class="page-subtitle">"Brief description"</div>
            </div>
            <div class="page-actions">
                <button class="btn btn-primary">"+ Add"</button>
            </div>
        </div>

        <div class="kpi-row">               // optional — only if page has metrics
            <div class="kpi-card"> ... </div>
        </div>

        <div class="section"> ... </div>    // one or more content sections
    </div>
}
```

### `main-area` vs `main-canvas`

Use `main-area` only for pages that manage their own internal scroll (split-pane, full-height fixed-header tables).
Examples: `support/index.rs`, `flags/index.rs`, `verification/index.rs`.

### Child components do NOT need `main-canvas`

If a component renders **inside a parent page** (tab panel, detail sub-view, child route inside an instance wrapper), it must NOT add `main-canvas`. The parent owns the scroll container.

Intentional no-canvas components: `billing/tenant.rs`, `billing/products.rs`, `folio_instance.rs`, `anchor_instance.rs`, `network_instance.rs`.

---

## 2. Design Tokens

### NEVER use hex or raw Tailwind color classes. Always use CSS vars or Tailwind token bridges.

#### CSS Vars — use in `style=` attributes

| CSS Var | Use |
|---------|-----|
| `var(--cobalt)` / `var(--cobalt-dim)` | Primary action / cobalt tint |
| `var(--green)` / `var(--green-dim)` | Success / live / active |
| `var(--amber)` / `var(--amber-dim)` | Warning / caution |
| `var(--red)` / `var(--red-dim)` | Danger (use `var(--error)` in code) |
| `var(--violet)` / `var(--violet-dim)` | AI / premium feature |
| `var(--bg-surface)` | Sidebar / topbar surface |
| `var(--bg-elevated)` | Card / elevated surface |
| `var(--border-default)` | Standard card border |
| `var(--border-subtle)` | Inner divider |
| `var(--text-primary)` | Heading text |
| `var(--text-secondary)` | Body / secondary text |
| `var(--text-muted)` | Caption / dim text |

#### Tailwind Tokens — use in `class=` attributes

| Tailwind class | Use |
|----------------|-----|
| `bg-surface-container` | Default card background |
| `bg-surface-container-low` | Slightly darker section bg |
| `bg-surface-container-high` | Lighter accent bg |
| `bg-surface-dim` | Page base / deepest bg |
| `text-on-surface` | Primary text |
| `text-on-surface-variant` | Secondary / muted text |
| `text-primary` | Cobalt action color |
| `border-outline-variant` | Card / input border |

#### Correct inline `style=` syntax

```rust
// ✅ CSS var syntax
<div class="kpi-value" style="color:var(--green)">{value}</div>
<span style="color:var(--error)">"Error"</span>

// ❌ Never hex in inline styles
<div style="color:#06966a">{value}</div>
<span style="color:#f87171">"Error"</span>
```

---

## 3. Buttons

`.btn` is the **required base class**. Modifier classes have zero visual effect without it.

```rust
// Primary action
<button class="btn btn-primary">"Save Changes"</button>

// Ghost / secondary
<button class="btn btn-ghost">"Cancel"</button>

// Small variants
<button class="btn btn-primary btn-sm">"+ Add"</button>
<button class="btn btn-ghost btn-sm">"Export"</button>

// Icon-only
<button class="btn btn-ghost btn-icon" title="Refresh"><svg .../></button>

// Destructive — filled red (delete, suspend, impersonate)
<button class="btn btn-danger">"Delete Account"</button>

// Low-emphasis danger — outline ghost with error color
<button class="btn btn-ghost" style="color:var(--error)">"Remove"</button>

// Cautionary — amber (escalate, flag)
<button class="btn btn-warn">"Escalate"</button>

// Button-styled link
<a href="/tenants" class="btn btn-ghost" style="text-decoration:none">"View All →"</a>

// Disabled
<button class="btn btn-primary opacity-50 cursor-not-allowed" disabled>"Processing…"</button>
```

**Rules:**
- `btn-ghost` / `btn-primary` / `btn-danger` **require** `btn` as base — never use alone
- Never write `px-4 py-2 rounded-lg font-semibold` for a button
- Never use `btn-primary-gradient` (legacy, being removed)
- Never use `bg-red-600`, `bg-amber-600` directly — use `btn-danger` / `btn-warn`

---

## 4. KPI Cards

```rust
<div class="kpi-row">
    <div class="kpi-card">
        <div class="kpi-label">"Active Tenants"</div>
        <div class="kpi-value mono">{count}</div>
        <div class="kpi-delta up">"↑ 3 this week"</div>
    </div>
    <div class="kpi-card">
        <div class="kpi-label">"Failed Jobs"</div>
        <div class="kpi-value mono" style="color:var(--error)">{failures}</div>
        <div class="kpi-delta down">"up 3 vs yesterday"</div>
    </div>
</div>
```

| Delta class | Color |
|---|---|
| `.kpi-delta.up` | `var(--green)` |
| `.kpi-delta.down` | `var(--red)` |
| `.kpi-delta.neutral` | `var(--text-muted)` |

---

## 5. Sections (Card Containers)

Every content block — tables, lists, forms, stat grids — lives inside a `.section`.

```rust
<div class="section">
    <div class="section-header">
        <div class="section-title">
            <svg viewBox="0 0 16 16" width="13" height="13" .../>  // optional icon
            "Section Name"
            <span class="section-count">{items.len()}</span>
        </div>
        <button class="btn btn-primary btn-sm">"+ Add"</button>
        // OR: <a href="..." class="section-action" style="text-decoration:none">"View All →"</a>
    </div>
    // table, list, or form content here — NO extra padding div needed
</div>
```

Never use raw `rounded-xl border border-outline-variant/20 p-6` as a card — use `.section`.

### Section + Tabs (combined pattern)

```rust
<div class="section">
    <div class="section-header">
        <div class="section-title">"Tenant Configuration"</div>
    </div>
    <div class="tab-bar">
        <button
            class=move || if tab.get()=="overview" { "tab active" } else { "tab" }
            on:click=move |_| tab.set("overview".to_string())
        >"Overview"</button>
        <button
            class=move || if tab.get()=="settings" { "tab active" } else { "tab" }
            on:click=move |_| tab.set("settings".to_string())
        >"Settings"</button>
    </div>
    <div class="p-6">
        // tab content
    </div>
</div>
```

---

## 6. Tables

Use a bare `<table>` inside a `.section`:

```rust
<div class="section">
    <div class="section-header"> ... </div>
    <table>
        <thead><tr>
            <th>"Name"</th>
            <th>"Status"</th>
        </tr></thead>
        <tbody>
            <tr>
                <td>
                    <div style="font-weight:600">{item.name}</div>
                    <div class="mono muted" style="font-size:10px">{item.id}</div>
                </td>
                <td><span class="plan-badge" style=status_style>{item.status}</span></td>
            </tr>
        </tbody>
    </table>
</div>
```

---

## 7. Status / Type Badge Pills (`.plan-badge`)

`.plan-badge` is used for **any** small status or type pill — not just plan tier labels.

```rust
// Dynamic status → CSS var mapping (copy this pattern exactly)
let status_style = match status.as_str() {
    "active"       => "color:var(--green);border-color:var(--green);background:var(--green-dim)",
    "provisioning" => "color:var(--cobalt);border-color:var(--cobalt);background:var(--cobalt-dim)",
    "suspended"    => "color:var(--error);border-color:var(--error);background:var(--red-dim)",
    "beta"         => "color:var(--amber);border-color:var(--amber);background:var(--amber-dim)",
    _              => "color:var(--text-muted);border-color:var(--border-default)",
};

view! {
    <span class="plan-badge" style=status_style>{status}</span>
}
```

Never write `px-2 py-0.5 rounded text-[8px] font-bold border uppercase tracking-wider text-emerald-400 bg-emerald-500/10` — use `.plan-badge`.

---

## 8. Form Inputs

`.form-input` and `.form-select` are the standard classes. Never compose input styles from scratch.

```rust
// Text input
<input type="text" class="form-input" placeholder="Enter name..."
    prop:value=value on:input=move |ev| value.set(event_target_value(&ev))/>

// Textarea
<textarea class="form-input" rows="4"
    prop:value=value on:input=move |ev| value.set(event_target_value(&ev))/>

// Select
<select class="form-select" on:change=move |ev| val.set(event_target_value(&ev)) prop:value=val>
    <option value="a">"Option A"</option>
</select>

// Disabled
<input class="form-input opacity-50 cursor-not-allowed" disabled prop:value=val/>
```

Never write `bg-surface-container border border-outline-variant/30 text-on-surface rounded-lg px-3 py-2.5 focus:ring-1 focus:ring-primary...` — use `.form-input`.

---

## 9. Tab Bars

```rust
let active_tab = RwSignal::new("overview".to_string());

view! {
    <div class="tab-bar">
        <button
            class=move || if active_tab.get()=="overview" { "tab active" } else { "tab" }
            on:click=move |_| active_tab.set("overview".to_string())
        >"Overview"</button>
        <button
            class=move || if active_tab.get()=="settings" { "tab active" } else { "tab" }
            on:click=move |_| active_tab.set("settings".to_string())
        >"Settings"</button>
    </div>
}
```

Never use raw `border-b flex gap-1 bg-surface-container-low/40 px-2 rounded-t-xl` for tabs — use `.tab-bar`.

---

## 10. Modals

```rust
<Show when=move || show_modal.get()>
    <div class="modal-overlay open" on:click=move |_| show_modal.set(false)>
        <div class="modal" on:click=|e| e.stop_propagation()>
            // Header
            <div style="display:flex;align-items:center;justify-content:space-between;margin-bottom:16px">
                <div style="font-size:14px;font-weight:600;color:var(--text-primary)">"Modal Title"</div>
                <button class="btn btn-ghost btn-icon btn-sm" on:click=move |_| show_modal.set(false)>"×"</button>
            </div>
            // Body
            <div style="display:flex;flex-direction:column;gap:12px">
                // content
            </div>
            // Footer — cancel always left of confirm
            <div style="display:flex;justify-content:flex-end;gap:8px;margin-top:20px">
                <button class="btn btn-ghost" on:click=move |_| show_modal.set(false)>"Cancel"</button>
                <button class="btn btn-primary" on:click=handle_submit>"Confirm"</button>
                // OR for destructive:
                <button class="btn btn-danger" on:click=handle_delete>"Delete"</button>
            </div>
        </div>
    </div>
</Show>
```

Never use raw `fixed inset-0 z-[100] bg-black/80 backdrop-blur-md flex items-center justify-center p-4` — use `.modal-overlay.open`. The z-index (500) is already set in CSS.

---

## 11. Shared UI & Theming

Token bridge chain: `index.css` CSS vars → `--color-*` bridge → `tailwind.config.js` → `shared-ui` Tailwind tokens.

**Rules:**
- New pages: raw HTML + design system classes. No `shared_ui::Card` for layout.
- New widgets: `shared_ui::components::ui::*` is fine — they inherit the theme.
- New shared-ui components: Tailwind semantic tokens only. No hex.
- Responsive: `main-canvas` + `col-hide-mobile` / `col-hide-tablet` on non-essential table columns.

---

## 12. Anti-Patterns ❌

### Buttons

```rust
// ❌ Modifier without base — btn-ghost alone has no effect
class="btn-ghost px-3.5 py-2 border..."

// ❌ Raw ad-hoc button
class="px-4 py-2 bg-red-600 hover:bg-red-700 text-white rounded-lg text-xs font-bold"
class="px-4 py-2 bg-amber-600 hover:bg-amber-700 text-white rounded-lg"
class="px-4 py-2 bg-surface-container-highest border border-outline-variant/30 rounded-lg"

// ❌ Legacy gradient
class="btn-primary-gradient px-4 py-2 rounded-lg"

// ✅ Use instead
class="btn btn-danger"      // red destructive
class="btn btn-warn"        // amber cautionary
class="btn btn-ghost"       // cancel / secondary
class="btn btn-primary"     // primary action
```

### Cards

```rust
// ❌ Raw card container
<div class="rounded-xl border border-outline-variant/20 p-6 bg-surface-container-low">
<div class="rounded-2xl border border-white/10 shadow-2xl p-6">

// ✅
<div class="section">
    <div class="section-header"> ... </div>
    // content
</div>
```

### Inputs

```rust
// ❌ Raw 60-char class string
class="bg-surface-container border border-outline-variant/30 text-on-surface text-sm rounded-lg px-3 py-2.5 focus:ring-1 focus:ring-primary focus:border-primary outline-none transition-all"

// ✅
class="form-input"
```

### Modals

```rust
// ❌ Raw overlay (wrong z-index, wrong blur, wrong background)
<div class="fixed inset-0 z-[100] bg-black/80 backdrop-blur-md flex items-center justify-center p-4">
    <div class="bg-surface w-full max-w-xl rounded-2xl border border-white/10 shadow-2xl">

// ✅
<div class="modal-overlay open">
    <div class="modal">
```

### Tabs

```rust
// ❌ Raw tab bar
<div class="border-b border-outline-variant/20 flex gap-1 bg-surface-container-low/40 px-2 rounded-t-xl">

// ✅
<div class="tab-bar">
    <button class="tab active">"Overview"</button>
    <button class="tab">"Settings"</button>
</div>
```

### Status Badges

```rust
// ❌ Raw badge
<span class="px-2 py-0.5 rounded text-[8px] font-bold border uppercase tracking-wider text-emerald-400 bg-emerald-500/10 border-emerald-500/20">

// ✅
<span class="plan-badge" style="color:var(--green);border-color:var(--green);background:var(--green-dim)">
```

### Inline Style Hex

```rust
// ❌
style="color:#a5b4fc"
style="background:#635BFF22;color:#635BFF"

// ✅
style="color:var(--cobalt)"
style="background:var(--cobalt-dim);color:var(--cobalt)"
```

---

## 13. Layout Class Quick Reference

| Class | Purpose |
|---|---|
| `.main-canvas` | Top-level route scroll container |
| `.main-area` | Full-height no-padding (split views only) |
| `.page-header` | Title + actions row |
| `.page-title` | Page heading |
| `.page-subtitle` | Page description |
| `.page-actions` | Button row (right-aligned) |
| `.kpi-row` | Flex row of KPI cards |
| `.kpi-card` | Individual metric card |
| `.kpi-label` | Metric name |
| `.kpi-value` / `.kpi-value.mono` | Large number |
| `.kpi-delta.up/.down/.neutral` | Trend line |
| `.section` | Card container |
| `.section-header` | Card title row (alias: `.section-hdr`) |
| `.section-title` | Icon + name inside header |
| `.section-count` | Dim count badge |
| `.section-action` | Right-side link in section-header |
| `.tab-bar` | Tab navigation container |
| `.tab` | Tab button |
| `.tab.active` | Active tab state |
| `.modal-overlay` | Fixed overlay backdrop (hidden by default) |
| `.modal-overlay.open` | Visible modal |
| `.modal` | Modal panel |
| `.btn` | Base button (required with all modifiers) |
| `.btn-primary` | Filled cobalt button |
| `.btn-ghost` | Outlined ghost button |
| `.btn-danger` | Filled red destructive button |
| `.btn-warn` | Amber cautionary button |
| `.btn-sm` | Small height variant |
| `.btn-icon` | Square icon-only button |
| `.plan-badge` | Status / type pill badge |
| `.form-input` | Text input / textarea |
| `.form-select` | Select / dropdown |
| `.stat-row` / `.s-label` / `.s-value` | Key-value pair row |
| `.rail-row` | Icon + name + status row |
| `.mono` | Monospace font |
| `.muted` | `var(--text-muted)` color |
| `.secondary` | `var(--text-faint)` color |
| `.col-hide-mobile` | Hide table column on mobile |
| `.col-hide-tablet` | Hide table column on tablet |
| `.entity-page` | Full-height CRM/record detail |
