/** @type {import('tailwindcss').Config} */
//
// IMPORTANT: Color values here MUST reference CSS custom properties (--color-*)
// defined in style/index.css, NOT hardcoded hex values. This ensures shared-ui
// components automatically inherit the correct palette for this app.
//
// See apps/shared-ui/THEMING.md for the full architecture guide.
//
module.exports = {
  darkMode: "class",
  content: [
    "./src/**/*.rs",
    "../shared-ui/src/**/*.rs",
    "./index.html"
  ],
  theme: {
    extend: {
      colors: {
        // ── shared-ui semantic tokens (mapped to CSS vars) ─────────────────
        "primary": {
          DEFAULT:    "var(--color-primary)",
          foreground: "var(--color-primary-foreground)",
        },
        "card": {
          DEFAULT:    "var(--color-card)",
          foreground: "var(--color-card-foreground)",
        },
        "background":  "var(--color-background)",
        "foreground":  "var(--color-foreground)",
        "popover": {
          DEFAULT:    "var(--color-popover)",
          foreground: "var(--color-popover-foreground)",
        },
        "muted": {
          DEFAULT:    "var(--color-muted)",
          foreground: "var(--color-muted-foreground)",
        },
        "accent": {
          DEFAULT:    "var(--color-accent)",
          foreground: "var(--color-accent-foreground)",
        },
        "secondary": {
          DEFAULT:    "var(--color-secondary)",
          foreground: "var(--color-secondary-foreground)",
        },
        "destructive": {
          DEFAULT:    "var(--color-destructive)",
          foreground: "var(--color-destructive-foreground)",
        },
        "success": {
          DEFAULT:    "var(--color-success)",
          foreground: "var(--color-success-foreground)",
        },
        "warning": {
          DEFAULT:    "var(--color-warning)",
          foreground: "var(--color-warning-foreground)",
        },
        "border": "var(--color-border)",
        "input":  "var(--color-input)",
        "ring":   "var(--color-ring)",

        // ── Platform-admin extended palette (Material Design 3 dynamic color) ─
        // These are kept for pages that still use Tailwind utility classes
        // directly (e.g. text-on-surface-variant, bg-surface-container, etc.)
        // Do NOT add new pages using these — use CSS vars instead.
        "outline":                   "#5b74b1",
        "surface-dim":               "#060e20",
        "on-secondary-container":    "#b1c0d6",
        "on-secondary":              "#122131",
        "secondary-dim":             "#909fb4",
        "on-primary-container":      "#97d8ff",
        "surface-container-high":    "#031d4b",
        "surface-container":         "#05183c",
        "on-primary-fixed-variant":  "#006286",
        "inverse-surface":           "#faf8ff",
        "on-background":             "#dee5ff",
        "on-surface-variant":        "#91aaeb",
        "surface":                   "#060e20",
        "on-error-container":        "#ff9993",
        "secondary-container":       "#2e3c4e",
        "on-primary":                "#004560",
        "surface-tint":              "#7bd0ff",
        "surface-bright":            "#002867",
        "inverse-primary":           "#00668b",
        "surface-variant":           "#00225a",
        "surface-container-lowest":  "#000000",
        "on-surface":                "#dee5ff",
        "surface-container-highest": "#00225a",
        "outline-variant":           "#2b4680",
        "surface-container-low":     "#06122d",
        "primary-container":         "#004c69",
        "on-error":                  "#490106",
        "error":                     "var(--color-destructive)",
      },
      fontFamily: {
        "headline": ["Inter", "sans-serif"],
        "body":     ["Inter", "sans-serif"],
        "label":    ["Inter", "sans-serif"],
        "sans":     ["Inter", "sans-serif"],
      },
      borderRadius: {
        "DEFAULT": "0.125rem",
        "lg":      "0.25rem",
        "xl":      "0.5rem",
        "full":    "0.75rem",
      },
    },
  },
  plugins: [],
}
