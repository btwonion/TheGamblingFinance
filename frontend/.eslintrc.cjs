/* eslint-env node */
// Frontend-Shell Phase 1: block raw hex colors outside `src/styles/**`.
// Tokens live in `src/styles/tokens.css`; Tailwind reads them via
// `var(--...)` so app code should never reach for a hex literal.
module.exports = {
  root: true,
  env: {
    browser: true,
    es2022: true,
    node: true,
  },
  parser: 'vue-eslint-parser',
  parserOptions: {
    parser: '@typescript-eslint/parser',
    sourceType: 'module',
    ecmaVersion: 2022,
    extraFileExtensions: ['.vue'],
  },
  extends: [
    'eslint:recommended',
    'plugin:vue/vue3-recommended',
    'plugin:@typescript-eslint/recommended',
  ],
  rules: {
    // Design-system-y component names like `Stat`, `Card`, `Button` are
    // intentional; vue's multi-word rule would fight us for nothing.
    'vue/multi-word-component-names': 'off',
    'no-restricted-syntax': [
      'error',
      {
        selector: "Literal[value=/^#[0-9a-fA-F]{3,8}$/]",
        message:
          'No raw hex colors outside src/styles/. Use Tailwind classes or CSS variables.',
      },
    ],
  },
  overrides: [
    { files: ['src/styles/**'], rules: { 'no-restricted-syntax': 'off' } },
    {
      files: ['src/types/api.ts'],
      rules: { '@typescript-eslint/no-explicit-any': 'off' },
    },
    {
      files: ['**/*.spec.ts', '**/__tests__/**'],
      rules: { 'no-unused-expressions': 'off' },
    },
  ],
  ignorePatterns: [
    'dist/',
    'node_modules/',
    'public/mockServiceWorker.js',
    'src/types/api.ts',
  ],
};
