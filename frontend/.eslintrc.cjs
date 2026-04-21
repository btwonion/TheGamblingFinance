/* eslint-env node */
// Phase 0 baseline. Frontend-Shell refines in Phase 1 — in particular,
// wiring up a `no-restricted-syntax` rule that bans hex color literals
// outside `src/styles/**`. The skeleton below is deliberately
// conservative so Phase 0 builds don't break on stub code.
module.exports = {
  root: true,
  env: {
    browser: true,
    es2022: true,
    node: true,
  },
  extends: [
    'eslint:recommended',
    'plugin:@typescript-eslint/recommended',
    'plugin:vue/vue3-recommended',
  ],
  parser: 'vue-eslint-parser',
  parserOptions: {
    parser: '@typescript-eslint/parser',
    ecmaVersion: 2022,
    sourceType: 'module',
    extraFileExtensions: ['.vue'],
  },
  rules: {
    // Will be narrowed by Frontend-Shell in Phase 1; see tokens.css.
    // 'no-restricted-syntax': [
    //   'error',
    //   {
    //     selector: "Literal[value=/^#[0-9a-fA-F]{3,8}$/]",
    //     message: 'Use CSS variables from tokens.css instead of hex literals.',
    //   },
    // ],
  },
  overrides: [
    {
      files: ['src/types/api.ts'],
      rules: {
        '@typescript-eslint/no-explicit-any': 'off',
      },
    },
  ],
  ignorePatterns: ['dist/', 'node_modules/', 'src/types/api.ts'],
};
