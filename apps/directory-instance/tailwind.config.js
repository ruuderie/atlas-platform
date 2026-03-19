/** @type {import('tailwindcss').Config} */
module.exports = {
  content: [
    "./src/**/*.rs",
    "../shared-ui/src/**/*.rs",
    "./index.html"
  ],
  darkMode: "class",
  theme: {
    extend: {
      colors: {
        background: 'var(--bg-surface, #f8f9fa)',
        foreground: 'var(--foreground, #191c1d)',
        card: {
          DEFAULT: 'var(--bg-surface, #ffffff)',
          foreground: 'var(--foreground, #191c1d)',
        },
        popover: {
          DEFAULT: 'var(--bg-surface, #ffffff)',
          foreground: 'var(--foreground, #191c1d)',
        },
        // We map the dynamic brand primary to the design system's primary
        primary: {
          DEFAULT: 'var(--brand-primary, #004289)',
          foreground: 'var(--primary-foreground, #ffffff)',
        },
        secondary: {
          DEFAULT: '#4c616c',
          foreground: '#ffffff',
        },
        muted: {
          DEFAULT: '#f3f4f5',
          foreground: '#526772',
        },
        accent: {
          DEFAULT: '#cfe6f2',
          foreground: '#071e27',
        },
        destructive: {
          DEFAULT: '#ba1a1a',
          foreground: '#ffffff',
        },
        border: '#c3c6d6',
        input: '#e1e3e4',
        ring: 'var(--brand-primary, #004289)',
        
        // Architectural Curator specific standard colors
        "inverse-on-surface": "#f0f1f2",
        "outline-variant": "#c3c6d6",
        "surface-container-highest": "#e1e3e4",
        "tertiary": "#7b2600",
        "primary-fixed-dim": "#abc7ff",
        "secondary-fixed": "#cfe6f2",
        "on-background": "#191c1d",
        "on-error-container": "#93000a",
        "on-primary": "#ffffff",
        "surface-tint": "#255dad",
        "surface-dim": "#d9dadb",
        "surface": "#f8f9fa",
        "on-secondary-fixed-variant": "#354a53",
        "primary-fixed": "#d7e2ff",
        "tertiary-container": "#a33500",
        "on-tertiary-container": "#ffc6b2",
        "secondary-fixed-dim": "#b4cad6",
        "inverse-primary": "#abc7ff",
        "tertiary-fixed-dim": "#ffb59b",
        "on-tertiary-fixed-variant": "#812800",
        "on-secondary": "#ffffff",
        "on-primary-fixed": "#001b3f",
        "error": "#ba1a1a",
        "on-primary-fixed-variant": "#00458f",
        "surface-container-lowest": "#ffffff",
        "on-surface": "#191c1d",
        "inverse-surface": "#2e3132",
        "surface-container-high": "#e7e8e9",
        "surface-variant": "#e1e3e4",
        "primary-container": "#2059a9",
        "secondary-container": "#cfe6f2",
        "on-surface-variant": "#434654",
        "tertiary-fixed": "#ffdbcf",
        "error-container": "#ffdad6",
        "on-tertiary-fixed": "#380d00",
        "on-secondary-container": "#526772",
        "outline": "#737685",
        "on-primary-container": "#bfd3ff",
        "on-secondary-fixed": "#071e27",
        "on-tertiary": "#ffffff",
        "surface-bright": "#f8f9fa",
        "on-error": "#ffffff",
        "surface-container-low": "#f3f4f5",
        "surface-container": "#edeeef"
      },
      borderRadius: {
        'DEFAULT': '0.125rem',
        'sm': '0.125rem',
        'md': '0.25rem',
        'lg': '0.25rem', 
        'xl': '0.5rem', 
        '2xl': '0.75rem',
        '3xl': '1rem',
        'full': '0.75rem'
      },
      fontFamily: {
        headline: ["Manrope", "sans-serif"],
        body: ["Inter", "sans-serif"],
        label: ["Inter", "sans-serif"],
        sans: ["Inter", "system-ui", "-apple-system", "sans-serif"],
        display: ["Manrope", "system-ui", "sans-serif"],
      },
      boxShadow: {
        'premium': '0 10px 30px rgba(25, 28, 29, 0.04), 0 4px 8px rgba(25, 28, 29, 0.02)',
        'premium-lg': '0 20px 40px rgba(25, 28, 29, 0.06), 0 8px 16px rgba(25, 28, 29, 0.04)',
        'premium-xl': '0 30px 60px rgba(25, 28, 29, 0.08), 0 12px 24px rgba(25, 28, 29, 0.06)',
        'glow': '0 0 24px rgba(0, 66, 137, 0.15)',
        'glow-lg': '0 0 48px rgba(0, 66, 137, 0.2)',
        'inner-soft': 'inset 0 1px 0 0 rgba(255, 255, 255, 0.1)',
        'card-hover': '0 12px 30px rgba(25, 28, 29, 0.08), 0 4px 10px rgba(25, 28, 29, 0.04)',
        'ambient': '0 10px 30px rgba(25, 28, 29, 0.04), 0 4px 8px rgba(25, 28, 29, 0.02)',
      },
      keyframes: {
        'slide-up': {
          '0%': { transform: 'translateY(16px)', opacity: '0' },
          '100%': { transform: 'translateY(0)', opacity: '1' },
        },
        'fade-scale': {
          '0%': { transform: 'scale(0.96)', opacity: '0' },
          '100%': { transform: 'scale(1)', opacity: '1' },
        },
        'shimmer': {
          '0%': { backgroundPosition: '-200% 0' },
          '100%': { backgroundPosition: '200% 0' },
        },
      },
      animation: {
        'slide-up': 'slide-up 0.5s cubic-bezier(0.16, 1, 0.3, 1)',
        'fade-scale': 'fade-scale 0.4s cubic-bezier(0.16, 1, 0.3, 1)',
        'shimmer': 'shimmer 2s linear infinite',
      },
    },
  },
  plugins: [],
}

