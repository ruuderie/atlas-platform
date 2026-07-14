# Multi-unit hub / Projects / G-27 — visual & behavioral parity checklist

Use before marking a P0b page `[x]` in [`page_queue.md`](page_queue.md). Compare against Stitch under `designs/stitch/project_pm/folio/`.

**Quality bar:** implement plan § Production quality bar · [`stitch_to_leptos_prompt.md`](stitch_to_leptos_prompt.md) · AGENTS.md §13.

## Global (every page)

- [x] No Stitch mock strings / fake dollars / fake contractor names in `view!`
- [x] No `bg-slate-*`, `bg-white`, CDN Tailwind, inline gradient styles
- [x] Colors from `tailwind.config.js` / `.folio-*` / page block in `main.css`
- [x] Links via `FolioRoute` / `NavIcon` (no hardcoded `"/l/..."`)
- [x] Closed vocabularies are enums at DTO boundary (e.g. `PropertyDocumentKind`, `PropertyTab`, `CreateMode`)
- [x] `<Suspense>` + skeleton / empty fallback per data section
- [x] Explicit error + retry; empty state with copy
- [x] Desktop (≥1024) and mobile (≤768) layouts use hub/proj CSS (activity rail collapses via `.hub-*`)
- [x] `prefers-reduced-motion` respected for press/sheet/lightbox (main.css)
- [x] Escape + overlay dismiss for sheets / lightbox

## Shared primitives

- [x] `PropertyTabBar` on hub / unit / systems / documents / portal
- [x] `ActivityRail` on hub Overview (not a tab) — empty state until property feed API
- [x] `StatusPill` for WO / project / occupancy statuses
- [x] `InterruptibleSheet` for create WO / log paid
- [x] `PhotoLightbox` + strip on ratings, WO detail, project evidence

## Property hub (`l_property_hub`)

- [x] KPI strip, units peek, Projects peek
- [x] Activity right rail (property-scoped component; empty until feed wired)
- [x] Tab bar: Overview | Units | Systems | Portal | Documents
- [x] Parent multi-unit asset opens hub (not generic leaf detail)

## Unit detail (`l_unit_detail`)

- [x] Spaces empty state + LTR vs short-term mode from `str_eligible`
- [x] Create WO CTA

## Project detail (`l_project_detail`)

- [x] Budget KPIs (budget / committed / actual / remaining)
- [x] Timeline (`ProjectTimelineKind` from API)
- [x] Child WO list with costs
- [x] G-27 rollup panel (composite, dims, vendors, coverage, pending CTA)
- [x] Scope via real `asset_id` on project (no Stitch design toggle)

## Documents / WO / Ratings / Queue

- [x] Documents: compose API + `?project=` filter + vault deep-link
- [x] WO create: optional project; Schedule / Log paid modes via sheet
- [x] WO detail: cost, complete → G-27; project crumb; photo strip
- [x] Ratings: landlord layout + ScorecardWidget sessions + project filter crumb
- [x] Queue: New / Log paid / Schedule / G-27 CTAs; row → WO detail

## Sign-off

| Page | Desktop | Mobile | Reviewer | Date |
|------|---------|--------|----------|------|
| Property hub | code | code | agent | 2026-07-14 |
| Unit detail | code | code | agent | 2026-07-14 |
| Project detail | code | code | agent | 2026-07-14 |
| Documents | code | code | agent | 2026-07-14 |
| WO create | code | code | agent | 2026-07-14 |
| WO detail | code | code | agent | 2026-07-14 |
| Ratings | code | code | agent | 2026-07-14 |
| Queue polish | code | code | agent | 2026-07-14 |

> Code-level parity pass vs Stitch IA/CSS tokens. Pixel QA in a browser against live data remains recommended before product release.
