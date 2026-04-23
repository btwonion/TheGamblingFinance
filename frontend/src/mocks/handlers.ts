import { http, HttpResponse, delay } from 'msw';
import type { components } from '@/types/api';

type User = components['schemas']['User'];

// Local in-memory session. Because MSW runs in the same origin as the
// page we share module state; no cookies involved in the mock.
interface MockSession {
  user: User | null;
}
const mockSession: MockSession = { user: null };

const ADMIN_USER: User = {
  id: '00000000-0000-7000-8000-000000000001',
  email: 'admin@example.com',
  display_name: 'Admin',
  role: 'admin',
  created_at: '2026-01-01T00:00:00Z',
  disabled_at: null,
};

const PLAYER_USER: User = {
  id: '00000000-0000-7000-8000-000000000002',
  email: 'player@example.com',
  display_name: 'Player',
  role: 'player',
  created_at: '2026-01-01T00:00:00Z',
  disabled_at: null,
};

function unauthorized() {
  return HttpResponse.json(
    { error: { code: 'unauthorized', message: 'invalid credentials' } },
    { status: 401 },
  );
}

// `VITE_API_BASE` (or its default `/api`) is the prefix for every real
// request. We pass it through here so the handlers match regardless of
// how the app was configured.
const base = (import.meta.env.VITE_API_BASE as string | undefined) || '/api';

export const handlers = [
  http.post(`${base}/auth/login`, async ({ request }) => {
    const body = (await request.json()) as { email?: string; password?: string } | null;
    await delay(500);
    if (body?.email === 'admin@example.com' && body.password === 'admin123') {
      mockSession.user = ADMIN_USER;
      return HttpResponse.json(ADMIN_USER, { status: 200 });
    }
    if (body?.email === 'player@example.com' && body.password === 'player123') {
      mockSession.user = PLAYER_USER;
      return HttpResponse.json(PLAYER_USER, { status: 200 });
    }
    return unauthorized();
  }),

  http.post(`${base}/auth/logout`, () => {
    mockSession.user = null;
    return new HttpResponse(null, { status: 204 });
  }),

  http.get(`${base}/auth/me`, () => {
    if (mockSession.user) {
      return HttpResponse.json(mockSession.user, { status: 200 });
    }
    return unauthorized();
  }),
];
