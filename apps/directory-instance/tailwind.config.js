/** @type {import('tailwindcss').Config} */
module.exports = {
  content: [
    "./src/**/*.rs",
    "../shared-ui/src/**/*.rs",
    "./index.html"
  ],
  theme: {
    extend: {
      colors: {
        background: 'var(--bg-surface)',
        foreground: 'var(--foreground)',
        card: {
          DEFAULT: 'var(--bg-surface)',
          foreground: 'var(--foreground)',
        },
        popover: {
          DEFAULT: 'var(--bg-surface)',
          foreground: 'var(--foreground)',
        },
        primary: {
          DEFAULT: 'var(--brand-primary)',
          foreground: 'var(--primary-foreground)',
        },
        secondary: {
          DEFAULT: '#f1f5f9',
          foreground: '#0f172a',
        },
        muted: {
          DEFAULT: '#f8fafc',
          foreground: '#64748b',
        },
        accent: {
          DEFAULT: '#f1f5f9',
          foreground: '#0f172a',
        },
        destructive: {
          DEFAULT: '#ef4444',
          foreground: '#f8fafc',
        },
        border: '#e2e8f0',
        input: '#e2e8f0',
        ring: 'var(--brand-primary)',
        // Premium additions
        surface: {
          DEFAULT: '#fafaf9',
          elevated: '#ffffff',
          overlay: 'rgba(255,255,255,0.8)',
        },
        neutral: {
          50: '#fafaf9',
          100: '#f5f5f4',
          200: '#e7e5e4',
          300: '#d6d3d1',
          400: '#a8a29e',
          500: '#78716c',
          600: '#57534e',
          700: '#44403c',
          800: '#292524',
          900: '#1c1917',
          950: '#0c0a09',
        },
      },
      borderRadius: {
        lg: '12px',
        md: '10px',
        sm: '6px',
        xl: '16px',
        '2xl': '20px',
        '3xl': '24px',
      },
      fontFamily: {
        sans: ['Inter', 'system-ui', '-apple-system', 'sans-serif'],
        display: ['DM Sans', 'Inter', 'system-ui', 'sans-serif'],
        mono: ['JetBrains Mono', 'Fira Code', 'monospace'],
      },
      boxShadow: {
        'premium': '0 1px 2px rgba(0,0,0,0.04), 0 4px 12px rgba(0,0,0,0.04), 0 16px 40px rgba(0,0,0,0.04)',
        'premium-lg': '0 4px 6px rgba(0,0,0,0.02), 0 12px 24px rgba(0,0,0,0.06), 0 24px 60px rgba(0,0,0,0.06)',
        'premium-xl': '0 8px 16px rgba(0,0,0,0.04), 0 24px 48px rgba(0,0,0,0.08), 0 48px 96px rgba(0,0,0,0.06)',
        'glow': '0 0 24px rgba(var(--brand-primary-rgb, 37,99,235), 0.15)',
        'glow-lg': '0 0 48px rgba(var(--brand-primary-rgb, 37,99,235), 0.2)',
        'inner-soft': 'inset 0 1px 4px rgba(0,0,0,0.04)',
        'card-hover': '0 8px 30px rgba(0,0,0,0.08), 0 2px 8px rgba(0,0,0,0.04)',
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
        'gradient-x': {
          '0%, 100%': { backgroundPosition: '0% 50%' },
          '50%': { backgroundPosition: '100% 50%' },
        },
        'float': {
          '0%, 100%': { transform: 'translateY(0)' },
          '50%': { transform: 'translateY(-6px)' },
        },
      },
      animation: {
        'slide-up': 'slide-up 0.5s cubic-bezier(0.16, 1, 0.3, 1)',
        'fade-scale': 'fade-scale 0.4s cubic-bezier(0.16, 1, 0.3, 1)',
        'shimmer': 'shimmer 2s linear infinite',
        'gradient-x': 'gradient-x 6s ease infinite',
        'float': 'float 3s ease-in-out infinite',
      },
    },
  },
  plugins: [],
}
