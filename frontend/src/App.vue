<script setup lang="ts">
import { computed } from 'vue';
import { useRoute } from 'vue-router';
import AppShell from '@/components/AppShell.vue';
import ToastHost from '@/components/ToastHost.vue';
import ConfirmHost from '@/components/ConfirmHost.vue';

// `/login`, `/_ui`, and the 404 page render standalone (no top/bottom nav);
// everything else lives inside the shell.
const route = useRoute();
const showShell = computed(
  () => route.name !== 'login' && route.name !== 'not-found' && route.name !== 'ui-demo',
);
</script>

<template>
  <AppShell v-if="showShell">
    <RouterView />
  </AppShell>
  <RouterView v-else />
  <ToastHost />
  <ConfirmHost />
</template>
