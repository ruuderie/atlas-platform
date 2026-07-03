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
  corePlugins: {
    preflight: false,  // disable Tailwind base reset — causes `border: 0 solid #e5e7eb` on all elements
  },
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

        // ── Platform-admin legacy MD3 token bridge ──────────────────────────
        // ALL values now reference CSS custom properties from style/index.css.
        // Hardcoded navy hex values (#031d4b, #06122d, #05183c etc.) have been
        // replaced — they were the root cause of the unauthorized blue bleed.
        "outline":                   "var(--border-subtle)",
        "surface-dim":               "var(--bg-base)",
        "on-secondary-container":    "var(--text-secondary)",
        "on-secondary":              "var(--text-muted)",
        "secondary-dim":             "var(--text-muted)",
        "on-primary-container":      "var(--text-primary)",
        "surface-container-high":    "var(--bg-elevated)",
        "surface-container":         "var(--bg-surface)",
        "on-primary-fixed-variant":  "var(--cobalt)",
        "inverse-surface":           "var(--text-primary)",
        "on-background":             "var(--text-primary)",
        "on-surface-variant":        "var(--text-secondary)",
        "surface":                   "var(--bg-base)",
        "on-error-container":        "var(--red)",
        "secondary-container":       "var(--bg-raised)",
        "on-primary":                "var(--text-primary)",
        "surface-tint":              "var(--cobalt)",
        "surface-bright":            "var(--bg-elevated)",
        "inverse-primary":           "var(--cobalt)",
        "surface-variant":           "var(--bg-surface)",
        "surface-container-lowest":  "var(--bg-base)",
        "on-surface":                "var(--text-primary)",
        "surface-container-highest": "var(--bg-elevated)",
        "outline-variant":           "var(--border-default)",
        "surface-container-low":     "var(--bg-surface)",
        "primary-container":         "var(--cobalt-dim)",
        "on-error":                  "var(--red)",
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
