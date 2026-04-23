import { defineStore } from 'pinia';
import { ref, computed } from 'vue';
import { login as apiLogin, logout as apiLogout, fetchMe, type User } from '@/api/auth';
import { ApiError } from '@/api/client';
import { router } from '@/router';

/**
 * `auth` store — the shell's only store. Frontend-Features owns the
 * other ones (`nights`, `players`, `leaderboard`).
 *
 * `bootstrapped` toggles on the first `fetchMe` attempt so the router
 * guard can tell "we haven't checked yet" from "we checked and the user
 * is logged out". Without it the very first navigation would flicker
 * past the guard.
 */
export const useAuthStore = defineStore('auth', () => {
  const user = ref<User | null>(null);
  const loading = ref(false);
  const error = ref<string | null>(null);
  const bootstrapped = ref(false);

  const isAuthed = computed(() => user.value !== null);
  const isAdmin = computed(() => user.value?.role === 'admin');

  async function login(email: string, password: string): Promise<boolean> {
    loading.value = true;
    error.value = null;
    try {
      user.value = await apiLogin(email, password);
      bootstrapped.value = true;
      return true;
    } catch (err) {
      // Identical error message for every failure — matches backend's
      // "no user enumeration" policy.
      error.value = 'invalid credentials';
      user.value = null;
      // Swallow to callers; they key off `error`/`isAuthed`.
      if (!(err instanceof ApiError)) throw err;
      return false;
    } finally {
      loading.value = false;
    }
  }

  async function logout(): Promise<void> {
    try {
      await apiLogout();
    } catch {
      // Even if the server logout fails, locally clear the session —
      // the cookie may already be expired.
    }
    user.value = null;
    router.push('/login');
  }

  async function doFetchMe(): Promise<void> {
    try {
      user.value = await fetchMe();
    } catch (err) {
      if (err instanceof ApiError && err.status === 401) {
        user.value = null;
      } else if (err instanceof ApiError) {
        user.value = null;
      } else {
        throw err;
      }
    } finally {
      bootstrapped.value = true;
    }
  }

  function clearSession(): void {
    user.value = null;
  }

  return {
    user,
    loading,
    error,
    bootstrapped,
    isAuthed,
    isAdmin,
    login,
    logout,
    fetchMe: doFetchMe,
    clearSession,
  };
});
