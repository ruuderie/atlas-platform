# shared-ui — Component Registry

> Status key: ✅ CSS-var compliant · ⚠️ Uses hardcoded Tailwind colors · 🔧 Partially migrated · 🚫 No styling (layout/logic only)

## Theming Status

See `THEMING.md` for the full architecture guide on how components should be styled.

The goal: all components should reference Tailwind semantic tokens (`bg-primary`, `bg-card`, etc.)
that resolve through CSS vars, **not** hardcoded hex or Tailwind palette colors.

---

## UI Primitives (components/ui/)

| Component | File | Status | Notes |
|---|---|---|---|
| `Button` | `ui/button.rs` | ⚠️ | Uses `bg-primary`, `bg-destructive`, `bg-secondary`, `bg-accent` — all mapped via tailwind.config.js to CSS vars. Works but depends on tailwind config being correct. |
| `Card` | `ui/card.rs` | ⚠️ | `bg-card`, `text-card-foreground` — same situation |
| `Input` | `ui/input.rs` | ⚠️ | `border-input`, `bg-background`, `text-foreground`, `text-muted-foreground` |
| `Label` | `ui/label.rs` | ✅ | No color classes, pure layout |
| `Switch` | `ui/switch.rs` | 🔧 | compact variant uses `var(--green)` directly (good), default uses `bg-primary` / `bg-input` |
| `Select` | `ui/select.rs` | ⚠️ | `bg-popover`, `bg-card`, `bg-accent`, `text-muted-foreground` |
| `Checkbox` | `ui/checkbox.rs` | ⚠️ | `bg-primary`, `border-input`, `ring-ring` |
| `Textarea` | `ui/textarea.rs` | ⚠️ | Same as Input |
| `Dialog` | `ui/dialog.rs` | ⚠️ | `bg-popover`, `border-border` |
| `Table` | `ui/table.rs` | ⚠️ | `bg-card`, `border-border` |
| `Badge` (ui) | `ui/badge.rs` | ⚠️ | Uses variants with hardcoded Tailwind palette |
| `Avatar` | `ui/avatar.rs` | ✅ | CSS-var aware |
| `Skeleton` | `ui/skeleton.rs` | ⚠️ | `bg-muted` |
| `Tooltip` | `ui/tooltip.rs` | ⚠️ | `bg-popover`, `text-popover-foreground` |
| `Tabs` | `ui/tabs.rs` | ⚠️ | `bg-muted`, `bg-background` |
| `Progress` | `ui/progress.rs` | ⚠️ | `bg-primary` |
| `Slider` | `ui/slider.rs` | ⚠️ | `bg-primary`, `bg-muted` |
| `Drawer` | `ui/drawer.rs` | ⚠️ | `bg-background` |
| `Sheet` | `ui/sheet.rs` | ⚠️ | `bg-background` |
| `Popover` | `ui/popover.rs` | ⚠️ | `bg-popover` |
| `Command` | `ui/command.rs` | ⚠️ | `bg-popover`, `text-muted-foreground` |
| `DropdownMenu` | `ui/dropdown_menu.rs` | ⚠️ | `bg-popover` |
| `RadioButton` | `ui/radio_button.rs` | ⚠️ | `border-input`, `text-primary` |
| `MultiSelect` | `ui/multi_select.rs` | ⚠️ | `bg-background`, `border-input` |
| `DatePicker` | `ui/date_picker.rs` | ⚠️ | `bg-popover`, `bg-primary` |
| `Alert` | `ui/alert.rs` | ⚠️ | `bg-destructive`, `text-muted-foreground` |
| `AlertDialog` | `ui/alert_dialog.rs` | ⚠️ | `bg-background` |
| `Collapsible` | `ui/collapsible.rs` | ✅ | Layout only |
| `Accordion` | `ui/accordion.rs` | ⚠️ | `bg-background` |
| `Pagination` | `ui/pagination.rs` | ⚠️ | `bg-accent` |
| `HoverCard` | `ui/hover_card.rs` | ⚠️ | `bg-popover` |
| `ContextMenu` | `ui/context_menu.rs` | ⚠️ | `bg-popover` |
| `Menubar` | `ui/menubar.rs` | ⚠️ | `bg-background`, `bg-accent` |
| `NavigationMenu` | `ui/navigation_menu.rs` | ⚠️ | `bg-background`, `bg-accent` |
| `Charts` | `ui/charts.rs` | ⚠️ | Uses `bg-card`, hardcoded colors for chart series |
| `DataTable` | `ui/data_table.rs` | ⚠️ | `bg-card`, `text-muted-foreground` |
| `InputOtp` | `ui/input_otp.rs` | ⚠️ | `border-input`, `ring-ring` |
| `AutoForm` | `ui/auto_form.rs` | ⚠️ | inherits from Input/Select/Checkbox |
| `Kbd` | `ui/kbd.rs` | ⚠️ | `bg-muted` |
| `Switch` | `ui/switch.rs` | 🔧 | Partially migrated (see above) |

---

## Composite Components (components/)

| Component | File | Status | Notes |
|---|---|---|---|
| `Badge` | `badge.rs` | ✅ | Uses semantic CSS classes (`badge`, `badge-*`) not Tailwind tokens — **good pattern** |
| `ThemeProvider` | `theme_provider.rs` | ✅ | Runtime override via `--color-primary`. See THEMING.md. |
| `Modal` | `modal.rs` | ⚠️ | `bg-background`, `border-border` |
| `Icon` | `icon.rs` | ✅ | No colors |
| `Tabs` | `tabs.rs` | ⚠️ | Uses hardcoded border colors |
| `EmailComposer` | `email_composer.rs` | ⚠️ | Has mixed inline styles and Tailwind tokens |
| `CrmTimeline` | `crm_timeline.rs` | 🔧 | Partially uses CSS vars |
| `CrmStageBar` | `crm_stage_bar.rs` | ✅ | Uses CSS vars for stage colors |
| `DataGrid` | `data_grid.rs` | ⚠️ | `bg-card`, `border-border` |
| `PropertiesEditor` | `properties_editor.rs` | ⚠️ | Mixed |
| `RelatedList` | `ui/related_list.rs` | ⚠️ | `bg-card`, `text-muted-foreground` |
| `AdminModuleSidebar` | `admin_module_sidebar.rs` | ⚠️ | Has hardcoded dark colors |

---

## Migration Priority (for platform-admin)

These are the components **actively imported** in platform-admin that should be
migrated first. The CSS var bridge in `style/index.css` means they already work
correctly — this is about the source components being self-contained for future apps.

| Priority | Component | Used in |
|---|---|---|
| 1 | `Button` | upsell_banner, milestone_modal, apps/create, apps/detail, billing/* |
| 2 | `Card` | apps/create, billing/tenant, upsell_banner |
| 3 | `Input` | dynamic_form, apps/create, apps/detail |
| 4 | `Select` | dynamic_form |
| 5 | `Checkbox` | dynamic_form |
| 6 | `Switch` | flags/index (complete the partial migration) |
| 7 | `Badge` (shared_ui) | apps/detail, billing/tenant |
| 8 | `Table/DataTable` | billing/tenant |
| 9 | `Modal` | milestone_modal |

---

## How to Make a Component CSS-var Compliant

**Before:**
```rust
clx! {MyButton, button, "bg-primary text-primary-foreground px-4 py-2 rounded-md"}
```

**After (no change needed if tailwind.config.js is correct!):**

Because `bg-primary` in `tailwind.config.js` now resolves to `var(--color-primary)`,
and `var(--color-primary)` is defined in the app's CSS to point at the correct palette
token, no change is needed to the component Rust code.

The migration is complete at the **tailwind.config.js** + **CSS bridge** level.
The only time you need to change component Rust code is if it uses:
- Hardcoded hex colors: `bg-[#05183c]` → use a semantic token instead
- Hardcoded Tailwind palette colors: `bg-blue-900` → use `bg-primary` or `bg-card`
- Inline `style` attributes with hardcoded hex → use `style="color:var(--some-token)"`

