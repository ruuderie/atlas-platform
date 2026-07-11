/** @type {import('tailwindcss').Config} */
//
// IMPORTANT: Color values here MUST reference CSS custom properties (--color-*)
// defined in style/index.css, NOT hardcoded hex values. This ensures shared-ui
// components automatically inherit the correct palette for this app.
//
// Opacity modifiers (bg-primary/15, text-on-surface-variant/40, etc.) use the
// `<alpha-value>` placeholder so DEFAULT utilities stay solid and `/N` variants
// mix correctly. Do NOT use a JS function that Number()-multiplies opacityValue —
// Tailwind often passes `var(--tw-*-opacity)` for DEFAULT, which becomes NaN%.
//
// See apps/shared-ui/THEMING.md and designs/stitch/.../shared/design-system.css.
//

/** @param {string} variable CSS custom property name including leading -- */
function withAlpha(variable) {
  return `color-mix(in srgb, var(${variable}) calc(<alpha-value> * 100%), transparent)`;
}

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
          DEFAULT:    withAlpha("--color-primary"),
          foreground: withAlpha("--color-primary-foreground"),
        },
        "card": {
          DEFAULT:    withAlpha("--color-card"),
          foreground: withAlpha("--color-card-foreground"),
        },
        "background":  withAlpha("--color-background"),
        "foreground":  withAlpha("--color-foreground"),
        "popover": {
          DEFAULT:    withAlpha("--color-popover"),
          foreground: withAlpha("--color-popover-foreground"),
        },
        "muted": {
          DEFAULT:    withAlpha("--color-muted"),
          foreground: withAlpha("--color-muted-foreground"),
        },
        "accent": {
          DEFAULT:    withAlpha("--color-accent"),
          foreground: withAlpha("--color-accent-foreground"),
        },
        "secondary": {
          DEFAULT:    withAlpha("--color-secondary"),
          foreground: withAlpha("--color-secondary-foreground"),
        },
        "destructive": {
          DEFAULT:    withAlpha("--color-destructive"),
          foreground: withAlpha("--color-destructive-foreground"),
        },
        "success": {
          DEFAULT:    withAlpha("--color-success"),
          foreground: withAlpha("--color-success-foreground"),
        },
        "warning": {
          DEFAULT:    withAlpha("--color-warning"),
          foreground: withAlpha("--color-warning-foreground"),
        },
        "border": withAlpha("--color-border"),
        "input":  withAlpha("--color-input"),
        "ring":   withAlpha("--color-ring"),

        // ── Platform-admin legacy MD3 token bridge ──────────────────────────
        "outline":                   withAlpha("--border-subtle"),
        "surface-dim":               withAlpha("--bg-base"),
        "on-secondary-container":    withAlpha("--text-secondary"),
        "on-secondary":              withAlpha("--text-muted"),
        "secondary-dim":             withAlpha("--text-muted"),
        "on-primary-container":      withAlpha("--text-primary"),
        "surface-container-high":    withAlpha("--bg-elevated"),
        "surface-container":         withAlpha("--bg-surface"),
        "on-primary-fixed-variant":  withAlpha("--cobalt"),
        "inverse-surface":           withAlpha("--text-primary"),
        "on-background":             withAlpha("--text-primary"),
        "on-surface-variant":        withAlpha("--text-secondary"),
        "surface":                   withAlpha("--bg-base"),
        "on-error-container":        withAlpha("--red"),
        "secondary-container":       withAlpha("--bg-surface-alt"),
        "on-primary":                withAlpha("--text-primary"),
        "surface-tint":              withAlpha("--cobalt"),
        "surface-bright":            withAlpha("--bg-elevated"),
        "inverse-primary":           withAlpha("--cobalt"),
        "surface-variant":           withAlpha("--bg-surface"),
        "surface-container-lowest":  withAlpha("--bg-base"),
        "on-surface":                withAlpha("--text-primary"),
        "surface-container-highest": withAlpha("--bg-elevated"),
        "outline-variant":           withAlpha("--border-default"),
        "surface-container-low":     withAlpha("--bg-surface"),
        "primary-container":         withAlpha("--cobalt-dim"),
        "on-error":                  withAlpha("--red"),
        "error":                     withAlpha("--color-destructive"),
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
