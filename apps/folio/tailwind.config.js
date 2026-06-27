/** @type {import('tailwindcss').Config} */
//
// ── Folio Design Token Reference ─────────────────────────────────────────────
// This palette is derived from the stitch prototype designs in:
//   designs/stitch/project_pm/folio/
// The stitch HTML files are the visual source of truth.
// DO NOT introduce colors outside this token set.
// See docs/folio/README.md for the full token mapping guide.
// ─────────────────────────────────────────────────────────────────────────────
module.exports = {
  content: {
    files: ["*.html", "./src/**/*.rs", "../../backend/src/migration/**/*.rs"],
  },
  darkMode: "class",
  theme: {
    extend: {
      colors: {
        // ── Surfaces (light mode) ──────────────────────────────────────────
        "background":                 "#f7f9fb",
        "surface":                    "#f7f9fb",
        "surface-bright":             "#f7f9fb",
        "surface-dim":                "#d8dadc",
        "surface-container-lowest":   "#ffffff",
        "surface-container-low":      "#f2f4f6",
        "surface-container":          "#eceef0",
        "surface-container-high":     "#e6e8ea",
        "surface-container-highest":  "#e0e3e5",
        "surface-variant":            "#e0e3e5",
        "surface-tint":               "#565e74",
        "inverse-surface":            "#2d3133",
        "inverse-on-surface":         "#eff1f3",

        // ── On-Surface (text / icon) ───────────────────────────────────────
        "on-background":              "#191c1e",
        "on-surface":                 "#191c1e",
        "on-surface-variant":         "#45464d",

        // ── Outline / borders ─────────────────────────────────────────────
        "outline":                    "#76777d",
        "outline-variant":            "#c6c6cd",

        // ── Primary — black/charcoal ──────────────────────────────────────
        "primary":                    "#000000",
        "on-primary":                 "#ffffff",
        "primary-fixed":              "#dae2fd",
        "primary-fixed-dim":          "#bec6e0",
        "primary-container":          "#131b2e",
        "on-primary-container":       "#7c839b",
        "on-primary-fixed":           "#131b2e",
        "on-primary-fixed-variant":   "#3f465c",
        "inverse-primary":            "#bec6e0",

        // ── Secondary — slate blue ─────────────────────────────────────────
        "secondary":                  "#515f74",
        "on-secondary":               "#ffffff",
        "secondary-fixed":            "#d5e3fc",
        "secondary-fixed-dim":        "#b9c7df",
        "secondary-container":        "#d5e3fc",
        "on-secondary-container":     "#57657a",
        "on-secondary-fixed":         "#0d1c2e",
        "on-secondary-fixed-variant": "#3a485b",

        // ── Tertiary — green (positive KPIs, upward trends) ───────────────
        "tertiary":                   "#196b4a",
        "on-tertiary":                "#ffffff",
        "tertiary-fixed":             "#85f8c4",
        "tertiary-fixed-dim":         "#68dba9",
        "tertiary-container":         "#002114",
        "on-tertiary-container":      "#069669",
        "on-tertiary-fixed":          "#002114",
        "on-tertiary-fixed-variant":  "#005137",

        // ── Error ─────────────────────────────────────────────────────────
        "error":                      "#ba1a1a",
        "on-error":                   "#ffffff",
        "error-container":            "#ffdad6",
        "on-error-container":         "#93000a",

        // ── Brand extras ──────────────────────────────────────────────────
        "bitcoin-orange":             "#F7931A",
      },

      fontFamily: {
        "sans":      ["Inter", "system-ui", "sans-serif"],
        "headline":  ["Inter"],
        "body":      ["Inter"],
        "label":     ["Inter"],
        "mono":      ["JetBrains Mono", "ui-monospace", "monospace"],
        "display":   ["Newsreader", "Georgia", "serif"],
      },

      borderRadius: {
        "DEFAULT": "0.125rem",
        "sm":      "0.25rem",
        "md":      "0.375rem",
        "lg":      "0.5rem",
        "xl":      "0.75rem",
        "2xl":     "1rem",
        "full":    "9999px",
      },
    },
  },
  plugins: [],
}
