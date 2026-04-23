<script setup lang="ts">
import { ref, onMounted, onBeforeUnmount } from 'vue';
import { useAuthStore } from '@/stores/auth';
import { useRouter } from 'vue-router';
import PlayerChip from './PlayerChip.vue';

const auth = useAuthStore();
const router = useRouter();

const menuOpen = ref(false);
const menuRef = ref<HTMLElement | null>(null);

function toggleMenu() {
  menuOpen.value = !menuOpen.value;
}

function goProfile() {
  menuOpen.value = false;
  router.push('/profile');
}

async function doLogout() {
  menuOpen.value = false;
  await auth.logout();
}

function onDocClick(ev: MouseEvent) {
  if (!menuOpen.value) return;
  if (menuRef.value && !menuRef.value.contains(ev.target as Node)) {
    menuOpen.value = false;
  }
}

onMounted(() => document.addEventListener('mousedown', onDocClick));
onBeforeUnmount(() => document.removeEventListener('mousedown', onDocClick));
</script>

<template>
  <header class="sticky top-0 z-30 bg-surface border-b border-border">
    <div class="flex items-center justify-between h-14 px-4 md:px-6">
      <RouterLink
        to="/"
        class="text-base md:text-lg font-bold text-text hover:text-primary focus-visible:outline focus-visible:outline-2 focus-visible:outline-focus rounded-sm"
      >
        Gambling Finance
      </RouterLink>

      <div v-if="auth.isAuthed" ref="menuRef" class="relative hidden md:block">
        <button
          type="button"
          class="flex items-center gap-2 h-11 min-w-11 px-2 rounded-md hover:bg-surface-2 focus:outline-none focus-visible:outline focus-visible:outline-2 focus-visible:outline-focus"
          :aria-expanded="menuOpen"
          aria-haspopup="menu"
          @click="toggleMenu"
        >
          <PlayerChip :name="auth.user?.display_name || 'User'" size="sm" />
        </button>
        <div
          v-if="menuOpen"
          role="menu"
          class="absolute right-0 top-full mt-2 min-w-[10rem] bg-surface border border-border rounded-md shadow-2 py-1"
        >
          <button
            type="button"
            role="menuitem"
            class="w-full text-left px-4 py-2 text-sm text-text hover:bg-surface-2 focus:outline-none focus-visible:bg-surface-2"
            @click="goProfile"
          >
            Profil
          </button>
          <button
            type="button"
            role="menuitem"
            class="w-full text-left px-4 py-2 text-sm text-negative hover:bg-surface-2 focus:outline-none focus-visible:bg-surface-2"
            @click="doLogout"
          >
            Abmelden
          </button>
        </div>
      </div>
    </div>
  </header>
</template>
