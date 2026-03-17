/** @type {import('tailwindcss').Config} */
module.exports = {
  content: [
    "./src/**/*.rs",
    "./index.html"
  ],
  theme: {
    extend: {
      colors: {
        background: '#0f172a',
        foreground: '#f8fafc',
        card: {
          DEFAULT: '#1e293b',
          foreground: '#f8fafc',
        },
        popover: {
          DEFAULT: '#1e293b',
          foreground: '#f8fafc',
        },
        primary: {
          DEFAULT: '#38bdf8',
          foreground: '#0f172a',
        },
        secondary: {
          DEFAULT: '#334155',
          foreground: '#f8fafc',
        },
        muted: {
          DEFAULT: '#334155',
          foreground: '#94a3b8',
        },
        accent: {
          DEFAULT: '#0ea5e9',
          foreground: '#f8fafc',
        },
        destructive: {
          DEFAULT: '#ef4444',
          foreground: '#f8fafc',
        },
        border: '#334155',
        input: '#334155',
        ring: '#38bdf8',
      },
      borderRadius: {
        lg: '0.5rem',
        md: 'calc(0.5rem - 2px)',
        sm: 'calc(0.5rem - 4px)',
      }
    },
  },
  plugins: [],
}
