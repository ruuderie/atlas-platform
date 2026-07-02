# Platform Admin — Style Guide

> **Reference standard**: `apps/platform-admin/src/pages/dashboard.rs` (Command Center)
> **Token source**: `apps/platform-admin/style/index.css`

---

## 1. Page Anatomy

Every full page in the admin shell follows this hierarchy:

```
<div class="main-canvas">          ← scroll container, 20px/24px padding, gap:20px
  <div class="page-header">        ← title row (flex, space-between)
    <div>
      <h1 class="page-title">      ← ~22px bold
      <p class="page-subtitle">    ← 12px muted
    </div>
    <div class="page-actions">     ← button row (flex, gap:8px)
  </div>

  <div class="kpi-row">            ← flex-wrap, auto-fills 4 cards at 1200px+
    <div class="kpi-card">
      <div class="kpi-label">
      <div class="kpi-value mono">  ← use style="color:var(--green)" etc.
      <div class="kpi-delta up|down|neutral">
    </div>
  </div>

  <div class="section">            ← card container (bg, border, radius)
    <div class="section-header">   ← OR .section-hdr (legacy alias)
      <div class="section-title">
        <svg .../>                  ← 13x13 icon (optional)
        "Section Name"
        <span class="section-count">{n}</span>
      </div>
    </div>
    <!-- table, list, form content -->
  </div>
</div>
```

### When to use `main-area` instead of `main-canvas`

Use `main-area` only for pages that **manage their own internal scroll** (e.g. split-pane views, full-height tables with fixed headers). Examples: `support/index.rs`, `flags/index.rs`, `verification/index.rs`.

---

## 2. Design Tokens

### Color Primitives (always use CSS vars, never hex/Tailwind-color in new code)

| Token | Value | Use |
|---|---|---|
| `--cobalt` | `#2079f7` | Primary/action |
| `--cobalt-dim` | `rgba(10,132,255,.12)` | Cobalt background tint |
| `--green` | `#06966a` | Success / live |
| `--green-dim` | `rgba(6,150,105,.14)` | Green background tint |
| `--amber` | `#f5a623` | Warning / beta |
| `--amber-dim` | `rgba(245,166,35,.12)` | Amber background tint |
| `--red` | `#e5484d` | Danger |
| `--red-dim` | `rgba(229,72,77,.14)` | Red background tint |
| `--error` | same as `--red` | Semantic alias — always use `--error` in code |

### Semantic Surface Tokens

| Token | Use |
|---|---|
| `--bg-base` | Page background |
| `--bg-elevated` | Card / section background |
| `--bg-overlay` | Modal / overlay background |
| `--border-default` | Standard card borders |
| `--border-subtle` | Inner dividers |
| `--text-primary` | Heading text |
| `--text-muted` | Subtitle / secondary text |
| `--text-faint` | Caption / disabled text |

---

## 3. Buttons

```html
<!-- Primary action -->
<button class="btn btn-primary">Save Changes</button>

<!-- Ghost / secondary -->
<button class="btn btn-ghost">Cancel</button>

<!-- Small variants -->
<button class="btn btn-primary btn-sm">...</button>
<button class="btn btn-ghost btn-sm">...</button>

<!-- Icon-only (no text, square) -->
<button class="btn btn-ghost btn-icon" title="Refresh">
  <svg .../>
</button>
```

**Rules:**
- Never use `btn-primary-gradient` in new pages (legacy class, being phased out)
- Never use inline `px-4 py-2 rounded-lg ...` for buttons — always use `.btn` composable classes
- Danger actions: `class="btn btn-ghost" style="color:var(--error)"` (not `btn-danger`)

---

## 4. KPI Cards

```rust
<div class="kpi-row">
    <div class="kpi-card">
        <div class="kpi-label">"Active Clients"</div>
        <div class="kpi-value mono">{count}</div>
        <div class="kpi-delta up">"up 12% this week"</div>
    </div>
    <div class="kpi-card">
        <div class="kpi-label">"Failed Jobs"</div>
        <div class="kpi-value mono" style="color:var(--red)">{failures}</div>
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

## 5. Tables

Use bare `<table>` — CSS resets it to the admin style automatically.

```html
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
      <td>
        <span class="plan-badge" style=format!("color:{c};border-color:{c};background:{b}", c=color_var, b=bg_var)>
            {item.status}
        </span>
      </td>
    </tr>
  </tbody>
</table>
```

Status badge color mapping:

| Status | Color var | Bg var |
|---|---|---|
| `active` | `--green` | `--green-dim` |
| `provisioning` | `--cobalt` | `--cobalt-dim` |
| `beta` | `--amber` | `--amber-dim` |
| `suspended` / `error` | `--red` | `--red-dim` |

---

## 6. Sections

```html
<div class="section">
  <div class="section-header">
    <div class="section-title">
      <svg .../>
      "API Keys"
      <span class="section-count">{keys.len()}</span>
    </div>
    <button class="btn btn-primary btn-sm">"+ Add Key"</button>
  </div>
  <!-- content -->
</div>
```

---

## 7. Shared UI Migration Policy

`shared_ui` components (`Card`, `Button`, `DataTable`, etc.) use hardcoded Tailwind utility classes and **do not automatically inherit design tokens** from `platform-admin`.

**Policy:**
- **New pages/components**: Always use raw HTML + design system classes (no `shared_ui` imports).
- **Existing pages using shared_ui**: Replace on next meaningful edit of that file.
- **Long-term goal**: `shared_ui` components should consume CSS vars from the parent app context so they auto-adapt to whichever app is hosting them (platform-admin, folio, anchor, etc.).

---

## 8. Layout Class Quick Reference

| Class | Purpose |
|---|---|
| `.main-canvas` | Standard page scroll container |
| `.main-area` | Full-height no-padding wrapper (split views) |
| `.page-header` | Title + actions row |
| `.page-title` | `h1` inside page-header |
| `.page-subtitle` | `p` inside page-header |
| `.page-actions` | Button row inside page-header |
| `.kpi-row` | Flex row of KPI cards |
| `.kpi-card` | Individual metric card |
| `.kpi-label` | Metric name |
| `.kpi-value` | Large number |
| `.kpi-value.mono` | Monospace large number |
| `.kpi-delta.up/.down/.neutral` | Trend line |
| `.section` | Card container |
| `.section-header` | Card title row (alias: `.section-hdr`) |
| `.section-title` | Icon + name inside section-header |
| `.section-count` | Dim count badge inside section-title |
| `.btn` | Base button |
| `.btn-primary` | Filled cobalt button |
| `.btn-ghost` | Outlined ghost button |
| `.btn-sm` | Small height variant |
| `.btn-icon` | Square icon-only button |
| `.plan-badge` | Pill badge for status/type tags |
| `.mono` | Monospace font helper |
| `.muted` | `var(--text-muted)` color helper |
| `.secondary` | `var(--text-faint)` color helper |
| `.entity-page` | Full-height CRM/record detail layout |
