<script setup lang="ts">
import { computed } from 'vue';
import { useAuthStore } from '@/stores/auth';

const auth = useAuthStore();

interface Tab {
  to: string;
  label: string;
  match: (path: string) => boolean;
}

const tabs = computed<Tab[]>(() => {
  const base: Tab[] = [
    { to: '/', label: 'Home', match: (p) => p === '/' },
    { to: '/nights', label: 'Nights', match: (p) => p.startsWith('/nights') },
    { to: '/leaderboard', label: 'Rangliste', match: (p) => p.startsWith('/leaderboard') },
    { to: '/profile', label: 'Profil', match: (p) => p.startsWith('/profile') },
  ];
  if (auth.isAdmin) {
    base.push({ to: '/admin/players', label: 'Admin', match: (p) => p.startsWith('/admin') });
  }
  return base;
});
</script>

<template>
  <nav
    class="md:hidden fixed bottom-0 inset-x-0 z-30 bg-surface border-t border-border"
    aria-label="Hauptnavigation"
  >
    <ul class="flex">
      <li v-for="tab in tabs" :key="tab.to" class="flex-1">
        <RouterLink
          :to="tab.to"
          v-slot="{ isActive, navigate, href }"
          custom
        >
          <a
            :href="href"
            :aria-current="tab.match($route.path) ? 'page' : undefined"
            :class="[
              'flex items-center justify-center h-14 min-h-11 text-sm font-medium ' +
                'focus:outline-none focus-visible:outline focus-visible:outline-2 focus-visible:outline-focus',
              tab.match($route.path)
                ? 'text-primary border-t-2 border-primary'
                : 'text-text-muted border-t-2 border-transparent',
              isActive ? '' : '',
            ]"
            @click="navigate"
          >
            {{ tab.label }}
          </a>
        </RouterLink>
      </li>
    </ul>
  </nav>
</template>
