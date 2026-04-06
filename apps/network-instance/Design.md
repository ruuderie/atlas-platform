# Design System Strategy: The Editorial Authority

## 1. Overview & Creative North Star

The Creative North Star for this design system is **"The Architectural Curator."**

Standard directory designs often feel cluttered, transactional, and "cheap" due to heavy borders and dense grids. This system rejects those tropes. We are building a high-end editorial experience that conveys trust through **intentional breathing room, asymmetric balance, and tonal depth.**

By moving away from the "box-within-a-box" layout, we position the platform as a premium authority. We use a sophisticated interplay of `manrope` for authoritative displays and `inter` for functional clarity, creating a modular framework that feels like a bespoke digital magazine rather than a spreadsheet.

---

## 2. Color & Surface Architecture

We convey reliability not through heavy-handed blues, but through a palette of deep navy (`primary: #004289`) and a sophisticated range of "warm-cool" neutrals.

### The "No-Line" Rule

**Explicit Instruction:** Designers are prohibited from using 1px solid borders to define sections or cards.
Structure must be achieved through **Background Color Shifts**. For instance, a search result card (`surface-container-lowest`) should sit on a background of `surface-container-low`. The edge is defined by the change in value, not a line.

### Surface Hierarchy & Nesting

Treat the UI as a physical stack of premium paper.

- **Base Layer:** `surface` (#f8f9fa)
- **Sectional Shifts:** Use `surface-container-low` for large content blocks.
- **Elevated Content:** Use `surface-container-lowest` (#ffffff) for active cards or search results to create a "lifted" feel.
- **Interactive Depth:** Nested elements (like a search bar inside a hero) should use `surface-container-high` to recede or `surface-container-lowest` to pop.

### The "Glass & Gradient" Rule

To escape the "flat" look, apply a subtle linear gradient to Hero sections transitioning from `primary` (#004289) to `primary-container` (#2059a9) at a 135-degree angle. For floating navigation or filters, use **Glassmorphism**:

- **Background:** `surface` at 80% opacity.
- **Effect:** `backdrop-filter: blur(12px)`.

---

## 3. Typography: The Editorial Voice

Our typography scale leverages two sans-serifs to distinguish between _brand authority_ and _functional utility_.

- **Display & Headlines (Manrope):** Use `display-lg` (3.5rem) and `headline-md` (1.75rem) with tight letter-spacing (-0.02em). This typeface provides the "Architectural" feel—stable, modern, and high-end.
- **Body & Labels (Inter):** All functional data (directories, descriptions, metadata) uses `inter`. It is optimized for legibility at small scales (`body-sm`: 0.75rem).
- **Visual Rhythm:** Maintain a 4:1 scale ratio between your largest display type and your primary body copy to create a dramatic, high-contrast hierarchy that guides the eye instantly to the "curated" content.

---

## 4. Elevation & Depth

In this system, elevation is a matter of light and shadow, not lines.

- **Tonal Layering:** Instead of a shadow, place a `surface-container-lowest` card on a `surface-container-low` background. This is the primary method of containment.
- **Ambient Shadows:** For floating elements (Modals, Popovers), use a multi-layered shadow:
- `box-shadow: 0 10px 30px rgba(25, 28, 29, 0.04), 0 4px 8px rgba(25, 28, 29, 0.02);`
- Note: The shadow color is a tinted version of `on-surface` (#191c1d), never pure black.
- **The "Ghost Border" Fallback:** If a border is required for accessibility (e.g., input fields), use `outline-variant` at **15% opacity**. It should be felt, not seen.

---

## 5. Components & Modular Elements

### Buttons (The "Call to Trust")

- **Primary:** Background: `primary` (#004289). Typography: `on-primary` (#ffffff). Shape: `md` (0.375rem).
- _Refinement:_ Add a subtle 1px inner-glow (top-down white at 10% opacity) to give the button a tactile, premium feel.
- **Secondary:** Background: `secondary-container` (#cfe6f2). Typography: `on-secondary-fixed-variant`. No border.

### Search & Result Cards

- **Forbid Dividers:** Do not use lines between search results. Use **Vertical Spacing Scale `8` (2.75rem)** to separate items.
- **The Hover State:** Upon hover, a card should shift from `surface-container-low` to `surface-container-lowest` and gain an Ambient Shadow.

### Chips & Filters

- Use `full` roundedness (9999px).
- Unselected: `surface-container-high`.
- Selected: `primary` with `on-primary` text.

### Inputs (Search Bars)

- Large padding: `spacing-4` (1.4rem).
- Background: `surface-container-lowest`.
- Shadow: Use a "soft inner-glow" (a subtle `surface-dim` shadow inside the top edge) to suggest the input is carved into the page.

---

## 6. Do’s and Don’ts

### Do

- **Do** use asymmetrical layouts. A 2/3 width search result list paired with a 1/3 width "Featured" section creates a professional, editorial look.
- **Do** use `spacing-20` (7rem) or `spacing-24` (8.5rem) for section margins. White space is a luxury signal.
- **Do** use `tertiary` (#7b2600) sparingly as an "Expertise" accent (e.g., a "Verified" badge or a "Top Rated" tag).

### Don't

- **Don’t** use pure black (#000000) for text. Use `on-surface` (#191c1d) to maintain a soft, premium contrast.
- **Don’t** use standard 1px dividers. If separation is needed, use a 1px height `surface-variant` line that only spans 80% of the container width, centered.
- **Don’t** cram information. If a directory listing has 10 data points, hide 6 of them behind a "Details" progressive disclosure to maintain a clean aesthetic.

---

## 7. Spacing Utility

All spacing must follow the defined scale to ensure mathematical harmony.

- **Component Internal Padding:** `3` (1rem) or `4` (1.4rem).
- **Section Gaps:** `12` (4rem) minimum.
- **Hero Padding:** `24` (8.5rem) to establish the "Editorial Authority" immediately upon arrival.
