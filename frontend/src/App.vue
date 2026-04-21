<script setup lang="ts">
// Phase 0 root. Frontend-Shell replaces this with `AppShell` wrapping
// `<RouterView />` in Phase 1. The placeholder below exists only so
// `vite build` produces a non-empty bundle we can smoke-test.
</script>

<template>
  <main class="min-h-screen bg-bg text-text">
    <RouterView />
  </main>
</template>
