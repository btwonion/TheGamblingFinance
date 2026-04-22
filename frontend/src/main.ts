import { createApp } from 'vue';
import { createPinia } from 'pinia';

import App from './App.vue';
import { router } from './router';
import { setupClient } from './api/client';
import './styles/index.css';

// MSW is only enabled when `VITE_USE_MOCKS === 'true'`. Default off —
// production hits the real Rust API behind Vite's proxy / nginx.
async function enableMocks(): Promise<void> {
  if (import.meta.env.VITE_USE_MOCKS === 'true') {
    const { worker } = await import('./mocks/browser');
    await worker.start({ onUnhandledRequest: 'bypass' });
  }
}

async function bootstrap() {
  await enableMocks();

  const app = createApp(App);
  const pinia = createPinia();
  app.use(pinia);
  app.use(router);

  // Wire the axios client to Pinia + router now that both exist. The
  // 401 response interceptor uses these to clear the session and
  // redirect — doing it here avoids a circular import at module top.
  setupClient(pinia, router);

  app.mount('#app');
}

bootstrap();
