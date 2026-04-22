import { client } from './client';
import type { components } from '@/types/api';

export type User = components['schemas']['User'];

/**
 * Exchange credentials for a session cookie. Backend sets `gf_sid`
 * (HttpOnly, Secure, SameSite=Lax) and returns the `User`.
 *
 * On failure: see plan.md §"Security" — the backend returns an
 * identical `invalid credentials` message regardless of whether the
 * email or the password was wrong, to avoid user enumeration.
 */
export async function login(email: string, password: string): Promise<User> {
  const { data } = await client.post<User>('/auth/login', { email, password });
  return data;
}

export async function logout(): Promise<void> {
  await client.post('/auth/logout');
}

export async function fetchMe(): Promise<User> {
  const { data } = await client.get<User>('/auth/me');
  return data;
}
