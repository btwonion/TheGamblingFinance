<script setup lang="ts">
import { computed } from 'vue';
import { useAuthStore } from '@/stores/auth';

const auth = useAuthStore();

interface NavItem {
  to: string;
  label: string;
  match: (path: string) => boolean;
}

const items = computed<NavItem[]>(() => {
  const base: NavItem[] = [
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
  <aside
    class="hidden md:flex md:flex-col w-56 shrink-0 bg-surface border-r border-border py-4"
    aria-label="Hauptnavigation"
  >
    <ul class="flex flex-col gap-1 px-2">
      <li v-for="item in items" :key="item.to">
        <RouterLink
          :to="item.to"
          v-slot="{ navigate, href }"
          custom
        >
          <a
            :href="href"
            :aria-current="item.match($route.path) ? 'page' : undefined"
            :class="[
              'flex items-center h-11 px-3 rounded-md text-sm font-medium ' +
                'focus:outline-none focus-visible:outline focus-visible:outline-2 focus-visible:outline-focus',
              item.match($route.path)
                ? 'bg-surface-2 text-primary'
                : 'text-text-muted hover:bg-surface-2 hover:text-text',
            ]"
            @click="navigate"
          >
            {{ item.label }}
          </a>
        </RouterLink>
      </li>
    </ul>
  </aside>
</template>
