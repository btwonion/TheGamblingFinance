import { createRouter, createWebHistory, type RouteRecordRaw } from 'vue-router';

// Placeholder landing component — just enough for Vite to build.
// Frontend-Shell replaces this with real views (Login, Dashboard, …)
// in Phase 1 and adds auth / admin guards.
const Placeholder = {
  template: `
    <section class="p-6">
      <h1 class="text-2xl font-bold">TheGamblingFinance</h1>
      <p class="text-text-muted mt-2">
        Phase 0 scaffold. Frontend-Shell fills in the real routes in Phase 1.
      </p>
    </section>
  `,
};

const routes: RouteRecordRaw[] = [
  { path: '/', name: 'home', component: Placeholder },
];

export const router = createRouter({
  history: createWebHistory(),
  routes,
});
