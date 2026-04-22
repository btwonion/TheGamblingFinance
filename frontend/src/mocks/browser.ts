import { setupWorker } from 'msw/browser';
import { handlers } from './handlers';

// Dev-only fixture worker. Toggle on with `VITE_USE_MOCKS=true` when
// running `npm run dev` so Frontend-Shell can work on the login flow
// without the Rust backend. `public/mockServiceWorker.js` is published
// by `npx msw init public/ --save`.
export const worker = setupWorker(...handlers);
