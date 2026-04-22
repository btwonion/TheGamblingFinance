import { describe, it, expect, beforeEach, vi } from 'vitest';
import { setActivePinia, createPinia } from 'pinia';

import { ApiError } from '@/api/client';
import type { components } from '@/types/api';

type User = components['schemas']['User'];

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

// Stub the auth API. Each test adjusts behaviour via `mockImplementation`.
vi.mock('@/api/auth', () => {
  return {
    login: vi.fn(),
    logout: vi.fn(),
    fetchMe: vi.fn(),
  };
});

// Stub the router so `logout()` can push '/login' without a full vue-router
// instance.
vi.mock('@/router', () => ({
  router: { push: vi.fn() },
}));

import { useAuthStore } from '../auth';
import * as authApi from '@/api/auth';
import { router } from '@/router';

describe('auth store', () => {
  beforeEach(() => {
    setActivePinia(createPinia());
    vi.clearAllMocks();
  });

  it('login success populates user + flips isAdmin', async () => {
    (authApi.login as unknown as ReturnType<typeof vi.fn>).mockResolvedValue(ADMIN_USER);
    const auth = useAuthStore();

    const ok = await auth.login('admin@example.com', 'admin123');

    expect(ok).toBe(true);
    expect(auth.user).toEqual(ADMIN_USER);
    expect(auth.isAuthed).toBe(true);
    expect(auth.isAdmin).toBe(true);
    expect(auth.error).toBeNull();
    expect(auth.bootstrapped).toBe(true);
  });

  it('player login sets isAdmin=false', async () => {
    (authApi.login as unknown as ReturnType<typeof vi.fn>).mockResolvedValue(PLAYER_USER);
    const auth = useAuthStore();

    await auth.login('player@example.com', 'player123');

    expect(auth.isAuthed).toBe(true);
    expect(auth.isAdmin).toBe(false);
  });

  it('login 401 sets error to the identical "invalid credentials" message', async () => {
    (authApi.login as unknown as ReturnType<typeof vi.fn>).mockRejectedValue(
      new ApiError({ code: 'unauthorized', message: 'invalid credentials' }, 401),
    );
    const auth = useAuthStore();

    const ok = await auth.login('admin@example.com', 'wrong');

    expect(ok).toBe(false);
    expect(auth.user).toBeNull();
    expect(auth.isAuthed).toBe(false);
    expect(auth.error).toBe('invalid credentials');
  });

  it('logout clears user and redirects to /login', async () => {
    (authApi.login as unknown as ReturnType<typeof vi.fn>).mockResolvedValue(ADMIN_USER);
    (authApi.logout as unknown as ReturnType<typeof vi.fn>).mockResolvedValue(undefined);
    const auth = useAuthStore();
    await auth.login('admin@example.com', 'admin123');
    expect(auth.isAuthed).toBe(true);

    await auth.logout();

    expect(auth.user).toBeNull();
    expect(router.push).toHaveBeenCalledWith('/login');
  });

  it('fetchMe success hydrates user and marks bootstrapped', async () => {
    (authApi.fetchMe as unknown as ReturnType<typeof vi.fn>).mockResolvedValue(ADMIN_USER);
    const auth = useAuthStore();
    expect(auth.bootstrapped).toBe(false);

    await auth.fetchMe();

    expect(auth.user).toEqual(ADMIN_USER);
    expect(auth.bootstrapped).toBe(true);
  });

  it('fetchMe 401 leaves user null but still bootstraps', async () => {
    (authApi.fetchMe as unknown as ReturnType<typeof vi.fn>).mockRejectedValue(
      new ApiError({ code: 'unauthorized', message: 'invalid credentials' }, 401),
    );
    const auth = useAuthStore();

    await auth.fetchMe();

    expect(auth.user).toBeNull();
    expect(auth.bootstrapped).toBe(true);
  });

  it('clearSession is synchronous and just nulls the user', () => {
    const auth = useAuthStore();
    auth.$patch({ user: ADMIN_USER });
    auth.clearSession();
    expect(auth.user).toBeNull();
  });
});
