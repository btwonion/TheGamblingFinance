import axios, {
  type AxiosInstance,
  type AxiosError,
  type InternalAxiosRequestConfig,
} from 'axios';
import type { Router } from 'vue-router';
import type { Pinia } from 'pinia';

/**
 * Shape of the error envelope the backend promises in `plan.md`:
 *   `{ error: { code, message, details? } }`
 *
 * We unwrap it into an `ApiError` so views/stores can distinguish business
 * errors from network / axios internal errors without string-sniffing.
 */
export interface ApiErrorPayload {
  code: string;
  message: string;
  details?: Record<string, unknown>;
}

export class ApiError extends Error {
  readonly code: string;
  readonly details?: Record<string, unknown>;
  readonly status?: number;

  constructor(payload: ApiErrorPayload, status?: number) {
    super(payload.message);
    this.name = 'ApiError';
    this.code = payload.code;
    this.details = payload.details;
    this.status = status;
  }
}

/**
 * Shared axios instance for every API call.
 *
 * - `baseURL` resolves to `VITE_API_BASE` (set at build time) or
 *   falls back to `/api` so dev with Vite's proxy works.
 * - `withCredentials: true` is mandatory because auth uses an
 *   HttpOnly cookie (`gf_sid`); the browser will not send it
 *   cross-origin without this flag.
 */
export const client: AxiosInstance = axios.create({
  baseURL: import.meta.env.VITE_API_BASE || '/api',
  withCredentials: true,
  headers: {
    'Content-Type': 'application/json',
  },
});

// Deferred refs to the store + router. We don't import them at module
// top-level because `api/client` is imported by `api/auth`, which is
// imported by `stores/auth`, which is imported by `router` — a real
// circular. `setupClient()` is called from `main.ts` after Pinia and
// the router exist.
let piniaRef: Pinia | null = null;
let routerRef: Router | null = null;

export function setupClient(pinia: Pinia, router: Router): void {
  piniaRef = pinia;
  routerRef = router;
}

function isEnvelope(data: unknown): data is { error: ApiErrorPayload } {
  return (
    typeof data === 'object' &&
    data !== null &&
    'error' in data &&
    typeof (data as { error: unknown }).error === 'object' &&
    (data as { error: unknown }).error !== null &&
    typeof (data as { error: { code?: unknown } }).error.code === 'string' &&
    typeof (data as { error: { message?: unknown } }).error.message === 'string'
  );
}

client.interceptors.request.use((config: InternalAxiosRequestConfig) => {
  // Nothing to do today; hook point for future tracing / CSRF headers.
  return config;
});

client.interceptors.response.use(
  (resp) => resp,
  async (error: AxiosError) => {
    const status = error.response?.status;
    const raw = error.response?.data;

    // Lazy import to avoid circular module-init problems.
    if (status === 401 && piniaRef && routerRef) {
      try {
        const { useAuthStore } = await import('@/stores/auth');
        const auth = useAuthStore(piniaRef);
        auth.clearSession();
      } catch {
        // If the store can't be loaded we still want to redirect; the
        // user is effectively logged out.
      }

      const current = routerRef.currentRoute.value;
      if (current.path !== '/login') {
        routerRef.push({
          path: '/login',
          query: { next: current.fullPath },
        });
      }
    }

    if (isEnvelope(raw)) {
      throw new ApiError(raw.error, status);
    }

    // Fallback so callers always get something with `.message`.
    throw new ApiError(
      {
        code: 'network_error',
        message: error.message || 'Network error',
      },
      status,
    );
  },
);
