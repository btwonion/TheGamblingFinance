/** @type {import('tailwindcss').Config} */
// Tailwind reads our design tokens via CSS custom properties declared
// in `src/styles/tokens.css`. This keeps a single source of truth —
// changing a token there immediately updates every utility class.
// The ESLint `no-restricted-syntax` rule (wired by Frontend-Shell in
// Phase 1) bans raw hex literals in `.vue`/`.ts` to prevent drift.
export default {
  content: ['./index.html', './src/**/*.{vue,ts,tsx}'],
  theme: {
    extend: {
      colors: {
        bg: 'var(--bg)',
        surface: 'var(--surface)',
        'surface-2': 'var(--surface-2)',
        border: 'var(--border)',
        'border-strong': 'var(--border-strong)',

        text: 'var(--text)',
        'text-muted': 'var(--text-muted)',
        'text-dim': 'var(--text-dim)',

        primary: 'var(--primary)',
        'primary-hover': 'var(--primary-hover)',
        'primary-active': 'var(--primary-active)',
        'primary-ink': 'var(--primary-ink)',

        positive: 'var(--positive)',
        'positive-soft': 'var(--positive-soft)',
        negative: 'var(--negative)',
        'negative-soft': 'var(--negative-soft)',
        warning: 'var(--warning)',
        'warning-soft': 'var(--warning-soft)',
        info: 'var(--info)',
        focus: 'var(--focus)',
      },
      borderRadius: {
        sm: 'var(--radius-sm)',
        md: 'var(--radius-md)',
        lg: 'var(--radius-lg)',
        xl: 'var(--radius-xl)',
      },
      boxShadow: {
        1: 'var(--shadow-1)',
        2: 'var(--shadow-2)',
        sheet: 'var(--shadow-sheet)',
      },
      transitionTimingFunction: {
        out: 'var(--ease-out)',
      },
      transitionDuration: {
        fast: 'var(--dur-fast)',
        med: 'var(--dur-med)',
      },
    },
  },
  plugins: [],
};
