/// <reference types="vitest" />
import { fileURLToPath, URL } from 'node:url';
import { defineConfig } from 'vite';
import vue from '@vitejs/plugin-vue';

// Vite config.
// - `@` alias → `src/` (matches tsconfig paths).
// - Dev proxy: everything under `/api` is forwarded to the Rust backend
//   on :8080 so the frontend can use `VITE_API_BASE = ''` and relative
//   URLs in dev. Production deployments put nginx in front and don't
//   need this.
// - Vitest `test` block uses happy-dom so Pinia / composable tests can
//   exercise DOM-backed APIs without a full browser.
export default defineConfig({
  plugins: [vue()],
  resolve: {
    alias: {
      '@': fileURLToPath(new URL('./src', import.meta.url)),
    },
  },
  server: {
    port: 5173,
    strictPort: true,
    proxy: {
      '/api': {
        target: 'http://localhost:8080',
        changeOrigin: false,
        secure: false,
      },
    },
  },
  build: {
    outDir: 'dist',
    sourcemap: true,
  },
  test: {
    environment: 'happy-dom',
    globals: true,
    include: ['src/**/__tests__/**/*.spec.ts', 'src/**/*.spec.ts'],
  },
});
