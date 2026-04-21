import axios, { type AxiosInstance } from 'axios';

/**
 * Shared axios instance for every API call.
 *
 * - `baseURL` resolves to `VITE_API_BASE` (set at build time) or
 *   falls back to `/api` so dev with Vite's proxy works.
 * - `withCredentials: true` is mandatory because auth uses an
 *   HttpOnly cookie (`gf_sid`); the browser will not send it
 *   cross-origin without this flag.
 *
 * Frontend-Shell adds interceptors (401 → redirect to /login,
 * error envelope unwrapping, toast wiring) in Phase 1.
 */
export const client: AxiosInstance = axios.create({
  baseURL: import.meta.env.VITE_API_BASE || '/api',
  withCredentials: true,
  headers: {
    'Content-Type': 'application/json',
  },
});
