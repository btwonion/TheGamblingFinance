import { createRouter, createWebHistory, type RouteRecordRaw } from 'vue-router';
import { useAuthStore } from '@/stores/auth';

// Lazy-load views so the login page doesn't carry the whole app bundle.
// Placeholder stubs under `views/_placeholders/` are intentionally
// simple — Frontend-Features will replace each with a real view in
// Phase 2. Keeping them here lets the router + guards compile and be
// tested today.
const routes: RouteRecordRaw[] = [
  {
    path: '/login',
    name: 'login',
    component: () => import('@/views/LoginView.vue'),
    meta: { public: true },
  },
  {
    path: '/',
    name: 'home',
    component: () => import('@/views/_placeholders/DashboardStub.vue'),
  },
  {
    path: '/nights',
    name: 'nights',
    component: () => import('@/views/_placeholders/NightsListStub.vue'),
  },
  {
    path: '/nights/:id',
    name: 'night-detail',
    component: () => import('@/views/_placeholders/NightDetailStub.vue'),
  },
  {
    path: '/nights/:id/settle',
    name: 'night-settle',
    component: () => import('@/views/_placeholders/NightSettleStub.vue'),
  },
  {
    path: '/leaderboard',
    name: 'leaderboard',
    component: () => import('@/views/_placeholders/LeaderboardStub.vue'),
  },
  {
    path: '/profile',
    name: 'profile',
    component: () => import('@/views/_placeholders/ProfileStub.vue'),
  },
  {
    path: '/admin/players',
    name: 'admin-players',
    component: () => import('@/views/_placeholders/AdminPlayersStub.vue'),
    meta: { admin: true },
  },
  {
    path: '/_ui',
    name: 'ui-demo',
    component: () => import('@/views/UiDemoView.vue'),
    meta: { public: true },
  },
  {
    path: '/:pathMatch(.*)*',
    name: 'not-found',
    component: () => import('@/views/_placeholders/NotFoundStub.vue'),
    meta: { public: true },
  },
];

export const router = createRouter({
  history: createWebHistory(),
  routes,
});

router.beforeEach(async (to) => {
  const auth = useAuthStore();

  // Lazily run `fetchMe` once per app lifetime so guards can read
  // `auth.isAuthed` reliably on direct-URL loads / page refreshes.
  if (!auth.bootstrapped) {
    await auth.fetchMe();
  }

  // Admin gate.
  if (to.meta.admin && !auth.isAdmin) {
    return { path: '/' };
  }

  // Public routes can always render.
  if (to.meta.public) {
    // Don't let logged-in users see the login screen again.
    if (to.name === 'login' && auth.isAuthed) {
      return { path: '/' };
    }
    return true;
  }

  // Everything else requires auth.
  if (!auth.isAuthed) {
    return {
      path: '/login',
      query: { next: to.fullPath },
    };
  }

  return true;
});
