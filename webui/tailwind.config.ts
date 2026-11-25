import type { Config } from 'tailwindcss';

export default {
  content: ['./index.html', './src/**/*.{js,ts,jsx,tsx}'],
  darkMode: 'class',
  theme: {
    extend: {
      colors: {
        // Aurora Design System — references CSS variables for theme switching
        'warm-black': 'var(--color-warm-black)',
        'golden-cream': 'var(--color-golden-cream)',
        'warm-ivory': 'var(--color-warm-ivory)',
        'amber-gold': 'var(--color-amber-gold)',
        'deep-olive': 'var(--color-deep-olive)',
        'warm-khaki': 'var(--color-warm-khaki)',
        'golden-sand': 'var(--color-golden-sand)',
        'pale-sand': 'var(--color-pale-sand)',
        'warm-taupe': 'var(--color-warm-taupe)',
        // Extended semantic tokens
        'bg-inset': 'var(--color-bg-inset)',
        'bg-active': 'var(--color-bg-active)',
        'border-subtle': 'var(--color-border-subtle)',
        'text-inverse': 'var(--color-text-inverse)',
        'accent-hover': 'var(--color-accent-hover)',
        'accent-subtle': 'var(--color-accent-subtle)',
        'accent-text': 'var(--color-accent-text)',
        // Status colors
        'vt-error': 'var(--color-error)',
        'vt-success': 'var(--color-success)',
        'vt-warning': 'var(--color-warning)',
        'vt-info': 'var(--color-info)',
      },
      fontFamily: {
        sans: ['Inter', 'system-ui', '-apple-system', 'sans-serif'],
        display: ['Inter', 'system-ui', '-apple-system', 'sans-serif'],
        mono: ['JetBrains Mono', 'Cascadia Code', 'Fira Code', 'Consolas', 'monospace'],
      },
      fontWeight: {
        light: '300',
        normal: '400',
        medium: '500',
        semibold: '600',
      },
      fontSize: {
        '2xs': '11px',
        xs: '12px',
        sm: '13px',
        base: '14px',
      },
      boxShadow: {
        // Subtle elevation
        'shadow-sm': '0 1px 2px rgba(50,78,122,.06)',
        'shadow-md': '0 4px 12px rgba(50,78,122,.10)',
        'shadow-lg': '0 8px 24px rgba(50,78,122,.14)',
        // Tab underlines (inset box-shadow technique)
        'tab-active': 'var(--color-accent) 0px -4px 0px 0px inset',
        'tab-hover': 'var(--color-golden-sand) 0px -4px 0px 0px inset',
        // Focus ring
        'focus-ring': '0 0 0 2px var(--color-accent)',
      },
      borderRadius: {
        tight: '4px',
        standard: '6px',
        content: '6px',
        comfortable: '8px',
        social: '12px',
        pill: '9999px',
      },
      letterSpacing: {
        'vt-tight': '-0.48px',
        'vt-tighter': '-1px',
        'vt-normal': 'normal',
        'vt-wide': '0.5px',
      },
      lineHeight: {
        'vt-display': '0.90',
        'vt-heading': '1.04',
        'vt-sub': '1.25',
        'vt-body': '1.50',
      },
      transitionTimingFunction: {
        'vt-ease': 'ease',
      },
      transitionDuration: {
        'vt-fast': '100ms',
        'vt-base': '180ms',
      },
    },
  },
  plugins: [],
} satisfies Config;
